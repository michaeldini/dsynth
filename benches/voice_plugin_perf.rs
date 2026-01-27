/// Saturation Benchmarks - Performance validation for 4-band adaptive saturator
///
/// Benchmarks validate:
/// 1. 4-band saturation processing speed
/// 2. Multi-stage cascade overhead
/// 3. Signal analysis cost (without pitch detection)
/// 4. Full engine processing with target <15% CPU @ 44.1kHz
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

#[cfg(feature = "voice-clap")]
use criterion::black_box;

#[cfg(feature = "voice-clap")]
use dsynth::audio::voice_engine::VoiceEngine;
#[cfg(feature = "voice-clap")]
use dsynth::dsp::effects::adaptive_saturator::AdaptiveSaturator;
#[cfg(feature = "voice-clap")]
use dsynth::dsp::signal_analyzer::{SignalAnalysis, SignalAnalyzer};
#[cfg(feature = "voice-clap")]
use dsynth::params_voice::VoiceParams;

/// Helper to create test analysis data
#[cfg(feature = "voice-clap")]
fn create_test_analysis() -> SignalAnalysis {
    SignalAnalysis {
        rms_level: 0.3,
        peak_level: 0.5,
        is_transient: false,
        transient_strength: 0.0,
        zcr_hz: 200.0,
        signal_type: dsynth::dsp::analysis::SignalType::Tonal,
        is_voiced: true,
        is_unvoiced: false,
        has_sibilance: false,
        sibilance_strength: 0.0,
        pitch_hz: 220.0,
        pitch_confidence: 0.8,
        is_pitched: true,
    }
}

/// Benchmark: Adaptive saturator with different drive levels
#[cfg(feature = "voice-clap")]
fn bench_saturator_per_character(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_saturator_drive_levels");

    let sample_rate = 44100.0;
    let analysis = create_test_analysis();

    // Test different drive levels (low, medium, high)
    for drive_level in [0.3, 0.6, 0.9] {
        group.bench_with_input(
            BenchmarkId::new("process_sample", format!("{:.0}%", drive_level * 100.0)),
            &drive_level,
            |b, &drive| {
                let mut saturator = AdaptiveSaturator::new(sample_rate);
                b.iter(|| {
                    black_box(saturator.process(
                        black_box(0.5),   // left
                        black_box(0.5),   // right
                        black_box(drive), // bass_drive
                        black_box(0.5),   // bass_mix
                        black_box(drive), // mid_drive
                        black_box(0.4),   // mid_mix
                        black_box(drive), // presence_drive
                        black_box(0.35),  // presence_mix
                        black_box(drive), // air_drive
                        black_box(0.15),  // air_mix
                        black_box(0.0),   // stereo_width
                        &analysis,
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: 3-stage cascade overhead
#[cfg(feature = "voice-clap")]
fn bench_three_stage_cascade(c: &mut Criterion) {
    let mut group = c.benchmark_group("saturation_stages");
    let sample_rate = 44100.0;
    let analysis = create_test_analysis();

    group.bench_function("3_stage_cascade", |b| {
        let mut saturator = AdaptiveSaturator::new(sample_rate);
        b.iter(|| {
            // This internally does 3-stage processing with 4-band saturation
            black_box(saturator.process(
                black_box(0.5),  // left
                black_box(0.5),  // right
                black_box(0.7),  // bass_drive
                black_box(0.5),  // bass_mix
                black_box(0.6),  // mid_drive
                black_box(0.4),  // mid_mix
                black_box(0.4),  // presence_drive
                black_box(0.35), // presence_mix
                black_box(0.1),  // air_drive
                black_box(0.15), // air_mix
                black_box(0.0),  // stereo_width
                &analysis,
            ))
        });
    });

    group.finish();
}

/// Benchmark: Signal analysis without pitch detection
#[cfg(feature = "voice-clap")]
fn bench_signal_analysis_no_pitch(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_analysis");

    group.bench_function("analyze_no_pitch", |b| {
        let mut analyzer = SignalAnalyzer::new_no_pitch(44100.0);
        b.iter(|| {
            black_box(analyzer.analyze(black_box(0.5), black_box(0.5)));
        });
    });

    // Compare with pitch detection enabled (for reference)
    group.bench_function("analyze_with_pitch", |b| {
        let mut analyzer = SignalAnalyzer::new(44100.0);
        b.iter(|| {
            black_box(analyzer.analyze(black_box(0.5), black_box(0.5)));
        });
    });

    group.finish();
}

/// Benchmark: Full voice engine processing
#[cfg(feature = "voice-clap")]
fn bench_voice_engine_full_process(c: &mut Criterion) {
    let mut group = c.benchmark_group("voice_engine");

    group.bench_function("process_sample", |b| {
        let mut engine = VoiceEngine::new(44100.0);
        let params = VoiceParams::default();
        engine.update_params(params);

        b.iter(|| {
            black_box(engine.process(black_box(0.5), black_box(0.5)));
        });
    });

    group.bench_function("process_buffer_512", |b| {
        let mut engine = VoiceEngine::new(44100.0);
        let params = VoiceParams::default();
        engine.update_params(params);

        let frame_count = 512;
        let input_left = vec![0.5; frame_count];
        let input_right = vec![0.5; frame_count];
        let mut output_left = vec![0.0; frame_count];
        let mut output_right = vec![0.0; frame_count];

        b.iter(|| {
            engine.process_buffer(
                black_box(&input_left),
                black_box(&input_right),
                black_box(&mut output_left),
                black_box(&mut output_right),
                black_box(frame_count),
            );
        });
    });

    group.finish();
}

/// Benchmark: Latency validation
#[cfg(feature = "voice-clap")]
fn bench_latency_measurement(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency");

    group.bench_function("engine_latency", |b| {
        let engine = VoiceEngine::new(44100.0);
        b.iter(|| {
            let latency = engine.get_latency();
            assert_eq!(latency, 0, "Latency should be 0 samples");
            black_box(latency)
        });
    });

    group.finish();
}

/// Benchmark: Drive levels (0%, 50%, 100%)
#[cfg(feature = "voice-clap")]
fn bench_drive_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("drive_levels");

    for drive in [0.0, 0.5, 1.0] {
        group.bench_with_input(
            BenchmarkId::new("process_drive", format!("{}%", (drive * 100.0) as i32)),
            &drive,
            |b, &drive| {
                let mut engine = VoiceEngine::new(44100.0);
                let mut params = VoiceParams::default();
                // Set all drive params to test value
                params.bass_drive = drive;
                params.mid_drive = drive;
                params.presence_drive = drive;
                params.air_drive = drive;
                engine.update_params(params);

                b.iter(|| {
                    black_box(engine.process(black_box(0.5), black_box(0.5)));
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "voice-clap")]
criterion_group!(
    benches,
    bench_saturator_per_character,
    bench_three_stage_cascade,
    bench_signal_analysis_no_pitch,
    bench_voice_engine_full_process,
    bench_latency_measurement,
    bench_drive_levels,
);

#[cfg(not(feature = "voice-clap"))]
fn bench_dummy(_: &mut Criterion) {}

#[cfg(not(feature = "voice-clap"))]
criterion_group!(benches, bench_dummy);

criterion_main!(benches);
