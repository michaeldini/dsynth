/*! DSynth Standalone Application Entry Point

 This module is the main entry point for DSynth when compiled as a standalone synthesizer
 application (as opposed to a CLAP plugin). It orchestrates the initialization and
 interconnection of four main components:

 1. **Parameter Buffer**: A lock-free triple-buffer for real-time safe parameter updates
    between the GUI thread and the audio engine thread
 2. **Audio Engine**: The core DSP synthesizer that generates sound samples
 3. **Audio Output**: Handles real-time audio I/O with CoreAudio (on macOS)
 4. **MIDI Input**: Receives MIDI note events from hardware/software controllers
 5. **GUI**: Interactive controls for the synthesizer parameters (VIZIA framework)

 The application uses thread-safe message passing and lock-free data structures to ensure
 audio thread safety without blocking the real-time audio callback. This prevents audio
 dropouts caused by lock contention.
*/

#[cfg(feature = "standalone")]
use crossbeam_channel::bounded;
#[cfg(feature = "standalone")]
use dsynth::audio::output::EngineEvent;
#[cfg(feature = "standalone")]
use dsynth::audio::{engine::SynthEngine, output::AudioOutput};
#[cfg(feature = "standalone")]
use dsynth::gui::run_standalone_gui;
#[cfg(feature = "standalone")]
use dsynth::midi::handler::MidiHandler;
#[cfg(feature = "standalone")]
use dsynth::params::SynthParams;
#[cfg(feature = "standalone")]
use dsynth::plugin::gui_param_change::GuiParamChange;
#[cfg(feature = "standalone")]
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

#[cfg(feature = "standalone")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DSynth - Digital Synthesizer");
    println!("============================\n");

    // Fixed sample rate for the audio engine. This determines the frequency at which audio
    // samples are generated and must match the audio device's sample rate. 44100 Hz is a
    // common professional audio rate balancing quality and CPU efficiency.
    const SAMPLE_RATE: f32 = 44100.0;

    // Create shared synthesizer parameters that both GUI and audio thread can access.
    // The GUI uses Arc<RwLock<>> for direct parameter reads, while the audio thread
    // receives parameter changes via a lock-free triple-buffer (GuiParamChange).
    let synth_params = Arc::new(RwLock::new(SynthParams::default()));

    // Create a lock-free triple-buffer for GUI parameter changes. VIZIA sends individual
    // parameter changes (param_id + normalized value) for efficiency. The audio thread
    // reads these changes and applies them to its local SynthParams copy.
    let (gui_param_producer, _gui_param_consumer) =
        triple_buffer::TripleBuffer::new(&GuiParamChange::default()).split();

    // Create the main parameter triple-buffer for sending full SynthParams to the engine.
    // This is what the audio engine actually reads from.
    let (params_producer, params_consumer) = dsynth::audio::create_parameter_buffer();

    // Create a bounded message channel for audio engine events. This channel is used to
    // send events to the audio engine from both the MIDI handler (when notes are played)
    // and the GUI (for control changes, note on/off from keyboard input, etc.). The bounded
    // capacity of 1024 prevents unbounded memory growth while being large enough to handle
    // bursts of MIDI input without dropping events. The audio thread receives from
    // event_rx, while MIDI and GUI send through event_tx.
    let (event_tx, event_rx) = bounded::<EngineEvent>(1024);

    // Initialize the synthesizer engine with the configured sample rate.
    // The engine receives parameter updates via the params_consumer triple-buffer.
    let engine = SynthEngine::new(SAMPLE_RATE, params_consumer);

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

    // Start and run the VIZIA GUI on the main thread. VIZIA provides a unified interface
    // shared between standalone and plugin, with keyboard-to-MIDI conversion for desktop use.
    // The GUI writes parameter changes to params_producer and sends MIDI events via event_tx.
    run_standalone_gui(
        synth_params,
        Arc::new(Mutex::new(gui_param_producer)),
        Arc::new(Mutex::new(params_producer)),
        event_tx,
    )?;

    Ok(())
}

// This alternative main function is compiled when the "standalone" feature is NOT enabled.
// It exists to provide a clear error message rather than a cryptic compiler error. DSynth
// can be compiled in two modes:
//
// 1. Standalone binary (this file): With "standalone" feature, produces a complete
//    synthesizer application with VIZIA GUI (winit backend)
// 2. Plugin (lib.rs): Without "standalone" feature, produces a CLAP plugin
//    that runs inside a DAW with VIZIA GUI (baseview backend)
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
