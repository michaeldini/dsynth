//! DSynth Kick CLAP plugin implemented via dsynth-clap

#![allow(deprecated)]

use crate::audio::kick_engine::{KickEngine, MidiEvent};
use crate::params_kick::KickParams;
use crate::plugin::kick_param_registry::{get_kick_registry, ParamId};
use dsynth_clap::{
    clap_sys, generate_clap_entry, ClapPlugin, ClapProcessor, Events, ParamDescriptor, ParamType,
    PluginDescriptor, PluginParams, PluginState, ProcessStatus,
};
use parking_lot::Mutex;
use std::ffi::CString;
use std::ffi::{c_void, CStr};
use std::sync::{Arc, OnceLock};

#[cfg(target_os = "macos")]
use clap_sys::ext::gui::CLAP_WINDOW_API_COCOA;
#[cfg(target_os = "windows")]
use clap_sys::ext::gui::CLAP_WINDOW_API_WIN32;
#[cfg(target_os = "linux")]
use clap_sys::ext::gui::CLAP_WINDOW_API_X11;

use clap_sys::ext::gui::clap_window;

#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use cocoa::foundation::NSRect;
#[cfg(target_os = "macos")]
use objc::msg_send;
#[cfg(target_os = "macos")]
use objc::sel;
#[cfg(target_os = "macos")]
use objc::sel_impl;

#[cfg(target_os = "macos")]
use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
#[cfg(target_os = "linux")]
use raw_window_handle::{RawWindowHandle, XlibWindowHandle};

#[cfg(target_os = "macos")]
unsafe fn cocoa_nsview_bounds_size(ns_view_ptr: *mut c_void) -> Option<(u32, u32)> {
    if ns_view_ptr.is_null() {
        return None;
    }

    let ns_view = ns_view_ptr as id;
    let bounds: NSRect = msg_send![ns_view, bounds];

    let width = bounds.size.width.round().max(1.0) as u32;
    let height = bounds.size.height.round().max(1.0) as u32;
    Some((width, height))
}

// =============================================================================
// Global parameter storage (dsynth-clap PluginParams is currently static)
// =============================================================================

fn shared_params() -> &'static Arc<Mutex<KickParams>> {
    static PARAMS: OnceLock<Arc<Mutex<KickParams>>> = OnceLock::new();
    PARAMS.get_or_init(|| Arc::new(Mutex::new(KickParams::default())))
}

// =============================================================================
// Plugin
// =============================================================================

pub struct DsynthKickPlugin {
    kick_params: Arc<Mutex<KickParams>>,
    gui_window: Option<crate::gui::kick_plugin_window::EditorWindowHandle>,
    gui_parent: Option<RawWindowHandle>,
    gui_size: (u32, u32),
}

impl ClapPlugin for DsynthKickPlugin {
    type Processor = DsynthKickProcessor;
    type Params = DsynthKickParams;

    fn descriptor() -> PluginDescriptor {
        PluginDescriptor::instrument("DSynth Kick", "com.dsynth.kick")
            .version(env!("CARGO_PKG_VERSION"))
            .description("Monophonic kick drum synthesizer")
            .vendor("DSynth")
            .with_features(&["synthesizer", "drum", "instrument"])
    }

    fn clap_descriptor() -> &'static clap_sys::plugin::clap_plugin_descriptor {
        use clap_sys::plugin::clap_plugin_descriptor;

        static mut DESCRIPTOR: Option<clap_plugin_descriptor> = None;
        static mut STRINGS: Option<DescriptorStrings> = None;

        struct DescriptorStrings {
            id: CString,
            name: CString,
            vendor: CString,
            version: CString,
            description: CString,
            feature1: CString,
            feature2: CString,
            feature3: CString,
            features: [*const i8; 4],
        }

        unsafe {
            if DESCRIPTOR.is_none() {
                let id = CString::new("com.dsynth.kick").unwrap();
                let name = CString::new("DSynth Kick").unwrap();
                let vendor = CString::new("DSynth").unwrap();
                let version = CString::new(env!("CARGO_PKG_VERSION")).unwrap();
                let description = CString::new("Monophonic kick drum synthesizer").unwrap();

                let feature1 = CString::new("instrument").unwrap();
                let feature2 = CString::new("synthesizer").unwrap();
                let feature3 = CString::new("drum").unwrap();

                let features = [
                    feature1.as_ptr(),
                    feature2.as_ptr(),
                    feature3.as_ptr(),
                    std::ptr::null(),
                ];

                let strings = DescriptorStrings {
                    id,
                    name,
                    vendor,
                    version,
                    description,
                    feature1,
                    feature2,
                    feature3,
                    features,
                };

                DESCRIPTOR = Some(clap_plugin_descriptor {
                    clap_version: clap_sys::version::CLAP_VERSION,
                    id: strings.id.as_ptr(),
                    name: strings.name.as_ptr(),
                    vendor: strings.vendor.as_ptr(),
                    url: std::ptr::null(),
                    manual_url: std::ptr::null(),
                    support_url: std::ptr::null(),
                    version: strings.version.as_ptr(),
                    description: strings.description.as_ptr(),
                    features: strings.features.as_ptr(),
                });

                STRINGS = Some(strings);
            }

            DESCRIPTOR.as_ref().unwrap()
        }
    }

    fn new() -> Self {
        // Ensure global params are initialized
        let kick_params = Arc::clone(shared_params());
        Self {
            kick_params,
            gui_window: None,
            gui_parent: None,
            gui_size: (720, 900),
        }
    }

    fn create_processor(&mut self, sample_rate: f32) -> Self::Processor {
        DsynthKickProcessor::new(sample_rate, Arc::clone(shared_params()))
    }

    fn has_gui() -> bool {
        true
    }

    fn gui_is_api_supported(&mut self, api: &CStr, _is_floating: bool) -> bool {
        #[cfg(target_os = "macos")]
        return api.to_bytes() == CLAP_WINDOW_API_COCOA.to_bytes();

        #[cfg(target_os = "windows")]
        return api.to_bytes() == CLAP_WINDOW_API_WIN32.to_bytes();

        #[cfg(target_os = "linux")]
        return api.to_bytes() == CLAP_WINDOW_API_X11.to_bytes();

        #[allow(unreachable_code)]
        {
            let _ = api;
            false
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn gui_get_preferred_api(&mut self, api: *mut *const i8, is_floating: *mut bool) -> bool {
        if api.is_null() || is_floating.is_null() {
            return false;
        }

        unsafe {
            *is_floating = false;
        }

        #[cfg(target_os = "macos")]
        unsafe {
            *api = CLAP_WINDOW_API_COCOA.as_ptr();
            return true;
        }

        #[cfg(target_os = "windows")]
        unsafe {
            *api = CLAP_WINDOW_API_WIN32.as_ptr();
            return true;
        }

        #[cfg(target_os = "linux")]
        unsafe {
            *api = CLAP_WINDOW_API_X11.as_ptr();
            return true;
        }

        #[allow(unreachable_code)]
        false
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn gui_get_size(&mut self, width: *mut u32, height: *mut u32) -> bool {
        if width.is_null() || height.is_null() {
            return false;
        }
        unsafe {
            *width = self.gui_size.0;
            *height = self.gui_size.1;
        }
        true
    }

    fn gui_can_resize(&mut self) -> bool {
        true
    }

    fn gui_set_size(&mut self, width: u32, height: u32) -> bool {
        let width = width.max(1);
        let height = height.max(1);
        self.gui_size = (width, height);

        // If the editor is open, reopen it at the new size.
        if let Some(parent) = self.gui_parent {
            self.gui_window = None;
            match crate::gui::kick_plugin_window::open_editor(
                parent,
                self.kick_params.clone(),
                width,
                height,
            ) {
                Some(handle) => {
                    self.gui_window = Some(handle);
                    true
                }
                None => false,
            }
        } else {
            true
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn gui_set_parent(&mut self, window: *const clap_window) -> bool {
        // Drop any existing window before reparenting.
        if self.gui_window.is_some() {
            self.gui_window = None;
        }

        if window.is_null() {
            self.gui_parent = None;
            self.gui_window = None;
            return true;
        }

        let window_ref = unsafe { &*window };

        #[cfg(target_os = "macos")]
        let raw_handle = unsafe {
            let cocoa_ptr = window_ref.specific.cocoa;
            let mut handle = AppKitWindowHandle::empty();
            handle.ns_view = cocoa_ptr;
            RawWindowHandle::AppKit(handle)
        };

        #[cfg(target_os = "windows")]
        let raw_handle = unsafe {
            let mut handle = Win32WindowHandle::empty();
            handle.hwnd = window_ref.specific.win32;
            RawWindowHandle::Win32(handle)
        };

        #[cfg(target_os = "linux")]
        let raw_handle = unsafe {
            let mut handle = XlibWindowHandle::empty();
            handle.window = window_ref.specific.x11;
            RawWindowHandle::Xlib(handle)
        };

        #[cfg(target_os = "macos")]
        let host_size = unsafe { cocoa_nsview_bounds_size(window_ref.specific.cocoa) };
        #[cfg(not(target_os = "macos"))]
        let host_size: Option<(u32, u32)> = None;

        let (width, height) = host_size.unwrap_or(self.gui_size);
        self.gui_size = (width, height);
        self.gui_parent = Some(raw_handle);

        match crate::gui::kick_plugin_window::open_editor(
            raw_handle,
            self.kick_params.clone(),
            width,
            height,
        ) {
            Some(handle) => {
                self.gui_window = Some(handle);
                true
            }
            None => false,
        }
    }

    fn gui_show(&mut self) -> bool {
        if self.gui_window.is_some() {
            return true;
        }

        let Some(parent) = self.gui_parent else {
            return false;
        };

        let (width, height) = self.gui_size;
        match crate::gui::kick_plugin_window::open_editor(
            parent,
            self.kick_params.clone(),
            width,
            height,
        ) {
            Some(handle) => {
                self.gui_window = Some(handle);
                true
            }
            None => false,
        }
    }

    fn gui_hide(&mut self) -> bool {
        self.gui_window = None;
        true
    }
}

// =============================================================================
// Processor
// =============================================================================

pub struct DsynthKickProcessor {
    engine: KickEngine,
    note_queue: Arc<Mutex<Vec<MidiEvent>>>,
    sample_rate: f32,
}

impl DsynthKickProcessor {
    pub fn new(sample_rate: f32, params: Arc<Mutex<KickParams>>) -> Self {
        let engine = KickEngine::new(sample_rate, Arc::clone(&params));
        let note_queue = engine.get_note_queue();

        Self {
            engine,
            note_queue,
            sample_rate,
        }
    }

    fn handle_events(&mut self, events: &Events) {
        unsafe {
            for i in 0..events.input_event_count() {
                let Some(event) = events.input_event(i) else {
                    continue;
                };

                if event.space_id != clap_sys::events::CLAP_CORE_EVENT_SPACE_ID {
                    continue;
                }

                match event.type_ {
                    clap_sys::events::CLAP_EVENT_NOTE_ON => {
                        let e = &*(event as *const _ as *const clap_sys::events::clap_event_note);
                        let mut q = self.note_queue.lock();
                        q.push(MidiEvent::NoteOn {
                            note: e.key as u8,
                            velocity: e.velocity as f32,
                        });
                    }
                    clap_sys::events::CLAP_EVENT_NOTE_OFF => {
                        let e = &*(event as *const _ as *const clap_sys::events::clap_event_note);
                        let mut q = self.note_queue.lock();
                        q.push(MidiEvent::NoteOff { note: e.key as u8 });
                    }
                    clap_sys::events::CLAP_EVENT_MIDI => {
                        let e = &*(event as *const _ as *const clap_sys::events::clap_event_midi);
                        let status = e.data[0] & 0xF0;
                        let key = e.data[1];
                        let vel = e.data[2];

                        let mut q = self.note_queue.lock();
                        match status {
                            0x90 => {
                                if vel == 0 {
                                    q.push(MidiEvent::NoteOff { note: key });
                                } else {
                                    q.push(MidiEvent::NoteOn {
                                        note: key,
                                        velocity: (vel as f32) / 127.0,
                                    });
                                }
                            }
                            0x80 => {
                                q.push(MidiEvent::NoteOff { note: key });
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl ClapProcessor for DsynthKickProcessor {
    fn process(&mut self, audio: &mut dsynth_clap::AudioBuffers, events: &Events) -> ProcessStatus {
        // Kick is a synth: ignore any audio inputs, produce stereo output.
        self.handle_events(events);

        let frames = audio.frames_count() as usize;

        unsafe {
            // Preferred: stereo.
            if let Some((left, right)) = audio.output_stereo_mut(0) {
                self.engine.process_block_stereo(left, right);
                return ProcessStatus::Continue;
            }

            // Fallback: write mono/partial outputs if host provided them.
            let Some(out0) = audio.output_channel_mut(0, 0) else {
                return ProcessStatus::Continue;
            };

            for out in out0.iter_mut().take(frames) {
                *out = self.engine.process_sample();
            }
        }

        ProcessStatus::Continue
    }

    fn activate(&mut self, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() <= f32::EPSILON {
            return;
        }

        // Recreate the engine at the new sample rate.
        let params = Arc::clone(shared_params());
        *self = Self::new(sample_rate, params);
    }

    fn reset(&mut self) {
        // Recreate engine state cheaply by re-instantiating.
        let params = Arc::clone(shared_params());
        *self = Self::new(self.sample_rate, params);
    }
}

// =============================================================================
// Parameters (normalized 0..1, mapped via existing KickParamRegistry)
// =============================================================================

pub struct DsynthKickParams;

impl DsynthKickParams {
    fn registry() -> &'static crate::plugin::kick_param_registry::KickParamRegistry {
        get_kick_registry()
    }
}

impl PluginParams for DsynthKickParams {
    fn param_count() -> u32 {
        Self::registry().param_count() as u32
    }

    fn param_descriptor(index: u32) -> Option<ParamDescriptor> {
        let reg = Self::registry();
        let id = *reg.param_ids().get(index as usize)?;
        Self::param_descriptor_by_id(id)
    }

    fn param_descriptor_by_id(id: ParamId) -> Option<ParamDescriptor> {
        let reg = Self::registry();
        let desc = reg.get_descriptor(id)?;

        // dsynth-clap currently normalizes values based on ParamDescriptor;
        // we keep the CLAP-facing domain strictly 0..1 and let KickParamRegistry
        // handle any skewing (log/exp) when applying to KickParams.
        Some(ParamDescriptor {
            id,
            name: desc.name.clone(),
            module: desc.module.clone(),
            param_type: ParamType::Float {
                min: 0.0,
                max: 1.0,
                default: desc.default,
            },
            unit: desc.unit.clone(),
            is_automatable: desc.automation
                == crate::plugin::param_descriptor::AutomationState::ReadWrite,
            is_hidden: false,
        })
    }

    fn get_param(id: ParamId) -> Option<f32> {
        let params = shared_params().lock();
        let value = Self::registry().get_param(&params, id);
        Some(value as f32)
    }

    fn set_param(id: ParamId, value: f32) {
        let mut params = shared_params().lock();
        Self::registry().apply_param(&mut params, id, value as f64);
    }

    fn save_state() -> PluginState {
        let reg = Self::registry();
        let params = shared_params().lock();

        let mut state = PluginState::default();
        state.version = 1;

        for &id in reg.param_ids() {
            let normalized = reg.get_param(&params, id) as f32;
            state.set_param(id, normalized);
        }

        state
    }

    fn load_state(state: &PluginState) {
        for (&id, &value) in state.params.iter() {
            Self::set_param(id, value);
        }
    }

    fn format_param(id: ParamId, value: f32) -> String {
        let reg = Self::registry();
        let desc = reg.get_descriptor(id);

        // `value` is normalized 0..1; convert to internal/plain value for display.
        let plain = reg.denormalize_value(id, value as f64) as f32;

        if let Some(desc) = desc {
            match &desc.param_type {
                crate::plugin::param_descriptor::ParamType::Bool => {
                    if value >= 0.5 {
                        "On".to_string()
                    } else {
                        "Off".to_string()
                    }
                }
                crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                    let idx = plain
                        .round()
                        .clamp(0.0, (variants.len().saturating_sub(1)) as f32)
                        as usize;
                    variants
                        .get(idx)
                        .cloned()
                        .unwrap_or_else(|| idx.to_string())
                }
                crate::plugin::param_descriptor::ParamType::Int { .. } => {
                    format!("{}", plain.round() as i32)
                }
                crate::plugin::param_descriptor::ParamType::Float { .. } => {
                    if let Some(unit) = &desc.unit {
                        format!("{:.2} {}", plain, unit)
                    } else {
                        format!("{:.2}", plain)
                    }
                }
            }
        } else {
            format!("{:.3}", value)
        }
    }

    fn parse_param(id: ParamId, text: &str) -> Option<f32> {
        let reg = Self::registry();
        let desc = reg.get_descriptor(id);
        let t = text.trim();

        if let Some(desc) = desc {
            match &desc.param_type {
                crate::plugin::param_descriptor::ParamType::Bool => {
                    let v = match t.to_ascii_lowercase().as_str() {
                        "1" | "true" | "on" | "yes" => 1.0,
                        "0" | "false" | "off" | "no" => 0.0,
                        _ => return None,
                    };
                    return Some(v);
                }
                crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                    if let Some((idx, _)) = variants
                        .iter()
                        .enumerate()
                        .find(|(_, v)| v.eq_ignore_ascii_case(t))
                    {
                        // Convert enum index (plain) into normalized using the registry.
                        let normalized = reg.normalize_value(id, idx as f64);
                        return Some(normalized as f32);
                    }
                }
                _ => {}
            }
        }

        // Fallback: parse a number, strip any trailing unit.
        let number_str = t.split_whitespace().next().unwrap_or("");
        let plain = number_str.parse::<f64>().ok()?;
        Some(reg.normalize_value(id, plain) as f32)
    }
}

// =============================================================================
// CLAP entry
// =============================================================================

generate_clap_entry!(DsynthKickPlugin);
