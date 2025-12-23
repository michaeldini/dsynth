pub mod additive_gui;
pub mod app;
pub mod keyboard;
pub mod messages;
pub mod preset_manager;
pub mod sections;

// Re-export main types for convenience
pub use app::{SynthGui, run_gui};
pub use messages::Message;
