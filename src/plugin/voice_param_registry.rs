/// Voice Enhancer Parameter Registry - INTELLIGENT ARCHITECTURE v2.0
///
/// **Simplified parameter registry for intelligent voice enhancement**
///
/// Parameter namespace: 0x0300_xxxx (voice enhancer)
///
/// Total Parameters: **20** (optimized for music vocals)
/// Organized sequentially for optimal DAW display:
/// 1. Input: 1 param (Input Gain)
/// 2. Analysis: 1 param (Pitch Confidence)
/// 3. Gate: 2 params (Enable + Threshold)
/// 4. Compressor: 5 params (Enable + Threshold/Ratio/Attack/Release)
/// 5. Exciter: 3 params (Enable + Amount/Mix)
/// 6. De-Esser: 2 params (Enable + Amount)
/// 7. Smart Delay: 4 params (Enable + Time/Feedback/Mix)
/// 8. Master: 2 params (Dry/Wet + Output Gain)
use crate::params_voice::VoiceParams;
use crate::plugin::param_descriptor::{ParamDescriptor, ParamId};
use std::collections::HashMap;
use std::sync::OnceLock;

// ============================================================================
// PARAMETER IDs (Namespace: 0x0300_xxxx)
// ============================================================================
// DAWs display parameters in sequential order, so we organize them logically:
// Input → Analysis → Gate → Compressor → Exciter → Master

// 1. Input Section (0x0300_0001)
pub const PARAM_VOICE_INPUT_GAIN: ParamId = 0x0300_0001;

// 2. Analysis Section (0x0300_0002)
pub const PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD: ParamId = 0x0300_0002;

// 3. Smart Gate Section (0x0300_0003 - 0x0300_0004)
pub const PARAM_VOICE_GATE_ENABLE: ParamId = 0x0300_0003;
pub const PARAM_VOICE_GATE_THRESHOLD: ParamId = 0x0300_0004;

// 4. Adaptive Compressor Section (0x0300_0005 - 0x0300_0009)
pub const PARAM_VOICE_COMP_ENABLE: ParamId = 0x0300_0005;
pub const PARAM_VOICE_COMP_THRESHOLD: ParamId = 0x0300_0006;
pub const PARAM_VOICE_COMP_RATIO: ParamId = 0x0300_0007;
pub const PARAM_VOICE_COMP_ATTACK: ParamId = 0x0300_0008;
pub const PARAM_VOICE_COMP_RELEASE: ParamId = 0x0300_0009;

// 5. Intelligent Exciter Section (0x0300_000A - 0x0300_000C)
pub const PARAM_VOICE_EXCITER_ENABLE: ParamId = 0x0300_000A;
pub const PARAM_VOICE_EXCITER_AMOUNT: ParamId = 0x0300_000B;
pub const PARAM_VOICE_EXCITER_MIX: ParamId = 0x0300_000C;

// 6. De-Esser Section (0x0300_000D - 0x0300_000E)
pub const PARAM_VOICE_DEESS_ENABLE: ParamId = 0x0300_000D;
pub const PARAM_VOICE_DEESS_AMOUNT: ParamId = 0x0300_000E;

// 7. Smart Delay Section (0x0300_000F - 0x0300_0012)
pub const PARAM_VOICE_DELAY_ENABLE: ParamId = 0x0300_000F;
pub const PARAM_VOICE_DELAY_TIME: ParamId = 0x0300_0010;
pub const PARAM_VOICE_DELAY_FEEDBACK: ParamId = 0x0300_0011;
pub const PARAM_VOICE_DELAY_MIX: ParamId = 0x0300_0012;
pub const PARAM_VOICE_DELAY_SENSITIVITY: ParamId = 0x0300_0013;

// 8. Master Section (0x0300_0014 - 0x0300_0015)
pub const PARAM_VOICE_DRY_WET: ParamId = 0x0300_0014;
pub const PARAM_VOICE_OUTPUT_GAIN: ParamId = 0x0300_0015;

// ============================================================================
// PARAMETER REGISTRY
// ============================================================================

static VOICE_PARAM_REGISTRY: OnceLock<HashMap<ParamId, ParamDescriptor>> = OnceLock::new();

pub fn get_voice_param_registry() -> &'static HashMap<ParamId, ParamDescriptor> {
    VOICE_PARAM_REGISTRY.get_or_init(|| {
        let mut registry = HashMap::new();

        // === Input/Output ===
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

        // === Signal Analysis ===
        registry.insert(
            PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD,
                "Pitch Confidence",
                "Analysis",
                0.0,
                1.0,
                0.6,
                Some("%"),
            ),
        );

        // === Smart Gate ===
        registry.insert(
            PARAM_VOICE_GATE_ENABLE,
            ParamDescriptor::bool(PARAM_VOICE_GATE_ENABLE, "Gate Enable", "Gate", true),
        );

        registry.insert(
            PARAM_VOICE_GATE_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_GATE_THRESHOLD,
                "Gate Threshold",
                "Gate",
                -80.0,
                -20.0,
                -50.0,
                Some("dB"),
            ),
        );

        // === Adaptive Compressor ===
        registry.insert(
            PARAM_VOICE_COMP_ENABLE,
            ParamDescriptor::bool(PARAM_VOICE_COMP_ENABLE, "Comp Enable", "Compressor", true),
        );

        registry.insert(
            PARAM_VOICE_COMP_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_THRESHOLD,
                "Comp Threshold",
                "Compressor",
                -40.0,
                0.0,
                -18.0,
                Some("dB"),
            ),
        );

        registry.insert(
            PARAM_VOICE_COMP_RATIO,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_RATIO,
                "Comp Ratio",
                "Compressor",
                1.0,
                20.0,
                3.5,
                Some(":1"),
            ),
        );

        registry.insert(
            PARAM_VOICE_COMP_ATTACK,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_ATTACK,
                "Comp Attack",
                "Compressor",
                0.1,
                100.0,
                8.0,
                Some("ms"),
            ),
        );

        registry.insert(
            PARAM_VOICE_COMP_RELEASE,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_RELEASE,
                "Comp Release",
                "Compressor",
                10.0,
                1000.0,
                150.0,
                Some("ms"),
            ),
        );

        // === Intelligent Exciter ===
        registry.insert(
            PARAM_VOICE_EXCITER_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_EXCITER_ENABLE,
                "Exciter Enable",
                "Exciter",
                true,
            ),
        );

        registry.insert(
            PARAM_VOICE_EXCITER_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_AMOUNT,
                "Exciter Amount",
                "Exciter",
                0.0,
                1.0,
                0.3,
                Some("%"),
            ),
        );

        registry.insert(
            PARAM_VOICE_EXCITER_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_MIX,
                "Exciter Mix",
                "Exciter",
                0.0,
                1.0,
                0.3,
                Some("%"),
            ),
        );

        // === De-Esser ===
        registry.insert(
            PARAM_VOICE_DEESS_ENABLE,
            ParamDescriptor::bool(PARAM_VOICE_DEESS_ENABLE, "De-Ess Enable", "De-Esser", true),
        );

        registry.insert(
            PARAM_VOICE_DEESS_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_DEESS_AMOUNT,
                "De-Ess Amount",
                "De-Esser",
                0.0,
                12.0,
                6.0,
                Some("dB"),
            ),
        );

        // === Smart Delay ===
        registry.insert(
            PARAM_VOICE_DELAY_ENABLE,
            ParamDescriptor::bool(PARAM_VOICE_DELAY_ENABLE, "Delay Enable", "Delay", false),
        );

        registry.insert(
            PARAM_VOICE_DELAY_TIME,
            ParamDescriptor::float(
                PARAM_VOICE_DELAY_TIME,
                "Delay Time",
                "Delay",
                50.0,
                500.0,
                120.0,
                Some("ms"),
            ),
        );

        registry.insert(
            PARAM_VOICE_DELAY_FEEDBACK,
            ParamDescriptor::float(
                PARAM_VOICE_DELAY_FEEDBACK,
                "Delay Feedback",
                "Delay",
                0.0,
                0.8,
                0.3,
                Some("%"),
            ),
        );

        registry.insert(
            PARAM_VOICE_DELAY_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_DELAY_MIX,
                "Delay Mix",
                "Delay",
                0.0,
                1.0,
                0.4,
                Some("%"),
            ),
        );
        registry.insert(
            PARAM_VOICE_DELAY_SENSITIVITY,
            ParamDescriptor::float(
                PARAM_VOICE_DELAY_SENSITIVITY,
                "Delay Sensitivity",
                "Smart Delay",
                0.0,
                1.0,
                0.5,
                Some("%"),
            ),
        );
        // === Master ===
        registry.insert(
            PARAM_VOICE_DRY_WET,
            ParamDescriptor::float(
                PARAM_VOICE_DRY_WET,
                "Dry/Wet",
                "Master",
                0.0,
                1.0,
                1.0,
                Some("%"),
            ),
        );

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
        // Input/Output
        PARAM_VOICE_INPUT_GAIN => params.input_gain = value,
        PARAM_VOICE_OUTPUT_GAIN => params.output_gain = value,

        // Signal Analysis
        PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD => params.pitch_confidence_threshold = value,

        // Smart Gate
        PARAM_VOICE_GATE_ENABLE => params.gate_enable = value > 0.5,
        PARAM_VOICE_GATE_THRESHOLD => params.gate_threshold = value,

        // Adaptive Compressor
        PARAM_VOICE_COMP_ENABLE => params.comp_enable = value > 0.5,
        PARAM_VOICE_COMP_THRESHOLD => params.comp_threshold = value,
        PARAM_VOICE_COMP_RATIO => params.comp_ratio = value,
        PARAM_VOICE_COMP_ATTACK => params.comp_attack = value,
        PARAM_VOICE_COMP_RELEASE => params.comp_release = value,

        // Intelligent Exciter
        PARAM_VOICE_EXCITER_ENABLE => params.exciter_enable = value > 0.5,
        PARAM_VOICE_EXCITER_AMOUNT => params.exciter_amount = value,
        PARAM_VOICE_EXCITER_MIX => params.exciter_mix = value,

        // De-Esser
        PARAM_VOICE_DEESS_ENABLE => params.deess_enable = value > 0.5,
        PARAM_VOICE_DEESS_AMOUNT => params.deess_amount = value,

        // Smart Delay
        PARAM_VOICE_DELAY_ENABLE => params.delay_enable = value > 0.5,
        PARAM_VOICE_DELAY_TIME => params.delay_time = value,
        PARAM_VOICE_DELAY_FEEDBACK => params.delay_feedback = value,
        PARAM_VOICE_DELAY_MIX => params.delay_mix = value,
        PARAM_VOICE_DELAY_SENSITIVITY => params.delay_sensitivity = value,

        // Master
        PARAM_VOICE_DRY_WET => params.dry_wet = value,

        _ => {
            // Unknown parameter - ignore silently
        }
    }
}

/// Get parameter value from VoiceParams
pub fn get_param(params: &VoiceParams, param_id: ParamId) -> Option<f32> {
    match param_id {
        // Input/Output
        PARAM_VOICE_INPUT_GAIN => Some(params.input_gain),
        PARAM_VOICE_OUTPUT_GAIN => Some(params.output_gain),

        // Signal Analysis
        PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD => Some(params.pitch_confidence_threshold),

        // Smart Gate
        PARAM_VOICE_GATE_ENABLE => Some(if params.gate_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_GATE_THRESHOLD => Some(params.gate_threshold),

        // Adaptive Compressor
        PARAM_VOICE_COMP_ENABLE => Some(if params.comp_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_COMP_THRESHOLD => Some(params.comp_threshold),
        PARAM_VOICE_COMP_RATIO => Some(params.comp_ratio),
        PARAM_VOICE_COMP_ATTACK => Some(params.comp_attack),
        PARAM_VOICE_COMP_RELEASE => Some(params.comp_release),

        // Intelligent Exciter
        PARAM_VOICE_EXCITER_ENABLE => Some(if params.exciter_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_EXCITER_AMOUNT => Some(params.exciter_amount),
        PARAM_VOICE_EXCITER_MIX => Some(params.exciter_mix),

        // De-Esser
        PARAM_VOICE_DEESS_ENABLE => Some(if params.deess_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_DEESS_AMOUNT => Some(params.deess_amount),

        // Smart Delay
        PARAM_VOICE_DELAY_ENABLE => Some(if params.delay_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_DELAY_TIME => Some(params.delay_time),
        PARAM_VOICE_DELAY_FEEDBACK => Some(params.delay_feedback),
        PARAM_VOICE_DELAY_MIX => Some(params.delay_mix),
        PARAM_VOICE_DELAY_SENSITIVITY => Some(params.delay_sensitivity),

        // Master
        PARAM_VOICE_DRY_WET => Some(params.dry_wet),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_initialized() {
        let registry = get_voice_param_registry();
        assert_eq!(registry.len(), 21); // 21 total parameters
    }

    #[test]
    fn test_all_params_have_descriptors() {
        let registry = get_voice_param_registry();

        // Input/Output
        assert!(registry.contains_key(&PARAM_VOICE_INPUT_GAIN));
        assert!(registry.contains_key(&PARAM_VOICE_OUTPUT_GAIN));

        // Signal Analysis
        assert!(registry.contains_key(&PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD));

        // Smart Gate
        assert!(registry.contains_key(&PARAM_VOICE_GATE_ENABLE));
        assert!(registry.contains_key(&PARAM_VOICE_GATE_THRESHOLD));

        // Adaptive Compressor
        assert!(registry.contains_key(&PARAM_VOICE_COMP_ENABLE));
        assert!(registry.contains_key(&PARAM_VOICE_COMP_THRESHOLD));
        assert!(registry.contains_key(&PARAM_VOICE_COMP_RATIO));
        assert!(registry.contains_key(&PARAM_VOICE_COMP_ATTACK));
        assert!(registry.contains_key(&PARAM_VOICE_COMP_RELEASE));

        // Intelligent Exciter
        assert!(registry.contains_key(&PARAM_VOICE_EXCITER_ENABLE));
        assert!(registry.contains_key(&PARAM_VOICE_EXCITER_AMOUNT));
        assert!(registry.contains_key(&PARAM_VOICE_EXCITER_MIX));

        // De-Esser
        assert!(registry.contains_key(&PARAM_VOICE_DEESS_ENABLE));
        assert!(registry.contains_key(&PARAM_VOICE_DEESS_AMOUNT));

        // Smart Delay
        assert!(registry.contains_key(&PARAM_VOICE_DELAY_ENABLE));
        assert!(registry.contains_key(&PARAM_VOICE_DELAY_TIME));
        assert!(registry.contains_key(&PARAM_VOICE_DELAY_FEEDBACK));
        assert!(registry.contains_key(&PARAM_VOICE_DELAY_MIX));
        assert!(registry.contains_key(&PARAM_VOICE_DELAY_SENSITIVITY));

        // Master
        assert!(registry.contains_key(&PARAM_VOICE_DRY_WET));
        assert!(registry.contains_key(&PARAM_VOICE_OUTPUT_GAIN));
    }

    #[test]
    fn test_apply_and_get_param() {
        let mut params = VoiceParams::default();

        // Test applying parameters
        apply_param(&mut params, PARAM_VOICE_GATE_THRESHOLD, -40.0);
        assert_eq!(params.gate_threshold, -40.0);

        apply_param(&mut params, PARAM_VOICE_COMP_RATIO, 5.0);
        assert_eq!(params.comp_ratio, 5.0);

        // Test getting parameters
        assert_eq!(get_param(&params, PARAM_VOICE_GATE_THRESHOLD), Some(-40.0));
        assert_eq!(get_param(&params, PARAM_VOICE_COMP_RATIO), Some(5.0));
    }

    #[test]
    fn test_bool_params() {
        let mut params = VoiceParams::default();

        // Test enable/disable
        apply_param(&mut params, PARAM_VOICE_GATE_ENABLE, 1.0);
        assert_eq!(params.gate_enable, true);

        apply_param(&mut params, PARAM_VOICE_GATE_ENABLE, 0.0);
        assert_eq!(params.gate_enable, false);

        // Test get bool
        params.comp_enable = true;
        assert_eq!(get_param(&params, PARAM_VOICE_COMP_ENABLE), Some(1.0));

        params.comp_enable = false;
        assert_eq!(get_param(&params, PARAM_VOICE_COMP_ENABLE), Some(0.0));
    }
}
