use nih_plug::prelude::*;
use std::sync::Arc;
use triple_buffer::Input;

use crate::audio::create_parameter_buffer;
use crate::audio::engine::SynthEngine;
use crate::params::{
    FilterEnvelopeParams, FilterParams, FilterType, LFOParams, LFOWaveform, OscillatorParams,
    SynthParams, VelocityParams, Waveform,
};

#[cfg(feature = "vst")]
use crate::gui::plugin_gui;

/// DSynth VST plugin wrapper
pub struct DSynthPlugin {
    params: Arc<DSynthParams>,
    params_producer: Input<SynthParams>,
    engine: SynthEngine,
    sample_rate: f32,
}

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

    #[id = "osc1_shape"]
    pub osc1_shape: FloatParam,

    // Oscillator 2 parameters
    #[id = "osc2_waveform"]
    pub osc2_waveform: EnumParam<Waveform>,

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

    #[id = "osc2_shape"]
    pub osc2_shape: FloatParam,

    // Oscillator 3 parameters
    #[id = "osc3_waveform"]
    pub osc3_waveform: EnumParam<Waveform>,

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

    #[id = "osc3_shape"]
    pub osc3_shape: FloatParam,

    // Filter 1 parameters
    #[id = "filter1_type"]
    pub filter1_type: EnumParam<FilterType>,

    #[id = "filter1_cutoff"]
    pub filter1_cutoff: FloatParam,

    #[id = "filter1_resonance"]
    pub filter1_resonance: FloatParam,

    #[id = "filter1_drive"]
    pub filter1_drive: FloatParam,

    #[id = "filter1_amount"]
    pub filter1_amount: FloatParam,

    // Filter 2 parameters
    #[id = "filter2_type"]
    pub filter2_type: EnumParam<FilterType>,

    #[id = "filter2_cutoff"]
    pub filter2_cutoff: FloatParam,

    #[id = "filter2_resonance"]
    pub filter2_resonance: FloatParam,

    #[id = "filter2_drive"]
    pub filter2_drive: FloatParam,

    // Filter 3 parameters
    #[id = "filter3_type"]
    pub filter3_type: EnumParam<FilterType>,

    #[id = "filter3_cutoff"]
    pub filter3_cutoff: FloatParam,

    #[id = "filter3_resonance"]
    pub filter3_resonance: FloatParam,

    #[id = "filter3_drive"]
    pub filter3_drive: FloatParam,

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
}

impl Default for DSynthPlugin {
    fn default() -> Self {
        let (producer, consumer) = create_parameter_buffer();
        Self {
            params: Arc::new(DSynthParams::default()),
            params_producer: producer,
            engine: SynthEngine::new(44100.0, consumer),
            sample_rate: 44100.0,
        }
    }
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
            osc1_shape: FloatParam::new(
                "Osc 1 Shape",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),

            // Oscillator 2
            osc2_waveform: EnumParam::new("Osc 2 Wave", Waveform::Saw),
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
            osc2_shape: FloatParam::new(
                "Osc 2 Shape",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),

            // Oscillator 3
            osc3_waveform: EnumParam::new("Osc 3 Wave", Waveform::Square),
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
            osc3_shape: FloatParam::new(
                "Osc 3 Shape",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
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
            filter1_drive: FloatParam::new(
                "Filter 1 Drive",
                1.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 10.0,
                },
            ),
            filter1_amount: FloatParam::new(
                "Filter 1 Amount",
                1.0,
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
            filter2_drive: FloatParam::new(
                "Filter 2 Drive",
                1.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 10.0,
                },
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
            filter3_drive: FloatParam::new(
                "Filter 3 Drive",
                1.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 10.0,
                },
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
        }
    }
}

impl DSynthPlugin {
    /// Convert DSynthParams to SynthParams for the audio engine
    fn convert_params(&self) -> SynthParams {
        let p = &self.params;

        SynthParams {
            master_gain: p.master_gain.value(),
            monophonic: p.monophonic.value(),

            oscillators: [
                OscillatorParams {
                    waveform: p.osc1_waveform.value(),
                    pitch: p.osc1_pitch.value(),
                    detune: p.osc1_detune.value(),
                    gain: p.osc1_gain.value(),
                    pan: p.osc1_pan.value(),
                    unison: p.osc1_unison.value() as usize,
                    unison_detune: p.osc1_unison_detune.value(),
                    phase: 0.0,
                    shape: p.osc1_shape.value(),
                    solo: false,
                },
                OscillatorParams {
                    waveform: p.osc2_waveform.value(),
                    pitch: p.osc2_pitch.value(),
                    detune: p.osc2_detune.value(),
                    gain: p.osc2_gain.value(),
                    pan: p.osc2_pan.value(),
                    unison: p.osc2_unison.value() as usize,
                    unison_detune: p.osc2_unison_detune.value(),
                    phase: 0.0,
                    shape: p.osc2_shape.value(),
                    solo: false,
                },
                OscillatorParams {
                    waveform: p.osc3_waveform.value(),
                    pitch: p.osc3_pitch.value(),
                    detune: p.osc3_detune.value(),
                    gain: p.osc3_gain.value(),
                    pan: p.osc3_pan.value(),
                    unison: p.osc3_unison.value() as usize,
                    unison_detune: p.osc3_unison_detune.value(),
                    phase: 0.0,
                    shape: p.osc3_shape.value(),
                    solo: false,
                },
            ],

            filters: [
                FilterParams {
                    filter_type: p.filter1_type.value(),
                    cutoff: p.filter1_cutoff.value(),
                    resonance: p.filter1_resonance.value(),
                    drive: p.filter1_drive.value(),
                    key_tracking: p.filter1_amount.value(), // Using filter1_amount for key_tracking
                },
                FilterParams {
                    filter_type: p.filter2_type.value(),
                    cutoff: p.filter2_cutoff.value(),
                    resonance: p.filter2_resonance.value(),
                    drive: p.filter2_drive.value(),
                    key_tracking: 0.0, // No key tracking parameter for filter 2 yet
                },
                FilterParams {
                    filter_type: p.filter3_type.value(),
                    cutoff: p.filter3_cutoff.value(),
                    resonance: p.filter3_resonance.value(),
                    drive: p.filter3_drive.value(),
                    key_tracking: 0.0, // No key tracking parameter for filter 3 yet
                },
            ],

            filter_envelopes: [
                FilterEnvelopeParams {
                    attack: p.fenv1_attack.value(),
                    decay: p.fenv1_decay.value(),
                    sustain: p.fenv1_sustain.value(),
                    release: p.fenv1_release.value(),
                    amount: p.fenv1_amount.value(),
                },
                FilterEnvelopeParams {
                    attack: p.fenv2_attack.value(),
                    decay: p.fenv2_decay.value(),
                    sustain: p.fenv2_sustain.value(),
                    release: p.fenv2_release.value(),
                    amount: p.fenv2_amount.value(),
                },
                FilterEnvelopeParams {
                    attack: p.fenv3_attack.value(),
                    decay: p.fenv3_decay.value(),
                    sustain: p.fenv3_sustain.value(),
                    release: p.fenv3_release.value(),
                    amount: p.fenv3_amount.value(),
                },
            ],

            lfos: [
                LFOParams {
                    waveform: p.lfo1_waveform.value(),
                    rate: p.lfo1_rate.value(),
                    depth: p.lfo1_depth.value(),
                    filter_amount: p.lfo1_filter_amount.value(),
                },
                LFOParams {
                    waveform: p.lfo2_waveform.value(),
                    rate: p.lfo2_rate.value(),
                    depth: p.lfo2_depth.value(),
                    filter_amount: p.lfo2_filter_amount.value(),
                },
                LFOParams {
                    waveform: p.lfo3_waveform.value(),
                    rate: p.lfo3_rate.value(),
                    depth: p.lfo3_depth.value(),
                    filter_amount: p.lfo3_filter_amount.value(),
                },
            ],

            velocity: VelocityParams {
                amp_sensitivity: p.velocity_amp.value(),
                filter_sensitivity: p.velocity_filter.value(),
                filter_env_sensitivity: p.velocity_filter_env.value(),
            },
        }
    }
}

impl Plugin for DSynthPlugin {
    const NAME: &'static str = "DSynth";
    const VENDOR: &'static str = "DSynth";
    const URL: &'static str = "https://github.com/yourusername/dsynth";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    #[cfg(feature = "vst")]
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        plugin_gui::create(self.params.clone(), plugin_gui::default_state())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Update sample rate
        self.sample_rate = buffer_config.sample_rate;

        // Recreate engine with new sample rate
        let (producer, consumer) = create_parameter_buffer();
        self.params_producer = producer;
        self.engine = SynthEngine::new(self.sample_rate, consumer);

        true
    }

    fn reset(&mut self) {
        // Reset all voices when transport stops or settings change
        self.engine.all_notes_off();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Convert and send parameters to engine
        let synth_params = self.convert_params();
        self.params_producer.write(synth_params);

        // Process MIDI events
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn { note, velocity, .. } => {
                    self.engine.note_on(note, velocity);
                }
                NoteEvent::NoteOff { note, .. } => {
                    self.engine.note_off(note);
                }
                NoteEvent::PolyPressure { .. } => {}
                NoteEvent::PolyVolume { .. } => {}
                NoteEvent::PolyPan { .. } => {}
                NoteEvent::PolyTuning { .. } => {}
                NoteEvent::PolyVibrato { .. } => {}
                NoteEvent::PolyExpression { .. } => {}
                NoteEvent::PolyBrightness { .. } => {}
                NoteEvent::MidiChannelPressure { .. } => {}
                NoteEvent::MidiPitchBend { .. } => {}
                NoteEvent::MidiCC { .. } => {}
                NoteEvent::MidiProgramChange { .. } => {}
                _ => {}
            }
        }

        // Get output channels
        let output = buffer.as_slice();
        let channels = output.len();

        if channels < 2 {
            return ProcessStatus::Normal;
        }

        let samples = output[0].len();

        // Process audio block
        for sample_idx in 0..samples {
            let sample = self.engine.process();

            // Write to stereo output
            output[0][sample_idx] = sample;
            output[1][sample_idx] = sample;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for DSynthPlugin {
    const CLAP_ID: &'static str = "com.dsynth.dsynth";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Digital Synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for DSynthPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"DSynthPluginVST3";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(DSynthPlugin);
nih_export_vst3!(DSynthPlugin);
