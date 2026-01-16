use criterion::{BenchmarkId, Criterion, black_box};
use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
use dsynth::audio::voice::Voice;
use dsynth::dsp::filter::BiquadFilter;
use dsynth::params::{
    FilterType, LFOWaveform, OscillatorParams, SynthParams, TransientShaperParams, VelocityParams,
    VoiceCompressorParams, Waveform,
};

/// Benchmark filter with dynamic modulation (realistic scenario)
/// This tests the actual optimization benefit of quantized coefficient updates
fn benchmark_filter_with_modulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_modulation");

    // Simulate LFO-modulated cutoff (changes every sample)
    group.bench_function("filter_modulated_cutoff", |b| {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_filter_type(FilterType::Lowpass);
        filter.set_resonance(2.0);

        b.iter(|| {
            // Simulate cutoff modulation every sample
            for i in 0..100 {
                let modulated = 1000.0 + (i as f32 * 10.0).sin() * 500.0;
                black_box(filter.set_cutoff(modulated));
                black_box(filter.process(black_box(0.5)));
            }
        });
    });

    group.finish();
}

/// Benchmark the cost impact of filter coefficient update rate.
///
/// We vary an internal `cutoff_update_interval` and measure:
/// - `set_cutoff_only_*`: isolates coefficient update overhead
/// - `set_cutoff_plus_process_*`: end-to-end cost (closer to real DSP loop)
fn benchmark_filter_coefficient_update_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_coeff_update_rate");

    for interval in [1u8, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("set_cutoff_only", interval),
            &interval,
            |b, &interval| {
                let mut filter = BiquadFilter::new(44100.0);
                filter.set_cutoff_update_interval(interval);
                filter.set_filter_type(FilterType::Lowpass);
                filter.set_resonance(2.0);

                b.iter(|| {
                    for i in 0..1024 {
                        let modulated = 1000.0 + (i as f32 * 0.01).sin() * 500.0;
                        black_box(filter.set_cutoff(modulated));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("set_cutoff_plus_process", interval),
            &interval,
            |b, &interval| {
                let mut filter = BiquadFilter::new(44100.0);
                filter.set_cutoff_update_interval(interval);
                filter.set_filter_type(FilterType::Lowpass);
                filter.set_resonance(2.0);

                b.iter(|| {
                    for i in 0..1024 {
                        let modulated = 1000.0 + (i as f32 * 0.01).sin() * 500.0;
                        black_box(filter.set_cutoff(modulated));
                        black_box(filter.process(black_box(0.5)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark engine with changing parameters (realistic scenario)
fn benchmark_engine_with_parameter_changes(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_parameter_changes");

    group.bench_function("engine_16_voices_param_changes", |b| {
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

    group.bench_function("engine_16_voices_no_param_changes", |b| {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Trigger 16 voices
        for i in 0..16 {
            engine.note_on(60 + i, 0.8);
        }

        b.iter(|| {
            // No parameter writes; measures steady-state processing + periodic buffer read
            for _ in 0..32 {
                black_box(engine.process_mono());
            }
        });
    });

    group.finish();
}

/// Benchmark a single voice in a deliberately "worst-case" configuration.
///
/// Goal: capture CPU cliffs from max unison, FM routing, hard-sync, modulation, and saturation.
fn benchmark_voice_worst_case(c: &mut Criterion) {
    use dsynth::params::{EnvelopeParams, FilterParams, LFOParams};

    let mut group = c.benchmark_group("voice_worst_case");

    group.bench_function("voice_process_worst_case", |b| {
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

        // Add FM routing to exercise process_with_fm paths.
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

        let mut voice_comp_params = VoiceCompressorParams::default();
        voice_comp_params.enabled = true;

        let mut transient_params = TransientShaperParams::default();
        transient_params.enabled = true;
        transient_params.attack_boost = 0.5;
        transient_params.sustain_reduction = 0.25;

        let wavetable_library = dsynth::dsp::wavetable_library::WavetableLibrary::new();

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
                true, // hard sync
                &voice_comp_params,
                &transient_params,
            ))
        });
    });

    group.finish();
}

/// Benchmark engine `process_block` cost by voice count and block size.
///
/// This matches plugin reality better than per-sample process() and highlights scaling.
fn benchmark_engine_block_scaling(c: &mut Criterion) {
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

                    // Apply a heavier-than-default synth configuration once (not in the loop).
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

                    // Keep effects disabled here; this bench focuses on voice + mixing scaling.
                    producer.write(params);

                    let mut left = vec![0.0f32; block_size];
                    let mut right = vec![0.0f32; block_size];

                    // Warmup to ensure params propagate (engine checks every 32 samples).
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

criterion::criterion_group!(
    benches,
    benchmark_filter_with_modulation,
    benchmark_filter_coefficient_update_rate,
    benchmark_engine_with_parameter_changes,
    benchmark_voice_worst_case,
    benchmark_engine_block_scaling
);
criterion::criterion_main!(benches);
