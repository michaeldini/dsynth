/// Voice Processing Parameter Registry - PERCEPTUAL VOCAL ENHANCEMENT
///
/// **Parameter registry for perceptual voice processor**
///
/// Parameter namespace: 0x0300_xxxx (voice plugin)
///
/// Total Parameters: **7 perceptual controls**
/// 1. Character: -1 to +1 (Warm ← → Bright)
/// 2. Intensity: 0 to 1 (Gentle ← → Aggressive) 
/// 3. Presence: -1 to +1 (Distant ← → Intimate)
/// 4. Dynamics: -1 to +1 (Compressed ← → Dynamic)
/// 5. Input Gain: -12 to +12 dB
/// 6. Output Gain: -12 to +12 dB
/// 7. Dry/Wet Mix: 0 to 1
use crate::params_voice::VoiceParams;
use crate::plugin::param_descriptor::{ParamDescriptor, ParamId};
use indexmap::IndexMap;
use std::sync::OnceLock;

// ============================================================================
// PARAMETER IDs (Namespace: 0x0300_xxxx) 
// ============================================================================

// Perceptual Controls (0x0300_0001-0x0300_0004)
pub const PARAM_VOICE_CHARACTER: ParamId = 0x0300_0001;
pub const PARAM_VOICE_INTENSITY: ParamId = 0x0300_0002;
pub const PARAM_VOICE_PRESENCE: ParamId = 0x0300_0003;
pub const PARAM_VOICE_DYNAMICS: ParamId = 0x0300_0004;

// I/O Controls (0x0300_0005-0x0300_0007)
pub const PARAM_VOICE_INPUT_GAIN: ParamId = 0x0300_0005;
pub const PARAM_VOICE_OUTPUT_GAIN: ParamId = 0x0300_0006;
pub const PARAM_VOICE_DRY_WET_MIX: ParamId = 0x0300_0007;

// ============================================================================
// PARAMETER REGISTRY
// ============================================================================

static VOICE_PARAM_REGISTRY: OnceLock<IndexMap<ParamId, ParamDescriptor>> = OnceLock::new();

pub fn get_voice_param_registry() -> &'static IndexMap<ParamId, ParamDescriptor> {
    VOICE_PARAM_REGISTRY.get_or_init(|| {
        let mut registry = IndexMap::new();

        // Perceptual Controls
        registry.insert(
            PARAM_VOICE_CHARACTER,
            ParamDescriptor::float(
                PARAM_VOICE_CHARACTER,
                "Character",
                "Character",
                -1.0,
                1.0,
                0.2,
                None,
            ),
        );

        registry.insert(
            PARAM_VOICE_INTENSITY,
            ParamDescriptor::float(
                PARAM_VOICE_INTENSITY,
                "Intensity",
                "Character", 
                0.0,
                1.0,
                0.4,
                Some("%"),
            ),
        );

        registry.insert(
            PARAM_VOICE_PRESENCE,
            ParamDescriptor::float(
                PARAM_VOICE_PRESENCE,
                "Presence",
                "Character",
                -1.0,
                1.0,
                0.3,
                None,
            ),
        );

        registry.insert(
            PARAM_VOICE_DYNAMICS,
            ParamDescriptor::float(
                PARAM_VOICE_DYNAMICS,
                "Dynamics",
                "Character",
                -1.0,
                1.0,
                0.1,
                None,
            ),
        );

        // I/O Controls
        registry.insert(
            PARAM_VOICE_INPUT_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_INPUT_GAIN,
                "Input Gain",
                "I/O",
                -12.0,
                12.0,
                0.0,
                Some("dB"),
            ),
        );

        registry.insert(
            PARAM_VOICE_OUTPUT_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_OUTPUT_GAIN,
                "Output Gain",
                "I/O",
                -12.0,
                12.0,
                0.0,
                Some("dB"),
            ),
        );

        registry.insert(
            PARAM_VOICE_DRY_WET_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_DRY_WET_MIX,
                "Dry/Wet",
                "I/O",
                0.0,
                1.0,
                1.0,
                Some("%"),
            ),
        );

        registry
    })
}

// ============================================================================
// PARAMETER ACCESS FUNCTIONS
// ============================================================================

/// Get parameter descriptor by ID
pub fn get_param_descriptor(param_id: ParamId) -> Option<&'static ParamDescriptor> {
    get_voice_param_registry().get(&param_id)
}

/// Apply normalized parameter value to VoiceParams
pub fn apply_param(param_id: ParamId, value: f32, params: &mut VoiceParams) {
    match param_id {
        PARAM_VOICE_CHARACTER => params.character = value.clamp(-1.0, 1.0),
        PARAM_VOICE_INTENSITY => params.intensity = value.clamp(0.0, 1.0),
        PARAM_VOICE_PRESENCE => params.presence = value.clamp(-1.0, 1.0),
        PARAM_VOICE_DYNAMICS => params.dynamics = value.clamp(-1.0, 1.0),
        PARAM_VOICE_INPUT_GAIN => params.input_gain = value.clamp(-12.0, 12.0),
        PARAM_VOICE_OUTPUT_GAIN => params.output_gain = value.clamp(-12.0, 12.0),
        PARAM_VOICE_DRY_WET_MIX => params.dry_wet_mix = value.clamp(0.0, 1.0),
        _ => {} // Unknown parameter
    }
}

/// Get normalized parameter value from VoiceParams
pub fn get_param(param_id: ParamId, params: &VoiceParams) -> Option<f32> {
    match param_id {
        PARAM_VOICE_CHARACTER => Some(params.character),
        PARAM_VOICE_INTENSITY => Some(params.intensity),
        PARAM_VOICE_PRESENCE => Some(params.presence),
        PARAM_VOICE_DYNAMICS => Some(params.dynamics),
        PARAM_VOICE_INPUT_GAIN => Some(params.input_gain),
        PARAM_VOICE_OUTPUT_GAIN => Some(params.output_gain),
        PARAM_VOICE_DRY_WET_MIX => Some(params.dry_wet_mix),
        _ => None, // Unknown parameter
    }
}

/// Count of voice parameters
pub fn voice_param_count() -> u32 {
    get_voice_param_registry().len() as u32
}

/// Get voice parameter info by index
pub fn voice_param_info(index: u32) -> Option<(&'static ParamId, &'static ParamDescriptor)> {
    get_voice_param_registry()
        .iter()
        .nth(index as usize)
}

/// Get all voice parameter IDs
pub fn voice_param_ids() -> Vec<ParamId> {
    get_voice_param_registry().keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_count() {
        assert_eq!(voice_param_count(), 7);
    }

    #[test]
    fn test_param_registry() {
        let registry = get_voice_param_registry();
        assert_eq!(registry.len(), 7);
        
        // Verify all parameter IDs exist
        assert!(registry.contains_key(&PARAM_VOICE_CHARACTER));
        assert!(registry.contains_key(&PARAM_VOICE_INTENSITY));
        assert!(registry.contains_key(&PARAM_VOICE_PRESENCE));
        assert!(registry.contains_key(&PARAM_VOICE_DYNAMICS));
        assert!(registry.contains_key(&PARAM_VOICE_INPUT_GAIN));
        assert!(registry.contains_key(&PARAM_VOICE_OUTPUT_GAIN));
        assert!(registry.contains_key(&PARAM_VOICE_DRY_WET_MIX));
    }

    #[test]
    fn test_apply_param() {
        let mut params = VoiceParams::default();
        
        apply_param(PARAM_VOICE_CHARACTER, 0.7, &mut params);
        assert_eq!(params.character, 0.7);
        
        apply_param(PARAM_VOICE_INTENSITY, 0.8, &mut params);
        assert_eq!(params.intensity, 0.8);
        
        apply_param(PARAM_VOICE_INPUT_GAIN, 6.0, &mut params);
        assert_eq!(params.input_gain, 6.0);
    }

    #[test]
    fn test_get_param() {
        let mut params = VoiceParams::default();
        params.character = 0.5;
        params.intensity = 0.3;
        
        assert_eq!(get_param(PARAM_VOICE_CHARACTER, &params), Some(0.5));
        assert_eq!(get_param(PARAM_VOICE_INTENSITY, &params), Some(0.3));
        assert_eq!(get_param(9999, &params), None); // Invalid ID
    }

    #[test]
    fn test_param_clamping() {
        let mut params = VoiceParams::default();
        
        // Test clamping
        apply_param(PARAM_VOICE_CHARACTER, 2.0, &mut params); // Should clamp to 1.0
        assert_eq!(params.character, 1.0);
        
        apply_param(PARAM_VOICE_INTENSITY, -0.5, &mut params); // Should clamp to 0.0
        assert_eq!(params.intensity, 0.0);
    }

    #[test]
    fn test_param_descriptors() {
        // Test character parameter
        let char_desc = get_param_descriptor(PARAM_VOICE_CHARACTER).unwrap();
        assert_eq!(char_desc.name, "Character");
        assert_eq!(char_desc.module, "Character");
        
        // Test intensity parameter  
        let intensity_desc = get_param_descriptor(PARAM_VOICE_INTENSITY).unwrap();
        assert_eq!(intensity_desc.name, "Intensity");
        assert_eq!(intensity_desc.unit, Some("%".to_string()));
    }
}