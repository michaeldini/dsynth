use criterion::{Criterion, black_box};
use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
use dsynth::dsp::filter::BiquadFilter;
use dsynth::params::{FilterType, SynthParams};

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
        
        b.iter(|| {
            // Simulate parameter changes (like from GUI or automation)
            let mut params = SynthParams::default();
            params.oscillators[0].unison = 4;
            params.filters[0].cutoff = 2000.0;
            producer.write(params);
            
            // Process samples
            for _ in 0..32 {
                black_box(engine.process());
            }
        });
    });
    
    group.finish();
}

criterion::criterion_group!(
    benches,
    benchmark_filter_with_modulation,
    benchmark_engine_with_parameter_changes
);
criterion::criterion_main!(benches);
