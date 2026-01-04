//! Single voice implementation for polyphonic synthesis.
//!
//! A Voice represents one "note" being played by the synthesizer. In a polyphonic synthesizer,
//! multiple voices can play simultaneously (DSynth has 16 pre-allocated voices in the engine).
//! Each voice is a complete mini-synthesizer with its own oscillators, filters, envelopes, and LFOs.
//!
//! # Voice Architecture: The Building Blocks
//!
//! A single Voice contains:
//! - **3 oscillators** (each with up to 7 unison voices for "supersaw" thick sounds)
//! - **3 biquad filters** (one per oscillator, with modulation)
//! - **1 ADSR envelope** (controls amplitude over time: attack → decay → sustain → release)
//! - **3 LFOs** (one per filter, for modulation like vibrato or wah effects)
//! - **Voice stealing metrics** (peak amplitude tracking to identify the "quietest" voice)
//!
//! # Voice Lifecycle: Birth, Life, Death
//!
//! Voices follow a state machine:
//! 1. **Idle** → Voice is not active, waiting to be assigned a note
//! 2. **Note On** → `note_on()` is called with MIDI note + velocity, envelope starts attack phase
//! 3. **Sustain** → Envelope reaches sustain level, voice produces continuous audio
//! 4. **Note Off** → `note_off()` is called, envelope enters release phase (fade out over ~1 second)
//! 5. **Dead** → Envelope finishes release, voice returns to idle state
//!
//! The engine reuses voices: when all 16 voices are busy and a new note arrives, the engine
//! "steals" the quietest voice (determined by `get_rms()`) and reassigns it to the new note.
//!
//! # Unison: Multiple Oscillators for Thickness
//!
//! Each of the 3 oscillator slots can have multiple "unison voices" (up to 7) playing slightly
//! detuned copies of the same waveform. This creates a thick, chorus-like "supersaw" effect
//! common in EDM and trance music.
//!
//! Example: Oscillator 1 with 7 unison voices generates 7 copies of the waveform, each with
//! slightly different frequency (±detune cents) and phase offset. These are mixed together
//! and normalized to prevent clipping.
//!
//! # Real-Time Optimization: Pre-Allocation
//!
//! All oscillators, filters, and LFOs are **pre-allocated** at Voice creation (no runtime
//! allocations). The `oscillators` field is a fixed-size 2D array: `[[Option<Oscillator>; 7]; 3]`.
//! We change the `active_unison[i]` count to enable/disable oscillators without allocating.
//!
//! This design ensures predictable performance in the audio callback—no allocations, no heap
//! fragmentation, just constant-time processing.

use crate::dsp::effects::compressor::Compressor;
use crate::dsp::{envelope::Envelope, filter::BiquadFilter, lfo::LFO, oscillator::Oscillator};
use crate::params::{
    EnvelopeParams, FilterParams, LFOParams, OscillatorParams, TransientShaperParams,
    VelocityParams, VoiceCompressorParams,
};

/// Maximum number of unison voices per oscillator slot.
///
/// This is a compile-time constant to enable pre-allocation of all oscillators. Seven unison
/// voices is typical for "supersaw" sounds—enough for thick chorus effects without excessive CPU.
/// The engine can use fewer (1-7) by changing the `active_unison` count, but memory for all 7
/// is always allocated per slot.
const MAX_UNISON_VOICES: usize = 7;

/// A single polyphonic voice combining oscillators, filters, envelopes, and LFOs.
///
/// This struct represents one "note" in a polyphonic synthesizer. The engine pre-allocates
/// 16 of these voices and assigns them to incoming MIDI notes. Each voice is a complete
/// mini-synthesizer with independent DSP components.
///
/// # Architecture
///
/// - **3 oscillator slots** (each with up to 7 unison voices): Generate raw waveforms
/// - **3 biquad filters**: Shape the oscillator output (lowpass, highpass, bandpass)
/// - **1 ADSR envelope**: Controls amplitude over time (attack → decay → sustain → release)
/// - **3 LFOs**: Modulate filter cutoff for effects like vibrato or auto-wah
/// - **Voice stealing metrics**: Track peak amplitude to identify the quietest voice
///
/// # Why Pre-Allocated Arrays?
///
/// The `oscillators` field is `[[Option<Oscillator>; 7]; 3]`—a fixed-size 2D array.
/// This avoids runtime allocations in the audio thread. We always allocate memory for
/// 7 unison voices per slot, but use `active_unison[i]` to control how many are actually
/// processed. This trades memory (always allocating max) for real-time safety (no allocations).
///
/// # Voice Stealing
///
/// When all 16 voices are busy and a new note arrives, the engine must "steal" a voice.
/// The `get_rms()` method returns the peak amplitude seen since note-on, allowing the engine
/// to steal the quietest voice. This minimizes audible artifacts (you won't notice a quiet
/// voice being stolen, but you'd hear a loud one suddenly cut off).
pub struct Voice {
    /// MIDI note number (0-127) assigned to this voice.
    /// Middle C (C4) is 60, A4 (concert pitch) is 69.
    note: u8,

    /// Note velocity (0.0-1.0) indicating how hard the key was pressed.
    /// Used for velocity-sensitive amplitude and filter cutoff modulation.
    velocity: f32,

    /// Whether this voice is currently producing sound.
    /// Becomes false when the envelope finishes its release phase.
    is_active: bool,

    /// Sample rate in Hz (e.g., 44100.0, 48000.0).
    /// Stored for potential future use; currently unused because DSP components
    /// are initialized with sample rate in the constructor.
    #[allow(dead_code)]
    sample_rate: f32,

    /// 2D array of oscillators: 3 slots × 7 unison voices per slot.
    ///
    /// Each slot can have 1-7 active unison voices controlled by `active_unison[i]`.
    /// Using `Option<Oscillator>` allows us to pre-allocate all 21 oscillators (3×7)
    /// and selectively process only the active ones. This avoids allocations in the
    /// audio thread while supporting dynamic unison count changes.
    ///
    /// Example: If `active_unison[0] = 3`, we process `oscillators[0][0..3]`.
    oscillators: [[Option<Oscillator>; MAX_UNISON_VOICES]; 3],

    /// Number of active unison voices for each oscillator slot (1-7).
    ///
    /// Controls how many oscillators in each slot are processed. Setting this to 1
    /// produces a single oscillator (no unison effect). Setting to 7 creates a thick
    /// "supersaw" sound with 7 detuned copies.
    active_unison: [usize; 3],

    /// Three biquad filters, one per oscillator slot.
    ///
    /// Each filter shapes its oscillator's output using lowpass, highpass, or bandpass
    /// filtering. The filter cutoff is modulated by LFOs, key tracking, and velocity.
    filters: [BiquadFilter; 3],

    /// ADSR envelope controlling the voice's amplitude over time.
    ///
    /// - **Attack**: Fade in from silence to full volume (typically 10-100ms)
    /// - **Decay**: Drop from peak to sustain level (typically 100-500ms)
    /// - **Sustain**: Hold at a constant level while the key is held (0.0-1.0)
    /// - **Release**: Fade out after key release (typically 500-2000ms)
    ///
    /// The envelope value (0.0-1.0) is multiplied with the audio output.
    envelope: Envelope,

    /// Per-filter ADSR envelopes.
    filter_envelopes: [Envelope; 3],

    /// Three LFOs (Low Frequency Oscillators), one per filter.
    ///
    /// LFOs generate slow-moving waveforms (typically <20 Hz) that modulate filter cutoff.
    /// This creates effects like vibrato (modulating pitch), tremolo (modulating amplitude),
    /// or auto-wah (modulating filter cutoff).
    lfos: [LFO; 3],

    /// Per-voice compressor for transient control.
    ///
    /// Catches transients before they hit the master mix, providing dynamic control
    /// at the individual voice level. Uses optimized mono compression with envelope
    /// follower updating every 4 samples for CPU efficiency.
    voice_compressor: Compressor,

    /// Previous phase of oscillator 1 (for hard sync chain detection).
    /// When Osc1's phase wraps from >1.0 to <1.0, we reset Osc2's phase to create
    /// the bright, aggressive harmonics characteristic of hard sync.
    osc1_phase_prev: f32,

    /// Previous phase of oscillator 2 (for hard sync chain detection).
    /// When Osc2's phase wraps, we reset Osc3's phase (sync chain: OSC1→OSC2→OSC3).
    osc2_phase_prev: f32,

    /// RMS (Root Mean Square) squared exponential moving average.
    ///
    /// **Currently unused** in favor of peak amplitude tracking for performance.
    /// Historical note: This was originally used for voice stealing but was replaced
    /// with `peak_amplitude` because RMS calculation costs 3-5% CPU per voice, whereas
    /// peak tracking costs <1%.
    rms_squared_ema: f32,

    /// Peak amplitude seen since the last note-on event.
    ///
    /// Used for voice stealing: the engine steals the voice with the lowest peak amplitude.
    /// This is much cheaper than RMS tracking (1-2% CPU vs 3-5%) and sufficient for
    /// identifying quiet voices. Reset to 0.0 on every note-on.
    peak_amplitude: f32,

    /// Last output sample (mono average of left and right channels).
    ///
    /// Stored for potential future use (e.g., detecting voice decay rate).
    /// Currently not used in voice stealing or processing logic.
    last_output: f32,

    /// Last *final* output sample after all processing (including anti-click/crossfade).
    ///
    /// Stored so we can avoid discontinuities when retriggering a note while the voice is
    /// still audible (e.g., fast repeated taps causing rapid release→attack restarts).
    last_output_left: f32,
    last_output_right: f32,

    /// One-shot flag: after note_on, we need to reset oscillator/filter running state
    /// once parameters (including unison phase offsets) have been applied.
    needs_dsp_reset_on_update: bool,

    /// Previous sample's oscillator outputs (for feedback FM).
    ///
    /// Stores the output of each oscillator from the previous sample, allowing
    /// any oscillator to use any other as a modulation source (including later
    /// oscillators modulating earlier ones via 1-sample delayed feedback).
    osc_outputs_prev: [f32; 3],

    /// Anti-click fade-in sample counter.
    ///
    /// Counts samples since note-on to apply a short exponential fade-in (1-2ms)
    /// at the start of every note. This prevents clicks/pops when voices are stolen
    /// while still producing sound. The fade is independent of envelope attack and
    /// ensures smooth transitions during voice stealing.
    ///
    /// Reset to 0 on note_on(), increments every sample during process().
    anti_click_samples: usize,

    /// Anti-click fade-in duration in samples (2ms at 44.1kHz = 88 samples).
    ///
    /// This value is calculated as: 0.002 * sample_rate
    anti_click_fade_samples: usize,

    /// Retrigger crossfade samples remaining.
    ///
    /// When a note is retriggered while the voice still has audible output (typically during
    /// the release tail), restarting the anti-click fade can cause a hard step (click):
    /// previous sample is non-zero, next sample starts near zero.
    ///
    /// We fix that by crossfading from the last final output into the new restarted output over
    /// a very short window.
    retrigger_xfade_samples_remaining: usize,
    retrigger_xfade_total_samples: usize,
    retrigger_prev_left: f32,
    retrigger_prev_right: f32,

    /// Remaining samples for mono legato de-click smoothing.
    ///
    /// In monophonic mode, we can switch notes without retriggering the voice (legato).
    /// A sudden note change can still cause a small click due to abrupt filter coefficient
    /// changes from key tracking / velocity modulation. We apply a very short cutoff smoothing
    /// window on legato note changes to keep the signal continuous.
    mono_declick_samples_remaining: usize,

    /// Total duration (in samples) of the mono legato de-click smoothing window.
    mono_declick_total_samples: usize,

    /// One-pole smoothing coefficient used during the mono de-click window.
    mono_declick_cutoff_coeff: f32,

    /// Per-filter smoothed cutoff frequency (Hz) used during mono legato transitions.
    mono_smoothed_cutoff_hz: [f32; 3],

    /// Last MIDI note that oscillator-dependent parameters were applied for.
    ///
    /// Used to avoid recomputing oscillator frequency/unison state when only non-osc
    /// parameters (e.g., filter cutoff) change.
    last_applied_note: u8,

    /// Cached last-applied oscillator parameters.
    last_applied_osc_params: [OscillatorParams; 3],

    /// Cached last-applied filter parameters.
    last_applied_filter_params: [FilterParams; 3],

    /// Cached last-applied LFO parameters.
    last_applied_lfo_params: [LFOParams; 3],

    /// Cached last-applied envelope parameters.
    last_applied_envelope_params: EnvelopeParams,

    /// Cached last-applied voice compressor parameters.
    ///
    /// Used to avoid reconfiguring the compressor every sample when params are unchanged.
    last_applied_voice_comp_params: VoiceCompressorParams,

    /// Cached per-oscillator base frequency (Hz) after applying note + pitch + detune.
    ///
    /// Used to avoid recomputing expensive `powf` work in the per-sample hot path when
    /// pitch modulation is active.
    osc_base_freq_hz: [f32; 3],

    /// Cached unison detune multipliers for each oscillator slot.
    ///
    /// Indexed by `[osc_slot][unison_idx]`.
    unison_detune_mul: [[f32; MAX_UNISON_VOICES]; 3],

    /// Cached key-tracking cutoff multiplier per filter.
    ///
    /// Key tracking depends only on the current note and the `key_tracking` amount, so we
    /// cache it to avoid a per-sample `powf` in the filter cutoff modulation path.
    filter_key_tracking_mul: [f32; 3],

    /// Whether any LFO is actively modulating pan.
    ///
    /// When false, we can cache per-oscillator pan gains and avoid per-sample sin/cos.
    pan_mod_active: bool,

    /// Cached equal-power panning gains (only used when `pan_mod_active` is false).
    cached_pan_left_gain: [f32; 3],
    cached_pan_right_gain: [f32; 3],
}

impl Voice {
    /// Create a new voice with all DSP components pre-allocated.
    ///
    /// This constructor allocates all memory up front (21 oscillators = 3 slots × 7 unison voices,
    /// 3 filters, 1 envelope, 3 LFOs) to ensure the audio thread never allocates during processing.
    /// The voice starts in an inactive state and waits for a `note_on()` call to begin producing sound.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0, 48000.0). Passed to all DSP components
    ///   so they can calculate frequency-dependent coefficients (e.g., filter poles, envelope rates).
    ///
    /// # Memory Allocation Strategy
    ///
    /// - All 21 oscillators (3×7) are allocated as `Some(Oscillator::new(sample_rate))`
    /// - Filters, envelope, and LFOs are allocated as structs
    /// - `active_unison` starts at `[1, 1, 1]` (one active oscillator per slot)
    /// - Total allocation: ~1-2 KB per voice, done once at engine initialization
    ///
    /// # Why Pre-Allocate All 7 Unison Voices?
    ///
    /// We could use `Vec<Oscillator>` and grow it dynamically, but that would allocate in the
    /// audio thread when increasing unison count. By pre-allocating all 7, we trade memory
    /// (~few KB) for real-time safety (zero allocations after construction).
    pub fn new(sample_rate: f32) -> Self {
        // Step 1: Pre-allocate all 21 oscillators (3 slots × 7 unison voices)
        // Start with a default array of None values
        let mut oscillators: [[Option<Oscillator>; MAX_UNISON_VOICES]; 3] = Default::default();

        // Step 2: Initialize all oscillator slots with actual Oscillator instances
        // This ensures no allocations happen when changing unison count—we just
        // enable/disable processing of pre-existing oscillators.
        for osc_slot in &mut oscillators {
            for osc_ref in osc_slot.iter_mut() {
                *osc_ref = Some(Oscillator::new(sample_rate));
            }
        }

        Self {
            note: 0,
            velocity: 0.0,
            is_active: false,
            sample_rate,
            oscillators,
            active_unison: [1, 1, 1], // Initially all slots have 1 active oscillator
            filters: [
                BiquadFilter::new(sample_rate),
                BiquadFilter::new(sample_rate),
                BiquadFilter::new(sample_rate),
            ],
            envelope: Envelope::new(sample_rate),
            filter_envelopes: [
                Envelope::new(sample_rate),
                Envelope::new(sample_rate),
                Envelope::new(sample_rate),
            ],
            lfos: [
                LFO::new(sample_rate),
                LFO::new(sample_rate),
                LFO::new(sample_rate),
            ],
            voice_compressor: Compressor::new(
                sample_rate,
                -12.0, // threshold_db: Higher for catching transients
                3.0,   // ratio: Moderate compression
                1.0,   // attack_ms: Fast for transient control
                50.0,  // release_ms: Quick release to avoid pumping
            ),
            rms_squared_ema: 0.0,
            peak_amplitude: 0.0,
            last_output: 0.0,
            last_output_left: 0.0,
            last_output_right: 0.0,
            needs_dsp_reset_on_update: false,
            osc_outputs_prev: [0.0; 3],
            osc1_phase_prev: 0.0, // Initialize hard sync phase tracking
            osc2_phase_prev: 0.0, // Initialize hard sync chain tracking
            anti_click_samples: 0,
            anti_click_fade_samples: (0.002 * sample_rate) as usize, // 2ms fade

            retrigger_xfade_samples_remaining: 0,
            retrigger_xfade_total_samples: (0.0015 * sample_rate) as usize, // 1.5ms
            retrigger_prev_left: 0.0,
            retrigger_prev_right: 0.0,

            // Mono legato de-click smoothing (very short; intended to remove clicks without audible portamento).
            mono_declick_samples_remaining: 0,
            mono_declick_total_samples: (0.0015 * sample_rate) as usize, // 1.5ms
            mono_declick_cutoff_coeff: (-1.0 / (0.0005 * sample_rate)).exp(), // 0.5ms time constant
            mono_smoothed_cutoff_hz: [0.0; 3],

            // Parameter caching for incremental updates.
            last_applied_note: 0,
            last_applied_osc_params: [OscillatorParams::default(); 3],
            last_applied_filter_params: [FilterParams::default(); 3],
            last_applied_lfo_params: [LFOParams::default(); 3],
            last_applied_envelope_params: EnvelopeParams::default(),
            last_applied_voice_comp_params: VoiceCompressorParams::default(),

            osc_base_freq_hz: [0.0; 3],
            unison_detune_mul: [[1.0; MAX_UNISON_VOICES]; 3],

            filter_key_tracking_mul: [1.0; 3],

            pan_mod_active: false,
            cached_pan_left_gain: [std::f32::consts::FRAC_1_SQRT_2; 3],
            cached_pan_right_gain: [std::f32::consts::FRAC_1_SQRT_2; 3],
        }
    }

    /// Trigger a note-on event, starting this voice's sound production.
    ///
    /// This method assigns a MIDI note and velocity to the voice, activates it, and triggers
    /// the ADSR envelope's attack phase. The voice will begin producing audio on the next
    /// `process()` call. If the voice was previously active (e.g., stolen from another note),
    /// this resets it completely.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127). Middle C (C4) = 60, A4 (concert pitch) = 69.
    ///   Converted to frequency using the formula: `f = 440 * 2^((note - 69) / 12)`
    /// * `velocity` - Note velocity (0.0-1.0) representing how hard the key was pressed.
    ///   Clamped to [0.0, 1.0] to handle out-of-range values gracefully. Used for velocity-sensitive
    ///   amplitude scaling and filter cutoff modulation.
    ///
    /// # State Changes
    ///
    /// - `is_active` set to `true`
    /// - `note` and `velocity` stored
    /// - Envelope enters attack phase (fade in to full volume)
    /// - LFOs reset to phase 0.0 (start from the beginning of their waveform)
    /// - Peak amplitude and RMS metrics reset to 0.0 (for voice stealing)
    ///
    /// # Example
    ///
    /// ```ignore
    /// voice.note_on(60, 0.8);  // Play middle C at 80% velocity
    /// voice.note_on(72, 1.0);  // Play C5 at 100% velocity (forte)
    /// ```
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        // If we're retriggering while still audible (e.g., fast repeated taps),
        // capture the last output so we can crossfade and avoid a step discontinuity.
        let was_active = self.is_active;
        if was_active {
            let prev_peak = self
                .last_output_left
                .abs()
                .max(self.last_output_right.abs());
            if prev_peak > 1.0e-4 {
                self.retrigger_prev_left = self.last_output_left;
                self.retrigger_prev_right = self.last_output_right;
                self.retrigger_xfade_samples_remaining = self.retrigger_xfade_total_samples;
            }
        }

        self.note = note;
        // Clamp velocity to valid range [0.0, 1.0] to handle edge cases
        // (e.g., MIDI controller sending out-of-spec values)
        self.velocity = velocity.clamp(0.0, 1.0);
        self.is_active = true;

        // Reset peak amplitude and output tracking for the new note
        // These are used for voice stealing—we want to measure this note's loudness,
        // not carry over metrics from the previous note.
        self.peak_amplitude = 0.0;
        self.last_output = 0.0;

        // Trigger the ADSR envelope's attack phase
        // The envelope will fade in from 0.0 to 1.0 over the attack time (typically 10-100ms)
        self.envelope.note_on();

        // Trigger filter envelopes
        for env in &mut self.filter_envelopes {
            env.note_on();
        }

        // DO NOT reset LFOs! Resetting LFO phase to 0 causes sudden jumps in modulation
        // (filter cutoff, gain, pan, pitch), creating audible discontinuities.
        // LFOs run continuously across note boundaries to avoid clicks.

        // Reset RMS tracking
        // Although we use peak amplitude for voice stealing, we reset RMS for consistency
        // (it's a historical artifact from when RMS was used for voice stealing).
        self.rms_squared_ema = 0.0;

        // CRITICAL: Reset envelope levels to zero BEFORE triggering attack.
        // If a voice is stolen while in release phase (e.g., current_level = 0.3),
        // the new note would start attacking from 0.3 instead of 0.0, causing a
        // sudden amplitude jump (click/pop). By resetting to 0.0 first, we ensure
        // all notes start from silence and attack cleanly.
        self.envelope.reset_level();
        for env in &mut self.filter_envelopes {
            env.reset_level();
        }

        // DO NOT reset LFOs! Resetting LFO phase to 0 causes sudden jumps in modulation
        // (filter cutoff, gain, pan, pitch), creating audible discontinuities.
        // LFOs should run continuously across note boundaries to avoid clicks.
        // Comment out: for lfo in &mut self.lfos { lfo.reset(); }

        // Reset hard sync phase tracking.
        // These track the previous phase of oscillators for hard sync detection.
        // If not reset, stale phase values from the previous note can cause incorrect
        // sync triggers at the start of the new note.
        self.osc1_phase_prev = 0.0;
        self.osc2_phase_prev = 0.0;

        // Reset FM feedback buffer.
        // This stores previous sample outputs for feedback FM synthesis.
        // Old values would cause discontinuities in FM modulation.
        self.osc_outputs_prev = [0.0; 3];

        // Reset anti-click fade counter to trigger fade-in for this note.
        // This ensures a smooth 2ms fade-in at the start of every note,
        // preventing clicks when voices are stolen while still producing sound.
        self.anti_click_samples = 0;

        // Any fresh note-on should start from a clean cutoff state.
        self.mono_declick_samples_remaining = 0;
        self.mono_smoothed_cutoff_hz = [0.0; 3];

        // Reset all DSP components IMMEDIATELY to prevent clicks from stale state.
        // This must happen in note_on() rather than being deferred to update_parameters()
        // because process() may be called multiple times before update_parameters() is called,
        // and those process() calls would use old oscillator/filter/compressor samples,
        // causing audible discontinuities (clicks/pops).

        // Reset oscillator buffers WITHOUT resetting phase to avoid discontinuities.
        // Phase discontinuities (jumping from 0.7 → 0.0) create audible clicks even with
        // anti-click fades. By only clearing the downsampler buffers and letting phase
        // continue, we maintain waveform continuity while still removing stale samples.
        for osc_slot in &mut self.oscillators {
            for osc_opt in osc_slot.iter_mut() {
                if let Some(osc) = osc_opt {
                    osc.reset_buffers();
                }
            }
        }

        // Reset all filters (clears delay lines: x1, x2, y1, y2)
        for filter in &mut self.filters {
            filter.reset();
        }

        // Reset per-voice compressor (clears envelope follower state)
        self.voice_compressor.reset();

        // Flag that update_parameters() should reapply phase offsets.
        // Even though we don't reset phase (to avoid discontinuities), we still need
        // update_parameters() to apply frequency and other parameter changes on the first
        // call after note_on().
        self.needs_dsp_reset_on_update = true;
    }

    /// Change the currently-playing note without hard-resetting DSP state.
    ///
    /// This is used for monophonic legato (last-note priority) when multiple keys are held.
    /// Unlike `note_on()`, this does **not** reset envelope levels, filters, compressors, or
    /// oscillator phase/buffers. That avoids a discontinuity (click/pop) that would otherwise
    /// occur when switching notes while the voice is already producing sound.
    pub fn note_change_legato(&mut self, note: u8, velocity: f32) {
        self.note = note;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.is_active = true;
        self.mono_declick_samples_remaining = self.mono_declick_total_samples;
        // Intentionally do not touch envelopes/LFOs/filters/anti-click state.
        // Frequency changes are applied immediately by the caller via update_parameters().
    }

    /// Trigger a note-off event, starting this voice's release phase.
    ///
    /// This method does NOT immediately silence the voice. Instead, it triggers the ADSR
    /// envelope's release phase, which fades the voice out over the release time (typically
    /// 500-2000ms). The voice remains active (`is_active == true`) during the release and
    /// only becomes inactive when the envelope fully completes.
    ///
    /// # Sustain Pedal Behavior
    ///
    /// If a MIDI sustain pedal is held, the DAW/MIDI handler should delay calling `note_off()`
    /// until the pedal is released. This implementation assumes the pedal logic is handled
    /// upstream (in the MIDI handler or engine), not in the voice itself.
    ///
    /// # Why Keep the Voice Active During Release?
    ///
    /// The voice continues processing audio during the release phase to produce a smooth fade-out.
    /// If we immediately set `is_active = false`, the sound would cut off abruptly (a "click"
    /// artifact). The envelope's `is_active()` method returns `false` when the release completes,
    /// at which point `process()` sets `is_active = false` and returns (0.0, 0.0).
    pub fn note_off(&mut self) {
        // Transition the envelope from sustain → release
        // The envelope will fade from its current level to 0.0 over the release time
        self.envelope.note_off();

        // Release filter envelopes
        for env in &mut self.filter_envelopes {
            env.note_off();
        }
    }

    /// Update all oscillator, filter, and LFO parameters for this voice.
    ///
    /// This method is called by the engine when parameters change (via the GUI or DAW automation).
    /// It reconfigures all DSP components without interrupting audio processing. The engine
    /// throttles parameter updates (every 32 samples ≈ 0.7ms) to reduce CPU overhead.
    ///
    /// # Arguments
    ///
    /// * `osc_params` - Oscillator parameters (waveform, pitch, detune, unison, etc.)
    /// * `filter_params` - Filter parameters (cutoff, resonance, type, key tracking, etc.)
    /// * `lfo_params` - LFO parameters (rate, waveform, depth, modulation amounts)
    /// * `envelope_params` - ADSR envelope parameters (attack, decay, sustain, release)
    ///
    /// # What Gets Updated
    ///
    /// For each of the 3 oscillator/filter/LFO groups:
    /// 1. **Oscillator frequency**: Calculated from MIDI note + pitch + detune + unison spread
    /// 2. **Oscillator waveform and shape**: Sine, square, saw, triangle, pulse (with morphing)
    /// 3. **Unison voices**: Active count (1-7) and per-voice detune/phase spread
    /// 4. **Filter type and base cutoff**: Lowpass, highpass, bandpass with resonance
    /// 5. **LFO rate and waveform**: Controls modulation speed and shape
    /// 6. **Envelope ADSR**: Attack, decay, sustain, and release times
    ///
    /// # Frequency Calculation Pipeline
    ///
    /// The final oscillator frequency is calculated as:
    /// ```ignore
    /// base_freq = 440 * 2^((note - 69) / 12)  // MIDI note to Hz
    /// pitch_mult = 2^(pitch / 12)             // Pitch shift in semitones
    /// detune_mult = 2^(detune / 1200)         // Fine detune in cents
    /// unison_detune = 2^(spread * offset / 12) // Per-voice unison spread
    /// final_freq = base_freq * pitch_mult * detune_mult * unison_detune
    /// ```
    ///
    /// # Why Not Update Filter Cutoff Here?
    ///
    /// We set the **base cutoff** here, but the actual cutoff is modulated per-sample in
    /// `process()` by LFOs, key tracking, and velocity. This allows dynamic modulation
    /// without calling `update_parameters()` every sample.
    pub fn update_parameters(
        &mut self,
        osc_params: &[OscillatorParams; 3],
        filter_params: &[FilterParams; 3],
        lfo_params: &[LFOParams; 3],
        envelope_params: &EnvelopeParams,
        wavetable_library: &crate::dsp::wavetable_library::WavetableLibrary,
    ) {
        let osc_params_changed = *osc_params != self.last_applied_osc_params;
        let filter_params_changed = *filter_params != self.last_applied_filter_params;
        let lfo_params_changed = *lfo_params != self.last_applied_lfo_params;
        let envelope_params_changed = *envelope_params != self.last_applied_envelope_params;
        let note_changed = self.note != self.last_applied_note;

        // If note_on() reset buffers, we must re-apply oscillator state at least once,
        // even if note/params match the previous note.
        let needs_osc_update = self.needs_dsp_reset_on_update || note_changed || osc_params_changed;
        let needs_filter_update = filter_params_changed;
        let needs_lfo_update = lfo_params_changed;
        let needs_envelope_update = envelope_params_changed;

        if !(needs_osc_update || needs_filter_update || needs_lfo_update || needs_envelope_update) {
            return;
        }

        let base_freq = if needs_osc_update {
            Self::midi_note_to_freq(self.note)
        } else {
            0.0
        };

        // Key tracking depends on note and filter key_tracking amount.
        // Update it whenever either the note changes or filter params change.
        if needs_filter_update || note_changed {
            let note_offset = self.note as f32 - 60.0; // C4 reference
            for i in 0..3 {
                let key_tracking = filter_params[i].key_tracking.clamp(0.0, 1.0);
                // Matches the previous per-sample formula:
                // cutoff_mult = 2^((note_offset * 100cents * key_tracking) / 1200)
                //            = 2^((note_offset * key_tracking) / 12)
                self.filter_key_tracking_mul[i] = 2.0_f32.powf((note_offset * key_tracking) / 12.0);
            }
        }

        for i in 0..3 {
            if needs_osc_update {
                let param = &osc_params[i];

                let target_unison = param.unison.clamp(1, MAX_UNISON_VOICES);
                self.active_unison[i] = target_unison;

                let pitch_mult = 2.0_f32.powf(param.pitch / 12.0);
                let detune_mult = 2.0_f32.powf(param.detune / 1200.0);
                let base_osc_freq = base_freq * pitch_mult * detune_mult;
                self.osc_base_freq_hz[i] = base_osc_freq;

                for unison_idx in 0..target_unison {
                    if let Some(ref mut osc) = self.oscillators[i][unison_idx] {
                        let unison_detune = if target_unison > 1 {
                            let spread = param.unison_detune / 100.0;
                            let offset = (unison_idx as f32 - (target_unison - 1) as f32 / 2.0)
                                / (target_unison - 1) as f32;
                            2.0_f32.powf(offset * spread / 12.0)
                        } else {
                            1.0
                        };

                        self.unison_detune_mul[i][unison_idx] = unison_detune;

                        let freq = base_osc_freq * unison_detune;
                        osc.set_frequency(freq);
                        osc.set_waveform(param.waveform);
                        osc.set_shape(param.shape);

                        if param.waveform == crate::params::Waveform::Additive {
                            osc.set_additive_harmonics(param.additive_harmonics);
                        }

                        if param.waveform == crate::params::Waveform::Wavetable {
                            osc.set_wavetable(param.wavetable_index, wavetable_library);
                            osc.set_wavetable_position(param.wavetable_position);
                        }

                        const GOLDEN_FRACTION: f32 = 0.618_033_95;
                        let phase_offset =
                            (param.phase + (unison_idx as f32) * GOLDEN_FRACTION) % 1.0;
                        osc.set_phase(phase_offset);

                        if self.needs_dsp_reset_on_update {
                            osc.apply_initial_phase();
                        }
                    }
                }
            }

            if needs_filter_update {
                self.filters[i].set_filter_type(filter_params[i].filter_type);
                self.filters[i].set_resonance(filter_params[i].resonance);
                self.filters[i].set_bandwidth(filter_params[i].bandwidth);

                self.filter_envelopes[i].set_attack(filter_params[i].envelope.attack);
                self.filter_envelopes[i].set_decay(filter_params[i].envelope.decay);
                self.filter_envelopes[i].set_sustain(filter_params[i].envelope.sustain);
                self.filter_envelopes[i].set_release(filter_params[i].envelope.release);
            }

            if needs_lfo_update {
                let lfo = &lfo_params[i];
                self.lfos[i].set_rate(lfo.rate);
                self.lfos[i].set_waveform(lfo.waveform);
            }
        }

        // Cache pan gains when pan modulation is not active.
        // This removes per-sample sin/cos calls in the voice hot path for the common case.
        if needs_lfo_update || osc_params_changed {
            let new_pan_mod_active = lfo_params
                .iter()
                .any(|p| (p.pan_amount * p.depth).abs() > 0.001);
            let pan_mod_just_disabled = self.pan_mod_active && !new_pan_mod_active;

            self.pan_mod_active = new_pan_mod_active;

            if !self.pan_mod_active && (osc_params_changed || pan_mod_just_disabled) {
                for i in 0..3 {
                    let pan = osc_params[i].pan.clamp(-1.0, 1.0);
                    let pan_radians = (pan + 1.0) * std::f32::consts::PI / 4.0;
                    self.cached_pan_left_gain[i] = pan_radians.cos();
                    self.cached_pan_right_gain[i] = pan_radians.sin();
                }
            }
        }

        if needs_envelope_update {
            self.envelope.set_attack(envelope_params.attack);
            self.envelope.set_decay(envelope_params.decay);
            self.envelope.set_sustain(envelope_params.sustain);
            self.envelope.set_release(envelope_params.release);
            self.envelope.set_attack_curve(envelope_params.attack_curve);
            self.envelope.set_decay_curve(envelope_params.decay_curve);
            self.envelope.set_release_curve(envelope_params.release_curve);
        }

        if osc_params_changed {
            self.last_applied_osc_params = *osc_params;
        }
        if filter_params_changed {
            self.last_applied_filter_params = *filter_params;
        }
        if lfo_params_changed {
            self.last_applied_lfo_params = *lfo_params;
        }
        if envelope_params_changed {
            self.last_applied_envelope_params = *envelope_params;
        }
        if needs_osc_update {
            self.last_applied_note = self.note;
        }

        if self.needs_dsp_reset_on_update {
            self.needs_dsp_reset_on_update = false;
        }
    }

    /// Convert MIDI note number to frequency in Hz using equal temperament tuning.
    ///
    /// The standard MIDI-to-frequency formula is:
    /// ```text
    /// f = 440 * 2^((note - 69) / 12)
    /// ```
    ///
    /// This is based on:
    /// - **A4 (MIDI note 69) = 440 Hz** (concert pitch reference)
    /// - **Equal temperament**: Each semitone is a frequency ratio of 2^(1/12) ≈ 1.059463
    /// - **Octave doubling**: Every 12 semitones doubles the frequency
    ///
    /// # Examples
    ///
    /// - Note 69 (A4) → 440.0 Hz
    /// - Note 60 (C4, middle C) → 261.63 Hz
    /// - Note 81 (A5) → 880.0 Hz (one octave above A4)
    /// - Note 57 (A3) → 220.0 Hz (one octave below A4)
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127). 0 = C-1 (~8.18 Hz), 127 = G9 (~12543 Hz)
    fn midi_note_to_freq(note: u8) -> f32 {
        440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
    }

    /// Process one sample of audio, generating stereo output.
    ///
    /// This is the **core audio processing function** called once per sample (44,100+ times per second).
    /// It orchestrates all DSP components to produce a single stereo sample pair (left, right).
    ///
    /// # Processing Pipeline (per sample)
    ///
    /// 1. **Check if active**: Return (0.0, 0.0) if voice is inactive or envelope finished
    /// 2. **Process envelope**: Get current envelope value (0.0-1.0) for amplitude control
    /// 3. **Calculate velocity-sensitive amplitude**: Scale output based on key velocity
    /// 4. **For each oscillator group (0-2)**:
    ///    a. Check if this oscillator is soloed (skip others if any solo is active)
    ///    b. Mix all active unison voices for this oscillator
    ///    c. Normalize by sqrt(unison_count) to prevent clipping
    ///    d. Process LFO for filter modulation
    ///    e. Calculate modulated filter cutoff (base + key tracking + velocity + LFO)
    ///    f. Apply filter to oscillator output
    ///    g. Apply stereo panning using equal-power law
    ///    h. Accumulate into output_left and output_right
    /// 5. **Apply envelope and velocity**: Multiply final mix by envelope * velocity_factor
    /// 6. **Track peak amplitude**: Update peak for voice stealing metrics
    /// 7. **Return stereo pair**: (left, right)
    ///
    /// # Arguments
    ///
    /// * `osc_params` - Oscillator parameters (gain, pan, solo)
    /// * `filter_params` - Filter parameters (cutoff, resonance, key tracking, bandwidth)
    /// * `lfo_params` - LFO parameters (depth, filter modulation amount)
    /// * `velocity_params` - Velocity sensitivity (amplitude and filter cutoff scaling)
    ///
    /// # Returns
    ///
    /// Stereo output samples `(left, right)` in the range [-1.0, 1.0] (nominal).
    /// If the voice is inactive or envelope finished, returns (0.0, 0.0).
    ///
    /// # Real-Time Performance
    ///
    /// This function is called 44,100+ times per second per voice, so it must be extremely efficient:
    /// - No allocations (all data is pre-allocated)
    /// - No locks (all data is voice-local or read-only)
    /// - Minimal branching (predictable execution paths)
    /// - ~10-15 μs per voice on modern CPUs (target: <20 μs to stay under 1ms for 16 voices)
    pub fn process(
        &mut self,
        osc_params: &[OscillatorParams; 3],
        filter_params: &[FilterParams; 3],
        lfo_params: &[LFOParams; 3],
        velocity_params: &VelocityParams,
        hard_sync_enabled: bool,
        voice_comp_params: &VoiceCompressorParams,
        transient_params: &TransientShaperParams,
    ) -> (f32, f32) {
        // === STEP 1: Early exit for inactive voices ===
        // If this voice isn't producing sound, return silence immediately.
        // This saves ~95% of CPU by skipping all DSP processing for idle voices.
        if !self.is_active {
            return (0.0, 0.0);
        }

        // === STEP 2: Process the ADSR envelope ===
        // Get the current envelope value (0.0 = silent, 1.0 = full volume).
        // The envelope progresses through attack → decay → sustain → release phases.
        let env_value = self.envelope.process();

        // Check if envelope has finished its release phase
        // When the envelope completes, the voice becomes inactive and returns to the idle pool.
        if !self.envelope.is_active() {
            self.is_active = false;
            return (0.0, 0.0);
        }

        // === STEP 3: Calculate velocity-sensitive amplitude ===
        // Standardized formula: output = 1.0 + sensitivity * (velocity - 0.5)
        // This maps velocity to amplitude scaling:
        //   Velocity 0.0 → 1.0 - 0.5 * sensitivity (quieter, e.g., 0.5× at sensitivity=1.0)
        //   Velocity 0.5 → 1.0 (no change, neutral)
        //   Velocity 1.0 → 1.0 + 0.5 * sensitivity (louder, e.g., 1.5× at sensitivity=1.0)
        //
        // Why 0.5 as the neutral point? It corresponds to MIDI velocity 64 (half of 127),
        // the typical "medium" playing strength.
        let velocity_factor = 1.0 + velocity_params.amp_sensitivity * (self.velocity - 0.5);

        // Initialize stereo output accumulators
        // Each oscillator's output will be panned and added to these.
        let mut output_left = 0.0;
        let mut output_right = 0.0;

        // === STEP 4: Check if any oscillator is soloed ===
        // If any oscillator has solo=true, we process ONLY the soloed oscillators.
        // This is a common DAW/synth feature for isolating sounds during sound design.
        let any_soloed = osc_params.iter().any(|o| o.solo);

        // === STEP 4.5: Process all LFOs for global routing matrix ===
        // Process all 3 LFOs upfront to get modulation values for pitch/gain/pan/PWM.
        // These are global modulations that affect all oscillators.
        // Filter modulation is still per-oscillator (processed later in Step 6).
        let mut lfo_values = [0.0; 3];
        let mut filter_env_values = [0.0; 3];
        for i in 0..3 {
            lfo_values[i] = self.lfos[i].process(); // Returns -1.0 to 1.0
            filter_env_values[i] = self.filter_envelopes[i].process();
        }

        // Calculate global LFO modulations (sum contributions from all 3 LFOs)
        // Pitch modulation in cents (bipolar: ±pitch_amount)
        let mut global_pitch_mod_cents = 0.0;
        // Gain modulation (bipolar: ±0.5 when gain_amount = 1.0)
        let mut global_gain_mod = 0.0;
        // Pan modulation (bipolar: ±1.0 when pan_amount = 1.0)
        let mut global_pan_mod = 0.0;
        // PWM/shape modulation (bipolar: ±1.0 when pwm_amount = 1.0)
        let mut global_pwm_mod = 0.0;

        for i in 0..3 {
            let lfo_val = lfo_values[i];
            let depth = lfo_params[i].depth;

            // Sum contributions from each LFO (bipolar modulation)
            global_pitch_mod_cents += lfo_val * lfo_params[i].pitch_amount * depth;
            global_gain_mod += lfo_val * lfo_params[i].gain_amount * depth * 0.5; // Scale to ±0.5
            global_pan_mod += lfo_val * lfo_params[i].pan_amount * depth;
            global_pwm_mod += lfo_val * lfo_params[i].pwm_amount * depth;
        }

        // === STEP 5: Generate all oscillator outputs (with feedback FM support) ===
        // We process oscillators in order: 0 → 1 → 2
        // Any oscillator can be modulated by any other oscillator, including "feedback"
        // where a later oscillator modulates an earlier one (e.g., Osc 3 → Osc 1).
        // Feedback uses the previous sample's output (1-sample delay), which is how
        // classic FM synthesizers like the Yamaha DX7 implemented feedback.
        //
        // Store raw oscillator outputs before filtering (needed for FM sources).
        let mut osc_outputs = self.osc_outputs_prev; // Start with previous sample's outputs

        for i in 0..3 {
            // Skip this oscillator if:
            // - Solo mode is active AND this oscillator is not soloed
            // - This oscillator has zero unison voices
            if any_soloed && !osc_params[i].solo {
                continue;
            }

            let unison_count = self.active_unison[i];
            if unison_count == 0 {
                continue;
            }

            // === STEP 5a: Apply LFO pitch and PWM modulation ===
            // Pitch modulation: Recalculate frequency with LFO cents offset
            // PWM modulation: Modify shape parameter with LFO offset
            let pitch_mod_active = global_pitch_mod_cents.abs() > 0.001;
            let pwm_mod_active = global_pwm_mod.abs() > 0.001;
            if pitch_mod_active || pwm_mod_active {
                let lfo_pitch_mult = if pitch_mod_active {
                    2.0_f32.powf(global_pitch_mod_cents / 1200.0)
                } else {
                    1.0
                };

                let base_osc_freq = self.osc_base_freq_hz[i] * lfo_pitch_mult;
                let modulated_shape = (osc_params[i].shape + global_pwm_mod).clamp(-1.0, 1.0);

                for unison_idx in 0..unison_count {
                    if let Some(ref mut osc) = self.oscillators[i][unison_idx] {
                        if pitch_mod_active {
                            let freq = base_osc_freq * self.unison_detune_mul[i][unison_idx];
                            osc.set_frequency(freq);
                        }
                        if pwm_mod_active {
                            osc.set_shape(modulated_shape);
                        }
                    }
                }
            }

            // Check if this oscillator should be frequency modulated
            let fm_config = osc_params[i].fm_source;
            let fm_amount = osc_params[i].fm_amount;

            // === STEP 5b: Mix all unison voices for this oscillator ===
            // Unison creates a thick sound by layering detuned copies of the same waveform.
            // We sum all unison voices and normalize to prevent clipping.
            let mut osc_sum = 0.0;

            // Determine if we should use FM synthesis for this oscillator
            let use_fm = if let Some(source_idx) = fm_config {
                // Only use FM if:
                // 1. Source oscillator index is valid (0-2)
                // 2. FM amount is non-zero
                // Note: We allow source_idx >= i (feedback) using previous sample's output
                source_idx < 3 && fm_amount.abs() > 0.001
            } else {
                false
            };

            if use_fm {
                // FM synthesis: modulate this oscillator's phase using a previous oscillator
                let source_idx = fm_config.unwrap();
                let modulator_output = osc_outputs[source_idx];

                for unison_idx in 0..unison_count {
                    if let Some(ref mut osc) = self.oscillators[i][unison_idx] {
                        osc_sum += osc.process_with_fm(modulator_output, fm_amount);
                    }
                }
            } else {
                // Standard synthesis: no FM modulation
                for unison_idx in 0..unison_count {
                    if let Some(ref mut osc) = self.oscillators[i][unison_idx] {
                        osc_sum += osc.process();
                    }
                }
            }

            // === STEP 5b: Normalize unison sum ===
            // Use peak normalization (divide by N) to preserve headroom.
            // With small detune amounts, unison voices can align in phase and produce
            // large instantaneous peaks; power normalization (sqrt(N)) would then drive
            // the master limiter frequently, which is perceived as distortion/pumping.
            //
            // If unison_normalize is disabled, we skip normalization for a thicker,
            // louder sound that intentionally drives the limiter/distortion.
            let osc_out = if osc_params[i].unison_normalize {
                let unison_count_f32 = unison_count as f32;
                osc_sum / unison_count_f32
            } else {
                // No normalization - raw sum for maximum thickness
                // Apply a small sqrt(N) compensation to prevent extreme levels
                let unison_count_f32 = unison_count as f32;
                osc_sum / unison_count_f32.sqrt()
            };

            // === STEP 5b.1: Apply per-oscillator saturation/warmth ===
            // Adds subtle harmonics before filtering for analog warmth
            // Uses tanh soft clipping with gain compensation (same pattern as filter drive)
            let osc_out = if osc_params[i].saturation > 0.001 {
                // Map saturation 0-1 to gain multiplier 1x-3x
                let drive_gain = 1.0 + osc_params[i].saturation * 2.0;
                let amplified = osc_out * drive_gain;
                // Soft saturation using tanh
                let saturated = amplified.tanh();
                // Gain compensation (perceptual loudness)
                saturated / drive_gain.sqrt()
            } else {
                osc_out
            };

            // Store the raw oscillator output (needed for FM routing)
            osc_outputs[i] = osc_out;

            // === STEP 5c: Hard Sync Chain - OSC1→OSC2→OSC3 ===
            // When hard sync is enabled, oscillators sync in a chain:
            // - OSC1 resets OSC2's phase when OSC1 completes a cycle
            // - OSC2 resets OSC3's phase when OSC2 completes a cycle
            // This creates cascading harmonic complexity, similar to classic analog synths
            // like the Sequential Prophet-5 and Moog Voyager.
            //
            // The sharp phase discontinuities generate bright, aggressive harmonics.
            // Classic use case: EDM leads, aggressive bass, complex evolving timbres
            // Example: All 3 oscillators slightly detuned → rich harmonic series
            if hard_sync_enabled {
                if i == 0 && self.active_unison[1] > 0 {
                    // OSC1→OSC2 sync
                    if let Some(ref osc1) = self.oscillators[0][0] {
                        let current_phase = osc1.get_phase();

                        // Detect wrap: previous phase was higher than current (e.g., 0.95 → 0.05)
                        if self.osc1_phase_prev > current_phase {
                            // Reset all Osc2 unison voices to phase 0.0
                            for unison_idx in 0..self.active_unison[1] {
                                if let Some(ref mut osc2) = self.oscillators[1][unison_idx] {
                                    osc2.set_phase(0.0);
                                    osc2.apply_initial_phase();
                                }
                            }
                        }

                        // Store current phase for next sample's comparison
                        self.osc1_phase_prev = current_phase;
                    }
                } else if i == 1 && self.active_unison[2] > 0 {
                    // OSC2→OSC3 sync
                    if let Some(ref osc2) = self.oscillators[1][0] {
                        let current_phase = osc2.get_phase();

                        // Detect wrap: previous phase was higher than current
                        if self.osc2_phase_prev > current_phase {
                            // Reset all Osc3 unison voices to phase 0.0
                            for unison_idx in 0..self.active_unison[2] {
                                if let Some(ref mut osc3) = self.oscillators[2][unison_idx] {
                                    osc3.set_phase(0.0);
                                    osc3.apply_initial_phase();
                                }
                            }
                        }

                        // Store current phase for next sample's comparison
                        self.osc2_phase_prev = current_phase;
                    }
                }
            }
        }

        // Store outputs for next sample (enables feedback FM)
        self.osc_outputs_prev = osc_outputs;

        // === STEP 6: Apply filters and generate stereo output ===
        // Now that all oscillators have been processed (and their outputs stored),
        // we can apply filtering and panning to generate the final stereo mix.
        for i in 0..3 {
            // Skip if this oscillator is inactive
            if any_soloed && !osc_params[i].solo {
                continue;
            }
            if self.active_unison[i] == 0 {
                continue;
            }

            let mut osc_out = osc_outputs[i];

            // === STEP 6a: Apply LFO gain modulation ===
            // Gain modulation (tremolo): Modulate oscillator amplitude
            if global_gain_mod.abs() > 0.001 {
                let gain_mult = (1.0 + global_gain_mod).clamp(0.0, 2.0);
                osc_out *= gain_mult;
            }

            // === STEP 6b: Process LFO for filter modulation ===
            // LFOs generate slow-moving waveforms (typically <20 Hz) that modulate parameters.
            // We already processed LFOs earlier; reuse the stored value.
            let lfo_value = lfo_values[i];

            // === STEP 6c: Calculate modulated filter cutoff ===
            // The cutoff frequency is determined by multiple factors:
            // 1. Base cutoff (set by user or automation)
            // 2. Key tracking (higher notes → higher cutoff, follows keyboard)
            // 3. Velocity sensitivity (harder key press → higher cutoff)
            // 4. LFO modulation (time-varying cutoff for wah/vibrato effects)

            let base_cutoff = filter_params[i].cutoff;

            // **Key tracking**: Scale filter cutoff with MIDI note number
            // If key_tracking = 1.0, the filter tracks the keyboard 1:1 (cutoff doubles per octave).
            // If key_tracking = 0.0, the multiplier is 1.0 and this offset becomes 0.0.
            let key_tracking_offset = base_cutoff * (self.filter_key_tracking_mul[i] - 1.0);

            // **Velocity to filter cutoff**: Harder key press opens the filter more
            // Uses the same standardized formula as amplitude (centered at velocity 0.5).
            // At velocity 0.0: cutoff reduced, at velocity 1.0: cutoff raised.
            let velocity_cutoff_offset =
                base_cutoff * velocity_params.filter_sensitivity * (self.velocity - 0.5);

            // **Combine all modulations and clamp to audible range [20 Hz, 20 kHz]**
            let modulated_cutoff = (base_cutoff
                + key_tracking_offset
                + velocity_cutoff_offset
                + lfo_value * lfo_params[i].filter_amount * lfo_params[i].depth
                + filter_env_values[i] * filter_params[i].envelope.amount)
                .clamp(20.0, 20000.0);

            // === STEP 6d: Apply pre-filter drive (saturation) ===
            // Pre-filter saturation adds warmth and presence by generating harmonics
            // BEFORE filtering. This is key for "analog" sound and punch.
            let driven_signal = if filter_params[i].drive > 0.001 {
                // Map drive 0-1 to gain multiplier 1x-3x
                let drive_gain = 1.0 + filter_params[i].drive * 2.0;
                let amplified = osc_out * drive_gain;
                // Soft saturation using tanh (tube-like warmth)
                let saturated = amplified.tanh();
                // Compensate for gain to maintain perceived level
                saturated / drive_gain.sqrt()
            } else {
                osc_out
            };

            // === STEP 6e: Update filter and apply to driven signal ===
            // During mono legato note changes, smooth cutoff briefly to avoid a small
            // click from instantaneous coefficient updates (key tracking / velocity).
            let cutoff_to_set = if self.mono_declick_samples_remaining > 0 {
                let prev = self.mono_smoothed_cutoff_hz[i];
                let coeff = self.mono_declick_cutoff_coeff;
                let smoothed = if prev <= 0.0 {
                    modulated_cutoff
                } else {
                    coeff * prev + (1.0 - coeff) * modulated_cutoff
                };
                self.mono_smoothed_cutoff_hz[i] = smoothed;
                smoothed
            } else {
                self.mono_smoothed_cutoff_hz[i] = modulated_cutoff;
                modulated_cutoff
            };

            self.filters[i].set_cutoff(cutoff_to_set);
            let filtered = self.filters[i].process(driven_signal);

            // === STEP 6e.1: Apply post-filter drive (saturation after filtering) ===
            // Post-filter saturation adds harmonics to the filtered signal
            // Creates different tonal character than pre-filter drive (presence & edge)
            let post_filtered = if filter_params[i].post_drive > 0.001 {
                // Map drive 0-1 to gain multiplier 1x-3x (same range as pre-filter drive)
                let drive_gain = 1.0 + filter_params[i].post_drive * 2.0;
                let amplified = filtered * drive_gain;
                // Soft saturation using tanh
                let saturated = amplified.tanh();
                // Gain compensation (perceptual loudness)
                saturated / drive_gain.sqrt()
            } else {
                filtered
            };

            // === STEP 6f: Apply LFO pan modulation and stereo panning ===
            // Pan: -1.0 (full left) to 1.0 (full right), 0.0 = center
            //
            // Apply LFO pan modulation first
            let (left_gain, right_gain) = if self.pan_mod_active {
                let modulated_pan = (osc_params[i].pan + global_pan_mod).clamp(-1.0, 1.0);

                // Equal-power panning law ensures constant perceived loudness as we pan.
                // We map pan to an angle (0 to π/2) and use sin/cos for the gain curves:
                //   pan = -1.0 → angle = 0       → left = 1.0, right = 0.0 (full left)
                //   pan =  0.0 → angle = π/4     → left = 0.707, right = 0.707 (center, -3dB each)
                //   pan = +1.0 → angle = π/2     → left = 0.0, right = 1.0 (full right)
                //
                // Why π/4 radians (45°) for center? sin(45°) = cos(45°) = 1/√2 ≈ 0.707,
                // and 0.707² + 0.707² = 1.0 (constant power).
                let pan_radians = (modulated_pan + 1.0) * std::f32::consts::PI / 4.0; // Map [-1, 1] to [0, π/2]
                (pan_radians.cos(), pan_radians.sin())
            } else {
                (self.cached_pan_left_gain[i], self.cached_pan_right_gain[i])
            };

            // Apply gain and panning, then accumulate into output channels
            let scaled = post_filtered * osc_params[i].gain;
            output_left += scaled * left_gain;
            output_right += scaled * right_gain;
        }

        // === STEP 7: Normalize for multiple active oscillators ===
        // When multiple oscillators are active, their signals sum and can exceed ±1.0.
        // We use simple division by N to prevent clipping, keeping the math straightforward.
        // The engine will handle polyphonic gain compensation (multiple voices playing).
        let active_osc_count = osc_params
            .iter()
            .enumerate()
            .filter(|(idx, p)| (!any_soloed || p.solo) && self.active_unison[*idx] > 0)
            .count();

        if active_osc_count > 1 {
            // Simple division by N to prevent clipping when stacking oscillators
            // This maintains clean headroom for the engine's polyphonic mixing
            let osc_normalization = 1.0 / active_osc_count as f32;
            output_left *= osc_normalization;
            output_right *= osc_normalization;
        }

        // === STEP 7.1: Gentle unison loudness compensation ===
        // Even with per-oscillator unison peak normalization (divide by N), increasing unison
        // generally increases perceived loudness and makes multi-note chords more likely to
        // drive the master limiter. Apply a mild, psychoacoustically-friendly compensation
        // based on the average unison count of oscillators that have unison normalization
        // enabled.
        //
        // This is intentionally gentler than sqrt(N) to avoid "sound destruction" at high
        // unison counts.
        let (norm_osc_count, norm_unison_sum) = osc_params
            .iter()
            .enumerate()
            .filter(|(idx, p)| {
                (!any_soloed || p.solo) && p.unison_normalize && self.active_unison[*idx] > 0
            })
            .fold((0_usize, 0_usize), |(count, sum), (idx, _p)| {
                (count + 1, sum + self.active_unison[idx])
            });

        if norm_osc_count > 0 {
            let avg_unison = (norm_unison_sum as f32 / norm_osc_count as f32).max(1.0);
            let unison_comp = 1.0 + 0.3 * avg_unison.log2();
            output_left /= unison_comp;
            output_right /= unison_comp;
        }

        // === STEP 8: Apply envelope and velocity-sensitive amplitude ===
        // Multiply the final mixed output by the envelope (0.0-1.0) and velocity factor.
        // This shapes the amplitude over time (ADSR) and scales by key velocity.
        output_left = output_left * env_value * velocity_factor;
        output_right = output_right * env_value * velocity_factor;

        // === STEP 9: Track peak amplitude for voice stealing ===
        // The engine uses peak amplitude to identify the quietest voice when all 16 voices
        // are busy and a new note arrives. We track the peak of both channels.
        //
        // Why peak instead of RMS? Peak tracking is much cheaper (~1% CPU vs 3-5% for RMS)
        // and sufficient for voice stealing. RMS (root mean square) is more accurate for
        // perceived loudness, but the cost outweighs the benefit in this use case.
        let output_peak = output_left.abs().max(output_right.abs());
        self.peak_amplitude = self.peak_amplitude.max(output_peak);

        // Store last output (mono average) for potential future use
        // Currently unused, but could be used for decay rate detection or debugging.
        self.last_output = (output_left + output_right) / 2.0;

        // === STEP 10: Apply per-voice compression (if enabled) ===
        // Per-voice compression catches transients before they hit the master mix.
        // Uses optimized mono compression with envelope follower updating every 4 samples
        // for CPU efficiency across 16 simultaneous voices.
        if voice_comp_params.enabled {
            if !self.last_applied_voice_comp_params.enabled
                || *voice_comp_params != self.last_applied_voice_comp_params
            {
                self.voice_compressor
                    .set_threshold(voice_comp_params.threshold);
                self.voice_compressor.set_ratio(voice_comp_params.ratio);
                self.voice_compressor.set_attack(voice_comp_params.attack);
                self.voice_compressor.set_release(voice_comp_params.release);
                self.voice_compressor.set_knee(voice_comp_params.knee);
                self.voice_compressor
                    .set_makeup_gain(voice_comp_params.makeup_gain);

                self.last_applied_voice_comp_params = *voice_comp_params;
            }

            // Apply fast mono compression
            (output_left, output_right) = self
                .voice_compressor
                .process_fast(output_left, output_right);
        }

        // === STEP 10.1: Apply transient shaper (envelope-based gain modulation) ===
        // Emphasizes attack transients and reduces sustain for punchier sounds
        // Works by multiplying gain during specific envelope stages
        if transient_params.enabled {
            use crate::dsp::envelope::EnvelopeStage;

            let gain_mult = match self.envelope.stage() {
                EnvelopeStage::Attack => {
                    // Boost transients during attack (0.0-1.0 → 1x to 2x gain)
                    1.0 + transient_params.attack_boost
                }
                EnvelopeStage::Sustain => {
                    // Reduce sustain level (0.0-1.0 → 1x to 0x gain)
                    1.0 - transient_params.sustain_reduction
                }
                _ => 1.0, // No change for decay/release/idle
            };

            output_left *= gain_mult;
            output_right *= gain_mult;
        }

        // === STEP 11: Apply anti-click fade-in (FINAL STAGE) ===
        // Apply a 5ms exponential fade-in at the start of every note.
        // This MUST be the very last processing step to ensure nothing can bypass it.
        // Prevents clicks/crackles when voices are stolen or notes retrigger quickly.
        // The fade is independent of envelope attack and uses exponential curve for smoothness.
        if self.anti_click_samples < self.anti_click_fade_samples {
            // Exponential fade: 0 → 1 over 5ms
            // Formula: 1 - e^(-5 * progress)
            // This creates a smooth curve that reaches ~99% at progress=1.0
            let progress = self.anti_click_samples as f32 / self.anti_click_fade_samples as f32;
            let fade = 1.0 - (-5.0 * progress).exp();

            output_left *= fade;
            output_right *= fade;

            self.anti_click_samples += 1;
        }

        // Very short crossfade on retrigger to avoid a step from release tail → restarted fade.
        if self.retrigger_xfade_samples_remaining > 0 {
            let total = self.retrigger_xfade_total_samples.max(1) as f32;
            let remaining = self.retrigger_xfade_samples_remaining as f32;
            let progress = 1.0 - (remaining / total);
            let t = progress.clamp(0.0, 1.0);

            output_left = self.retrigger_prev_left * (1.0 - t) + output_left * t;
            output_right = self.retrigger_prev_right * (1.0 - t) + output_right * t;

            self.retrigger_xfade_samples_remaining -= 1;
        }

        if self.mono_declick_samples_remaining > 0 {
            self.mono_declick_samples_remaining -= 1;
        }

        // Store final output for click-free retriggers.
        self.last_output_left = output_left;
        self.last_output_right = output_right;

        // === STEP 12: Return stereo output pair ===
        (output_left, output_right)
    }

    /// Get current amplitude level for voice stealing decisions.
    ///
    /// This method returns the **peak amplitude** seen since the last `note_on()` call.
    /// The engine uses this to identify the quietest voice when all 16 voices are busy
    /// and a new note arrives—it steals the voice with the lowest peak amplitude.
    ///
    /// # Why Peak Instead of RMS?
    ///
    /// Originally, this function calculated RMS (Root Mean Square) using an exponential
    /// moving average, which is more accurate for perceived loudness. However, RMS calculation
    /// costs 3-5% CPU per voice (×16 voices = 48-80% total), which is unacceptable.
    ///
    /// Peak tracking costs <1% CPU and is "good enough" for voice stealing:
    /// - A voice with peak = 0.01 is quieter than a voice with peak = 0.5
    /// - Stealing the quieter voice produces less audible artifacts
    /// - The slight inaccuracy vs. RMS doesn't matter perceptually
    ///
    /// # Return Value
    ///
    /// Peak amplitude in the range [0.0, ∞), though typical values are [0.0, 1.0].
    /// Returns 0.0 for inactive voices or voices that haven't produced any output yet.
    ///
    /// # Name Mismatch
    ///
    /// The method is called `get_rms()` for historical reasons (it used to return RMS),
    /// but it actually returns peak amplitude. Renaming would break the API, so the name
    /// remains but the implementation changed for performance.
    #[inline]
    #[must_use]
    pub fn peak_amplitude(&self) -> f32 {
        // Return peak amplitude seen since note on
        // This is a simple, fast metric that works well for voice stealing
        self.peak_amplitude
    }

    /// Check if this voice is currently active (producing sound).
    ///
    /// A voice is active from the moment `note_on()` is called until the ADSR envelope
    /// completes its release phase. During the release phase, the voice is still active
    /// (producing audio) even though the key has been released.
    ///
    /// The engine uses this to count how many voices are currently producing sound
    /// (for CPU usage estimation and voice stealing decisions).
    ///
    /// # Return Value
    ///
    /// - `true`: Voice is producing audio (note-on, sustain, or release phase)
    /// - `false`: Voice is idle, waiting to be assigned a note
    #[inline]
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get the MIDI note number currently assigned to this voice.
    ///
    /// Returns the note number (0-127) set by the most recent `note_on()` call.
    /// For inactive voices, this returns the last note played (or 0 if never activated).
    ///
    /// The engine uses this to implement monophonic mode (last-note-priority):
    /// when you release a key in monophonic mode, the engine checks if any other
    /// keys are still held and retrieves their note numbers to continue playing.
    ///
    /// # Return Value
    ///
    /// MIDI note number (0-127):
    /// - 0 = C-1 (~8.18 Hz)
    /// - 60 = C4 (middle C, ~261.63 Hz)
    /// - 69 = A4 (concert pitch, 440 Hz)
    /// - 127 = G9 (~12543 Hz)
    #[inline]
    #[must_use]
    pub fn note(&self) -> u8 {
        self.note
    }

    /// Reset all voice state to initial values, returning it to the idle pool.
    ///
    /// This method clears all internal state, making the voice ready for reuse.
    /// It's typically called by the engine when implementing "panic" or "all notes off"
    /// functionality—immediately silencing all voices without waiting for release envelopes.
    ///
    /// # What Gets Reset
    ///
    /// - `is_active` set to `false` (voice returns to idle)
    /// - `note` and `velocity` cleared to 0
    /// - Envelope reset to initial state (ready for next note-on)
    /// - All LFOs reset to phase 0.0
    /// - All oscillators reset (phase to 0.0, clear internal state)
    /// - All filters reset (clear delay lines, remove residual ringing)
    /// - Peak amplitude and RMS metrics cleared
    ///
    /// # When to Use
    ///
    /// - **Panic button**: User hits "all notes off" in the DAW or GUI
    /// - **Transport stop**: DAW stops playback and wants to immediately silence all voices
    /// - **Manual cleanup**: Debugging or testing scenarios
    ///
    /// # Difference from note_off()
    ///
    /// `note_off()` triggers a gradual fade-out (release envelope), while `reset()`
    /// immediately silences the voice. Use `reset()` for emergency stops, `note_off()`
    /// for musical key releases.
    pub fn reset(&mut self) {
        // Mark voice as inactive, returning it to the idle pool
        self.is_active = false;
        self.note = 0;
        self.velocity = 0.0;

        // Reset envelope to initial state (ready for next attack)
        self.envelope.reset();

        // DO NOT reset LFOs - they should run continuously to avoid modulation discontinuities
        // Comment out: for lfo in &mut self.lfos { lfo.reset(); }

        // Reset all oscillators (clear phase, DC offset, downsampler state)
        // We iterate through all 21 pre-allocated oscillators (3 slots × 7 unison)
        for osc_slot in &mut self.oscillators {
            for osc_opt in osc_slot.iter_mut() {
                if let Some(osc) = osc_opt {
                    osc.reset();
                }
            }
        }

        // Reset all filters (clear delay lines to remove residual ringing)
        // This is important to prevent "ghost" resonances from the previous note
        for filter in &mut self.filters {
            filter.reset();
        }

        // Reset voice stealing metrics
        self.rms_squared_ema = 0.0;
        self.peak_amplitude = 0.0;
        self.last_output = 0.0;
        self.needs_dsp_reset_on_update = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::Waveform;
    use approx::assert_relative_eq;

    /// Helper function to create default oscillator parameters for testing.
    ///
    /// Returns an array of 3 OscillatorParams with default values (typically sine wave,
    /// no detune, 1 unison voice, etc.). Used by tests to avoid repetitive parameter setup.
    fn default_osc_params() -> [OscillatorParams; 3] {
        [OscillatorParams::default(); 3]
    }

    /// Helper function to create default wavetable library for testing.
    ///
    /// Returns an empty WavetableLibrary. Used by tests to avoid repetitive parameter setup.
    fn default_wavetable_library() -> crate::dsp::wavetable_library::WavetableLibrary {
        crate::dsp::wavetable_library::WavetableLibrary::new()
    }

    /// Helper function to create default filter parameters for testing.
    ///
    /// Returns an array of 3 FilterParams with default values (typically lowpass,
    /// moderate cutoff and resonance). Used by tests to avoid repetitive parameter setup.
    fn default_filter_params() -> [FilterParams; 3] {
        [FilterParams::default(); 3]
    }

    /// Helper function to create default LFO parameters for testing.
    ///
    /// Returns an array of 3 LFOParams with default values (typically slow sine wave
    /// with minimal modulation). Used by tests to avoid repetitive parameter setup.
    fn default_lfo_params() -> [LFOParams; 3] {
        [LFOParams::default(); 3]
    }

    /// Helper function to create default velocity parameters for testing.
    ///
    /// Returns VelocityParams with default sensitivity values.
    /// Used by tests to avoid repetitive parameter setup.
    fn default_velocity_params() -> VelocityParams {
        VelocityParams::default()
    }

    /// Helper function to create default envelope parameters for testing.
    ///
    /// Returns EnvelopeParams with default ADSR values.
    /// Used by tests to avoid repetitive parameter setup.
    fn default_envelope_params() -> EnvelopeParams {
        EnvelopeParams::default()
    }

    /// Helper function to create default voice compressor parameters for testing.
    ///
    /// Returns VoiceCompressorParams with default values (disabled by default).
    /// Used by tests to avoid repetitive parameter setup.
    fn default_voice_comp_params() -> VoiceCompressorParams {
        VoiceCompressorParams::default()
    }

    /// Helper function to create default transient shaper parameters for testing.
    ///
    /// Returns TransientShaperParams with default values (disabled by default).
    /// Used by tests to avoid repetitive parameter setup.
    fn default_transient_params() -> TransientShaperParams {
        TransientShaperParams::default()
    }

    /// Test that a newly created voice is in the correct initial state.
    ///
    /// Verifies:
    /// - Voice is inactive (not producing sound)
    /// - Note number is 0 (uninitialized)
    /// - Voice can be created without panicking (no allocation failures)
    #[test]
    fn test_voice_creation() {
        let voice = Voice::new(44100.0);
        assert!(!voice.is_active());
        assert_eq!(voice.note(), 0);
    }

    /// Test that note_on() correctly activates the voice and stores parameters.
    ///
    /// Verifies:
    /// - Voice becomes active after note_on()
    /// - Note number is stored correctly
    /// - Velocity is stored correctly (within floating-point precision)
    ///
    /// This tests the basic note triggering mechanism used by the engine.
    #[test]
    fn test_note_on_activates_voice() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60, 0.8);

        assert!(voice.is_active());
        assert_eq!(voice.note(), 60);
        assert_relative_eq!(voice.velocity, 0.8, epsilon = 0.001);
    }

    /// Test that velocity values outside [0.0, 1.0] are clamped correctly.
    ///
    /// Verifies:
    /// - Velocity > 1.0 is clamped to 1.0
    /// - Velocity < 0.0 is clamped to 0.0
    ///
    /// This prevents invalid MIDI values or automation data from causing
    /// unexpected behavior (e.g., amplitude >1.0 causing clipping).
    #[test]
    fn test_velocity_clamping() {
        let mut voice = Voice::new(44100.0);

        voice.note_on(60, 1.5);
        assert_eq!(voice.velocity, 1.0);

        voice.note_on(60, -0.5);
        assert_eq!(voice.velocity, 0.0);
    }

    /// Test the MIDI note-to-frequency conversion formula.
    ///
    /// Verifies:
    /// - A4 (note 69) = 440 Hz (concert pitch reference)
    /// - C4 (note 60) = ~261.63 Hz (middle C)
    /// - A5 (note 81) = 880 Hz (one octave above A4)
    ///
    /// This tests the equal temperament tuning formula used by all synthesizers.
    /// Formula: f = 440 * 2^((note - 69) / 12)
    #[test]
    fn test_midi_note_to_freq() {
        // A4 = 440 Hz
        assert_relative_eq!(Voice::midi_note_to_freq(69), 440.0, epsilon = 0.01);

        // C4 = ~261.63 Hz
        assert_relative_eq!(Voice::midi_note_to_freq(60), 261.63, epsilon = 0.01);

        // A5 = 880 Hz (one octave up)
        assert_relative_eq!(Voice::midi_note_to_freq(81), 880.0, epsilon = 0.01);
    }

    /// Test that an active voice produces non-zero audio output.
    ///
    /// Verifies:
    /// - After note_on() and parameter updates, the voice generates audio
    /// - Output is non-zero within 1000 samples (sufficient for attack phase)
    ///
    /// This is a basic sanity check that the DSP pipeline is functioning.
    /// We don't check exact output values (too brittle), just that sound is produced.
    #[test]
    fn test_voice_produces_output() {
        let mut voice = Voice::new(44100.0);
        let mut osc_params = default_osc_params();
        osc_params[0].gain = 0.25; // Enable oscillator 1 to produce sound
        let filter_params = default_filter_params();
        let lfo_params = default_lfo_params();
        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice.note_on(60, 0.8);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        // Process enough samples for the attack phase to produce audible output
        // Attack is typically 10-100ms, so 1000 samples at 44.1kHz = ~22ms
        let mut found_nonzero = false;
        for _ in 0..1000 {
            let (left, right) = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
            if (left.abs() + right.abs()) / 2.0 > 0.001 {
                found_nonzero = true;
                break;
            }
        }

        assert!(found_nonzero, "Voice should produce non-zero output");
    }

    /// Test that a voice eventually becomes inactive after note_off() and release.
    ///
    /// Verifies:
    /// - Voice remains active during sustain phase
    /// - After note_off(), voice stays active during release phase
    /// - Voice becomes inactive when release envelope completes
    ///
    /// This tests the ADSR envelope lifecycle and ensures voices don't get "stuck"
    /// in active state (which would waste CPU and prevent voice reuse).
    #[test]
    fn test_voice_stops_after_release() {
        let mut voice = Voice::new(44100.0);
        let osc_params = default_osc_params();
        let filter_params = default_filter_params();
        let lfo_params = default_lfo_params();
        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice.note_on(60, 0.8);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        // Process to sustain phase (5000 samples at 44.1kHz = ~113ms)
        // This should be enough to reach sustain for typical attack/decay times
        for _ in 0..5000 {
            let _ = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
        }

        assert!(voice.is_active());

        // Trigger release envelope
        voice.note_off();

        // Process through release phase (should eventually become inactive)
        // Release is typically 500-2000ms, so 20,000 samples = ~450ms should be sufficient
        // We allow longer to account for very long release settings
        let mut became_inactive = false;
        for _ in 0..20000 {
            let _ = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
            if !voice.is_active() {
                became_inactive = true;
                break;
            }
        }

        assert!(
            became_inactive,
            "Voice should become inactive after release"
        );
    }

    /// Test that an inactive voice produces (0.0, 0.0) output.
    ///
    /// Verifies:
    /// - Inactive voices return (0.0, 0.0) without processing DSP
    ///
    /// This is an important optimization: inactive voices skip all DSP processing,
    /// saving ~95% of CPU when voices are in the idle pool.
    #[test]
    fn test_inactive_voice_produces_no_output() {
        let voice = Voice::new(44100.0);
        let osc_params = default_osc_params();
        let filter_params = default_filter_params();
        let lfo_params = default_lfo_params();
        let velocity_params = default_velocity_params();

        // Inactive voice should produce zero without processing
        let mut voice_mut = voice;
        let (left, right) = voice_mut.process(
            &osc_params,
            &filter_params,
            &lfo_params,
            &velocity_params,
            false,
            &default_voice_comp_params(),
            &default_transient_params(),
        );
        assert_eq!((left, right), (0.0, 0.0));
    }

    /// Test that RMS tracking (actually peak amplitude) updates correctly.
    ///
    /// Verifies:
    /// - Peak amplitude is zero initially
    /// - Peak amplitude becomes non-zero after processing audio
    ///
    /// The `get_rms()` method actually returns peak amplitude (historical naming).
    /// This metric is used for voice stealing—the engine steals the voice with
    /// the lowest peak amplitude when all 16 voices are busy.
    #[test]
    fn test_rms_tracking() {
        let mut voice = Voice::new(44100.0);
        let mut osc_params = default_osc_params();
        osc_params[0].gain = 0.25; // Enable oscillator 1 to produce sound
        let filter_params = default_filter_params();
        let lfo_params = default_lfo_params();
        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice.note_on(60, 0.8);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        // Process enough samples for peak amplitude to update
        // Peak tracking happens per-sample, so 256 samples is plenty
        for _ in 0..256 {
            let _ = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
        }

        // Peak amplitude should be non-zero for an active voice producing sound
        assert!(
            voice.peak_amplitude() > 0.0,
            "RMS should be > 0 for active voice"
        );
    }

    /// Test that reset() clears all voice state correctly.
    ///
    /// Verifies:
    /// - Voice becomes inactive after reset()
    /// - Note and velocity are cleared to 0
    /// - Peak amplitude (RMS) is cleared to 0
    ///
    /// This tests the "all notes off" / "panic" functionality. After reset(),
    /// the voice should be in the same state as a newly created voice, ready
    /// to be assigned to a new note.
    #[test]
    fn test_reset() {
        let mut voice = Voice::new(44100.0);
        let osc_params = default_osc_params();
        let filter_params = default_filter_params();
        let lfo_params = default_lfo_params();
        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        // Activate voice and process some audio
        voice.note_on(60, 0.8);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        for _ in 0..100 {
            let _ = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
        }

        // Reset should clear all state
        voice.reset();

        assert!(!voice.is_active());
        assert_eq!(voice.note(), 0);
        assert_eq!(voice.velocity, 0.0);
        assert_eq!(voice.peak_amplitude(), 0.0);
    }

    #[test]
    fn test_lfo_pitch_routing_modulates_frequency() {
        // Test that LFO pitch routing applies vibrato (frequency modulation)
        let sample_rate = 44100.0;
        let mut voice = Voice::new(sample_rate);

        let mut osc_params = default_osc_params();
        osc_params[0].gain = 1.0;
        osc_params[0].waveform = Waveform::Sine;
        osc_params[0].gain = 1.0;

        let filter_params = default_filter_params();
        let mut lfo_params = default_lfo_params();

        // Set LFO1 to full depth, max rate, with 100 cents pitch amount
        lfo_params[0].depth = 1.0;
        lfo_params[0].rate = 20.0;
        lfo_params[0].pitch_amount = 100.0; // ±100 cents = ±1 semitone

        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice.note_on(60, 1.0);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        // Process several samples and verify pitch modulation occurs
        // (frequency changes over time due to LFO)
        let sample1 = voice.process(
            &osc_params,
            &filter_params,
            &lfo_params,
            &velocity_params,
            false,
            &default_voice_comp_params(),
            &default_transient_params(),
        );

        // Advance LFO by 1/4 period (should be near different LFO value)
        for _ in 0..(sample_rate as usize / (lfo_params[0].rate as usize * 4)) {
            let _ = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
        }

        let sample2 = voice.process(
            &osc_params,
            &filter_params,
            &lfo_params,
            &velocity_params,
            false,
            &default_voice_comp_params(),
            &default_transient_params(),
        );

        // With pitch modulation, the samples should differ significantly
        // (Note: exact values are hard to predict, but they shouldn't be identical)
        assert_ne!(
            sample1.0, sample2.0,
            "Pitch modulation should change output over time"
        );
    }

    #[test]
    fn test_lfo_gain_routing_applies_tremolo() {
        // Test that LFO gain routing applies tremolo (amplitude modulation)
        let sample_rate = 44100.0;
        let mut voice = Voice::new(sample_rate);

        let mut osc_params = default_osc_params();
        osc_params[0].gain = 1.0;
        osc_params[0].waveform = Waveform::Sine;
        osc_params[0].gain = 1.0;

        let filter_params = default_filter_params();
        let mut lfo_params = default_lfo_params();

        // Set LFO1 to full depth with maximum gain modulation
        lfo_params[0].depth = 1.0;
        lfo_params[0].rate = 10.0;
        lfo_params[0].gain_amount = 1.0; // Maximum tremolo depth

        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice.note_on(60, 1.0);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        // Collect samples over one LFO period
        let num_samples = (sample_rate / lfo_params[0].rate) as usize;
        let mut samples = Vec::new();

        for _ in 0..num_samples {
            let sample = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
            samples.push(sample.0.abs() + sample.1.abs()); // Sum L+R for amplitude
        }

        // With tremolo, amplitude should vary significantly
        let max_amp = samples.iter().cloned().fold(0.0f32, f32::max);
        let min_amp = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        // With gain_amount=1.0 and depth=1.0, we should see amplitude variation
        // (min should be noticeably less than max)
        assert!(
            max_amp > min_amp * 1.5,
            "Tremolo should create amplitude variation (max={}, min={})",
            max_amp,
            min_amp
        );
    }

    #[test]
    fn test_lfo_pan_routing_creates_auto_pan() {
        // Test that LFO pan routing creates auto-pan (stereo position modulation)
        let sample_rate = 44100.0;
        let mut voice = Voice::new(sample_rate);

        let mut osc_params = default_osc_params();
        osc_params[0].gain = 1.0;
        osc_params[0].waveform = Waveform::Sine;
        osc_params[0].gain = 1.0;
        osc_params[0].pan = 0.0; // Center pan

        let filter_params = default_filter_params();
        let mut lfo_params = default_lfo_params();

        // Set LFO1 to full depth with maximum pan modulation
        lfo_params[0].depth = 1.0;
        lfo_params[0].rate = 5.0;
        lfo_params[0].pan_amount = 1.0; // Full auto-pan range

        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice.note_on(60, 1.0);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        // Process samples and find when left/right channels differ significantly
        let mut found_left_bias = false;
        let mut found_right_bias = false;

        for _ in 0..(sample_rate as usize) {
            let sample = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );

            let left = sample.0.abs();
            let right = sample.1.abs();

            // Check for significant left bias
            if left > right * 1.2 {
                found_left_bias = true;
            }
            // Check for significant right bias
            if right > left * 1.2 {
                found_right_bias = true;
            }

            if found_left_bias && found_right_bias {
                break;
            }
        }

        assert!(
            found_left_bias && found_right_bias,
            "Auto-pan should create both left and right biased stereo positions"
        );
    }

    #[test]
    fn test_lfo_pwm_routing_modulates_pulse_width() {
        // Test that LFO PWM routing modulates pulse width on square/pulse waveforms
        let sample_rate = 44100.0;
        let mut voice = Voice::new(sample_rate);

        let mut osc_params = default_osc_params();
        osc_params[0].gain = 1.0;
        osc_params[0].waveform = Waveform::Square;
        osc_params[0].gain = 1.0;
        osc_params[0].shape = 0.0; // 50% duty cycle baseline

        let filter_params = default_filter_params();
        let mut lfo_params = default_lfo_params();

        // Set LFO1 to full depth with PWM modulation
        lfo_params[0].depth = 1.0;
        lfo_params[0].rate = 10.0;
        lfo_params[0].pwm_amount = 0.5; // Moderate PWM depth

        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice.note_on(60, 1.0);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        // Process samples and verify spectral content changes
        // (PWM creates characteristic harmonic variation)
        let mut samples1 = Vec::new();
        for _ in 0..100 {
            let sample = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
            samples1.push(sample.0);
        }

        // Advance by 1/2 LFO period (opposite PWM phase)
        for _ in 0..((sample_rate / (lfo_params[0].rate * 2.0)) as usize) {
            let _ = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
        }

        let mut samples2 = Vec::new();
        for _ in 0..100 {
            let sample = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
            samples2.push(sample.0);
        }

        // Compare sample sets - they should differ due to PWM modulation
        let diff: f32 = samples1
            .iter()
            .zip(samples2.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();

        assert!(
            diff > 1.0,
            "PWM modulation should change waveform shape over time (diff={})",
            diff
        );
    }

    #[test]
    fn test_lfo_routing_multiple_lfos_sum() {
        // Test that multiple LFOs with routing sum their contributions (global modulation)
        let sample_rate = 44100.0;
        let mut voice = Voice::new(sample_rate);

        let mut osc_params = default_osc_params();
        osc_params[0].gain = 1.0;
        osc_params[0].waveform = Waveform::Sine;
        osc_params[0].gain = 1.0;

        let filter_params = default_filter_params();
        let mut lfo_params = default_lfo_params();

        // Enable pitch routing on all 3 LFOs with different rates
        lfo_params[0].depth = 1.0;
        lfo_params[0].rate = 5.0;
        lfo_params[0].pitch_amount = 20.0; // ±20 cents

        lfo_params[1].depth = 1.0;
        lfo_params[1].rate = 7.0;
        lfo_params[1].pitch_amount = 30.0; // ±30 cents

        lfo_params[2].depth = 1.0;
        lfo_params[2].rate = 3.0;
        lfo_params[2].pitch_amount = 10.0; // ±10 cents

        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice.note_on(60, 1.0);
        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        // Process samples - the 3 LFOs should create complex modulation
        let mut samples = Vec::new();
        for _ in 0..1000 {
            let sample = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
            samples.push(sample.0);
        }

        // With 3 LFOs at different rates, we should see complex variation
        // (not just a simple sine wave pattern)
        let max_val = samples.iter().cloned().fold(0.0f32, f32::max);
        let min_val = samples.iter().cloned().fold(0.0f32, f32::min);

        // Verify we have both positive and negative values (complex waveform)
        assert!(
            max_val > 0.1 && min_val < -0.1,
            "Multiple LFO routing should create complex modulation pattern"
        );
    }

    #[test]
    fn test_lfo_routing_zero_amount_no_effect() {
        // Test that routing amount of 0.0 disables modulation
        let sample_rate = 44100.0;
        let mut voice1 = Voice::new(sample_rate);
        let mut voice2 = Voice::new(sample_rate);

        let mut osc_params = default_osc_params();
        osc_params[0].gain = 1.0;
        osc_params[0].waveform = Waveform::Sine;
        osc_params[0].gain = 1.0;

        let filter_params = default_filter_params();

        // Voice 1: LFO enabled but routing amount = 0.0
        let mut lfo_params1 = default_lfo_params();
        lfo_params1[0].depth = 1.0;
        lfo_params1[0].rate = 10.0;
        lfo_params1[0].pitch_amount = 0.0; // Disabled
        lfo_params1[0].gain_amount = 0.0; // Disabled
        lfo_params1[0].pan_amount = 0.0; // Disabled
        lfo_params1[0].pwm_amount = 0.0; // Disabled

        // Voice 2: LFO completely disabled
        let lfo_params2 = default_lfo_params();

        let envelope_params = default_envelope_params();
        let velocity_params = default_velocity_params();
        let wavetable_library = default_wavetable_library();

        voice1.note_on(60, 1.0);
        voice2.note_on(60, 1.0);

        voice1.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params1,
            &envelope_params,
            &wavetable_library,
        );
        voice2.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params2,
            &envelope_params,
            &wavetable_library,
        );

        // Process same number of samples on both voices
        for _ in 0..100 {
            let sample1 = voice1.process(
                &osc_params,
                &filter_params,
                &lfo_params1,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );
            let sample2 = voice2.process(
                &osc_params,
                &filter_params,
                &lfo_params2,
                &velocity_params,
                false,
                &default_voice_comp_params(),
                &default_transient_params(),
            );

            // With routing amounts at 0.0, both voices should produce identical output
            assert_relative_eq!(sample1.0, sample2.0, epsilon = 0.001);
            assert_relative_eq!(sample1.1, sample2.1, epsilon = 0.001);
        }
    }
}
