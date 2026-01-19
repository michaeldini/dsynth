//! Plugin instance management - bridges trait-based API to CLAP C callbacks

use crate::{
    entry::log_entry, plugin::ClapPlugin, processor::ClapProcessor, AudioBuffers, Events,
    ProcessStatus,
};
use clap_sys::host::clap_host;
use clap_sys::plugin::*;
use clap_sys::process::*;
use std::ffi::CStr;
use std::os::raw::c_void;

/// Wraps a trait-based plugin implementation and bridges it to CLAP C API
pub struct PluginInstance<P: ClapPlugin> {
    /// The trait-based plugin implementation
    pub(crate) plugin: P,
    /// The processor instance (created during activate)
    processor: Option<P::Processor>,
    /// Host callback
    host: *const clap_host,
    /// Current sample rate
    sample_rate: f32,
    /// Whether the plugin is currently activated
    is_activated: bool,
}

impl<P: ClapPlugin> PluginInstance<P> {
    /// Create a new plugin instance
    pub fn new(host: *const clap_host) -> Box<Self> {
        log_entry("PluginInstance::new() called");
        Box::new(Self {
            plugin: P::new(),
            processor: None,
            host,
            sample_rate: 44100.0, // Default, will be set during activate
            is_activated: false,
        })
    }

    /// Initialize the plugin (called by host after construction)
    ///
    /// # Safety
    /// Must only be called by the CLAP host on a valid instance.
    pub unsafe fn init(&mut self) -> bool {
        log_entry("PluginInstance::init() called");
        self.plugin.init();
        true
    }

    /// Destroy the plugin (called by host before freeing)
    ///
    /// # Safety
    /// Must only be called by the CLAP host on a valid instance.
    pub unsafe fn destroy(&mut self) {
        log_entry("PluginInstance::destroy() called");
        self.deactivate();
        // Drop happens automatically
    }

    /// Activate the plugin for processing
    ///
    /// # Safety
    /// Must only be called by the CLAP host with a valid sample rate.
    pub unsafe fn activate(
        &mut self,
        sample_rate: f64,
        _min_frames: u32,
        _max_frames: u32,
    ) -> bool {
        log_entry(&format!(
            "PluginInstance::activate() called (sample_rate={})",
            sample_rate
        ));
        if self.is_activated {
            return false;
        }

        self.sample_rate = sample_rate as f32;
        if self.processor.is_none() {
            let processor = self.plugin.create_processor(self.sample_rate);
            self.processor = Some(processor);
        }

        if let Some(processor) = self.processor.as_mut() {
            processor.activate(self.sample_rate);
        }
        self.is_activated = true;
        true
    }

    /// Deactivate the plugin
    ///
    /// # Safety
    /// Must only be called by the CLAP host on a valid instance.
    pub unsafe fn deactivate(&mut self) {
        log_entry("PluginInstance::deactivate() called");
        if !self.is_activated {
            return;
        }

        if let Some(processor) = self.processor.as_mut() {
            processor.deactivate();
        }
        self.is_activated = false;
    }

    /// Main audio processing callback
    ///
    /// # Safety
    /// `process` must be a valid pointer to a CLAP `clap_process` struct provided by the host.
    pub unsafe fn process(&mut self, process: *const clap_process) -> clap_process_status {
        if process.is_null() {
            return CLAP_PROCESS_ERROR;
        }

        let process = &*process;

        // Get processor
        let processor = match self.processor.as_mut() {
            Some(p) => p,
            None => return CLAP_PROCESS_ERROR,
        };

        // Create safe audio buffers wrapper
        let mut audio_buffers = match AudioBuffers::from_clap_process(process) {
            Ok(buffers) => buffers,
            Err(_) => return CLAP_PROCESS_ERROR,
        };

        // Create events wrapper
        let events = Events::from_clap_process(process);

        // Call the trait-based processor
        let status = processor.process(&mut audio_buffers, &events);

        // Convert status back to CLAP constants
        match status {
            ProcessStatus::Continue => CLAP_PROCESS_CONTINUE,
            ProcessStatus::ContinueIfNotQuiet => CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
            ProcessStatus::Tail => CLAP_PROCESS_TAIL,
            ProcessStatus::Sleep => CLAP_PROCESS_SLEEP,
        }
    }

    /// Get the plugin descriptor
    pub fn descriptor() -> &'static clap_plugin_descriptor {
        P::clap_descriptor()
    }

    /// Get the host callback
    pub fn host(&self) -> *const clap_host {
        self.host
    }

    /// Convert from plugin pointer to instance reference
    ///
    /// # Safety
    /// `plugin` must be a valid CLAP plugin pointer created by this library.
    pub unsafe fn from_ptr<'a>(plugin: *const clap_plugin) -> &'a Self {
        let instance_ptr = (*plugin).plugin_data as *const Self;
        &*instance_ptr
    }

    /// Convert from plugin pointer to mutable instance reference
    ///
    /// # Safety
    /// `plugin` must be a valid CLAP plugin pointer created by this library.
    pub unsafe fn from_ptr_mut<'a>(plugin: *const clap_plugin) -> &'a mut Self {
        let instance_ptr = (*plugin).plugin_data as *mut Self;
        &mut *instance_ptr
    }
}

/// Create the clap_plugin struct for a plugin instance
///
/// # Safety
/// `instance` must be a valid, heap-allocated pointer created by `PluginInstance::<P>::new()`.
pub unsafe fn create_clap_plugin<P: ClapPlugin>(instance: *mut PluginInstance<P>) -> clap_plugin {
    log_entry("create_clap_plugin() called");
    clap_plugin {
        desc: P::clap_descriptor(),
        plugin_data: instance as *mut c_void,
        init: Some(plugin_init::<P>),
        destroy: Some(plugin_destroy::<P>),
        activate: Some(plugin_activate::<P>),
        deactivate: Some(plugin_deactivate::<P>),
        start_processing: Some(plugin_start_processing::<P>),
        stop_processing: Some(plugin_stop_processing::<P>),
        reset: Some(plugin_reset::<P>),
        process: Some(plugin_process::<P>),
        get_extension: Some(plugin_get_extension::<P>),
        on_main_thread: Some(plugin_on_main_thread::<P>),
    }
}

// CLAP plugin callbacks - these forward to PluginInstance methods

unsafe extern "C" fn plugin_init<P: ClapPlugin>(plugin: *const clap_plugin) -> bool {
    log_entry("plugin_init() called");
    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.init()
}

unsafe extern "C" fn plugin_destroy<P: ClapPlugin>(plugin: *const clap_plugin) {
    log_entry("plugin_destroy() called");
    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.destroy();
    // Free the instance memory
    let _ = Box::from_raw(instance as *mut PluginInstance<P>);
}

unsafe extern "C" fn plugin_activate<P: ClapPlugin>(
    plugin: *const clap_plugin,
    sample_rate: f64,
    min_frames_count: u32,
    max_frames_count: u32,
) -> bool {
    log_entry("plugin_activate() called");
    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.activate(sample_rate, min_frames_count, max_frames_count)
}

unsafe extern "C" fn plugin_deactivate<P: ClapPlugin>(plugin: *const clap_plugin) {
    log_entry("plugin_deactivate() called");
    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.deactivate();
}

unsafe extern "C" fn plugin_start_processing<P: ClapPlugin>(plugin: *const clap_plugin) -> bool {
    let _ = PluginInstance::<P>::from_ptr(plugin);
    true // No-op for now
}

unsafe extern "C" fn plugin_stop_processing<P: ClapPlugin>(plugin: *const clap_plugin) {
    let _ = PluginInstance::<P>::from_ptr(plugin);
    // No-op for now
}

unsafe extern "C" fn plugin_reset<P: ClapPlugin>(plugin: *const clap_plugin) {
    log_entry("plugin_reset() called");
    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    if let Some(processor) = instance.processor.as_mut() {
        processor.reset();
    }
}

unsafe extern "C" fn plugin_process<P: ClapPlugin>(
    plugin: *const clap_plugin,
    process: *const clap_process,
) -> clap_process_status {
    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.process(process)
}

unsafe extern "C" fn plugin_get_extension<P: ClapPlugin>(
    plugin: *const clap_plugin,
    id: *const i8,
) -> *const c_void {
    log_entry("plugin_get_extension() called");
    if id.is_null() {
        return std::ptr::null();
    }

    let id_str = match CStr::from_ptr(id).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };

    let instance = PluginInstance::<P>::from_ptr(plugin);

    // Convert CStr constants to str for matching
    let audio_ports_id = clap_sys::ext::audio_ports::CLAP_EXT_AUDIO_PORTS
        .to_str()
        .unwrap_or("");
    let note_ports_id = clap_sys::ext::note_ports::CLAP_EXT_NOTE_PORTS
        .to_str()
        .unwrap_or("");
    let params_id = clap_sys::ext::params::CLAP_EXT_PARAMS
        .to_str()
        .unwrap_or("");
    let state_id = clap_sys::ext::state::CLAP_EXT_STATE.to_str().unwrap_or("");
    let gui_id = clap_sys::ext::gui::CLAP_EXT_GUI.to_str().unwrap_or("");

    if id_str == audio_ports_id {
        crate::extensions::audio_ports::get_extension::<P>() as *const _ as *const c_void
    } else if id_str == note_ports_id {
        crate::extensions::note_ports::get_extension::<P>() as *const _ as *const c_void
    } else if id_str == params_id {
        crate::extensions::params::get_extension::<P>(instance.host) as *const _ as *const c_void
    } else if id_str == state_id {
        crate::extensions::state::get_extension::<P>() as *const _ as *const c_void
    } else if id_str == gui_id {
        crate::extensions::gui::get_extension::<P>() as *const _ as *const c_void
    } else {
        std::ptr::null()
    }
}

unsafe extern "C" fn plugin_on_main_thread<P: ClapPlugin>(_plugin: *const clap_plugin) {
    // No-op for now - could be used for GUI updates
}
