//! Core synthesis engine module.
//!
//! This module contains the main `SynthEngine` that orchestrates polyphonic voice management,
//! parameter updates, and effects processing.

#[cfg(test)]
pub mod tests;

use crate::audio::voice::Voice;
use crate::dsp::effects::{
    AutoPan, Bitcrusher, Chorus, CombFilter, Compressor, Distortion, Exciter, Flanger,
    MultibandDistortion, Phaser, Reverb, RingModulator, StereoDelay, StereoWidener, Tremolo,
    Waveshaper,
};
use crate::dsp::lookahead_limiter::LookAheadLimiter;
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

    /// Stack of currently pressed notes in monophonic mode.
    /// When multiple keys are held and you release one, we check this stack to see if
    /// there's another note to play. This implements "last-note priority":
    /// Hold C, press E, release E → plays C again automatically. Essential for keyboards.
    ///
    /// Stores `(note, velocity)` so that returning to a previously-held note preserves
    /// its original velocity (instead of using a fixed fallback).
    note_stack: Vec<(u8, f32)>,

    /// Counter for throttling parameter updates
    /// We don't check the parameter triple-buffer every sample (too expensive and unnecessary).
    /// Instead, we check every `param_update_interval` samples. This counter tracks progress.
    sample_counter: u32,

    /// How many samples between parameter update checks
    /// Set to 32, which at 44.1kHz = 32/44100 ≈ 0.7ms. This is fast enough that parameter
    /// changes feel instant to users but slow enough to be negligible CPU cost. Audio-rate
    /// effects (like LFO) still work because they're applied per-sample within the voice DSP.
    param_update_interval: u32,

    /// Smoothed polyphonic gain compensation.
    ///
    /// A hard step in poly compensation (e.g., 1.0 → 1/√2 when going 1→2 voices)
    /// is an instantaneous gain change and can be audible as a click.
    poly_gain: f32,

    /// Smoothing coefficient when poly_gain needs to decrease (more attenuation).
    poly_gain_attack_coeff: f32,

    /// Smoothing coefficient when poly_gain needs to increase (less attenuation).
    poly_gain_release_coeff: f32,

    /// Look-ahead limiter for transparent peak limiting with minimal artifacts
    lookahead_limiter: LookAheadLimiter,

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
    exciter: Exciter,

    /// Wavetable library for wavetable synthesis
    wavetable_library: WavetableLibrary,

    /// Current tempo in BPM from DAW transport (defaults to 120.0)
    /// Updated by CLAP plugin from host transport events
    current_tempo_bpm: f64,

    /// Previous tempo sync modes for LFOs and effects (for phase reset detection)
    /// Order: [LFO1, LFO2, LFO3, Chorus, Phaser, Flanger, Tremolo, AutoPan]
    previous_sync_modes: [crate::params::TempoSync; 8],
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

        // Polyphonic gain compensation smoothing.
        // Keep attack fast enough to prevent overload, but smooth enough to avoid clicks
        // when active voice count changes (pressing/releasing keys).
        let poly_attack_s = 0.0010; // 1ms
        let poly_release_s = 0.0100; // 10ms
        let poly_gain_attack_coeff = (-1.0 / (poly_attack_s * sample_rate)).exp();
        let poly_gain_release_coeff = (-1.0 / (poly_release_s * sample_rate)).exp();

        // Load wavetables from compile-time embedded data (no runtime file dependencies)
        let wavetable_library = WavetableLibrary::load_from_embedded().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load embedded wavetables: {}", e);
            WavetableLibrary::with_builtin_wavetables()
        });

        // Initialize look-ahead limiter
        // 5ms look-ahead, 0.99 threshold, 0.5ms attack, 50ms release
        let lookahead_limiter = LookAheadLimiter::new(sample_rate, 5.0, 0.99, 0.5, 50.0);

        Self {
            sample_rate,
            voices,
            params_consumer,
            current_params: SynthParams::default(),
            note_stack: Vec::new(),
            sample_counter: 0,
            param_update_interval: 32, // Update every 32 samples (~0.7ms at 44.1kHz)
            poly_gain: 1.0,
            poly_gain_attack_coeff,
            poly_gain_release_coeff,
            lookahead_limiter,
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
            exciter: Exciter::new(sample_rate),

            wavetable_library,

            current_tempo_bpm: 120.0, // Default tempo
            previous_sync_modes: [crate::params::TempoSync::Hz; 8], // All default to Hz mode
        }
    }

    /// Check and apply parameter updates from the triple-buffer at throttled intervals.
    ///
    /// This function implements **parameter throttling** - a critical real-time audio pattern that:
    /// - Reads parameter changes from the lock-free triple-buffer (pushed by the GUI thread)
    /// - Applies them only every 32 samples (~0.7ms at 44.1kHz) instead of every sample
    /// - Balances responsiveness (fast-enough GUI feedback) with CPU efficiency (fewer updates)
    ///
    /// ## Why Throttling?
    /// Updating every sample would:
    /// - Force expensive operations (recalculating filter coefficients, updating all 16 voices) 44,100 times/sec
    /// - Cause unnecessary CPU overhead since human ears can't perceive changes faster than ~10ms
    /// - Violate real-time audio best practices (minimize work in hot loops)
    ///
    /// At 44.1kHz, updating every 32 samples = ~1378 updates/second = ~10ms granularity, which:
    /// - Feels responsive to user interaction (knob turns appear immediate)
    /// - Allows smooth parameter sweeps (no audible steps)
    /// - Reduces CPU load from ~16,800 updates/sec → ~1,378 updates/sec
    ///
    /// ## Algorithm
    /// 1. Increment sample counter each call
    /// 2. Return early if counter hasn't reached the interval yet (< 32 samples)
    /// 3. Reset counter to 0
    /// 4. Read latest parameters from triple-buffer (non-blocking)
    /// 5. If parameters changed: Apply to all active voices and effects
    /// 6. Handle tempo-synced LFO rate calculations
    /// 7. Detect sync mode changes (for LFO phase reset)
    ///
    /// ## Triple-Buffer Pattern
    /// The triple-buffer is a lock-free data structure with three slots:
    /// - **GUI thread writes** to one slot while the audio thread reads from another
    /// - **No locking needed** - audio thread never blocks on GUI thread
    /// - **Two-phase handshake** swaps slots once per `maybe_update_params()` call
    /// - See [TRIPLE_BUFFER.md](TRIPLE_BUFFER.md) for detailed explanation
    ///
    /// ## Tempo Syncing
    /// When an LFO has `tempo_sync != Hz`:
    /// - Converts musical time divisions (e.g., 1/4 note, 1/8 triplet) to Hz
    /// - Uses current BPM from DAW (via CLAP extension)
    /// - Formula: `rate_hz = tempo_division_to_hz(sync_mode, bpm)`
    /// - Modified rates are passed to voices, not stored to `current_params`
    ///
    /// ## Sync Mode Changes
    /// When user switches an LFO from "Hz" mode to "1/4 Note" tempo sync:
    /// - Detects the mode change here
    /// - Marks it in `previous_sync_modes` array
    /// - Voices receive the mode change and reset LFO phase (see `Voice::update_parameters`)
    /// - This prevents audible discontinuities when switching sync modes
    ///
    /// ## Called From
    /// Called at the start of `process()` every audio sample, but only does
    /// real work every 32 samples due to the throttling check.
    ///
    /// # Performance Note
    /// Despite being `#[inline]`, the early-return pattern (sample_counter < param_update_interval)
    /// means this function has near-zero cost on 31 of every 32 calls - just a counter increment
    /// and comparison. Only 1 in 32 calls actually updates parameters (which is heavier work).
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

        // Apply tempo-synced rates to LFOs before passing to voices
        let mut modified_lfos = self.current_params.lfos;
        for (i, lfo_params) in modified_lfos.iter_mut().enumerate() {
            use crate::params::TempoSync;

            // Check if sync mode changed (for phase reset)
            let sync_mode_changed = self.previous_sync_modes[i] != lfo_params.tempo_sync;
            if sync_mode_changed {
                self.previous_sync_modes[i] = lfo_params.tempo_sync;
                // Phase reset will be handled when voices are updated
                // (voices have direct access to LFO objects)
            }

            // Calculate effective rate
            if lfo_params.tempo_sync != TempoSync::Hz {
                lfo_params.rate =
                    Self::tempo_division_to_hz(lfo_params.tempo_sync, self.current_tempo_bpm);
            }
        }

        // Update effects parameters
        self.update_effects_params();

        // Update all active voices with current parameters (using tempo-synced LFO rates)
        for voice in &mut self.voices {
            if voice.is_active() {
                voice.update_parameters(
                    &self.current_params.oscillators,
                    &self.current_params.filters,
                    &modified_lfos, // Use modified LFO params with tempo-synced rates
                    &self.current_params.envelope,
                    &self.wavetable_library,
                );
            }
        }
    }

    /// Update effects processors with current parameters
    fn update_effects_params(&mut self) {
        // Extract all effect params at once to avoid borrow issues with get_effective_rate
        // Using full structs for consistency and maintainability
        let effects = &self.current_params.effects;
        let reverb_params = effects.reverb;
        let delay_params = effects.delay;
        let chorus_params = effects.chorus;
        let distortion_params = effects.distortion;
        let mb_dist = effects.multiband_distortion;
        let stereo_widener_params = effects.stereo_widener;
        let phaser_params = effects.phaser;
        let flanger_params = effects.flanger;
        let tremolo_params = effects.tremolo;
        let autopan_params = effects.auto_pan;
        let comb_filter_params = effects.comb_filter;
        let ring_mod_params = effects.ring_mod;
        let compressor_params = effects.compressor;
        let bitcrusher_params = effects.bitcrusher;
        let waveshaper_params = effects.waveshaper;
        let exciter_params = effects.exciter;

        // Update reverb
        self.reverb.set_room_size(reverb_params.room_size);
        self.reverb.set_damping(reverb_params.damping);
        self.reverb.set_wet(reverb_params.wet);
        self.reverb.set_dry(reverb_params.dry);
        self.reverb.set_width(reverb_params.width);

        // Update delay
        self.delay.set_time(delay_params.time_ms);
        self.delay.set_feedback(delay_params.feedback);
        self.delay.set_wet(delay_params.wet);
        self.delay.set_dry(delay_params.dry);

        // Update chorus with tempo sync
        let chorus_rate = self.get_effective_rate(
            chorus_params.rate,
            chorus_params.tempo_sync,
            3, // Index in previous_sync_modes (LFO1=0, LFO2=1, LFO3=2, Chorus=3)
        );
        self.chorus.set_rate(chorus_rate);
        self.chorus.set_depth(chorus_params.depth);
        self.chorus.set_mix(chorus_params.mix);

        // Update distortion
        self.distortion.set_drive(distortion_params.drive);
        self.distortion.set_mix(distortion_params.mix);
        self.distortion.set_type(distortion_params.dist_type.into());

        // Update multiband distortion
        self.multiband_distortion
            .set_low_mid_freq(mb_dist.low_mid_freq);
        self.multiband_distortion
            .set_mid_high_freq(mb_dist.mid_high_freq);
        self.multiband_distortion.set_drive_low(mb_dist.drive_low);
        self.multiband_distortion.set_drive_mid(mb_dist.drive_mid);
        self.multiband_distortion.set_drive_high(mb_dist.drive_high);
        self.multiband_distortion.set_gain_low(mb_dist.gain_low);
        self.multiband_distortion.set_gain_mid(mb_dist.gain_mid);
        self.multiband_distortion.set_gain_high(mb_dist.gain_high);
        self.multiband_distortion.set_mix(mb_dist.mix);

        // Update stereo widener
        self.stereo_widener
            .set_haas_delay(stereo_widener_params.haas_delay_ms);
        self.stereo_widener
            .set_haas_mix(stereo_widener_params.haas_mix);
        self.stereo_widener.set_width(stereo_widener_params.width);
        self.stereo_widener
            .set_mid_gain(stereo_widener_params.mid_gain);
        self.stereo_widener
            .set_side_gain(stereo_widener_params.side_gain);

        // Update phaser with tempo sync
        let phaser_rate = self.get_effective_rate(
            phaser_params.rate,
            phaser_params.tempo_sync,
            4, // Phaser index in previous_sync_modes
        );
        self.phaser.set_rate(phaser_rate);
        self.phaser.set_depth(phaser_params.depth);
        self.phaser.set_feedback(phaser_params.feedback);
        self.phaser.set_mix(phaser_params.mix);

        // Update flanger with tempo sync
        let flanger_rate = self.get_effective_rate(
            flanger_params.rate,
            flanger_params.tempo_sync,
            5, // Flanger index in previous_sync_modes
        );
        self.flanger.set_rate(flanger_rate);
        self.flanger.set_feedback(flanger_params.feedback);
        self.flanger.set_mix(flanger_params.mix);
        // Flanger depth controls delay range (depth maps to max delay)
        let flanger_max_delay = 0.5 + flanger_params.depth * 14.5; // 0.5-15ms
        self.flanger.set_delay_range(0.5, flanger_max_delay);

        // Update tremolo with tempo sync
        let tremolo_rate = self.get_effective_rate(
            tremolo_params.rate,
            tremolo_params.tempo_sync,
            6, // Tremolo index in previous_sync_modes
        );
        self.tremolo.set_rate(tremolo_rate);
        self.tremolo.set_depth(tremolo_params.depth);

        // Update auto-pan with tempo sync
        let autopan_rate = self.get_effective_rate(
            autopan_params.rate,
            autopan_params.tempo_sync,
            7, // AutoPan index in previous_sync_modes
        );
        self.auto_pan.set_rate(autopan_rate);
        self.auto_pan.set_depth(autopan_params.depth);

        // Update comb filter
        self.comb_filter.set_frequency(comb_filter_params.frequency);
        self.comb_filter.set_feedback(comb_filter_params.feedback);
        self.comb_filter.set_mix(comb_filter_params.mix);

        // Update ring modulator
        self.ring_modulator.set_frequency(ring_mod_params.frequency);
        self.ring_modulator.set_depth(ring_mod_params.depth);

        // Update compressor
        self.compressor.set_threshold(compressor_params.threshold);
        self.compressor.set_ratio(compressor_params.ratio);
        self.compressor.set_attack(compressor_params.attack);
        self.compressor.set_release(compressor_params.release);

        // Update bitcrusher
        self.bitcrusher
            .set_sample_rate(bitcrusher_params.sample_rate);
        self.bitcrusher.set_bit_depth(bitcrusher_params.bit_depth);

        // Update waveshaper
        self.waveshaper.set_drive(waveshaper_params.drive);
        self.waveshaper.set_mix(waveshaper_params.mix);

        // Update exciter
        self.exciter.set_frequency(exciter_params.frequency);
        self.exciter.set_drive(exciter_params.drive);
        self.exciter.set_mix(exciter_params.mix);
    }

    /// Process one stereo sample and return both left and right channels.
    ///
    /// **This is the primary real-time audio processing method.** It's called once per audio
    /// sample by the audio thread (44,100 times per second at 44.1kHz). Everything must happen
    /// in microseconds without blocking.
    ///
    /// ## Why Stereo?
    ///
    /// The synthesizer generates true stereo internally through:
    /// - Voice panning and stereo positioning
    /// - Stereo effects (reverb, delay, chorus, auto-pan, stereo widener)
    /// - Spatial processing (mid/side, Haas effect)
    ///
    /// Returning stereo preserves this spatial information. Use `process_mono()` if you need
    /// mono output (it averages the channels).
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
    ///    - Each voice generates its own stereo sample
    ///    - Voices run independently, in parallel
    ///
    /// 3. **Mixing**:
    ///    - Add all voice outputs together (separately for left and right)
    ///    - Apply polyphonic gain compensation to prevent distortion
    ///
    /// 4. **Effects Chain**:
    ///    - Process through 16+ stereo effects (when enabled)
    ///    - Order is intentional: dynamics → distortion → modulation → reverb
    ///
    /// 5. **Master Processing**:
    ///    - Apply master gain
    ///    - Look-ahead limiter for transparent peak limiting
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
    /// let (left, right) = engine.process();
    ///
    /// // Verify stereo output is in valid range
    /// assert!(left.is_finite() && right.is_finite());
    /// assert!(left.abs() < 2.0 && right.abs() < 2.0);
    /// ```
    pub fn process(&mut self) -> (f32, f32) {
        self.maybe_update_params();

        // Mix all voices - stereo
        let mut output_left = 0.0;
        let mut output_right = 0.0;
        let mut active_count = 0;
        for voice in &mut self.voices {
            let (left, right) = voice.process(
                &self.current_params.oscillators,
                &self.current_params.filters,
                &self.current_params.lfos,
                &self.current_params.velocity,
                self.current_params.hard_sync_enabled,
                &self.current_params.voice_compressor,
                &self.current_params.transient_shaper,
            );
            if voice.is_active() {
                output_left += left;
                output_right += right;
                active_count += 1;
            }
        }

        // Polyphonic gain compensation: prevent distortion when many keys are pressed.
        // IMPORTANT: smooth changes in this gain. A step change when active_count changes
        // (e.g., pressing a second key) can be audible as a click.
        // Uses gentler exponent (0.35 vs 0.5) to maintain perceived loudness with limiter protection
        let target_poly_gain = if active_count > 1 {
            1.0 / (active_count as f32).powf(0.35)
        } else {
            1.0
        };

        if target_poly_gain < self.poly_gain {
            let coeff = self.poly_gain_attack_coeff;
            self.poly_gain = coeff * self.poly_gain + (1.0 - coeff) * target_poly_gain;
        } else {
            let coeff = self.poly_gain_release_coeff;
            self.poly_gain = coeff * self.poly_gain + (1.0 - coeff) * target_poly_gain;
        }

        output_left *= self.poly_gain;
        output_right *= self.poly_gain;

        // Apply master gain
        let master = self.current_params.master_gain;
        output_left *= master;
        output_right *= master;

        // Effects chain (processed in series)
        // Order is intentional for sound quality:
        // 1. Dynamics (compressor) - control peaks first
        // 2. Distortion/saturation (distortion, waveshaper, bitcrusher) - add harmonics
        // 3. Multiband distortion - frequency-specific saturation
        // 4. Harmonic enhancement (exciter) - frequency-specific harmonics after multiband
        // 5. Filter effects (comb filter, phaser, flanger) - frequency/phase manipulation
        // 6. Pitch modulation (ring modulator, tremolo) - amplitude/frequency effects
        // 7. Chorus - adds width/detuning
        // 8. Delay - rhythmic repeats
        // 9. Spatial effects (auto-pan, stereo widener) - stereo field manipulation
        // 10. Reverb last - final ambience/space
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
        if self.current_params.effects.exciter.enabled {
            (out_l, out_r) = self.exciter.process(out_l, out_r);
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

        // Look-ahead limiter for transparent peak limiting with minimal artifacts
        self.lookahead_limiter.process(out_l, out_r)
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
    /// let (left, right) = engine.process();
    /// assert!(left.abs() < 2.0 && right.abs() < 2.0, "Output should be in valid range");
    /// ```
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        // MIDI semantics: NoteOn with velocity 0 is equivalent to NoteOff.
        // Avoid activating a voice that should be silent.
        if velocity <= 0.0 {
            return;
        }
        if self.current_params.monophonic {
            // Monophonic mode: last-note priority.
            // If at least one key was already held, switching notes should be legato
            // (no hard DSP reset), otherwise a fast note change can click/pop.
            let had_held_note = !self.note_stack.is_empty();

            // Add/update note in stack.
            if let Some(existing) = self.note_stack.iter_mut().find(|(n, _)| *n == note) {
                existing.1 = velocity;
            } else {
                self.note_stack.push((note, velocity));
            }

            if had_held_note {
                self.voices[0].note_change_legato(note, velocity);
            } else {
                // No keys were held: treat this as a normal note-on (retrigger envelope).
                self.voices[0].note_on(note, velocity);
            }

            // Apply parameter-dependent frequency/timbre immediately.
            let lfo_params = self.get_tempo_synced_lfo_params();
            self.voices[0].update_parameters(
                &self.current_params.oscillators,
                &self.current_params.filters,
                &lfo_params,
                &self.current_params.envelope,
                &self.wavetable_library,
            );
        } else {
            // Polyphonic mode: original behavior
            // Get tempo-synced LFO params before borrowing voices
            let lfo_params = self.get_tempo_synced_lfo_params();

            // First, try to find an inactive voice
            if let Some(voice) = self.voices.iter_mut().find(|v| !v.is_active()) {
                voice.note_on(note, velocity);
                voice.update_parameters(
                    &self.current_params.oscillators,
                    &self.current_params.filters,
                    &lfo_params,
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
                &lfo_params,
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
    /// let (left, right) = engine.process();
    /// assert!(left.is_finite() && right.is_finite(), "Output should be finite");
    /// ```
    pub fn note_off(&mut self, note: u8) {
        if self.current_params.monophonic {
            // Monophonic mode: remove note from stack
            if let Some(pos) = self.note_stack.iter().position(|(n, _)| *n == note) {
                self.note_stack.remove(pos);
            }

            // If there are still notes in the stack, retrigger the most recent one
            if let Some(&(last_note, last_vel)) = self.note_stack.last() {
                // Last-note priority, legato: switch pitch without hard-resetting DSP.
                let lfo_params = self.get_tempo_synced_lfo_params();
                self.voices[0].note_change_legato(last_note, last_vel);
                self.voices[0].update_parameters(
                    &self.current_params.oscillators,
                    &self.current_params.filters,
                    &lfo_params,
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
                a.peak_amplitude()
                    .partial_cmp(&b.peak_amplitude())
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

    /// Set the current tempo from DAW transport (CLAP plugin only)
    ///
    /// This updates the internal tempo used for tempo-synced LFO and effect rates.
    /// When tempo_sync mode is not Hz, the rate parameter is converted to Hz based
    /// on the current tempo using musical divisions (1/4, 1/8T, etc.).
    ///
    /// # Arguments
    /// * `bpm` - Tempo in beats per minute (clamped to 20.0-999.0 for sanity)
    ///
    /// # Example
    /// ```
    /// use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
    /// let (_producer, consumer) = create_parameter_buffer();
    /// let mut engine = SynthEngine::new(44100.0, consumer);
    ///
    /// engine.set_tempo(140.0); // Set tempo to 140 BPM
    /// ```
    pub fn set_tempo(&mut self, bpm: f64) {
        self.current_tempo_bpm = bpm.clamp(20.0, 999.0);
    }

    /// Convert tempo sync mode to Hz based on current tempo
    ///
    /// This calculates the Hz rate for a given musical division at the current tempo.
    /// Results are clamped to 0.01-20 Hz to stay within valid LFO/effect rate ranges.
    ///
    /// Musical divisions:
    /// - Whole (1/1) = 4 beats per cycle
    /// - Half (1/2) = 2 beats per cycle
    /// - Quarter (1/4) = 1 beat per cycle
    /// - Eighth (1/8) = 0.5 beats per cycle
    /// - Sixteenth (1/16) = 0.25 beats per cycle
    /// - Triplets: multiply by 2/3 (e.g., 1/4T = 2/3 beat)
    /// - Dotted: multiply by 1.5 (e.g., 1/4D = 1.5 beats)
    ///
    /// # Arguments
    /// * `sync_mode` - The tempo sync mode (Hz, Quarter, EighthT, etc.)
    /// * `bpm` - Tempo in beats per minute
    ///
    /// # Returns
    /// Frequency in Hz, clamped to 0.01-20.0 Hz
    #[inline]
    fn tempo_division_to_hz(sync_mode: crate::params::TempoSync, bpm: f64) -> f32 {
        use crate::params::TempoSync;

        if sync_mode == TempoSync::Hz {
            return 0.0; // Signal to use raw Hz value
        }

        // Calculate beats per cycle based on musical division
        let beats_per_cycle = match sync_mode {
            TempoSync::Hz => return 0.0,
            TempoSync::Whole => 4.0,
            TempoSync::Half => 2.0,
            TempoSync::Quarter => 1.0,
            TempoSync::Eighth => 0.5,
            TempoSync::Sixteenth => 0.25,
            TempoSync::ThirtySecond => 0.125,
            TempoSync::QuarterT => 2.0 / 3.0, // 3 triplets per 2 beats
            TempoSync::EighthT => 1.0 / 3.0,  // 3 triplets per beat
            TempoSync::SixteenthT => 0.5 / 3.0, // 3 triplets per half beat
            TempoSync::QuarterD => 1.5,       // Dotted = 1.5× normal
            TempoSync::EighthD => 0.75,
            TempoSync::SixteenthD => 0.375,
        };

        // Convert BPM to cycles per second
        let beats_per_second = bpm / 60.0;
        let cycles_per_second = beats_per_second / beats_per_cycle;

        // Clamp to valid LFO/effect rate range (0.01 to 20 Hz)
        (cycles_per_second as f32).clamp(0.01, 20.0)
    }

    /// Get the effective rate for an LFO or effect, applying tempo sync if needed
    ///
    /// This helper calculates the actual Hz rate to use based on the tempo_sync mode.
    /// If sync mode changed from the previous call, it resets the phase to 0.0 for
    /// predictable musical timing.
    ///
    /// # Arguments
    /// * `hz_rate` - The raw Hz rate parameter (used when tempo_sync = Hz)
    /// * `tempo_sync` - The tempo sync mode (Hz, Quarter, EighthT, etc.)
    /// * `sync_index` - Index in previous_sync_modes array for this LFO/effect
    ///
    /// # Returns
    /// Effective rate in Hz (clamped to 0.01-20 Hz)
    #[inline]
    fn get_effective_rate(
        &mut self,
        hz_rate: f32,
        tempo_sync: crate::params::TempoSync,
        sync_index: usize,
    ) -> f32 {
        use crate::params::TempoSync;

        // Check if sync mode changed (for phase reset)
        let sync_mode_changed = self.previous_sync_modes[sync_index] != tempo_sync;
        if sync_mode_changed {
            self.previous_sync_modes[sync_index] = tempo_sync;

            // Reset phase when sync mode changes
            // This ensures predictable timing when switching sync modes
            match sync_index {
                3 => self.chorus.reset_phase(),   // Chorus
                4 => self.phaser.reset_phase(),   // Phaser
                5 => self.flanger.reset_phase(),  // Flanger
                6 => self.tremolo.reset_phase(),  // Tremolo
                7 => self.auto_pan.reset_phase(), // AutoPan
                _ => {}                           // LFOs handled separately in voice update
            }
        }

        // Calculate effective rate
        if tempo_sync == TempoSync::Hz {
            hz_rate // Use raw Hz value
        } else {
            Self::tempo_division_to_hz(tempo_sync, self.current_tempo_bpm)
        }
    }

    /// Get tempo-synced LFO parameters
    ///
    /// This helper applies tempo sync to LFO rates based on current tempo and sync modes.
    /// Used when triggering new notes to ensure they get the correct tempo-synced rates.
    ///
    /// # Returns
    /// Array of 3 LFO params with tempo-synced rates applied
    #[inline]
    fn get_tempo_synced_lfo_params(&self) -> [crate::params::LFOParams; 3] {
        use crate::params::TempoSync;

        let mut modified_lfos = self.current_params.lfos;
        for lfo_params in modified_lfos.iter_mut() {
            if lfo_params.tempo_sync != TempoSync::Hz {
                lfo_params.rate =
                    Self::tempo_division_to_hz(lfo_params.tempo_sync, self.current_tempo_bpm);
            }
        }
        modified_lfos
    }

    /// Process one sample and return mono output (averaged from stereo).
    ///
    /// This is a convenience method that averages the left and right channels into a single
    /// mono signal. Use this only if you truly need mono output (e.g., mono hardware, testing).
    ///
    /// **⚠️ Note**: This discards stereo spatial information from effects like reverb, chorus,
    /// auto-pan, and stereo widener. For most applications, use `process()` which returns
    /// true stereo.
    ///
    /// ## When to Use Mono
    ///
    /// - Output device is mono only (rare)
    /// - Testing/debugging where stereo doesn't matter
    /// - Specific mixing scenarios requiring mono sum
    ///
    /// # Returns
    /// A single mono audio sample (-1.0 to +1.0), averaged from left and right channels
    ///
    /// # Example
    /// ```
    /// use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
    /// let (_producer, consumer) = create_parameter_buffer();
    /// let mut engine = SynthEngine::new(44100.0, consumer);
    ///
    /// engine.note_on(60, 0.8);
    /// let mono_sample = engine.process_mono();
    ///
    /// assert!(mono_sample.is_finite());
    /// assert!(mono_sample.abs() < 2.0);
    /// ```
    pub fn process_mono(&mut self) -> f32 {
        let (left, right) = self.process();
        (left + right) / 2.0
    }

    /// Process a block of stereo audio samples efficiently.
    ///
    /// **This is the preferred method for CLAP plugins** and any code that processes audio
    /// in blocks rather than individual samples. It's more efficient than calling `process()`
    /// in a loop because:
    /// - Loop overhead is eliminated
    /// - Parameter throttling (every 32 samples) is amortized across the block
    /// - Can be optimized more aggressively by the compiler (SIMD vectorization, loop unrolling)
    ///
    /// DAW hosts typically call this with blocks of 256-4096 samples per buffer, so it's
    /// critical that this method is fast.
    ///
    /// True stereo output is preserved: left and right channels are generated independently
    /// by the synthesizer's panning, stereo effects, and spatial processing.
    ///
    /// # Arguments
    /// * `left` - Output buffer for left channel (will be filled with samples)
    /// * `right` - Output buffer for right channel (will be filled with samples)
    ///
    /// # Panics
    /// Does not panic. If buffers have different lengths, processes up to the shorter length.
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
    /// ```
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());

        for i in 0..len {
            let (l, r) = self.process();
            left[i] = l;
            right[i] = r;
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
