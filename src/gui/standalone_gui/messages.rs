use crate::params::{DistortionType, FilterType, LFOWaveform, Waveform};
use iced::keyboard;

/// Hierarchical message types to reduce boilerplate
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscTab {
    Basic,
    Harmonics,
}

#[derive(Debug, Clone)]
pub enum OscillatorMessage {
    WaveformChanged(Waveform),
    PitchChanged(f32),
    DetuneChanged(f32),
    GainChanged(f32),
    PanChanged(f32),
    UnisonChanged(usize),
    UnisonDetuneChanged(f32),
    PhaseChanged(f32),
    ShapeChanged(f32),
    SoloToggled(bool),
    FmSourceChanged(Option<usize>),
    FmAmountChanged(f32),
    AdditiveHarmonicChanged(usize, f32), // (harmonic_index, amplitude)
}

#[derive(Debug, Clone)]
pub enum FilterMessage {
    TypeChanged(FilterType),
    CutoffChanged(f32),
    ResonanceChanged(f32),
    BandwidthChanged(f32),
    KeyTrackingChanged(f32),
}

#[derive(Debug, Clone)]
pub enum LFOMessage {
    WaveformChanged(LFOWaveform),
    RateChanged(f32),
    DepthChanged(f32),
    FilterAmountChanged(f32),
    PitchAmountChanged(f32),
    GainAmountChanged(f32),
    PanAmountChanged(f32),
    PwmAmountChanged(f32),
}

#[derive(Debug, Clone)]
pub enum ReverbMessage {
    RoomSizeChanged(f32),
    DampingChanged(f32),
    WetChanged(f32),
    DryChanged(f32),
    WidthChanged(f32),
}

#[derive(Debug, Clone)]
pub enum DelayMessage {
    TimeChanged(f32),
    FeedbackChanged(f32),
    WetChanged(f32),
    DryChanged(f32),
}

#[derive(Debug, Clone)]
pub enum ChorusMessage {
    RateChanged(f32),
    DepthChanged(f32),
    MixChanged(f32),
}

#[derive(Debug, Clone)]
pub enum DistortionMessage {
    DriveChanged(f32),
    MixChanged(f32),
    TypeChanged(DistortionType),
}

#[derive(Debug, Clone)]
pub enum EnvelopeMessage {
    AttackChanged(f32),
    DecayChanged(f32),
    SustainChanged(f32),
    ReleaseChanged(f32),
}

#[derive(Debug, Clone)]
pub enum Message {
    // Indexed parameter groups
    Oscillator(usize, OscillatorMessage),
    Filter(usize, FilterMessage),
    LFO(usize, LFOMessage),

    // Effects
    Reverb(ReverbMessage),
    Delay(DelayMessage),
    Chorus(ChorusMessage),
    Distortion(DistortionMessage),

    // Envelope
    Envelope(EnvelopeMessage),

    // Velocity Sensitivity
    VelocityAmpChanged(f32),
    VelocityFilterChanged(f32),

    // Master
    MasterGainChanged(f32),
    MonophonicToggled(bool),
    PanicPressed,

    // Oscillator tabs
    OscTabChanged(usize, OscTab), // (oscillator_index, new_tab)

    // Keyboard events
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),

    // Preset management
    PresetNameChanged(String),
    SavePreset,
    LoadPreset,
    PresetLoaded(Box<Result<crate::params::SynthParams, String>>),
    Randomize,
}
