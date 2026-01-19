//! CLAP state extension implementation

use crate::{instance::PluginInstance, param::PluginParams, plugin::ClapPlugin};
use clap_sys::ext::state::*;
use clap_sys::stream::*;
use std::sync::OnceLock;

/// Get the state extension for a plugin type
pub fn get_extension<P: ClapPlugin>() -> &'static clap_plugin_state {
    static EXT: OnceLock<clap_plugin_state> = OnceLock::new();
    EXT.get_or_init(|| clap_plugin_state {
        save: Some(state_save::<P>),
        load: Some(state_load::<P>),
    })
}

unsafe extern "C" fn state_save<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    stream: *const clap_ostream,
) -> bool {
    if stream.is_null() {
        return false;
    }

    let _instance = PluginInstance::<P>::from_ptr(plugin);

    // Get current parameter state
    let state = P::Params::save_state();

    // Serialize to JSON
    match state.to_bytes() {
        Ok(bytes) => {
            // Write to stream
            let stream = &*stream;
            if let Some(write_fn) = stream.write {
                let written = write_fn(
                    stream,
                    bytes.as_ptr() as *const std::os::raw::c_void,
                    bytes.len() as u64,
                );
                written == bytes.len() as i64
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

unsafe extern "C" fn state_load<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    stream: *const clap_istream,
) -> bool {
    if stream.is_null() {
        return false;
    }

    let _instance = PluginInstance::<P>::from_ptr_mut(plugin);
    let stream = &*stream;

    // Read all data from stream
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];

    if let Some(read_fn) = stream.read {
        loop {
            let n_read = read_fn(stream, buffer.as_mut_ptr() as *mut _, buffer.len() as u64);
            if n_read <= 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..n_read as usize]);
        }
    } else {
        return false;
    }

    // Deserialize and apply state
    match crate::PluginState::from_bytes(&bytes) {
        Ok(state) => {
            P::Params::load_state(&state);
            true
        }
        Err(_) => false,
    }
}
