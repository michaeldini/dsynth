/// CLAP Plugin Descriptor
/// 
/// Defines the plugin metadata and capabilities exposed to CLAP hosts.

use clap_sys::plugin::clap_plugin_descriptor;
use std::ffi::CStr;

/// Plugin ID - unique identifier for the CLAP plugin
pub const PLUGIN_ID: &CStr = c"com.dsynth.dsynth";

/// Plugin name displayed in DAWs
pub const PLUGIN_NAME: &CStr = c"DSynth";

/// Plugin vendor/author
pub const PLUGIN_VENDOR: &CStr = c"DSynth";

/// Plugin version
pub const PLUGIN_VERSION: &CStr = c"0.3.0";

/// Plugin URL
pub const PLUGIN_URL: &CStr = c"https://github.com/yourusername/dsynth";

/// Plugin description
pub const PLUGIN_DESCRIPTION: &CStr = c"Polyphonic wavetable synthesizer with 3 oscillators, filters, LFOs, and effects";

/// Plugin features - tells DAWs what category this plugin belongs to
pub const PLUGIN_FEATURES: &[*const i8] = &[
    c"instrument".as_ptr(),
    c"synthesizer".as_ptr(),
    c"stereo".as_ptr(),
    std::ptr::null(),
];

/// CLAP plugin descriptor - static metadata about the plugin
pub static DESCRIPTOR: clap_plugin_descriptor = clap_plugin_descriptor {
    clap_version: clap_sys::version::CLAP_VERSION,
    id: PLUGIN_ID.as_ptr(),
    name: PLUGIN_NAME.as_ptr(),
    vendor: PLUGIN_VENDOR.as_ptr(),
    url: PLUGIN_URL.as_ptr(),
    manual_url: std::ptr::null(),
    support_url: std::ptr::null(),
    version: PLUGIN_VERSION.as_ptr(),
    description: PLUGIN_DESCRIPTION.as_ptr(),
    features: PLUGIN_FEATURES.as_ptr(),
};
