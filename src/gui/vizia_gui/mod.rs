// VIZIA GUI module - unified GUI for both plugin and standalone targets

pub mod messages;
pub mod shared_ui;
pub mod state;
pub mod widgets;

pub use messages::GuiMessage;
pub use state::GuiState;

#[cfg(feature = "clap")]
pub mod plugin_window;

#[cfg(feature = "standalone")]
pub mod standalone_window;

#[cfg(feature = "standalone")]
pub use standalone_window::run_standalone_gui;
