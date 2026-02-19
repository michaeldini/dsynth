//! DSynth Voice Enhancer CLAP plugin implemented via dsynth-clap

use crate::audio::voice_engine::VoiceEngine;
use crate::params_voice::VoiceParams;
use crate::plugin::voice_param_registry;
use dsynth_clap::ParamId;
use dsynth_clap::{
    clap_sys, generate_clap_entry, ClapPlugin, ClapProcessor, Events, ParamDescriptor, ParamType,
    PluginDescriptor, PluginParams, PluginState, ProcessStatus,
};
use parking_lot::Mutex;
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

// =============================================================================
// Global parameter storage (dsynth-clap PluginParams is currently static)
// =============================================================================

fn shared_params() -> &'static Arc<Mutex<VoiceParams>> {
    static PARAMS: OnceLock<Arc<Mutex<VoiceParams>>> = OnceLock::new();
    PARAMS.get_or_init(|| Arc::new(Mutex::new(VoiceParams::default())))
}

static PARAMS_DIRTY: AtomicBool = AtomicBool::new(true);

// =============================================================================
// Plugin
// =============================================================================

pub struct DsynthVoicePlugin;

impl ClapPlugin for DsynthVoicePlugin {
    type Processor = DsynthVoiceProcessor;
    type Params = DsynthVoiceParams;

    fn descriptor() -> PluginDescriptor {
        PluginDescriptor::effect("DSynth Voice", "com.dsynth.voice-enhancer")
            .version(env!("CARGO_PKG_VERSION"))
            .description("Voice enhancement with pitch-tracked sub oscillator")
            .vendor("DSynth")
            .with_features(&["audio-effect", "voice", "dynamics", "eq"])
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
            feature4: CString,
        }

        struct DescriptorFeatures {
            features: [*const i8; 5],
        }

        unsafe impl Sync for DescriptorFeatures {}
        unsafe impl Send for DescriptorFeatures {}

        impl DescriptorStrings {
            fn new() -> Self {
                let id = CString::new("com.dsynth.voice-enhancer").unwrap();
                let name = CString::new("DSynth Voice").unwrap();
                let vendor = CString::new("DSynth").unwrap();
                let version = CString::new(env!("CARGO_PKG_VERSION")).unwrap();
                let description =
                    CString::new("Voice enhancement with pitch-tracked sub oscillator").unwrap();

                let feature1 = CString::new("audio-effect").unwrap();
                let feature2 = CString::new("voice").unwrap();
                let feature3 = CString::new("dynamics").unwrap();
                let feature4 = CString::new("eq").unwrap();

                Self {
                    id,
                    name,
                    vendor,
                    version,
                    description,
                    feature1,
                    feature2,
                    feature3,
                    feature4,
                }
            }
        }

        let strings = STRINGS.get_or_init(DescriptorStrings::new);
        let features = FEATURES.get_or_init(|| DescriptorFeatures {
            features: [
                strings.feature1.as_ptr(),
                strings.feature2.as_ptr(),
                strings.feature3.as_ptr(),
                strings.feature4.as_ptr(),
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
        // Ensure params exist before any host queries.
        let _ = shared_params();
        DsynthVoicePlugin
    }

    fn create_processor(&mut self, sample_rate: f32) -> Self::Processor {
        DsynthVoiceProcessor::new(sample_rate)
    }
}

// =============================================================================
// Processor
// =============================================================================

pub struct DsynthVoiceProcessor {
    engine: VoiceEngine,
    sample_rate: f32,
}

impl DsynthVoiceProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let mut engine = VoiceEngine::new(sample_rate);

        // Prime engine with current params.
        let params = shared_params().lock().clone();
        engine.update_params(params);
        PARAMS_DIRTY.store(false, Ordering::Release);

        Self {
            engine,
            sample_rate,
        }
    }

    fn sync_params_if_dirty(&mut self) {
        if PARAMS_DIRTY.swap(false, Ordering::AcqRel) {
            let params = shared_params().lock().clone();
            self.engine.update_params(params);
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

                if event.type_ == clap_sys::events::CLAP_EVENT_PARAM_VALUE {
                    let e =
                        &*(event as *const _ as *const clap_sys::events::clap_event_param_value);
                    let normalized = (e.value as f32).clamp(0.0, 1.0);

                    let mut params = shared_params().lock();
                    if let Some(desc) = DsynthVoiceParams::descriptor_by_id(e.param_id) {
                        let denorm = desc.denormalize(normalized);
                        voice_param_registry::apply_param(e.param_id, denorm, &mut params);
                        PARAMS_DIRTY.store(true, Ordering::Release);
                    }
                }
            }
        }
    }
}

impl ClapProcessor for DsynthVoiceProcessor {
    fn process(&mut self, audio: &mut dsynth_clap::AudioBuffers, events: &Events) -> ProcessStatus {
        self.handle_events(events);
        self.sync_params_if_dirty();

        let frames = audio.frames_count() as usize;

        unsafe {
            // Voice is an effect: requires stereo input and produces stereo output.
            if let Some((in_l, in_r, out_l, out_r)) = audio.io_stereo_mut(0, 0) {
                // Process audio through the enhancement chain
                let n = frames
                    .min(in_l.len())
                    .min(in_r.len())
                    .min(out_l.len())
                    .min(out_r.len());

                // DEBUG: Check input values
                let input_sum: f32 = in_l.iter().take(n).map(|v| v.abs()).sum();

                self.engine.process_buffer(in_l, in_r, out_l, out_r, n);

                // DEBUG: Check output values
                let output_sum: f32 = out_l.iter().take(n).map(|v| v.abs()).sum();

                use std::fs::OpenOptions;
                use std::io::Write;
                let _ = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .and_then(|mut f| {
                        writeln!(
                            f,
                            "[PROCESS] frames={}, input_sum={:.6}, output_sum={:.6}",
                            n, input_sum, output_sum
                        )
                    });

                // If host gave more frames than we processed, clear remainder.
                for i in n..frames.min(out_l.len()) {
                    out_l[i] = 0.0;
                }
                for i in n..frames.min(out_r.len()) {
                    out_r[i] = 0.0;
                }
            } else {
                // Fallback: can't get stereo I/O - try to at least output silence
                // This should never happen for a properly configured effect plugin
                if let Some((out_l, out_r)) = audio.output_stereo_mut(0) {
                    for i in 0..frames.min(out_l.len()).min(out_r.len()) {
                        out_l[i] = 0.0;
                        out_r[i] = 0.0;
                    }
                }
                // Return Continue (not Sleep) so host keeps calling us
            }
        }

        ProcessStatus::Continue
    }

    fn activate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.engine = VoiceEngine::new(sample_rate);
        PARAMS_DIRTY.store(true, Ordering::Release);
        self.sync_params_if_dirty();
    }

    fn reset(&mut self) {
        self.engine.reset();
    }

    fn deactivate(&mut self) {
        self.engine.reset();
    }

    fn latency(&self) -> u32 {
        self.engine.get_latency()
    }
}

// =============================================================================
// Parameters (normalized 0..1, mapped via voice_param_registry)
// =============================================================================

pub struct DsynthVoiceParams;

impl DsynthVoiceParams {
    fn registry(
    ) -> &'static indexmap::IndexMap<ParamId, crate::plugin::param_descriptor::ParamDescriptor>
    {
        voice_param_registry::get_voice_param_registry()
    }

    fn descriptor_by_id(
        id: ParamId,
    ) -> Option<&'static crate::plugin::param_descriptor::ParamDescriptor> {
        voice_param_registry::get_param_descriptor(id)
    }
}

impl PluginParams for DsynthVoiceParams {
    fn param_count() -> u32 {
        Self::registry().len() as u32
    }

    fn param_descriptor(index: u32) -> Option<ParamDescriptor> {
        let reg = Self::registry();
        let id = *reg.keys().nth(index as usize)?;
        Self::param_descriptor_by_id(id)
    }

    fn param_descriptor_by_id(id: ParamId) -> Option<ParamDescriptor> {
        let desc = Self::descriptor_by_id(id)?;

        // Match main synth behavior: treat enums and ints as Float (stepped via is_stepped flag)
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
        let params = shared_params().lock();
        let denorm = voice_param_registry::get_param(id, &params)?;
        let desc = Self::descriptor_by_id(id)?;
        Some(desc.normalize_value(denorm))
    }

    fn set_param(id: ParamId, value: f32) {
        let mut params = shared_params().lock();
        if let Some(desc) = Self::descriptor_by_id(id) {
            let denorm = desc.denormalize(value);
            voice_param_registry::apply_param(id, denorm, &mut params);
        }
        PARAMS_DIRTY.store(true, Ordering::Release);
    }

    fn save_state() -> PluginState {
        let reg = Self::registry();
        let params = shared_params().lock();

        let mut state = PluginState {
            version: 1,
            ..Default::default()
        };

        for &id in reg.keys() {
            if let Some(desc) = Self::descriptor_by_id(id) {
                if let Some(denorm) = voice_param_registry::get_param(id, &params) {
                    let normalized = desc.normalize_value(denorm);
                    state.set_param(id, normalized);
                }
            }
        }

        state
    }

    fn load_state(state: &PluginState) {
        {
            let mut params = shared_params().lock();
            for (&id, &value) in state.params.iter() {
                if let Some(desc) = Self::descriptor_by_id(id) {
                    let denorm = desc.denormalize(value);
                    voice_param_registry::apply_param(id, denorm, &mut params);
                }
            }
        }
        PARAMS_DIRTY.store(true, Ordering::Release);
    }

    fn format_param(id: ParamId, value: f32) -> String {
        let Some(desc) = Self::descriptor_by_id(id) else {
            return format!("{:.2}", value);
        };

        // `value` is normalized 0..1
        desc.format_value(value)
    }

    fn parse_param(id: ParamId, text: &str) -> Option<f32> {
        let t = text.trim();

        if let Some(desc) = Self::descriptor_by_id(id) {
            match &desc.param_type {
                crate::plugin::param_descriptor::ParamType::Bool => {
                    if t.eq_ignore_ascii_case("on") || t == "1" {
                        return Some(1.0);
                    }
                    if t.eq_ignore_ascii_case("off") || t == "0" {
                        return Some(0.0);
                    }
                }
                crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                    if let Some((idx, _)) = variants
                        .iter()
                        .enumerate()
                        .find(|(_, v)| v.eq_ignore_ascii_case(t))
                    {
                        return Some(desc.normalize_value(idx as f32));
                    }
                }
                _ => {}
            }
        }

        // Fallback: parse leading number (strip any trailing unit).
        let number_str = t.split_whitespace().next().unwrap_or("");
        let plain = number_str.parse::<f32>().ok()?;
        Self::descriptor_by_id(id).map(|desc| desc.normalize_value(plain))
    }
}

// =============================================================================
// CLAP entry
// =============================================================================

generate_clap_entry!(DsynthVoicePlugin);
