/// Parameter Registry for Kick Drum CLAP Plugin
///
/// Simplified registry for kick drum parameters with CLAP metadata.
use super::param_descriptor::*;
use crate::params_kick::{DistortionType, KickParams};
use std::collections::HashMap;
use std::sync::OnceLock;

// Re-export ParamId for public use
pub use super::param_descriptor::ParamId;

/// Global kick parameter registry
static KICK_REGISTRY: OnceLock<KickParamRegistry> = OnceLock::new();

/// Get the global kick parameter registry
pub fn get_kick_registry() -> &'static KickParamRegistry {
    KICK_REGISTRY.get_or_init(KickParamRegistry::new)
}

// Parameter IDs for kick synth (using unique namespace: 0x0200_0000)
pub const PARAM_KICK_OSC1_PITCH_START: ParamId = 0x0200_0001;
pub const PARAM_KICK_OSC1_PITCH_END: ParamId = 0x0200_0002;
pub const PARAM_KICK_OSC1_PITCH_DECAY: ParamId = 0x0200_0003;
pub const PARAM_KICK_OSC1_LEVEL: ParamId = 0x0200_0004;

pub const PARAM_KICK_OSC2_PITCH_START: ParamId = 0x0200_0010;
pub const PARAM_KICK_OSC2_PITCH_END: ParamId = 0x0200_0011;
pub const PARAM_KICK_OSC2_PITCH_DECAY: ParamId = 0x0200_0012;
pub const PARAM_KICK_OSC2_LEVEL: ParamId = 0x0200_0013;

pub const PARAM_KICK_AMP_ATTACK: ParamId = 0x0200_0020;
pub const PARAM_KICK_AMP_DECAY: ParamId = 0x0200_0021;
pub const PARAM_KICK_AMP_SUSTAIN: ParamId = 0x0200_0022;
pub const PARAM_KICK_AMP_RELEASE: ParamId = 0x0200_0023;

pub const PARAM_KICK_FILTER_CUTOFF: ParamId = 0x0200_0030;
pub const PARAM_KICK_FILTER_RESONANCE: ParamId = 0x0200_0031;
pub const PARAM_KICK_FILTER_ENV_AMOUNT: ParamId = 0x0200_0032;
pub const PARAM_KICK_FILTER_ENV_DECAY: ParamId = 0x0200_0033;

pub const PARAM_KICK_DISTORTION_AMOUNT: ParamId = 0x0200_0040;
pub const PARAM_KICK_DISTORTION_TYPE: ParamId = 0x0200_0041;

pub const PARAM_KICK_MASTER_LEVEL: ParamId = 0x0200_0050;
pub const PARAM_KICK_VELOCITY_SENSITIVITY: ParamId = 0x0200_0051;

/// Parameter registry for kick drum synth
pub struct KickParamRegistry {
    descriptors: HashMap<ParamId, ParamDescriptor>,
    param_ids: Vec<ParamId>,
}

impl KickParamRegistry {
    pub fn new() -> Self {
        let mut descriptors = HashMap::new();
        let mut param_ids = Vec::new();

        macro_rules! add_param {
            ($id:expr, $desc:expr) => {
                descriptors.insert($id, $desc);
                param_ids.push($id);
            };
        }

        // Body Oscillator (Osc 1)
        add_param!(
            PARAM_KICK_OSC1_PITCH_START,
            ParamDescriptor::float_log(
                PARAM_KICK_OSC1_PITCH_START,
                "Start Pitch",
                "Body Osc",
                40.0,
                500.0,
                150.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_KICK_OSC1_PITCH_END,
            ParamDescriptor::float_log(
                PARAM_KICK_OSC1_PITCH_END,
                "End Pitch",
                "Body Osc",
                30.0,
                200.0,
                55.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_KICK_OSC1_PITCH_DECAY,
            ParamDescriptor::float_log(
                PARAM_KICK_OSC1_PITCH_DECAY,
                "Pitch Decay",
                "Body Osc",
                10.0,
                500.0,
                100.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_KICK_OSC1_LEVEL,
            ParamDescriptor::float(
                PARAM_KICK_OSC1_LEVEL,
                "Level",
                "Body Osc",
                0.0,
                1.0,
                0.8,
                Some("%")
            )
        );

        // Click Oscillator (Osc 2)
        add_param!(
            PARAM_KICK_OSC2_PITCH_START,
            ParamDescriptor::float_log(
                PARAM_KICK_OSC2_PITCH_START,
                "Start Pitch",
                "Click Osc",
                100.0,
                2000.0,
                1000.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_KICK_OSC2_PITCH_END,
            ParamDescriptor::float_log(
                PARAM_KICK_OSC2_PITCH_END,
                "End Pitch",
                "Click Osc",
                50.0,
                1000.0,
                200.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_KICK_OSC2_PITCH_DECAY,
            ParamDescriptor::float_log(
                PARAM_KICK_OSC2_PITCH_DECAY,
                "Pitch Decay",
                "Click Osc",
                1.0,
                100.0,
                20.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_KICK_OSC2_LEVEL,
            ParamDescriptor::float(
                PARAM_KICK_OSC2_LEVEL,
                "Level",
                "Click Osc",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );

        // Amplitude Envelope
        add_param!(
            PARAM_KICK_AMP_ATTACK,
            ParamDescriptor::float_log(
                PARAM_KICK_AMP_ATTACK,
                "Attack",
                "Envelope",
                0.1,
                100.0,
                1.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_KICK_AMP_DECAY,
            ParamDescriptor::float_log(
                PARAM_KICK_AMP_DECAY,
                "Decay",
                "Envelope",
                50.0,
                2000.0,
                500.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_KICK_AMP_SUSTAIN,
            ParamDescriptor::float(
                PARAM_KICK_AMP_SUSTAIN,
                "Sustain",
                "Envelope",
                0.0,
                1.0,
                0.0,
                Some("%")
            )
        );
        add_param!(
            PARAM_KICK_AMP_RELEASE,
            ParamDescriptor::float_log(
                PARAM_KICK_AMP_RELEASE,
                "Release",
                "Envelope",
                10.0,
                500.0,
                50.0,
                Some("ms")
            )
        );

        // Filter
        add_param!(
            PARAM_KICK_FILTER_CUTOFF,
            ParamDescriptor::float_log(
                PARAM_KICK_FILTER_CUTOFF,
                "Cutoff",
                "Filter",
                20.0,
                20000.0,
                5000.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_KICK_FILTER_RESONANCE,
            ParamDescriptor::float(
                PARAM_KICK_FILTER_RESONANCE,
                "Resonance",
                "Filter",
                0.0,
                1.0,
                0.2,
                None
            )
        );
        add_param!(
            PARAM_KICK_FILTER_ENV_AMOUNT,
            ParamDescriptor::float(
                PARAM_KICK_FILTER_ENV_AMOUNT,
                "Env Amount",
                "Filter",
                -1.0,
                1.0,
                0.3,
                None
            )
        );
        add_param!(
            PARAM_KICK_FILTER_ENV_DECAY,
            ParamDescriptor::float_log(
                PARAM_KICK_FILTER_ENV_DECAY,
                "Env Decay",
                "Filter",
                10.0,
                2000.0,
                300.0,
                Some("ms")
            )
        );

        // Distortion
        add_param!(
            PARAM_KICK_DISTORTION_AMOUNT,
            ParamDescriptor::float(
                PARAM_KICK_DISTORTION_AMOUNT,
                "Amount",
                "Distortion",
                0.0,
                1.0,
                0.15,
                Some("%")
            )
        );
        add_param!(
            PARAM_KICK_DISTORTION_TYPE,
            ParamDescriptor::enum_param(
                PARAM_KICK_DISTORTION_TYPE,
                "Type",
                "Distortion",
                vec!["Soft".into(), "Hard".into(), "Tube".into(), "Foldback".into()],
                0
            )
        );

        // Master
        add_param!(
            PARAM_KICK_MASTER_LEVEL,
            ParamDescriptor::float(
                PARAM_KICK_MASTER_LEVEL,
                "Master Level",
                "Master",
                0.0,
                1.0,
                0.8,
                Some("%")
            )
        );
        add_param!(
            PARAM_KICK_VELOCITY_SENSITIVITY,
            ParamDescriptor::float(
                PARAM_KICK_VELOCITY_SENSITIVITY,
                "Velocity",
                "Master",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );

        Self {
            descriptors,
            param_ids,
        }
    }

    pub fn get_descriptor(&self, id: ParamId) -> Option<&ParamDescriptor> {
        self.descriptors.get(&id)
    }

    pub fn param_count(&self) -> usize {
        self.param_ids.len()
    }

    pub fn param_ids(&self) -> &[ParamId] {
        &self.param_ids
    }

    /// Normalize a parameter value from internal range to 0.0-1.0
    pub fn normalize_value(&self, id: ParamId, value: f64) -> f64 {
        if let Some(desc) = self.get_descriptor(id) {
            desc.normalize_value(value as f32) as f64
        } else {
            0.0
        }
    }

    /// Denormalize a parameter value from 0.0-1.0 to internal range
    pub fn denormalize_value(&self, id: ParamId, normalized: f64) -> f64 {
        if let Some(desc) = self.get_descriptor(id) {
            desc.denormalize(normalized as f32) as f64
        } else {
            0.0
        }
    }

    /// Apply parameter change to KickParams
    pub fn apply_param(&self, params: &mut KickParams, id: ParamId, normalized: f64) {
        let value = self.denormalize_value(id, normalized);

        match id {
            PARAM_KICK_OSC1_PITCH_START => params.osc1_pitch_start = value as f32,
            PARAM_KICK_OSC1_PITCH_END => params.osc1_pitch_end = value as f32,
            PARAM_KICK_OSC1_PITCH_DECAY => params.osc1_pitch_decay = value as f32,
            PARAM_KICK_OSC1_LEVEL => params.osc1_level = value as f32,

            PARAM_KICK_OSC2_PITCH_START => params.osc2_pitch_start = value as f32,
            PARAM_KICK_OSC2_PITCH_END => params.osc2_pitch_end = value as f32,
            PARAM_KICK_OSC2_PITCH_DECAY => params.osc2_pitch_decay = value as f32,
            PARAM_KICK_OSC2_LEVEL => params.osc2_level = value as f32,

            PARAM_KICK_AMP_ATTACK => params.amp_attack = value as f32,
            PARAM_KICK_AMP_DECAY => params.amp_decay = value as f32,
            PARAM_KICK_AMP_SUSTAIN => params.amp_sustain = value as f32,
            PARAM_KICK_AMP_RELEASE => params.amp_release = value as f32,

            PARAM_KICK_FILTER_CUTOFF => params.filter_cutoff = value as f32,
            PARAM_KICK_FILTER_RESONANCE => params.filter_resonance = value as f32,
            PARAM_KICK_FILTER_ENV_AMOUNT => params.filter_env_amount = value as f32,
            PARAM_KICK_FILTER_ENV_DECAY => params.filter_env_decay = value as f32,

            PARAM_KICK_DISTORTION_AMOUNT => params.distortion_amount = value as f32,
            PARAM_KICK_DISTORTION_TYPE => {
                params.distortion_type = match value as usize {
                    0 => DistortionType::Soft,
                    1 => DistortionType::Hard,
                    2 => DistortionType::Tube,
                    3 => DistortionType::Foldback,
                    _ => DistortionType::Soft,
                };
            }

            PARAM_KICK_MASTER_LEVEL => params.master_level = value as f32,
            PARAM_KICK_VELOCITY_SENSITIVITY => params.velocity_sensitivity = value as f32,

            _ => {}
        }
    }

    /// Get normalized parameter value from KickParams
    pub fn get_param(&self, params: &KickParams, id: ParamId) -> f64 {
        let value: f64 = match id {
            PARAM_KICK_OSC1_PITCH_START => params.osc1_pitch_start as f64,
            PARAM_KICK_OSC1_PITCH_END => params.osc1_pitch_end as f64,
            PARAM_KICK_OSC1_PITCH_DECAY => params.osc1_pitch_decay as f64,
            PARAM_KICK_OSC1_LEVEL => params.osc1_level as f64,

            PARAM_KICK_OSC2_PITCH_START => params.osc2_pitch_start as f64,
            PARAM_KICK_OSC2_PITCH_END => params.osc2_pitch_end as f64,
            PARAM_KICK_OSC2_PITCH_DECAY => params.osc2_pitch_decay as f64,
            PARAM_KICK_OSC2_LEVEL => params.osc2_level as f64,

            PARAM_KICK_AMP_ATTACK => params.amp_attack as f64,
            PARAM_KICK_AMP_DECAY => params.amp_decay as f64,
            PARAM_KICK_AMP_SUSTAIN => params.amp_sustain as f64,
            PARAM_KICK_AMP_RELEASE => params.amp_release as f64,

            PARAM_KICK_FILTER_CUTOFF => params.filter_cutoff as f64,
            PARAM_KICK_FILTER_RESONANCE => params.filter_resonance as f64,
            PARAM_KICK_FILTER_ENV_AMOUNT => params.filter_env_amount as f64,
            PARAM_KICK_FILTER_ENV_DECAY => params.filter_env_decay as f64,

            PARAM_KICK_DISTORTION_AMOUNT => params.distortion_amount as f64,
            PARAM_KICK_DISTORTION_TYPE => match params.distortion_type {
                DistortionType::Soft => 0.0,
                DistortionType::Hard => 1.0,
                DistortionType::Tube => 2.0,
                DistortionType::Foldback => 3.0,
            },

            PARAM_KICK_MASTER_LEVEL => params.master_level as f64,
            PARAM_KICK_VELOCITY_SENSITIVITY => params.velocity_sensitivity as f64,

            _ => 0.0,
        };

        self.normalize_value(id, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = get_kick_registry();
        assert!(registry.param_count() > 0);
        assert_eq!(registry.param_count(), registry.param_ids().len());
    }

    #[test]
    fn test_normalize_denormalize() {
        let registry = get_kick_registry();
        let test_value = 150.0; // Osc1 start pitch
        
        let normalized = registry.normalize_value(PARAM_KICK_OSC1_PITCH_START, test_value);
        let denormalized = registry.denormalize_value(PARAM_KICK_OSC1_PITCH_START, normalized);
        
        assert!((denormalized - test_value).abs() < 0.1);
    }

    #[test]
    fn test_apply_and_get_param() {
        let registry = get_kick_registry();
        let mut params = KickParams::default();
        
        // Set osc1 pitch start to 200Hz (normalized)
        let target_value = 200.0;
        let normalized = registry.normalize_value(PARAM_KICK_OSC1_PITCH_START, target_value);
        registry.apply_param(&mut params, PARAM_KICK_OSC1_PITCH_START, normalized);
        
        assert!((params.osc1_pitch_start - target_value as f32).abs() < 0.1);
        
        // Verify we can read it back
        let read_normalized = registry.get_param(&params, PARAM_KICK_OSC1_PITCH_START);
        assert!((read_normalized - normalized).abs() < 0.01);
    }

    #[test]
    fn test_distortion_type_enum() {
        let registry = get_kick_registry();
        let mut params = KickParams::default();
        
        // Set to Hard distortion (index 1)
        registry.apply_param(&mut params, PARAM_KICK_DISTORTION_TYPE, 0.33); // ~1/3 = index 1
        assert_eq!(params.distortion_type, DistortionType::Hard);
        
        // Verify readback
        let normalized = registry.get_param(&params, PARAM_KICK_DISTORTION_TYPE);
        assert!((normalized - 0.25).abs() < 0.1); // Index 1 of 4 options = 0.25 normalized
    }
}
