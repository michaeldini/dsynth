// VIZIA GUI - unified for both CLAP plugin and standalone
// vizia is now a non-optional dependency, always available
pub mod vizia_gui;

// #[cfg(feature = "clap")]
// #[path = "plugin_gui.rs"]
// pub mod plugin_gui;

// Re-export standalone entry point
#[cfg(feature = "standalone")]
pub use vizia_gui::run_standalone_gui;
