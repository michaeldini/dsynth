use dsynth::audio::{engine::{SynthEngine, create_parameter_buffer}, output::AudioOutput};
use dsynth::midi::{handler::{MidiEvent, velocity_to_float}};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_full_audio_pipeline() {
    // Create parameter buffer
    let (_param_producer, param_consumer) = create_parameter_buffer();
    
    // Create synth engine
    let engine = SynthEngine::new(44100.0, param_consumer);
    let engine_arc = Arc::new(Mutex::new(engine));
    
    // Try to create audio output (may fail in CI environments)
    let audio_result = AudioOutput::new(engine_arc.clone());
    
    match audio_result {
        Ok(_audio) => {
            println!("Audio output started successfully");
            
            // Trigger a note
            {
                let mut engine = engine_arc.lock().unwrap();
                engine.note_on(60, 0.8);
            }
            
            // Let it play for a bit
            thread::sleep(Duration::from_millis(100));
            
            // Check that voices are active
            {
                let engine = engine_arc.lock().unwrap();
                assert!(engine.active_voice_count() > 0);
            }
            
            // Release note
            {
                let mut engine = engine_arc.lock().unwrap();
                engine.note_off(60);
            }
            
            println!("Full audio pipeline test passed");
        }
        Err(e) => {
            println!("Audio output not available (expected in CI): {}", e);
        }
    }
}

#[test]
fn test_midi_to_engine_integration() {
    // Create parameter buffer
    let (_param_producer, param_consumer) = create_parameter_buffer();
    
    // Create synth engine
    let engine = SynthEngine::new(44100.0, param_consumer);
    let mut engine = engine;
    
    // Simulate MIDI events
    let note_on = MidiEvent::NoteOn { note: 60, velocity: 100 };
    let note_off = MidiEvent::NoteOff { note: 60 };
    
    // Process note on
    match note_on {
        MidiEvent::NoteOn { note, velocity } => {
            engine.note_on(note, velocity_to_float(velocity));
        }
        _ => {}
    }
    
    assert_eq!(engine.active_voice_count(), 1);
    
    // Process some audio
    for _ in 0..100 {
        engine.process();
    }
    
    // Process note off
    match note_off {
        MidiEvent::NoteOff { note } => {
            engine.note_off(note);
        }
        _ => {}
    }
    
    // Voice should still be active (in release phase)
    assert_eq!(engine.active_voice_count(), 1);
    
    println!("MIDI to engine integration test passed");
}

#[test]
fn test_polyphonic_performance() {
    // Create parameter buffer
    let (_param_producer, param_consumer) = create_parameter_buffer();
    
    // Create synth engine
    let mut engine = SynthEngine::new(44100.0, param_consumer);
    
    // Trigger maximum polyphony
    for i in 0..16 {
        engine.note_on(60 + i, 0.8);
    }
    
    assert_eq!(engine.active_voice_count(), 16);
    
    // Process a buffer of samples
    let buffer_size = 512;
    let start = std::time::Instant::now();
    
    for _ in 0..buffer_size {
        let _sample = engine.process();
    }
    
    let elapsed = start.elapsed();
    
    println!("Processed {} samples with 16 voices in {:?}", buffer_size, elapsed);
    println!("Time per sample: {:?}", elapsed / buffer_size);
    
    // At 44.1kHz, we need to process samples faster than ~22.7µs each
    // With 512 samples, should complete in well under 12ms
    assert!(elapsed.as_millis() < 50, "Performance regression: took {:?}", elapsed);
}
