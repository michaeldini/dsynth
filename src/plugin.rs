// Shared parameter + state system used by:
// - dsynth-clap plugin adapters (main/kick/voice)
// - the unified VIZIA GUI (standalone + plugin)
pub mod gui_param_change;
pub mod param_descriptor;
pub mod param_registry;
pub mod param_update;
pub mod state;

// Kick drum synthesizer parameter registry
#[cfg(feature = "kick-clap")]
pub mod kick_param_registry;

// Voice enhancer parameter registry
#[cfg(feature = "voice-clap")]
pub mod voice_param_registry;
