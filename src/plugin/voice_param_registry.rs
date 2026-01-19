/// Voice Enhancer Parameter Registry
///
/// Central registry for all voice enhancement parameters with CLAP plugin integration.
/// Parameter namespace: 0x0300_xxxx (voice enhancer)
///
/// This registry provides:
/// - Unique parameter IDs
/// - Parameter descriptors (type, range, unit, default)
/// - Normalization/denormalization for CLAP (0.0-1.0 range)
/// - Parameter lookup and application
use crate::params_voice::{SubOscWaveform, VoiceParams};
use crate::plugin::param_descriptor::{ParamDescriptor, ParamId};
use std::collections::HashMap;
use std::sync::OnceLock;

// ============================================================================
// PARAMETER IDs (Namespace: 0x0300_xxxx)
// ============================================================================

// Input/Output (0x0300_0000 - 0x0300_000F)
pub const PARAM_VOICE_INPUT_GAIN: ParamId = 0x0300_0001;
pub const PARAM_VOICE_OUTPUT_GAIN: ParamId = 0x0300_0002;

// Noise Gate (0x0300_0010 - 0x0300_001F)
pub const PARAM_VOICE_GATE_THRESHOLD: ParamId = 0x0300_0010;
pub const PARAM_VOICE_GATE_RATIO: ParamId = 0x0300_0011;
pub const PARAM_VOICE_GATE_ATTACK: ParamId = 0x0300_0012;
pub const PARAM_VOICE_GATE_RELEASE: ParamId = 0x0300_0013;
pub const PARAM_VOICE_GATE_HOLD: ParamId = 0x0300_0014;

// Parametric EQ - Band 1 (0x0300_0020 - 0x0300_002F)
pub const PARAM_VOICE_EQ_BAND1_FREQ: ParamId = 0x0300_0020;
pub const PARAM_VOICE_EQ_BAND1_GAIN: ParamId = 0x0300_0021;
pub const PARAM_VOICE_EQ_BAND1_Q: ParamId = 0x0300_0022;

// Parametric EQ - Band 2 (0x0300_0030 - 0x0300_003F)
pub const PARAM_VOICE_EQ_BAND2_FREQ: ParamId = 0x0300_0030;
pub const PARAM_VOICE_EQ_BAND2_GAIN: ParamId = 0x0300_0031;
pub const PARAM_VOICE_EQ_BAND2_Q: ParamId = 0x0300_0032;

// Parametric EQ - Band 3 (0x0300_0040 - 0x0300_004F)
pub const PARAM_VOICE_EQ_BAND3_FREQ: ParamId = 0x0300_0040;
pub const PARAM_VOICE_EQ_BAND3_GAIN: ParamId = 0x0300_0041;
pub const PARAM_VOICE_EQ_BAND3_Q: ParamId = 0x0300_0042;

// Parametric EQ - Band 4 (0x0300_0050 - 0x0300_005F)
pub const PARAM_VOICE_EQ_BAND4_FREQ: ParamId = 0x0300_0050;
pub const PARAM_VOICE_EQ_BAND4_GAIN: ParamId = 0x0300_0051;
pub const PARAM_VOICE_EQ_BAND4_Q: ParamId = 0x0300_0052;

// Parametric EQ - Master (0x0300_0060 - 0x0300_006F)
pub const PARAM_VOICE_EQ_MASTER_GAIN: ParamId = 0x0300_0060;

// Compressor (0x0300_0070 - 0x0300_007F)
pub const PARAM_VOICE_COMP_THRESHOLD: ParamId = 0x0300_0070;
pub const PARAM_VOICE_COMP_RATIO: ParamId = 0x0300_0071;
pub const PARAM_VOICE_COMP_ATTACK: ParamId = 0x0300_0072;
pub const PARAM_VOICE_COMP_RELEASE: ParamId = 0x0300_0073;
pub const PARAM_VOICE_COMP_KNEE: ParamId = 0x0300_0074;
pub const PARAM_VOICE_COMP_MAKEUP_GAIN: ParamId = 0x0300_0075;

// De-Esser (0x0300_0080 - 0x0300_008F)
pub const PARAM_VOICE_DEESS_THRESHOLD: ParamId = 0x0300_0080;
pub const PARAM_VOICE_DEESS_FREQUENCY: ParamId = 0x0300_0081;
pub const PARAM_VOICE_DEESS_RATIO: ParamId = 0x0300_0082;
pub const PARAM_VOICE_DEESS_AMOUNT: ParamId = 0x0300_0083;

// Pitch Detector (0x0300_0090 - 0x0300_009F)
pub const PARAM_VOICE_PITCH_SMOOTHING: ParamId = 0x0300_0090;
pub const PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD: ParamId = 0x0300_0091;

// Sub Oscillator (0x0300_00A0 - 0x0300_00AF)
pub const PARAM_VOICE_SUB_OCTAVE: ParamId = 0x0300_00A0;
pub const PARAM_VOICE_SUB_LEVEL: ParamId = 0x0300_00A1;
pub const PARAM_VOICE_SUB_WAVEFORM: ParamId = 0x0300_00A2;
pub const PARAM_VOICE_SUB_RAMP_TIME: ParamId = 0x0300_00A3;

// Exciter (0x0300_00B0 - 0x0300_00BF)
pub const PARAM_VOICE_EXCITER_AMOUNT: ParamId = 0x0300_00B0;
pub const PARAM_VOICE_EXCITER_FREQUENCY: ParamId = 0x0300_00B1;
pub const PARAM_VOICE_EXCITER_HARMONICS: ParamId = 0x0300_00B2;
pub const PARAM_VOICE_EXCITER_MIX: ParamId = 0x0300_00B3;

// Master (0x0300_00C0 - 0x0300_00CF)
pub const PARAM_VOICE_DRY_WET: ParamId = 0x0300_00C0;

// ============================================================================
// PARAMETER REGISTRY
// ============================================================================

static VOICE_REGISTRY: OnceLock<VoiceParamRegistry> = OnceLock::new();

pub fn get_voice_registry() -> &'static VoiceParamRegistry {
    VOICE_REGISTRY.get_or_init(VoiceParamRegistry::new)
}

pub struct VoiceParamRegistry {
    descriptors: HashMap<ParamId, ParamDescriptor>,
    param_ids: Vec<ParamId>,
}

impl Default for VoiceParamRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceParamRegistry {
    pub fn new() -> Self {
        let mut descriptors = HashMap::new();
        let mut param_ids = Vec::new();

        Self::register_params(&mut descriptors, &mut param_ids);

        Self {
            descriptors,
            param_ids,
        }
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn get(&self, param_id: &ParamId) -> Option<&ParamDescriptor> {
        self.descriptors.get(param_id)
    }

    pub fn keys(&self) -> impl Iterator<Item = &ParamId> {
        self.param_ids.iter()
    }

    fn register_params(
        descriptors: &mut HashMap<ParamId, ParamDescriptor>,
        param_ids: &mut Vec<ParamId>,
    ) {
        macro_rules! add_param {
            ($id:expr, $desc:expr) => {
                descriptors.insert($id, $desc);
                param_ids.push($id);
            };
        }

        // Input/Output
        add_param!(
            PARAM_VOICE_INPUT_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_INPUT_GAIN,
                "Input Gain",
                "Input/Output",
                -12.0,
                12.0,
                0.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_OUTPUT_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_OUTPUT_GAIN,
                "Output Gain",
                "Input/Output",
                -12.0,
                12.0,
                0.0,
                Some("dB")
            )
        );

        // Noise Gate
        add_param!(
            PARAM_VOICE_GATE_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_GATE_THRESHOLD,
                "Gate Threshold",
                "Noise Gate",
                -80.0,
                -20.0,
                -50.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_GATE_RATIO,
            ParamDescriptor::float(
                PARAM_VOICE_GATE_RATIO,
                "Gate Ratio",
                "Noise Gate",
                1.0,
                10.0,
                5.0,
                Some(":1")
            )
        );

        add_param!(
            PARAM_VOICE_GATE_ATTACK,
            ParamDescriptor::float(
                PARAM_VOICE_GATE_ATTACK,
                "Gate Attack",
                "Noise Gate",
                0.1,
                50.0,
                5.0,
                Some("ms")
            )
        );

        add_param!(
            PARAM_VOICE_GATE_RELEASE,
            ParamDescriptor::float(
                PARAM_VOICE_GATE_RELEASE,
                "Gate Release",
                "Noise Gate",
                10.0,
                500.0,
                100.0,
                Some("ms")
            )
        );

        add_param!(
            PARAM_VOICE_GATE_HOLD,
            ParamDescriptor::float(
                PARAM_VOICE_GATE_HOLD,
                "Gate Hold",
                "Noise Gate",
                0.0,
                200.0,
                50.0,
                Some("ms")
            )
        );

        // Parametric EQ - Band 1
        add_param!(
            PARAM_VOICE_EQ_BAND1_FREQ,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND1_FREQ,
                "EQ Band 1 Freq",
                "Parametric EQ",
                20.0,
                500.0,
                80.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_EQ_BAND1_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND1_GAIN,
                "EQ Band 1 Gain",
                "Parametric EQ",
                -12.0,
                12.0,
                0.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_EQ_BAND1_Q,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND1_Q,
                "EQ Band 1 Q",
                "Parametric EQ",
                0.1,
                10.0,
                1.0,
                None
            )
        );

        // Parametric EQ - Band 2
        add_param!(
            PARAM_VOICE_EQ_BAND2_FREQ,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND2_FREQ,
                "EQ Band 2 Freq",
                "Parametric EQ",
                100.0,
                2000.0,
                400.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_EQ_BAND2_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND2_GAIN,
                "EQ Band 2 Gain",
                "Parametric EQ",
                -12.0,
                12.0,
                0.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_EQ_BAND2_Q,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND2_Q,
                "EQ Band 2 Q",
                "Parametric EQ",
                0.1,
                10.0,
                1.0,
                None
            )
        );

        // Parametric EQ - Band 3
        add_param!(
            PARAM_VOICE_EQ_BAND3_FREQ,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND3_FREQ,
                "EQ Band 3 Freq",
                "Parametric EQ",
                1000.0,
                8000.0,
                3000.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_EQ_BAND3_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND3_GAIN,
                "EQ Band 3 Gain",
                "Parametric EQ",
                -12.0,
                12.0,
                0.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_EQ_BAND3_Q,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND3_Q,
                "EQ Band 3 Q",
                "Parametric EQ",
                0.1,
                10.0,
                1.0,
                None
            )
        );

        // Parametric EQ - Band 4
        add_param!(
            PARAM_VOICE_EQ_BAND4_FREQ,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND4_FREQ,
                "EQ Band 4 Freq",
                "Parametric EQ",
                2000.0,
                20000.0,
                8000.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_EQ_BAND4_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND4_GAIN,
                "EQ Band 4 Gain",
                "Parametric EQ",
                -12.0,
                12.0,
                0.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_EQ_BAND4_Q,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_BAND4_Q,
                "EQ Band 4 Q",
                "Parametric EQ",
                0.1,
                10.0,
                1.0,
                None
            )
        );

        // Parametric EQ - Master
        add_param!(
            PARAM_VOICE_EQ_MASTER_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_EQ_MASTER_GAIN,
                "EQ Master Gain",
                "Parametric EQ",
                -12.0,
                12.0,
                0.0,
                Some("dB")
            )
        );

        // Compressor
        add_param!(
            PARAM_VOICE_COMP_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_THRESHOLD,
                "Comp Threshold",
                "Compressor",
                -40.0,
                0.0,
                -20.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_COMP_RATIO,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_RATIO,
                "Comp Ratio",
                "Compressor",
                1.0,
                20.0,
                3.0,
                Some(":1")
            )
        );

        add_param!(
            PARAM_VOICE_COMP_ATTACK,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_ATTACK,
                "Comp Attack",
                "Compressor",
                0.1,
                100.0,
                10.0,
                Some("ms")
            )
        );

        add_param!(
            PARAM_VOICE_COMP_RELEASE,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_RELEASE,
                "Comp Release",
                "Compressor",
                10.0,
                1000.0,
                100.0,
                Some("ms")
            )
        );

        add_param!(
            PARAM_VOICE_COMP_KNEE,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_KNEE,
                "Comp Knee",
                "Compressor",
                0.0,
                12.0,
                6.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_COMP_MAKEUP_GAIN,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_MAKEUP_GAIN,
                "Comp Makeup",
                "Compressor",
                0.0,
                24.0,
                0.0,
                Some("dB")
            )
        );

        // De-Esser
        add_param!(
            PARAM_VOICE_DEESS_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_DEESS_THRESHOLD,
                "De-Ess Threshold",
                "De-Esser",
                -40.0,
                0.0,
                -25.0,
                Some("dB")
            )
        );

        add_param!(
            PARAM_VOICE_DEESS_FREQUENCY,
            ParamDescriptor::float(
                PARAM_VOICE_DEESS_FREQUENCY,
                "De-Ess Frequency",
                "De-Esser",
                4000.0,
                10000.0,
                6000.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_DEESS_RATIO,
            ParamDescriptor::float(
                PARAM_VOICE_DEESS_RATIO,
                "De-Ess Ratio",
                "De-Esser",
                1.0,
                10.0,
                4.0,
                Some(":1")
            )
        );

        add_param!(
            PARAM_VOICE_DEESS_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_DEESS_AMOUNT,
                "De-Ess Amount",
                "De-Esser",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );

        // Pitch Detector
        add_param!(
            PARAM_VOICE_PITCH_SMOOTHING,
            ParamDescriptor::float(
                PARAM_VOICE_PITCH_SMOOTHING,
                "Pitch Smoothing",
                "Pitch Detection",
                0.0,
                1.0,
                0.7,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD,
                "Pitch Confidence",
                "Pitch Detection",
                0.0,
                1.0,
                0.6,
                Some("%")
            )
        );

        // Sub Oscillator
        add_param!(
            PARAM_VOICE_SUB_OCTAVE,
            ParamDescriptor::float(
                PARAM_VOICE_SUB_OCTAVE,
                "Sub Octave",
                "Sub Oscillator",
                -2.0,
                0.0,
                -1.0,
                Some("oct")
            )
        );

        add_param!(
            PARAM_VOICE_SUB_LEVEL,
            ParamDescriptor::float(
                PARAM_VOICE_SUB_LEVEL,
                "Sub Level",
                "Sub Oscillator",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_SUB_WAVEFORM,
            ParamDescriptor::enum_param(
                PARAM_VOICE_SUB_WAVEFORM,
                "Sub Waveform",
                "Sub Oscillator",
                vec![
                    "Sine".to_string(),
                    "Triangle".to_string(),
                    "Square".to_string(),
                    "Saw".to_string()
                ],
                0
            )
        );
        add_param!(
            PARAM_VOICE_SUB_RAMP_TIME,
            ParamDescriptor::float(
                PARAM_VOICE_SUB_RAMP_TIME,
                "Sub Ramp Time",
                "Sub Oscillator",
                1.0,
                100.0,
                10.0,
                Some("ms")
            )
        );

        // Exciter
        add_param!(
            PARAM_VOICE_EXCITER_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_AMOUNT,
                "Exciter Amount",
                "Exciter",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_EXCITER_FREQUENCY,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_FREQUENCY,
                "Exciter Frequency",
                "Exciter",
                2000.0,
                10000.0,
                4000.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_EXCITER_HARMONICS,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_HARMONICS,
                "Exciter Harmonics",
                "Exciter",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_EXCITER_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_MIX,
                "Exciter Mix",
                "Exciter",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );

        // Master
        add_param!(
            PARAM_VOICE_DRY_WET,
            ParamDescriptor::float(
                PARAM_VOICE_DRY_WET,
                "Dry/Wet",
                "Master",
                0.0,
                1.0,
                1.0,
                Some("%")
            )
        );
    }
}

/// Get parameter descriptor by ID
pub fn get_param_descriptor(param_id: ParamId) -> Option<&'static ParamDescriptor> {
    get_voice_registry().get(&param_id)
}

/// Apply parameter value to VoiceParams struct
pub fn apply_param(params: &mut VoiceParams, param_id: ParamId, denorm_value: f32) {
    match param_id {
        // Input/Output
        PARAM_VOICE_INPUT_GAIN => params.input_gain = denorm_value,
        PARAM_VOICE_OUTPUT_GAIN => params.output_gain = denorm_value,

        // Noise Gate
        PARAM_VOICE_GATE_THRESHOLD => params.gate_threshold = denorm_value,
        PARAM_VOICE_GATE_RATIO => params.gate_ratio = denorm_value,
        PARAM_VOICE_GATE_ATTACK => params.gate_attack = denorm_value,
        PARAM_VOICE_GATE_RELEASE => params.gate_release = denorm_value,
        PARAM_VOICE_GATE_HOLD => params.gate_hold = denorm_value,

        // Parametric EQ
        PARAM_VOICE_EQ_BAND1_FREQ => params.eq_band1_freq = denorm_value,
        PARAM_VOICE_EQ_BAND1_GAIN => params.eq_band1_gain = denorm_value,
        PARAM_VOICE_EQ_BAND1_Q => params.eq_band1_q = denorm_value,

        PARAM_VOICE_EQ_BAND2_FREQ => params.eq_band2_freq = denorm_value,
        PARAM_VOICE_EQ_BAND2_GAIN => params.eq_band2_gain = denorm_value,
        PARAM_VOICE_EQ_BAND2_Q => params.eq_band2_q = denorm_value,

        PARAM_VOICE_EQ_BAND3_FREQ => params.eq_band3_freq = denorm_value,
        PARAM_VOICE_EQ_BAND3_GAIN => params.eq_band3_gain = denorm_value,
        PARAM_VOICE_EQ_BAND3_Q => params.eq_band3_q = denorm_value,

        PARAM_VOICE_EQ_BAND4_FREQ => params.eq_band4_freq = denorm_value,
        PARAM_VOICE_EQ_BAND4_GAIN => params.eq_band4_gain = denorm_value,
        PARAM_VOICE_EQ_BAND4_Q => params.eq_band4_q = denorm_value,

        PARAM_VOICE_EQ_MASTER_GAIN => params.eq_master_gain = denorm_value,

        // Compressor
        PARAM_VOICE_COMP_THRESHOLD => params.comp_threshold = denorm_value,
        PARAM_VOICE_COMP_RATIO => params.comp_ratio = denorm_value,
        PARAM_VOICE_COMP_ATTACK => params.comp_attack = denorm_value,
        PARAM_VOICE_COMP_RELEASE => params.comp_release = denorm_value,
        PARAM_VOICE_COMP_KNEE => params.comp_knee = denorm_value,
        PARAM_VOICE_COMP_MAKEUP_GAIN => params.comp_makeup_gain = denorm_value,

        // De-Esser
        PARAM_VOICE_DEESS_THRESHOLD => params.deess_threshold = denorm_value,
        PARAM_VOICE_DEESS_FREQUENCY => params.deess_frequency = denorm_value,
        PARAM_VOICE_DEESS_RATIO => params.deess_ratio = denorm_value,
        PARAM_VOICE_DEESS_AMOUNT => params.deess_amount = denorm_value,

        // Pitch Detector
        PARAM_VOICE_PITCH_SMOOTHING => params.pitch_smoothing = denorm_value,
        PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD => params.pitch_confidence_threshold = denorm_value,

        // Sub Oscillator
        PARAM_VOICE_SUB_OCTAVE => params.sub_octave = denorm_value,
        PARAM_VOICE_SUB_LEVEL => params.sub_level = denorm_value,
        PARAM_VOICE_SUB_WAVEFORM => {
            let index = denorm_value.round() as usize;
            params.sub_waveform = SubOscWaveform::from_index(index);
        }
        PARAM_VOICE_SUB_RAMP_TIME => params.sub_ramp_time = denorm_value,

        // Exciter
        PARAM_VOICE_EXCITER_AMOUNT => params.exciter_amount = denorm_value,
        PARAM_VOICE_EXCITER_FREQUENCY => params.exciter_frequency = denorm_value,
        PARAM_VOICE_EXCITER_HARMONICS => params.exciter_harmonics = denorm_value,
        PARAM_VOICE_EXCITER_MIX => params.exciter_mix = denorm_value,

        // Master
        PARAM_VOICE_DRY_WET => params.dry_wet = denorm_value,

        _ => {} // Unknown parameter
    }
}

/// Get parameter value from VoiceParams struct
pub fn get_param(params: &VoiceParams, param_id: ParamId) -> Option<f32> {
    match param_id {
        // Input/Output
        PARAM_VOICE_INPUT_GAIN => Some(params.input_gain),
        PARAM_VOICE_OUTPUT_GAIN => Some(params.output_gain),

        // Noise Gate
        PARAM_VOICE_GATE_THRESHOLD => Some(params.gate_threshold),
        PARAM_VOICE_GATE_RATIO => Some(params.gate_ratio),
        PARAM_VOICE_GATE_ATTACK => Some(params.gate_attack),
        PARAM_VOICE_GATE_RELEASE => Some(params.gate_release),
        PARAM_VOICE_GATE_HOLD => Some(params.gate_hold),

        // Parametric EQ
        PARAM_VOICE_EQ_BAND1_FREQ => Some(params.eq_band1_freq),
        PARAM_VOICE_EQ_BAND1_GAIN => Some(params.eq_band1_gain),
        PARAM_VOICE_EQ_BAND1_Q => Some(params.eq_band1_q),

        PARAM_VOICE_EQ_BAND2_FREQ => Some(params.eq_band2_freq),
        PARAM_VOICE_EQ_BAND2_GAIN => Some(params.eq_band2_gain),
        PARAM_VOICE_EQ_BAND2_Q => Some(params.eq_band2_q),

        PARAM_VOICE_EQ_BAND3_FREQ => Some(params.eq_band3_freq),
        PARAM_VOICE_EQ_BAND3_GAIN => Some(params.eq_band3_gain),
        PARAM_VOICE_EQ_BAND3_Q => Some(params.eq_band3_q),

        PARAM_VOICE_EQ_BAND4_FREQ => Some(params.eq_band4_freq),
        PARAM_VOICE_EQ_BAND4_GAIN => Some(params.eq_band4_gain),
        PARAM_VOICE_EQ_BAND4_Q => Some(params.eq_band4_q),

        PARAM_VOICE_EQ_MASTER_GAIN => Some(params.eq_master_gain),

        // Compressor
        PARAM_VOICE_COMP_THRESHOLD => Some(params.comp_threshold),
        PARAM_VOICE_COMP_RATIO => Some(params.comp_ratio),
        PARAM_VOICE_COMP_ATTACK => Some(params.comp_attack),
        PARAM_VOICE_COMP_RELEASE => Some(params.comp_release),
        PARAM_VOICE_COMP_KNEE => Some(params.comp_knee),
        PARAM_VOICE_COMP_MAKEUP_GAIN => Some(params.comp_makeup_gain),

        // De-Esser
        PARAM_VOICE_DEESS_THRESHOLD => Some(params.deess_threshold),
        PARAM_VOICE_DEESS_FREQUENCY => Some(params.deess_frequency),
        PARAM_VOICE_DEESS_RATIO => Some(params.deess_ratio),
        PARAM_VOICE_DEESS_AMOUNT => Some(params.deess_amount),

        // Pitch Detector
        PARAM_VOICE_PITCH_SMOOTHING => Some(params.pitch_smoothing),
        PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD => Some(params.pitch_confidence_threshold),

        // Sub Oscillator
        PARAM_VOICE_SUB_OCTAVE => Some(params.sub_octave),
        PARAM_VOICE_SUB_LEVEL => Some(params.sub_level),
        PARAM_VOICE_SUB_WAVEFORM => Some(params.sub_waveform.to_index() as f32),
        PARAM_VOICE_SUB_RAMP_TIME => Some(params.sub_ramp_time),

        // Exciter
        PARAM_VOICE_EXCITER_AMOUNT => Some(params.exciter_amount),
        PARAM_VOICE_EXCITER_FREQUENCY => Some(params.exciter_frequency),
        PARAM_VOICE_EXCITER_HARMONICS => Some(params.exciter_harmonics),
        PARAM_VOICE_EXCITER_MIX => Some(params.exciter_mix),

        // Master
        PARAM_VOICE_DRY_WET => Some(params.dry_wet),

        _ => None, // Unknown parameter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_waveform_has_four_variants() {
        let registry = get_voice_registry();
        let descriptor = registry
            .get(&PARAM_VOICE_SUB_WAVEFORM)
            .expect("Sub waveform parameter should exist");

        if let crate::plugin::param_descriptor::ParamType::Enum { variants } =
            &descriptor.param_type
        {
            assert_eq!(variants.len(), 4, "Sub waveform should have 4 variants");
            assert_eq!(variants[0], "Sine");
            assert_eq!(variants[1], "Triangle");
            assert_eq!(variants[2], "Square");
            assert_eq!(variants[3], "Saw");
        } else {
            panic!("Sub waveform parameter should be an Enum type");
        }
    }

    #[test]
    fn test_clap_descriptor_is_enum() {
        // Test that the CLAP wrapper returns Enum type
        use crate::voice_clap::DsynthVoiceParams;
        use dsynth_clap::param::{ParamType, PluginParams};

        let descriptor = DsynthVoiceParams::param_descriptor_by_id(PARAM_VOICE_SUB_WAVEFORM)
            .expect("Sub waveform parameter should exist");

        match &descriptor.param_type {
            ParamType::Enum { variants, .. } => {
                assert_eq!(variants.len(), 4, "CLAP descriptor should have 4 variants");
                assert_eq!(variants[0], "Sine");
                assert_eq!(variants[1], "Triangle");
                assert_eq!(variants[2], "Square");
                assert_eq!(variants[3], "Saw");
            }
            ParamType::Bool { .. } => panic!("CLAP descriptor is Bool, should be Enum!"),
            _ => panic!("CLAP descriptor is wrong type, should be Enum!"),
        }
    }
}
