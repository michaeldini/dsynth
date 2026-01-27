//! Main Synthesizer Performance Benchmarks
//!
//! Tests the full polyphonic synthesizer under realistic and stress conditions.
//!
//! Performance Target: <11% CPU for 16 voices @ 44.1kHz
//!
//! Run: `cargo bench -- main_synth_perf`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dsynth::audio::{
    engine::{create_parameter_buffer, SynthEngine},
    voice::Voice,
};
use dsynth::params::{
    EnvelopeParams, FilterParams, FilterType, LFOParams, LFOWaveform, OscillatorParams,
    SynthParams, TransientShaperParams, VelocityParams, VoiceCompressorParams, Waveform,
};

/// Benchmark single voice processing with all features
fn bench_voice_processing(c: &mut Criterion) {
    let mut voice = Voice::new(44100.0);
    let osc_params = [OscillatorParams::default(); 3];
    let filter_params = [FilterParams::default(); 3];
    let lfo_params = [LFOParams::default(); 3];
    let envelope_params = EnvelopeParams::default();
    let velocity_params = VelocityParams::default();
    let voice_comp_params = VoiceCompressorParams::default();
    let transient_params = TransientShaperParams::default();
    let wavetable_library = dsynth::dsp::WavetableLibrary::new();

    voice.note_on(60, 0.8);
    voice.update_parameters(
        &osc_params,
        &filter_params,
        &lfo_params,
        &envelope_params,
        &wavetable_library,
    );

    c.bench_function("voice_process", |b| {
        b.iter(|| {
            black_box(voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                false,
                &voice_comp_params,
                &transient_params,
            ))
        });
    });
}

/// Benchmark engine polyphony scaling (8 voices)
fn bench_engine_8_voices(c: &mut Criterion) {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Trigger 8 voices
    for i in 0..8 {
        engine.note_on(60 + i, 0.8);
    }

    c.bench_function("engine_8_voices", |b| {
        b.iter(|| black_box(engine.process_mono()));
    });
}

/// Benchmark engine at full polyphony (16 voices)
fn bench_engine_16_voices(c: &mut Criterion) {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Trigger 16 voices (max polyphony)
    for i in 0..16 {
        engine.note_on(60 + i, 0.8);
    }

    c.bench_function("engine_16_voices", |b| {
        b.iter(|| black_box(engine.process_mono()));
    });
}

/// Benchmark engine with real-time parameter changes
///
/// Simulates filter cutoff modulation from automation/LFO
fn bench_engine_parameter_automation(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_parameter_automation");

    group.bench_function("16_voices_modulated_cutoff", |b| {
        let (mut producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Trigger 16 voices
        for i in 0..16 {
            engine.note_on(60 + i, 0.8);
        }

        let mut params = SynthParams::default();
        params.oscillators[0].unison = 4;
        params.filters[0].cutoff = 2000.0;

        let mut cutoff = 2000.0f32;
        b.iter(|| {
            // Simulate parameter changes (like from GUI or automation)
            cutoff = if cutoff < 5000.0 {
                cutoff + 5.0
            } else {
                2000.0
            };
            params.filters[0].cutoff = cutoff;
            producer.write(params);

            // Process samples (engine checks for updates every 32 samples)
            for _ in 0..32 {
                black_box(engine.process_mono());
            }
        });
    });

    group.finish();
}

/// Benchmark worst-case CPU usage (stress test)
///
/// Max unison + FM routing + hard sync + saturation + effects
fn bench_engine_stress_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_stress_test");

    group.bench_function("voice_worst_case", |b| {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60, 0.9);

        let mut osc_params = [OscillatorParams::default(); 3];
        for (i, p) in osc_params.iter_mut().enumerate() {
            p.gain = 0.7;
            p.waveform = if i == 0 {
                Waveform::Wavetable
            } else {
                Waveform::Saw
            };
            p.unison = 7;
            p.unison_detune = 80.0;
            p.unison_normalize = true;
            p.shape = 0.4;
            p.saturation = 1.0;
            p.wavetable_index = 0;
            p.wavetable_position = 0.5;
        }

        // Add FM routing to exercise process_with_fm paths
        osc_params[1].fm_source = Some(0);
        osc_params[1].fm_amount = 8.0;
        osc_params[2].fm_source = Some(1);
        osc_params[2].fm_amount = 6.0;

        let mut filter_params = [FilterParams::default(); 3];
        for fp in &mut filter_params {
            fp.filter_type = FilterType::Lowpass;
            fp.cutoff = 2000.0;
            fp.resonance = 3.0;
            fp.drive = 1.0;
            fp.post_drive = 1.0;
            fp.key_tracking = 0.7;
            fp.envelope.amount = 4000.0;
        }

        let mut lfo_params = [LFOParams::default(); 3];
        for lp in &mut lfo_params {
            lp.waveform = LFOWaveform::Sine;
            lp.rate = 5.0;
            lp.depth = 1.0;
            lp.filter_amount = 3000.0;
            lp.pitch_amount = 50.0;
            lp.gain_amount = 0.5;
            lp.pan_amount = 0.5;
            lp.pwm_amount = 0.5;
        }

        let envelope_params = EnvelopeParams::default();
        let velocity_params = VelocityParams::default();

        let voice_comp_params = VoiceCompressorParams {
            enabled: true,
            ..Default::default()
        };

        let transient_params = TransientShaperParams {
            enabled: true,
            attack_boost: 0.5,
            sustain_reduction: 0.25,
        };

        let wavetable_library = dsynth::dsp::WavetableLibrary::new();

        voice.update_parameters(
            &osc_params,
            &filter_params,
            &lfo_params,
            &envelope_params,
            &wavetable_library,
        );

        b.iter(|| {
            black_box(voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &velocity_params,
                true, // hard sync enabled
                &voice_comp_params,
                &transient_params,
            ))
        });
    });

    group.finish();
}

/// Benchmark block processing with different voice counts and buffer sizes
///
/// Tests scaling with realistic plugin buffer sizes (64, 256, 1024 samples)
fn bench_engine_block_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_block_scaling");

    for voices in [1usize, 4, 8, 16] {
        for block_size in [64usize, 256, 1024] {
            group.bench_with_input(
                BenchmarkId::new(format!("voices_{voices}"), block_size),
                &block_size,
                |b, &block_size| {
                    let (mut producer, consumer) = create_parameter_buffer();
                    let mut engine = SynthEngine::new(44100.0, consumer);

                    for i in 0..voices {
                        engine.note_on(60 + i as u8, 0.8);
                    }

                    // Apply realistic configuration
                    let mut params = SynthParams::default();
                    for (i, p) in params.oscillators.iter_mut().enumerate() {
                        p.gain = 0.7;
                        p.waveform = if i == 0 {
                            Waveform::Wavetable
                        } else {
                            Waveform::Saw
                        };
                        p.unison = 7;
                        p.unison_detune = 60.0;
                        p.unison_normalize = true;
                        p.shape = 0.2;
                        p.saturation = 0.7;
                        p.wavetable_index = 0;
                        p.wavetable_position = 0.3;
                    }
                    params.hard_sync_enabled = true;
                    params.voice_compressor.enabled = true;
                    params.transient_shaper.enabled = true;

                    producer.write(params);

                    let mut left = vec![0.0f32; block_size];
                    let mut right = vec![0.0f32; block_size];

                    // Warmup to ensure params propagate
                    engine.process_block(&mut left, &mut right);
                    engine.process_block(&mut left, &mut right);

                    b.iter(|| {
                        engine.process_block(&mut left, &mut right);
                        black_box(left[0]);
                        black_box(right[0]);
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    main_synth_perf_benches,
    bench_voice_processing,
    bench_engine_8_voices,
    bench_engine_16_voices,
    bench_engine_parameter_automation,
    bench_engine_stress_test,
    bench_engine_block_scaling,
);

criterion_main!(main_synth_perf_benches);
