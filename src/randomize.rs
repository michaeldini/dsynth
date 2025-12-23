use rand::Rng;

use crate::params::{FilterType, LFOWaveform, SynthParams, Waveform};

/// Generate randomized parameters for sound design exploration.
///
/// This is the single source of truth used by both the standalone GUI and the plugin GUI.
pub fn randomize_synth_params<R: Rng + ?Sized>(rng: &mut R) -> SynthParams {
    let waveforms = [
        Waveform::Sine,
        Waveform::Saw,
        Waveform::Square,
        Waveform::Triangle,
        Waveform::Pulse,
    ];
    let filter_types = [
        FilterType::Lowpass,
        FilterType::Highpass,
        FilterType::Bandpass,
    ];
    let lfo_waveforms = [
        LFOWaveform::Sine,
        LFOWaveform::Triangle,
        LFOWaveform::Square,
        LFOWaveform::Saw,
    ];

    let mut params = SynthParams::default();

    // Oscillators
    for osc in &mut params.oscillators {
        osc.waveform = waveforms[rng.gen_range(0..waveforms.len())];
        osc.pitch = rng.gen_range(-24.0f32..=24.0f32).round();
        osc.detune = rng.gen_range(-50.0f32..=50.0f32).round();
        osc.gain = rng.gen_range(0.2..=0.8);
        osc.pan = rng.gen_range(-1.0..=1.0);
        osc.unison = rng.gen_range(1..=7);
        osc.unison_detune = rng.gen_range(0.0..=50.0);
        osc.phase = rng.gen_range(0.0..=1.0);
        osc.shape = rng.gen_range(-0.8..=0.8);
        // Keep solo/other toggles deterministic (default).
    }

    // Filters - capped resonance to avoid self-oscillation distortion
    for filter in &mut params.filters {
        filter.filter_type = filter_types[rng.gen_range(0..filter_types.len())];
        filter.cutoff = rng.gen_range(200.0..=10000.0);
        filter.resonance = rng.gen_range(0.5..=3.0);  // Reduced max from 5.0 to 3.0
        filter.bandwidth = rng.gen_range(0.5..=3.0);
        filter.key_tracking = rng.gen_range(0.0..=1.0);
    }

    // LFOs - reduced filter modulation amount for less harsh sweeps
    for lfo in &mut params.lfos {
        lfo.waveform = lfo_waveforms[rng.gen_range(0..lfo_waveforms.len())];
        lfo.rate = rng.gen_range(0.1..=8.0);         // Slightly slower max
        lfo.depth = rng.gen_range(0.0..=0.8);        // Reduced from 1.0
        lfo.filter_amount = rng.gen_range(0.0..=2000.0);  // Reduced from 3000
    }

    // Velocity
    params.velocity.amp_sensitivity = rng.gen_range(0.3..=1.0);
    params.velocity.filter_sensitivity = rng.gen_range(0.0..=0.8);

    // ADSR Envelope - ensure reasonable attack time to prevent clicks
    params.envelope.attack = rng.gen_range(0.005..=0.5);  // 5ms-500ms attack
    params.envelope.decay = rng.gen_range(0.05..=1.0);    // 50ms-1s decay
    params.envelope.sustain = rng.gen_range(0.3..=0.9);   // 30%-90% sustain
    params.envelope.release = rng.gen_range(0.1..=2.0);   // 100ms-2s release

    // Master - slightly reduced range for safety
    params.master_gain = rng.gen_range(0.3..=0.6);

    params
}
