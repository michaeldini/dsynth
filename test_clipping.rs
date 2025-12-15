// Quick test to check output levels with multiple voices
use dsynth::audio::engine::SynthEngine;
use dsynth::params::SynthParams;
use triple_buffer::TripleBuffer;

fn main() {
    let sample_rate = 44100.0;
    let (params_producer, params_consumer) = TripleBuffer::new(SynthParams::default()).split();
    let mut engine = SynthEngine::new(sample_rate, params_consumer);

    // Initialize parameters with producer
    let mut current_params = SynthParams::default();
    params_producer.write(current_params);

    // Trigger 4 notes
    engine.note_on(60, 0.8); // C
    engine.note_on(64, 0.8); // E
    engine.note_on(67, 0.8); // G
    engine.note_on(72, 0.8); // High C

    // Process a few thousand samples to get past attack
    let mut max_sample = 0.0f32;
    let mut clipped_samples = 0;
    let total_samples = 10000;

    for i in 0..total_samples {
        let (left, right) = engine.process_stereo();
        let peak = left.abs().max(right.abs());
        max_sample = max_sample.max(peak);

        if peak > 1.0 {
            clipped_samples += 1;
            if i < 100 {
                println!("Sample {}: L={:.3}, R={:.3}", i, left, right);
            }
        }
    }

    println!("\n=== RESULTS ===");
    println!("Max sample value: {:.3}", max_sample);
    println!(
        "Clipped samples: {}/{} ({:.1}%)",
        clipped_samples,
        total_samples,
        100.0 * clipped_samples as f32 / total_samples as f32
    );
    println!("Active voices: {}", engine.active_voice_count());

    if max_sample > 1.0 {
        println!("\n❌ CLIPPING DETECTED!");
    } else {
        println!("\n✅ No clipping");
    }
}
