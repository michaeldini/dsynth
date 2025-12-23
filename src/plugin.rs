// CLAP plugin implementation module structure

// Phase 1: Parameter System for CLAP Migration
pub mod gui_param_change;
pub mod param_descriptor;
pub mod param_registry;
pub mod param_update;
pub mod state;

// Phase 2: CLAP Plugin Implementation
#[cfg(feature = "clap")]
pub mod clap;
