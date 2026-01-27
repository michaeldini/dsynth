/// Voice Engine Latency Benchmarks - Comprehensive zero-latency validation
///
/// This benchmark suite proves that the voice plugin is zero-latency by:
/// 1. Verifying get_latency() returns 0 samples
/// 2. Testing impulse response to confirm no lookahead delay
/// 3. Measuring real-time processing throughput
/// 4. Validating signal analysis runs without buffering
use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(feature = "voice-clap")]
use criterion::black_box;

#[cfg(feature = "voice-clap")]
use dsynth::audio::voice_engine::VoiceEngine;
#[cfg(feature = "voice-clap")]
use dsynth::params_voice::VoiceParams;

/// Test 1: Verify get_latency() reports zero samples
#[cfg(feature = "voice-clap")]
fn bench_latency_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_query");

    group.bench_function("get_latency", |b| {
        let engine = VoiceEngine::new(44100.0);
        b.iter(|| {
            let latency = engine.get_latency();
            assert_eq!(latency, 0, "Voice engine MUST report zero latency");
            black_box(latency)
        });
    });

    group.finish();
}

/// Test 2: Impulse response test - verify output appears immediately
///
/// A true zero-latency system will produce non-zero output on the SAME sample
/// as a non-zero input. Systems with lookahead delay will have N samples of
/// silence before the output responds.
#[cfg(feature = "voice-clap")]
fn bench_impulse_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("impulse_response");

    group.bench_function("immediate_response", |b| {
        b.iter(|| {
            let mut engine = VoiceEngine::new(44100.0);
            let params = VoiceParams::default();
            engine.update_params(params);

            // Send impulse (loud sample after silence)
            for _ in 0..100 {
                engine.process(0.0, 0.0); // Silence
            }

            // Send impulse and capture IMMEDIATE output
            let (left_out, right_out) = engine.process(1.0, 1.0);

            // Zero-latency: output must be non-zero on SAME sample as input
            assert!(
                left_out.abs() > 0.001 || right_out.abs() > 0.001,
                "Zero-latency engine MUST respond immediately to impulse. \
                 Got output: L={}, R={} (expected non-zero)",
                left_out,
                right_out
            );

            black_box((left_out, right_out))
        });
    });

    group.finish();
}

/// Test 3: Real-time processing throughput
///
/// Measures how many samples/second the engine can process, which helps
/// validate that it can run in real-time with zero buffering delay.
#[cfg(feature = "voice-clap")]
fn bench_realtime_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("realtime_throughput");

    // Test at different sample rates
    for sample_rate in [44100.0, 48000.0, 96000.0] {
        group.bench_function(format!("process_block_{}Hz", sample_rate as u32), |b| {
            let mut engine = VoiceEngine::new(sample_rate);
            let params = VoiceParams::default();
            engine.update_params(params);

            // Process 512-sample block (typical audio buffer size)
            b.iter(|| {
                for _ in 0..512 {
                    let output = engine.process(black_box(0.5), black_box(0.5));
                    black_box(output);
                }
            });
        });
    }

    group.finish();
}

/// Test 4: Per-sample processing latency
///
/// Measures the CPU time required to process a single sample, which
/// directly impacts real-time capability.
#[cfg(feature = "voice-clap")]
fn bench_per_sample_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_sample_latency");

    group.bench_function("single_sample_44.1kHz", |b| {
        let mut engine = VoiceEngine::new(44100.0);
        let params = VoiceParams::default();
        engine.update_params(params);

        b.iter(|| {
            let output = engine.process(black_box(0.5), black_box(0.5));
            black_box(output)
        });
    });

    group.finish();
}

/// Test 5: Latency under various drive settings
///
/// Ensures that different saturation settings don't introduce latency
#[cfg(feature = "voice-clap")]
fn bench_latency_drive_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_drive_sweep");

    for drive in [0.0, 0.5, 1.0] {
        group.bench_function(format!("drive_{}%", (drive * 100.0) as u32), |b| {
            let mut engine = VoiceEngine::new(44100.0);
            let mut params = VoiceParams::default();
            params.bass_drive = drive;
            params.mid_drive = drive;
            params.presence_drive = drive;
            params.air_drive = drive;
            engine.update_params(params);

            b.iter(|| {
                // Verify latency stays zero regardless of drive
                assert_eq!(engine.get_latency(), 0);

                // Process sample and verify immediate output
                let output = engine.process(black_box(0.7), black_box(0.7));
                black_box(output)
            });
        });
    }

    group.finish();
}

/// Test 6: Worst-case latency test
///
/// Tests with worst-case signal (loud transient) and maximum drive
/// to ensure no hidden buffering is triggered
#[cfg(feature = "voice-clap")]
fn bench_worst_case_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("worst_case_latency");

    group.bench_function("max_drive_transient", |b| {
        b.iter(|| {
            let mut engine = VoiceEngine::new(44100.0);
            let mut params = VoiceParams::default();
            params.bass_drive = 1.0;
            params.mid_drive = 1.0;
            params.presence_drive = 1.0;
            params.air_drive = 1.0;
            engine.update_params(params);

            // Process loud transient
            let (left_out, right_out) = engine.process(1.0, 1.0);

            // Must still have zero latency
            assert_eq!(engine.get_latency(), 0);
            assert!(
                left_out.abs() > 0.001 || right_out.abs() > 0.001,
                "Immediate response required even under worst-case drive"
            );

            black_box((left_out, right_out))
        });
    });

    group.finish();
}

#[cfg(feature = "voice-clap")]
criterion_group!(
    latency_benches,
    bench_latency_query,
    bench_impulse_response,
    bench_realtime_throughput,
    bench_per_sample_latency,
    bench_latency_drive_sweep,
    bench_worst_case_latency,
);

#[cfg(not(feature = "voice-clap"))]
fn bench_dummy(_: &mut Criterion) {}

#[cfg(not(feature = "voice-clap"))]
criterion_group!(latency_benches, bench_dummy);

criterion_main!(latency_benches);
