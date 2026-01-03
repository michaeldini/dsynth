use super::super::state::PluginState;
/// CLAP State Extension
///
/// Handles plugin state save/load using the PluginState serialization system.
use clap_sys::ext::state::clap_plugin_state;
use clap_sys::stream::{clap_istream, clap_ostream};

fn log_to_file(msg: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/dsynth_clap.log")
    {
        let _ = writeln!(
            file,
            "[{}] STATE: {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            msg
        );
        let _ = file.sync_all();
    }
}

/// Save plugin state to stream
///
/// # Safety
///
/// This function is an FFI boundary called by the CLAP host. Safety requirements:
/// - `plugin` must be a valid pointer to a DSynthClapPlugin instance
/// - `stream` must be a valid pointer to a clap_ostream with a valid `write` function pointer
/// - Must only be called on the main thread (CLAP requirement for state extension)
/// - The plugin instance must not be destroyed while this function executes
/// - The stream must remain valid for the duration of this call
pub unsafe extern "C" fn state_save(
    plugin: *const clap_sys::plugin::clap_plugin,
    stream: *const clap_ostream,
) -> bool {
    if plugin.is_null() || stream.is_null() {
        return false;
    }

    // Get current parameters from the plugin instance
    let instance = unsafe { super::plugin::DSynthClapPlugin::from_ptr(plugin) };

    // Sync current_params from processor (the processor has the most up-to-date values)
    let params = if let Some(processor) = &instance.processor {
        processor.current_params
    } else {
        instance.current_params
    };

    // Debug logging
    log_to_file(&format!(
        "state_save called - osc1_gain: {}",
        params.oscillators[0].gain
    ));

    let state = PluginState::from_params(params, None);

    match state.to_bytes() {
        Ok(bytes) => {
            log_to_file(&format!("state_save successful - {} bytes", bytes.len()));
            unsafe { write_to_stream(stream, &bytes) }
        }
        Err(e) => {
            log_to_file(&format!("state_save failed: {:?}", e));
            false
        }
    }
}

/// Load plugin state from stream
///
/// # Safety
///
/// This function is an FFI boundary called by the CLAP host. Safety requirements:
/// - `plugin` must be a valid pointer to a DSynthClapPlugin instance
/// - `stream` must be a valid pointer to a clap_istream with a valid `read` function pointer
/// - Must only be called on the main thread (CLAP requirement for state extension)
/// - The plugin instance must not be destroyed while this function executes
/// - The stream must remain valid for the duration of this call
/// - The caller must ensure the stream contains valid PluginState data
pub unsafe extern "C" fn state_load(
    plugin: *const clap_sys::plugin::clap_plugin,
    stream: *const clap_istream,
) -> bool {
    if plugin.is_null() || stream.is_null() {
        return false;
    }

    log_to_file("state_load called");

    match unsafe { read_from_stream(stream) } {
        Ok(bytes) => {
            log_to_file(&format!("state_load read {} bytes", bytes.len()));
            match PluginState::from_bytes(&bytes) {
                Ok(state) => {
                    let params = state.params().clone();
                    log_to_file(&format!(
                        "state_load params - osc1_gain: {}",
                        params.oscillators[0].gain
                    ));

                    // Apply state to plugin instance
                    let instance = unsafe { super::plugin::DSynthClapPlugin::from_ptr(plugin) };
                    instance.current_params = params;

                    // Update shared GUI state
                    let mut gui_params = instance.synth_params.write();
                    *gui_params = params;

                    // Apply to processor if active
                    if let Some(processor) = &mut instance.processor {
                        processor.current_params = params;
                        processor.param_producer.write(params);
                    }

                    log_to_file("state_load successful");
                    true
                }
                Err(e) => {
                    log_to_file(&format!("state_load deserialize failed: {:?}", e));
                    false
                }
            }
        }
        Err(e) => {
            log_to_file(&format!("state_load read failed: {:?}", e));
            false
        }
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
