// VIZIA GUI module

// Shared pieces used by multiple targets (including kick-clap)
#[cfg(any(
    feature = "clap",
    feature = "standalone",
    feature = "kick-clap",
    feature = "kick-synth"
))]
pub mod messages;

#[cfg(any(
    feature = "clap",
    feature = "standalone",
    feature = "kick-clap",
    feature = "kick-synth"
))]
pub mod theme;

#[cfg(any(
    feature = "clap",
    feature = "standalone",
    feature = "kick-clap",
    feature = "kick-synth"
))]
pub mod widgets;

#[cfg(any(
    feature = "clap",
    feature = "standalone",
    feature = "kick-clap",
    feature = "kick-synth"
))]
pub use messages::GuiMessage;

// Main poly synth UI/state (not needed for kick-clap)
#[cfg(any(feature = "clap", feature = "standalone"))]
pub mod shared_ui;

#[cfg(any(feature = "clap", feature = "standalone"))]
pub mod state;

#[cfg(any(feature = "clap", feature = "standalone"))]
pub use state::GuiState;

// Window backends
#[cfg(feature = "clap")]
pub mod plugin_window;

#[cfg(feature = "kick-clap")]
pub mod kick_plugin_window;

#[cfg(feature = "standalone")]
pub mod standalone_window;

#[cfg(feature = "standalone")]
pub use standalone_window::run_standalone_gui;
