// Test to check stereo balance and potential phase issues
use dsynth::audio::engine::SynthEngine;
use dsynth::params::SynthParams;
use triple_buffer::TripleBuffer;

fn main() {
    let sample_rate = 44100.0;
    let default_params = SynthParams::default();
    let (mut params_producer, params_consumer) = TripleBuffer::new(&default_params).split();
    let mut engine = SynthEngine::new(sample_rate, params_consumer);
    
    // Initialize parameters
    params_producer.write(default_params);
    
    println!("Testing with 4 simultaneous notes...\n");
    
    // Trigger 4 notes with the same velocity
    engine.note_on(60, 0.8); // C
    engine.note_on(64, 0.8); // E
    engine.note_on(67, 0.8); // G
    engine.note_on(72, 0.8); // High C
    
    // Process samples and analyze
    let mut left_sum = 0.0f64;
    let mut right_sum = 0.0f64;
    let mut max_left = 0.0f32;
    let mut max_right = 0.0f32;
    let mut diff_sum = 0.0f64;
    let total_samples = 44100; // 1 second
    
    for _ in 0..total_samples {
        let (left, right) = engine.process_stereo();
        
        left_sum += left as f64;
        right_sum += right as f64;
        max_left = max_left.max(left.abs());
        max_right = max_right.max(right.abs());
        diff_sum += (left - right).abs() as f64;
    }
    
    let avg_left = left_sum / total_samples as f64;
    let avg_right = right_sum / total_samples as f64;
    let avg_diff = diff_sum / total_samples as f64;
    
    println!("=== STEREO ANALYSIS ===");
    println!("Left channel:");
    println!("  Max: {:.4}", max_left);
    println!("  Avg: {:.6}", avg_left);
    println!("Right channel:");
    println!("  Max: {:.4}", max_right);
    println!("  Avg: {:.6}", avg_right);
    println!("\nStereo difference: {:.6}", avg_diff);
    println!("Balance ratio (L/R): {:.3}", max_left / max_right.max(0.001));
    println!("\nActive voices: {}", engine.active_voice_count());
    
    // Now test with 3 notes
    println!("\n\n=== Testing with 3 notes (should sound good) ===");
    engine.all_notes_off();
    
    engine.note_on(60, 0.8); //  C
    engine.note_on(64, 0.8); // E
    engine.note_on(67, 0.8); // G
    
    let mut max3_left = 0.0f32;
    let mut max3_right = 0.0f32;
    
    for _ in 0..total_samples {
        let (left, right) = engine.process_stereo();
        max3_left = max3_left.max(left.abs());
        max3_right = max3_right.max(right.abs());
    }
    
    println!("3 notes - Left max: {:.4}, Right max: {:.4}", max3_left, max3_right);
    println!("4 notes - Left max: {:.4}, Right max: {:.4}", max_left, max_right);
    println!("\nRatio (4 notes / 3 notes): {:.3}", max_left / max3_left.max(0.001));
}
