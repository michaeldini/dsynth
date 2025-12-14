// Thin wrapper so `crate::gui::plugin_gui` stays stable while the implementation
// lives in the extracted module files under src/gui/plugin_gui/.

#[path = "plugin_gui/mod.rs"]
mod extracted;

#[allow(unused_imports)]
pub(crate) use extracted::{Message, PluginGui, create, default_state};
