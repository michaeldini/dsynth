// CLAP plugin implementation module structure

// Phase 1: Parameter System for CLAP Migration
pub mod gui_param_change;
pub mod param_descriptor;
pub mod param_registry;
pub mod param_update;
pub mod state;

// Kick drum synthesizer parameter registry
#[cfg(any(feature = "kick-clap", feature = "kick-synth"))]
pub mod kick_param_registry;

// Phase 2: CLAP Plugin Implementation
#[cfg(any(feature = "clap", feature = "kick-clap"))]
pub mod clap;
