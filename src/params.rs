use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "vst")]
use nih_plug::prelude::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "vst", derive(Enum))]
pub enum Waveform {
    #[default]
    Sine,
    Saw,
    Square,
    Triangle,
    Pulse,
    WhiteNoise,
    PinkNoise,
    Additive,
}

impl fmt::Display for Waveform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Waveform::Sine => write!(f, "Sine"),
            Waveform::Saw => write!(f, "Saw"),
            Waveform::Square => write!(f, "Square"),
            Waveform::Triangle => write!(f, "Triangle"),
            Waveform::Pulse => write!(f, "Pulse"),
            Waveform::WhiteNoise => write!(f, "White Noise"),
            Waveform::PinkNoise => write!(f, "Pink Noise"),
            Waveform::Additive => write!(f, "Additive"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "vst", derive(Enum))]
pub enum FilterType {
    #[default]
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
    #[serde(default)]
    pub fm_source: Option<usize>, // FM source oscillator index (0-2), None = no FM
    #[serde(default)]
    pub fm_amount: f32, // FM modulation depth (0.0 to 10.0)
    #[serde(default)]
    pub additive_harmonics: [f32; 8], // Harmonic amplitudes for additive synthesis (0.0 to 1.0)
}

impl Default for OscillatorParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            pitch: 0.0,
            detune: 0.0,
            gain: 0.0, // Default to 0.0 (off) - individual oscillators are enabled explicitly in SynthParams::default()
            pan: 0.0,
            unison: 1,
            fm_source: None,
            fm_amount: 0.0,
            unison_detune: 10.0,
            phase: 0.0,
            shape: 0.0,
            solo: false,
            additive_harmonics: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // Default: fundamental only
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FilterParams {
    pub filter_type: FilterType,
    pub cutoff: f32,       // Hz, 20.0 to 20000.0
    pub resonance: f32,    // Q factor, 0.5 to 10.0
    pub bandwidth: f32,    // Bandwidth in octaves for bandpass (0.1 to 4.0)
    pub key_tracking: f32, // Key tracking amount (0.0 to 1.0)
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Lowpass,
            cutoff: 1000.0,
            resonance: 0.707,
            bandwidth: 1.0, // 1 octave for bandpass
            key_tracking: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnvelopeParams {
    pub attack: f32,  // seconds, 0.001 to 5.0
    pub decay: f32,   // seconds, 0.001 to 5.0
    pub sustain: f32, // level, 0.0 to 1.0
    pub release: f32, // seconds, 0.001 to 5.0
}

impl Default for EnvelopeParams {
    fn default() -> Self {
        // Match dsp::envelope::Envelope defaults
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
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
            amount: 0.0, // Disabled by default so cutoff slider is more immediately audible
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "vst", derive(Enum))]
pub enum LFOWaveform {
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "vst", derive(Enum))]
pub enum DistortionType {
    #[default]
    Tanh,
    SoftClip,
    HardClip,
    Cubic,
}

impl fmt::Display for DistortionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistortionType::Tanh => write!(f, "Tanh"),
            DistortionType::SoftClip => write!(f, "Soft Clip"),
            DistortionType::HardClip => write!(f, "Hard Clip"),
            DistortionType::Cubic => write!(f, "Cubic"),
        }
    }
}

impl From<DistortionType> for crate::dsp::effects::distortion::DistortionType {
    fn from(dt: DistortionType) -> Self {
        match dt {
            DistortionType::Tanh => crate::dsp::effects::distortion::DistortionType::Tanh,
            DistortionType::SoftClip => crate::dsp::effects::distortion::DistortionType::SoftClip,
            DistortionType::HardClip => crate::dsp::effects::distortion::DistortionType::HardClip,
            DistortionType::Cubic => crate::dsp::effects::distortion::DistortionType::Cubic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LFOParams {
    pub waveform: LFOWaveform,
    pub rate: f32,          // Hz, 0.01 to 20.0
    pub depth: f32,         // 0.0 to 1.0
    pub filter_amount: f32, // Filter modulation in Hz, 0.0 to 5000.0

    // LFO routing matrix (global, affects all oscillators)
    #[serde(default)]
    pub pitch_amount: f32, // Pitch modulation in cents, 0.0 to 100.0 (bipolar: ±100 cents)

    #[serde(default)]
    pub gain_amount: f32, // Gain modulation, 0.0 to 1.0 (bipolar: ±0.5)

    #[serde(default)]
    pub pan_amount: f32, // Pan modulation, 0.0 to 1.0 (bipolar: ±1.0 for full stereo)

    #[serde(default)]
    pub pwm_amount: f32, // PWM/shape modulation, 0.0 to 1.0 (bipolar: ±1.0)
}

impl Default for LFOParams {
    fn default() -> Self {
        Self {
            waveform: LFOWaveform::Sine,
            rate: 2.0,
            depth: 0.0,         // Disabled by default
            filter_amount: 0.0, // Disabled by default
            pitch_amount: 0.0,  // Disabled by default
            gain_amount: 0.0,   // Disabled by default
            pan_amount: 0.0,    // Disabled by default
            pwm_amount: 0.0,    // Disabled by default
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
}

impl Default for VelocityParams {
    fn default() -> Self {
        Self {
            amp_sensitivity: 0.7,
            filter_sensitivity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReverbParams {
    pub room_size: f32, // 0.0 to 1.0
    pub damping: f32,   // 0.0 to 1.0 (0.0 = bright, 1.0 = dark)
    pub wet: f32,       // 0.0 to 1.0
    pub dry: f32,       // 0.0 to 1.0
    pub width: f32,     // 0.0 to 1.0 (stereo width)
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            room_size: 0.5,
            damping: 0.5,
            wet: 0.33,
            dry: 0.67,
            width: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DelayParams {
    pub time_ms: f32,  // 1.0 to 2000.0
    pub feedback: f32, // 0.0 to 0.95
    pub wet: f32,      // 0.0 to 1.0
    pub dry: f32,      // 0.0 to 1.0
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            time_ms: 500.0,
            feedback: 0.3,
            wet: 0.3,
            dry: 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChorusParams {
    pub rate: f32,  // 0.1 to 5.0 Hz
    pub depth: f32, // 0.0 to 1.0
    pub mix: f32,   // 0.0 to 1.0
}

impl Default for ChorusParams {
    fn default() -> Self {
        Self {
            rate: 0.5,
            depth: 0.5,
            mix: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DistortionParams {
    pub drive: f32, // 0.0 to 1.0
    pub mix: f32,   // 0.0 to 1.0
    pub dist_type: DistortionType,
}

impl Default for DistortionParams {
    fn default() -> Self {
        Self {
            drive: 0.0,
            mix: 0.5,
            dist_type: DistortionType::Tanh,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EffectsParams {
    pub reverb: ReverbParams,
    pub delay: DelayParams,
    pub chorus: ChorusParams,
    pub distortion: DistortionParams,
}

impl Default for EffectsParams {
    fn default() -> Self {
        Self {
            reverb: ReverbParams::default(),
            delay: DelayParams::default(),
            chorus: ChorusParams::default(),
            distortion: DistortionParams::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SynthParams {
    pub oscillators: [OscillatorParams; 3],
    pub filters: [FilterParams; 3],
    pub lfos: [LFOParams; 3],
    #[serde(default)]
    pub envelope: EnvelopeParams,
    pub velocity: VelocityParams,
    #[serde(default)]
    pub effects: EffectsParams,
    pub master_gain: f32, // 0.0 to 1.0
    pub monophonic: bool, // Monophonic mode - only one note at a time
}

impl Default for SynthParams {
    fn default() -> Self {
        // Create oscillator defaults with only the first oscillator enabled
        let mut osc1 = OscillatorParams::default();
        osc1.gain = 0.25; // Oscillator 1 is enabled

        let mut osc2 = OscillatorParams::default();
        osc2.waveform = Waveform::Saw; // Different waveform for variety
        osc2.gain = 0.0; // Oscillator 2 is disabled by default

        let mut osc3 = OscillatorParams::default();
        osc3.waveform = Waveform::Square; // Different waveform for variety
        osc3.gain = 0.0; // Oscillator 3 is disabled by default

        Self {
            oscillators: [osc1, osc2, osc3],
            filters: [FilterParams::default(); 3],
            lfos: [LFOParams::default(); 3],
            envelope: EnvelopeParams::default(),
            velocity: VelocityParams::default(),
            effects: EffectsParams::default(),
            master_gain: 0.5,
            monophonic: false,
        }
    }
}
