use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use dsynth::audio::{
    engine::{SynthEngine, create_parameter_buffer},
    voice::Voice,
};
use dsynth::dsp::{envelope::Envelope, filter::BiquadFilter, oscillator::Oscillator};
use dsynth::params::{FilterType, Waveform};

fn benchmark_oscillator(c: &mut Criterion) {
    let mut group = c.benchmark_group("oscillator");

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

fn benchmark_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter");

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

fn benchmark_envelope(c: &mut Criterion) {
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

fn benchmark_voice(c: &mut Criterion) {
    use dsynth::params::{
        EnvelopeParams, FilterParams, LFOParams, OscillatorParams, VelocityParams,
    };

    let mut voice = Voice::new(44100.0);
    let osc_params = [OscillatorParams::default(); 3];
    let filter_params = [FilterParams::default(); 3];
    let lfo_params = [LFOParams::default(); 3];
    let envelope_params = EnvelopeParams::default();
    let velocity_params = VelocityParams::default();
    let wavetable_library = dsynth::dsp::wavetable_library::WavetableLibrary::new();

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
            black_box(voice.process(&osc_params, &filter_params, &lfo_params, &velocity_params))
        });
    });
}

fn benchmark_engine(c: &mut Criterion) {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Trigger 8 voices
    for i in 0..8 {
        engine.note_on(60 + i, 0.8);
    }

    c.bench_function("engine_8_voices", |b| {
        b.iter(|| black_box(engine.process()));
    });
}

fn benchmark_engine_full_polyphony(c: &mut Criterion) {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Trigger 16 voices (max polyphony)
    for i in 0..16 {
        engine.note_on(60 + i, 0.8);
    }

    c.bench_function("engine_16_voices", |b| {
        b.iter(|| black_box(engine.process()));
    });
}

criterion_group!(
    benches,
    benchmark_oscillator,
    benchmark_filter,
    benchmark_envelope,
    benchmark_voice,
    benchmark_engine,
    benchmark_engine_full_polyphony
);
criterion_main!(benches);
