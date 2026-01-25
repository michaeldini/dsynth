/// Voice Saturation Parameter Registry - MINIMAL ANALOG EMULATION
///
/// **Simplified parameter registry for analog vocal saturation with parallel processing**
///
/// Parameter namespace: 0x0300_xxxx (voice plugin)
///
/// Total Parameters: **5** (minimal analog design + mix)
/// 1. Input Gain: -12 to +12 dB
/// 2. Saturation Character: Warm/Smooth/Punchy (Int 0-2)
/// 3. Saturation Drive: 0.0-1.0 (calibrated for transparent enhancement)
/// 4. Saturation Mix: 0.0-1.0 (dry/wet blend, 0.3-0.5 optimal for vocals)
/// 5. Output Gain: -12 to +12 dB
use crate::params_voice::VoiceParams;
use crate::plugin::param_descriptor::{ParamDescriptor, ParamId};
use std::collections::HashMap;
use std::sync::OnceLock;

// ============================================================================
// PARAMETER IDs (Namespace: 0x0300_xxxx)
// ============================================================================

pub const PARAM_VOICE_INPUT_GAIN: ParamId = 0x0300_0001;
pub const PARAM_VOICE_SATURATION_CHARACTER: ParamId = 0x0300_0002;
pub const PARAM_VOICE_SATURATION_DRIVE: ParamId = 0x0300_0003;
pub const PARAM_VOICE_SATURATION_MIX: ParamId = 0x0300_0004;
pub const PARAM_VOICE_OUTPUT_GAIN: ParamId = 0x0300_0005;

// ============================================================================
// PARAMETER REGISTRY
// ============================================================================

static VOICE_PARAM_REGISTRY: OnceLock<HashMap<ParamId, ParamDescriptor>> = OnceLock::new();

pub fn get_voice_param_registry() -> &'static HashMap<ParamId, ParamDescriptor> {
    VOICE_PARAM_REGISTRY.get_or_init(|| {
        let mut registry = HashMap::new();

        // Input Gain (-12 to +12 dB)
        registry.insert(
            PARAM_VOICE_INPUT_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_INPUT_GAIN,
                "Input Gain",
                "Input",
                -12.0,
                12.0,
                0.0,
                Some("dB"),
            ),
        );

        // Saturation Character (Warm/Smooth/Punchy as Int with 3 values)
        // NOTE: For CLAP plugin, this will be exposed as Float with is_stepped flag
        // in the actual CLAP wrapper. Here we just define it as Int.
        let character_descriptor = ParamDescriptor::int(
            PARAM_VOICE_SATURATION_CHARACTER,
            "Character",
            "Saturation",
            0,
            2,
            0, // Default: Warm
        );
        registry.insert(PARAM_VOICE_SATURATION_CHARACTER, character_descriptor);

        // Saturation Drive (0.0-1.0, logarithmic for smooth vocal control)
        let drive_descriptor = ParamDescriptor::float(
            PARAM_VOICE_SATURATION_DRIVE,
            "Drive",
            "Saturation",
            0.0,
            1.0,
            0.5, // Default 50% = moderate saturation
            Some("%"),
        );
        registry.insert(PARAM_VOICE_SATURATION_DRIVE, drive_descriptor);

        // Saturation Mix (0.0-1.0 dry/wet blend)
        let mix_descriptor = ParamDescriptor::float(
            PARAM_VOICE_SATURATION_MIX,
            "Mix",
            "Saturation",
            0.0,
            1.0,
            0.4, // Default 40% wet = transparent parallel saturation
            Some("%"),
        );
        registry.insert(PARAM_VOICE_SATURATION_MIX, mix_descriptor);

        // Output Gain (-12 to +12 dB)
        registry.insert(
            PARAM_VOICE_OUTPUT_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_OUTPUT_GAIN,
                "Output Gain",
                "Output",
                -12.0,
                12.0,
                0.0,
                Some("dB"),
            ),
        );

        registry
    })
}

/// Get parameter descriptor by ID
pub fn get_param_descriptor(param_id: ParamId) -> Option<&'static ParamDescriptor> {
    get_voice_param_registry().get(&param_id)
}

/// Apply parameter value to VoiceParams
pub fn apply_param(params: &mut VoiceParams, param_id: ParamId, value: f32) {
    match param_id {
        PARAM_VOICE_INPUT_GAIN => params.input_gain = value.clamp(-12.0, 12.0),
        PARAM_VOICE_SATURATION_CHARACTER => {
            params.saturation_character = value.round() as u8;
            params.saturation_character = params.saturation_character.clamp(0, 2);
        }
        PARAM_VOICE_SATURATION_DRIVE => params.saturation_drive = value.clamp(0.0, 1.0),
        PARAM_VOICE_SATURATION_MIX => params.saturation_mix = value.clamp(0.0, 1.0),
        PARAM_VOICE_OUTPUT_GAIN => params.output_gain = value.clamp(-12.0, 12.0),
        _ => {
            // Unknown parameter - ignore silently
        }
    }
}

/// Get parameter value from VoiceParams
pub fn get_param(params: &VoiceParams, param_id: ParamId) -> Option<f32> {
    match param_id {
        PARAM_VOICE_INPUT_GAIN => Some(params.input_gain),
        PARAM_VOICE_SATURATION_CHARACTER => Some(params.saturation_character as f32),
        PARAM_VOICE_SATURATION_DRIVE => Some(params.saturation_drive),
        PARAM_VOICE_SATURATION_MIX => Some(params.saturation_mix),
        PARAM_VOICE_OUTPUT_GAIN => Some(params.output_gain),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_initialized() {
        let registry = get_voice_param_registry();
        assert_eq!(registry.len(), 5); // 5 total parameters (added mix)
    }

    #[test]
    fn test_all_params_have_descriptors() {
        let registry = get_voice_param_registry();

        assert!(registry.contains_key(&PARAM_VOICE_INPUT_GAIN));
        assert!(registry.contains_key(&PARAM_VOICE_SATURATION_CHARACTER));
        assert!(registry.contains_key(&PARAM_VOICE_SATURATION_DRIVE));
        assert!(registry.contains_key(&PARAM_VOICE_OUTPUT_GAIN));
    }

    #[test]
    fn test_apply_and_get_param() {
        let mut params = VoiceParams::default();

        // Test applying parameters
        apply_param(&mut params, PARAM_VOICE_SATURATION_DRIVE, 0.7);
        assert_eq!(params.saturation_drive, 0.7);

        apply_param(&mut params, PARAM_VOICE_INPUT_GAIN, 3.0);
        assert_eq!(params.input_gain, 3.0);

        // Test getting parameters
        assert_eq!(get_param(&params, PARAM_VOICE_SATURATION_DRIVE), Some(0.7));
        assert_eq!(get_param(&params, PARAM_VOICE_INPUT_GAIN), Some(3.0));
    }

    #[test]
    fn test_character_param() {
        let mut params = VoiceParams::default();

        // Test character selection
        apply_param(&mut params, PARAM_VOICE_SATURATION_CHARACTER, 0.0);
        assert_eq!(params.saturation_character, 0); // Warm

        apply_param(&mut params, PARAM_VOICE_SATURATION_CHARACTER, 1.0);
        assert_eq!(params.saturation_character, 1); // Smooth

        apply_param(&mut params, PARAM_VOICE_SATURATION_CHARACTER, 2.0);
        assert_eq!(params.saturation_character, 2); // Punchy

        // Test clamping
        apply_param(&mut params, PARAM_VOICE_SATURATION_CHARACTER, 99.0);
        assert_eq!(params.saturation_character, 2); // Clamped to max
    }

    #[test]
    fn test_drive_param_clamping() {
        let mut params = VoiceParams::default();

        // Test clamping at boundaries
        apply_param(&mut params, PARAM_VOICE_SATURATION_DRIVE, -0.5);
        assert_eq!(params.saturation_drive, 0.0);

        apply_param(&mut params, PARAM_VOICE_SATURATION_DRIVE, 1.5);
        assert_eq!(params.saturation_drive, 1.0);
    }

    #[test]
    fn test_gain_param_clamping() {
        let mut params = VoiceParams::default();

        // Test input gain clamping
        apply_param(&mut params, PARAM_VOICE_INPUT_GAIN, -20.0);
        assert_eq!(params.input_gain, -12.0); // Clamped to min

        apply_param(&mut params, PARAM_VOICE_INPUT_GAIN, 20.0);
        assert_eq!(params.input_gain, 12.0); // Clamped to max

        // Test output gain clamping
        apply_param(&mut params, PARAM_VOICE_OUTPUT_GAIN, -20.0);
        assert_eq!(params.output_gain, -12.0);

        apply_param(&mut params, PARAM_VOICE_OUTPUT_GAIN, 20.0);
        assert_eq!(params.output_gain, 12.0);
    }

    #[test]
    fn test_character_descriptor_is_int() {
        let descriptor = get_param_descriptor(PARAM_VOICE_SATURATION_CHARACTER).unwrap();
        // Character should be Int type with 3 values (0, 1, 2)
        match &descriptor.param_type {
            crate::plugin::param_descriptor::ParamType::Int { min, max } => {
                assert_eq!(*min, 0);
                assert_eq!(*max, 2);
            }
            _ => panic!("Character parameter should be Int type"),
        }
    }
}
