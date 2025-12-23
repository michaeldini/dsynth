use dsynth::audio::engine::SynthEngine;
use dsynth::params::SynthParams;

fn main() {
    let (producer, consumer) = triple_buffer::triple_buffer(&SynthParams::default());
    let mut engine = SynthEngine::new(44100.0, consumer);
    
    // Trigger a note
    engine.note_on(60, 0.8);
    
    // Process a few samples and check if we get non-zero output
    let mut has_sound = false;
    for i in 0..1000 {
        let (left, right) = engine.process_stereo();
        if left.abs() > 0.0001 || right.abs() > 0.0001 {
            println!("Sample {}: L={:.6}, R={:.6}", i, left, right);
            has_sound = true;
            if i > 10 { break; }
        }
    }
    
    if has_sound {
        println!("\n✓ Audio generation working!");
    } else {
        println!("\n✗ No audio generated - silence");
    }
}
