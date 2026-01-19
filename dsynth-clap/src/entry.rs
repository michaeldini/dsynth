//! CLAP entry point generation

use std::fs::OpenOptions;
use std::io::Write;

pub fn log_entry(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/clap_entry.log")
    {
        let _ = writeln!(file, "[CLAP] {}", msg);
    }
}

/// Generate CLAP entry point for a plugin
#[macro_export]
macro_rules! generate_clap_entry {
    ($plugin_type:ty) => {
        mod __clap_entry_impl {
            use super::*;

            pub unsafe extern "C" fn init(_plugin_path: *const std::os::raw::c_char) -> bool {
                $crate::entry::log_entry("init() called");
                true
            }

            pub unsafe extern "C" fn deinit() {
                $crate::entry::log_entry("deinit() called");
                // No cleanup needed
            }

            pub unsafe extern "C" fn get_factory(
                factory_id: *const std::os::raw::c_char,
            ) -> *const std::os::raw::c_void {
                use std::ffi::CStr;

                $crate::entry::log_entry("get_factory() called");

                if factory_id.is_null() {
                    $crate::entry::log_entry("get_factory: factory_id is null");
                    return std::ptr::null();
                }

                let factory_id_str = match CStr::from_ptr(factory_id).to_str() {
                    Ok(s) => {
                        $crate::entry::log_entry(&format!(
                            "get_factory: requested factory '{}'",
                            s
                        ));
                        s
                    }
                    Err(_) => {
                        $crate::entry::log_entry("get_factory: failed to parse factory_id");
                        return std::ptr::null();
                    }
                };

                let plugin_factory_id = match CStr::from_ptr(
                    $crate::clap_sys::plugin_factory::CLAP_PLUGIN_FACTORY_ID.as_ptr(),
                )
                .to_str()
                {
                    Ok(s) => s,
                    Err(_) => return std::ptr::null(),
                };

                if factory_id_str == plugin_factory_id {
                    $crate::entry::log_entry("get_factory: returning plugin factory");
                    &FACTORY as *const _ as *const std::os::raw::c_void
                } else {
                    $crate::entry::log_entry(&format!(
                        "get_factory: unknown factory '{}', returning null",
                        factory_id_str
                    ));
                    std::ptr::null()
                }
            }

            pub unsafe extern "C" fn factory_get_plugin_count(
                _factory: *const $crate::clap_sys::plugin_factory::clap_plugin_factory,
            ) -> u32 {
                1
            }

            pub unsafe extern "C" fn factory_get_plugin_descriptor(
                _factory: *const $crate::clap_sys::plugin_factory::clap_plugin_factory,
                index: u32,
            ) -> *const $crate::clap_sys::plugin::clap_plugin_descriptor {
                if index == 0 {
                    <$plugin_type>::clap_descriptor()
                } else {
                    std::ptr::null()
                }
            }

            pub unsafe extern "C" fn factory_create_plugin(
                _factory: *const $crate::clap_sys::plugin_factory::clap_plugin_factory,
                host: *const $crate::clap_sys::host::clap_host,
                plugin_id: *const std::os::raw::c_char,
            ) -> *const $crate::clap_sys::plugin::clap_plugin {
                use std::ffi::CStr;

                $crate::entry::log_entry("factory_create_plugin() called");

                if host.is_null() || plugin_id.is_null() {
                    $crate::entry::log_entry("factory_create_plugin: host or plugin_id is null");
                    return std::ptr::null();
                }

                let requested_id = match CStr::from_ptr(plugin_id).to_str() {
                    Ok(s) => {
                        $crate::entry::log_entry(&format!(
                            "factory_create_plugin: requested plugin '{}'",
                            s
                        ));
                        s
                    }
                    Err(_) => {
                        $crate::entry::log_entry(
                            "factory_create_plugin: failed to parse plugin_id",
                        );
                        return std::ptr::null();
                    }
                };

                let descriptor = <$plugin_type>::clap_descriptor();
                let expected_id = match CStr::from_ptr((*descriptor).id).to_str() {
                    Ok(s) => s,
                    Err(_) => return std::ptr::null(),
                };

                if requested_id != expected_id {
                    $crate::entry::log_entry(&format!(
                        "factory_create_plugin: ID mismatch - requested '{}', expected '{}'",
                        requested_id, expected_id
                    ));
                    return std::ptr::null();
                }

                $crate::entry::log_entry("factory_create_plugin: creating plugin instance");
                let instance = $crate::instance::PluginInstance::<$plugin_type>::new(host);
                let instance_ptr = Box::into_raw(instance);
                let clap_plugin = Box::new($crate::instance::create_clap_plugin::<$plugin_type>(
                    instance_ptr,
                ));
                let plugin_ptr = Box::into_raw(clap_plugin);
                $crate::entry::log_entry(&format!(
                    "factory_create_plugin: returning plugin at {:p}",
                    plugin_ptr
                ));
                plugin_ptr
            }

            pub static FACTORY: $crate::clap_sys::plugin_factory::clap_plugin_factory =
                $crate::clap_sys::plugin_factory::clap_plugin_factory {
                    get_plugin_count: Some(factory_get_plugin_count),
                    get_plugin_descriptor: Some(factory_get_plugin_descriptor),
                    create_plugin: Some(factory_create_plugin),
                };

            pub static ENTRY: $crate::clap_sys::entry::clap_plugin_entry =
                $crate::clap_sys::entry::clap_plugin_entry {
                    clap_version: $crate::clap_sys::version::CLAP_VERSION,
                    init: Some(init),
                    deinit: Some(deinit),
                    get_factory: Some(get_factory),
                };
        }

        // CLAP requires a static symbol named `clap_entry`.
        // Export it with C linkage in a data section so hosts can find it.
        #[no_mangle]
        #[link_section = "__DATA,__data"]
        pub static clap_entry: $crate::clap_sys::entry::clap_plugin_entry =
            $crate::clap_sys::entry::clap_plugin_entry {
                clap_version: $crate::clap_sys::version::CLAP_VERSION,
                init: Some(__clap_entry_impl::init),
                deinit: Some(__clap_entry_impl::deinit),
                get_factory: Some(__clap_entry_impl::get_factory),
            };
    };
}
