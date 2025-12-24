/// Parameter Descriptor System for CLAP Plugin
///
/// This module defines a comprehensive parameter descriptor system that abstracts
/// how parameters are exposed to CLAP hosts, while remaining format-agnostic.
///
/// All 45+ parameters in DSynth are mapped with:
/// - Unique u32 ID (consistent across sessions)
/// - Human-readable name and module
/// - Type information (Float, Bool, Enum)
/// - Range and normalization (0.0-1.0)
/// - Units for display
/// - Logarithmic/exponential skewing for freq/time params
/// - Automation support flags
use std::fmt;

/// Unique identifier for each parameter (must be < 0xFFFFFFFF)
/// Uses a namespace approach: upper 8 bits = module, lower 24 bits = parameter index
pub type ParamId = u32;

// Parameter ID namespace (upper 8 bits)
const MODULE_MASTER: u8 = 0x00;
const MODULE_OSC1: u8 = 0x01;
const MODULE_OSC2: u8 = 0x02;
const MODULE_OSC3: u8 = 0x03;
const MODULE_FILTER1: u8 = 0x04;
const MODULE_FILTER2: u8 = 0x05;
const MODULE_FILTER3: u8 = 0x06;
const MODULE_LFO1: u8 = 0x07;
const MODULE_LFO2: u8 = 0x08;
const MODULE_LFO3: u8 = 0x09;
const MODULE_ENVELOPE: u8 = 0x0A;
const MODULE_VELOCITY: u8 = 0x0B;
const MODULE_EFFECTS: u8 = 0x0C;

// Helper function to create parameter IDs
const fn make_param_id(module: u8, index: u32) -> ParamId {
    ((module as u32) << 24) | (index & 0xFFFFFF)
}

// Master parameters
pub const PARAM_MASTER_GAIN: ParamId = make_param_id(MODULE_MASTER, 0);
pub const PARAM_MONOPHONIC: ParamId = make_param_id(MODULE_MASTER, 1);

// Oscillator 1
pub const PARAM_OSC1_WAVEFORM: ParamId = make_param_id(MODULE_OSC1, 0);
pub const PARAM_OSC1_PITCH: ParamId = make_param_id(MODULE_OSC1, 1);
pub const PARAM_OSC1_DETUNE: ParamId = make_param_id(MODULE_OSC1, 2);
pub const PARAM_OSC1_GAIN: ParamId = make_param_id(MODULE_OSC1, 3);
pub const PARAM_OSC1_PAN: ParamId = make_param_id(MODULE_OSC1, 4);
pub const PARAM_OSC1_UNISON: ParamId = make_param_id(MODULE_OSC1, 5);
pub const PARAM_OSC1_UNISON_DETUNE: ParamId = make_param_id(MODULE_OSC1, 6);
pub const PARAM_OSC1_PHASE: ParamId = make_param_id(MODULE_OSC1, 7);
pub const PARAM_OSC1_SHAPE: ParamId = make_param_id(MODULE_OSC1, 8);
pub const PARAM_OSC1_FM_SOURCE: ParamId = make_param_id(MODULE_OSC1, 9);
pub const PARAM_OSC1_FM_AMOUNT: ParamId = make_param_id(MODULE_OSC1, 10);
pub const PARAM_OSC1_H1: ParamId = make_param_id(MODULE_OSC1, 11);
pub const PARAM_OSC1_H2: ParamId = make_param_id(MODULE_OSC1, 12);
pub const PARAM_OSC1_H3: ParamId = make_param_id(MODULE_OSC1, 13);
pub const PARAM_OSC1_H4: ParamId = make_param_id(MODULE_OSC1, 14);
pub const PARAM_OSC1_H5: ParamId = make_param_id(MODULE_OSC1, 15);
pub const PARAM_OSC1_H6: ParamId = make_param_id(MODULE_OSC1, 16);
pub const PARAM_OSC1_H7: ParamId = make_param_id(MODULE_OSC1, 17);
pub const PARAM_OSC1_H8: ParamId = make_param_id(MODULE_OSC1, 18);
pub const PARAM_OSC1_SOLO: ParamId = make_param_id(MODULE_OSC1, 19);

// Oscillator 2 (same structure)
pub const PARAM_OSC2_WAVEFORM: ParamId = make_param_id(MODULE_OSC2, 0);
pub const PARAM_OSC2_PITCH: ParamId = make_param_id(MODULE_OSC2, 1);
pub const PARAM_OSC2_DETUNE: ParamId = make_param_id(MODULE_OSC2, 2);
pub const PARAM_OSC2_GAIN: ParamId = make_param_id(MODULE_OSC2, 3);
pub const PARAM_OSC2_PAN: ParamId = make_param_id(MODULE_OSC2, 4);
pub const PARAM_OSC2_UNISON: ParamId = make_param_id(MODULE_OSC2, 5);
pub const PARAM_OSC2_UNISON_DETUNE: ParamId = make_param_id(MODULE_OSC2, 6);
pub const PARAM_OSC2_PHASE: ParamId = make_param_id(MODULE_OSC2, 7);
pub const PARAM_OSC2_SHAPE: ParamId = make_param_id(MODULE_OSC2, 8);
pub const PARAM_OSC2_FM_SOURCE: ParamId = make_param_id(MODULE_OSC2, 9);
pub const PARAM_OSC2_FM_AMOUNT: ParamId = make_param_id(MODULE_OSC2, 10);
pub const PARAM_OSC2_H1: ParamId = make_param_id(MODULE_OSC2, 11);
pub const PARAM_OSC2_H2: ParamId = make_param_id(MODULE_OSC2, 12);
pub const PARAM_OSC2_H3: ParamId = make_param_id(MODULE_OSC2, 13);
pub const PARAM_OSC2_H4: ParamId = make_param_id(MODULE_OSC2, 14);
pub const PARAM_OSC2_H5: ParamId = make_param_id(MODULE_OSC2, 15);
pub const PARAM_OSC2_H6: ParamId = make_param_id(MODULE_OSC2, 16);
pub const PARAM_OSC2_H7: ParamId = make_param_id(MODULE_OSC2, 17);
pub const PARAM_OSC2_H8: ParamId = make_param_id(MODULE_OSC2, 18);
pub const PARAM_OSC2_SOLO: ParamId = make_param_id(MODULE_OSC2, 19);

// Oscillator 3 (same structure)
pub const PARAM_OSC3_WAVEFORM: ParamId = make_param_id(MODULE_OSC3, 0);
pub const PARAM_OSC3_PITCH: ParamId = make_param_id(MODULE_OSC3, 1);
pub const PARAM_OSC3_DETUNE: ParamId = make_param_id(MODULE_OSC3, 2);
pub const PARAM_OSC3_GAIN: ParamId = make_param_id(MODULE_OSC3, 3);
pub const PARAM_OSC3_PAN: ParamId = make_param_id(MODULE_OSC3, 4);
pub const PARAM_OSC3_UNISON: ParamId = make_param_id(MODULE_OSC3, 5);
pub const PARAM_OSC3_UNISON_DETUNE: ParamId = make_param_id(MODULE_OSC3, 6);
pub const PARAM_OSC3_PHASE: ParamId = make_param_id(MODULE_OSC3, 7);
pub const PARAM_OSC3_SHAPE: ParamId = make_param_id(MODULE_OSC3, 8);
pub const PARAM_OSC3_FM_SOURCE: ParamId = make_param_id(MODULE_OSC3, 9);
pub const PARAM_OSC3_FM_AMOUNT: ParamId = make_param_id(MODULE_OSC3, 10);
pub const PARAM_OSC3_H1: ParamId = make_param_id(MODULE_OSC3, 11);
pub const PARAM_OSC3_H2: ParamId = make_param_id(MODULE_OSC3, 12);
pub const PARAM_OSC3_H3: ParamId = make_param_id(MODULE_OSC3, 13);
pub const PARAM_OSC3_H4: ParamId = make_param_id(MODULE_OSC3, 14);
pub const PARAM_OSC3_H5: ParamId = make_param_id(MODULE_OSC3, 15);
pub const PARAM_OSC3_H6: ParamId = make_param_id(MODULE_OSC3, 16);
pub const PARAM_OSC3_H7: ParamId = make_param_id(MODULE_OSC3, 17);
pub const PARAM_OSC3_H8: ParamId = make_param_id(MODULE_OSC3, 18);
pub const PARAM_OSC3_SOLO: ParamId = make_param_id(MODULE_OSC3, 19);

// Filter 1
pub const PARAM_FILTER1_TYPE: ParamId = make_param_id(MODULE_FILTER1, 0);
pub const PARAM_FILTER1_CUTOFF: ParamId = make_param_id(MODULE_FILTER1, 1);
pub const PARAM_FILTER1_RESONANCE: ParamId = make_param_id(MODULE_FILTER1, 2);
pub const PARAM_FILTER1_BANDWIDTH: ParamId = make_param_id(MODULE_FILTER1, 3);
pub const PARAM_FILTER1_KEY_TRACKING: ParamId = make_param_id(MODULE_FILTER1, 4);

// Filter 2
pub const PARAM_FILTER2_TYPE: ParamId = make_param_id(MODULE_FILTER2, 0);
pub const PARAM_FILTER2_CUTOFF: ParamId = make_param_id(MODULE_FILTER2, 1);
pub const PARAM_FILTER2_RESONANCE: ParamId = make_param_id(MODULE_FILTER2, 2);
pub const PARAM_FILTER2_BANDWIDTH: ParamId = make_param_id(MODULE_FILTER2, 3);
pub const PARAM_FILTER2_KEY_TRACKING: ParamId = make_param_id(MODULE_FILTER2, 4);

// Filter 3
pub const PARAM_FILTER3_TYPE: ParamId = make_param_id(MODULE_FILTER3, 0);
pub const PARAM_FILTER3_CUTOFF: ParamId = make_param_id(MODULE_FILTER3, 1);
pub const PARAM_FILTER3_RESONANCE: ParamId = make_param_id(MODULE_FILTER3, 2);
pub const PARAM_FILTER3_BANDWIDTH: ParamId = make_param_id(MODULE_FILTER3, 3);
pub const PARAM_FILTER3_KEY_TRACKING: ParamId = make_param_id(MODULE_FILTER3, 4);

// LFO 1
pub const PARAM_LFO1_WAVEFORM: ParamId = make_param_id(MODULE_LFO1, 0);
pub const PARAM_LFO1_RATE: ParamId = make_param_id(MODULE_LFO1, 1);
pub const PARAM_LFO1_DEPTH: ParamId = make_param_id(MODULE_LFO1, 2);
pub const PARAM_LFO1_FILTER_AMOUNT: ParamId = make_param_id(MODULE_LFO1, 3);
pub const PARAM_LFO1_PITCH_AMOUNT: ParamId = make_param_id(MODULE_LFO1, 4);
pub const PARAM_LFO1_GAIN_AMOUNT: ParamId = make_param_id(MODULE_LFO1, 5);
pub const PARAM_LFO1_PAN_AMOUNT: ParamId = make_param_id(MODULE_LFO1, 6);
pub const PARAM_LFO1_PWM_AMOUNT: ParamId = make_param_id(MODULE_LFO1, 7);

// LFO 2
pub const PARAM_LFO2_WAVEFORM: ParamId = make_param_id(MODULE_LFO2, 0);
pub const PARAM_LFO2_RATE: ParamId = make_param_id(MODULE_LFO2, 1);
pub const PARAM_LFO2_DEPTH: ParamId = make_param_id(MODULE_LFO2, 2);
pub const PARAM_LFO2_FILTER_AMOUNT: ParamId = make_param_id(MODULE_LFO2, 3);
pub const PARAM_LFO2_PITCH_AMOUNT: ParamId = make_param_id(MODULE_LFO2, 4);
pub const PARAM_LFO2_GAIN_AMOUNT: ParamId = make_param_id(MODULE_LFO2, 5);
pub const PARAM_LFO2_PAN_AMOUNT: ParamId = make_param_id(MODULE_LFO2, 6);
pub const PARAM_LFO2_PWM_AMOUNT: ParamId = make_param_id(MODULE_LFO2, 7);

// LFO 3
pub const PARAM_LFO3_WAVEFORM: ParamId = make_param_id(MODULE_LFO3, 0);
pub const PARAM_LFO3_RATE: ParamId = make_param_id(MODULE_LFO3, 1);
pub const PARAM_LFO3_DEPTH: ParamId = make_param_id(MODULE_LFO3, 2);
pub const PARAM_LFO3_FILTER_AMOUNT: ParamId = make_param_id(MODULE_LFO3, 3);
pub const PARAM_LFO3_PITCH_AMOUNT: ParamId = make_param_id(MODULE_LFO3, 4);
pub const PARAM_LFO3_GAIN_AMOUNT: ParamId = make_param_id(MODULE_LFO3, 5);
pub const PARAM_LFO3_PAN_AMOUNT: ParamId = make_param_id(MODULE_LFO3, 6);
pub const PARAM_LFO3_PWM_AMOUNT: ParamId = make_param_id(MODULE_LFO3, 7);

// Envelope (shared by all voices)
pub const PARAM_ENVELOPE_ATTACK: ParamId = make_param_id(MODULE_ENVELOPE, 0);
pub const PARAM_ENVELOPE_DECAY: ParamId = make_param_id(MODULE_ENVELOPE, 1);
pub const PARAM_ENVELOPE_SUSTAIN: ParamId = make_param_id(MODULE_ENVELOPE, 2);
pub const PARAM_ENVELOPE_RELEASE: ParamId = make_param_id(MODULE_ENVELOPE, 3);

// Velocity sensitivity
pub const PARAM_VELOCITY_AMP: ParamId = make_param_id(MODULE_VELOCITY, 0);
pub const PARAM_VELOCITY_FILTER: ParamId = make_param_id(MODULE_VELOCITY, 1);

// Effects
pub const PARAM_REVERB_ROOM_SIZE: ParamId = make_param_id(MODULE_EFFECTS, 0);
pub const PARAM_REVERB_DAMPING: ParamId = make_param_id(MODULE_EFFECTS, 1);
pub const PARAM_REVERB_WET: ParamId = make_param_id(MODULE_EFFECTS, 2);
pub const PARAM_REVERB_DRY: ParamId = make_param_id(MODULE_EFFECTS, 3);
pub const PARAM_REVERB_WIDTH: ParamId = make_param_id(MODULE_EFFECTS, 4);
pub const PARAM_DELAY_TIME_MS: ParamId = make_param_id(MODULE_EFFECTS, 5);
pub const PARAM_DELAY_FEEDBACK: ParamId = make_param_id(MODULE_EFFECTS, 6);
pub const PARAM_DELAY_WET: ParamId = make_param_id(MODULE_EFFECTS, 7);
pub const PARAM_DELAY_DRY: ParamId = make_param_id(MODULE_EFFECTS, 8);
pub const PARAM_CHORUS_RATE: ParamId = make_param_id(MODULE_EFFECTS, 9);
pub const PARAM_CHORUS_DEPTH: ParamId = make_param_id(MODULE_EFFECTS, 10);
pub const PARAM_CHORUS_MIX: ParamId = make_param_id(MODULE_EFFECTS, 11);
pub const PARAM_DISTORTION_TYPE: ParamId = make_param_id(MODULE_EFFECTS, 12);
pub const PARAM_DISTORTION_DRIVE: ParamId = make_param_id(MODULE_EFFECTS, 13);
pub const PARAM_DISTORTION_MIX: ParamId = make_param_id(MODULE_EFFECTS, 14);

/// Value skewing for logarithmic/exponential parameter curves
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueSkew {
    /// Linear: value = normalized (0.0 to 1.0)
    Linear,
    /// Logarithmic: useful for frequencies, time constants
    /// Formula: value = min * (max / min) ^ normalized
    Logarithmic,
    /// Exponential with custom exponent
    /// Formula: value = min + (max - min) * normalized ^ exponent
    Exponential { exponent: f32 },
}

/// Parameter type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
    /// Float parameter with range and skewing
    Float { min: f32, max: f32, skew: ValueSkew },
    /// Boolean parameter (on/off)
    Bool,
    /// Enumerated parameter with discrete values
    Enum { variants: Vec<String> },
    /// Integer parameter with range
    Int { min: i32, max: i32 },
}

/// Automation capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationState {
    /// Parameter cannot be automated
    None,
    /// Parameter can be read (modulated) but not written by host
    Read,
    /// Parameter can be written by host (automatable)
    ReadWrite,
}

/// Complete descriptor for a single parameter
#[derive(Debug, Clone)]
pub struct ParamDescriptor {
    /// Unique parameter ID (< 0xFFFFFFFF)
    pub id: ParamId,
    /// Human-readable name ("Master Gain", "Osc 1 Pitch", etc.)
    pub name: String,
    /// Module name for grouping ("Master", "Oscillator 1", "Filter 1", etc.)
    pub module: String,
    /// Parameter type (Float, Bool, Enum, Int)
    pub param_type: ParamType,
    /// Default value (normalized 0.0-1.0)
    pub default: f32,
    /// Unit string for display ("Hz", "dB", "ms", "%", etc.)
    pub unit: Option<String>,
    /// Automation capabilities
    pub automation: AutomationState,
}

impl ParamDescriptor {
    /// Create a new float parameter descriptor
    pub fn float(
        id: ParamId,
        name: impl Into<String>,
        module: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
        unit: Option<&str>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            module: module.into(),
            param_type: ParamType::Float {
                min,
                max,
                skew: ValueSkew::Linear,
            },
            default: Self::normalize(default, min, max),
            unit: unit.map(|s| s.to_string()),
            automation: AutomationState::ReadWrite,
        }
    }

    /// Create a new float parameter with logarithmic skewing
    pub fn float_log(
        id: ParamId,
        name: impl Into<String>,
        module: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
        unit: Option<&str>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            module: module.into(),
            param_type: ParamType::Float {
                min,
                max,
                skew: ValueSkew::Logarithmic,
            },
            default: Self::normalize_log(default, min, max),
            unit: unit.map(|s| s.to_string()),
            automation: AutomationState::ReadWrite,
        }
    }

    /// Create a new float parameter with exponential skewing
    pub fn float_exp(
        id: ParamId,
        name: impl Into<String>,
        module: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
        exponent: f32,
        unit: Option<&str>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            module: module.into(),
            param_type: ParamType::Float {
                min,
                max,
                skew: ValueSkew::Exponential { exponent },
            },
            default: Self::normalize_exp(default, min, max, exponent),
            unit: unit.map(|s| s.to_string()),
            automation: AutomationState::ReadWrite,
        }
    }

    /// Create a new boolean parameter
    pub fn bool(
        id: ParamId,
        name: impl Into<String>,
        module: impl Into<String>,
        default: bool,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            module: module.into(),
            param_type: ParamType::Bool,
            default: if default { 1.0 } else { 0.0 },
            unit: None,
            automation: AutomationState::ReadWrite,
        }
    }

    /// Create a new enum parameter
    pub fn enum_param(
        id: ParamId,
        name: impl Into<String>,
        module: impl Into<String>,
        variants: Vec<String>,
        default_index: usize,
    ) -> Self {
        let default_normalized = if variants.is_empty() {
            0.0
        } else {
            (default_index as f32) / ((variants.len() - 1) as f32)
        };

        Self {
            id,
            name: name.into(),
            module: module.into(),
            param_type: ParamType::Enum { variants },
            default: default_normalized,
            unit: None,
            automation: AutomationState::ReadWrite,
        }
    }

    /// Create a new integer parameter
    pub fn int(
        id: ParamId,
        name: impl Into<String>,
        module: impl Into<String>,
        min: i32,
        max: i32,
        default: i32,
    ) -> Self {
        let default_normalized = if max > min {
            ((default - min) as f32) / ((max - min) as f32)
        } else {
            0.0
        };

        Self {
            id,
            name: name.into(),
            module: module.into(),
            param_type: ParamType::Int { min, max },
            default: default_normalized,
            unit: None,
            automation: AutomationState::ReadWrite,
        }
    }

    /// Denormalize a value (0.0-1.0) to the actual parameter range
    pub fn denormalize(&self, normalized: f32) -> f32 {
        match &self.param_type {
            ParamType::Float { min, max, skew } => match skew {
                ValueSkew::Linear => min + (max - min) * normalized,
                ValueSkew::Logarithmic => {
                    if *min <= 0.0 {
                        // Fallback to linear for invalid ranges
                        min + (max - min) * normalized
                    } else {
                        min * (max / min).powf(normalized)
                    }
                }
                ValueSkew::Exponential { exponent } => {
                    min + (max - min) * normalized.powf(*exponent)
                }
            },
            ParamType::Bool => {
                if normalized > 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            ParamType::Enum { variants } => {
                let index = (normalized * (variants.len() - 1) as f32).round() as usize;
                index as f32
            }
            ParamType::Int { min, max } => {
                let range = max - min;
                ((normalized * range as f32).round() as i32 + min) as f32
            }
        }
    }

    /// Normalize a denormalized value back to 0.0-1.0 range
    pub fn normalize_value(&self, value: f32) -> f32 {
        match &self.param_type {
            ParamType::Float { min, max, skew } => match skew {
                ValueSkew::Linear => Self::normalize(value, *min, *max),
                ValueSkew::Logarithmic => Self::normalize_log(value, *min, *max),
                ValueSkew::Exponential { exponent } => {
                    Self::normalize_exp(value, *min, *max, *exponent)
                }
            },
            ParamType::Bool => {
                if value > 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            ParamType::Enum { variants } => {
                let count = variants.len() as f32;
                if count <= 1.0 {
                    0.0
                } else {
                    (value / (count - 1.0)).clamp(0.0, 1.0)
                }
            }
            ParamType::Int { min, max } => {
                let range = (*max - *min) as f32;
                if range <= 0.0 {
                    0.0
                } else {
                    ((value - *min as f32) / range).clamp(0.0, 1.0)
                }
            }
        }
    }

    /// Normalize a value to 0.0-1.0 range (linear)
    fn normalize(value: f32, min: f32, max: f32) -> f32 {
        if max <= min {
            0.0
        } else {
            ((value - min) / (max - min)).clamp(0.0, 1.0)
        }
    }

    /// Normalize a value to 0.0-1.0 range (logarithmic)
    fn normalize_log(value: f32, min: f32, max: f32) -> f32 {
        if min <= 0.0 || max <= 0.0 {
            Self::normalize(value, min, max)
        } else {
            ((value / min).ln() / (max / min).ln()).clamp(0.0, 1.0)
        }
    }

    /// Normalize a value to 0.0-1.0 range (exponential)
    fn normalize_exp(value: f32, min: f32, max: f32, exponent: f32) -> f32 {
        if max <= min || exponent == 0.0 {
            Self::normalize(value, min, max)
        } else {
            let normalized = Self::normalize(value, min, max);
            // Invert the exponential curve: if denormalize is v^exp, then normalize is v^(1/exp)
            normalized.powf(1.0 / exponent).clamp(0.0, 1.0)
        }
    }

    /// Format a normalized value (0.0-1.0) as a human-readable string with units
    pub fn format_value(&self, normalized: f32) -> String {
        match &self.param_type {
            ParamType::Float { .. } => {
                let value = self.denormalize(normalized);

                // Format based on unit type
                match self.unit.as_deref() {
                    Some("Hz") => {
                        // Frequency: 1 decimal for <100, no decimals for >=100
                        if value < 100.0 {
                            format!("{:.1} Hz", value)
                        } else {
                            format!("{:.0} Hz", value)
                        }
                    }
                    Some("ms") => format!("{:.2} ms", value),
                    Some("s") => format!("{:.2} s", value),
                    Some("semitones") => format!("{:.1} semitones", value),
                    Some("cents") => format!("{:.1} cents", value),
                    Some("%") => format!("{:.0}%", value * 100.0),
                    Some("dB") => format!("{:.1} dB", value),
                    Some("lin") => format!("{:.2}", value),
                    Some("") | None => format!("{:.2}", value),
                    Some(unit) => format!("{:.2} {}", value, unit),
                }
            }
            ParamType::Bool => {
                if normalized > 0.5 {
                    "On".to_string()
                } else {
                    "Off".to_string()
                }
            }
            ParamType::Enum { variants } => {
                let index = (normalized * (variants.len() - 1) as f32).round() as usize;
                let index = index.min(variants.len().saturating_sub(1));
                variants
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string())
            }
            ParamType::Int { .. } => {
                let value = self.denormalize(normalized) as i32;
                format!("{}", value)
            }
        }
    }
}

impl fmt::Display for ParamDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) [{}]",
            self.name,
            self.module,
            match &self.param_type {
                ParamType::Float { .. } => "Float",
                ParamType::Bool => "Bool",
                ParamType::Enum { .. } => "Enum",
                ParamType::Int { .. } => "Int",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_ids_unique() {
        // Ensure all parameter IDs are unique
        let ids = vec![
            PARAM_MASTER_GAIN,
            PARAM_MONOPHONIC,
            PARAM_OSC1_WAVEFORM,
            PARAM_OSC1_PITCH,
            PARAM_FILTER1_CUTOFF,
            PARAM_FILTER1_RESONANCE,
            PARAM_LFO1_RATE,
            PARAM_ENVELOPE_ATTACK,
            PARAM_VELOCITY_AMP,
            PARAM_REVERB_ROOM_SIZE,
        ];

        let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, ids.len(), "Parameter IDs must be unique");
    }

    #[test]
    fn test_linear_normalization() {
        let desc = ParamDescriptor::float(0, "Test", "Test", 0.0, 100.0, 50.0, Some("units"));
        let denorm = desc.denormalize(0.5);
        assert!((denorm - 50.0).abs() < 0.01, "Linear normalization failed");
    }

    #[test]
    fn test_log_normalization() {
        let desc =
            ParamDescriptor::float_log(0, "Frequency", "Test", 20.0, 20000.0, 1000.0, Some("Hz"));
        let denorm = desc.denormalize(0.5);
        // Logarithmic middle should be around sqrt(20 * 20000) ≈ 632
        assert!(
            denorm > 500.0 && denorm < 700.0,
            "Log normalization failed: {}",
            denorm
        );
    }

    #[test]
    fn test_enum_denormalization() {
        let variants = vec!["Sine".to_string(), "Saw".to_string(), "Square".to_string()];
        let desc = ParamDescriptor::enum_param(0, "Waveform", "Test", variants, 1);

        assert!((desc.denormalize(0.0) - 0.0).abs() < 0.01);
        assert!((desc.denormalize(0.5) - 1.0).abs() < 0.01);
        assert!((desc.denormalize(1.0) - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_param_id_namespace() {
        // Verify that parameter IDs use namespacing correctly
        let master_id = PARAM_MASTER_GAIN;
        let osc1_id = PARAM_OSC1_PITCH;
        let filter1_id = PARAM_FILTER1_CUTOFF;

        let master_module = (master_id >> 24) as u8;
        let osc1_module = (osc1_id >> 24) as u8;
        let filter1_module = (filter1_id >> 24) as u8;

        assert_eq!(master_module, MODULE_MASTER);
        assert_eq!(osc1_module, MODULE_OSC1);
        assert_eq!(filter1_module, MODULE_FILTER1);
    }
}
