//! CLAP audio ports extension implementation

use crate::{plugin::ClapPlugin, PortConfig};
use clap_sys::ext::audio_ports::*;
use clap_sys::id::CLAP_INVALID_ID;
use clap_sys::string_sizes::CLAP_NAME_SIZE;
use std::ffi::CStr;
use std::sync::OnceLock;

static STEREO_PORT_TYPE: &CStr = c"stereo";

/// Get the audio ports extension for a plugin type
pub fn get_extension<P: ClapPlugin>() -> &'static clap_plugin_audio_ports {
    static EXT: OnceLock<clap_plugin_audio_ports> = OnceLock::new();
    EXT.get_or_init(|| clap_plugin_audio_ports {
        count: Some(audio_ports_count::<P>),
        get: Some(audio_ports_get::<P>),
    })
}

unsafe extern "C" fn audio_ports_count<P: ClapPlugin>(
    _plugin: *const clap_sys::plugin::clap_plugin,
    is_input: bool,
) -> u32 {
    let descriptor = P::descriptor();
    match descriptor.audio_ports {
        PortConfig::Instrument => {
            // Instrument: 0 inputs, 1 stereo output
            if is_input {
                0
            } else {
                1
            }
        }
        PortConfig::Effect => {
            // Effect: 1 stereo input, 1 stereo output
            1
        }
        PortConfig::Custom { inputs, outputs } => {
            if is_input {
                inputs
            } else {
                outputs
            }
        }
    }
}

unsafe extern "C" fn audio_ports_get<P: ClapPlugin>(
    _plugin: *const clap_sys::plugin::clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool {
    if info.is_null() {
        return false;
    }

    let descriptor = P::descriptor();
    let count = match descriptor.audio_ports {
        PortConfig::Instrument => {
            if is_input {
                0
            } else {
                1
            }
        }
        PortConfig::Effect => 1,
        PortConfig::Custom { inputs, outputs } => {
            if is_input {
                inputs
            } else {
                outputs
            }
        }
    };

    if index >= count {
        return false;
    }

    let info = &mut *info;

    // Set port ID
    info.id = if is_input {
        index
    } else {
        0x1000 + index // Offset output IDs
    };

    // Set port name
    let name = if is_input {
        format!("Input {}\0", index + 1)
    } else {
        format!("Output {}\0", index + 1)
    };
    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(CLAP_NAME_SIZE - 1);
    std::ptr::copy_nonoverlapping(
        name_bytes.as_ptr(),
        info.name.as_mut_ptr() as *mut u8,
        copy_len,
    );
    info.name[copy_len] = 0;

    // Set stereo configuration
    info.channel_count = 2;
    info.flags = CLAP_AUDIO_PORT_IS_MAIN;

    // Set port type to stereo (pointer to static C string)
    info.port_type = STEREO_PORT_TYPE.as_ptr();

    // In-place processing support
    // For effects: output port pairs with corresponding input port
    // For instruments: no input ports, so no in-place processing
    match descriptor.audio_ports {
        PortConfig::Effect => {
            // Effect: output port 0 pairs with input port 0
            if is_input {
                // Input ports: no in-place pair (they reference outputs, not vice versa)
                info.in_place_pair = CLAP_INVALID_ID;
            } else {
                // Output port pairs with corresponding input port (same index)
                info.in_place_pair = index;
            }
        }
        PortConfig::Instrument => {
            // Instrument: no input ports, so no in-place processing
            info.in_place_pair = CLAP_INVALID_ID;
        }
        PortConfig::Custom { inputs, .. } => {
            // Custom: support in-place if both input and output exist at same index
            if !is_input && index < inputs {
                info.in_place_pair = index;
            } else {
                info.in_place_pair = CLAP_INVALID_ID;
            }
        }
    }

    true
}
