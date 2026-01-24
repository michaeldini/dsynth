/// Saturation Benchmarks - Performance validation for analog saturation plugin
///
/// Benchmarks validate:
/// 1. Per-character saturation processing speed
/// 2. 3-stage cascade overhead
/// 3. Signal analysis cost (without pitch detection)
/// 4. Full engine processing with target <100 samples latency @ 44.1kHz
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dsynth::audio::voice_engine::VoiceEngine;
use dsynth::dsp::effects::adaptive_saturator::{AdaptiveSaturator, SaturationCharacter};
use dsynth::dsp::signal_analyzer::{SignalAnalysis, SignalAnalyzer};
use dsynth::params_voice::VoiceParams;

/// Helper to create test analysis data
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

/// Benchmark: Adaptive saturator per character type
fn bench_saturator_per_character(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_saturator_characters");

    let sample_rate = 44100.0;
    let analysis = create_test_analysis();

    for character in [
        SaturationCharacter::Warm,
        SaturationCharacter::Smooth,
        SaturationCharacter::Punchy,
    ] {
        group.bench_with_input(
            BenchmarkId::new("process_sample", format!("{:?}", character)),
            &character,
            |b, &character| {
                let mut saturator = AdaptiveSaturator::new(sample_rate);
                b.iter(|| {
                    black_box(saturator.process(
                        black_box(0.5),
                        black_box(0.5),
                        black_box(0.7),
                        character,
                        &analysis,
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: 3-stage cascade overhead
fn bench_three_stage_cascade(c: &mut Criterion) {
    let mut group = c.benchmark_group("saturation_stages");
    let sample_rate = 44100.0;
    let analysis = create_test_analysis();

    group.bench_function("3_stage_cascade_warm", |b| {
        let mut saturator = AdaptiveSaturator::new(sample_rate);
        b.iter(|| {
            // This internally does 3-stage processing
            black_box(saturator.process(
                black_box(0.5),
                black_box(0.5),
                black_box(0.7),
                SaturationCharacter::Warm,
                &analysis,
            ))
        });
    });

    group.finish();
}

/// Benchmark: Signal analysis without pitch detection
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
fn bench_voice_engine_full_process(c: &mut Criterion) {
    let mut group = c.benchmark_group("voice_engine");

    group.bench_function("process_sample", |b| {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();
        params.saturation_drive = 0.5;
        params.saturation_character = 0; // Warm
        engine.update_params(params);

        b.iter(|| {
            black_box(engine.process(black_box(0.5), black_box(0.5)));
        });
    });

    group.bench_function("process_buffer_512", |b| {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();
        params.saturation_drive = 0.5;
        params.saturation_character = 1; // Smooth
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
fn bench_drive_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("drive_levels");

    for drive in [0.0, 0.5, 1.0] {
        group.bench_with_input(
            BenchmarkId::new("process_drive", format!("{}%", (drive * 100.0) as i32)),
            &drive,
            |b, &drive| {
                let mut engine = VoiceEngine::new(44100.0);
                let mut params = VoiceParams::default();
                params.saturation_drive = drive;
                params.saturation_character = 0; // Warm
                engine.update_params(params);

                b.iter(|| {
                    black_box(engine.process(black_box(0.5), black_box(0.5)));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_saturator_per_character,
    bench_three_stage_cascade,
    bench_signal_analysis_no_pitch,
    bench_voice_engine_full_process,
    bench_latency_measurement,
    bench_drive_levels,
);
criterion_main!(benches);
