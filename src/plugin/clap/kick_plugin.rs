/// CLAP Plugin Implementation for Kick Drum Synth
use super::kick_processor::KickClapProcessor;
use crate::params_kick::KickParams;
use crate::plugin::kick_param_registry::{get_kick_registry, ParamId};
use clap_sys::ext::audio_ports::{
    clap_audio_port_info, clap_plugin_audio_ports, CLAP_AUDIO_PORT_IS_MAIN,
};
#[cfg(target_os = "macos")]
use clap_sys::ext::gui::CLAP_WINDOW_API_COCOA;
#[cfg(target_os = "windows")]
use clap_sys::ext::gui::CLAP_WINDOW_API_WIN32;
#[cfg(target_os = "linux")]
use clap_sys::ext::gui::CLAP_WINDOW_API_X11;
use clap_sys::ext::gui::{clap_plugin_gui, clap_window};
use clap_sys::ext::note_ports::{
    clap_note_port_info, clap_plugin_note_ports, CLAP_NOTE_DIALECT_CLAP,
};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::version::CLAP_VERSION;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::OnceLock;

#[cfg(target_os = "macos")]
use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
#[cfg(target_os = "linux")]
use raw_window_handle::{RawWindowHandle, XlibWindowHandle};

use parking_lot::Mutex;
use std::sync::Arc;

#[cfg(target_os = "macos")]
use std::ffi::c_void;

#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use cocoa::foundation::NSRect;
#[cfg(target_os = "macos")]
use objc::msg_send;
#[cfg(target_os = "macos")]
use objc::sel;
#[cfg(target_os = "macos")]
use objc::sel_impl;

#[cfg(target_os = "macos")]
unsafe fn cocoa_nsview_bounds_size(ns_view_ptr: *mut c_void) -> Option<(u32, u32)> {
    if ns_view_ptr.is_null() {
        return None;
    }

    let ns_view = ns_view_ptr as id;
    let bounds: NSRect = msg_send![ns_view, bounds];

    let width = bounds.size.width.round().max(1.0) as u32;
    let height = bounds.size.height.round().max(1.0) as u32;
    Some((width, height))
}

fn kick_log_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("DSYNTH_KICK_LOG").is_some())
}

/// Log to file for debugging (since we have no console in DAW)
fn log_kick(msg: &str) {
    if !kick_log_enabled() {
        return;
    }
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/dsynth_kick_clap.log")
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
    }
}

/// Null-terminated feature list (slice lives for entire program lifetime)
const KICK_FEATURES: &[*const std::os::raw::c_char] = &[
    c"instrument".as_ptr(),
    c"synthesizer".as_ptr(),
    c"drum".as_ptr(),
    c"mono".as_ptr(),
    ptr::null(),
];

/// Plugin descriptor
static KICK_DESCRIPTOR: clap_sys::plugin::clap_plugin_descriptor =
    clap_sys::plugin::clap_plugin_descriptor {
        clap_version: CLAP_VERSION,
        id: c"com.dsynth.kick".as_ptr(),
        name: c"DSynth Kick".as_ptr(),
        vendor: c"DSynth".as_ptr(),
        url: c"https://github.com/yourusername/dsynth".as_ptr(),
        manual_url: c"https://github.com/yourusername/dsynth".as_ptr(),
        support_url: c"https://github.com/yourusername/dsynth".as_ptr(),
        version: c"0.3.0".as_ptr(),
        description: c"Monophonic kick drum synthesizer".as_ptr(),
        features: KICK_FEATURES.as_ptr(),
    };

/// DSynth Kick CLAP Plugin instance
pub struct KickClapPlugin {
    pub plugin: clap_plugin,
    _host: *const clap_host,
    pub processor: Option<KickClapProcessor>,
    params_ext: clap_sys::ext::params::clap_plugin_params,
    state_ext: clap_sys::ext::state::clap_plugin_state,
    audio_ports_ext: clap_plugin_audio_ports,
    note_ports_ext: clap_plugin_note_ports,
    gui_ext: clap_plugin_gui,
    gui_window: Option<Box<dyn std::any::Any>>,
    kick_params: Arc<Mutex<KickParams>>,

    // GUI sizing state
    gui_size: (u32, u32),
    gui_parent: Option<RawWindowHandle>,
}

impl KickClapPlugin {
    /// Create a new plugin instance
    pub fn new(host: *const clap_host) -> Box<Self> {
        let kick_params = Arc::new(Mutex::new(KickParams::default()));

        let mut plugin = Box::new(KickClapPlugin {
            plugin: clap_plugin {
                desc: &KICK_DESCRIPTOR,
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
            params_ext: create_params_ext(),
            state_ext: create_state_ext(),
            audio_ports_ext: create_audio_ports_ext(),
            note_ports_ext: create_note_ports_ext(),
            gui_ext: create_gui_ext(),
            gui_window: None,
            kick_params,

            gui_size: (720, 900),
            gui_parent: None,
        });

        plugin.plugin.plugin_data = plugin.as_mut() as *mut _ as *mut _;
        plugin
    }

    /// Get plugin instance from plugin pointer
    pub unsafe fn from_ptr<'a>(plugin: *const clap_plugin) -> &'a mut Self {
        unsafe { &mut *((*plugin).plugin_data as *mut Self) }
    }

    // CLAP Plugin Callbacks

    unsafe extern "C" fn init(plugin: *const clap_plugin) -> bool {
        log_kick("init called");
        if plugin.is_null() {
            log_kick("ERROR: plugin is null in init");
            return false;
        }

        let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
        // Create processor now since some hosts might not call activate()
        instance.processor = Some(KickClapProcessor::new(
            44100.0,
            instance.kick_params.clone(),
        ));
        log_kick("init successful");
        true
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        log_kick("destroy called");
        if plugin.is_null() {
            log_kick("ERROR: plugin is null in destroy");
            return;
        }

        let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
        unsafe {
            drop(Box::from_raw(instance as *mut KickClapPlugin));
        }
        log_kick("destroy successful");
    }

    unsafe extern "C" fn activate(
        plugin: *const clap_plugin,
        sample_rate: f64,
        _min_frames_count: u32,
        _max_frames_count: u32,
    ) -> bool {
        log_kick(&format!("activate called with sample_rate={}", sample_rate));
        if plugin.is_null() {
            log_kick("ERROR: plugin is null in activate");
            return false;
        }

        let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
        instance.processor = Some(KickClapProcessor::new(
            sample_rate as f32,
            instance.kick_params.clone(),
        ));
        log_kick("activate successful");
        true
    }

    unsafe extern "C" fn deactivate(_plugin: *const clap_plugin) {
        log_kick("deactivate called");
    }

    unsafe extern "C" fn start_processing(_plugin: *const clap_plugin) -> bool {
        log_kick("start_processing called");
        true
    }

    unsafe extern "C" fn stop_processing(_plugin: *const clap_plugin) {
        log_kick("stop_processing called");
    }

    unsafe extern "C" fn reset(_plugin: *const clap_plugin) {
        log_kick("reset called");
    }

    unsafe extern "C" fn process(
        plugin: *const clap_plugin,
        process: *const clap_sys::process::clap_process,
    ) -> clap_sys::process::clap_process_status {
        if plugin.is_null() || process.is_null() {
            log_kick("ERROR: plugin or process is null in process");
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
        if let Some(processor) = &mut instance.processor {
            unsafe { processor.process(process) }
        } else {
            log_kick("ERROR: processor is None in process");
            clap_sys::process::CLAP_PROCESS_ERROR
        }
    }

    unsafe extern "C" fn get_extension(
        plugin: *const clap_plugin,
        id: *const std::os::raw::c_char,
    ) -> *const std::ffi::c_void {
        log_kick("get_extension called");
        if plugin.is_null() || id.is_null() {
            log_kick("ERROR: plugin or id is null in get_extension");
            return ptr::null();
        }

        let id_str = unsafe { CStr::from_ptr(id) };
        log_kick(&format!("Requested extension: {:?}", id_str));

        let instance = unsafe { KickClapPlugin::from_ptr(plugin) };

        if id_str == clap_sys::ext::params::CLAP_EXT_PARAMS {
            log_kick("Returning params extension");
            return &instance.params_ext as *const _ as *const std::ffi::c_void;
        }

        if id_str == clap_sys::ext::state::CLAP_EXT_STATE {
            log_kick("Returning state extension");
            return &instance.state_ext as *const _ as *const std::ffi::c_void;
        }

        if id_str == clap_sys::ext::audio_ports::CLAP_EXT_AUDIO_PORTS {
            log_kick("Returning audio_ports extension");
            return &instance.audio_ports_ext as *const _ as *const std::ffi::c_void;
        }

        if id_str == clap_sys::ext::note_ports::CLAP_EXT_NOTE_PORTS {
            log_kick("Returning note_ports extension");
            return &instance.note_ports_ext as *const _ as *const std::ffi::c_void;
        }

        if id_str == clap_sys::ext::gui::CLAP_EXT_GUI {
            log_kick("Returning gui extension");
            return &instance.gui_ext as *const _ as *const std::ffi::c_void;
        }

        log_kick(&format!("Extension not found: {:?}", id_str));
        ptr::null()
    }

    unsafe extern "C" fn on_main_thread(_plugin: *const clap_plugin) {
        log_kick("on_main_thread called");
    }
}

// GUI Extension
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

unsafe extern "C" fn gui_is_api_supported(
    _plugin: *const clap_plugin,
    api: *const i8,
    _is_floating: bool,
) -> bool {
    if api.is_null() {
        return false;
    }

    let api_str = unsafe { CStr::from_ptr(api) };

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
    true
}

unsafe extern "C" fn gui_destroy(plugin: *const clap_plugin) {
    if plugin.is_null() {
        return;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
    log_kick("gui_destroy: dropping editor window and clearing parent");

    instance.gui_window = None;
    instance.gui_parent = None;
}

unsafe extern "C" fn gui_set_scale(_plugin: *const clap_plugin, _scale: f64) -> bool {
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

    if _plugin.is_null() {
        return false;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(_plugin) };

    unsafe {
        *width = instance.gui_size.0;
        *height = instance.gui_size.1;
    }
    true
}

unsafe extern "C" fn gui_can_resize(_plugin: *const clap_plugin) -> bool {
    true
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
    if _plugin.is_null() {
        return false;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(_plugin) };
    let width = _width.max(1);
    let height = _height.max(1);

    instance.gui_size = (width, height);

    // If the editor is open, reopen it at the new size.
    if let Some(parent) = instance.gui_parent {
        instance.gui_window = None;

        match crate::gui::kick_plugin_window::open_editor(
            parent,
            instance.kick_params.clone(),
            width,
            height,
        ) {
            Some(handle) => {
                instance.gui_window = Some(Box::new(handle));
                true
            }
            None => false,
        }
    } else {
        true
    }
}

unsafe extern "C" fn gui_set_parent(
    plugin: *const clap_plugin,
    window: *const clap_window,
) -> bool {
    if plugin.is_null() {
        return false;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(plugin) };

    // Drop any existing window before reparenting to avoid baseview holding a stale parent.
    if instance.gui_window.is_some() {
        log_kick("gui_set_parent: dropping existing window before creating new");
        instance.gui_window = None;
    }

    if window.is_null() {
        log_kick("gui_set_parent: received null parent, closing window");
        instance.gui_window = None;
        instance.gui_parent = None;
        return true;
    }

    let window_ref = unsafe { &*window };

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

    #[cfg(target_os = "macos")]
    let host_size = unsafe { cocoa_nsview_bounds_size(window_ref.specific.cocoa) };
    #[cfg(not(target_os = "macos"))]
    let host_size: Option<(u32, u32)> = None;

    let (width, height) = host_size.unwrap_or(instance.gui_size);
    instance.gui_size = (width, height);
    instance.gui_parent = Some(raw_handle);

    match crate::gui::kick_plugin_window::open_editor(
        raw_handle,
        instance.kick_params.clone(),
        width,
        height,
    ) {
        Some(handle) => {
            instance.gui_window = Some(Box::new(handle));
            true
        }
        None => false,
    }
}

unsafe extern "C" fn gui_set_transient(
    _plugin: *const clap_plugin,
    _window: *const clap_window,
) -> bool {
    false
}

unsafe extern "C" fn gui_suggest_title(_plugin: *const clap_plugin, _title: *const i8) {}

unsafe extern "C" fn gui_show(plugin: *const clap_plugin) -> bool {
    if plugin.is_null() {
        return false;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(plugin) };

    if instance.gui_window.is_some() {
        log_kick("gui_show: editor already open");
        return true;
    }

    let Some(parent) = instance.gui_parent else {
        log_kick("gui_show: no parent window set");
        return false;
    };

    let (width, height) = instance.gui_size;

    match crate::gui::kick_plugin_window::open_editor(
        parent,
        instance.kick_params.clone(),
        width,
        height,
    ) {
        Some(handle) => {
            instance.gui_window = Some(Box::new(handle));
            log_kick("gui_show: editor created");
            true
        }
        None => {
            log_kick("gui_show: failed to create editor");
            false
        }
    }
}

unsafe extern "C" fn gui_hide(plugin: *const clap_plugin) -> bool {
    if plugin.is_null() {
        return false;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(plugin) };

    if instance.gui_window.is_some() {
        log_kick("gui_hide: closing editor window");
    } else {
        log_kick("gui_hide: no window to close");
    }

    instance.gui_window = None;
    true
}

// Parameters Extension
fn create_params_ext() -> clap_sys::ext::params::clap_plugin_params {
    clap_sys::ext::params::clap_plugin_params {
        count: Some(params_count),
        get_info: Some(params_get_info),
        get_value: Some(params_get_value),
        value_to_text: Some(params_value_to_text),
        text_to_value: Some(params_text_to_value),
        flush: Some(params_flush),
    }
}

unsafe extern "C" fn params_count(_plugin: *const clap_plugin) -> u32 {
    get_kick_registry().param_count() as u32
}

unsafe extern "C" fn params_get_info(
    _plugin: *const clap_plugin,
    param_index: u32,
    param_info: *mut clap_sys::ext::params::clap_param_info,
) -> bool {
    if param_info.is_null() {
        return false;
    }

    let registry = get_kick_registry();
    let param_ids = registry.param_ids();

    if param_index as usize >= param_ids.len() {
        return false;
    }

    let param_id = param_ids[param_index as usize];
    let Some(desc) = registry.get_descriptor(param_id) else {
        return false;
    };

    fill_kick_param_info(param_info, param_id, desc);
    true
}

/// Fill CLAP param info structure from descriptor
fn fill_kick_param_info(
    info: *mut clap_sys::ext::params::clap_param_info,
    param_id: ParamId,
    descriptor: &crate::plugin::param_descriptor::ParamDescriptor,
) {
    use crate::plugin::param_descriptor::ParamType;
    use clap_sys::ext::params::CLAP_PARAM_IS_AUTOMATABLE;

    unsafe {
        (*info).id = param_id;
        (*info).flags = CLAP_PARAM_IS_AUTOMATABLE;

        // Copy parameter name
        let name_bytes = descriptor.name.as_bytes();
        let copy_len = name_bytes
            .len()
            .min(clap_sys::string_sizes::CLAP_NAME_SIZE - 1);
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            (*info).name.as_mut_ptr() as *mut u8,
            copy_len,
        );
        (*info).name[copy_len] = 0;

        // Copy module path
        let module_bytes = descriptor.module.as_bytes();
        let module_copy_len = module_bytes
            .len()
            .min(clap_sys::string_sizes::CLAP_PATH_SIZE - 1);
        std::ptr::copy_nonoverlapping(
            module_bytes.as_ptr(),
            (*info).module.as_mut_ptr() as *mut u8,
            module_copy_len,
        );
        (*info).module[module_copy_len] = 0;

        // Set min/max/default values in the parameter's plain value domain.
        match &descriptor.param_type {
            ParamType::Float { min, max, .. } => {
                (*info).min_value = *min as f64;
                (*info).max_value = *max as f64;
                (*info).default_value = descriptor.default as f64;
            }
            ParamType::Bool => {
                (*info).min_value = 0.0;
                (*info).max_value = 1.0;
                (*info).default_value = if descriptor.default > 0.5 { 1.0 } else { 0.0 };
            }
            ParamType::Enum { variants } => {
                // Enums use plain integer indices.
                let max_index = variants.len().saturating_sub(1);
                (*info).min_value = 0.0;
                (*info).max_value = max_index as f64;
                (*info).default_value = descriptor.default as f64;
            }
            ParamType::Int { min, max } => {
                (*info).min_value = *min as f64;
                (*info).max_value = *max as f64;
                (*info).default_value = descriptor.default as f64;
            }
        }
    }
}

unsafe extern "C" fn params_get_value(
    plugin: *const clap_plugin,
    param_id: ParamId,
    value: *mut f64,
) -> bool {
    if plugin.is_null() || value.is_null() {
        return false;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
    if let Some(processor) = &instance.processor {
        // CLAP uses the plain value domain here.
        let plain = processor.get_param(param_id);
        unsafe { *value = plain };
        true
    } else {
        false
    }
}

unsafe extern "C" fn params_value_to_text(
    _plugin: *const clap_plugin,
    param_id: ParamId,
    value: f64,
    display: *mut std::os::raw::c_char,
    size: u32,
) -> bool {
    if display.is_null() || size == 0 {
        return false;
    }

    let registry = get_kick_registry();
    let Some(desc) = registry.get_descriptor(param_id) else {
        return false;
    };

    // CLAP passes the plain value domain here. ParamDescriptor::format_value expects normalized.
    let normalized = desc.normalize_value(value as f32);
    let text = desc.format_value(normalized);
    let c_text = CString::new(text).unwrap_or_else(|_| CString::new("").unwrap());
    let bytes = c_text.as_bytes_with_nul();
    let copy_len = std::cmp::min(bytes.len(), size as usize);

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), display as *mut u8, copy_len);
    }

    true
}

unsafe extern "C" fn params_text_to_value(
    _plugin: *const clap_plugin,
    param_id: ParamId,
    display: *const std::os::raw::c_char,
    value: *mut f64,
) -> bool {
    if display.is_null() || value.is_null() {
        return false;
    }

    let text = unsafe { CStr::from_ptr(display) }.to_str().unwrap_or("");

    let registry = get_kick_registry();
    let Some(desc) = registry.get_descriptor(param_id) else {
        return false;
    };

    // Try to parse the numeric value from the string (strip units)
    let numeric_text = text.split_whitespace().next().unwrap_or(text);
    if let Ok(parsed) = numeric_text.parse::<f32>() {
        // Return the plain value domain back to the host.
        let clamped = match &desc.param_type {
            crate::plugin::param_descriptor::ParamType::Float { min, max, .. } => {
                parsed.clamp(*min, *max)
            }
            crate::plugin::param_descriptor::ParamType::Int { min, max } => {
                (parsed.round() as i32).clamp(*min, *max) as f32
            }
            crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                let max_index = variants.len().saturating_sub(1) as f32;
                parsed.round().clamp(0.0, max_index)
            }
            crate::plugin::param_descriptor::ParamType::Bool => {
                if parsed > 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
        };
        unsafe { *value = clamped as f64 };
        true
    } else {
        false
    }
}

unsafe extern "C" fn params_flush(
    plugin: *const clap_plugin,
    in_events: *const clap_sys::events::clap_input_events,
    _out_events: *const clap_sys::events::clap_output_events,
) {
    if plugin.is_null() || in_events.is_null() {
        return;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
    let Some(processor) = &mut instance.processor else {
        return;
    };

    let event_count = unsafe { ((*in_events).size.unwrap())(in_events) };

    for i in 0..event_count {
        let event_header = unsafe {
            ((*in_events).get.unwrap())(in_events, i) as *const clap_sys::events::clap_event_header
        };
        if event_header.is_null() {
            continue;
        }

        let header = unsafe { &*event_header };
        if header.type_ == clap_sys::events::CLAP_EVENT_PARAM_VALUE {
            let param_event = event_header as *const clap_sys::events::clap_event_param_value;
            let param = unsafe { &*param_event };

            // CLAP param events use the plain value domain.
            log_kick(&format!(
                "params_flush param_id=0x{:08x} value={}",
                param.param_id, param.value
            ));

            processor.set_param(param.param_id, param.value);
        }
    }
}

// State Extension
fn create_state_ext() -> clap_sys::ext::state::clap_plugin_state {
    clap_sys::ext::state::clap_plugin_state {
        save: Some(state_save),
        load: Some(state_load),
    }
}

unsafe extern "C" fn state_save(
    plugin: *const clap_plugin,
    stream: *const clap_sys::stream::clap_ostream,
) -> bool {
    if plugin.is_null() || stream.is_null() {
        return false;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
    let Some(processor) = &instance.processor else {
        return false;
    };

    let params = processor.get_params();
    let serialized = match bincode::serialize(&params) {
        Ok(data) => data,
        Err(_) => return false,
    };

    let stream_ref = unsafe { &*stream };
    let write_fn = stream_ref.write.unwrap();

    let bytes_written = unsafe {
        write_fn(
            stream,
            serialized.as_ptr() as *const std::ffi::c_void,
            serialized.len() as u64,
        )
    };

    bytes_written == serialized.len() as i64
}

unsafe extern "C" fn state_load(
    plugin: *const clap_plugin,
    stream: *const clap_sys::stream::clap_istream,
) -> bool {
    if plugin.is_null() || stream.is_null() {
        return false;
    }

    let instance = unsafe { KickClapPlugin::from_ptr(plugin) };
    let Some(processor) = &mut instance.processor else {
        return false;
    };

    let stream_ref = unsafe { &*stream };
    let read_fn = stream_ref.read.unwrap();

    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 4096];

    loop {
        let bytes_read = unsafe {
            read_fn(
                stream,
                chunk.as_mut_ptr() as *mut std::ffi::c_void,
                chunk.len() as u64,
            )
        };

        if bytes_read <= 0 {
            break;
        }

        buffer.extend_from_slice(&chunk[..bytes_read as usize]);
    }

    match bincode::deserialize::<KickParams>(&buffer) {
        Ok(params) => {
            processor.set_params(params);
            true
        }
        Err(_) => false,
    }
}

// Audio Ports Extension
fn create_audio_ports_ext() -> clap_plugin_audio_ports {
    clap_plugin_audio_ports {
        count: Some(audio_ports_count),
        get: Some(audio_ports_get),
    }
}

unsafe extern "C" fn audio_ports_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {
    if is_input {
        0
    } else {
        1
    }
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

    let info_ref = unsafe { &mut *info };
    info_ref.id = 0;

    let name = b"Main Output\0";
    let copy_len = std::cmp::min(name.len(), info_ref.name.len());
    unsafe {
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            info_ref.name.as_mut_ptr() as *mut u8,
            copy_len,
        );
    }

    info_ref.flags = CLAP_AUDIO_PORT_IS_MAIN;
    info_ref.channel_count = 2; // Stereo
    info_ref.port_type = ptr::null();
    info_ref.in_place_pair = clap_sys::id::CLAP_INVALID_ID;

    true
}

// Note Ports Extension
fn create_note_ports_ext() -> clap_plugin_note_ports {
    clap_plugin_note_ports {
        count: Some(note_ports_count),
        get: Some(note_ports_get),
    }
}

unsafe extern "C" fn note_ports_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {
    if is_input {
        1
    } else {
        0
    }
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

    let info_ref = unsafe { &mut *info };
    info_ref.id = 0;

    let name = b"Note Input\0";
    let copy_len = std::cmp::min(name.len(), info_ref.name.len());
    unsafe {
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            info_ref.name.as_mut_ptr() as *mut u8,
            copy_len,
        );
    }

    info_ref.supported_dialects = CLAP_NOTE_DIALECT_CLAP;
    info_ref.preferred_dialect = CLAP_NOTE_DIALECT_CLAP;

    true
}

// CLAP Entry Point
// CRITICAL: Must be a static variable with C linkage, not a function!
// The CLAP spec requires hosts to look up this symbol as a static struct.
#[no_mangle]
#[link_section = "__DATA,__data"]
pub static clap_entry: clap_sys::entry::clap_plugin_entry = clap_sys::entry::clap_plugin_entry {
    clap_version: CLAP_VERSION,
    init: Some(entry_init),
    deinit: Some(entry_deinit),
    get_factory: Some(entry_get_factory),
};

unsafe extern "C" fn entry_init(_plugin_path: *const std::os::raw::c_char) -> bool {
    log_kick("entry_init called");
    true
}

unsafe extern "C" fn entry_deinit() {
    log_kick("entry_deinit called");
}

unsafe extern "C" fn entry_get_factory(
    factory_id: *const std::os::raw::c_char,
) -> *const std::ffi::c_void {
    log_kick("entry_get_factory called");
    if factory_id.is_null() {
        log_kick("ERROR: factory_id is null");
        return ptr::null();
    }

    let id = unsafe { CStr::from_ptr(factory_id) };
    log_kick(&format!("Requested factory: {:?}", id));
    if id.to_bytes_with_nul() == CLAP_PLUGIN_FACTORY_ID.to_bytes_with_nul() {
        log_kick("Returning kick plugin factory");
        return &KICK_FACTORY as *const _ as *const std::ffi::c_void;
    }

    log_kick("ERROR: Factory not found");
    ptr::null()
}

static KICK_FACTORY: clap_plugin_factory = clap_plugin_factory {
    get_plugin_count: Some(factory_get_plugin_count),
    get_plugin_descriptor: Some(factory_get_plugin_descriptor),
    create_plugin: Some(factory_create_plugin),
};

unsafe extern "C" fn factory_get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {
    1
}

unsafe extern "C" fn factory_get_plugin_descriptor(
    _factory: *const clap_plugin_factory,
    index: u32,
) -> *const clap_sys::plugin::clap_plugin_descriptor {
    if index == 0 {
        &KICK_DESCRIPTOR
    } else {
        ptr::null()
    }
}

unsafe extern "C" fn factory_create_plugin(
    _factory: *const clap_plugin_factory,
    host: *const clap_host,
    plugin_id: *const std::os::raw::c_char,
) -> *const clap_plugin {
    log_kick("factory_create_plugin called");
    if host.is_null() || plugin_id.is_null() {
        log_kick("ERROR: host or plugin_id is null");
        return ptr::null();
    }

    let id = unsafe { CStr::from_ptr(plugin_id) };
    log_kick(&format!("Requested plugin ID: {:?}", id));
    let expected_id = unsafe { CStr::from_ptr(KICK_DESCRIPTOR.id) };
    log_kick(&format!("Expected plugin ID: {:?}", expected_id));

    if id != expected_id {
        log_kick("ERROR: ID mismatch");
        return ptr::null();
    }

    log_kick("Creating KickClapPlugin instance...");
    let plugin_box = KickClapPlugin::new(host);
    let plugin_ptr = Box::into_raw(plugin_box);

    // Return pointer to the clap_plugin field (first field in struct)
    let clap_plugin_ptr = unsafe { &(*plugin_ptr).plugin as *const clap_plugin };
    log_kick(&format!("Plugin struct at {:p}", plugin_ptr));
    log_kick(&format!("Returning clap_plugin at {:p}", clap_plugin_ptr));
    log_kick(&format!("Plugin descriptor at {:p}", unsafe {
        (*clap_plugin_ptr).desc
    }));
    clap_plugin_ptr
}
