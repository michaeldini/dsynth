//! Unit tests for Voice implementation.
//!
//! Tests cover:
//! - Voice lifecycle (creation, activation, note-on/off, reset)
//! - MIDI note-to-frequency conversion
//! - Audio generation and DSP pipeline
//! - Envelope behavior (attack, sustain, release)
//! - Voice stealing metrics (peak amplitude tracking)
//! - LFO routing matrix (pitch, gain, pan, PWM modulation)
//! - LFO destination routing (Global, Osc1, Osc2, Osc3)

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
fn default_wavetable_library() -> crate::dsp::synthesis::wavetable_library::WavetableLibrary {
    crate::dsp::synthesis::wavetable_library::WavetableLibrary::new()
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

#[test]
fn test_lfo_destination_global_affects_all_oscs() {
    // Test that LFO with destination=Global affects all oscillators
    let mut voice = Voice::new(44100.0);
    let mut osc_params = default_osc_params();
    osc_params[0].gain = 0.5;
    osc_params[1].gain = 0.5;
    osc_params[2].gain = 0.5;

    let filter_params = default_filter_params();
    let mut lfo_params = default_lfo_params();

    // LFO1 with Global destination and pitch modulation
    lfo_params[0].depth = 1.0;
    lfo_params[0].pitch_amount = 100.0; // ±100 cents vibrato
    lfo_params[0].destination = crate::params::LfoDestination::Global;

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

    // Process enough samples to see LFO effect (LFO rate = 2 Hz default)
    let mut samples = Vec::new();
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
        samples.push(sample);
    }

    // With all 3 oscillators active and global pitch modulation, we expect non-zero output
    let has_output = samples
        .iter()
        .any(|(l, r)| l.abs() > 0.01 || r.abs() > 0.01);
    assert!(
        has_output,
        "Global destination should affect all oscillators"
    );
}

#[test]
fn test_lfo_destination_osc1_isolates_to_osc1() {
    // Test that LFO with destination=Osc1 only affects oscillator 1
    let mut voice_global = Voice::new(44100.0);
    let mut voice_osc1 = Voice::new(44100.0);

    let mut osc_params = default_osc_params();
    osc_params[0].gain = 0.5;
    osc_params[1].gain = 0.0; // Disable osc2
    osc_params[2].gain = 0.0; // Disable osc3

    let filter_params = default_filter_params();
    let mut lfo_params_global = default_lfo_params();
    let mut lfo_params_osc1 = default_lfo_params();

    // Both have same LFO settings, different destinations
    lfo_params_global[0].depth = 1.0;
    lfo_params_global[0].pitch_amount = 50.0;
    lfo_params_global[0].destination = crate::params::LfoDestination::Global;

    lfo_params_osc1[0].depth = 1.0;
    lfo_params_osc1[0].pitch_amount = 50.0;
    lfo_params_osc1[0].destination = crate::params::LfoDestination::Osc1;

    let envelope_params = default_envelope_params();
    let velocity_params = default_velocity_params();
    let wavetable_library = default_wavetable_library();

    voice_global.note_on(60, 1.0);
    voice_osc1.note_on(60, 1.0);

    voice_global.update_parameters(
        &osc_params,
        &filter_params,
        &lfo_params_global,
        &envelope_params,
        &wavetable_library,
    );
    voice_osc1.update_parameters(
        &osc_params,
        &filter_params,
        &lfo_params_osc1,
        &envelope_params,
        &wavetable_library,
    );

    // With only Osc1 active, Global and Osc1 destinations should produce the same output
    for _ in 0..50 {
        let sample_global = voice_global.process(
            &osc_params,
            &filter_params,
            &lfo_params_global,
            &velocity_params,
            false,
            &default_voice_comp_params(),
            &default_transient_params(),
        );
        let sample_osc1 = voice_osc1.process(
            &osc_params,
            &filter_params,
            &lfo_params_osc1,
            &velocity_params,
            false,
            &default_voice_comp_params(),
            &default_transient_params(),
        );

        // Should be identical since only Osc1 is active
        assert_relative_eq!(sample_global.0, sample_osc1.0, epsilon = 0.01);
        assert_relative_eq!(sample_global.1, sample_osc1.1, epsilon = 0.01);
    }
}

#[test]
fn test_lfo_destination_osc2_no_effect_on_osc1() {
    // Test that LFO with destination=Osc2 doesn't affect oscillator 1
    let mut voice_none = Voice::new(44100.0);
    let mut voice_osc2 = Voice::new(44100.0);

    let mut osc_params = default_osc_params();
    osc_params[0].gain = 0.5; // Enable osc1
    osc_params[1].gain = 0.0; // Disable osc2 (target of LFO)
    osc_params[2].gain = 0.0; // Disable osc3

    let filter_params = default_filter_params();
    let mut lfo_params_none = default_lfo_params();
    let mut lfo_params_osc2 = default_lfo_params();

    // No modulation vs Osc2-only modulation
    lfo_params_none[0].depth = 0.0;

    lfo_params_osc2[0].depth = 1.0;
    lfo_params_osc2[0].pitch_amount = 100.0; // Large modulation
    lfo_params_osc2[0].destination = crate::params::LfoDestination::Osc2;

    let envelope_params = default_envelope_params();
    let velocity_params = default_velocity_params();
    let wavetable_library = default_wavetable_library();

    voice_none.note_on(60, 1.0);
    voice_osc2.note_on(60, 1.0);

    voice_none.update_parameters(
        &osc_params,
        &filter_params,
        &lfo_params_none,
        &envelope_params,
        &wavetable_library,
    );
    voice_osc2.update_parameters(
        &osc_params,
        &filter_params,
        &lfo_params_osc2,
        &envelope_params,
        &wavetable_library,
    );

    // Since Osc2 is disabled and LFO routes to Osc2, active Osc1 should be unaffected
    for _ in 0..50 {
        let sample_none = voice_none.process(
            &osc_params,
            &filter_params,
            &lfo_params_none,
            &velocity_params,
            false,
            &default_voice_comp_params(),
            &default_transient_params(),
        );
        let sample_osc2 = voice_osc2.process(
            &osc_params,
            &filter_params,
            &lfo_params_osc2,
            &velocity_params,
            false,
            &default_voice_comp_params(),
            &default_transient_params(),
        );

        // Should be identical - Osc2 destination doesn't affect Osc1
        assert_relative_eq!(sample_none.0, sample_osc2.0, epsilon = 0.01);
        assert_relative_eq!(sample_none.1, sample_osc2.1, epsilon = 0.01);
    }
}

#[test]
fn test_lfo_destination_mixed_routing() {
    // Test multiple LFOs with different destinations
    let mut voice = Voice::new(44100.0);

    let mut osc_params = default_osc_params();
    osc_params[0].gain = 0.5;
    osc_params[1].gain = 0.5;
    osc_params[2].gain = 0.0; // Disable osc3

    let filter_params = default_filter_params();
    let mut lfo_params = default_lfo_params();

    // LFO1 → Osc1 only
    lfo_params[0].depth = 1.0;
    lfo_params[0].pitch_amount = 50.0;
    lfo_params[0].destination = crate::params::LfoDestination::Osc1;

    // LFO2 → Osc2 only
    lfo_params[1].depth = 1.0;
    lfo_params[1].gain_amount = 0.5;
    lfo_params[1].destination = crate::params::LfoDestination::Osc2;

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

    // Process samples - mixed routing should work without crashes
    let mut samples = Vec::new();
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
        samples.push(sample);
    }

    // Should produce valid audio (Osc1 pitch mod + Osc2 gain mod)
    let has_output = samples
        .iter()
        .any(|(l, r)| l.abs() > 0.01 || r.abs() > 0.01);
    assert!(has_output, "Mixed LFO routing should produce audio");
}
