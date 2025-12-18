//! # DSynth - Digital Synthesizer Library
//!
//! DSynth is a polyphonic synthesizer library written in Rust that can be used in two ways:
//!
//! 1. **As a library**: Import and use the synthesizer modules directly in your own Rust code
//! 2. **As a plugin**: Compiled as a VST3/CLAP plugin that runs inside a DAW (Digital Audio Workstation)
//! 3. **As a standalone app**: A complete synthesizer application with GUI and audio I/O
//!
//! ## Architecture Overview
//!
//! The library is organized into several core modules:
//! - **audio**: Real-time audio engine and sample generation
//! - **dsp**: Digital Signal Processing algorithms (oscillators, filters, envelopes, LFOs)
//! - **gui**: User interface for controlling parameters (shared by standalone and plugin)
//! - **midi**: MIDI input handling for receiving note events
//! - **params**: Parameter definitions and management
//! - **preset**: Preset loading/saving functionality
//!
//! ## Compilation Modes
//!
//! The codebase supports multiple compilation targets through Cargo features:
//! - `standalone`: Builds a complete app with audio I/O and GUI
//! - `vst`: Builds VST3/CLAP plugins for DAWs
//! - `simd`: Enables portable SIMD optimizations for DSP algorithms
//!
//! Different features enable/disable different modules and dependencies to keep builds
//! lightweight for each use case.

//! Enable portable SIMD support when the "simd" feature is enabled. This is a crate-level
//! attribute that unlocks Rust's portable SIMD API (`std::simd`) for use throughout the library.
//! SIMD (Single Instruction Multiple Data) allows processing multiple audio samples in parallel,
//! dramatically improving performance for DSP operations like filtering and oscillator generation.
//!
//! Portable SIMD is an unstable Rust feature, so it requires this explicit opt-in. The conditional
//! compilation (`cfg_attr`) means this only applies when building with `--features simd`. Without
//! this feature, the code uses standard scalar operations instead, which is slower but doesn't
//! require nightly Rust.
#![cfg_attr(feature = "simd", feature(portable_simd))]

/// The **audio** module contains the real-time synthesizer engine and audio I/O.
///
/// Key components:
/// - `SynthEngine`: The main synthesis engine that processes MIDI events and parameter changes,
///   manages 16 polyphonic voices, and generates audio samples
/// - `AudioOutput`: Handles platform-specific audio I/O (CoreAudio on macOS, WASAPI on Windows)
///   and runs the audio callback thread
/// - `EngineEvent`: The event types that the engine processes (note on/off, parameter changes, etc.)
///
/// This is the heart of the synthesizer where sound is actually created. It reads input events,
/// looks up current parameters, and generates output audio samples at the configured sample rate.
pub mod audio;

/// The **dsp** (Digital Signal Processing) module contains the low-level audio algorithms.
///
/// Key components:
/// - `Oscillator`: Generates basic waveforms (sine, square, sawtooth, triangle)
/// - `Filter`: Implements resonant low-pass, high-pass, and band-pass filters
/// - `Envelope`: ADSR envelope generator for shaping sound over time
/// - `LFO`: Low-frequency oscillators for modulating other parameters
/// - `Waveform`: Pre-computed lookup tables for efficient waveform generation
/// - `Downsampler`: Reduces sample rate for anti-aliasing
///
/// These modules are reusable building blocks used by the audio engine. They're kept separate
/// to make them independently testable and reusable in different contexts. Many of these
/// implementations are optimized with SIMD when the "simd" feature is enabled.
pub mod dsp;

/// The **gui** module provides the user interface for controlling the synthesizer.
///
/// This module is conditionally compiled when either `standalone` or `vst` features are enabled,
/// since both the standalone app and VST plugin need a GUI for users to control parameters.
///
/// Key components:
/// - `run_gui()`: The main GUI loop that handles user interactions
/// - Interactive controls for all synthesizer parameters (cutoff, resonance, envelope, etc.)
/// - Keyboard input handling for playing notes (in standalone mode)
/// - Real-time visualization and feedback
///
/// The GUI communicates with the audio engine through:
/// 1. **Parameter producer**: Sends parameter updates (filter cutoff, volume, etc.)
/// 2. **Event channel**: Sends user-triggered events (note on/off from keyboard)
///
/// By keeping the GUI module conditional, plugin builds don't include unused GUI code,
/// reducing binary size. VST/CLAP hosts provide their own GUI framework, but DSynth still
/// uses this for its parameter controls.
#[cfg(any(feature = "standalone", feature = "vst"))]
pub mod gui;

/// The **midi** module handles incoming MIDI input from hardware controllers and software.
///
/// Key components:
/// - `MidiHandler`: Manages MIDI device connections and input processing
/// - Event detection for note on/off, control changes, pitch bend, etc.
/// - Conversion of MIDI data into engine events
///
/// The MIDI handler runs on its own thread and sends events through a lock-free channel
/// to the audio engine. This allows hardware controllers and DAWs to trigger notes and
/// modulate parameters in real-time. In standalone mode, this is how external keyboards
/// and controllers are connected. In VST mode, the DAW provides MIDI routing.
pub mod midi;

/// The **params** module defines all synthesizer parameters and their metadata.
///
/// This module specifies:
/// - All available parameters (cutoff frequency, resonance, envelope times, LFO rate, etc.)
/// - Parameter types (float, integer, enum)
/// - Valid ranges and default values
/// - Parameter naming and descriptions for the GUI
///
/// This serves as a contract between the UI and the audio engine - both need to know
/// what parameters exist and their valid ranges. Keeping parameters in one place makes
/// it easier to add new controls consistently across both the GUI and the audio engine.
pub mod params;

/// The **preset** module handles loading and saving synthesizer configurations.
///
/// This module manages:
/// - Saving the current parameter values to a JSON file (a "preset")
/// - Loading a previously saved preset back into the synthesizer
/// - Preset file I/O and serialization
///
/// Presets allow users to save their favorite synthesizer configurations and recall them later.
/// This is important for musicians and sound designers who want to build and share specific sounds.
/// Presets are typically stored in JSON format for readability and ease of editing.
pub mod preset;

/// The **randomize** module provides utilities for generating randomized parameter sets.
///
/// This module contains functions for sound design exploration:
/// - `randomize_synth_params()`: Generates a random but musically useful parameter configuration
///
/// Randomization is useful for discovering new sounds and creative exploration. The randomization
/// logic ensures parameters stay within reasonable ranges to avoid silent or broken sounds.
pub mod randomize;

/// The **plugin** module contains the VST3/CLAP plugin wrapper code.
///
/// This is conditionally compiled ONLY when the `vst` feature is enabled. The plugin module
/// acts as a bridge between the synthesizer library and plugin host standards:
/// - Implements VST3/CLAP plugin interface
/// - Handles plugin lifecycle (creation, destruction, processing)
/// - Routes plugin parameters to DSynth's internal parameters
/// - Manages audio processing callbacks from the host
///
/// The `#[path = "plugin.rs"]` attribute tells Rust to look for this module in the file
/// `plugin.rs` instead of using a folder named `plugin/`. This keeps the source directory
/// flatter while maintaining the same module structure. The module is gated behind the "vst"
/// feature so it's completely absent from standalone builds, avoiding VST dependencies
/// for users who only need the library or standalone app.
#[cfg(feature = "vst")]
#[path = "plugin.rs"]
mod plugin;
