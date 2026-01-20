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
use crate::params_voice::{RingModWaveform, SubOscWaveform, VoiceParams};
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
pub const PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD: ParamId = 0x0300_0090;

// Pitch-Controlled Filter Sweep (0x0300_0110 - 0x0300_011F)
pub const PARAM_VOICE_FILTER_FOLLOW_ENABLE: ParamId = 0x0300_0110;
pub const PARAM_VOICE_FILTER_FOLLOW_MIN_FREQ: ParamId = 0x0300_0111;
pub const PARAM_VOICE_FILTER_FOLLOW_MAX_FREQ: ParamId = 0x0300_0112;
pub const PARAM_VOICE_FILTER_FOLLOW_RESONANCE: ParamId = 0x0300_0113;
pub const PARAM_VOICE_FILTER_FOLLOW_AMOUNT: ParamId = 0x0300_0114;
pub const PARAM_VOICE_FILTER_FOLLOW_MIX: ParamId = 0x0300_0115;

// Pitch Correction / Auto-Tune (0x0300_0120 - 0x0300_012F)
pub const PARAM_VOICE_PITCH_CORRECTION_ENABLE: ParamId = 0x0300_0120;
pub const PARAM_VOICE_PITCH_CORRECTION_SCALE: ParamId = 0x0300_0121;
pub const PARAM_VOICE_PITCH_CORRECTION_ROOT: ParamId = 0x0300_0122;
pub const PARAM_VOICE_PITCH_CORRECTION_SPEED: ParamId = 0x0300_0123;
pub const PARAM_VOICE_PITCH_CORRECTION_AMOUNT: ParamId = 0x0300_0124;

// Sub Oscillator (0x0300_00A0 - 0x0300_00AF)
pub const PARAM_VOICE_SUB_ENABLE: ParamId = 0x0300_00A0;
pub const PARAM_VOICE_SUB_OCTAVE: ParamId = 0x0300_00A1;
pub const PARAM_VOICE_SUB_LEVEL: ParamId = 0x0300_00A2;
pub const PARAM_VOICE_SUB_WAVEFORM: ParamId = 0x0300_00A3;
pub const PARAM_VOICE_SUB_RAMP_TIME: ParamId = 0x0300_00A4;

// Harmonizer Oscillator 2 (0x0300_00B0 - 0x0300_00BF)
pub const PARAM_VOICE_HARM2_ENABLE: ParamId = 0x0300_00B0;
pub const PARAM_VOICE_HARM2_SEMITONES: ParamId = 0x0300_00B1;
pub const PARAM_VOICE_HARM2_LEVEL: ParamId = 0x0300_00B2;
pub const PARAM_VOICE_HARM2_WAVEFORM: ParamId = 0x0300_00B3;
pub const PARAM_VOICE_HARM2_RAMP_TIME: ParamId = 0x0300_00B4;

// Harmonizer Oscillator 3 (0x0300_00C0 - 0x0300_00CF)
pub const PARAM_VOICE_HARM3_ENABLE: ParamId = 0x0300_00C0;
pub const PARAM_VOICE_HARM3_SEMITONES: ParamId = 0x0300_00C1;
pub const PARAM_VOICE_HARM3_LEVEL: ParamId = 0x0300_00C2;
pub const PARAM_VOICE_HARM3_WAVEFORM: ParamId = 0x0300_00C3;
pub const PARAM_VOICE_HARM3_RAMP_TIME: ParamId = 0x0300_00C4;

// Harmonizer Oscillator 4 (0x0300_00D0 - 0x0300_00DF)
pub const PARAM_VOICE_HARM4_ENABLE: ParamId = 0x0300_00D0;
pub const PARAM_VOICE_HARM4_SEMITONES: ParamId = 0x0300_00D1;
pub const PARAM_VOICE_HARM4_LEVEL: ParamId = 0x0300_00D2;
pub const PARAM_VOICE_HARM4_WAVEFORM: ParamId = 0x0300_00D3;
pub const PARAM_VOICE_HARM4_RAMP_TIME: ParamId = 0x0300_00D4;

// Ring Modulator (0x0300_00E0 - 0x0300_00EF)
pub const PARAM_VOICE_RING_MOD_ENABLE: ParamId = 0x0300_00E0;
pub const PARAM_VOICE_RING_MOD_HARMONIC: ParamId = 0x0300_00E1;
pub const PARAM_VOICE_RING_MOD_WAVEFORM: ParamId = 0x0300_00E2;
pub const PARAM_VOICE_RING_MOD_DEPTH: ParamId = 0x0300_00E3;
pub const PARAM_VOICE_RING_MOD_MIX: ParamId = 0x0300_00E4;

// Exciter (0x0300_00F0 - 0x0300_00FF)
pub const PARAM_VOICE_EXCITER_AMOUNT: ParamId = 0x0300_00F0;
pub const PARAM_VOICE_EXCITER_FREQUENCY: ParamId = 0x0300_00F1;
pub const PARAM_VOICE_EXCITER_HARMONICS: ParamId = 0x0300_00F2;
pub const PARAM_VOICE_EXCITER_MIX: ParamId = 0x0300_00F3;
pub const PARAM_VOICE_EXCITER_FOLLOW_ENABLE: ParamId = 0x0300_00F4;
pub const PARAM_VOICE_EXCITER_FOLLOW_AMOUNT: ParamId = 0x0300_00F5;

// Master (0x0300_0100 - 0x0300_010F)
pub const PARAM_VOICE_DRY_WET: ParamId = 0x0300_0100;

// Vocal Doubler (0x0300_0130 - 0x0300_013F)
pub const PARAM_VOICE_DOUBLER_ENABLE: ParamId = 0x0300_0130;
pub const PARAM_VOICE_DOUBLER_DELAY: ParamId = 0x0300_0131;
pub const PARAM_VOICE_DOUBLER_DETUNE: ParamId = 0x0300_0132;
pub const PARAM_VOICE_DOUBLER_STEREO_WIDTH: ParamId = 0x0300_0133;
pub const PARAM_VOICE_DOUBLER_MIX: ParamId = 0x0300_0134;

// Vocal Choir (0x0300_0140 - 0x0300_014F)
pub const PARAM_VOICE_CHOIR_ENABLE: ParamId = 0x0300_0140;
pub const PARAM_VOICE_CHOIR_NUM_VOICES: ParamId = 0x0300_0141;
pub const PARAM_VOICE_CHOIR_DETUNE: ParamId = 0x0300_0142;
pub const PARAM_VOICE_CHOIR_DELAY_SPREAD: ParamId = 0x0300_0143;
pub const PARAM_VOICE_CHOIR_STEREO_SPREAD: ParamId = 0x0300_0144;
pub const PARAM_VOICE_CHOIR_MIX: ParamId = 0x0300_0145;

// Multiband Distortion (0x0300_0150 - 0x0300_015F)
pub const PARAM_VOICE_MB_DIST_ENABLE: ParamId = 0x0300_0150;
pub const PARAM_VOICE_MB_DIST_LOW_MID_FREQ: ParamId = 0x0300_0151;
pub const PARAM_VOICE_MB_DIST_MID_HIGH_FREQ: ParamId = 0x0300_0152;
pub const PARAM_VOICE_MB_DIST_DRIVE_LOW: ParamId = 0x0300_0153;
pub const PARAM_VOICE_MB_DIST_DRIVE_MID: ParamId = 0x0300_0154;
pub const PARAM_VOICE_MB_DIST_DRIVE_HIGH: ParamId = 0x0300_0155;
pub const PARAM_VOICE_MB_DIST_GAIN_LOW: ParamId = 0x0300_0156;
pub const PARAM_VOICE_MB_DIST_GAIN_MID: ParamId = 0x0300_0157;
pub const PARAM_VOICE_MB_DIST_GAIN_HIGH: ParamId = 0x0300_0158;
pub const PARAM_VOICE_MB_DIST_MIX: ParamId = 0x0300_0159;

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

        // Pitch Detector (smoothing is now adaptive)
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
            PARAM_VOICE_SUB_ENABLE,
            ParamDescriptor::bool(PARAM_VOICE_SUB_ENABLE, "Sub Enable", "Sub Oscillator", true)
        );

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

        // Harmonizer Oscillator 2 (Major 3rd)
        add_param!(
            PARAM_VOICE_HARM2_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_HARM2_ENABLE,
                "Harm2 Enable",
                "Harmonizer 2",
                false
            )
        );

        add_param!(
            PARAM_VOICE_HARM2_SEMITONES,
            ParamDescriptor::float(
                PARAM_VOICE_HARM2_SEMITONES,
                "Harm2 Semitones",
                "Harmonizer 2",
                -24.0,
                24.0,
                4.0,
                Some("st")
            )
        );

        add_param!(
            PARAM_VOICE_HARM2_LEVEL,
            ParamDescriptor::float(
                PARAM_VOICE_HARM2_LEVEL,
                "Harm2 Level",
                "Harmonizer 2",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_HARM2_WAVEFORM,
            ParamDescriptor::enum_param(
                PARAM_VOICE_HARM2_WAVEFORM,
                "Harm2 Waveform",
                "Harmonizer 2",
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
            PARAM_VOICE_HARM2_RAMP_TIME,
            ParamDescriptor::float(
                PARAM_VOICE_HARM2_RAMP_TIME,
                "Harm2 Ramp Time",
                "Harmonizer 2",
                1.0,
                100.0,
                10.0,
                Some("ms")
            )
        );

        // Harmonizer Oscillator 3 (Perfect 5th)
        add_param!(
            PARAM_VOICE_HARM3_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_HARM3_ENABLE,
                "Harm3 Enable",
                "Harmonizer 3",
                false
            )
        );

        add_param!(
            PARAM_VOICE_HARM3_SEMITONES,
            ParamDescriptor::float(
                PARAM_VOICE_HARM3_SEMITONES,
                "Harm3 Semitones",
                "Harmonizer 3",
                -24.0,
                24.0,
                7.0,
                Some("st")
            )
        );

        add_param!(
            PARAM_VOICE_HARM3_LEVEL,
            ParamDescriptor::float(
                PARAM_VOICE_HARM3_LEVEL,
                "Harm3 Level",
                "Harmonizer 3",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_HARM3_WAVEFORM,
            ParamDescriptor::enum_param(
                PARAM_VOICE_HARM3_WAVEFORM,
                "Harm3 Waveform",
                "Harmonizer 3",
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
            PARAM_VOICE_HARM3_RAMP_TIME,
            ParamDescriptor::float(
                PARAM_VOICE_HARM3_RAMP_TIME,
                "Harm3 Ramp Time",
                "Harmonizer 3",
                1.0,
                100.0,
                10.0,
                Some("ms")
            )
        );

        // Harmonizer Oscillator 4 (Octave Up)
        add_param!(
            PARAM_VOICE_HARM4_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_HARM4_ENABLE,
                "Harm4 Enable",
                "Harmonizer 4",
                false
            )
        );

        add_param!(
            PARAM_VOICE_HARM4_SEMITONES,
            ParamDescriptor::float(
                PARAM_VOICE_HARM4_SEMITONES,
                "Harm4 Semitones",
                "Harmonizer 4",
                -24.0,
                24.0,
                12.0,
                Some("st")
            )
        );

        add_param!(
            PARAM_VOICE_HARM4_LEVEL,
            ParamDescriptor::float(
                PARAM_VOICE_HARM4_LEVEL,
                "Harm4 Level",
                "Harmonizer 4",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_HARM4_WAVEFORM,
            ParamDescriptor::enum_param(
                PARAM_VOICE_HARM4_WAVEFORM,
                "Harm4 Waveform",
                "Harmonizer 4",
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
            PARAM_VOICE_HARM4_RAMP_TIME,
            ParamDescriptor::float(
                PARAM_VOICE_HARM4_RAMP_TIME,
                "Harm4 Ramp Time",
                "Harmonizer 4",
                1.0,
                100.0,
                10.0,
                Some("ms")
            )
        );

        // Ring Modulator
        add_param!(
            PARAM_VOICE_RING_MOD_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_RING_MOD_ENABLE,
                "Ring Mod Enable",
                "Ring Modulator",
                false
            )
        );

        add_param!(
            PARAM_VOICE_RING_MOD_HARMONIC,
            ParamDescriptor::float(
                PARAM_VOICE_RING_MOD_HARMONIC,
                "Ring Mod Harmonic",
                "Ring Modulator",
                0.5,
                8.0,
                2.0,
                Some("x")
            )
        );

        add_param!(
            PARAM_VOICE_RING_MOD_WAVEFORM,
            ParamDescriptor::enum_param(
                PARAM_VOICE_RING_MOD_WAVEFORM,
                "Ring Mod Waveform",
                "Ring Modulator",
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
            PARAM_VOICE_RING_MOD_DEPTH,
            ParamDescriptor::float(
                PARAM_VOICE_RING_MOD_DEPTH,
                "Ring Mod Depth",
                "Ring Modulator",
                0.0,
                1.0,
                1.0,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_RING_MOD_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_RING_MOD_MIX,
                "Ring Mod Mix",
                "Ring Modulator",
                0.0,
                1.0,
                0.5,
                Some("%")
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

        // Pitch-Controlled Filter Sweep
        add_param!(
            PARAM_VOICE_FILTER_FOLLOW_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_FILTER_FOLLOW_ENABLE,
                "Filter Follow Enable",
                "Filter Follow",
                false
            )
        );

        add_param!(
            PARAM_VOICE_FILTER_FOLLOW_MIN_FREQ,
            ParamDescriptor::float(
                PARAM_VOICE_FILTER_FOLLOW_MIN_FREQ,
                "Filter Min Freq",
                "Filter Follow",
                100.0,
                2000.0,
                150.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_FILTER_FOLLOW_MAX_FREQ,
            ParamDescriptor::float(
                PARAM_VOICE_FILTER_FOLLOW_MAX_FREQ,
                "Filter Max Freq",
                "Filter Follow",
                2000.0,
                20000.0,
                12000.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_FILTER_FOLLOW_RESONANCE,
            ParamDescriptor::float(
                PARAM_VOICE_FILTER_FOLLOW_RESONANCE,
                "Filter Resonance",
                "Filter Follow",
                0.1,
                10.0,
                2.5,
                Some("Q")
            )
        );

        add_param!(
            PARAM_VOICE_FILTER_FOLLOW_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_FILTER_FOLLOW_AMOUNT,
                "Filter Tracking",
                "Filter Follow",
                0.0,
                2.0,
                1.0,
                Some("x")
            )
        );

        add_param!(
            PARAM_VOICE_FILTER_FOLLOW_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_FILTER_FOLLOW_MIX,
                "Filter Mix",
                "Filter Follow",
                0.0,
                1.0,
                1.0,
                Some("%")
            )
        );

        // Exciter
        add_param!(
            PARAM_VOICE_EXCITER_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_AMOUNT,
                "Amount",
                "Exciter",
                0.0,
                1.0,
                0.0,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_EXCITER_FREQUENCY,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_FREQUENCY,
                "Frequency",
                "Exciter",
                2000.0,
                12000.0,
                5000.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_EXCITER_HARMONICS,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_HARMONICS,
                "Harmonics",
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
                "Mix",
                "Exciter",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_EXCITER_FOLLOW_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_EXCITER_FOLLOW_ENABLE,
                "Follow Enable",
                "Exciter",
                false
            )
        );

        add_param!(
            PARAM_VOICE_EXCITER_FOLLOW_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_EXCITER_FOLLOW_AMOUNT,
                "Follow Amount",
                "Exciter",
                1.0,
                4.0,
                1.5,
                Some("×")
            )
        );

        // Pitch Correction / Auto-Tune
        add_param!(
            PARAM_VOICE_PITCH_CORRECTION_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_PITCH_CORRECTION_ENABLE,
                "Pitch Correct Enable",
                "Pitch Correction",
                false
            )
        );

        add_param!(
            PARAM_VOICE_PITCH_CORRECTION_SCALE,
            ParamDescriptor::float(
                PARAM_VOICE_PITCH_CORRECTION_SCALE,
                "Scale",
                "Pitch Correction",
                0.0,
                4.0,
                0.0,
                None // 0=Chromatic, 1=Major, 2=Minor, 3=Pentatonic, 4=MinorPent
            )
        );

        add_param!(
            PARAM_VOICE_PITCH_CORRECTION_ROOT,
            ParamDescriptor::float(
                PARAM_VOICE_PITCH_CORRECTION_ROOT,
                "Root Note",
                "Pitch Correction",
                0.0,
                11.0,
                0.0,
                None // 0=C, 1=C#, 2=D, ... 11=B
            )
        );

        add_param!(
            PARAM_VOICE_PITCH_CORRECTION_SPEED,
            ParamDescriptor::float(
                PARAM_VOICE_PITCH_CORRECTION_SPEED,
                "Retune Speed",
                "Pitch Correction",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_PITCH_CORRECTION_AMOUNT,
            ParamDescriptor::float(
                PARAM_VOICE_PITCH_CORRECTION_AMOUNT,
                "Correction Amount",
                "Pitch Correction",
                0.0,
                1.0,
                0.0,
                Some("%")
            )
        );

        // Vocal Doubler
        add_param!(
            PARAM_VOICE_DOUBLER_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_DOUBLER_ENABLE,
                "Doubler Enable",
                "Doubler",
                false
            )
        );

        add_param!(
            PARAM_VOICE_DOUBLER_DELAY,
            ParamDescriptor::float(
                PARAM_VOICE_DOUBLER_DELAY,
                "Delay Time",
                "Doubler",
                5.0,
                15.0,
                10.0,
                Some("ms")
            )
        );

        add_param!(
            PARAM_VOICE_DOUBLER_DETUNE,
            ParamDescriptor::float(
                PARAM_VOICE_DOUBLER_DETUNE,
                "Detune",
                "Doubler",
                0.0,
                10.0,
                5.0,
                Some("¢") // cents symbol
            )
        );

        add_param!(
            PARAM_VOICE_DOUBLER_STEREO_WIDTH,
            ParamDescriptor::float(
                PARAM_VOICE_DOUBLER_STEREO_WIDTH,
                "Stereo Width",
                "Doubler",
                0.0,
                1.0,
                0.7,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_DOUBLER_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_DOUBLER_MIX,
                "Mix",
                "Doubler",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );

        // Vocal Choir
        add_param!(
            PARAM_VOICE_CHOIR_ENABLE,
            ParamDescriptor::bool(PARAM_VOICE_CHOIR_ENABLE, "Choir Enable", "Choir", false)
        );

        add_param!(
            PARAM_VOICE_CHOIR_NUM_VOICES,
            ParamDescriptor::float(
                PARAM_VOICE_CHOIR_NUM_VOICES,
                "Num Voices",
                "Choir",
                2.0,
                8.0,
                4.0,
                None
            )
        );

        add_param!(
            PARAM_VOICE_CHOIR_DETUNE,
            ParamDescriptor::float(
                PARAM_VOICE_CHOIR_DETUNE,
                "Detune",
                "Choir",
                0.0,
                30.0,
                15.0,
                Some("¢") // cents symbol
            )
        );

        add_param!(
            PARAM_VOICE_CHOIR_DELAY_SPREAD,
            ParamDescriptor::float(
                PARAM_VOICE_CHOIR_DELAY_SPREAD,
                "Delay Spread",
                "Choir",
                10.0,
                40.0,
                25.0,
                Some("ms")
            )
        );

        add_param!(
            PARAM_VOICE_CHOIR_STEREO_SPREAD,
            ParamDescriptor::float(
                PARAM_VOICE_CHOIR_STEREO_SPREAD,
                "Stereo Spread",
                "Choir",
                0.0,
                1.0,
                0.8,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_CHOIR_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_CHOIR_MIX,
                "Mix",
                "Choir",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );

        // Multiband Distortion
        add_param!(
            PARAM_VOICE_MB_DIST_ENABLE,
            ParamDescriptor::bool(
                PARAM_VOICE_MB_DIST_ENABLE,
                "MB Dist Enable",
                "MB Distortion",
                false
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_LOW_MID_FREQ,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_LOW_MID_FREQ,
                "Low-Mid Freq",
                "MB Distortion",
                50.0,
                500.0,
                200.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_MID_HIGH_FREQ,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_MID_HIGH_FREQ,
                "Mid-High Freq",
                "MB Distortion",
                1000.0,
                8000.0,
                2000.0,
                Some("Hz")
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_DRIVE_LOW,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_DRIVE_LOW,
                "Low Drive",
                "MB Distortion",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_DRIVE_MID,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_DRIVE_MID,
                "Mid Drive",
                "MB Distortion",
                0.0,
                1.0,
                0.2,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_DRIVE_HIGH,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_DRIVE_HIGH,
                "High Drive",
                "MB Distortion",
                0.0,
                1.0,
                0.1,
                Some("%")
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_GAIN_LOW,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_GAIN_LOW,
                "Low Gain",
                "MB Distortion",
                0.0,
                2.0,
                1.0,
                None
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_GAIN_MID,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_GAIN_MID,
                "Mid Gain",
                "MB Distortion",
                0.0,
                2.0,
                1.0,
                None
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_GAIN_HIGH,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_GAIN_HIGH,
                "High Gain",
                "MB Distortion",
                0.0,
                2.0,
                1.0,
                None
            )
        );

        add_param!(
            PARAM_VOICE_MB_DIST_MIX,
            ParamDescriptor::float(
                PARAM_VOICE_MB_DIST_MIX,
                "Mix",
                "MB Distortion",
                0.0,
                1.0,
                0.5,
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

        // Exciter
        PARAM_VOICE_EXCITER_AMOUNT => params.exciter_amount = denorm_value,
        PARAM_VOICE_EXCITER_FREQUENCY => params.exciter_frequency = denorm_value,
        PARAM_VOICE_EXCITER_HARMONICS => params.exciter_harmonics = denorm_value,
        PARAM_VOICE_EXCITER_MIX => params.exciter_mix = denorm_value,
        PARAM_VOICE_EXCITER_FOLLOW_ENABLE => params.exciter_follow_enable = denorm_value > 0.5,
        PARAM_VOICE_EXCITER_FOLLOW_AMOUNT => params.exciter_follow_amount = denorm_value,

        // Pitch Detector
        PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD => params.pitch_confidence_threshold = denorm_value,

        // Pitch Correction / Auto-Tune
        PARAM_VOICE_PITCH_CORRECTION_ENABLE => params.pitch_correction_enable = denorm_value > 0.5,
        PARAM_VOICE_PITCH_CORRECTION_SCALE => {
            params.pitch_correction_scale = denorm_value.round() as u8
        }
        PARAM_VOICE_PITCH_CORRECTION_ROOT => {
            params.pitch_correction_root = denorm_value.round() as u8
        }
        PARAM_VOICE_PITCH_CORRECTION_SPEED => params.pitch_correction_speed = denorm_value,
        PARAM_VOICE_PITCH_CORRECTION_AMOUNT => params.pitch_correction_amount = denorm_value,

        // Pitch-Controlled Filter Sweep
        PARAM_VOICE_FILTER_FOLLOW_ENABLE => params.filter_follow_enable = denorm_value > 0.5,
        PARAM_VOICE_FILTER_FOLLOW_MIN_FREQ => params.filter_follow_min_freq = denorm_value,
        PARAM_VOICE_FILTER_FOLLOW_MAX_FREQ => params.filter_follow_max_freq = denorm_value,
        PARAM_VOICE_FILTER_FOLLOW_RESONANCE => params.filter_follow_resonance = denorm_value,
        PARAM_VOICE_FILTER_FOLLOW_AMOUNT => params.filter_follow_amount = denorm_value,
        PARAM_VOICE_FILTER_FOLLOW_MIX => params.filter_follow_mix = denorm_value,

        // Sub Oscillator
        PARAM_VOICE_SUB_ENABLE => params.sub_enable = denorm_value > 0.5,
        PARAM_VOICE_SUB_OCTAVE => params.sub_octave = denorm_value,
        PARAM_VOICE_SUB_LEVEL => params.sub_level = denorm_value,
        PARAM_VOICE_SUB_WAVEFORM => {
            let index = denorm_value.round() as usize;
            params.sub_waveform = SubOscWaveform::from_index(index);
        }
        PARAM_VOICE_SUB_RAMP_TIME => params.sub_ramp_time = denorm_value,

        // Harmonizer 2
        PARAM_VOICE_HARM2_ENABLE => params.harm2_enable = denorm_value > 0.5,
        PARAM_VOICE_HARM2_SEMITONES => params.harm2_semitones = denorm_value,
        PARAM_VOICE_HARM2_LEVEL => params.harm2_level = denorm_value,
        PARAM_VOICE_HARM2_WAVEFORM => {
            let index = denorm_value.round() as usize;
            params.harm2_waveform = SubOscWaveform::from_index(index);
        }
        PARAM_VOICE_HARM2_RAMP_TIME => params.harm2_ramp_time = denorm_value,

        // Harmonizer 3
        PARAM_VOICE_HARM3_ENABLE => params.harm3_enable = denorm_value > 0.5,
        PARAM_VOICE_HARM3_SEMITONES => params.harm3_semitones = denorm_value,
        PARAM_VOICE_HARM3_LEVEL => params.harm3_level = denorm_value,
        PARAM_VOICE_HARM3_WAVEFORM => {
            let index = denorm_value.round() as usize;
            params.harm3_waveform = SubOscWaveform::from_index(index);
        }
        PARAM_VOICE_HARM3_RAMP_TIME => params.harm3_ramp_time = denorm_value,

        // Harmonizer 4
        PARAM_VOICE_HARM4_ENABLE => params.harm4_enable = denorm_value > 0.5,
        PARAM_VOICE_HARM4_SEMITONES => params.harm4_semitones = denorm_value,
        PARAM_VOICE_HARM4_LEVEL => params.harm4_level = denorm_value,
        PARAM_VOICE_HARM4_WAVEFORM => {
            let index = denorm_value.round() as usize;
            params.harm4_waveform = SubOscWaveform::from_index(index);
        }
        PARAM_VOICE_HARM4_RAMP_TIME => params.harm4_ramp_time = denorm_value,

        // Ring Modulator
        PARAM_VOICE_RING_MOD_ENABLE => params.ring_mod_enable = denorm_value > 0.5,
        PARAM_VOICE_RING_MOD_HARMONIC => params.ring_mod_harmonic = denorm_value,
        PARAM_VOICE_RING_MOD_WAVEFORM => {
            let index = denorm_value.round() as usize;
            params.ring_mod_waveform = RingModWaveform::from_index(index);
        }
        PARAM_VOICE_RING_MOD_DEPTH => params.ring_mod_depth = denorm_value,
        PARAM_VOICE_RING_MOD_MIX => params.ring_mod_mix = denorm_value,

        // Vocal Doubler
        PARAM_VOICE_DOUBLER_ENABLE => params.doubler_enable = denorm_value > 0.5,
        PARAM_VOICE_DOUBLER_DELAY => params.doubler_delay = denorm_value,
        PARAM_VOICE_DOUBLER_DETUNE => params.doubler_detune = denorm_value,
        PARAM_VOICE_DOUBLER_STEREO_WIDTH => params.doubler_stereo_width = denorm_value,
        PARAM_VOICE_DOUBLER_MIX => params.doubler_mix = denorm_value,

        // Vocal Choir
        PARAM_VOICE_CHOIR_ENABLE => params.choir_enable = denorm_value > 0.5,
        PARAM_VOICE_CHOIR_NUM_VOICES => params.choir_num_voices = denorm_value.round() as usize,
        PARAM_VOICE_CHOIR_DETUNE => params.choir_detune = denorm_value,
        PARAM_VOICE_CHOIR_DELAY_SPREAD => params.choir_delay_spread = denorm_value,
        PARAM_VOICE_CHOIR_STEREO_SPREAD => params.choir_stereo_spread = denorm_value,
        PARAM_VOICE_CHOIR_MIX => params.choir_mix = denorm_value,

        // Multiband Distortion
        PARAM_VOICE_MB_DIST_ENABLE => params.mb_dist_enable = denorm_value > 0.5,
        PARAM_VOICE_MB_DIST_LOW_MID_FREQ => params.mb_dist_low_mid_freq = denorm_value,
        PARAM_VOICE_MB_DIST_MID_HIGH_FREQ => params.mb_dist_mid_high_freq = denorm_value,
        PARAM_VOICE_MB_DIST_DRIVE_LOW => params.mb_dist_drive_low = denorm_value,
        PARAM_VOICE_MB_DIST_DRIVE_MID => params.mb_dist_drive_mid = denorm_value,
        PARAM_VOICE_MB_DIST_DRIVE_HIGH => params.mb_dist_drive_high = denorm_value,
        PARAM_VOICE_MB_DIST_GAIN_LOW => params.mb_dist_gain_low = denorm_value,
        PARAM_VOICE_MB_DIST_GAIN_MID => params.mb_dist_gain_mid = denorm_value,
        PARAM_VOICE_MB_DIST_GAIN_HIGH => params.mb_dist_gain_high = denorm_value,
        PARAM_VOICE_MB_DIST_MIX => params.mb_dist_mix = denorm_value,

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
        PARAM_VOICE_PITCH_CONFIDENCE_THRESHOLD => Some(params.pitch_confidence_threshold),

        // Pitch Correction / Auto-Tune
        PARAM_VOICE_PITCH_CORRECTION_ENABLE => Some(if params.pitch_correction_enable {
            1.0
        } else {
            0.0
        }),
        PARAM_VOICE_PITCH_CORRECTION_SCALE => Some(params.pitch_correction_scale as f32),
        PARAM_VOICE_PITCH_CORRECTION_ROOT => Some(params.pitch_correction_root as f32),
        PARAM_VOICE_PITCH_CORRECTION_SPEED => Some(params.pitch_correction_speed),
        PARAM_VOICE_PITCH_CORRECTION_AMOUNT => Some(params.pitch_correction_amount),

        // Pitch-Controlled Filter Sweep
        PARAM_VOICE_FILTER_FOLLOW_ENABLE => Some(if params.filter_follow_enable {
            1.0
        } else {
            0.0
        }),
        PARAM_VOICE_FILTER_FOLLOW_MIN_FREQ => Some(params.filter_follow_min_freq),
        PARAM_VOICE_FILTER_FOLLOW_MAX_FREQ => Some(params.filter_follow_max_freq),
        PARAM_VOICE_FILTER_FOLLOW_RESONANCE => Some(params.filter_follow_resonance),

        // Sub Oscillator
        PARAM_VOICE_SUB_ENABLE => Some(if params.sub_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_SUB_OCTAVE => Some(params.sub_octave),
        PARAM_VOICE_SUB_LEVEL => Some(params.sub_level),
        PARAM_VOICE_SUB_WAVEFORM => Some(params.sub_waveform.to_index() as f32),
        PARAM_VOICE_SUB_RAMP_TIME => Some(params.sub_ramp_time),

        // Harmonizer 2
        PARAM_VOICE_HARM2_ENABLE => Some(if params.harm2_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_HARM2_SEMITONES => Some(params.harm2_semitones),
        PARAM_VOICE_HARM2_LEVEL => Some(params.harm2_level),
        PARAM_VOICE_HARM2_WAVEFORM => Some(params.harm2_waveform.to_index() as f32),
        PARAM_VOICE_HARM2_RAMP_TIME => Some(params.harm2_ramp_time),

        // Harmonizer 3
        PARAM_VOICE_HARM3_ENABLE => Some(if params.harm3_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_HARM3_SEMITONES => Some(params.harm3_semitones),
        PARAM_VOICE_HARM3_LEVEL => Some(params.harm3_level),
        PARAM_VOICE_HARM3_WAVEFORM => Some(params.harm3_waveform.to_index() as f32),
        PARAM_VOICE_HARM3_RAMP_TIME => Some(params.harm3_ramp_time),

        // Harmonizer 4
        PARAM_VOICE_HARM4_ENABLE => Some(if params.harm4_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_HARM4_SEMITONES => Some(params.harm4_semitones),
        PARAM_VOICE_HARM4_LEVEL => Some(params.harm4_level),
        PARAM_VOICE_HARM4_WAVEFORM => Some(params.harm4_waveform.to_index() as f32),
        PARAM_VOICE_HARM4_RAMP_TIME => Some(params.harm4_ramp_time),

        // Ring Modulator
        PARAM_VOICE_RING_MOD_ENABLE => Some(if params.ring_mod_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_RING_MOD_HARMONIC => Some(params.ring_mod_harmonic),
        PARAM_VOICE_RING_MOD_WAVEFORM => Some(params.ring_mod_waveform.to_index() as f32),
        PARAM_VOICE_RING_MOD_DEPTH => Some(params.ring_mod_depth),
        PARAM_VOICE_RING_MOD_MIX => Some(params.ring_mod_mix),

        // Vocal Doubler
        PARAM_VOICE_DOUBLER_ENABLE => Some(if params.doubler_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_DOUBLER_DELAY => Some(params.doubler_delay),
        PARAM_VOICE_DOUBLER_DETUNE => Some(params.doubler_detune),
        PARAM_VOICE_DOUBLER_STEREO_WIDTH => Some(params.doubler_stereo_width),
        PARAM_VOICE_DOUBLER_MIX => Some(params.doubler_mix),

        // Vocal Choir
        PARAM_VOICE_CHOIR_ENABLE => Some(if params.choir_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_CHOIR_NUM_VOICES => Some(params.choir_num_voices as f32),
        PARAM_VOICE_CHOIR_DETUNE => Some(params.choir_detune),
        PARAM_VOICE_CHOIR_DELAY_SPREAD => Some(params.choir_delay_spread),
        PARAM_VOICE_CHOIR_STEREO_SPREAD => Some(params.choir_stereo_spread),
        PARAM_VOICE_CHOIR_MIX => Some(params.choir_mix),

        // Exciter
        PARAM_VOICE_EXCITER_AMOUNT => Some(params.exciter_amount),
        PARAM_VOICE_EXCITER_FREQUENCY => Some(params.exciter_frequency),
        PARAM_VOICE_EXCITER_HARMONICS => Some(params.exciter_harmonics),
        PARAM_VOICE_EXCITER_MIX => Some(params.exciter_mix),
        PARAM_VOICE_EXCITER_FOLLOW_ENABLE => Some(if params.exciter_follow_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_EXCITER_FOLLOW_AMOUNT => Some(params.exciter_follow_amount),

        // Multiband Distortion
        PARAM_VOICE_MB_DIST_ENABLE => Some(if params.mb_dist_enable { 1.0 } else { 0.0 }),
        PARAM_VOICE_MB_DIST_LOW_MID_FREQ => Some(params.mb_dist_low_mid_freq),
        PARAM_VOICE_MB_DIST_MID_HIGH_FREQ => Some(params.mb_dist_mid_high_freq),
        PARAM_VOICE_MB_DIST_DRIVE_LOW => Some(params.mb_dist_drive_low),
        PARAM_VOICE_MB_DIST_DRIVE_MID => Some(params.mb_dist_drive_mid),
        PARAM_VOICE_MB_DIST_DRIVE_HIGH => Some(params.mb_dist_drive_high),
        PARAM_VOICE_MB_DIST_GAIN_LOW => Some(params.mb_dist_gain_low),
        PARAM_VOICE_MB_DIST_GAIN_MID => Some(params.mb_dist_gain_mid),
        PARAM_VOICE_MB_DIST_GAIN_HIGH => Some(params.mb_dist_gain_high),
        PARAM_VOICE_MB_DIST_MIX => Some(params.mb_dist_mix),

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
    fn test_clap_descriptor_is_float() {
        // Test that the CLAP wrapper converts Enum to Float (as per main synth pattern)
        // This ensures DAW compatibility (Reaper doesn't properly support CLAP Enum type)
        use crate::voice_clap::DsynthVoiceParams;
        use dsynth_clap::param::{ParamType, PluginParams};

        let descriptor = DsynthVoiceParams::param_descriptor_by_id(PARAM_VOICE_SUB_WAVEFORM)
            .expect("Sub waveform parameter should exist");

        match &descriptor.param_type {
            ParamType::Float { min, max, .. } => {
                assert_eq!(*min, 0.0, "CLAP Float min should be 0.0");
                assert_eq!(*max, 1.0, "CLAP Float max should be 1.0");
                // Enum converted to Float for DAW compatibility
            }
            ParamType::Bool { .. } => panic!("CLAP descriptor is Bool, should be Float!"),
            _ => panic!("CLAP descriptor should be Float (converted from Enum)!"),
        }
    }

    #[test]
    fn test_filter_follow_params_registered() {
        let registry = get_voice_registry();

        // Verify all 6 filter follow parameters are registered
        let filter_params = [
            (PARAM_VOICE_FILTER_FOLLOW_ENABLE, "Filter Follow Enable"),
            (PARAM_VOICE_FILTER_FOLLOW_MIN_FREQ, "Filter Min Freq"),
            (PARAM_VOICE_FILTER_FOLLOW_MAX_FREQ, "Filter Max Freq"),
            (PARAM_VOICE_FILTER_FOLLOW_RESONANCE, "Filter Resonance"),
            (PARAM_VOICE_FILTER_FOLLOW_AMOUNT, "Filter Tracking"),
            (PARAM_VOICE_FILTER_FOLLOW_MIX, "Filter Mix"),
        ];

        for (id, expected_name) in &filter_params {
            let desc = registry
                .get(id)
                .expect(&format!("Filter param 0x{:08x} should be registered", id));
            assert_eq!(&desc.name, expected_name, "Parameter name mismatch");
        }

        // Verify total count includes new parameters
        assert!(
            registry.len() >= 50,
            "Registry should have at least 50 parameters (was {})",
            registry.len()
        );
    }
}
