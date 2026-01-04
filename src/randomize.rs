use rand::Rng;

use crate::params::{FilterType, LFOWaveform, SynthParams, Waveform};

/// Generate randomized parameters for sound design exploration.
///
/// This is the single source of truth used by both the standalone GUI and the plugin GUI.
pub fn randomize_synth_params<R: Rng + ?Sized>(rng: &mut R) -> SynthParams {
    use crate::params::DistortionType;

    let waveforms = [
        Waveform::Sine,
        Waveform::Saw,
        Waveform::Square,
        Waveform::Triangle,
        Waveform::Pulse,
        Waveform::WhiteNoise,
        Waveform::PinkNoise,
        Waveform::Additive,
        Waveform::Wavetable,
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
    let distortion_types = [
        DistortionType::Tanh,
        DistortionType::SoftClip,
        DistortionType::HardClip,
        DistortionType::Cubic,
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
        osc.unison_detune = rng.gen_range(0.0..=100.0);
        osc.unison_normalize = rng.gen_bool(0.7); // 70% chance to normalize (prevent clipping)
        osc.phase = rng.gen_range(0.0..=1.0);
        osc.shape = rng.gen_range(-0.8..=0.8);
        osc.saturation = rng.gen_range(0.0..=0.5); // Moderate oscillator saturation

        // FM synthesis parameters
        osc.fm_source = if rng.gen_bool(0.3) {
            Some(rng.gen_range(0..3)) // Random FM source (osc 0, 1, or 2)
        } else {
            None // 70% chance of no FM
        };
        osc.fm_amount = if osc.fm_source.is_some() {
            rng.gen_range(0.0..=5.0) // Moderate FM amount when enabled
        } else {
            0.0
        };

        // Additive synthesis harmonics (for Additive waveform)
        if osc.waveform == Waveform::Additive {
            for i in 0..8 {
                osc.additive_harmonics[i] = rng.gen_range(0.0..=1.0);
            }
        }

        // Wavetable parameters (for Wavetable waveform)
        if osc.waveform == Waveform::Wavetable {
            osc.wavetable_index = rng.gen_range(0..=10); // Assuming 10+ wavetables available
            osc.wavetable_position = rng.gen_range(0.0..=1.0);
        }

        // Keep solo/other toggles deterministic (default).
    }

    // Filters - capped resonance to avoid self-oscillation distortion
    for filter in &mut params.filters {
        filter.filter_type = filter_types[rng.gen_range(0..filter_types.len())];
        filter.cutoff = rng.gen_range(200.0..=10000.0);
        filter.resonance = rng.gen_range(0.5..=3.0); // Reduced max from 5.0 to 3.0
        filter.bandwidth = rng.gen_range(0.5..=3.0);
        filter.key_tracking = rng.gen_range(0.0..=1.0);
        filter.drive = rng.gen_range(0.0..=0.5); // Moderate pre-filter saturation
        filter.post_drive = rng.gen_range(0.0..=0.5); // Moderate post-filter saturation

        // Filter envelope
        filter.envelope.attack = rng.gen_range(0.01..=0.3);
        filter.envelope.decay = rng.gen_range(0.05..=1.0);
        filter.envelope.sustain = rng.gen_range(0.2..=0.8);
        filter.envelope.release = rng.gen_range(0.1..=1.5);
        filter.envelope.amount = if rng.gen_bool(0.5) {
            rng.gen_range(-3000.0..=3000.0) // 50% chance of filter envelope modulation
        } else {
            0.0
        };
    }

    // LFOs - bipolar modulation amounts for more expressive control
    for lfo in &mut params.lfos {
        lfo.waveform = lfo_waveforms[rng.gen_range(0..lfo_waveforms.len())];
        lfo.rate = rng.gen_range(0.1..=8.0); // Slightly slower max
        lfo.depth = rng.gen_range(0.0..=0.8); // Reduced from 1.0
        lfo.filter_amount = rng.gen_range(-2000.0..=2000.0); // Bipolar filter modulation
        lfo.pitch_amount = rng.gen_range(-50.0..=50.0); // Bipolar pitch modulation in cents
        lfo.gain_amount = rng.gen_range(-0.5..=0.5); // Bipolar gain modulation
        lfo.pan_amount = rng.gen_range(0.0..=0.8); // Pan modulation
        lfo.pwm_amount = rng.gen_range(0.0..=0.7); // PWM/shape modulation
    }

    // Velocity
    params.velocity.amp_sensitivity = rng.gen_range(0.3..=1.0);
    params.velocity.filter_sensitivity = rng.gen_range(0.0..=0.8);

    // ADSR Envelope - ensure reasonable attack time to prevent clicks
    params.envelope.attack = rng.gen_range(0.005..=0.5); // 5ms-500ms attack
    params.envelope.decay = rng.gen_range(0.05..=1.0); // 50ms-1s decay
    params.envelope.sustain = rng.gen_range(0.3..=0.9); // 30%-90% sustain
    params.envelope.release = rng.gen_range(0.1..=2.0); // 100ms-2s release
    params.envelope.attack_curve = rng.gen_range(-1.0..=1.0); // Full curve range
    params.envelope.decay_curve = rng.gen_range(-1.0..=1.0);
    params.envelope.release_curve = rng.gen_range(-1.0..=1.0);

    // Master - slightly reduced range for safety
    params.master_gain = rng.gen_range(0.3..=0.6);
    params.monophonic = rng.gen_bool(0.2); // 20% chance of monophonic mode
    params.hard_sync_enabled = rng.gen_bool(0.15); // 15% chance of hard sync for bright harmonics

    // Voice-level processing
    params.voice_compressor.enabled = rng.gen_bool(0.3); // 30% chance enabled
    if params.voice_compressor.enabled {
        params.voice_compressor.threshold = rng.gen_range(-30.0..=-6.0);
        params.voice_compressor.ratio = rng.gen_range(2.0..=8.0);
        params.voice_compressor.attack = rng.gen_range(0.5..=10.0);
        params.voice_compressor.release = rng.gen_range(20.0..=150.0);
        params.voice_compressor.knee = rng.gen_range(0.0..=12.0);
        params.voice_compressor.makeup_gain = rng.gen_range(0.0..=15.0);
    }

    params.transient_shaper.enabled = rng.gen_bool(0.25); // 25% chance enabled
    if params.transient_shaper.enabled {
        params.transient_shaper.attack_boost = rng.gen_range(0.0..=0.6);
        params.transient_shaper.sustain_reduction = rng.gen_range(0.0..=0.5);
    }

    // Effects - add some variation while keeping it musical
    // Reverb
    params.effects.reverb.enabled = rng.gen_bool(0.5); // 50% chance enabled
    params.effects.reverb.room_size = rng.gen_range(0.2..=0.9);
    params.effects.reverb.damping = rng.gen_range(0.2..=0.8);
    params.effects.reverb.wet = rng.gen_range(0.1..=0.5);
    params.effects.reverb.dry = rng.gen_range(0.5..=1.0);
    params.effects.reverb.width = rng.gen_range(0.5..=1.0);

    // Delay
    params.effects.delay.enabled = rng.gen_bool(0.4); // 40% chance enabled
    params.effects.delay.time_ms = rng.gen_range(100.0..=1000.0);
    params.effects.delay.feedback = rng.gen_range(0.1..=0.7);
    params.effects.delay.wet = rng.gen_range(0.1..=0.4);
    params.effects.delay.dry = rng.gen_range(0.6..=1.0);

    // Chorus
    params.effects.chorus.enabled = rng.gen_bool(0.35); // 35% chance enabled
    params.effects.chorus.rate = rng.gen_range(0.2..=3.0);
    params.effects.chorus.depth = rng.gen_range(0.2..=0.8);
    params.effects.chorus.mix = rng.gen_range(0.2..=0.6);

    // Distortion
    params.effects.distortion.enabled = rng.gen_bool(0.3); // 30% chance enabled
    params.effects.distortion.dist_type =
        distortion_types[rng.gen_range(0..distortion_types.len())];
    params.effects.distortion.drive = rng.gen_range(0.0..=0.6); // Moderate distortion
    params.effects.distortion.mix = rng.gen_range(0.3..=0.7);

    // Multiband Distortion
    params.effects.multiband_distortion.enabled = rng.gen_bool(0.2); // 20% chance enabled
    if params.effects.multiband_distortion.enabled {
        params.effects.multiband_distortion.low_mid_freq = rng.gen_range(150.0..=400.0);
        params.effects.multiband_distortion.mid_high_freq = rng.gen_range(1500.0..=4000.0);
        params.effects.multiband_distortion.drive_low = rng.gen_range(0.0..=0.6);
        params.effects.multiband_distortion.drive_mid = rng.gen_range(0.0..=0.6);
        params.effects.multiband_distortion.drive_high = rng.gen_range(0.0..=0.5);
        params.effects.multiband_distortion.gain_low = rng.gen_range(0.7..=1.3);
        params.effects.multiband_distortion.gain_mid = rng.gen_range(0.7..=1.3);
        params.effects.multiband_distortion.gain_high = rng.gen_range(0.7..=1.3);
        params.effects.multiband_distortion.mix = rng.gen_range(0.3..=0.7);
    }

    // Stereo Widener
    params.effects.stereo_widener.enabled = rng.gen_bool(0.3); // 30% chance enabled
    if params.effects.stereo_widener.enabled {
        params.effects.stereo_widener.haas_delay_ms = rng.gen_range(2.0..=15.0);
        params.effects.stereo_widener.haas_mix = rng.gen_range(0.2..=0.6);
        params.effects.stereo_widener.width = rng.gen_range(1.0..=1.8);
        params.effects.stereo_widener.mid_gain = rng.gen_range(0.8..=1.2);
        params.effects.stereo_widener.side_gain = rng.gen_range(1.0..=1.5);
    }

    // Phaser
    params.effects.phaser.enabled = rng.gen_bool(0.25); // 25% chance enabled
    if params.effects.phaser.enabled {
        params.effects.phaser.rate = rng.gen_range(0.2..=4.0);
        params.effects.phaser.depth = rng.gen_range(0.3..=0.8);
        params.effects.phaser.feedback = rng.gen_range(-0.7..=0.7);
        params.effects.phaser.mix = rng.gen_range(0.3..=0.6);
    }

    // Flanger
    params.effects.flanger.enabled = rng.gen_bool(0.2); // 20% chance enabled
    if params.effects.flanger.enabled {
        params.effects.flanger.rate = rng.gen_range(0.2..=3.0);
        params.effects.flanger.depth = rng.gen_range(0.3..=0.7);
        params.effects.flanger.feedback = rng.gen_range(-0.7..=0.7);
        params.effects.flanger.mix = rng.gen_range(0.3..=0.6);
    }

    // Tremolo
    params.effects.tremolo.enabled = rng.gen_bool(0.2); // 20% chance enabled
    if params.effects.tremolo.enabled {
        params.effects.tremolo.rate = rng.gen_range(2.0..=12.0);
        params.effects.tremolo.depth = rng.gen_range(0.3..=0.7);
    }

    // Auto Pan
    params.effects.auto_pan.enabled = rng.gen_bool(0.15); // 15% chance enabled
    if params.effects.auto_pan.enabled {
        params.effects.auto_pan.rate = rng.gen_range(0.3..=2.0);
        params.effects.auto_pan.depth = rng.gen_range(0.4..=0.8);
    }

    // Comb Filter
    params.effects.comb_filter.enabled = rng.gen_bool(0.15); // 15% chance enabled
    if params.effects.comb_filter.enabled {
        params.effects.comb_filter.frequency = rng.gen_range(100.0..=2000.0);
        params.effects.comb_filter.feedback = rng.gen_range(-0.7..=0.7);
        params.effects.comb_filter.mix = rng.gen_range(0.3..=0.6);
    }

    // Ring Modulator
    params.effects.ring_mod.enabled = rng.gen_bool(0.15); // 15% chance enabled
    if params.effects.ring_mod.enabled {
        params.effects.ring_mod.frequency = rng.gen_range(100.0..=3000.0);
        params.effects.ring_mod.depth = rng.gen_range(0.3..=0.7);
    }

    // Compressor
    params.effects.compressor.enabled = rng.gen_bool(0.3); // 30% chance enabled
    if params.effects.compressor.enabled {
        params.effects.compressor.threshold = rng.gen_range(-30.0..=-10.0);
        params.effects.compressor.ratio = rng.gen_range(2.0..=8.0);
        params.effects.compressor.attack = rng.gen_range(5.0..=30.0);
        params.effects.compressor.release = rng.gen_range(50.0..=200.0);
    }

    // Bitcrusher
    params.effects.bitcrusher.enabled = rng.gen_bool(0.2); // 20% chance enabled
    if params.effects.bitcrusher.enabled {
        params.effects.bitcrusher.sample_rate = rng.gen_range(2000.0..=22050.0);
        params.effects.bitcrusher.bit_depth = rng.gen_range(4..=12);
    }

    // Waveshaper
    params.effects.waveshaper.enabled = rng.gen_bool(0.25); // 25% chance enabled
    if params.effects.waveshaper.enabled {
        params.effects.waveshaper.drive = rng.gen_range(1.5..=6.0);
        params.effects.waveshaper.mix = rng.gen_range(0.3..=0.6);
    }

    // Exciter
    params.effects.exciter.enabled = rng.gen_bool(0.25); // 25% chance enabled
    if params.effects.exciter.enabled {
        params.effects.exciter.frequency = rng.gen_range(3000.0..=9000.0);
        params.effects.exciter.drive = rng.gen_range(0.3..=0.7);
        params.effects.exciter.mix = rng.gen_range(0.2..=0.5);
    }

    params
}
