//! # Audio Module - Real-Time Synthesis Engine
//!
//! This module contains the core real-time audio synthesis system. It handles:
//! - **Audio generation**: The synthesizer engine that creates sound samples
//! - **Voice management**: Individual polyphonic voices that handle note playback
//! - **Event processing**: Consuming MIDI events and parameter changes
//! - **Audio I/O**: (standalone only) Platform-specific audio output to speakers
//!
//! ## Architecture Overview
//!
//! The audio module follows a producer-consumer pattern for thread-safe real-time audio:
//!
//!
//! GUI/MIDI Thread          Audio Thread (Real-Time)
//! ───────────────          ────────────────────────
//! Parameter Updates ──→ [Triple Buffer] ──→ Engine reads parameters
//!                                              │
//! MIDI Events/GUI ───→ [Event Channel] ──→ Engine processes events
//! (Note On/Off)            (bounded,1024)     │
//!                                              |
//!                                           Synth Engine
//!                                              │
//!                                           Voices (1-16)
//!                                              │
//!                                            DSP Modules
//!                                            (Oscillators,
//!                                             Filters,
//!                                             Envelopes)
//!                                              │
//!                                              │
//!                                           Audio Output (frames)
//!                                              │
//!                                              │
//!                                        CoreAudio Callback
//!                                              │
//!                                              │
//!                                           Speakers
//!
//!
//! ## Why This Design?
//!
//! Audio processing has two conflicting requirements:
//! - **Real-time safety**: Audio callbacks must complete within microseconds without blocking
//! - **Interactivity**: GUI/MIDI updates should happen instantly without waiting
//!
//! Traditional locking (mutexes) won't work because:
//! 1. Lock acquisition is unpredictable (varies based on system load)
//! 2. Even brief waits cause audio glitches (pops/clicks)
//! 3. Priority inversion: GUI thread blocks real-time audio thread
//!
//! This module solves it with lock-free data structures:
//! - **Triple Buffer**: For parameter updates (no waiting)
//! - **Bounded MPSC Channel**: For events (lock-free queue)
//!
//! This allows GUI and audio to run independently without interfering with each other.

/// The **engine** module contains the main synthesis engine and core audio processing.
///
/// Key responsibilities:
/// - `SynthEngine`: The main synthesis engine that:
///   - Manages 16 polyphonic voices (multiple simultaneous notes)
///   - Processes incoming MIDI/GUI events (note on/off, parameter changes)
///   - Reads current parameter values from the triple-buffer
///   - Generates output audio samples by calling each voice's process() method
///   - Mixes voice outputs and applies global effects
/// - `EngineEvent`: The event types that trigger synthesis changes
/// - `create_parameter_buffer()`: Creates the lock-free triple-buffer for parameters
///
/// The engine is the orchestrator - it coordinates all the synthesis modules (voices, DSP)
/// and manages the real-time audio processing loop. It must be extremely efficient since
/// it runs on the real-time audio thread where every microsecond counts.
pub mod engine;

/// The **output** module provides platform-specific audio I/O (only in standalone mode).
///
/// This module is conditionally compiled ONLY when the "standalone" feature is enabled.
/// It's responsible for:
/// - Registering an audio callback with the OS (CoreAudio on macOS, WASAPI on Windows, etc.)
/// - Running the audio callback on a high-priority audio thread
/// - Managing the audio device and sample rate
/// - Converting the engine's generated samples into speaker output
/// - Handling audio device errors gracefully (e.g., device disconnected)
///
/// The output module runs the actual real-time audio loop. It continuously asks the engine
/// for new audio samples and sends them to the audio device. This is where audio callbacks
/// happen - callbacks are extremely time-critical and cannot block.
///
/// Note: This module is NOT included in plugin builds because the DAW host provides its
/// own audio I/O and callback mechanism. Plugin code (in plugin.rs) interfaces directly
/// with the host's audio system instead.
#[cfg(feature = "standalone")]
pub mod output;

/// The **voice** module implements a single polyphonic voice.
///
/// Each voice is an independent synthesizer that can play one note at a time. A voice includes:
/// - State tracking: Is it active? What note is being played?
/// - One or more oscillators: Generate the raw waveform
/// - Filter: Shape the timbre (cutoff frequency, resonance)
/// - Envelope: Control amplitude over time (attack, decay, sustain, release)
/// - LFO modulation: Add movement to oscillator/filter parameters
/// - Voice allocation: Assign voices to incoming notes
///
/// The synthesizer typically has 16 voices. When a MIDI note on arrives:
/// 1. An idle voice is allocated
/// 2. The voice initializes its state (note number, velocity)
/// 3. The voice's envelope starts its attack phase
/// 4. Each audio frame, the voice generates one sample
///
/// When a note off arrives:
/// 1. The voice's envelope enters release phase
/// 2. After release completes, the voice becomes idle again
///
/// Voices are kept separate so they can process independently and in parallel.
pub mod voice;

/// Re-export `create_parameter_buffer` for convenient access from outside the audio module.
///
/// This function creates the lock-free triple-buffer used for parameter updates.
/// It's re-exported here so users of the audio module don't need to know that it lives
/// in the `engine` submodule - they can just use `audio::create_parameter_buffer()`.
///
/// This is a common Rust pattern: internal organization details (where the function lives)
/// are hidden from external API consumers (what functions are available).
pub use engine::create_parameter_buffer;
