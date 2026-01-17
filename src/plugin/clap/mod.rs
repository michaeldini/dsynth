/// CLAP Plugin Module
///
/// Entry point and exports for the CLAP plugin.

#[cfg(feature = "clap")]
pub mod descriptor;
#[cfg(feature = "clap")]
pub mod params;
#[cfg(feature = "clap")]
pub mod plugin;
#[cfg(feature = "clap")]
pub mod processor;
#[cfg(feature = "clap")]
pub mod state;

#[cfg(feature = "kick-clap")]
pub mod kick_plugin;
#[cfg(feature = "kick-clap")]
pub mod kick_processor;

#[cfg(feature = "voice-clap")]
pub mod voice_plugin;
#[cfg(feature = "voice-clap")]
pub mod voice_processor;

#[cfg(feature = "clap")]
pub use plugin::DSynthClapPlugin;

#[cfg(feature = "kick-clap")]
pub use kick_plugin::KickClapPlugin;

// Main CLAP plugin entry point (polyphonic synth)
#[cfg(feature = "clap")]
use clap_sys::entry::clap_plugin_entry;
#[cfg(feature = "clap")]
use clap_sys::host::clap_host;
#[cfg(feature = "clap")]
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
#[cfg(feature = "clap")]
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
#[cfg(feature = "clap")]
use clap_sys::version::CLAP_VERSION;
#[cfg(feature = "clap")]
use std::ffi::CStr;
#[cfg(feature = "clap")]
use std::os::raw::c_void;

#[cfg(feature = "clap")]
pub(crate) fn log_to_file(msg: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;

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
        let _ = file.sync_all();
    }
}

#[cfg(feature = "clap")]
unsafe extern "C" fn plugin_factory_get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {
    1 // We have one plugin: DSynth
}

#[cfg(feature = "clap")]
unsafe extern "C" fn plugin_factory_get_plugin_descriptor(
    _factory: *const clap_plugin_factory,
    index: u32,
) -> *const clap_plugin_descriptor {
    if index == 0 {
        &descriptor::DESCRIPTOR
    } else {
        std::ptr::null()
    }
}

#[cfg(feature = "clap")]
unsafe extern "C" fn plugin_factory_create_plugin(
    _factory: *const clap_plugin_factory,
    host: *const clap_host,
    plugin_id: *const i8,
) -> *const clap_plugin {
    log_to_file("plugin_factory_create_plugin() called");
    if host.is_null() || plugin_id.is_null() {
        log_to_file("ERROR: host or plugin_id is null");
        return std::ptr::null();
    }

    let id = unsafe { CStr::from_ptr(plugin_id) };
    log_to_file(&format!("Requested plugin ID: {:?}", id));
    if id != descriptor::PLUGIN_ID {
        log_to_file(&format!(
            "ERROR: ID mismatch, expected {:?}",
            descriptor::PLUGIN_ID
        ));
        return std::ptr::null();
    }

    log_to_file("Creating DSynthClapPlugin instance...");
    let plugin = DSynthClapPlugin::new(host);
    let plugin_ptr = &plugin.plugin as *const clap_plugin;
    Box::leak(plugin); // Keep alive until destroy is called
    log_to_file(&format!("Plugin instance created at {:p}", plugin_ptr));
    plugin_ptr
}

#[cfg(feature = "clap")]
static PLUGIN_FACTORY: clap_plugin_factory = clap_plugin_factory {
    get_plugin_count: Some(plugin_factory_get_plugin_count),
    get_plugin_descriptor: Some(plugin_factory_get_plugin_descriptor),
    create_plugin: Some(plugin_factory_create_plugin),
};

#[cfg(feature = "clap")]
unsafe extern "C" fn entry_init(_plugin_path: *const i8) -> bool {
    log_to_file("entry_init() called");
    true
}

#[cfg(feature = "clap")]
unsafe extern "C" fn entry_deinit() {
    // Cleanup if needed
}

#[cfg(feature = "clap")]
unsafe extern "C" fn entry_get_factory(factory_id: *const i8) -> *const c_void {
    log_to_file("entry_get_factory() called");
    if factory_id.is_null() {
        return std::ptr::null();
    }

    let id = unsafe { CStr::from_ptr(factory_id) };
    log_to_file(&format!("Requested factory: {:?}", id));
    if id.to_bytes_with_nul() == CLAP_PLUGIN_FACTORY_ID.to_bytes_with_nul() {
        log_to_file("Returning plugin factory");
        &PLUGIN_FACTORY as *const _ as *const c_void
    } else {
        log_to_file("Factory not found");
        std::ptr::null()
    }
}

// CLAP requires a static clap_entry symbol with C linkage.
// We create it by returning a pointer to a static struct.
#[cfg(feature = "clap")]
use std::sync::OnceLock;

#[cfg(feature = "clap")]
static ENTRY: OnceLock<clap_plugin_entry> = OnceLock::new();

#[cfg(feature = "clap")]
#[unsafe(no_mangle)]
pub extern "C" fn get_clap_entry() -> *const clap_plugin_entry {
    ENTRY.get_or_init(|| clap_plugin_entry {
        clap_version: CLAP_VERSION,
        init: Some(entry_init),
        deinit: Some(entry_deinit),
        get_factory: Some(entry_get_factory),
    })
}

// The actual C-linkage symbol that CLAP hosts look for
#[cfg(feature = "clap")]
#[unsafe(no_mangle)]
#[unsafe(link_section = "__DATA,__data")]
pub static clap_entry: clap_plugin_entry = clap_plugin_entry {
    clap_version: CLAP_VERSION,
    init: Some(entry_init),
    deinit: Some(entry_deinit),
    get_factory: Some(entry_get_factory),
};
