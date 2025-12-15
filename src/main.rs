/*! DSynth Standalone Application Entry Point

 This module is the main entry point for DSynth when compiled as a standalone synthesizer
 application (as opposed to a VST3/CLAP plugin). It orchestrates the initialization and
 interconnection of four main components:

 1. **Parameter Buffer**: A lock-free triple-buffer for real-time safe parameter updates
    between the GUI thread and the audio engine thread
 2. **Audio Engine**: The core DSP synthesizer that generates sound samples
 3. **Audio Output**: Handles real-time audio I/O with CoreAudio (on macOS)
 4. **MIDI Input**: Receives MIDI note events from hardware/software controllers
 5. **GUI**: Interactive controls for the synthesizer parameters

 The application uses thread-safe message passing and lock-free data structures to ensure
 audio thread safety without blocking the real-time audio callback. This prevents audio
 dropouts caused by lock contention.
*/

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

    // Fixed sample rate for the audio engine. This determines the frequency at which audio
    // samples are generated and must match the audio device's sample rate. 44100 Hz is a
    // common professional audio rate balancing quality and CPU efficiency.
    const SAMPLE_RATE: f32 = 44100.0;

    // Create a lock-free triple-buffer for parameter updates. This is a concurrent data
    // structure that allows the GUI thread to update synthesizer parameters (like filter
    // cutoff, envelope times, LFO rates, etc.) without blocking the audio thread. The
    // "triple-buffer" pattern uses three buffers in rotation: one being read by audio,
    // one being written to by GUI, and one waiting. This eliminates the need for locks
    // while ensuring the audio thread always has consistent parameter values.
    let (param_producer, param_consumer) = create_parameter_buffer();

    // Create a bounded message channel for audio engine events. This channel is used to
    // send events to the audio engine from both the MIDI handler (when notes are played)
    // and the GUI (for control changes, note on/off from keyboard input, etc.). The bounded
    // capacity of 1024 prevents unbounded memory growth while being large enough to handle
    // bursts of MIDI input without dropping events. The audio thread receives from
    // event_rx, while MIDI and GUI send through event_tx.
    let (event_tx, event_rx) = bounded::<EngineEvent>(1024);

    // Initialize the synthesizer engine with the configured sample rate and parameter consumer.
    // The SynthEngine is responsible for:
    // - Managing 16 polyphonic voices (multiple simultaneous notes)
    // - Consuming events (MIDI notes, parameter changes) from the event channel
    // - Reading current parameter values from the parameter buffer
    // - Running the core DSP algorithms (oscillators, filters, envelopes, LFOs)
    // - Generating output audio samples at the specified sample rate
    // The engine itself doesn't produce audio to speakers; it just generates samples.
    // AudioOutput (below) handles the actual I/O with the operating system.
    let engine = SynthEngine::new(SAMPLE_RATE, param_consumer);

    // Initialize and start the audio output handler. This component:
    // - Creates a real-time audio callback registered with CoreAudio
    // - Continuously asks the engine for new audio samples
    // - Sends those samples to the audio device for playback
    // - Handles the audio thread's event loop, consuming events from event_rx
    //
    // This runs on a high-priority audio thread to minimize latency. If audio fails to
    // start (e.g., no audio device available), we continue anyway since audio is not
    // strictly required to use the GUI.
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

    // Initialize and start the MIDI input handler. This component:
    // - Connects to available MIDI input devices (hardware or software)
    // - Listens for MIDI note on/off, control change, and other messages
    // - Converts MIDI messages into EngineEvents and sends them via event_tx
    // - Runs on its own thread to avoid blocking other components
    //
    // The MIDI handler uses the same event_tx channel as the GUI, so both MIDI and
    // keyboard/GUI events are processed by the audio engine in a unified way. MIDI is
    // optional - it's OK if it fails to initialize (e.g., on systems without MIDI support
    // or if no devices are connected). The GUI keyboard controls will still work.
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

    // Start and run the GUI on the main thread. This is the user-facing interface that:
    // - Displays interactive controls for all synthesizer parameters
    // - Allows clicking/dragging to adjust values (cutoff frequency, resonance, etc.)
    // - Enables keyboard input for playing notes (using run_gui's built-in key handlers)
    // - Provides visual feedback about the synth's state
    // - Sends parameter updates through param_producer to the audio engine
    // - Sends note events through event_tx to the audio engine
    //
    // The GUI is blocking - it runs on the main thread and processes events until the
    // user closes the window, at which point the Result is returned and the program exits.
    // This is the reason other components (audio, MIDI) run on background threads.
    run_gui(param_producer, event_tx)?;

    Ok(())
}

// This alternative main function is compiled when the "standalone" feature is NOT enabled.
// It exists to provide a clear error message rather than a cryptic compiler error. DSynth
// can be compiled in two modes:
//
// 1. Standalone binary (this file): With "standalone" feature, produces a complete
//    synthesizer application
// 2. Plugin (plugin.rs): Without "standalone" feature, produces a VST3/CLAP plugin
//    that runs inside a DAW
//
// This fallback ensures users who try to run "cargo run" without the proper feature flag
// get helpful guidance instead of a confusing build error. The build will succeed but will
// immediately print instructions and exit.
// NOTE: This is redundant because Cargo.toml already specifies "standalone" as the default feature.
#[cfg(not(feature = "standalone"))]
fn main() {
    eprintln!("This binary requires the 'standalone' feature.");
    eprintln!("Build with: cargo build --features standalone");
    std::process::exit(1);
}
