/// CLAP Voice Enhancer Plugin
///
/// Main plugin interface implementing CLAP lifecycle and extensions.
/// This plugin processes audio input through a vocal enhancement chain.
use clap_sys::ext::audio_ports::{
    clap_audio_port_info, clap_plugin_audio_ports, CLAP_AUDIO_PORT_IS_MAIN, CLAP_EXT_AUDIO_PORTS,
};
use clap_sys::ext::params::{clap_plugin_params, CLAP_EXT_PARAMS};
use clap_sys::ext::state::{clap_plugin_state, CLAP_EXT_STATE};
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::version::CLAP_VERSION;
use clap_sys::{entry::clap_plugin_entry, host::clap_host};

use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, RwLock};

use crate::params_voice::VoiceParams;
use crate::plugin::voice_param_registry::{self, get_voice_registry};

use super::voice_processor::VoiceProcessor;

// Plugin metadata
const PLUGIN_ID: &str = "com.dsynth.voice-enhancer";
const PLUGIN_NAME: &str = "DSynth Voice Enhancer";
const PLUGIN_VENDOR: &str = "DSynth";
const PLUGIN_URL: &str = "https://github.com/yourusername/dsynth";
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
const PLUGIN_DESCRIPTION: &str = "Voice enhancement with pitch-tracked sub oscillator";

/// Main plugin struct
pub struct VoicePlugin {
    host: *const clap_host,
    processor: Option<VoiceProcessor>,
    params: Arc<RwLock<VoiceParams>>,
}

impl VoicePlugin {
    fn new(host: *const clap_host) -> Box<Self> {
        Box::new(Self {
            host,
            processor: None,
            params: Arc::new(RwLock::new(VoiceParams::default())),
        })
    }

    fn host(&self) -> &clap_host {
        unsafe { &*self.host }
    }
}

// ============================================================================
// CLAP Plugin Interface Implementation
// ============================================================================

unsafe extern "C" fn plugin_init(plugin: *const clap_plugin) -> bool {
    true // No initialization needed
}

unsafe extern "C" fn plugin_destroy(plugin: *const clap_plugin) {
    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    if !plugin_data.is_null() {
        drop(Box::from_raw(plugin_data));
    }
}

unsafe extern "C" fn plugin_activate(
    plugin: *const clap_plugin,
    sample_rate: f64,
    _min_frames_count: u32,
    _max_frames_count: u32,
) -> bool {
    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    let plugin = &mut *plugin_data;

    let processor = VoiceProcessor::new(sample_rate as f32, plugin.params.clone());
    plugin.processor = Some(processor);

    true
}

unsafe extern "C" fn plugin_deactivate(plugin: *const clap_plugin) {
    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    let plugin = &mut *plugin_data;

    if let Some(ref mut processor) = plugin.processor {
        processor.deactivate();
    }
    plugin.processor = None;
}

unsafe extern "C" fn plugin_start_processing(plugin: *const clap_plugin) -> bool {
    true
}

unsafe extern "C" fn plugin_stop_processing(plugin: *const clap_plugin) {
    // Nothing to do
}

unsafe extern "C" fn plugin_reset(plugin: *const clap_plugin) {
    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    let plugin = &mut *plugin_data;

    if let Some(ref mut processor) = plugin.processor {
        processor.reset();
    }
}

unsafe extern "C" fn plugin_process(
    plugin: *const clap_plugin,
    process: *const clap_sys::process::clap_process,
) -> clap_sys::process::clap_process_status {
    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    let plugin = &mut *plugin_data;

    if let Some(ref mut processor) = plugin.processor {
        processor.process(process)
    } else {
        clap_sys::process::CLAP_PROCESS_SLEEP
    }
}

unsafe extern "C" fn plugin_get_extension(
    _plugin: *const clap_plugin,
    id: *const c_char,
) -> *const c_void {
    let id = CStr::from_ptr(id).to_bytes_with_nul();

    if id == CLAP_EXT_PARAMS.to_bytes_with_nul() {
        &PLUGIN_PARAMS as *const _ as *const c_void
    } else if id == CLAP_EXT_AUDIO_PORTS.to_bytes_with_nul() {
        &PLUGIN_AUDIO_PORTS as *const _ as *const c_void
    } else if id == CLAP_EXT_STATE.to_bytes_with_nul() {
        &PLUGIN_STATE as *const _ as *const c_void
    } else {
        ptr::null()
    }
}

unsafe extern "C" fn plugin_on_main_thread(_plugin: *const clap_plugin) {
    // Nothing to do
}

// ============================================================================
// Audio Ports Extension
// ============================================================================

unsafe extern "C" fn audio_ports_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {
    1 // One stereo port for input and output
}

unsafe extern "C" fn audio_ports_get(
    _plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool {
    if index != 0 {
        return false;
    }

    let info = &mut *info;
    info.id = if is_input { 0 } else { 1 };

    let name = if is_input {
        CString::new("Audio Input").unwrap()
    } else {
        CString::new("Audio Output").unwrap()
    };

    let name_bytes = name.as_bytes_with_nul();
    let len = name_bytes.len().min(256);
    ptr::copy_nonoverlapping(name_bytes.as_ptr(), info.name.as_mut_ptr() as *mut u8, len);

    info.flags = CLAP_AUDIO_PORT_IS_MAIN;
    info.channel_count = 2;
    info.port_type = b"stereo\0".as_ptr() as *const c_char;
    info.in_place_pair = if is_input { 1 } else { 0 }; // Input pairs with output

    true
}

static PLUGIN_AUDIO_PORTS: clap_plugin_audio_ports = clap_plugin_audio_ports {
    count: Some(audio_ports_count),
    get: Some(audio_ports_get),
};

// ============================================================================
// Parameters Extension
// ============================================================================

unsafe extern "C" fn params_count(_plugin: *const clap_plugin) -> u32 {
    get_voice_registry().len() as u32
}

unsafe extern "C" fn params_get_info(
    _plugin: *const clap_plugin,
    param_index: u32,
    param_info: *mut clap_sys::ext::params::clap_param_info,
) -> bool {
    let param_index = param_index as usize;
    let registry = get_voice_registry();
    let params: Vec<_> = registry.keys().copied().collect();

    if param_index >= params.len() {
        return false;
    }

    let param_id = params[param_index];
    if let Some(descriptor) = voice_param_registry::get_param_descriptor(param_id) {
        fill_voice_param_info(param_info, param_id, descriptor);
        true
    } else {
        false
    }
}

/// Fill CLAP param info from descriptor
fn fill_voice_param_info(
    info: *mut clap_sys::ext::params::clap_param_info,
    param_id: u32,
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

        // Set min/max/default values - CLAP expects normalized 0.0-1.0 range
        match &descriptor.param_type {
            ParamType::Float { min, max, .. } => {
                (*info).min_value = 0.0;
                (*info).max_value = 1.0;
                // Normalize default value
                let normalized_default = (descriptor.default - min) / (max - min);
                (*info).default_value = normalized_default.clamp(0.0, 1.0) as f64;
            }
            ParamType::Bool => {
                (*info).min_value = 0.0;
                (*info).max_value = 1.0;
                (*info).default_value = if descriptor.default > 0.5 { 1.0 } else { 0.0 };
            }
            ParamType::Enum { variants } => {
                (*info).min_value = 0.0;
                (*info).max_value = 1.0;
                // Normalize enum index to 0-1
                let max_index = (variants.len().saturating_sub(1)) as f32;
                let normalized_default = if max_index > 0.0 {
                    descriptor.default / max_index
                } else {
                    0.0
                };
                (*info).default_value = normalized_default.clamp(0.0, 1.0) as f64;
            }
            ParamType::Int { min, max } => {
                (*info).min_value = 0.0;
                (*info).max_value = 1.0;
                // Normalize int value
                let range = (max - min) as f32;
                let normalized_default = if range > 0.0 {
                    (descriptor.default - (*min as f32)) / range
                } else {
                    0.0
                };
                (*info).default_value = normalized_default.clamp(0.0, 1.0) as f64;
            }
        }
    }
}

unsafe extern "C" fn params_get_value(
    plugin: *const clap_plugin,
    param_id: u32,
    value: *mut f64,
) -> bool {
    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    let plugin = &*plugin_data;

    if let Ok(params_guard) = plugin.params.read() {
        if let Some(denorm_value) = voice_param_registry::get_param(&params_guard, param_id) {
            if let Some(descriptor) = voice_param_registry::get_param_descriptor(param_id) {
                // Normalize to 0.0-1.0 range
                let normalized = match &descriptor.param_type {
                    crate::plugin::param_descriptor::ParamType::Float { min, max, .. } => {
                        (denorm_value - min) / (max - min)
                    }
                    crate::plugin::param_descriptor::ParamType::Bool => denorm_value,
                    crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                        denorm_value / (variants.len() - 1) as f32
                    }
                    crate::plugin::param_descriptor::ParamType::Int { min, max } => {
                        (denorm_value - (*min as f32)) / ((max - min) as f32)
                    }
                };
                *value = normalized as f64;
                return true;
            }
        }
    }

    false
}

unsafe extern "C" fn params_value_to_text(
    _plugin: *const clap_plugin,
    param_id: u32,
    value: f64,
    display: *mut c_char,
    size: u32,
) -> bool {
    if let Some(descriptor) = voice_param_registry::get_param_descriptor(param_id) {
        // Denormalize from 0.0-1.0 to parameter range
        let denorm_value = match &descriptor.param_type {
            crate::plugin::param_descriptor::ParamType::Float { min, max, .. } => {
                min + (value as f32) * (max - min)
            }
            crate::plugin::param_descriptor::ParamType::Bool => value as f32,
            crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                ((value as f32) * (variants.len() - 1) as f32).round()
            }
            crate::plugin::param_descriptor::ParamType::Int { min, max } => {
                (*min as f32) + (value as f32) * ((max - min) as f32)
            }
        };

        let text = format!(
            "{:.2}{}",
            denorm_value,
            descriptor.unit.as_deref().unwrap_or("")
        );
        let text_cstr = CString::new(text).unwrap();
        let text_bytes = text_cstr.as_bytes_with_nul();
        let len = text_bytes.len().min(size as usize);
        ptr::copy_nonoverlapping(text_bytes.as_ptr(), display as *mut u8, len);
        true
    } else {
        false
    }
}

unsafe extern "C" fn params_text_to_value(
    _plugin: *const clap_plugin,
    param_id: u32,
    display: *const c_char,
    value: *mut f64,
) -> bool {
    // Not implemented - text parsing is optional
    false
}

unsafe extern "C" fn params_flush(
    plugin: *const clap_plugin,
    in_events: *const clap_sys::events::clap_input_events,
    _out_events: *const clap_sys::events::clap_output_events,
) {
    // Parameters are applied in process() callback via events
    // This is called when not processing (e.g., when stopped)
    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    let plugin = &mut *plugin_data;

    if !in_events.is_null() && plugin.processor.is_some() {
        // Process parameter changes even when not actively processing audio
        let processor = plugin.processor.as_mut().unwrap();

        let events = &*in_events;
        let event_count = (events.size.unwrap())(events);

        for i in 0..event_count {
            let event_header = (events.get.unwrap())(events, i);
            if event_header.is_null() {
                continue;
            }

            let header = &*event_header;
            if header.type_ == clap_sys::events::CLAP_EVENT_PARAM_VALUE {
                let param_event = event_header as *const clap_sys::events::clap_event_param_value;
                let param_event = &*param_event;

                // Denormalize from CLAP 0.0-1.0 to parameter range
                if let Some(descriptor) =
                    voice_param_registry::get_param_descriptor(param_event.param_id)
                {
                    let denorm_value = match &descriptor.param_type {
                        crate::plugin::param_descriptor::ParamType::Float { min, max, .. } => {
                            min + (param_event.value as f32) * (max - min)
                        }
                        crate::plugin::param_descriptor::ParamType::Bool => {
                            param_event.value as f32
                        }
                        crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                            ((param_event.value as f32) * (variants.len() - 1) as f32).round()
                        }
                        crate::plugin::param_descriptor::ParamType::Int { min, max } => {
                            (*min as f32) + (param_event.value as f32) * ((max - min) as f32)
                        }
                    };

                    if let Ok(mut params_guard) = plugin.params.write() {
                        voice_param_registry::apply_param(
                            &mut params_guard,
                            param_event.param_id,
                            denorm_value,
                        );
                    }
                }
            }
        }
    }
}

static PLUGIN_PARAMS: clap_plugin_params = clap_plugin_params {
    count: Some(params_count),
    get_info: Some(params_get_info),
    get_value: Some(params_get_value),
    value_to_text: Some(params_value_to_text),
    text_to_value: Some(params_text_to_value),
    flush: Some(params_flush),
};

// ============================================================================
// State Extension (Save/Load)
// ============================================================================

unsafe extern "C" fn state_save(
    plugin: *const clap_plugin,
    stream: *const clap_sys::stream::clap_ostream,
) -> bool {
    // Safety checks
    if plugin.is_null() || stream.is_null() {
        return false;
    }

    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    if plugin_data.is_null() {
        return false;
    }

    let plugin = &*plugin_data;

    if let Ok(params_guard) = plugin.params.read() {
        if let Ok(json) = serde_json::to_string(&*params_guard) {
            let bytes = json.as_bytes();
            let stream = &*stream;

            if stream.write.is_none() {
                return false;
            }

            let write_fn = stream.write.unwrap();
            let written = write_fn(stream, bytes.as_ptr() as *const c_void, bytes.len() as u64);
            return written == bytes.len() as i64;
        }
    }

    false
}

unsafe extern "C" fn state_load(
    plugin: *const clap_plugin,
    stream: *const clap_sys::stream::clap_istream,
) -> bool {
    // Safety checks
    if plugin.is_null() || stream.is_null() {
        return false;
    }

    let plugin_data = (*plugin).plugin_data as *mut VoicePlugin;
    if plugin_data.is_null() {
        return false;
    }

    let plugin = &mut *plugin_data;

    // Read stream into buffer
    let mut buffer = Vec::new();
    let stream = &*stream;

    if stream.read.is_none() {
        return false;
    }

    let read_fn = stream.read.unwrap();

    loop {
        let mut chunk = [0u8; 4096];
        let bytes_read = read_fn(
            stream,
            chunk.as_mut_ptr() as *mut c_void,
            chunk.len() as u64,
        );

        if bytes_read <= 0 {
            break;
        }

        buffer.extend_from_slice(&chunk[..bytes_read as usize]);
    }

    // Deserialize parameters
    if let Ok(json_str) = String::from_utf8(buffer) {
        if let Ok(loaded_params) = serde_json::from_str::<VoiceParams>(&json_str) {
            if let Ok(mut params_guard) = plugin.params.write() {
                *params_guard = loaded_params;

                // Note: Don't update processor here - it will be activated later by the host
                // with the correct sample rate, and will read the updated params then.
                // Trying to activate the processor now can cause issues if:
                // 1. Processor is None (not yet activated)
                // 2. Processor is active but with wrong sample rate
                // 3. Multiple state loads before activation

                return true;
            }
        }
    }

    false
}

static PLUGIN_STATE: clap_plugin_state = clap_plugin_state {
    save: Some(state_save),
    load: Some(state_load),
};

// ============================================================================
// Plugin Descriptor & Factory
// ============================================================================

static PLUGIN_DESCRIPTOR: clap_plugin_descriptor = clap_plugin_descriptor {
    clap_version: CLAP_VERSION,
    id: c"com.dsynth.voice-enhancer".as_ptr(),
    name: c"DSynth Voice".as_ptr(),
    vendor: c"DSynth".as_ptr(),
    url: c"https://github.com/yourusername/dsynth".as_ptr(),
    manual_url: c"https://github.com/yourusername/dsynth".as_ptr(),
    support_url: c"https://github.com/yourusername/dsynth".as_ptr(),
    version: c"0.3.0".as_ptr(),
    description: c"Voice enhancement with pitch-tracked sub oscillator".as_ptr(),
    features: [
        b"audio-effect\0".as_ptr() as *const c_char,
        b"stereo\0".as_ptr() as *const c_char,
        b"vocal\0".as_ptr() as *const c_char,
        ptr::null(),
    ]
    .as_ptr(),
};

unsafe extern "C" fn plugin_factory_get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {
    1
}

unsafe extern "C" fn plugin_factory_get_plugin_descriptor(
    _factory: *const clap_plugin_factory,
    index: u32,
) -> *const clap_plugin_descriptor {
    if index == 0 {
        &PLUGIN_DESCRIPTOR
    } else {
        ptr::null()
    }
}

unsafe extern "C" fn plugin_factory_create_plugin(
    _factory: *const clap_plugin_factory,
    host: *const clap_host,
    plugin_id: *const c_char,
) -> *const clap_plugin {
    let id = CStr::from_ptr(plugin_id);

    if id.to_bytes() != PLUGIN_ID.as_bytes() {
        return ptr::null();
    }

    let mut plugin_box = VoicePlugin::new(host);
    let plugin_ptr = Box::into_raw(plugin_box);

    let clap_plugin = Box::new(clap_plugin {
        desc: &PLUGIN_DESCRIPTOR,
        plugin_data: plugin_ptr as *mut c_void,
        init: Some(plugin_init),
        destroy: Some(plugin_destroy),
        activate: Some(plugin_activate),
        deactivate: Some(plugin_deactivate),
        start_processing: Some(plugin_start_processing),
        stop_processing: Some(plugin_stop_processing),
        reset: Some(plugin_reset),
        process: Some(plugin_process),
        get_extension: Some(plugin_get_extension),
        on_main_thread: Some(plugin_on_main_thread),
    });

    Box::into_raw(clap_plugin)
}

static PLUGIN_FACTORY: clap_plugin_factory = clap_plugin_factory {
    get_plugin_count: Some(plugin_factory_get_plugin_count),
    get_plugin_descriptor: Some(plugin_factory_get_plugin_descriptor),
    create_plugin: Some(plugin_factory_create_plugin),
};

// ============================================================================
// CLAP Entry Point
// ============================================================================

unsafe extern "C" fn entry_init(_plugin_path: *const c_char) -> bool {
    true
}

unsafe extern "C" fn entry_deinit() {
    // Nothing to clean up
}

unsafe extern "C" fn entry_get_factory(factory_id: *const c_char) -> *const c_void {
    let id = CStr::from_ptr(factory_id);

    if id.to_bytes_with_nul() == CLAP_PLUGIN_FACTORY_ID.to_bytes_with_nul() {
        &PLUGIN_FACTORY as *const _ as *const c_void
    } else {
        ptr::null()
    }
}

#[no_mangle]
pub static clap_entry: clap_plugin_entry = clap_plugin_entry {
    clap_version: CLAP_VERSION,
    init: Some(entry_init),
    deinit: Some(entry_deinit),
    get_factory: Some(entry_get_factory),
};
