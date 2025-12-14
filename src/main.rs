#[cfg(feature = "standalone")]
use crossbeam_channel::bounded;
#[cfg(feature = "standalone")]
use dsynth::audio::output::EngineEvent;
#[cfg(feature = "standalone")]
use dsynth::audio::{create_parameter_buffer, engine::SynthEngine, output::AudioOutput};
#[cfg(feature = "standalone")]
use dsynth::gui::run_gui;
#[cfg(feature = "standalone")]
use dsynth::midi::handler::MidiHandler;

#[cfg(feature = "standalone")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DSynth - Digital Synthesizer");
    println!("============================\n");

    const SAMPLE_RATE: f32 = 44100.0;

    // Create triple-buffer for parameters
    let (param_producer, param_consumer) = create_parameter_buffer();

    // Events from MIDI/GUI -> audio thread
    let (event_tx, event_rx) = bounded::<EngineEvent>(1024);

    // Create synth engine
    let engine = SynthEngine::new(SAMPLE_RATE, param_consumer);

    // Start audio output
    println!("Starting audio output...");
    let _audio_output = match AudioOutput::new(engine, event_rx) {
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
    let _midi_handler = match MidiHandler::new_with_engine_event_sender(event_tx.clone()) {
        Ok(handler) => {
            println!("✓ MIDI handler started");
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
    run_gui(param_producer, event_tx)?;

    Ok(())
}

#[cfg(not(feature = "standalone"))]
fn main() {
    eprintln!("This binary requires the 'standalone' feature.");
    eprintln!("Build with: cargo build --features standalone");
    std::process::exit(1);
}
