//! Kick Drum Synthesizer Performance Benchmarks
//!
//! Tests the monophonic kick synthesizer under various configurations.
//!
//! Performance Target: <5% CPU for monophonic @ 44.1kHz
//!
//! Run: `cargo bench --features kick-clap -- kick_synth_perf`

use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(feature = "kick-clap")]
use criterion::{black_box, BenchmarkId};
#[cfg(feature = "kick-clap")]
use dsynth::audio::kick_engine::KickEngine;
#[cfg(feature = "kick-clap")]
use dsynth::params_kick::{DistortionType, KickParams};
#[cfg(feature = "kick-clap")]
use parking_lot::Mutex;
#[cfg(feature = "kick-clap")]
use std::sync::Arc;

#[cfg(feature = "kick-clap")]
const SAMPLE_RATE: f32 = 44100.0;

/// Benchmark kick trigger and processing (clean settings)
#[cfg(feature = "kick-clap")]
fn bench_kick_clean(c: &mut Criterion) {
    c.bench_function("kick_trigger_clean", |b| {
        let mut params = KickParams::default();
        params.master_level = 0.8;
        params.distortion_enabled = false;
        params.clipper_enabled = true;
        params.clipper_threshold = 0.95;

        let params_arc = Arc::new(Mutex::new(params));
        let mut engine = KickEngine::new(SAMPLE_RATE, Arc::clone(&params_arc));

        b.iter(|| {
            engine.trigger(1.0);
            // Process attack phase (first 512 samples)
            for _ in 0..512 {
                black_box(engine.process_sample());
            }
        });
    });
}

/// Benchmark kick with maximum processing (stress test)
#[cfg(feature = "kick-clap")]
fn bench_kick_stress_test(c: &mut Criterion) {
    c.bench_function("kick_trigger_max_processing", |b| {
        let mut params = KickParams::default();
        params.osc1_level = 1.0;
        params.osc2_level = 0.8;
        params.amp_attack = 0.1;
        params.amp_decay = 400.0;
        params.distortion_enabled = true;
        params.distortion_amount = 1.0;
        params.distortion_type = DistortionType::Hard;
        params.mb_enabled = true;
        params.mb_sub_threshold = -24.0;
        params.mb_body_threshold = -18.0;
        params.mb_click_threshold = -14.0;
        params.mb_mix = 1.0;
        params.clipper_enabled = true;
        params.clipper_threshold = 0.85;
        params.master_level = 1.0;

        let params_arc = Arc::new(Mutex::new(params));
        let mut engine = KickEngine::new(SAMPLE_RATE, Arc::clone(&params_arc));

        b.iter(|| {
            engine.trigger(1.0);
            // Process attack phase (first 512 samples)
            for _ in 0..512 {
                black_box(engine.process_sample());
            }
        });
    });
}

/// Benchmark key tracking performance (chromatic playback)
#[cfg(feature = "kick-clap")]
fn bench_kick_key_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("kick_key_tracking");

    for key_tracking in [0.0, 0.5, 1.0] {
        group.bench_with_input(
            BenchmarkId::new("tracking", format!("{}%", (key_tracking * 100.0) as i32)),
            &key_tracking,
            |b, &tracking| {
                let mut params = KickParams::default();
                params.key_tracking = tracking;

                let params_arc = Arc::new(Mutex::new(params));
                let mut engine = KickEngine::new(SAMPLE_RATE, Arc::clone(&params_arc));

                b.iter(|| {
                    // Trigger kicks to test performance
                    for _ in [36, 48, 60, 72] {
                        engine.trigger(1.0);
                        for _ in 0..256 {
                            black_box(engine.process_sample());
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark distortion types
#[cfg(feature = "kick-clap")]
fn bench_kick_distortion_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("kick_distortion_types");

    for dist_type in [
        DistortionType::Soft,
        DistortionType::Hard,
        DistortionType::Tube,
        DistortionType::Foldback,
    ] {
        group.bench_with_input(
            BenchmarkId::new("distortion", format!("{:?}", dist_type)),
            &dist_type,
            |b, &dt| {
                let mut params = KickParams::default();
                params.distortion_enabled = true;
                params.distortion_amount = 0.8;
                params.distortion_type = dt;

                let params_arc = Arc::new(Mutex::new(params));
                let mut engine = KickEngine::new(SAMPLE_RATE, Arc::clone(&params_arc));

                b.iter(|| {
                    engine.trigger(1.0);
                    for _ in 0..512 {
                        black_box(engine.process_sample());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark block processing
#[cfg(feature = "kick-clap")]
fn bench_kick_block_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("kick_block_processing");

    for block_size in [64, 256, 1024] {
        group.bench_with_input(
            BenchmarkId::new("block_size", block_size),
            &block_size,
            |b, &size| {
                let params = Arc::new(Mutex::new(KickParams::default()));
                let mut engine = KickEngine::new(SAMPLE_RATE, Arc::clone(&params));

                b.iter(|| {
                    engine.trigger(1.0);
                    let mut buffer = vec![0.0f32; size];
                    engine.process_block(&mut buffer);
                    black_box(buffer[0]);
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "kick-clap")]
criterion_group!(
    kick_synth_perf_benches,
    bench_kick_clean,
    bench_kick_stress_test,
    bench_kick_key_tracking,
    bench_kick_distortion_types,
    bench_kick_block_processing,
);

#[cfg(not(feature = "kick-clap"))]
fn bench_dummy(_: &mut Criterion) {}

#[cfg(not(feature = "kick-clap"))]
criterion_group!(kick_synth_perf_benches, bench_dummy);

criterion_main!(kick_synth_perf_benches);
