use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "vst")]
use nih_plug::prelude::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "vst", derive(Enum))]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
    Pulse,
}

impl fmt::Display for Waveform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Waveform::Sine => write!(f, "Sine"),
            Waveform::Saw => write!(f, "Saw"),
            Waveform::Square => write!(f, "Square"),
            Waveform::Triangle => write!(f, "Triangle"),
            Waveform::Pulse => write!(f, "Pulse"),
        }
    }
}

impl Default for Waveform {
    fn default() -> Self {
        Self::Sine
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "vst", derive(Enum))]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
}

impl fmt::Display for FilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterType::Lowpass => write!(f, "Lowpass"),
            FilterType::Highpass => write!(f, "Highpass"),
            FilterType::Bandpass => write!(f, "Bandpass"),
        }
    }
}

impl Default for FilterType {
    fn default() -> Self {
        Self::Lowpass
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OscillatorParams {
    pub waveform: Waveform,
    pub pitch: f32,         // In semitones, ±24
    pub detune: f32,        // In cents, ±50
    pub gain: f32,          // 0.0 to 1.0
    pub pan: f32,           // -1.0 (left) to 1.0 (right), 0.0 = center
    pub unison: usize,      // Number of unison voices (1-7)
    pub unison_detune: f32, // Unison spread in cents (0-50)
    pub phase: f32,         // Initial phase offset (0.0 to 1.0)
    pub shape: f32,         // Wave shaping amount (-1.0 to 1.0)
    pub solo: bool,         // Solo mode - when any osc is soloed, only soloed oscs are heard
}

impl Default for OscillatorParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            pitch: 0.0,
            detune: 0.0,
            gain: 0.33,
            pan: 0.0,
            unison: 1,
            unison_detune: 10.0,
            phase: 0.0,
            shape: 0.0,
            solo: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FilterParams {
    pub filter_type: FilterType,
    pub cutoff: f32,       // Hz, 20.0 to 20000.0
    pub resonance: f32,    // Q factor, 0.5 to 10.0
    pub drive: f32,        // Pre-filter drive/saturation (1.0 to 10.0)
    pub key_tracking: f32, // Key tracking amount (0.0 to 1.0)
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Lowpass,
            cutoff: 1000.0,
            resonance: 0.707,
            drive: 1.0,
            key_tracking: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FilterEnvelopeParams {
    pub attack: f32,  // seconds, 0.001 to 5.0
    pub decay: f32,   // seconds, 0.001 to 5.0
    pub sustain: f32, // level, 0.0 to 1.0
    pub release: f32, // seconds, 0.001 to 5.0
    pub amount: f32,  // modulation depth in Hz, -10000.0 to 10000.0
}

impl Default for FilterEnvelopeParams {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            amount: 2000.0, // 2kHz modulation range
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LFOWaveform {
    Sine,
    Triangle,
    Square,
    Saw,
}

impl fmt::Display for LFOWaveform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LFOWaveform::Sine => write!(f, "Sine"),
            LFOWaveform::Triangle => write!(f, "Triangle"),
            LFOWaveform::Square => write!(f, "Square"),
            LFOWaveform::Saw => write!(f, "Saw"),
        }
    }
}

impl Default for LFOWaveform {
    fn default() -> Self {
        Self::Sine
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LFOParams {
    pub waveform: LFOWaveform,
    pub rate: f32,          // Hz, 0.01 to 20.0
    pub depth: f32,         // 0.0 to 1.0
    pub filter_amount: f32, // Filter modulation in Hz, 0.0 to 5000.0
}

impl Default for LFOParams {
    fn default() -> Self {
        Self {
            waveform: LFOWaveform::Sine,
            rate: 2.0,
            depth: 0.5,
            filter_amount: 500.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VelocityParams {
    /// Velocity sensitivity for amplitude (0.0 = no velocity sensitivity, 1.0 = full sensitivity)
    ///
    /// Formula: `output_amplitude = 1.0 + amp_sensitivity * (velocity - 0.5)`
    /// - At velocity 0.0: amplitude is (1.0 - 0.5 * sensitivity)
    /// - At velocity 0.5: amplitude is exactly 1.0 (no change)
    /// - At velocity 1.0: amplitude is (1.0 + 0.5 * sensitivity)
    pub amp_sensitivity: f32,

    /// Velocity sensitivity for filter cutoff frequency (0.0 = no velocity sensitivity, 1.0 = full sensitivity)
    ///
    /// Formula: `cutoff_offset = filter_sensitivity * (velocity - 0.5)`
    /// Higher velocity raises the filter cutoff, lower velocity lowers it.
    pub filter_sensitivity: f32,

    /// Velocity sensitivity for filter envelope amount (0.0 = no velocity sensitivity, 1.0 = full sensitivity)
    ///
    /// Formula: `env_amount = 1.0 + filter_env_sensitivity * (velocity - 0.5)`
    /// Controls how much the filter envelope modulates the cutoff based on velocity.
    pub filter_env_sensitivity: f32,
}

impl Default for VelocityParams {
    fn default() -> Self {
        Self {
            amp_sensitivity: 0.7,
            filter_sensitivity: 0.5,
            filter_env_sensitivity: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SynthParams {
    pub oscillators: [OscillatorParams; 3],
    pub filters: [FilterParams; 3],
    pub filter_envelopes: [FilterEnvelopeParams; 3],
    pub lfos: [LFOParams; 3],
    pub velocity: VelocityParams,
    pub master_gain: f32, // 0.0 to 1.0
    pub monophonic: bool, // Monophonic mode - only one note at a time
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            oscillators: [OscillatorParams::default(); 3],
            filters: [FilterParams::default(); 3],
            filter_envelopes: [FilterEnvelopeParams::default(); 3],
            lfos: [LFOParams::default(); 3],
            velocity: VelocityParams::default(),
            master_gain: 0.5,
            monophonic: false,
        }
    }
}
