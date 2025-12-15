// Test with extreme parameter settings to reproduce the issue
use dsynth::audio::engine::SynthEngine;
use dsynth::params::SynthParams;
use triple_buffer::TripleBuffer;

fn main() {
    let sample_rate = 44100.0;
    
    println!("Testing different parameter configurations...\n");
    
    // Test 1: High resonance
    println!("=== TEST 1: High Resonance ===");
    let mut params = SynthParams::default();
    for filter in &mut params.filters {
        filter.resonance = 9.0; // Very high Q
    }
    test_config("High Resonance", params);
    
    // Test 2: Narrow bandwidth (for bandpass)
    println!("\n=== TEST 2: Narrow Bandwidth ===");
    let mut params = SynthParams::default();
    for filter in &mut params.filters {
        filter.bandwidth = 0.2; // Very narrow
    }
    test_config("Narrow Bandwidth", params);
    
    // Test 3: High oscillator gains
    println!("\n=== TEST 3: High Oscillator Gains ===");
    let mut params = SynthParams::default();
    for osc in &mut params.oscillators {
        osc.gain = 0.8; // Much higher than default 0.25
    }
    test_config("High Osc Gains", params);
    
    // Test 4: Combination of all
    println!("\n=== TEST 4: Extreme Combination ===");
    let mut params = SynthParams::default();
    for filter in &mut params.filters {
        filter.resonance = 9.0;
        filter.bandwidth = 0.2;
    }
    for osc in &mut params.oscillators {
        osc.gain = 0.7;
    }
    test_config("Extreme Settings", params);
}

fn test_config(name: &str, params: SynthParams) {
    let sample_rate = 44100.0;
    let (mut params_producer, params_consumer) = TripleBuffer::new(&params).split();
    let mut engine = SynthEngine::new(sample_rate, params_consumer);
    
    params_producer.write(params);
    
    // Trigger 4 notes
    engine.note_on(60, 0.8);
    engine.note_on(64, 0.8);
    engine.note_on(67, 0.8);
    engine.note_on(72, 0.8);
    
    let mut max_val = 0.0f32;
    let mut clipped = 0;
    let mut nan_count = 0;
    let mut inf_count = 0;
    let total_samples = 10000;
    
    for _ in 0..total_samples {
        let (left, right) = engine.process_stereo();
        
        if left.is_nan() || right.is_nan() {
            nan_count += 1;
        }
        if left.is_infinite() || right.is_infinite() {
            inf_count += 1;
        }
        
        let peak = left.abs().max(right.abs());
        max_val = max_val.max(peak);
        if peak > 1.0 {
            clipped += 1;
        }
    }
    
    println!("{}: Max={:.3}, Clipped={}/{}, NaN={}, Inf={}", 
             name, max_val, clipped, total_samples, nan_count, inf_count);
    
    if nan_count > 0 || inf_count > 0 {
        println!("  ❌ NUMERICAL INSTABILITY DETECTED!");
    } else if clipped > 100 {
        println!("  ⚠️  Significant clipping");
    } else {
        println!("  ✓ OK");
    }
}
