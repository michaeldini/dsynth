// This file contains the expanded DSynthParams with ALL synth parameters
// Use this as a reference to expand plugin.rs

use crate::params::{FilterType, LFOWaveform, Waveform};
use nih_plug::prelude::*;

/// Complete plugin parameters that map to ALL SynthParams
#[derive(Params)]
pub struct DSynthParamsFull {
    // === MASTER ===
    #[id = "master_gain"]
    pub master_gain: FloatParam,

    #[id = "monophonic"]
    pub monophonic: BoolParam,

    // === OSCILLATOR 1 ===
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
    #[id = "osc1_phase"]
    pub osc1_phase: FloatParam,
    #[id = "osc1_shape"]
    pub osc1_shape: FloatParam,
    #[id = "osc1_solo"]
    pub osc1_solo: BoolParam,

    // === OSCILLATOR 2 ===
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
    #[id = "osc2_phase"]
    pub osc2_phase: FloatParam,
    #[id = "osc2_shape"]
    pub osc2_shape: FloatParam,
    #[id = "osc2_solo"]
    pub osc2_solo: BoolParam,

    // === OSCILLATOR 3 ===
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
    #[id = "osc3_phase"]
    pub osc3_phase: FloatParam,
    #[id = "osc3_shape"]
    pub osc3_shape: FloatParam,
    #[id = "osc3_solo"]
    pub osc3_solo: BoolParam,

    // === FILTER 1 ===
    #[id = "filter1_type"]
    pub filter1_type: EnumParam<FilterType>,
    #[id = "filter1_cutoff"]
    pub filter1_cutoff: FloatParam,
    #[id = "filter1_resonance"]
    pub filter1_resonance: FloatParam,
    #[id = "filter1_drive"]
    pub filter1_drive: FloatParam,
    #[id = "filter1_key_tracking"]
    pub filter1_key_tracking: FloatParam,

    // === FILTER 2 ===
    #[id = "filter2_type"]
    pub filter2_type: EnumParam<FilterType>,
    #[id = "filter2_cutoff"]
    pub filter2_cutoff: FloatParam,
    #[id = "filter2_resonance"]
    pub filter2_resonance: FloatParam,
    #[id = "filter2_drive"]
    pub filter2_drive: FloatParam,
    #[id = "filter2_key_tracking"]
    pub filter2_key_tracking: FloatParam,

    // === FILTER 3 ===
    #[id = "filter3_type"]
    pub filter3_type: EnumParam<FilterType>,
    #[id = "filter3_cutoff"]
    pub filter3_cutoff: FloatParam,
    #[id = "filter3_resonance"]
    pub filter3_resonance: FloatParam,
    #[id = "filter3_drive"]
    pub filter3_drive: FloatParam,
    #[id = "filter3_key_tracking"]
    pub filter3_key_tracking: FloatParam,

    // === FILTER ENVELOPE 1 ===
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

    // === FILTER ENVELOPE 2 ===
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

    // === FILTER ENVELOPE 3 ===
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

    // === LFO 1 ===
    #[id = "lfo1_waveform"]
    pub lfo1_waveform: EnumParam<LFOWaveform>,
    #[id = "lfo1_rate"]
    pub lfo1_rate: FloatParam,
    #[id = "lfo1_depth"]
    pub lfo1_depth: FloatParam,
    #[id = "lfo1_filter_amount"]
    pub lfo1_filter_amount: FloatParam,

    // === LFO 2 ===
    #[id = "lfo2_waveform"]
    pub lfo2_waveform: EnumParam<LFOWaveform>,
    #[id = "lfo2_rate"]
    pub lfo2_rate: FloatParam,
    #[id = "lfo2_depth"]
    pub lfo2_depth: FloatParam,
    #[id = "lfo2_filter_amount"]
    pub lfo2_filter_amount: FloatParam,

    // === LFO 3 ===
    #[id = "lfo3_waveform"]
    pub lfo3_waveform: EnumParam<LFOWaveform>,
    #[id = "lfo3_rate"]
    pub lfo3_rate: FloatParam,
    #[id = "lfo3_depth"]
    pub lfo3_depth: FloatParam,
    #[id = "lfo3_filter_amount"]
    pub lfo3_filter_amount: FloatParam,

    // === VELOCITY SENSITIVITY ===
    #[id = "velocity_amp"]
    pub velocity_amp: FloatParam,
    #[id = "velocity_filter"]
    pub velocity_filter: FloatParam,
    #[id = "velocity_filter_env"]
    pub velocity_filter_env: FloatParam,
}
