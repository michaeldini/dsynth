//! Core plugin trait

use crate::{ClapProcessor, PluginDescriptor, PluginParams};
use clap_sys::ext::gui::clap_window;
use std::ffi::CStr;

/// Main plugin trait - implement this to create a CLAP plugin
///
/// Note: `Send`/`Sync` are intentionally NOT required because plugin instances often
/// hold GUI state/handles which are not `Send`/`Sync`.
pub trait ClapPlugin: 'static {
    /// The processor type that handles audio processing
    type Processor: ClapProcessor;

    /// The parameter type
    type Params: PluginParams;

    /// Get plugin metadata and configuration (static)
    fn descriptor() -> PluginDescriptor;

    /// Get the CLAP descriptor pointer (for C API)
    fn clap_descriptor() -> &'static clap_sys::plugin::clap_plugin_descriptor;

    /// Create a new plugin instance
    fn new() -> Self;

    /// Initialize the plugin (called after construction)
    fn init(&mut self) {
        // Default: no-op
    }

    /// Create a new processor instance
    fn create_processor(&mut self, sample_rate: f32) -> Self::Processor;

    // ---------------------------------------------------------------------
    // Optional GUI support (CLAP_EXT_GUI)
    // ---------------------------------------------------------------------

    /// Whether this plugin provides a custom GUI.
    fn has_gui() -> bool {
        false
    }

    /// GUI: check whether an API is supported.
    fn gui_is_api_supported(&mut self, _api: &CStr, _is_floating: bool) -> bool {
        false
    }

    /// GUI: return preferred API (C string pointer) and whether floating.
    fn gui_get_preferred_api(&mut self, _api: *mut *const i8, _is_floating: *mut bool) -> bool {
        false
    }

    /// GUI: called before showing.
    fn gui_create(&mut self, _api: &CStr, _is_floating: bool) -> bool {
        true
    }

    /// GUI: destroy and release any window state.
    fn gui_destroy(&mut self) {}

    /// GUI: set scaling factor.
    fn gui_set_scale(&mut self, _scale: f64) -> bool {
        true
    }

    /// GUI: current preferred size.
    fn gui_get_size(&mut self, _width: *mut u32, _height: *mut u32) -> bool {
        false
    }

    /// GUI: whether window can resize.
    fn gui_can_resize(&mut self) -> bool {
        false
    }

    /// GUI: apply a new size.
    fn gui_set_size(&mut self, _width: u32, _height: u32) -> bool {
        false
    }

    /// GUI: set parent window handle.
    fn gui_set_parent(&mut self, _window: *const clap_window) -> bool {
        false
    }

    /// GUI: show the editor.
    fn gui_show(&mut self) -> bool {
        false
    }

    /// GUI: hide the editor.
    fn gui_hide(&mut self) -> bool {
        false
    }
}
