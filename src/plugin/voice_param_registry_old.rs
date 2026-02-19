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

// Vocal Character (0x0300_0052)
pub const PARAM_VOICE_VOCAL_CHARACTER: ParamId = 0x0300_0052;

// Limiter (0x0300_0030-0x0300_0031)
pub const PARAM_VOICE_LIMITER_THRESHOLD: ParamId = 0x0300_0030;
pub const PARAM_VOICE_LIMITER_RELEASE: ParamId = 0x0300_0031;

pub const PARAM_VOICE_OUTPUT_GAIN: ParamId = 0x0300_000C;

// ============================================================================
// PARAMETER REGISTRY
// ============================================================================

static VOICE_PARAM_REGISTRY: OnceLock<IndexMap<ParamId, ParamDescriptor>> = OnceLock::new();

pub fn get_voice_param_registry() -> &'static IndexMap<ParamId, ParamDescriptor> {
    VOICE_PARAM_REGISTRY.get_or_init(|| {
        let mut registry = IndexMap::new();

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

        // Attack Enhancer
        registry.insert(
            PARAM_VOICE_TRANSIENT_ATTACK,
            ParamDescriptor::float(
                PARAM_VOICE_TRANSIENT_ATTACK,
                "Attack Enhance",
                "Attack",
                -1.0,
                1.0,
                0.0,
                None,
            ),
        );

        // De-Esser
        registry.insert(
            PARAM_VOICE_DE_ESSER_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_DE_ESSER_AMOUNT,
                "De-Esser Amount",
                "De-Esser",
                0.0,
                1.0,
                0.0,
                Some("%"),
            ),
        );
        registry.insert(
            PARAM_VOICE_DE_ESSER_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_DE_ESSER_THRESHOLD,
                "De-Esser Threshold",
                "De-Esser",
                0.0,
                1.0,
                0.6,
                Some("%"),
            ),
        );
        registry.insert(
            PARAM_VOICE_SIBILANCE_FREQUENCY,
            ParamDescriptor::float(
                PARAM_VOICE_SIBILANCE_FREQUENCY,
                "Sibilance Freq",
                "De-Esser",
                3000.0,
                10000.0,
                6500.0,
                Some("Hz"),
            ),
        );
        registry.insert(
            PARAM_VOICE_DE_ESSER_LISTEN_HF,
            ParamDescriptor::bool(
                PARAM_VOICE_DE_ESSER_LISTEN_HF,
                "De-Esser Listen",
                "De-Esser",
                false,
            ),
        );

        // Bass Band
        registry.insert(
            PARAM_VOICE_BASS_DRIVE,
            ParamDescriptor::float(
                PARAM_VOICE_BASS_DRIVE,
                "Bass Drive",
                "Bass",
                0.0,
                1.0,
                0.6,
                Some("%"),
            ),
        );
        registry.insert(
            PARAM_VOICE_BASS_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_BASS_MIX,
                "Bass Mix",
                "Bass",
                0.0,
                1.0,
                0.5,
                Some("%"),
            ),
        );

        // Mids Band
        registry.insert(
            PARAM_VOICE_MID_DRIVE,
            ParamDescriptor::float(
                PARAM_VOICE_MID_DRIVE,
                "Mid Drive",
                "Mids",
                0.0,
                1.0,
                0.5,
                Some("%"),
            ),
        );
        registry.insert(
            PARAM_VOICE_MID_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_MID_MIX,
                "Mid Mix",
                "Mids",
                0.0,
                1.0,
                0.4,
                Some("%"),
            ),
        );

        // Presence Band
        registry.insert(
            PARAM_VOICE_PRESENCE_DRIVE,
            ParamDescriptor::float(
                PARAM_VOICE_PRESENCE_DRIVE,
                "Presence Drive",
                "Presence",
                0.0,
                1.0,
                0.35,
                Some("%"),
            ),
        );
        registry.insert(
            PARAM_VOICE_PRESENCE_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_PRESENCE_MIX,
                "Presence Mix",
                "Presence",
                0.0,
                1.0,
                0.35,
                Some("%"),
            ),
        );

        // Air Band
        registry.insert(
            PARAM_VOICE_AIR_DRIVE,
            ParamDescriptor::float(
                PARAM_VOICE_AIR_DRIVE,
                "Air Drive",
                "Air",
                0.0,
                1.0,
                0.1,
                Some("%"),
            ),
        );
        registry.insert(
            PARAM_VOICE_AIR_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_AIR_MIX,
                "Air Mix",
                "Air",
                0.0,
                1.0,
                0.15,
                Some("%"),
            ),
        );
        // Global Mix
        registry.insert(
            PARAM_VOICE_GLOBAL_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_GLOBAL_MIX,
                "Global Mix",
                "Master",
                0.0,
                1.0,
                1.0,
                Some("%"),
            ),
        );
        // Character parameter for vocal shaping
        registry.insert(
            PARAM_VOICE_VOCAL_CHARACTER,
            ParamDescriptor::float(
                PARAM_VOICE_VOCAL_CHARACTER,
                "Character",
                "Tone",
                -1.0,
                1.0,
                0.1,
                None, // No unit (warm ← → bright)
            ),
        );
        // Limiter
        registry.insert(
            PARAM_VOICE_LIMITER_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_LIMITER_THRESHOLD,
                "Limiter Threshold",
                "Limiter",
                -20.0,
                0.0,
                -6.0,
                Some("dB"),
            ),
        );
        registry.insert(
            PARAM_VOICE_LIMITER_RELEASE,
            ParamDescriptor::float(
                PARAM_VOICE_LIMITER_RELEASE,
                "Limiter Release",
                "Limiter",
                50.0,
                500.0,
                200.0,
                Some("ms"),
            ),
        );

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
        PARAM_VOICE_TRANSIENT_ATTACK => params.transient_attack = value.clamp(-1.0, 1.0),
        PARAM_VOICE_DE_ESSER_AMOUNT => params.de_esser_amount = value.clamp(0.0, 1.0),
        PARAM_VOICE_DE_ESSER_THRESHOLD => params.de_esser_threshold = value.clamp(0.0, 1.0),
        PARAM_VOICE_SIBILANCE_FREQUENCY => {
            params.sibilance_frequency = value.clamp(3000.0, 10000.0)
        }
        PARAM_VOICE_DE_ESSER_LISTEN_HF => params.de_esser_listen_hf = value >= 0.5,
        PARAM_VOICE_BASS_DRIVE => params.bass_drive = value.clamp(0.0, 1.0),
        PARAM_VOICE_BASS_MIX => params.bass_mix = value.clamp(0.0, 1.0),
        PARAM_VOICE_MID_DRIVE => params.mid_drive = value.clamp(0.0, 1.0),
        PARAM_VOICE_MID_MIX => params.mid_mix = value.clamp(0.0, 1.0),
        PARAM_VOICE_PRESENCE_DRIVE => params.presence_drive = value.clamp(0.0, 1.0),
        PARAM_VOICE_PRESENCE_MIX => params.presence_mix = value.clamp(0.0, 1.0),
        PARAM_VOICE_AIR_DRIVE => params.air_drive = value.clamp(0.0, 1.0),
        PARAM_VOICE_AIR_MIX => params.air_mix = value.clamp(0.0, 1.0),
        PARAM_VOICE_GLOBAL_MIX => params.global_mix = value.clamp(0.0, 1.0),
        PARAM_VOICE_VOCAL_CHARACTER => params.vocal_character = value.clamp(-1.0, 1.0),
        PARAM_VOICE_LIMITER_THRESHOLD => params.limiter_threshold = value.clamp(-20.0, 0.0),
        PARAM_VOICE_LIMITER_RELEASE => params.limiter_release = value.clamp(50.0, 500.0),
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
        PARAM_VOICE_TRANSIENT_ATTACK => Some(params.transient_attack),
        PARAM_VOICE_DE_ESSER_AMOUNT => Some(params.de_esser_amount),
        PARAM_VOICE_DE_ESSER_THRESHOLD => Some(params.de_esser_threshold),
        PARAM_VOICE_SIBILANCE_FREQUENCY => Some(params.sibilance_frequency),
        PARAM_VOICE_DE_ESSER_LISTEN_HF => Some(if params.de_esser_listen_hf { 1.0 } else { 0.0 }),
        PARAM_VOICE_BASS_DRIVE => Some(params.bass_drive),
        PARAM_VOICE_BASS_MIX => Some(params.bass_mix),
        PARAM_VOICE_MID_DRIVE => Some(params.mid_drive),
        PARAM_VOICE_MID_MIX => Some(params.mid_mix),
        PARAM_VOICE_PRESENCE_DRIVE => Some(params.presence_drive),
        PARAM_VOICE_PRESENCE_MIX => Some(params.presence_mix),
        PARAM_VOICE_AIR_DRIVE => Some(params.air_drive),
        PARAM_VOICE_AIR_MIX => Some(params.air_mix),
        PARAM_VOICE_GLOBAL_MIX => Some(params.global_mix),
        PARAM_VOICE_VOCAL_CHARACTER => Some(params.vocal_character),
        PARAM_VOICE_LIMITER_THRESHOLD => Some(params.limiter_threshold),
        PARAM_VOICE_LIMITER_RELEASE => Some(params.limiter_release),
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
        assert_eq!(registry.len(), 19); // Updated for removed parameters
    }

    #[test]
    fn test_parameter_order_preserved() {
        // IndexMap preserves insertion order, so iteration should match registration order
        let registry = get_voice_param_registry();
        let param_ids: Vec<ParamId> = registry.keys().copied().collect();

        // Expected order based on insertion in registry initialization
        let expected_order = vec![
            PARAM_VOICE_INPUT_GAIN,
            PARAM_VOICE_TRANSIENT_ATTACK,
            PARAM_VOICE_DE_ESSER_AMOUNT,
            PARAM_VOICE_DE_ESSER_THRESHOLD,
            PARAM_VOICE_SIBILANCE_FREQUENCY,
            PARAM_VOICE_DE_ESSER_LISTEN_HF,
            PARAM_VOICE_BASS_DRIVE,
            PARAM_VOICE_BASS_MIX,
            PARAM_VOICE_MID_DRIVE,
            PARAM_VOICE_MID_MIX,
            PARAM_VOICE_PRESENCE_DRIVE,
            PARAM_VOICE_PRESENCE_MIX,
            PARAM_VOICE_AIR_DRIVE,
            PARAM_VOICE_AIR_MIX,
            PARAM_VOICE_GLOBAL_MIX,
            PARAM_VOICE_VOCAL_CHARACTER,
            PARAM_VOICE_LIMITER_THRESHOLD,
            PARAM_VOICE_LIMITER_RELEASE,
            PARAM_VOICE_OUTPUT_GAIN,
        ];

        assert_eq!(param_ids, expected_order, "Parameter order not preserved!");
    }

    #[test]
    fn test_all_params_have_descriptors() {
        let registry = get_voice_param_registry();

        assert!(registry.contains_key(&PARAM_VOICE_INPUT_GAIN));
        assert!(registry.contains_key(&PARAM_VOICE_DE_ESSER_AMOUNT));
        assert!(registry.contains_key(&PARAM_VOICE_DE_ESSER_THRESHOLD));
        assert!(registry.contains_key(&PARAM_VOICE_SIBILANCE_FREQUENCY));
        assert!(registry.contains_key(&PARAM_VOICE_DE_ESSER_LISTEN_HF));
        assert!(registry.contains_key(&PARAM_VOICE_BASS_DRIVE));
        assert!(registry.contains_key(&PARAM_VOICE_BASS_MIX));
        assert!(registry.contains_key(&PARAM_VOICE_MID_DRIVE));
        assert!(registry.contains_key(&PARAM_VOICE_MID_MIX));
        assert!(registry.contains_key(&PARAM_VOICE_PRESENCE_DRIVE));
        assert!(registry.contains_key(&PARAM_VOICE_PRESENCE_MIX));
        assert!(registry.contains_key(&PARAM_VOICE_AIR_DRIVE));
        assert!(registry.contains_key(&PARAM_VOICE_AIR_MIX));
        assert!(registry.contains_key(&PARAM_VOICE_GLOBAL_MIX));
        assert!(registry.contains_key(&PARAM_VOICE_OUTPUT_GAIN));
    }

    #[test]
    fn test_apply_and_get_param() {
        let mut params = VoiceParams::default();

        // Test applying parameters
        apply_param(&mut params, PARAM_VOICE_BASS_DRIVE, 0.7);
        assert_eq!(params.bass_drive, 0.7);

        apply_param(&mut params, PARAM_VOICE_MID_MIX, 0.3);
        assert_eq!(params.mid_mix, 0.3);

        apply_param(&mut params, PARAM_VOICE_INPUT_GAIN, 3.0);
        assert_eq!(params.input_gain, 3.0);

        apply_param(&mut params, PARAM_VOICE_DE_ESSER_AMOUNT, 0.7);
        assert_eq!(params.de_esser_amount, 0.7);

        apply_param(&mut params, PARAM_VOICE_DE_ESSER_LISTEN_HF, 1.0);
        assert!(params.de_esser_listen_hf);

        // Test character parameter
        apply_param(&mut params, PARAM_VOICE_VOCAL_CHARACTER, 0.5);
        assert_eq!(params.vocal_character, 0.5);

        // Test getting parameters
        assert_eq!(get_param(&params, PARAM_VOICE_BASS_DRIVE), Some(0.7));
        assert_eq!(get_param(&params, PARAM_VOICE_MID_MIX), Some(0.3));
        assert_eq!(get_param(&params, PARAM_VOICE_INPUT_GAIN), Some(3.0));
        assert_eq!(get_param(&params, PARAM_VOICE_DE_ESSER_AMOUNT), Some(0.7));
        assert_eq!(
            get_param(&params, PARAM_VOICE_DE_ESSER_LISTEN_HF),
            Some(1.0)
        );
        assert_eq!(get_param(&params, PARAM_VOICE_VOCAL_CHARACTER), Some(0.5));
    }

    #[test]
    fn test_drive_param_clamping() {
        let mut params = VoiceParams::default();

        // Test clamping at boundaries for all drive parameters
        apply_param(&mut params, PARAM_VOICE_BASS_DRIVE, -0.5);
        assert_eq!(params.bass_drive, 0.0);

        apply_param(&mut params, PARAM_VOICE_MID_DRIVE, 1.5);
        assert_eq!(params.mid_drive, 1.0);

        apply_param(&mut params, PARAM_VOICE_PRESENCE_DRIVE, 0.5);
        assert_eq!(params.presence_drive, 0.5);
    }

    #[test]
    fn test_mix_param_clamping() {
        let mut params = VoiceParams::default();

        // Test clamping for mix parameters
        apply_param(&mut params, PARAM_VOICE_BASS_MIX, -0.1);
        assert_eq!(params.bass_mix, 0.0);

        apply_param(&mut params, PARAM_VOICE_MID_MIX, 1.2);
        assert_eq!(params.mid_mix, 1.0);

        apply_param(&mut params, PARAM_VOICE_PRESENCE_MIX, 0.7);
        assert_eq!(params.presence_mix, 0.7);
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
}
