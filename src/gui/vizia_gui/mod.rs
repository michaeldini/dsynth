// VIZIA GUI module - unified GUI for both plugin and standalone targets

pub mod messages;
pub mod state;
pub mod widgets;

pub use messages::GuiMessage;
pub use state::GuiState;

#[cfg(feature = "clap")]
pub mod plugin_window;

// TODO: Add standalone_window module in Phase 4
// #[cfg(feature = "standalone")]
// pub mod standalone_window;
