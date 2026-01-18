//! DSynth (main polyphonic synth) CLAP plugin implemented via dsynth-clap

#![allow(deprecated)]

use crate::audio::engine::SynthEngine;
use crate::params::SynthParams;
use crate::plugin::gui_param_change::GuiParamChange;
use crate::plugin::param_registry;
use crate::plugin::param_update::{param_apply, param_get};
use dsynth_clap::ParamId;
use dsynth_clap::{
    clap_sys, generate_clap_entry, ClapPlugin, ClapProcessor, Events, ParamDescriptor, ParamType,
    PluginDescriptor, PluginParams, PluginState, ProcessStatus,
};
use parking_lot::{Mutex, RwLock};
use std::ffi::{c_void, CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use triple_buffer::{Input, Output, TripleBuffer};

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

fn shared_params() -> &'static Arc<RwLock<SynthParams>> {
    static PARAMS: OnceLock<Arc<RwLock<SynthParams>>> = OnceLock::new();
    PARAMS.get_or_init(|| Arc::new(RwLock::new(SynthParams::default())))
}

static PARAMS_DIRTY: AtomicBool = AtomicBool::new(true);

// =============================================================================
// Plugin
// =============================================================================

pub struct DsynthMainPlugin {
    synth_params: Arc<RwLock<SynthParams>>,

    gui_window: Option<crate::gui::plugin_window::EditorWindowHandle>,
    gui_parent: Option<RawWindowHandle>,
    gui_size: (u32, u32),

    gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,
    gui_param_consumer: Option<Output<GuiParamChange>>,
}

impl ClapPlugin for DsynthMainPlugin {
    type Processor = DsynthMainProcessor;
    type Params = DsynthMainParams;

    fn descriptor() -> PluginDescriptor {
        PluginDescriptor::instrument("DSynth", "com.dsynth.dsynth")
            .version(env!("CARGO_PKG_VERSION"))
            .description(
                "Polyphonic wavetable synthesizer with 3 oscillators, filters, LFOs, and effects",
            )
            .vendor("DSynth")
            .with_features(&["instrument", "synthesizer", "stereo"])
    }

    fn clap_descriptor() -> &'static clap_sys::plugin::clap_plugin_descriptor {
        use clap_sys::plugin::clap_plugin_descriptor;

        static DESCRIPTOR: OnceLock<clap_plugin_descriptor> = OnceLock::new();
        static STRINGS: OnceLock<DescriptorStrings> = OnceLock::new();
        static FEATURES: OnceLock<DescriptorFeatures> = OnceLock::new();

        struct DescriptorStrings {
            id: CString,
            name: CString,
            vendor: CString,
            version: CString,
            description: CString,
            feature1: CString,
            feature2: CString,
            feature3: CString,
        }

        struct DescriptorFeatures {
            features: [*const i8; 4],
        }

        unsafe impl Sync for DescriptorFeatures {}
        unsafe impl Send for DescriptorFeatures {}

        impl DescriptorStrings {
            fn new() -> Self {
                let id = CString::new("com.dsynth.dsynth").unwrap();
                let name = CString::new("DSynth").unwrap();
                let vendor = CString::new("DSynth").unwrap();
                let version = CString::new(env!("CARGO_PKG_VERSION")).unwrap();
                let description = CString::new(
                    "Polyphonic wavetable synthesizer with 3 oscillators, filters, LFOs, and effects",
                )
                .unwrap();

                let feature1 = CString::new("instrument").unwrap();
                let feature2 = CString::new("synthesizer").unwrap();
                let feature3 = CString::new("stereo").unwrap();

                Self {
                    id,
                    name,
                    vendor,
                    version,
                    description,
                    feature1,
                    feature2,
                    feature3,
                }
            }
        }

        let strings = STRINGS.get_or_init(DescriptorStrings::new);
        let features = FEATURES.get_or_init(|| DescriptorFeatures {
            features: [
                strings.feature1.as_ptr(),
                strings.feature2.as_ptr(),
                strings.feature3.as_ptr(),
                std::ptr::null(),
            ],
        });
        DESCRIPTOR.get_or_init(|| clap_plugin_descriptor {
            clap_version: clap_sys::version::CLAP_VERSION,
            id: strings.id.as_ptr(),
            name: strings.name.as_ptr(),
            vendor: strings.vendor.as_ptr(),
            url: std::ptr::null(),
            manual_url: std::ptr::null(),
            support_url: std::ptr::null(),
            version: strings.version.as_ptr(),
            description: strings.description.as_ptr(),
            features: features.features.as_ptr(),
        })
    }

    fn new() -> Self {
        let synth_params = Arc::clone(shared_params());

        let gui_param_buffer = TripleBuffer::new(&GuiParamChange::default());
        let (gui_param_producer, gui_param_consumer) = gui_param_buffer.split();

        Self {
            synth_params,
            gui_window: None,
            gui_parent: None,
            gui_size: (
                crate::gui::theme::WINDOW_WIDTH,
                crate::gui::theme::WINDOW_HEIGHT,
            ),
            gui_param_producer: Arc::new(Mutex::new(gui_param_producer)),
            gui_param_consumer: Some(gui_param_consumer),
        }
    }

    fn create_processor(&mut self, sample_rate: f32) -> Self::Processor {
        let consumer = self.gui_param_consumer.take().unwrap_or_else(|| {
            let buf = TripleBuffer::new(&GuiParamChange::default());
            let (_p, c) = buf.split();
            c
        });
        DsynthMainProcessor::new(sample_rate, consumer)
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

        // Safety: we validated pointers are non-null above.
        unsafe { self.gui_get_preferred_api_unchecked(api, is_floating) }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn gui_get_size(&mut self, width: *mut u32, height: *mut u32) -> bool {
        if width.is_null() || height.is_null() {
            return false;
        }

        // Safety: we validated pointers are non-null above.
        unsafe { self.gui_get_size_unchecked(width, height) }
    }

    fn gui_can_resize(&mut self) -> bool {
        false
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

        // Safety: we validated `window` is non-null above.
        unsafe { self.gui_set_parent_unchecked(window) }
    }

    fn gui_show(&mut self) -> bool {
        if self.gui_window.is_some() {
            return true;
        }

        let Some(parent) = self.gui_parent else {
            return false;
        };

        match crate::gui::plugin_window::open_editor(
            parent,
            self.synth_params.clone(),
            self.gui_param_producer.clone(),
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

impl DsynthMainPlugin {
    unsafe fn gui_get_preferred_api_unchecked(
        &mut self,
        api: *mut *const i8,
        is_floating: *mut bool,
    ) -> bool {
        *is_floating = false;

        #[cfg(target_os = "macos")]
        {
            *api = CLAP_WINDOW_API_COCOA.as_ptr();
            return true;
        }

        #[cfg(target_os = "windows")]
        {
            *api = CLAP_WINDOW_API_WIN32.as_ptr();
            return true;
        }

        #[cfg(target_os = "linux")]
        {
            *api = CLAP_WINDOW_API_X11.as_ptr();
            return true;
        }

        #[allow(unreachable_code)]
        false
    }

    unsafe fn gui_get_size_unchecked(&mut self, width: *mut u32, height: *mut u32) -> bool {
        *width = self.gui_size.0;
        *height = self.gui_size.1;
        true
    }

    unsafe fn gui_set_parent_unchecked(&mut self, window: *const clap_window) -> bool {
        let window_ref = &*window;

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

        match crate::gui::plugin_window::open_editor(
            raw_handle,
            self.synth_params.clone(),
            self.gui_param_producer.clone(),
        ) {
            Some(handle) => {
                self.gui_window = Some(handle);
                true
            }
            None => false,
        }
    }
}

// =============================================================================
// Processor
// =============================================================================

pub struct DsynthMainProcessor {
    engine: SynthEngine,

    param_producer: Input<SynthParams>,
    current_params: SynthParams,

    gui_param_consumer: Output<GuiParamChange>,
    last_gui_change: GuiParamChange,

    sample_rate: f32,
}

impl DsynthMainProcessor {
    pub fn new(sample_rate: f32, gui_param_consumer: Output<GuiParamChange>) -> Self {
        let (mut producer, consumer) = crate::audio::create_parameter_buffer();
        let engine = SynthEngine::new(sample_rate, consumer);

        let initial_params = *shared_params().read();
        producer.write(initial_params);

        PARAMS_DIRTY.store(false, Ordering::Release);

        Self {
            engine,
            param_producer: producer,
            current_params: initial_params,
            gui_param_consumer,
            last_gui_change: GuiParamChange::default(),
            sample_rate,
        }
    }

    fn sync_params_if_dirty(&mut self) {
        if PARAMS_DIRTY.swap(false, Ordering::AcqRel) {
            let params = *shared_params().read();
            self.current_params = params;
            self.param_producer.write(self.current_params);
        }
    }

    #[inline]
    fn maybe_apply_gui_param_change(&mut self) {
        let change = *self.gui_param_consumer.read();
        if change == self.last_gui_change {
            return;
        }

        self.last_gui_change = change;
        if change.param_id == 0 {
            return;
        }

        // Special signal: 0xFFFFFFFF means "full sync" (e.g., after randomization)
        if change.param_id == 0xFFFF_FFFF {
            self.current_params = *shared_params().read();
            self.param_producer.write(self.current_params);
            return;
        }

        // Apply GUI change to the audio-thread copy.
        param_apply::apply_param(&mut self.current_params, change.param_id, change.normalized);
        self.param_producer.write(self.current_params);
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
                        self.engine.note_on(e.key as u8, e.velocity as f32);
                    }
                    clap_sys::events::CLAP_EVENT_NOTE_OFF => {
                        let e = &*(event as *const _ as *const clap_sys::events::clap_event_note);
                        self.engine.note_off(e.key as u8);
                    }
                    clap_sys::events::CLAP_EVENT_MIDI => {
                        let e = &*(event as *const _ as *const clap_sys::events::clap_event_midi);
                        let status = e.data[0] & 0xF0;
                        let key = e.data[1];
                        let vel = e.data[2];

                        match status {
                            0x90 => {
                                if vel == 0 {
                                    self.engine.note_off(key);
                                } else {
                                    self.engine.note_on(key, (vel as f32) / 127.0);
                                }
                            }
                            0x80 => {
                                self.engine.note_off(key);
                            }
                            _ => {}
                        }
                    }
                    clap_sys::events::CLAP_EVENT_PARAM_VALUE => {
                        // Some hosts may send param events in the process event stream.
                        // Handle them for compatibility, updating the audio-thread params only.
                        let e = &*(event as *const _
                            as *const clap_sys::events::clap_event_param_value);
                        let id = e.param_id as ParamId;
                        let normalized = e.value as f32;
                        param_apply::apply_param(&mut self.current_params, id, normalized);
                        self.param_producer.write(self.current_params);
                    }
                    clap_sys::events::CLAP_EVENT_TRANSPORT => {
                        let e =
                            &*(event as *const _ as *const clap_sys::events::clap_event_transport);
                        const CLAP_TRANSPORT_HAS_TEMPO: u32 = 1 << 0;
                        if (e.flags & CLAP_TRANSPORT_HAS_TEMPO) != 0 {
                            self.engine.set_tempo(e.tempo);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl ClapProcessor for DsynthMainProcessor {
    fn process(&mut self, audio: &mut dsynth_clap::AudioBuffers, events: &Events) -> ProcessStatus {
        self.sync_params_if_dirty();
        self.maybe_apply_gui_param_change();
        self.handle_events(events);

        let frames = audio.frames_count() as usize;

        unsafe {
            let Some((out_l, out_r)) = audio.output_stereo_mut(0) else {
                return ProcessStatus::Continue;
            };

            let n = frames.min(out_l.len()).min(out_r.len());

            for i in 0..n {
                let (l, r) = self.engine.process();
                out_l[i] = l;
                out_r[i] = r;
            }

            for i in n..frames {
                if i < out_l.len() {
                    out_l[i] = 0.0;
                }
                if i < out_r.len() {
                    out_r[i] = 0.0;
                }
            }
        }

        ProcessStatus::Continue
    }

    fn activate(&mut self, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() <= f32::EPSILON {
            return;
        }

        self.sample_rate = sample_rate;

        // Recreate engine + param buffer at the new rate.
        let (producer, consumer) = crate::audio::create_parameter_buffer();
        self.engine = SynthEngine::new(sample_rate, consumer);
        self.param_producer = producer;

        let params = *shared_params().read();
        self.current_params = params;
        self.param_producer.write(self.current_params);
    }

    fn reset(&mut self) {
        // Recreate engine + param buffer, keep GUI consumer.
        let (producer, consumer) = crate::audio::create_parameter_buffer();
        self.engine = SynthEngine::new(self.sample_rate, consumer);
        self.param_producer = producer;
        self.last_gui_change = GuiParamChange::default();

        let params = *shared_params().read();
        self.current_params = params;
        self.param_producer.write(self.current_params);
    }

    fn deactivate(&mut self) {
        // No-op: engine will be dropped when processor is dropped.
    }
}

// =============================================================================
// Parameters (normalized 0..1, mapped via existing ParamRegistry)
// =============================================================================

pub struct DsynthMainParams;

impl DsynthMainParams {
    fn registry() -> &'static param_registry::ParamRegistry {
        param_registry::get_registry()
    }

    fn descriptor_by_id(
        id: ParamId,
    ) -> Option<&'static crate::plugin::param_descriptor::ParamDescriptor> {
        param_registry::get_registry().get(id)
    }

    fn get_normalized(params: &SynthParams, id: ParamId) -> Option<f32> {
        let denorm = param_get::get_param(params, id);
        let desc = Self::descriptor_by_id(id)?;
        Some(desc.normalize_value(denorm))
    }
}

impl PluginParams for DsynthMainParams {
    fn param_count() -> u32 {
        Self::registry().count() as u32
    }

    fn param_descriptor(index: u32) -> Option<ParamDescriptor> {
        let reg = Self::registry();
        let id = reg.get_id_by_index(index as usize)?;
        Self::param_descriptor_by_id(id)
    }

    fn param_descriptor_by_id(id: ParamId) -> Option<ParamDescriptor> {
        let desc = Self::descriptor_by_id(id)?;

        let param_type = match &desc.param_type {
            crate::plugin::param_descriptor::ParamType::Bool => ParamType::Bool {
                default: desc.default > 0.5,
            },
            _ => ParamType::Float {
                min: 0.0,
                max: 1.0,
                default: desc.default,
            },
        };

        Some(ParamDescriptor {
            id,
            name: desc.name.clone(),
            module: desc.module.clone(),
            param_type,
            unit: desc.unit.clone(),
            is_automatable: desc.automation
                == crate::plugin::param_descriptor::AutomationState::ReadWrite,
            is_hidden: false,
        })
    }

    fn get_param(id: ParamId) -> Option<f32> {
        let params = shared_params().read();
        Self::get_normalized(&params, id)
    }

    fn set_param(id: ParamId, value: f32) {
        {
            let mut params = shared_params().write();
            param_apply::apply_param(&mut params, id, value);
        }
        PARAMS_DIRTY.store(true, Ordering::Release);
    }

    fn save_state() -> PluginState {
        let reg = Self::registry();
        let params = shared_params().read();

        let mut state = PluginState::default();
        state.version = 1;

        for id in reg.iter_ids() {
            if let Some(normalized) = Self::get_normalized(&params, id) {
                state.set_param(id, normalized);
            }
        }

        state
    }

    fn load_state(state: &PluginState) {
        {
            let mut params = shared_params().write();
            for (&id, &normalized) in state.params.iter() {
                param_apply::apply_param(&mut params, id, normalized);
            }
        }
        PARAMS_DIRTY.store(true, Ordering::Release);
    }

    fn format_param(id: ParamId, value: f32) -> String {
        let Some(desc) = Self::descriptor_by_id(id) else {
            return format!("{:.3}", value);
        };

        // `value` is normalized 0..1 because we expose CLAP-facing domain as 0..1.
        desc.format_value(value)
    }

    fn parse_param(id: ParamId, text: &str) -> Option<f32> {
        let t = text.trim();

        let Some(desc) = Self::descriptor_by_id(id) else {
            return t.parse::<f32>().ok();
        };

        match &desc.param_type {
            crate::plugin::param_descriptor::ParamType::Bool => {
                if t.eq_ignore_ascii_case("on") || t == "1" {
                    return Some(1.0);
                }
                if t.eq_ignore_ascii_case("off") || t == "0" {
                    return Some(0.0);
                }
                None
            }
            crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                // Accept either variant name or numeric index.
                if let Some((idx, _)) = variants
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.eq_ignore_ascii_case(t))
                {
                    let idx_f = idx as f32;
                    return Some(desc.normalize_value(idx_f));
                }

                // Fallback numeric (plain value or index)
                let number_str = t.split_whitespace().next().unwrap_or("");
                if let Ok(v) = number_str.parse::<f32>() {
                    return Some(desc.normalize_value(v));
                }
                None
            }
            _ => {
                // Parse leading number (strip any trailing unit) then normalize.
                let number_str = t.split_whitespace().next().unwrap_or("");
                let v = number_str.parse::<f32>().ok()?;
                Some(desc.normalize_value(v))
            }
        }
    }
}

// =============================================================================
// CLAP entry
// =============================================================================

generate_clap_entry!(DsynthMainPlugin);
