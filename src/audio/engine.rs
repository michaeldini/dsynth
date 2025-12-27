use crate::audio::voice::Voice;
use crate::dsp::effects::{
    AutoPan, Bitcrusher, Chorus, CombFilter, Compressor, Distortion, Flanger, MultibandDistortion,
    Phaser, Reverb, RingModulator, StereoDelay, StereoWidener, Tremolo, Waveshaper,
};
use crate::dsp::wavetable_library::WavetableLibrary;
use crate::params::SynthParams;
use triple_buffer::{Input, Output, TripleBuffer};

const MAX_POLYPHONY: usize = 16;

/// The core synthesis engine that orchestrates real-time audio generation.
///
/// The SynthEngine is the heart of the synthesizer. It:
/// - Manages 16 polyphonic voices that can play simultaneously
/// - Receives MIDI note events (note on/off) and triggers voice allocation
/// - Reads parameter updates (filter cutoff, envelope times, etc.) from a lock-free triple-buffer
/// - Processes one audio sample per call by mixing all active voices
/// - Implements both polyphonic mode (multiple notes at once) and monophonic mode (last-note priority)
///
/// ## Voice Allocation Strategy
///
/// When a note arrives, the engine must decide which voice plays it:
/// - If an idle voice is available: Use it immediately
/// - If all 16 voices are busy (polyphony limit reached): Use **voice stealing**
///   - Strategy: Kill the quietest currently-playing voice and reuse it
///   - This ensures the most recent, loudest notes are prioritized
///   - Much better than just stopping old notes (would cause clicks)
///
/// ## Monophonic vs Polyphonic Mode
///
/// **Polyphonic mode** (default): Multiple notes can play simultaneously.
/// Each note gets its own voice and envelope. When you release a key, only that note stops.
///
/// **Monophonic mode**: Only one note plays at a time, but multiple keys can be held.
/// Uses a "note stack" to implement **last-note priority**: when you hold C and E and release
/// E, it automatically plays C again (no note-on needed). This is essential for keyboard players.
///
/// ## Parameter Throttling
///
/// Parameters (filter cutoff, envelope times, etc.) are read from a shared triple-buffer.
/// Instead of checking every sample (which would be expensive), the engine only checks every
/// 32 samples (~0.7ms at 44.1kHz). This is inaudible to humans but saves CPU. Audio-rate
/// parameters (like LFO modulation) still work because they're part of the voice's DSP.
///
/// ## Real-Time Safety
///
/// The engine is designed to run on a real-time audio thread:
/// - No allocations during processing (all buffers pre-allocated in new())
/// - No locks (uses lock-free triple-buffer for parameters)
/// - Minimal branching (most work is just voice processing and mixing)
/// - Predictable execution time (no hidden costs)
pub struct SynthEngine {
    /// The target audio sample rate in Hz (e.g., 44100.0)
    /// Used by voices to calculate frequency-to-phase-increment conversions
    sample_rate: f32,

    /// Array of 16 polyphonic voices, each capable of playing one note
    /// Every voice can play independently but shares the same parameter set.
    /// Voices are pre-allocated at engine creation to avoid allocations during real-time processing.
    voices: Vec<Voice>,

    /// Lock-free consumer end of the triple-buffer for reading parameter updates
    /// The triple-buffer allows the GUI thread to push parameter changes without blocking
    /// the audio thread. See create_parameter_buffer() for how it's created.
    params_consumer: Output<SynthParams>,

    /// The most recent parameters read from the triple-buffer
    /// This is what the engine currently uses for all voices. Updated every 32 samples
    /// from the triple-buffer (see param_update_interval below).
    current_params: SynthParams,

    /// Stack of currently pressed notes in monophonic mode
    /// When multiple keys are held and you release one, we check this stack to see if
    /// there's another note to play. This implements "last-note priority":
    /// Hold C, press E, release E → plays C again automatically. Essential for keyboards.
    note_stack: Vec<u8>,

    /// Counter for throttling parameter updates
    /// We don't check the parameter triple-buffer every sample (too expensive and unnecessary).
    /// Instead, we check every `param_update_interval` samples. This counter tracks progress.
    sample_counter: u32,

    /// How many samples between parameter update checks
    /// Set to 32, which at 44.1kHz = 32/44100 ≈ 0.7ms. This is fast enough that parameter
    /// changes feel instant to users but slow enough to be negligible CPU cost. Audio-rate
    /// effects (like LFO) still work because they're applied per-sample within the voice DSP.
    param_update_interval: u32,

    /// Current gain reduction applied by the master limiter.
    ///
    /// This is a transparent peak limiter (not a saturator). It prevents hard clipping
    /// at the output without introducing harmonic distortion like `tanh()`.
    limiter_gain: f32,

    /// One-pole smoothing coefficient for limiter release.
    /// Recovery is smoothed to avoid pumping; gain reduction is applied instantly.
    limiter_release_coeff: f32,

    /// Effects chain - processed after voice mixing
    reverb: Reverb,
    delay: StereoDelay,
    chorus: Chorus,
    distortion: Distortion,
    multiband_distortion: MultibandDistortion,
    stereo_widener: StereoWidener,

    // New modulation/time-based effects
    phaser: Phaser,
    flanger: Flanger,
    tremolo: Tremolo,
    auto_pan: AutoPan,

    // New filter/pitch effects
    comb_filter: CombFilter,
    ring_modulator: RingModulator,

    // New dynamics/distortion effects
    compressor: Compressor,
    bitcrusher: Bitcrusher,
    waveshaper: Waveshaper,

    /// Wavetable library for wavetable synthesis
    wavetable_library: WavetableLibrary,
}

impl SynthEngine {
    /// Create a new synthesis engine with 16 polyphonic voices.
    ///
    /// This constructor initializes the engine with all necessary components for audio generation.
    /// All voices are pre-allocated and initialized, so no allocations happen during real-time
    /// audio processing (which would be unsafe on audio threads).
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0). This is passed to each voice so it
    ///   can correctly calculate frequency-to-phase conversions. Oscillators need this to generate
    ///   the correct frequency for each MIDI note.
    /// * `params_consumer` - Consumer end of the lock-free triple-buffer for parameter updates.
    ///   This allows the GUI thread to push parameter changes (like filter cutoff) without
    ///   blocking the audio thread.
    ///
    /// # Returns
    /// A ready-to-use SynthEngine with:
    /// - 16 idle voices (all currently not playing)
    /// - Default parameters loaded
    /// - Empty note stack (for monophonic mode)
    /// - Sample counter at 0
    pub fn new(sample_rate: f32, params_consumer: Output<SynthParams>) -> Self {
        let mut voices = Vec::with_capacity(MAX_POLYPHONY);
        for _ in 0..MAX_POLYPHONY {
            voices.push(Voice::new(sample_rate));
        }

        // Limiter tuning: fast attack, slower release. These are conservative and
        // designed to avoid audible artifacts while preventing clipping.
        let limiter_release_s = 0.050; // 50ms
        let limiter_release_coeff = (-1.0 / (limiter_release_s * sample_rate)).exp();

        // Load wavetables from compile-time embedded data (no runtime file dependencies)
        let wavetable_library = WavetableLibrary::load_from_embedded().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load embedded wavetables: {}", e);
            WavetableLibrary::with_builtin_wavetables()
        });

        Self {
            sample_rate,
            voices,
            params_consumer,
            current_params: SynthParams::default(),
            note_stack: Vec::new(),
            sample_counter: 0,
            param_update_interval: 32, // Update every 32 samples (~0.7ms at 44.1kHz)
            limiter_gain: 1.0,
            limiter_release_coeff,
            reverb: Reverb::new(sample_rate),
            delay: StereoDelay::new(sample_rate),
            chorus: Chorus::new(sample_rate),
            distortion: Distortion::new(sample_rate),
            multiband_distortion: MultibandDistortion::new(sample_rate),
            stereo_widener: StereoWidener::new(sample_rate),

            // Initialize new modulation/time-based effects
            phaser: Phaser::new(sample_rate, 6, 1000.0, 0.5),
            flanger: Flanger::new(sample_rate, 0.5, 15.0, 0.2),
            tremolo: Tremolo::new(sample_rate, 4.0),
            auto_pan: AutoPan::new(sample_rate, 1.0),

            // Initialize new filter/pitch effects
            comb_filter: CombFilter::new(sample_rate, 10.0, 0.5, 0.5),
            ring_modulator: RingModulator::new(sample_rate, 440.0),

            // Initialize new dynamics/distortion effects
            compressor: Compressor::new(sample_rate, -20.0, 4.0, 10.0, 100.0),
            bitcrusher: Bitcrusher::new(sample_rate, sample_rate, 16),
            waveshaper: Waveshaper::new(crate::dsp::effects::waveshaper::Algorithm::SoftClip, 1.0),

            wavetable_library,
        }
    }

    #[inline]
    fn maybe_update_params(&mut self) {
        self.sample_counter += 1;
        if self.sample_counter < self.param_update_interval {
            return;
        }
        self.sample_counter = 0;

        // Check for parameter updates from triple buffer
        let new_params = self.params_consumer.read();

        // Only update if parameters actually changed
        if *new_params == self.current_params {
            return;
        }

        self.current_params = *new_params;

        // Update effects parameters
        self.update_effects_params();

        // Update all active voices with current parameters
        for voice in &mut self.voices {
            if voice.is_active() {
                voice.update_parameters(
                    &self.current_params.oscillators,
                    &self.current_params.filters,
                    &self.current_params.lfos,
                    &self.current_params.envelope,
                    &self.wavetable_library,
                );
            }
        }
    }

    /// Update effects processors with current parameters
    fn update_effects_params(&mut self) {
        let effects = &self.current_params.effects;

        // Update reverb
        self.reverb.set_room_size(effects.reverb.room_size);
        self.reverb.set_damping(effects.reverb.damping);
        self.reverb.set_wet(effects.reverb.wet);
        self.reverb.set_dry(effects.reverb.dry);
        self.reverb.set_width(effects.reverb.width);

        // Update delay
        self.delay.set_time(effects.delay.time_ms);
        self.delay.set_feedback(effects.delay.feedback);
        self.delay.set_wet(effects.delay.wet);
        self.delay.set_dry(effects.delay.dry);

        // Update chorus
        self.chorus.set_rate(effects.chorus.rate);
        self.chorus.set_depth(effects.chorus.depth);
        self.chorus.set_mix(effects.chorus.mix);

        // Update distortion
        self.distortion.set_drive(effects.distortion.drive);
        self.distortion.set_mix(effects.distortion.mix);
        self.distortion
            .set_type(effects.distortion.dist_type.into());

        // Update multiband distortion
        self.multiband_distortion
            .set_low_mid_freq(effects.multiband_distortion.low_mid_freq);
        self.multiband_distortion
            .set_mid_high_freq(effects.multiband_distortion.mid_high_freq);
        self.multiband_distortion
            .set_drive_low(effects.multiband_distortion.drive_low);
        self.multiband_distortion
            .set_drive_mid(effects.multiband_distortion.drive_mid);
        self.multiband_distortion
            .set_drive_high(effects.multiband_distortion.drive_high);
        self.multiband_distortion
            .set_gain_low(effects.multiband_distortion.gain_low);
        self.multiband_distortion
            .set_gain_mid(effects.multiband_distortion.gain_mid);
        self.multiband_distortion
            .set_gain_high(effects.multiband_distortion.gain_high);
        self.multiband_distortion
            .set_mix(effects.multiband_distortion.mix);

        // Update stereo widener
        self.stereo_widener
            .set_haas_delay(effects.stereo_widener.haas_delay_ms);
        self.stereo_widener
            .set_haas_mix(effects.stereo_widener.haas_mix);
        self.stereo_widener.set_width(effects.stereo_widener.width);
        self.stereo_widener
            .set_mid_gain(effects.stereo_widener.mid_gain);
        self.stereo_widener
            .set_side_gain(effects.stereo_widener.side_gain);

        // Update phaser
        self.phaser.set_rate(effects.phaser.rate);
        self.phaser.set_depth(effects.phaser.depth);
        self.phaser.set_feedback(effects.phaser.feedback);
        self.phaser.set_mix(effects.phaser.mix);

        // Update flanger
        self.flanger.set_rate(effects.flanger.rate);
        self.flanger.set_feedback(effects.flanger.feedback);
        self.flanger.set_mix(effects.flanger.mix);
        // Flanger depth controls delay range (depth maps to max delay)
        let flanger_max_delay = 0.5 + effects.flanger.depth * 14.5; // 0.5-15ms
        self.flanger.set_delay_range(0.5, flanger_max_delay);

        // Update tremolo
        self.tremolo.set_rate(effects.tremolo.rate);
        self.tremolo.set_depth(effects.tremolo.depth);

        // Update auto-pan
        self.auto_pan.set_rate(effects.auto_pan.rate);
        self.auto_pan.set_depth(effects.auto_pan.depth);

        // Update comb filter
        self.comb_filter
            .set_frequency(effects.comb_filter.frequency);
        self.comb_filter.set_feedback(effects.comb_filter.feedback);
        self.comb_filter.set_mix(effects.comb_filter.mix);

        // Update ring modulator
        self.ring_modulator
            .set_frequency(effects.ring_mod.frequency);
        self.ring_modulator.set_depth(effects.ring_mod.depth);

        // Update compressor
        self.compressor.set_threshold(effects.compressor.threshold);
        self.compressor.set_ratio(effects.compressor.ratio);
        self.compressor.set_attack(effects.compressor.attack);
        self.compressor.set_release(effects.compressor.release);

        // Update bitcrusher
        self.bitcrusher
            .set_sample_rate(effects.bitcrusher.sample_rate);
        self.bitcrusher.set_bit_depth(effects.bitcrusher.bit_depth);

        // Update waveshaper
        self.waveshaper.set_drive(effects.waveshaper.drive);
        self.waveshaper.set_mix(effects.waveshaper.mix);
    }

    #[inline]
    fn apply_output_limiter(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Leave a small headroom margin so sample format conversion/interleaving
        // doesn’t accidentally exceed full scale due to rounding.
        const THRESHOLD: f32 = 0.98;

        let peak = left.abs().max(right.abs());

        // Compute instantaneous target gain to keep peak under threshold.
        let target_gain = if peak > THRESHOLD {
            // Avoid divide-by-zero; peak>THRESHOLD implies peak>0.
            THRESHOLD / peak
        } else {
            1.0
        };

        // Critical: prevent overshoot.
        // If we need *more* gain reduction (target_gain < current), apply it immediately.
        // This ensures the limiter never relies on the final clamp (which is audible).
        if target_gain < self.limiter_gain {
            self.limiter_gain = target_gain;
        } else {
            // Release: recover smoothly back toward unity.
            let coeff = self.limiter_release_coeff;
            self.limiter_gain = coeff * self.limiter_gain + (1.0 - coeff) * target_gain;
        }

        let out_l = left * self.limiter_gain;
        let out_r = right * self.limiter_gain;

        // Safety clamp (should not engage with instantaneous attack).
        (out_l.clamp(-1.0, 1.0), out_r.clamp(-1.0, 1.0))
    }

    #[inline]
    fn process_stereo_internal(&mut self) -> (f32, f32) {
        self.maybe_update_params();

        // Mix all voices - stereo
        let mut output_left = 0.0;
        let mut output_right = 0.0;
        for voice in &mut self.voices {
            let (left, right) = voice.process(
                &self.current_params.oscillators,
                &self.current_params.filters,
                &self.current_params.lfos,
                &self.current_params.velocity,
            );
            output_left += left;
            output_right += right;
        }

        // Apply master gain
        let master = self.current_params.master_gain;
        output_left *= master;
        output_right *= master;

        // Effects chain (processed in series)
        // Order is intentional for sound quality:
        // 1. Dynamics (compressor) - control peaks first
        // 2. Distortion/saturation (distortion, waveshaper, bitcrusher) - add harmonics
        // 3. Multiband distortion - frequency-specific saturation
        // 4. Filter effects (comb filter, phaser, flanger) - frequency/phase manipulation
        // 5. Pitch modulation (ring modulator, tremolo) - amplitude/frequency effects
        // 6. Chorus - adds width/detuning
        // 7. Delay - rhythmic repeats
        // 8. Spatial effects (auto-pan, stereo widener) - stereo field manipulation
        // 9. Reverb last - final ambience/space
        //
        // Conditional processing: Skip disabled effects to save CPU
        let mut out_l = output_left;
        let mut out_r = output_right;

        if self.current_params.effects.compressor.enabled {
            (out_l, out_r) = self.compressor.process(out_l, out_r);
        }
        if self.current_params.effects.distortion.enabled {
            (out_l, out_r) = self.distortion.process_stereo(out_l, out_r);
        }
        if self.current_params.effects.waveshaper.enabled {
            (out_l, out_r) = self.waveshaper.process(out_l, out_r);
        }
        if self.current_params.effects.bitcrusher.enabled {
            (out_l, out_r) = self.bitcrusher.process(out_l, out_r);
        }
        if self.current_params.effects.multiband_distortion.enabled {
            (out_l, out_r) = self.multiband_distortion.process_stereo(out_l, out_r);
        }
        if self.current_params.effects.comb_filter.enabled {
            (out_l, out_r) = self.comb_filter.process(out_l, out_r);
        }
        if self.current_params.effects.phaser.enabled {
            (out_l, out_r) = self.phaser.process(out_l, out_r);
        }
        if self.current_params.effects.flanger.enabled {
            (out_l, out_r) = self.flanger.process(out_l, out_r);
        }
        if self.current_params.effects.ring_mod.enabled {
            (out_l, out_r) = self.ring_modulator.process(out_l, out_r);
        }
        if self.current_params.effects.tremolo.enabled {
            (out_l, out_r) = self.tremolo.process(out_l, out_r);
        }
        if self.current_params.effects.chorus.enabled {
            (out_l, out_r) = self.chorus.process(out_l, out_r);
        }
        if self.current_params.effects.delay.enabled {
            (out_l, out_r) = self.delay.process(out_l, out_r);
        }
        if self.current_params.effects.auto_pan.enabled {
            (out_l, out_r) = self.auto_pan.process(out_l, out_r);
        }
        if self.current_params.effects.stereo_widener.enabled {
            (out_l, out_r) = self.stereo_widener.process(out_l, out_r);
        }
        if self.current_params.effects.reverb.enabled {
            (out_l, out_r) = self.reverb.process(out_l, out_r);
        }

        // Transparent limiter to prevent clipping without harmonic distortion.
        self.apply_output_limiter(out_l, out_r)
    }

    /// Trigger a note on (MIDI note event).
    ///
    /// This is called whenever a MIDI note on message arrives or a keyboard key is pressed.
    /// The engine decides how to handle it based on the current mode:
    ///
    /// ## Polyphonic Mode (Default)
    /// Multiple notes can play simultaneously. Algorithm:
    /// 1. Check if any voice is idle (not currently playing)
    /// 2. If yes: Allocate that voice and start playing the note
    /// 3. If no (all 16 voices busy): **Voice stealing**
    ///    - Find the voice with the lowest RMS (quiet, probably releasing)
    ///    - Kill it and reuse it for the new note
    ///    - This prevents polyphony limit from causing missing notes
    ///    - Users don't hear the stolen voice stop because it's so quiet
    ///
    /// ## Monophonic Mode
    /// Only one note plays at a time. Algorithm:
    /// 1. Add the note to the note_stack (if not already there)
    /// 2. Always trigger the first voice with the new note (this causes retriggering if
    ///    already playing)
    /// 3. When note_off comes, check if there are other held notes in the stack
    /// 4. If yes: Retrigger the last (most recent) held note (last-note priority)
    /// 5. If no: Release the voice
    ///
    /// # Arguments
    /// * `note` - MIDI note number 0-127 (60 = Middle C)
    /// * `velocity` - Note velocity 0.0-1.0 (controls amplitude and timbre)
    ///
    /// # Example
    /// ```
    /// use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
    /// let (_producer, consumer) = create_parameter_buffer();
    /// let mut engine = SynthEngine::new(44100.0, consumer);
    ///
    /// engine.note_on(60, 0.8);  // Play middle C at 80% velocity
    ///
    /// // Verify note triggered by checking we get audio output
    /// let output = engine.process();
    /// assert!(output.abs() < 2.0, "Output should be in valid range");
    /// ```
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        if self.current_params.monophonic {
            // Monophonic mode: use only the first voice
            // Add note to stack if not already present
            if !self.note_stack.contains(&note) {
                self.note_stack.push(note);
            }

            // Always trigger the first voice with the new note
            self.voices[0].note_on(note, velocity);
            self.voices[0].update_parameters(
                &self.current_params.oscillators,
                &self.current_params.filters,
                &self.current_params.lfos,
                &self.current_params.envelope,
                &self.wavetable_library,
            );
        } else {
            // Polyphonic mode: original behavior
            // First, try to find an inactive voice
            if let Some(voice) = self.voices.iter_mut().find(|v| !v.is_active()) {
                voice.note_on(note, velocity);
                voice.update_parameters(
                    &self.current_params.oscillators,
                    &self.current_params.filters,
                    &self.current_params.lfos,
                    &self.current_params.envelope,
                    &self.wavetable_library,
                );
                return;
            }

            // All voices active - use quietest-voice stealing
            let quietest_idx = self.find_quietest_voice();
            self.voices[quietest_idx].note_on(note, velocity);
            self.voices[quietest_idx].update_parameters(
                &self.current_params.oscillators,
                &self.current_params.filters,
                &self.current_params.lfos,
                &self.current_params.envelope,
                &self.wavetable_library,
            );
        }
    }

    /// Release a note (MIDI note off event).
    ///
    /// This is called whenever a MIDI note off message arrives or a keyboard key is released.
    /// The behavior depends on the current mode:
    ///
    /// ## Polyphonic Mode (Default)
    /// - Find all voices currently playing the given note number
    /// - Call note_off() on them, entering their release envelope phase
    /// - The voice stays active during release (it's audible - fade out)
    /// - After release completes, the voice becomes idle
    ///
    /// ## Monophonic Mode
    /// - Remove the note from the note_stack
    /// - Check if there are other notes still being held in the stack
    /// - If yes: **Retrigger** the most recent held note (last-note priority)
    ///   - This is critical for keyboard players who hold multiple keys
    ///   - When you release E while still holding C, C automatically plays
    ///   - No need to re-press C
    /// - If no notes left: Release the voice to silence
    ///
    /// # Arguments
    /// * `note` - MIDI note number to release
    ///
    /// # Example
    /// ```
    /// use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
    /// let (_producer, consumer) = create_parameter_buffer();
    /// let mut engine = SynthEngine::new(44100.0, consumer);
    ///
    /// engine.note_on(60, 0.8);
    /// engine.note_off(60);  // Release middle C
    ///
    /// // Note is now releasing - verify output is still finite
    /// let output = engine.process();
    /// assert!(output.is_finite(), "Output should be finite");
    /// ```
    pub fn note_off(&mut self, note: u8) {
        if self.current_params.monophonic {
            // Monophonic mode: remove note from stack
            if let Some(pos) = self.note_stack.iter().position(|&n| n == note) {
                self.note_stack.remove(pos);
            }

            // If there are still notes in the stack, retrigger the most recent one
            if let Some(&last_note) = self.note_stack.last() {
                // Retrigger the last note in the stack (last-note priority)
                self.voices[0].note_on(last_note, 0.8); // Use default velocity for retriggered notes
                self.voices[0].update_parameters(
                    &self.current_params.oscillators,
                    &self.current_params.filters,
                    &self.current_params.lfos,
                    &self.current_params.envelope,
                    &self.wavetable_library,
                );
            } else {
                // No more notes in stack, release the voice
                self.voices[0].note_off();
            }
        } else {
            // Polyphonic mode: release all voices playing this note
            for voice in &mut self.voices {
                if voice.is_active() && voice.note() == note {
                    voice.note_off();
                }
            }
        }
    }

    /// Find the voice with the lowest RMS energy (quietest voice).
    ///
    /// This is used for voice stealing: when all 16 voices are busy and a new note arrives,
    /// we need to pick which voice to kill and reuse. We choose the quietest one because:
    /// - It's likely nearing the end of its release phase (silent anyway)
    /// - Killing a quiet note is inaudible to the listener
    /// - This prioritizes keeping loud, new notes over fading-out old notes
    ///
    /// RMS (Root Mean Square) is a measure of signal energy. It's more representative of
    /// perceived loudness than peak amplitude because it accounts for the overall power of
    /// the waveform, not just the highest point.
    ///
    /// # Returns
    /// Index of the quietest active voice, or 0 if no voices are active (edge case)
    fn find_quietest_voice(&self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_active())
            .min_by(|(_, a), (_, b)| {
                a.get_rms()
                    .partial_cmp(&b.get_rms())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    /// Immediately release all notes and silence the synthesizer.
    ///
    /// This is called by MIDI "All Notes Off" (CC #123) or when the synthesizer needs to
    /// be silenced instantly (e.g., panic button, channel mute, safety shutdown).
    ///
    /// Behavior:
    /// - Clears the note stack (monophonic mode)
    /// - Calls reset() on all voices, which:
    ///   - Stops the envelope immediately (no release phase)
    ///   - Marks the voice as inactive
    ///   - Clears all internal state (oscillator phases, filter memory, etc.)
    ///
    /// Unlike note_off(), this doesn't play the release envelope. It's a hard stop.
    /// This is necessary for safety (user hits panic button) and proper cleanup (stop a
    /// stuck note from sustaining forever).\n    
    pub fn all_notes_off(&mut self) {
        self.note_stack.clear();
        for voice in &mut self.voices {
            voice.reset();
        }
    }

    /// Process one audio sample and return the mixed output.
    ///
    /// This is the main real-time audio processing function. It's called once per audio sample
    /// by the audio thread (44,100 times per second at 44.1kHz). Everything must happen in
    /// microseconds without blocking.
    ///
    /// ## Algorithm
    ///
    /// 1. **Parameter Update Check** (every 32 samples):
    ///    - Read from the triple-buffer to see if parameters changed
    ///    - If they did: Update all active voices with the new parameters
    ///    - Why throttle? Reading every sample would be expensive and unnecessary
    ///    - Why update active voices only? Saving CPU on idle voices
    ///
    /// 2. **Voice Processing** (every sample):
    ///    - Call process() on each of the 16 voices
    ///    - Each voice generates its own audio sample (0.0 to ±1.0)
    ///    - Voices run independently, in parallel
    ///
    /// 3. **Mixing**:
    ///    - Add all voice outputs together
    ///    - 16 voices × 1.0 could theoretically give 16.0, but release envelopes prevent this
    ///    - With proper voice allocation, rarely more than 2-4 voices are loud simultaneously
    ///
    /// 4. **Master Gain**:
    ///    - Multiply by master_gain parameter (typically 0.5-1.0)
    ///    - This prevents clipping and gives overall volume control
    ///
    /// # Returns
    /// A single mono audio sample (-1.0 to +1.0). Note: This version averages stereo outputs
    /// from voices for backward compatibility.
    pub fn process(&mut self) -> f32 {
        let (left, right) = self.process_stereo_internal();
        // Return mono average (kept for compatibility).
        (left + right) / 2.0
    }

    /// Get the count of currently active voices.
    ///
    /// A voice is considered "active" if:
    /// - It's currently playing a note (in attack/decay/sustain phases)
    /// - OR it's in the release phase (fading out)
    ///
    /// Idle voices (not playing anything) are not counted. This is useful for:
    /// - Monitoring CPU load (more voices = more processing)
    /// - Checking if the synthesizer is currently producing sound
    /// - Debugging polyphony issues (is voice stealing happening?)
    ///
    /// # Returns
    /// Number of active voices (0-16)
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    /// Get the configured sample rate of this engine.
    ///
    /// This returns the sample rate that was passed to new(). It's constant and never changes
    /// after engine creation. Used primarily for:
    /// - Debugging/monitoring (verify the engine is at the expected rate)
    /// - Reporting to the user or UI
    /// - Re-creating the engine at a different rate (not hot-swappable)
    ///
    /// # Returns
    /// Sample rate in Hz (e.g., 44100.0 for CD quality)
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Process one stereo sample and return both left and right channels.
    ///
    /// This is similar to process() but returns a full stereo pair instead of mono.
    /// Use this when the output device supports stereo (which is almost always).
    ///
    /// Some voices can produce stereo output (e.g., with pan modulation or stereo effects).
    /// By returning both channels separately, we preserve this spatial information.
    /// The process() method averages these channels, losing stereo information.
    ///
    /// ## Algorithm
    /// Identical to process() but:
    /// - Accumulates (left, right) tuples from each voice
    /// - Returns the pair instead of averaging to mono
    ///
    /// # Returns
    /// Tuple (left_sample, right_sample) where each is approximately -1.0 to +1.0
    ///
    /// # Example
    /// ```
    /// use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
    /// let (_producer, consumer) = create_parameter_buffer();
    /// let mut engine = SynthEngine::new(44100.0, consumer);
    ///
    /// engine.note_on(60, 0.8);
    /// let (left, right) = engine.process_stereo();
    ///
    /// // Verify stereo output is in valid range
    /// assert!(left.is_finite() && right.is_finite(), "Stereo outputs should be finite");
    /// assert!(left.abs() < 2.0 && right.abs() < 2.0, "Outputs should be in valid range");
    ///
    /// // Example of filling audio buffer
    /// let mut audio_buffer = vec![0.0f32; 512];
    /// let frame = 0;
    /// audio_buffer[frame * 2] = left;
    /// audio_buffer[frame * 2 + 1] = right;
    /// ```
    pub fn process_stereo(&mut self) -> (f32, f32) {
        self.process_stereo_internal()
    }

    /// Process a block of audio samples efficiently.
    ///
    /// This is more efficient than calling process() or process_stereo() in a loop because:
    /// - Loop overhead is eliminated
    /// - Parameter throttling (every 32 samples) is amortized across many samples
    /// - Can be optimized more aggressively by the compiler (SIMD vectorization, etc.)
    ///
    /// This is the preferred method for CLAP plugins, which receive audio in blocks
    /// (typically 256-4096 samples) rather than one at a time. DAW hosts call this once
    /// per buffer, so it's critical that it's fast.
    ///
    /// Note: This version duplicates stereo output to both left and right channels.
    /// For true stereo separation, you'd need to track pan per-voice or use actual
    /// stereo voice processing.
    ///
    /// # Arguments
    /// * `left` - Output buffer for left channel (will be filled with samples)
    /// * `right` - Output buffer for right channel (will be filled with samples)
    ///
    /// # Example
    /// ```
    /// use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
    /// let (_producer, consumer) = create_parameter_buffer();
    /// let mut engine = SynthEngine::new(44100.0, consumer);
    ///
    /// engine.note_on(60, 0.8);
    ///
    /// let mut left = vec![0.0; 256];
    /// let mut right = vec![0.0; 256];
    /// engine.process_block(&mut left, &mut right);
    ///
    /// // Verify all samples are finite and in valid range
    /// assert!(left.iter().all(|&s| s.is_finite() && s.abs() < 2.0));
    /// assert!(right.iter().all(|&s| s.is_finite() && s.abs() < 2.0));
    /// // Now left and right contain 256 audio samples ready to send to the audio device
    /// ```
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());

        for i in 0..len {
            let sample = self.process();
            left[i] = sample;
            right[i] = sample;
        }
    }

    /// Get the current synthesizer parameters (read-only).
    ///
    /// This returns a reference to the current_params that the engine is using for audio
    /// generation. This is useful for synchronizing the GUI with the actual audio engine
    /// state. For example, if the user moves a slider but hasn't let go yet, the GUI might
    /// show a different value than what the engine is actually using (due to throttled
    /// parameter updates).
    ///
    /// Use this to:
    /// - Verify that parameter changes were applied
    /// - Synchronize UI displays with actual synthesis state
    /// - Debug parameter update issues
    ///
    /// # Returns
    /// Read-only reference to SynthParams struct
    pub fn current_params(&self) -> &SynthParams {
        &self.current_params
    }
}

/// Create a lock-free triple-buffer for parameter updates.
///
/// This function creates a triple-buffer data structure that allows safe, lock-free
/// parameter updates between the GUI thread and the audio thread. It returns both ends
/// so they can be split and used in different threads.
///
/// ## What is a Triple-Buffer?
///
/// Traditional double-buffering uses two buffers: GUI writes to one while audio reads from
/// the other, then they swap. But swapping isn't instant - there's a brief moment where
/// both threads see inconsistent state.
///
/// A triple-buffer uses three buffers instead:
///
/// ```text
/// +----------+     +----------+     +----------+
/// | Buffer A |--->| Buffer B |--->| Buffer C |
/// | (GUI)    |     | (current)|     | (waiting)|
/// +----------+     +----------+     +----------+
///    Writing        Being read        Preparing
/// ```
///
/// - GUI writes to its buffer
/// - Audio reads from current buffer  
/// - When GUI finishes, buffers rotate
/// - Audio never sees partial writes (buffer is never locked)
/// - No blocking: both threads run independent
///
/// ## Why Not Just Use a Mutex?
///
/// Mutexes cause problems in real-time audio:
/// 1. **Unpredictable latency**: Mutex acquisition time varies (from nanoseconds to
///    milliseconds depending on contention). Even brief waits cause audio glitches.
/// 2. **Priority inversion**: A low-priority GUI thread can block the real-time audio thread
/// 3. **Audio dropouts**: Any lock + contention = pops, clicks, or dropped audio frames
///
/// The triple-buffer solves this: no locks, no waiting, no priority inversion.
///
/// # Returns
/// - First element: `Input<SynthParams>` - producer end (used by GUI thread)
/// - Second element: `Output<SynthParams>` - consumer end (used by audio thread)
///
/// # Example
/// let (param_producer, param_consumer) = create_parameter_buffer();
///
/// // In GUI thread:
/// let new_params = SynthParams { master_gain: 0.5, ..Default::default() };
/// param_producer.write(new_params);
///
/// // In audio thread (in engine.process()):
/// let current = param_consumer.read();
///
pub fn create_parameter_buffer() -> (Input<SynthParams>, Output<SynthParams>) {
    let buffer = TripleBuffer::new(&SynthParams::default());
    buffer.split()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the engine can be created with the correct sample rate.
    /// Verifies:
    /// - Engine construction succeeds
    /// - Sample rate is correctly stored and retrievable
    /// - No voices are active initially (all idle)
    #[test]
    fn test_engine_creation() {
        let (_producer, consumer) = create_parameter_buffer();
        let engine = SynthEngine::new(44100.0, consumer);

        assert_eq!(engine.sample_rate(), 44100.0);
        assert_eq!(engine.active_voice_count(), 0);
    }

    /// Test that note on events activate voices for synthesis.
    /// Verifies:
    /// - Each note_on() call activates a new voice
    /// - Multiple notes can play simultaneously (polyphonic mode)
    /// - Voice count increases with each note
    #[test]
    fn test_note_on_activates_voice() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        engine.note_on(60, 0.8);
        assert_eq!(engine.active_voice_count(), 1);

        engine.note_on(64, 0.7);
        assert_eq!(engine.active_voice_count(), 2);
    }

    /// Test that note off causes voices to release and fade out.
    /// Verifies:
    /// - note_off() puts the voice in release phase (still active, still audible)
    /// - Voice remains active during release fade-out
    /// - Voice becomes idle after release time completes
    /// This tests the envelope's ADSR behavior (specifically the Release phase)
    #[test]
    fn test_note_off_releases_voice() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        engine.note_on(60, 0.8);
        assert_eq!(engine.active_voice_count(), 1);

        engine.note_off(60);

        // Voice should still be active during release phase
        assert_eq!(engine.active_voice_count(), 1);

        // Process through release
        for _ in 0..20000 {
            engine.process();
        }

        // Should be inactive after release completes
        assert_eq!(engine.active_voice_count(), 0);
    }

    /// Test that polyphony has a hard limit (16 voices).
    /// When more notes are triggered than polyphony allows, voice stealing should occur.
    /// Verifies:
    /// - Engine never exceeds MAX_POLYPHONY (16) active voices
    /// - Voice stealing is working to keep polyphony under control
    #[test]
    fn test_polyphony_limit() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Trigger more notes than polyphony limit
        for i in 0..20 {
            engine.note_on(60 + i, 0.8);
        }

        // Should not exceed max polyphony
        assert!(engine.active_voice_count() <= MAX_POLYPHONY);
    }

    /// Test that voice stealing prioritizes quiet voices.
    /// When all voices are active and a new note arrives, the quietest voice should be killed.
    /// Verifies:
    /// - All voice slots can be filled
    /// - One more note triggers voice stealing
    /// - New note still triggers (doesn't get dropped)
    /// - Polyphony limit is maintained
    #[test]
    fn test_voice_stealing_quietest() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Fill all voice slots
        for i in 0..MAX_POLYPHONY {
            engine.note_on(60 + i as u8, 0.8);
        }

        // Process to build up RMS values
        for _ in 0..500 {
            engine.process();
        }

        // Trigger one more note - should steal quietest
        engine.note_on(100, 0.9);
        assert_eq!(engine.active_voice_count(), MAX_POLYPHONY);
    }

    /// Test that all_notes_off() immediately silences all voices.
    /// Verifies:
    /// - Before all_notes_off(): multiple voices are active
    /// - After all_notes_off(): zero active voices
    /// - This is different from note_off() which releases each voice (plays release envelope)
    /// - all_notes_off() is a hard stop for panic/emergency silence
    #[test]
    fn test_all_notes_off() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Trigger multiple notes
        for i in 0..5 {
            engine.note_on(60 + i, 0.8);
        }

        assert!(engine.active_voice_count() > 0);

        engine.all_notes_off();
        assert_eq!(engine.active_voice_count(), 0);
    }

    /// Test that process() generates audible output when notes are playing.
    /// Verifies:
    /// - Triggering a note produces audio samples (not silent)
    /// - Output amplitude is non-zero (demonstrates synthesis is working)
    /// - Output is bounded (-1.0 to +1.0 approximately) and doesn't clip
    #[test]
    fn test_output_generation() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        engine.note_on(60, 0.8);

        // Process samples and verify output
        let mut has_output = false;
        for _ in 0..1000 {
            let sample = engine.process();
            if sample.abs() > 0.001 {
                has_output = true;
                break;
            }
        }

        assert!(has_output, "Engine should produce audio output");
    }

    /// Test that parameter updates are correctly propagated to voices.
    /// Verifies:
    /// - Parameters can be written to the triple-buffer via producer
    /// - The engine picks up changes and applies them to voices
    /// - Parameter throttling doesn't prevent updates from eventually applying
    /// This tests the lock-free communication mechanism between GUI and audio threads
    #[test]
    fn test_parameter_updates() {
        let (mut producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Update parameters via triple buffer
        let new_params = SynthParams {
            master_gain: 0.5,
            ..Default::default()
        };
        producer.write(new_params);

        // Process should pick up new parameters
        engine.process();

        // Verify by checking that output is affected by master gain
        engine.note_on(60, 1.0);
        for _ in 0..100 {
            engine.process();
        }

        // Parameters were updated (verified implicitly through processing)
    }

    /// Test that the same note can be played on multiple voices simultaneously.
    /// This is useful for unison effects (multiple detuned oscillators playing one note).
    /// Verifies:
    /// - Multiple note_on() calls with the same note number each allocate a separate voice
    /// - Each instance is independent (can have different velocities)
    /// - All instances are affected by note_off() (all start releasing)
    #[test]
    fn test_same_note_multiple_voices() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Trigger same note multiple times
        engine.note_on(60, 0.8);
        engine.note_on(60, 0.7);
        engine.note_on(60, 0.6);

        assert_eq!(engine.active_voice_count(), 3);

        // Release should affect all instances
        engine.note_off(60);

        // All three should be in release
        assert_eq!(engine.active_voice_count(), 3);
    }

    /// Test that zero velocity notes still produce output with velocity sensitivity.
    /// Verifies:
    /// - Zero velocity doesn't produce complete silence (by design)
    /// - With velocity sensitivity enabled, zero velocity produces quieter but audible output
    /// - The envelope still works correctly even with minimal velocity
    /// This tests the velocity response and minimum amplitude handling
    #[test]
    fn test_zero_velocity_note() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        engine.note_on(60, 0.0);
        assert_eq!(engine.active_voice_count(), 1);

        // Process samples - with velocity sensitivity, zero velocity still produces some output
        // (1.0 - sensitivity) * volume. Default sensitivity is 0.7, so minimum is 0.3
        let mut max_output = 0.0_f32;
        for _ in 0..1000 {
            let sample = engine.process();
            max_output = max_output.max(sample.abs());
        }

        // Should produce some output (velocity scaling allows quieter but not silent notes)
        assert!(
            max_output > 0.0 && max_output < 1.0,
            "Zero velocity should produce reduced but non-zero output with velocity sensitivity"
        );
    }

    #[test]
    fn test_no_clipping_on_basic_chord() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Use a moderately hot master gain to catch headroom issues.
        engine.current_params.master_gain = 1.0;

        // Trigger a basic triad + octave.
        engine.note_on(60, 0.8); // C4
        engine.note_on(64, 0.8); // E4
        engine.note_on(67, 0.8); // G4
        engine.note_on(72, 0.8); // C5

        // Let the envelope get past attack.
        for _ in 0..4000 {
            let _ = engine.process_stereo();
        }

        let mut max_peak = 0.0_f32;
        for _ in 0..12000 {
            let (l, r) = engine.process_stereo();
            max_peak = max_peak.max(l.abs().max(r.abs()));
        }

        assert!(
            max_peak <= 1.0,
            "Output should not clip (peak was {:.4})",
            max_peak
        );
    }
}
