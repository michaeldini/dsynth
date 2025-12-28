use super::super::{
    param_descriptor::{ParamDescriptor, ParamId, ParamType},
    param_registry,
};
/// CLAP Parameters Extension
///
/// Exposes DSynth parameters to CLAP hosts using the param_registry system.
use clap_sys::ext::params::{CLAP_PARAM_IS_AUTOMATABLE, clap_param_info};

/// Get parameter count
pub unsafe extern "C" fn params_count(_plugin: *const clap_sys::plugin::clap_plugin) -> u32 {
    let registry = param_registry::get_registry();
    registry.count() as u32
}

/// Get parameter info by index
pub unsafe extern "C" fn params_get_info(
    _plugin: *const clap_sys::plugin::clap_plugin,
    param_index: u32,
    param_info: *mut clap_param_info,
) -> bool {
    if param_info.is_null() {
        return false;
    }

    let registry = param_registry::get_registry();
    let Some(param_id) = registry.get_id_by_index(param_index as usize) else {
        return false;
    };

    let Some(descriptor) = registry.get(param_id) else {
        return false;
    };

    fill_param_info(param_info, param_id, descriptor);
    true
}

/// Get parameter value (normalized 0-1)
pub unsafe extern "C" fn params_get_value(
    plugin: *const clap_sys::plugin::clap_plugin,
    param_id: ParamId,
    out_value: *mut f64,
) -> bool {
    if out_value.is_null() || plugin.is_null() {
        return false;
    }

    use super::super::param_update::param_get;
    use super::plugin::DSynthClapPlugin;

    let instance = unsafe { &*((*plugin).plugin_data as *const DSynthClapPlugin) };

    // Read from processor's current_params if available, otherwise use plugin's
    let params = if let Some(processor) = &instance.processor {
        &processor.current_params
    } else {
        &instance.current_params
    };

    let value = param_get::get_param(params, param_id);

    let registry = param_registry::get_registry();
    if let Some(descriptor) = registry.get(param_id) {
        unsafe {
            *out_value = normalize_param_value(descriptor, value) as f64;
        }
        true
    } else {
        false
    }
}

/// Convert parameter value to text (for display)
pub unsafe extern "C" fn params_value_to_text(
    _plugin: *const clap_sys::plugin::clap_plugin,
    param_id: ParamId,
    value: f64,
    out_buffer: *mut i8,
    out_buffer_capacity: u32,
) -> bool {
    if out_buffer.is_null() || out_buffer_capacity == 0 {
        return false;
    }

    let registry = param_registry::get_registry();
    let Some(descriptor) = registry.get(param_id) else {
        return false;
    };

    let denormalized = descriptor.denormalize(value as f32);
    let text = format_param_value(descriptor, denormalized);

    copy_string_to_buffer(&text, out_buffer, out_buffer_capacity as usize)
}

/// Convert text to parameter value (for text input)
pub unsafe extern "C" fn params_text_to_value(
    _plugin: *const clap_sys::plugin::clap_plugin,
    param_id: ParamId,
    param_value_text: *const i8,
    out_value: *mut f64,
) -> bool {
    if param_value_text.is_null() || out_value.is_null() {
        return false;
    }

    let text = match unsafe { std::ffi::CStr::from_ptr(param_value_text).to_str() } {
        Ok(t) => t,
        Err(_) => return false,
    };

    let registry = param_registry::get_registry();
    let Some(descriptor) = registry.get(param_id) else {
        return false;
    };

    if let Ok(value) = text.parse::<f32>() {
        unsafe {
            *out_value = normalize_param_value(descriptor, value) as f64;
        }
        true
    } else {
        false
    }
}

/// Flush parameter changes (called before/after processing)
pub unsafe extern "C" fn params_flush(
    plugin: *const clap_sys::plugin::clap_plugin,
    in_events: *const clap_sys::events::clap_input_events,
    _out_events: *const clap_sys::events::clap_output_events,
) {
    if plugin.is_null() || in_events.is_null() {
        return;
    }

    use super::super::param_update::param_apply;
    use super::plugin::DSynthClapPlugin;

    let instance = unsafe { &mut *((*plugin).plugin_data as *mut DSynthClapPlugin) };

    // Process parameter change events
    let events = unsafe { &*in_events };
    let size_fn = events.size.expect("in_events.size is null");
    let get_fn = events.get.expect("in_events.get is null");

    let event_count = unsafe { size_fn(events) };

    for i in 0..event_count {
        let event = unsafe { get_fn(events, i) };
        if event.is_null() {
            continue;
        }

        let header = unsafe { &*(event as *const clap_sys::events::clap_event_header) };

        if header.type_ == clap_sys::events::CLAP_EVENT_PARAM_VALUE as u16 {
            let param_event =
                unsafe { &*(event as *const clap_sys::events::clap_event_param_value) };
            let param_id = param_event.param_id as ParamId;
            let normalized = param_event.value as f32;

            // Update plugin's stored params
            param_apply::apply_param(&mut instance.current_params, param_id, normalized);

            // Update shared GUI state
            if let Ok(mut gui_params) = instance.synth_params.write() {
                param_apply::apply_param(&mut gui_params, param_id, normalized);
            }

            // Also update processor if it exists
            if let Some(processor) = &mut instance.processor {
                param_apply::apply_param(&mut processor.current_params, param_id, normalized);
                processor.param_producer.write(processor.current_params);
            }
        }
    }
}

// Helper functions

fn fill_param_info(info: *mut clap_param_info, param_id: ParamId, descriptor: &ParamDescriptor) {
    unsafe {
        (*info).id = param_id;
        (*info).flags = CLAP_PARAM_IS_AUTOMATABLE;

        // Copy parameter name
        let name_bytes = &descriptor.name.as_bytes();
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
        let module_bytes = &descriptor.module.as_bytes();
        let module_copy_len = module_bytes
            .len()
            .min(clap_sys::string_sizes::CLAP_PATH_SIZE - 1);
        std::ptr::copy_nonoverlapping(
            module_bytes.as_ptr(),
            (*info).module.as_mut_ptr() as *mut u8,
            module_copy_len,
        );
        (*info).module[module_copy_len] = 0;

        // Set min/max/default values
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
            ParamType::Enum { .. } => {
                // Enums use normalized range 0.0-1.0 (CLAP standard)
                (*info).min_value = 0.0;
                (*info).max_value = 1.0;
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

fn normalize_param_value(descriptor: &ParamDescriptor, value: f32) -> f32 {
    match &descriptor.param_type {
        ParamType::Float { min, max, .. } => (value - min) / (max - min),
        ParamType::Bool => {
            if value > 0.5 {
                1.0
            } else {
                0.0
            }
        }
        ParamType::Enum { variants } => {
            // value is denormalized enum index (0.0, 1.0, 2.0, ...)
            // normalize to 0.0-1.0 range
            if variants.len() > 1 {
                value / (variants.len() - 1) as f32
            } else {
                0.0
            }
        }
        ParamType::Int { min, max } => (value - *min as f32) / (*max - *min) as f32,
    }
}

fn format_param_value(descriptor: &ParamDescriptor, value: f32) -> String {
    if let Some(unit) = descriptor.unit.as_ref() {
        format!("{:.2} {}", value, unit)
    } else {
        format!("{:.2}", value)
    }
}

fn copy_string_to_buffer(text: &str, buffer: *mut i8, capacity: usize) -> bool {
    unsafe {
        let bytes = text.as_bytes();
        let copy_len = bytes.len().min(capacity - 1);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, copy_len);
        *(buffer.offset(copy_len as isize)) = 0;
        true
    }
}

/// Create the params extension vtable
pub fn create_params_ext() -> clap_sys::ext::params::clap_plugin_params {
    clap_sys::ext::params::clap_plugin_params {
        count: Some(params_count),
        get_info: Some(params_get_info),
        get_value: Some(params_get_value),
        value_to_text: Some(params_value_to_text),
        text_to_value: Some(params_text_to_value),
        flush: Some(params_flush),
    }
}
