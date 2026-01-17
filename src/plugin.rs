// CLAP plugin implementation module structure

// Phase 1: Parameter System for CLAP Migration
pub mod gui_param_change;
pub mod param_descriptor;
pub mod param_registry;
pub mod param_update;
pub mod state;

// Kick drum synthesizer parameter registry
#[cfg(any(feature = "kick-clap"))]
pub mod kick_param_registry;

// Voice enhancer parameter registry
#[cfg(any(feature = "voice-clap"))]
pub mod voice_param_registry;

// Phase 2: CLAP Plugin Implementation
#[cfg(any(feature = "clap", feature = "kick-clap", feature = "voice-clap"))]
pub mod clap;
