#[cfg(feature = "standalone")]
use dsynth::audio::{create_parameter_buffer, engine::SynthEngine, output::AudioOutput};
#[cfg(feature = "standalone")]
use dsynth::gui::run_gui;
#[cfg(feature = "standalone")]
use dsynth::midi::handler::{MidiEvent, MidiHandler};
#[cfg(feature = "standalone")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "standalone")]
use std::thread;

#[cfg(feature = "standalone")]
fn velocity_to_float(v: u8) -> f32 {
    v as f32 / 127.0
}

#[cfg(feature = "standalone")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DSynth - Digital Synthesizer");
    println!("============================\n");

    const SAMPLE_RATE: f32 = 44100.0;

    // Create triple-buffer for parameters
    let (param_producer, param_consumer) = create_parameter_buffer();

    // Create synth engine
    let engine = SynthEngine::new(SAMPLE_RATE, param_consumer);
    let engine_arc = Arc::new(Mutex::new(engine));

    // Start audio output
    println!("Starting audio output...");
    let _audio_output = match AudioOutput::new(engine_arc.clone()) {
        Ok(output) => {
            println!("✓ Audio output started at {} Hz", output.sample_rate());
            Some(output)
        }
        Err(e) => {
            eprintln!("✗ Failed to start audio: {}", e);
            None
        }
    };

    // Start MIDI input
    println!("\nStarting MIDI input...");
    let _midi_handler = match MidiHandler::new() {
        Ok((handler, receiver)) => {
            println!("✓ MIDI handler started");

            // Spawn MIDI event processing thread
            let engine_clone = engine_arc.clone();
            thread::spawn(move || {
                println!("MIDI event processor running...");
                while let Ok(event) = receiver.recv() {
                    let mut engine = engine_clone.lock().unwrap();
                    match event {
                        MidiEvent::NoteOn { note, velocity } => {
                            engine.note_on(note, velocity_to_float(velocity));
                        }
                        MidiEvent::NoteOff { note } => {
                            engine.note_off(note);
                        }
                        MidiEvent::ControlChange { .. } => {
                            // CC handling can be added here
                        }
                    }
                }
            });

            Some(handler)
        }
        Err(e) => {
            eprintln!("✗ Failed to start MIDI: {}", e);
            eprintln!("   (This is OK - MIDI input is optional)");
            None
        }
    };

    println!("\n--- Starting GUI ---\n");
    println!("Keyboard mapping:");
    println!("  AWSEDFTGYHUJKOLP - Piano keys (C4-D#5)");
    println!("  ZXCVBNM - Lower octave (C3-B3)\n");

    // Run GUI on main thread
    run_gui(param_producer, engine_arc)?;

    Ok(())
}

#[cfg(not(feature = "standalone"))]
fn main() {
    eprintln!("This binary requires the 'standalone' feature.");
    eprintln!("Build with: cargo build --features standalone");
    std::process::exit(1);
}
