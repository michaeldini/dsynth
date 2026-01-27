//! Core DSP Component Benchmarks
//!
//! Tests individual building blocks in isolation (oscillators, filters, envelopes).
//! These are the fastest benchmarks - run often during development.
//!
//! Run: `cargo bench -- dsp_primitives`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dsynth::dsp::{BiquadFilter, Envelope, Oscillator};
use dsynth::params::{FilterType, Waveform};

/// Benchmark oscillator waveform generation
///
/// Performance Targets @ 44.1kHz:
/// - Sine: < 30ns/sample
/// - Saw/Square/Triangle: < 50ns/sample (4× oversampled)
fn bench_oscillator_waveforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("oscillator_waveforms");

    for waveform in [
        Waveform::Sine,
        Waveform::Saw,
        Waveform::Square,
        Waveform::Triangle,
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", waveform)),
            &waveform,
            |b, &waveform| {
                let mut osc = Oscillator::new(44100.0);
                osc.set_waveform(waveform);
                osc.set_frequency(440.0);
                b.iter(|| black_box(osc.process()));
            },
        );
    }

    group.finish();
}

/// Benchmark filter types (lowpass, highpass, bandpass)
///
/// Performance Targets @ 44.1kHz:
/// - Biquad filter processing: < 20ns/sample
fn bench_filter_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_types");

    for filter_type in [
        FilterType::Lowpass,
        FilterType::Highpass,
        FilterType::Bandpass,
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", filter_type)),
            &filter_type,
            |b, &filter_type| {
                let mut filter = BiquadFilter::new(44100.0);
                filter.set_filter_type(filter_type);
                filter.set_cutoff(1000.0);
                filter.set_resonance(2.0);
                b.iter(|| black_box(filter.process(black_box(0.5))));
            },
        );
    }

    group.finish();
}

/// Benchmark ADSR envelope generation
///
/// Performance Targets @ 44.1kHz:
/// - Envelope processing: < 10ns/sample
fn bench_envelope_processing(c: &mut Criterion) {
    let mut env = Envelope::new(44100.0);
    env.set_attack(0.01);
    env.set_decay(0.1);
    env.set_sustain(0.7);
    env.set_release(0.2);
    env.note_on();

    c.bench_function("envelope_process", |b| {
        b.iter(|| black_box(env.process()));
    });
}

criterion_group!(
    dsp_primitives_benches,
    bench_oscillator_waveforms,
    bench_filter_types,
    bench_envelope_processing,
);

criterion_main!(dsp_primitives_benches);
