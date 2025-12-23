use super::super::state::PluginState;
use crate::params::SynthParams;
/// CLAP State Extension
///
/// Handles plugin state save/load using the PluginState serialization system.
use clap_sys::ext::state::clap_plugin_state;
use clap_sys::stream::{clap_istream, clap_ostream};

/// Save plugin state to stream
pub unsafe extern "C" fn state_save(
    plugin: *const clap_sys::plugin::clap_plugin,
    stream: *const clap_ostream,
) -> bool {
    if plugin.is_null() || stream.is_null() {
        return false;
    }

    // Get current parameters from the plugin instance
    // In full implementation, we'd extract this from the plugin's processor
    let params = SynthParams::default(); // Placeholder
    let state = PluginState::from_params(params, None);

    match state.to_bytes() {
        Ok(bytes) => unsafe { write_to_stream(stream, &bytes) },
        Err(_) => false,
    }
}

/// Load plugin state from stream
pub unsafe extern "C" fn state_load(
    plugin: *const clap_sys::plugin::clap_plugin,
    stream: *const clap_istream,
) -> bool {
    if plugin.is_null() || stream.is_null() {
        return false;
    }

    match unsafe { read_from_stream(stream) } {
        Ok(bytes) => {
            match PluginState::from_bytes(&bytes) {
                Ok(_state) => {
                    // In full implementation, apply state to processor
                    true
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

// Helper functions for stream I/O

unsafe fn write_to_stream(stream: *const clap_ostream, data: &[u8]) -> bool {
    let stream = unsafe { &*stream };
    let write_fn = stream.write.expect("ostream.write is null");

    let mut offset = 0;
    while offset < data.len() {
        let chunk = &data[offset..];
        let written = unsafe { write_fn(stream, chunk.as_ptr() as *const _, chunk.len() as u64) };

        if written < 0 {
            return false;
        }

        offset += written as usize;
    }

    true
}

unsafe fn read_from_stream(stream: *const clap_istream) -> Result<Vec<u8>, std::io::Error> {
    let stream = unsafe { &*stream };
    let read_fn = stream.read.expect("istream.read is null");

    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 4096];

    loop {
        let bytes_read =
            unsafe { read_fn(stream, chunk.as_mut_ptr() as *mut _, chunk.len() as u64) };

        if bytes_read < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream read error",
            ));
        }

        if bytes_read == 0 {
            break;
        }

        buffer.extend_from_slice(&chunk[..bytes_read as usize]);
    }

    Ok(buffer)
}

/// Create the state extension vtable
pub fn create_state_ext() -> clap_plugin_state {
    clap_plugin_state {
        save: Some(state_save),
        load: Some(state_load),
    }
}
