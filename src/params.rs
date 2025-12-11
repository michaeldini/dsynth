use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl fmt::Display for Waveform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Waveform::Sine => write!(f, "Sine"),
            Waveform::Saw => write!(f, "Saw"),
            Waveform::Square => write!(f, "Square"),
            Waveform::Triangle => write!(f, "Triangle"),
        }
    }
}

impl Default for Waveform {
    fn default() -> Self {
        Self::Sine
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FilterParams {
    pub filter_type: FilterType,
    pub cutoff: f32,    // Hz, 20.0 to 20000.0
    pub resonance: f32, // Q factor, 0.5 to 10.0
    pub drive: f32,     // Pre-filter drive/saturation (1.0 to 10.0)
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Lowpass,
            cutoff: 1000.0,
            resonance: 0.707,
            drive: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SynthParams {
    pub oscillators: [OscillatorParams; 3],
    pub filters: [FilterParams; 3],
    pub master_gain: f32, // 0.0 to 1.0
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            oscillators: [OscillatorParams::default(); 3],
            filters: [FilterParams::default(); 3],
            master_gain: 0.5,
        }
    }
}
