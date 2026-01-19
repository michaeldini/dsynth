//! CLAP parameters extension implementation

use crate::{instance::PluginInstance, param::PluginParams, plugin::ClapPlugin};
use clap_sys::events::*;
use clap_sys::ext::params::*;
use clap_sys::string_sizes::{CLAP_NAME_SIZE, CLAP_PATH_SIZE};
use std::ffi::CStr;
use std::sync::OnceLock;

/// Get the parameters extension for a plugin type
pub fn get_extension<P: ClapPlugin>() -> &'static clap_plugin_params {
    static EXT: OnceLock<clap_plugin_params> = OnceLock::new();
    EXT.get_or_init(|| clap_plugin_params {
        count: Some(params_count::<P>),
        get_info: Some(params_get_info::<P>),
        get_value: Some(params_get_value::<P>),
        value_to_text: Some(params_value_to_text::<P>),
        text_to_value: Some(params_text_to_value::<P>),
        flush: Some(params_flush::<P>),
    })
}

unsafe extern "C" fn params_count<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
) -> u32 {
    let _instance = PluginInstance::<P>::from_ptr(plugin);
    P::Params::param_count()
}

unsafe extern "C" fn params_get_info<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    param_index: u32,
    param_info: *mut clap_param_info,
) -> bool {
    if param_info.is_null() {
        return false;
    }

    let _instance = PluginInstance::<P>::from_ptr(plugin);

    if let Some(descriptor) = P::Params::param_descriptor(param_index) {
        let info = &mut *param_info;

        // Set ID
        info.id = descriptor.id;

        // Set flags
        info.flags = 0;
        if descriptor.is_automatable {
            info.flags |= CLAP_PARAM_IS_AUTOMATABLE;
        }
        if descriptor.is_hidden {
            info.flags |= CLAP_PARAM_IS_HIDDEN;
        }
        if descriptor.is_stepped() {
            info.flags |= CLAP_PARAM_IS_STEPPED;
        }
        // clap-sys 0.3.0 does not expose boolean/enum flags, so we only
        // use CLAP_PARAM_IS_STEPPED to indicate discrete params.

        // Copy name
        let name_bytes = descriptor.name.as_bytes();
        let copy_len = name_bytes.len().min(CLAP_NAME_SIZE - 1);
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            info.name.as_mut_ptr() as *mut u8,
            copy_len,
        );
        info.name[copy_len] = 0;

        // Copy module
        let module_bytes = descriptor.module.as_bytes();
        let copy_len = module_bytes.len().min(CLAP_PATH_SIZE - 1);
        std::ptr::copy_nonoverlapping(
            module_bytes.as_ptr(),
            info.module.as_mut_ptr() as *mut u8,
            copy_len,
        );
        info.module[copy_len] = 0;

        // Set min/max/default (normalized to 0-1)
        info.min_value = 0.0;
        info.max_value = 1.0;
        info.default_value = descriptor.normalize(descriptor.default_value()) as f64;

        true
    } else {
        false
    }
}

unsafe extern "C" fn params_get_value<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    param_id: u32,
    out_value: *mut f64,
) -> bool {
    if out_value.is_null() {
        return false;
    }

    let _instance = PluginInstance::<P>::from_ptr(plugin);

    if let Some(value) = P::Params::get_param(param_id) {
        // Plugin params are already normalized (0.0-1.0)
        *out_value = value.clamp(0.0, 1.0) as f64;
        return true;
    }

    false
}

unsafe extern "C" fn params_value_to_text<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    param_id: u32,
    value: f64,
    out_buffer: *mut i8,
    out_buffer_capacity: u32,
) -> bool {
    if out_buffer.is_null() || out_buffer_capacity == 0 {
        return false;
    }

    let _instance = PluginInstance::<P>::from_ptr(plugin);

    if P::Params::param_descriptor_by_id(param_id).is_some() {
        // `value` is normalized (0.0-1.0)
        let text = P::Params::format_param(param_id, value as f32);

        let text_bytes = text.as_bytes();
        let copy_len = text_bytes.len().min(out_buffer_capacity as usize - 1);
        std::ptr::copy_nonoverlapping(text_bytes.as_ptr(), out_buffer as *mut u8, copy_len);
        *(out_buffer.add(copy_len)) = 0;

        return true;
    }

    false
}

unsafe extern "C" fn params_text_to_value<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    param_id: u32,
    param_value_text: *const i8,
    out_value: *mut f64,
) -> bool {
    if param_value_text.is_null() || out_value.is_null() {
        return false;
    }

    let _instance = PluginInstance::<P>::from_ptr(plugin);

    if let Ok(text) = CStr::from_ptr(param_value_text).to_str() {
        if let Some(normalized_value) = P::Params::parse_param(param_id, text.trim()) {
            // `parse_param` returns normalized (0.0-1.0)
            *out_value = normalized_value.clamp(0.0, 1.0) as f64;
            return true;
        }
    }

    false
}

unsafe extern "C" fn params_flush<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    in_events: *const clap_input_events,
    _out_events: *const clap_output_events,
) {
    if in_events.is_null() {
        return;
    }

    let _instance = PluginInstance::<P>::from_ptr_mut(plugin);
    let events = &*in_events;

    if let Some(size_fn) = events.size {
        let event_count = size_fn(events);

        for i in 0..event_count {
            if let Some(get_fn) = events.get {
                let event = get_fn(events, i);
                if !event.is_null() {
                    let header = &*event;

                    // Handle parameter value events
                    if header.space_id == CLAP_CORE_EVENT_SPACE_ID
                        && header.type_ == CLAP_EVENT_PARAM_VALUE
                    {
                        let param_event = event as *const clap_event_param_value;
                        let param_event = &*param_event;

                        // Apply parameter change
                        if P::Params::param_descriptor_by_id(param_event.param_id).is_some() {
                            // `param_event.value` is normalized (0.0-1.0)
                            P::Params::set_param(
                                param_event.param_id,
                                (param_event.value as f32).clamp(0.0, 1.0),
                            );
                        }
                    }
                }
            }
        }
    }
}
