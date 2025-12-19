use nih_plug::prelude::*;

use crate::params::{DistortionType, FilterType, LFOWaveform, Waveform};

/// Plugin parameters that map to our SynthParams
#[derive(Params)]
pub struct DSynthParams {
    /// Master output gain
    #[id = "master_gain"]
    pub master_gain: FloatParam,

    /// Monophonic mode
    #[id = "monophonic"]
    pub monophonic: BoolParam,

    // Oscillator 1 parameters
    #[id = "osc1_waveform"]
    pub osc1_waveform: EnumParam<Waveform>,

    #[id = "osc1_solo"]
    pub osc1_solo: BoolParam,

    #[id = "osc1_pitch"]
    pub osc1_pitch: FloatParam,

    #[id = "osc1_detune"]
    pub osc1_detune: FloatParam,

    #[id = "osc1_gain"]
    pub osc1_gain: FloatParam,

    #[id = "osc1_pan"]
    pub osc1_pan: FloatParam,

    #[id = "osc1_unison"]
    pub osc1_unison: IntParam,

    #[id = "osc1_unison_detune"]
    pub osc1_unison_detune: FloatParam,

    #[id = "osc1_phase"]
    pub osc1_phase: FloatParam,

    #[id = "osc1_shape"]
    pub osc1_shape: FloatParam,

    #[id = "osc1_fm_source"]
    pub osc1_fm_source: IntParam,

    #[id = "osc1_fm_amount"]
    pub osc1_fm_amount: FloatParam,

    // Oscillator 2 parameters
    #[id = "osc2_waveform"]
    pub osc2_waveform: EnumParam<Waveform>,

    #[id = "osc2_solo"]
    pub osc2_solo: BoolParam,

    #[id = "osc2_pitch"]
    pub osc2_pitch: FloatParam,

    #[id = "osc2_detune"]
    pub osc2_detune: FloatParam,

    #[id = "osc2_gain"]
    pub osc2_gain: FloatParam,

    #[id = "osc2_pan"]
    pub osc2_pan: FloatParam,

    #[id = "osc2_unison"]
    pub osc2_unison: IntParam,

    #[id = "osc2_unison_detune"]
    pub osc2_unison_detune: FloatParam,

    #[id = "osc2_phase"]
    pub osc2_phase: FloatParam,

    #[id = "osc2_shape"]
    pub osc2_shape: FloatParam,

    #[id = "osc2_fm_source"]
    pub osc2_fm_source: IntParam,

    #[id = "osc2_fm_amount"]
    pub osc2_fm_amount: FloatParam,

    // Oscillator 3 parameters
    #[id = "osc3_waveform"]
    pub osc3_waveform: EnumParam<Waveform>,

    #[id = "osc3_solo"]
    pub osc3_solo: BoolParam,

    #[id = "osc3_pitch"]
    pub osc3_pitch: FloatParam,

    #[id = "osc3_detune"]
    pub osc3_detune: FloatParam,

    #[id = "osc3_gain"]
    pub osc3_gain: FloatParam,

    #[id = "osc3_pan"]
    pub osc3_pan: FloatParam,

    #[id = "osc3_unison"]
    pub osc3_unison: IntParam,

    #[id = "osc3_unison_detune"]
    pub osc3_unison_detune: FloatParam,

    #[id = "osc3_phase"]
    pub osc3_phase: FloatParam,

    #[id = "osc3_shape"]
    pub osc3_shape: FloatParam,

    #[id = "osc3_fm_source"]
    pub osc3_fm_source: IntParam,

    #[id = "osc3_fm_amount"]
    pub osc3_fm_amount: FloatParam,

    // Filter 1 parameters
    #[id = "filter1_type"]
    pub filter1_type: EnumParam<FilterType>,

    #[id = "filter1_cutoff"]
    pub filter1_cutoff: FloatParam,

    #[id = "filter1_resonance"]
    pub filter1_resonance: FloatParam,

    #[id = "filter1_bandwidth"]
    pub filter1_bandwidth: FloatParam,

    #[id = "filter1_key_tracking"]
    pub filter1_key_tracking: FloatParam,

    // Filter 2 parameters
    #[id = "filter2_type"]
    pub filter2_type: EnumParam<FilterType>,

    #[id = "filter2_cutoff"]
    pub filter2_cutoff: FloatParam,

    #[id = "filter2_resonance"]
    pub filter2_resonance: FloatParam,

    #[id = "filter2_bandwidth"]
    pub filter2_bandwidth: FloatParam,

    #[id = "filter2_key_tracking"]
    pub filter2_key_tracking: FloatParam,

    // Filter 3 parameters
    #[id = "filter3_type"]
    pub filter3_type: EnumParam<FilterType>,

    #[id = "filter3_cutoff"]
    pub filter3_cutoff: FloatParam,

    #[id = "filter3_resonance"]
    pub filter3_resonance: FloatParam,

    #[id = "filter3_bandwidth"]
    pub filter3_bandwidth: FloatParam,

    #[id = "filter3_key_tracking"]
    pub filter3_key_tracking: FloatParam,

    // Filter Envelope 1
    #[id = "fenv1_attack"]
    pub fenv1_attack: FloatParam,

    #[id = "fenv1_decay"]
    pub fenv1_decay: FloatParam,

    #[id = "fenv1_sustain"]
    pub fenv1_sustain: FloatParam,

    #[id = "fenv1_release"]
    pub fenv1_release: FloatParam,

    #[id = "fenv1_amount"]
    pub fenv1_amount: FloatParam,

    // Filter Envelope 2
    #[id = "fenv2_attack"]
    pub fenv2_attack: FloatParam,

    #[id = "fenv2_decay"]
    pub fenv2_decay: FloatParam,

    #[id = "fenv2_sustain"]
    pub fenv2_sustain: FloatParam,

    #[id = "fenv2_release"]
    pub fenv2_release: FloatParam,

    #[id = "fenv2_amount"]
    pub fenv2_amount: FloatParam,

    // Filter Envelope 3
    #[id = "fenv3_attack"]
    pub fenv3_attack: FloatParam,

    #[id = "fenv3_decay"]
    pub fenv3_decay: FloatParam,

    #[id = "fenv3_sustain"]
    pub fenv3_sustain: FloatParam,

    #[id = "fenv3_release"]
    pub fenv3_release: FloatParam,

    #[id = "fenv3_amount"]
    pub fenv3_amount: FloatParam,

    // LFO 1
    #[id = "lfo1_waveform"]
    pub lfo1_waveform: EnumParam<LFOWaveform>,

    #[id = "lfo1_rate"]
    pub lfo1_rate: FloatParam,

    #[id = "lfo1_depth"]
    pub lfo1_depth: FloatParam,

    #[id = "lfo1_filter_amount"]
    pub lfo1_filter_amount: FloatParam,

    // LFO 2
    #[id = "lfo2_waveform"]
    pub lfo2_waveform: EnumParam<LFOWaveform>,

    #[id = "lfo2_rate"]
    pub lfo2_rate: FloatParam,

    #[id = "lfo2_depth"]
    pub lfo2_depth: FloatParam,

    #[id = "lfo2_filter_amount"]
    pub lfo2_filter_amount: FloatParam,

    // LFO 3
    #[id = "lfo3_waveform"]
    pub lfo3_waveform: EnumParam<LFOWaveform>,

    #[id = "lfo3_rate"]
    pub lfo3_rate: FloatParam,

    #[id = "lfo3_depth"]
    pub lfo3_depth: FloatParam,

    #[id = "lfo3_filter_amount"]
    pub lfo3_filter_amount: FloatParam,

    // Velocity Sensitivity
    #[id = "velocity_amp"]
    pub velocity_amp: FloatParam,

    #[id = "velocity_filter"]
    pub velocity_filter: FloatParam,

    #[id = "velocity_filter_env"]
    pub velocity_filter_env: FloatParam,

    // Envelope (ADSR)
    #[id = "envelope_attack"]
    pub envelope_attack: FloatParam,

    #[id = "envelope_decay"]
    pub envelope_decay: FloatParam,

    #[id = "envelope_sustain"]
    pub envelope_sustain: FloatParam,

    #[id = "envelope_release"]
    pub envelope_release: FloatParam,

    // Effects - Reverb
    #[id = "reverb_room_size"]
    pub reverb_room_size: FloatParam,

    #[id = "reverb_damping"]
    pub reverb_damping: FloatParam,

    #[id = "reverb_wet"]
    pub reverb_wet: FloatParam,

    #[id = "reverb_dry"]
    pub reverb_dry: FloatParam,

    #[id = "reverb_width"]
    pub reverb_width: FloatParam,

    // Effects - Delay
    #[id = "delay_time_ms"]
    pub delay_time_ms: FloatParam,

    #[id = "delay_feedback"]
    pub delay_feedback: FloatParam,

    #[id = "delay_wet"]
    pub delay_wet: FloatParam,

    #[id = "delay_dry"]
    pub delay_dry: FloatParam,

    // Effects - Chorus
    #[id = "chorus_rate"]
    pub chorus_rate: FloatParam,

    #[id = "chorus_depth"]
    pub chorus_depth: FloatParam,

    #[id = "chorus_mix"]
    pub chorus_mix: FloatParam,

    // Effects - Distortion
    #[id = "distortion_type"]
    pub distortion_type: EnumParam<DistortionType>,

    #[id = "distortion_drive"]
    pub distortion_drive: FloatParam,

    #[id = "distortion_mix"]
    pub distortion_mix: FloatParam,
}

impl Default for DSynthParams {
    fn default() -> Self {
        Self {
            master_gain: FloatParam::new(
                "Master Gain",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            monophonic: BoolParam::new("Monophonic", false),

            // Oscillator 1
            osc1_waveform: EnumParam::new("Osc 1 Wave", Waveform::Sine),
            osc1_solo: BoolParam::new("Osc 1 Solo", false),
            osc1_pitch: FloatParam::new(
                "Osc 1 Pitch",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" semi"),
            osc1_detune: FloatParam::new(
                "Osc 1 Detune",
                0.0,
                FloatRange::Linear {
                    min: -50.0,
                    max: 50.0,
                },
            )
            .with_unit(" cents"),
            osc1_gain: FloatParam::new(
                "Osc 1 Gain",
                0.33,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc1_pan: FloatParam::new(
                "Osc 1 Pan",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),
            osc1_unison: IntParam::new("Osc 1 Unison", 1, IntRange::Linear { min: 1, max: 7 }),
            osc1_unison_detune: FloatParam::new(
                "Osc 1 Unison Detune",
                10.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 50.0,
                },
            )
            .with_unit(" cents"),
            osc1_phase: FloatParam::new(
                "Osc 1 Phase",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc1_shape: FloatParam::new(
                "Osc 1 Shape",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),
            osc1_fm_source: IntParam::new(
                "Osc 1 FM Source",
                -1, // -1 = None, 0-2 = Osc 1-3
                IntRange::Linear { min: -1, max: 2 },
            ),
            osc1_fm_amount: FloatParam::new(
                "Osc 1 FM Amount",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            ),

            // Oscillator 2
            osc2_waveform: EnumParam::new("Osc 2 Wave", Waveform::Saw),
            osc2_solo: BoolParam::new("Osc 2 Solo", false),
            osc2_pitch: FloatParam::new(
                "Osc 2 Pitch",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" semi"),
            osc2_detune: FloatParam::new(
                "Osc 2 Detune",
                0.0,
                FloatRange::Linear {
                    min: -50.0,
                    max: 50.0,
                },
            )
            .with_unit(" cents"),
            osc2_gain: FloatParam::new(
                "Osc 2 Gain",
                0.33,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc2_pan: FloatParam::new(
                "Osc 2 Pan",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),
            osc2_unison: IntParam::new("Osc 2 Unison", 1, IntRange::Linear { min: 1, max: 7 }),
            osc2_unison_detune: FloatParam::new(
                "Osc 2 Unison Detune",
                10.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 50.0,
                },
            )
            .with_unit(" cents"),
            osc2_phase: FloatParam::new(
                "Osc 2 Phase",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc2_shape: FloatParam::new(
                "Osc 2 Shape",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),
            osc2_fm_source: IntParam::new(
                "Osc 2 FM Source",
                -1,
                IntRange::Linear { min: -1, max: 2 },
            ),
            osc2_fm_amount: FloatParam::new(
                "Osc 2 FM Amount",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            ),

            // Oscillator 3
            osc3_waveform: EnumParam::new("Osc 3 Wave", Waveform::Square),
            osc3_solo: BoolParam::new("Osc 3 Solo", false),
            osc3_pitch: FloatParam::new(
                "Osc 3 Pitch",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" semi"),
            osc3_detune: FloatParam::new(
                "Osc 3 Detune",
                0.0,
                FloatRange::Linear {
                    min: -50.0,
                    max: 50.0,
                },
            )
            .with_unit(" cents"),
            osc3_gain: FloatParam::new(
                "Osc 3 Gain",
                0.33,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc3_pan: FloatParam::new(
                "Osc 3 Pan",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),
            osc3_unison: IntParam::new("Osc 3 Unison", 1, IntRange::Linear { min: 1, max: 7 }),
            osc3_unison_detune: FloatParam::new(
                "Osc 3 Unison Detune",
                10.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 50.0,
                },
            )
            .with_unit(" cents"),
            osc3_phase: FloatParam::new(
                "Osc 3 Phase",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc3_shape: FloatParam::new(
                "Osc 3 Shape",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),
            osc3_fm_source: IntParam::new(
                "Osc 3 FM Source",
                -1,
                IntRange::Linear { min: -1, max: 2 },
            ),
            osc3_fm_amount: FloatParam::new(
                "Osc 3 FM Amount",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            ),

            // Filter 1
            filter1_type: EnumParam::new("Filter 1 Type", FilterType::Lowpass),
            filter1_cutoff: FloatParam::new(
                "Filter 1 Cutoff",
                8000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz"),
            filter1_resonance: FloatParam::new(
                "Filter 1 Resonance",
                0.7,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            ),
            filter1_bandwidth: FloatParam::new(
                "Filter 1 Bandwidth",
                1.0,
                FloatRange::Linear { min: 0.1, max: 4.0 },
            ),
            filter1_key_tracking: FloatParam::new(
                "Filter 1 Key Tracking",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            // Filter 2
            filter2_type: EnumParam::new("Filter 2 Type", FilterType::Lowpass),
            filter2_cutoff: FloatParam::new(
                "Filter 2 Cutoff",
                8000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz"),
            filter2_resonance: FloatParam::new(
                "Filter 2 Resonance",
                0.7,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            ),
            filter2_bandwidth: FloatParam::new(
                "Filter 2 Bandwidth",
                1.0,
                FloatRange::Linear { min: 0.1, max: 4.0 },
            ),
            filter2_key_tracking: FloatParam::new(
                "Filter 2 Key Tracking",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            // Filter 3
            filter3_type: EnumParam::new("Filter 3 Type", FilterType::Lowpass),
            filter3_cutoff: FloatParam::new(
                "Filter 3 Cutoff",
                8000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz"),
            filter3_resonance: FloatParam::new(
                "Filter 3 Resonance",
                0.7,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            ),
            filter3_bandwidth: FloatParam::new(
                "Filter 3 Bandwidth",
                1.0,
                FloatRange::Linear { min: 0.1, max: 4.0 },
            ),
            filter3_key_tracking: FloatParam::new(
                "Filter 3 Key Tracking",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            // Filter Envelope 1
            fenv1_attack: FloatParam::new(
                "Filter Env 1 Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv1_decay: FloatParam::new(
                "Filter Env 1 Decay",
                0.1,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv1_sustain: FloatParam::new(
                "Filter Env 1 Sustain",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            fenv1_release: FloatParam::new(
                "Filter Env 1 Release",
                0.2,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv1_amount: FloatParam::new(
                "Filter Env 1 Amount",
                2000.0,
                FloatRange::Linear {
                    min: -10000.0,
                    max: 10000.0,
                },
            )
            .with_unit(" Hz"),

            // Filter Envelope 2
            fenv2_attack: FloatParam::new(
                "Filter Env 2 Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv2_decay: FloatParam::new(
                "Filter Env 2 Decay",
                0.1,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv2_sustain: FloatParam::new(
                "Filter Env 2 Sustain",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            fenv2_release: FloatParam::new(
                "Filter Env 2 Release",
                0.2,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv2_amount: FloatParam::new(
                "Filter Env 2 Amount",
                2000.0,
                FloatRange::Linear {
                    min: -10000.0,
                    max: 10000.0,
                },
            )
            .with_unit(" Hz"),

            // Filter Envelope 3
            fenv3_attack: FloatParam::new(
                "Filter Env 3 Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv3_decay: FloatParam::new(
                "Filter Env 3 Decay",
                0.1,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv3_sustain: FloatParam::new(
                "Filter Env 3 Sustain",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            fenv3_release: FloatParam::new(
                "Filter Env 3 Release",
                0.2,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            fenv3_amount: FloatParam::new(
                "Filter Env 3 Amount",
                2000.0,
                FloatRange::Linear {
                    min: -10000.0,
                    max: 10000.0,
                },
            )
            .with_unit(" Hz"),

            // LFO 1
            lfo1_waveform: EnumParam::new("LFO 1 Waveform", LFOWaveform::Sine),
            lfo1_rate: FloatParam::new(
                "LFO 1 Rate",
                5.0,
                FloatRange::Skewed {
                    min: 0.01,
                    max: 20.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz"),
            lfo1_depth: FloatParam::new(
                "LFO 1 Depth",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            lfo1_filter_amount: FloatParam::new(
                "LFO 1 Filter Amount",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),

            // LFO 2
            lfo2_waveform: EnumParam::new("LFO 2 Waveform", LFOWaveform::Sine),
            lfo2_rate: FloatParam::new(
                "LFO 2 Rate",
                5.0,
                FloatRange::Skewed {
                    min: 0.01,
                    max: 20.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz"),
            lfo2_depth: FloatParam::new(
                "LFO 2 Depth",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            lfo2_filter_amount: FloatParam::new(
                "LFO 2 Filter Amount",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),

            // LFO 3
            lfo3_waveform: EnumParam::new("LFO 3 Waveform", LFOWaveform::Sine),
            lfo3_rate: FloatParam::new(
                "LFO 3 Rate",
                5.0,
                FloatRange::Skewed {
                    min: 0.01,
                    max: 20.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz"),
            lfo3_depth: FloatParam::new(
                "LFO 3 Depth",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            lfo3_filter_amount: FloatParam::new(
                "LFO 3 Filter Amount",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),

            // Velocity Sensitivity
            velocity_amp: FloatParam::new(
                "Velocity Amp",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            velocity_filter: FloatParam::new(
                "Velocity Filter",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            velocity_filter_env: FloatParam::new(
                "Velocity Filter Env",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            // Envelope (ADSR)
            envelope_attack: FloatParam::new(
                "Envelope Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            envelope_decay: FloatParam::new(
                "Envelope Decay",
                0.1,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            envelope_sustain: FloatParam::new(
                "Envelope Sustain",
                0.7,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            envelope_release: FloatParam::new(
                "Envelope Release",
                0.2,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),

            // Reverb
            reverb_room_size: FloatParam::new(
                "Reverb Room Size",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            reverb_damping: FloatParam::new(
                "Reverb Damping",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            reverb_wet: FloatParam::new(
                "Reverb Wet",
                0.33,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            reverb_dry: FloatParam::new(
                "Reverb Dry",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            reverb_width: FloatParam::new(
                "Reverb Width",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            // Delay
            delay_time_ms: FloatParam::new(
                "Delay Time",
                250.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 2000.0,
                },
            )
            .with_unit(" ms"),
            delay_feedback: FloatParam::new(
                "Delay Feedback",
                0.3,
                FloatRange::Linear {
                    min: 0.0,
                    max: 0.95,
                },
            ),
            delay_wet: FloatParam::new("Delay Wet", 0.3, FloatRange::Linear { min: 0.0, max: 1.0 }),
            delay_dry: FloatParam::new("Delay Dry", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),

            // Chorus
            chorus_rate: FloatParam::new(
                "Chorus Rate",
                1.5,
                FloatRange::Linear { min: 0.1, max: 5.0 },
            )
            .with_unit(" Hz"),
            chorus_depth: FloatParam::new(
                "Chorus Depth",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            chorus_mix: FloatParam::new(
                "Chorus Mix",
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            // Distortion
            distortion_type: EnumParam::new("Distortion Type", DistortionType::Tanh),
            distortion_drive: FloatParam::new(
                "Distortion Drive",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            distortion_mix: FloatParam::new(
                "Distortion Mix",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}
