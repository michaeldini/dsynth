// CLAP plugin window integration with VIZIA
//
// This module is only compiled when the "clap" feature is enabled.

use crate::gui::shared_ui;
use crate::gui::theme;
use crate::gui::GuiState;
use crate::params::SynthParams;
use crate::plugin::gui_param_change::GuiParamChange;
use parking_lot::{Mutex, RwLock};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle as RawHandle};
use std::sync::Arc;
use triple_buffer::Input;
use vizia::prelude::*;

// Import Application and WindowHandle directly from vizia_baseview since vizia doesn't re-export
// when both baseview and winit features are enabled (conditional compilation in vizia/src/lib.rs)
use vizia_baseview::{Application, WindowHandle as ViziaBaseviewWindowHandle};

/// Wrapper to make RawWindowHandle implement HasRawWindowHandle
struct WindowHandleWrapper(RawHandle);

unsafe impl HasRawWindowHandle for WindowHandleWrapper {
    fn raw_window_handle(&self) -> RawHandle {
        self.0
    }
}

// NOTE: This should match the CLAP gui_get_size() defaults so the parented
// baseview surface fills the host-provided editor area instead of being
// letterboxed (e.g. in REAPER).

/// Opens a VIZIA-based plugin editor window.
///
/// Important: the returned handle must be kept alive by the plugin instance.
/// Dropping it can stop the baseview event loop and result in a "static" UI
/// (renders, but receives no input).
pub fn open_editor(
    parent_window: raw_window_handle::RawWindowHandle,
    synth_params: Arc<RwLock<SynthParams>>,
    gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,
) -> Option<EditorWindowHandle> {
    // Debug: Log GUI creation
    let debug_msg = "DEBUG: open_editor called - Starting GUI creation\n";
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/dsynth_debug.log")
        .and_then(|mut f| std::io::Write::write_all(&mut f, debug_msg.as_bytes()));

    // Wrap the raw window handle
    let window_wrapper = WindowHandleWrapper(parent_window);

    let handle = Application::new(move |cx| {
        // Initialize GUI state with shared parameter access
        GuiState::new(synth_params.clone(), gui_param_producer.clone()).build(cx);

        // Build the shared UI
        shared_ui::build_ui(cx);
    })
    .inner_size((theme::WINDOW_WIDTH, theme::WINDOW_HEIGHT))
    .open_parented(&window_wrapper);

    Some(EditorWindowHandle { _inner: handle })
}

/// Wrapper which keeps the baseview window handle alive.
pub struct EditorWindowHandle {
    _inner: ViziaBaseviewWindowHandle,
}
