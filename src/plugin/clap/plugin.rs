use clap_sys::ext::audio_ports::{
    CLAP_AUDIO_PORT_IS_MAIN, clap_audio_port_info, clap_plugin_audio_ports,
};
use clap_sys::ext::gui::{CLAP_WINDOW_API_COCOA, clap_plugin_gui, clap_window};
use clap_sys::ext::note_ports::{
    CLAP_NOTE_DIALECT_CLAP, clap_note_port_info, clap_plugin_note_ports,
};
use clap_sys::host::clap_host;
/// CLAP Plugin Implementation
///
/// Main plugin structure and CLAP interface implementation.
use clap_sys::plugin::clap_plugin;
use std::ffi::CStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::ptr;

#[cfg(target_os = "macos")]
use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
#[cfg(target_os = "linux")]
use raw_window_handle::{RawWindowHandle, XlibWindowHandle};

fn log_to_file(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/dsynth_clap.log")
    {
        let _ = writeln!(
            file,
            "[{}] {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            msg
        );
        let _ = file.sync_all(); // Force flush to disk
    }
}

use super::descriptor;
use super::params;
use super::processor::ClapProcessor;
use super::state;

pub use descriptor::DESCRIPTOR;

use crate::params::SynthParams;
use crate::plugin::gui_param_change::GuiParamChange;
use crate::plugin::param_update::ParamUpdateBuffer;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use triple_buffer::{Input, Output, TripleBuffer};

/// DSynth CLAP Plugin instance
pub struct DSynthClapPlugin {
    pub plugin: clap_plugin,
    _host: *const clap_host,
    pub processor: Option<ClapProcessor>,
    pub current_params: SynthParams,
    params_ext: clap_sys::ext::params::clap_plugin_params,
    state_ext: clap_sys::ext::state::clap_plugin_state,
    audio_ports_ext: clap_plugin_audio_ports,
    note_ports_ext: clap_plugin_note_ports,
    gui_ext: clap_plugin_gui,
    gui_window: Option<Box<dyn std::any::Any>>,
    /// Shared parameter state for GUI
    pub synth_params: Arc<RwLock<SynthParams>>,
    /// Parameter update buffer for GUI → audio thread communication
    pub param_update_buffer: Arc<ParamUpdateBuffer>,

    /// GUI -> audio thread param-change producer (lock-free buffer)
    pub gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,
    /// Stored consumer end so it can survive processor recreation
    gui_param_consumer: Option<Output<GuiParamChange>>,
}

impl DSynthClapPlugin {
    /// Create a new plugin instance
    pub fn new(host: *const clap_host) -> Box<Self> {
        let gui_param_buffer = TripleBuffer::new(&GuiParamChange::default());
        let (gui_param_producer, gui_param_consumer) = gui_param_buffer.split();

        let mut plugin = Box::new(DSynthClapPlugin {
            plugin: clap_plugin {
                desc: &DESCRIPTOR,
                plugin_data: ptr::null_mut(),
                init: Some(Self::init),
                destroy: Some(Self::destroy),
                activate: Some(Self::activate),
                deactivate: Some(Self::deactivate),
                start_processing: Some(Self::start_processing),
                stop_processing: Some(Self::stop_processing),
                reset: Some(Self::reset),
                process: Some(Self::process),
                get_extension: Some(Self::get_extension),
                on_main_thread: Some(Self::on_main_thread),
            },
            _host: host,
            processor: None,
            current_params: SynthParams::default(),
            params_ext: params::create_params_ext(),
            state_ext: state::create_state_ext(),
            audio_ports_ext: create_audio_ports_ext(),
            note_ports_ext: create_note_ports_ext(),
            gui_ext: create_gui_ext(),
            gui_window: None,
            synth_params: Arc::new(RwLock::new(SynthParams::default())),
            param_update_buffer: Arc::new(ParamUpdateBuffer::new()),
            gui_param_producer: Arc::new(Mutex::new(gui_param_producer)),
            gui_param_consumer: Some(gui_param_consumer),
        });

        // Set plugin_data to point to self
        plugin.plugin.plugin_data = plugin.as_mut() as *mut _ as *mut _;
        plugin
    }

    /// Get plugin instance from plugin pointer
    unsafe fn from_ptr<'a>(plugin: *const clap_plugin) -> &'a mut Self {
        unsafe { &mut *((*plugin).plugin_data as *mut Self) }
    }

    // CLAP Plugin Callbacks (must be associated functions for Self:: references)

    unsafe extern "C" fn init(plugin: *const clap_plugin) -> bool {
        if plugin.is_null() {
            return false;
        }

        let instance = unsafe { DSynthClapPlugin::from_ptr(plugin) };

        // Create processor now since some hosts might not call activate()
        let consumer = instance.gui_param_consumer.take().unwrap_or_else(|| {
            let buf = TripleBuffer::new(&GuiParamChange::default());
            let (_p, c) = buf.split();
            c
        });

        let processor = ClapProcessor::new(44100.0, consumer);
        instance.processor = Some(processor);

        true
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        if plugin.is_null() {
            return;
        }

        let instance = unsafe { DSynthClapPlugin::from_ptr(plugin) };
        unsafe {
            drop(Box::from_raw(instance as *mut DSynthClapPlugin));
        }
    }

    unsafe extern "C" fn activate(
        plugin: *const clap_plugin,
        sample_rate: f64,
        _min_frames_count: u32,
        _max_frames_count: u32,
    ) -> bool {
        if plugin.is_null() {
            return false;
        }

        let instance = unsafe { DSynthClapPlugin::from_ptr(plugin) };

        // Recover the GUI consumer from any existing processor before recreating.
        if let Some(old) = &mut instance.processor {
            if let Some(cons) = old.take_gui_param_consumer() {
                instance.gui_param_consumer = Some(cons);
            }
        }

        // Create or recreate processor with the host's sample rate
        let consumer = instance.gui_param_consumer.take().unwrap_or_else(|| {
            let buf = TripleBuffer::new(&GuiParamChange::default());
            let (_p, c) = buf.split();
            c
        });
        let mut processor = ClapProcessor::new(sample_rate as f32, consumer);

        // Sync any parameters that were set during init to the processor
        processor.current_params = instance.current_params;
        processor.param_producer.write(instance.current_params);

        instance.processor = Some(processor);

        true
    }

    unsafe extern "C" fn deactivate(plugin: *const clap_plugin) {
        if plugin.is_null() {
            return;
        }

        let instance = unsafe { DSynthClapPlugin::from_ptr(plugin) };

        // Recover GUI consumer, then destroy processor when deactivating
        if let Some(processor) = &mut instance.processor {
            if let Some(cons) = processor.take_gui_param_consumer() {
                instance.gui_param_consumer = Some(cons);
            }
        }
        instance.processor = None;
    }

    unsafe extern "C" fn start_processing(_plugin: *const clap_plugin) -> bool {
        true
    }

    unsafe extern "C" fn stop_processing(_plugin: *const clap_plugin) {
        // No special processing stop logic needed
    }

    unsafe extern "C" fn reset(plugin: *const clap_plugin) {
        if plugin.is_null() {
            return;
        }

        let instance = unsafe { DSynthClapPlugin::from_ptr(plugin) };

        if let Some(processor) = &mut instance.processor {
            let processor: &mut ClapProcessor = processor;
            processor.deactivate();
        }
    }

    unsafe extern "C" fn process(
        plugin: *const clap_plugin,
        process: *const clap_sys::process::clap_process,
    ) -> i32 {
        if plugin.is_null() || process.is_null() {
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        let instance = unsafe { DSynthClapPlugin::from_ptr(plugin) };

        if let Some(processor) = &mut instance.processor {
            let processor: &mut ClapProcessor = processor;
            unsafe { processor.process(process) }
        } else {
            clap_sys::process::CLAP_PROCESS_ERROR
        }
    }

    unsafe extern "C" fn get_extension(
        plugin: *const clap_plugin,
        id: *const i8,
    ) -> *const std::ffi::c_void {
        if plugin.is_null() || id.is_null() {
            return ptr::null();
        }

        let instance = unsafe { DSynthClapPlugin::from_ptr(plugin) };
        let id_str = unsafe { CStr::from_ptr(id) };

        if id_str == CStr::from_bytes_with_nul(b"clap.params\0").unwrap() {
            &instance.params_ext as *const _ as *const std::ffi::c_void
        } else if id_str == CStr::from_bytes_with_nul(b"clap.state\0").unwrap() {
            &instance.state_ext as *const _ as *const std::ffi::c_void
        } else if id_str == CStr::from_bytes_with_nul(b"clap.audio-ports\0").unwrap() {
            &instance.audio_ports_ext as *const _ as *const std::ffi::c_void
        } else if id_str == CStr::from_bytes_with_nul(b"clap.note-ports\0").unwrap() {
            &instance.note_ports_ext as *const _ as *const std::ffi::c_void
        } else if id_str == CStr::from_bytes_with_nul(b"clap.gui\0").unwrap() {
            &instance.gui_ext as *const _ as *const std::ffi::c_void
        } else {
            ptr::null()
        }
    }

    unsafe extern "C" fn on_main_thread(_plugin: *const clap_plugin) {
        // Called periodically on main thread for GUI updates
    }
}

// Audio Ports Extension

unsafe extern "C" fn audio_ports_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {
    if is_input { 0 } else { 1 } // No audio input, one stereo output
}

unsafe extern "C" fn audio_ports_get(
    _plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool {
    if info.is_null() || is_input || index != 0 {
        return false;
    }

    let info = unsafe { &mut *info };
    info.id = 0;

    // Copy port name
    let name = b"Audio Output\0";
    let copy_len = name.len().min(clap_sys::string_sizes::CLAP_NAME_SIZE - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(name.as_ptr(), info.name.as_mut_ptr() as *mut u8, copy_len);
    }

    info.flags = CLAP_AUDIO_PORT_IS_MAIN;
    info.channel_count = 2; // Stereo
    info.port_type = ptr::null(); // Default port type
    info.in_place_pair = clap_sys::id::CLAP_INVALID_ID;

    true
}

fn create_audio_ports_ext() -> clap_plugin_audio_ports {
    clap_plugin_audio_ports {
        count: Some(audio_ports_count),
        get: Some(audio_ports_get),
    }
}

// Note Ports Extension (for MIDI input)

unsafe extern "C" fn note_ports_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {
    if is_input { 1 } else { 0 } // One note input port, no note output
}

unsafe extern "C" fn note_ports_get(
    _plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_note_port_info,
) -> bool {
    if info.is_null() || !is_input || index != 0 {
        return false;
    }

    let info = unsafe { &mut *info };
    info.id = 0;

    // Copy port name
    let name = b"Note Input\0";
    let copy_len = name.len().min(clap_sys::string_sizes::CLAP_NAME_SIZE - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(name.as_ptr(), info.name.as_mut_ptr() as *mut u8, copy_len);
    }

    info.supported_dialects = CLAP_NOTE_DIALECT_CLAP;
    info.preferred_dialect = CLAP_NOTE_DIALECT_CLAP;

    true
}

fn create_note_ports_ext() -> clap_plugin_note_ports {
    clap_plugin_note_ports {
        count: Some(note_ports_count),
        get: Some(note_ports_get),
    }
}

/// Create GUI extension for CLAP
fn create_gui_ext() -> clap_plugin_gui {
    clap_plugin_gui {
        is_api_supported: Some(gui_is_api_supported),
        get_preferred_api: Some(gui_get_preferred_api),
        create: Some(gui_create),
        destroy: Some(gui_destroy),
        set_scale: Some(gui_set_scale),
        get_size: Some(gui_get_size),
        can_resize: Some(gui_can_resize),
        get_resize_hints: Some(gui_get_resize_hints),
        adjust_size: Some(gui_adjust_size),
        set_size: Some(gui_set_size),
        set_parent: Some(gui_set_parent),
        set_transient: Some(gui_set_transient),
        suggest_title: Some(gui_suggest_title),
        show: Some(gui_show),
        hide: Some(gui_hide),
    }
}

// GUI Extension Callbacks

unsafe extern "C" fn gui_is_api_supported(
    _plugin: *const clap_plugin,
    api: *const i8,
    _is_floating: bool,
) -> bool {
    if api.is_null() {
        return false;
    }

    let api_str = unsafe { CStr::from_ptr(api) };

    // Support platform-specific windowing APIs
    #[cfg(target_os = "macos")]
    return api_str.to_bytes() == CLAP_WINDOW_API_COCOA.to_bytes();

    #[cfg(target_os = "windows")]
    return api_str.to_bytes() == CLAP_WINDOW_API_WIN32.to_bytes();

    #[cfg(target_os = "linux")]
    return api_str.to_bytes() == CLAP_WINDOW_API_X11.to_bytes();

    #[allow(unreachable_code)]
    false
}

unsafe extern "C" fn gui_get_preferred_api(
    _plugin: *const clap_plugin,
    api: *mut *const i8,
    _is_floating: *mut bool,
) -> bool {
    if api.is_null() {
        return false;
    }

    #[cfg(target_os = "macos")]
    {
        unsafe {
            *api = CLAP_WINDOW_API_COCOA.as_ptr() as *const i8;
        }
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        *api = CLAP_WINDOW_API_WIN32.as_ptr() as *const i8;
        return true;
    }

    #[cfg(target_os = "linux")]
    {
        *api = CLAP_WINDOW_API_X11.as_ptr() as *const i8;
        return true;
    }

    #[allow(unreachable_code)]
    false
}

unsafe extern "C" fn gui_create(
    _plugin: *const clap_plugin,
    _api: *const i8,
    _is_floating: bool,
) -> bool {
    // GUI window will be created in set_parent
    true
}

unsafe extern "C" fn gui_destroy(_plugin: *const clap_plugin) {
    // GUI cleanup handled in set_parent when parent is null
}

unsafe extern "C" fn gui_set_scale(_plugin: *const clap_plugin, _scale: f64) -> bool {
    // TODO: Update GUI scale factor
    true
}

unsafe extern "C" fn gui_get_size(
    _plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    if width.is_null() || height.is_null() {
        return false;
    }

    unsafe {
        *width = 1200;
        *height = 800;
    }
    true
}

unsafe extern "C" fn gui_can_resize(_plugin: *const clap_plugin) -> bool {
    false // Fixed size for now
}

unsafe extern "C" fn gui_get_resize_hints(
    _plugin: *const clap_plugin,
    _hints: *mut clap_sys::ext::gui::clap_gui_resize_hints,
) -> bool {
    false
}

unsafe extern "C" fn gui_adjust_size(
    _plugin: *const clap_plugin,
    _width: *mut u32,
    _height: *mut u32,
) -> bool {
    false
}

unsafe extern "C" fn gui_set_size(_plugin: *const clap_plugin, _width: u32, _height: u32) -> bool {
    false
}

unsafe extern "C" fn gui_set_parent(
    plugin: *const clap_plugin,
    window: *const clap_window,
) -> bool {
    if plugin.is_null() {
        return false;
    }

    let instance = unsafe { DSynthClapPlugin::from_ptr(plugin) };

    if window.is_null() {
        // Close GUI
        instance.gui_window = None;
        return true;
    }

    // Get the window pointer from the CLAP window structure
    let window_ref = unsafe { &*window };

    // Create the appropriate RawWindowHandle for the target platform (raw-window-handle 0.5 API)
    #[cfg(target_os = "macos")]
    let raw_handle = unsafe {
        let cocoa_ptr = window_ref.specific.cocoa;
        let mut handle = AppKitWindowHandle::empty();
        handle.ns_view = cocoa_ptr;
        RawWindowHandle::AppKit(handle)
    };

    #[cfg(target_os = "windows")]
    let raw_handle = unsafe {
        let mut handle = Win32WindowHandle::empty();
        handle.hwnd = window_ref.specific.win32;
        RawWindowHandle::Win32(handle)
    };

    #[cfg(target_os = "linux")]
    let raw_handle = unsafe {
        let mut handle = XlibWindowHandle::empty();
        handle.window = window_ref.specific.x11;
        RawWindowHandle::Xlib(handle)
    };

    // Open the VIZIA window with proper parameter update buffer
    match crate::gui::vizia_gui::plugin_window::open_editor(
        raw_handle,
        instance.synth_params.clone(),
        instance.gui_param_producer.clone(),
    ) {
        Some(handle) => {
            instance.gui_window = Some(Box::new(handle));
            true
        }
        None => {
            log_to_file("Failed to open VIZIA GUI");
            false
        }
    }
}

unsafe extern "C" fn gui_set_transient(
    _plugin: *const clap_plugin,
    _window: *const clap_window,
) -> bool {
    false
}

unsafe extern "C" fn gui_suggest_title(_plugin: *const clap_plugin, _title: *const i8) {
    // Optional: update window title
}

unsafe extern "C" fn gui_show(_plugin: *const clap_plugin) -> bool {
    true
}

unsafe extern "C" fn gui_hide(_plugin: *const clap_plugin) -> bool {
    true
}
