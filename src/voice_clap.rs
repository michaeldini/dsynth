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
}

impl ClapProcessor for DsynthVoiceProcessor {
    fn process(
        &mut self,
        audio: &mut dsynth_clap::AudioBuffers,
        _events: &Events,
    ) -> ProcessStatus {
        self.sync_params_if_dirty();

        let frames = audio.frames_count() as usize;

        unsafe {
            // Voice is an effect: requires stereo input and produces stereo output.
            let Some((in_l, in_r, out_l, out_r)) = audio.io_stereo_mut(0, 0) else {
                self.engine.reset();
                return ProcessStatus::Sleep;
            };

            // Defensive: hosts should provide matching sizes, but keep safe.
            let n = frames
                .min(in_l.len())
                .min(in_r.len())
                .min(out_l.len())
                .min(out_r.len());

            self.engine.process_buffer(in_l, in_r, out_l, out_r, n);

            // If host gave more frames than we processed, clear remainder.
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
// Parameters (denormalized values, normalized by dsynth-clap ParamDescriptor)
// =============================================================================

pub struct DsynthVoiceParams;

impl DsynthVoiceParams {
    fn registry() -> &'static voice_param_registry::VoiceParamRegistry {
        voice_param_registry::get_voice_registry()
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

        let param_type = match &desc.param_type {
            crate::plugin::param_descriptor::ParamType::Float { min, max, .. } => {
                ParamType::Float {
                    min: *min,
                    max: *max,
                    default: desc.default,
                }
            }
            crate::plugin::param_descriptor::ParamType::Bool => ParamType::Bool {
                default: desc.default > 0.5,
            },
            crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                let default_idx = desc.default.round().max(0.0) as usize;
                ParamType::Enum {
                    variants: variants.clone(),
                    default: default_idx.min(variants.len().saturating_sub(1)),
                }
            }
            crate::plugin::param_descriptor::ParamType::Int { min, max } => ParamType::Int {
                min: *min,
                max: *max,
                default: desc.default.round() as i32,
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
        voice_param_registry::get_param(&params, id)
    }

    fn set_param(id: ParamId, value: f32) {
        let mut params = shared_params().lock();
        voice_param_registry::apply_param(&mut params, id, value);
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
            if let Some(value) = voice_param_registry::get_param(&params, id) {
                state.set_param(id, value);
            }
        }

        state
    }

    fn load_state(state: &PluginState) {
        {
            let mut params = shared_params().lock();
            for (&id, &value) in state.params.iter() {
                voice_param_registry::apply_param(&mut params, id, value);
            }
        }
        PARAMS_DIRTY.store(true, Ordering::Release);
    }

    fn format_param(id: ParamId, value: f32) -> String {
        let Some(desc) = Self::descriptor_by_id(id) else {
            return format!("{:.2}", value);
        };

        match &desc.param_type {
            crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                let idx = value.round().max(0.0) as usize;
                variants
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| format!("{}", idx))
            }
            crate::plugin::param_descriptor::ParamType::Bool => {
                if value > 0.5 {
                    "On".to_string()
                } else {
                    "Off".to_string()
                }
            }
            _ => {
                if let Some(unit) = &desc.unit {
                    format!("{:.2} {}", value, unit)
                } else {
                    format!("{:.2}", value)
                }
            }
        }
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
                        return Some(idx as f32);
                    }
                }
                _ => {}
            }
        }

        // Fallback: parse leading number (strip any trailing unit).
        let number_str = t.split_whitespace().next().unwrap_or("");
        number_str.parse::<f32>().ok()
    }
}

// =============================================================================
// CLAP entry
// =============================================================================

generate_clap_entry!(DsynthVoicePlugin);
