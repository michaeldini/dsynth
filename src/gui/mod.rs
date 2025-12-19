pub mod controls;

#[cfg(feature = "vst")]
#[path = "plugin_gui.rs"]
pub mod plugin_gui;

// Standalone GUI is now modularized in standalone_gui/
#[cfg(feature = "standalone")]
pub mod standalone_gui;

// Re-export main types for backward compatibility
#[cfg(feature = "standalone")]
pub use standalone_gui::{run_gui, Message, SynthGui};

// Re-export message sub-types for convenience
#[cfg(feature = "standalone")]
pub use standalone_gui::messages::{
    ChorusMessage, DelayMessage, DistortionMessage, EnvelopeMessage, FilterMessage, LFOMessage,
    OscillatorMessage, ReverbMessage,
};
