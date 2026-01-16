use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dsynth::audio::{
    engine::{create_parameter_buffer, SynthEngine},
    voice::Voice,
};
use dsynth::dsp::{envelope::Envelope, filter::BiquadFilter, oscillator::Oscillator};
use dsynth::params::{DistortionType, FilterType, SynthParams, Waveform};

#[cfg(feature = "kick-clap")]
use dsynth::audio::kick_engine::KickEngine;
#[cfg(feature = "kick-clap")]
use dsynth::params_kick::KickParams;
#[cfg(feature = "kick-clap")]
use parking_lot::Mutex;
#[cfg(feature = "kick-clap")]
use std::sync::Arc;

#[path = "../tests/util/meter.rs"]
mod meter;

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
        EnvelopeParams, FilterParams, LFOParams, OscillatorParams, TransientShaperParams,
        VelocityParams, VoiceCompressorParams,
    };

    let mut voice = Voice::new(44100.0);
    let osc_params = [OscillatorParams::default(); 3];
    let filter_params = [FilterParams::default(); 3];
    let lfo_params = [LFOParams::default(); 3];
    let envelope_params = EnvelopeParams::default();
    let velocity_params = VelocityParams::default();
    let voice_comp_params = VoiceCompressorParams::default();
    let transient_params = TransientShaperParams::default();
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

fn benchmark_engine(c: &mut Criterion) {
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

fn benchmark_engine_full_polyphony(c: &mut Criterion) {
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

const SAMPLE_RATE: f32 = 44100.0;
const SCENE_SAMPLES: usize = 44_100;
const ATTACK_SKIP: usize = 512;

#[derive(Clone, Copy)]
enum MainScene {
    Clean,
    Abuse,
}

impl MainScene {
    fn label(&self) -> &'static str {
        match self {
            MainScene::Clean => "clean",
            MainScene::Abuse => "abuse",
        }
    }
}

fn build_main_params(scene: MainScene) -> SynthParams {
    let mut params = SynthParams::default();

    match scene {
        MainScene::Clean => {
            // Use default master_gain (now 1.0) to test headroom optimization
            params.filters[0].cutoff = 12_000.0;
            params.filters[0].resonance = 0.9;

            params.oscillators[0].waveform = Waveform::Saw;
            params.oscillators[0].gain = 0.5;
            params.oscillators[0].unison = 1;

            params.oscillators[1].waveform = Waveform::Triangle;
            params.oscillators[1].gain = 0.25;
            params.oscillators[1].unison = 1;

            params.oscillators[2].waveform = Waveform::Sine;
            params.oscillators[2].gain = 0.15;
            params.oscillators[2].unison = 1;
        }
        MainScene::Abuse => {
            params.master_gain = 1.0;
            params.filters[0].cutoff = 18_000.0;
            params.filters[0].resonance = 6.0;
            params.filters[0].drive = 0.6;
            params.filters[0].post_drive = 0.4;

            // Osc 1: Saw with high unison
            params.oscillators[0].waveform = Waveform::Saw;
            params.oscillators[0].gain = 0.8;
            params.oscillators[0].unison = 5;
            params.oscillators[0].unison_detune = 30.0;
            params.oscillators[0].unison_normalize = false;
            params.oscillators[0].saturation = 0.5;

            // Osc 2: Square with unison
            params.oscillators[1].waveform = Waveform::Square;
            params.oscillators[1].gain = 0.5;
            params.oscillators[1].unison = 3;
            params.oscillators[1].unison_detune = 25.0;
            params.oscillators[1].detune = 12.0;
            params.oscillators[1].unison_normalize = false;

            // Osc 3: Off to reduce CPU
            params.oscillators[2].gain = 0.0;

            params.effects.distortion.enabled = true;
            params.effects.distortion.drive = 0.8;
            params.effects.distortion.mix = 0.7;
            params.effects.distortion.dist_type = DistortionType::HardClip;

            params.effects.multiband_distortion.enabled = true;
            params.effects.multiband_distortion.drive_low = 0.7;
            params.effects.multiband_distortion.drive_mid = 0.7;
            params.effects.multiband_distortion.drive_high = 0.7;
            params.effects.multiband_distortion.gain_low = 1.0;
            params.effects.multiband_distortion.gain_mid = 1.0;
            params.effects.multiband_distortion.gain_high = 1.0;
            params.effects.multiband_distortion.mix = 0.6;
        }
    }

    params
}

fn render_main_scene(scene: MainScene) -> meter::LoudnessMetrics {
    let (mut producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(SAMPLE_RATE, consumer);
    let params = build_main_params(scene);
    producer.write(params);

    let notes: &[u8] = match scene {
        MainScene::Clean => &[60, 64, 67, 72],
        MainScene::Abuse => &[36, 43, 48, 52, 55, 60, 64, 67], // 8 voices instead of 16
    };

    for &note in notes {
        engine.note_on(note, 1.0);
    }

    for _ in 0..ATTACK_SKIP {
        engine.process_mono();
    }

    let mut left = vec![0.0f32; SCENE_SAMPLES];
    let mut right = vec![0.0f32; SCENE_SAMPLES];
    engine.process_block(&mut left, &mut right);

    meter::analyze_stereo(&left, &right)
}

fn benchmark_loudness_main(c: &mut Criterion) {
    let mut group = c.benchmark_group("loudness_main");

    for scene in [MainScene::Clean, MainScene::Abuse] {
        let preview = render_main_scene(scene);
        println!(
            "loudness_main/{}: peak={:.4} rms={:.4} crest={:.4}",
            scene.label(),
            preview.peak,
            preview.rms,
            preview.crest
        );

        group.bench_with_input(
            BenchmarkId::new("scene", scene.label()),
            &scene,
            |b, &scene| {
                b.iter(|| black_box(render_main_scene(scene)));
            },
        );
    }

    group.finish();
}

#[cfg(feature = "kick-clap")]
#[derive(Clone, Copy)]
enum KickScene {
    Clean,
    Abuse,
}

#[cfg(feature = "kick-clap")]
impl KickScene {
    fn label(&self) -> &'static str {
        match self {
            KickScene::Clean => "clean",
            KickScene::Abuse => "abuse",
        }
    }
}

#[cfg(feature = "kick-clap")]
fn build_kick_params(scene: KickScene) -> KickParams {
    let mut params = KickParams::default();

    match scene {
        KickScene::Clean => {
            params.master_level = 0.8;
            params.distortion_enabled = false;
            params.clipper_enabled = true;
            params.clipper_threshold = 0.95;
        }
        KickScene::Abuse => {
            params.osc1_level = 1.0;
            params.osc2_level = 0.8;
            params.amp_attack = 0.1;
            params.amp_decay = 400.0;
            params.distortion_enabled = true;
            params.distortion_amount = 1.0;
            params.distortion_type = dsynth::params_kick::DistortionType::Hard;
            params.mb_enabled = true;
            params.mb_sub_threshold = -24.0;
            params.mb_body_threshold = -18.0;
            params.mb_click_threshold = -14.0;
            params.mb_mix = 1.0;
            params.clipper_enabled = true;
            params.clipper_threshold = 0.85;
            params.master_level = 1.0;
        }
    }

    params
}

#[cfg(feature = "kick-clap")]
fn render_kick_scene(scene: KickScene) -> meter::LoudnessMetrics {
    let params = Arc::new(Mutex::new(build_kick_params(scene)));
    let mut engine = KickEngine::new(SAMPLE_RATE, Arc::clone(&params));

    engine.trigger(1.0);
    for _ in 0..ATTACK_SKIP {
        engine.process_sample();
    }

    let mut buffer = vec![0.0f32; SCENE_SAMPLES];
    engine.process_block(&mut buffer);

    meter::analyze_mono(&buffer)
}

#[cfg(feature = "kick-clap")]
fn benchmark_loudness_kick(c: &mut Criterion) {
    let mut group = c.benchmark_group("loudness_kick");

    for scene in [KickScene::Clean, KickScene::Abuse] {
        let preview = render_kick_scene(scene);
        println!(
            "loudness_kick/{}: peak={:.4} rms={:.4} crest={:.4}",
            scene.label(),
            preview.peak,
            preview.rms,
            preview.crest
        );

        group.bench_with_input(
            BenchmarkId::new("scene", scene.label()),
            &scene,
            |b, &scene| {
                b.iter(|| black_box(render_kick_scene(scene)));
            },
        );
    }

    group.finish();
}

#[cfg(feature = "kick-clap")]
criterion_group!(
    benches,
    benchmark_oscillator,
    benchmark_filter,
    benchmark_envelope,
    benchmark_voice,
    benchmark_engine,
    benchmark_engine_full_polyphony,
    benchmark_loudness_main,
    benchmark_loudness_kick
);

#[cfg(not(feature = "kick-clap"))]
criterion_group!(
    benches,
    benchmark_oscillator,
    benchmark_filter,
    benchmark_envelope,
    benchmark_voice,
    benchmark_engine,
    benchmark_engine_full_polyphony,
    benchmark_loudness_main
);

criterion_main!(benches);
