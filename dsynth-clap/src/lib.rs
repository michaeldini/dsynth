//! DSynth CLAP Plugin Framework
//!
//! A reusable library for building CLAP audio plugins with minimal boilerplate.
//!
//! # Architecture
//!
//! This library provides trait-based abstractions over the CLAP plugin API:
//!
//! - **`ClapPlugin`**: Core plugin trait defining metadata and lifecycle
//! - **`ClapProcessor`**: Audio processing trait for real-time DSP
//! - **`PluginParams`**: Parameter management with automation support
//! - **`PluginState`**: Save/load functionality for presets
//!
//! # Example
//!
//! ```ignore
//! use dsynth_clap::*;
//!
//! struct MyPlugin;
//!
//! impl ClapPlugin for MyPlugin {
//!     type Processor = MyProcessor;
//!     type Params = MyParams;
//!     
//!     fn descriptor(&self) -> PluginDescriptor {
//!         PluginDescriptor::instrument("My Synth", "com.example.mysynth")
//!             .version("1.0.0")
//!             .with_features(&["synthesizer", "instrument"])
//!     }
//! }
//! ```

pub mod descriptor;
pub mod entry;
pub mod extensions;
pub mod instance;
pub mod param;
pub mod plugin;
pub mod processor;
pub mod state;

// Re-export clap-sys for macro use
pub use clap_sys;

// Re-exports for convenience
pub use descriptor::PluginDescriptor;
pub use instance::PluginInstance;
pub use param::{ParamDescriptor, ParamId, ParamType, PluginParams};
pub use plugin::ClapPlugin;
pub use processor::{AudioBuffers, ClapProcessor, Events, ProcessStatus};
pub use state::PluginState;

/// Audio port configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortConfig {
    /// No audio input, stereo output (synthesizer)
    Instrument,
    /// Stereo input, stereo output (audio effect)
    Effect,
    /// Custom port configuration
    Custom { inputs: u32, outputs: u32 },
}

/// MIDI port configuration  
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotePortConfig {
    /// No MIDI ports
    None,
    /// One MIDI input port
    Input,
    /// Custom MIDI configuration
    Custom { inputs: u32, outputs: u32 },
}
