/// Kick Drum Synthesizer Parameters
/// Simplified parameter set optimized for kick drum synthesis
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KickParams {
    // Oscillator 1 - Body/Tone
    pub osc1_pitch_start: f32, // Starting pitch (Hz, typically 100-400Hz)
    pub osc1_pitch_end: f32,   // Ending pitch (Hz, typically 40-80Hz)
    pub osc1_pitch_decay: f32, // Pitch envelope decay time (ms, 10-500ms)
    pub osc1_level: f32,       // Body oscillator level (0.0-1.0)

    // Oscillator 2 - Click/Transient
    pub osc2_pitch_start: f32, // Click starting pitch (Hz, typically 1000-8000Hz)
    pub osc2_pitch_end: f32,   // Click ending pitch (Hz, typically 100-500Hz)
    pub osc2_pitch_decay: f32, // Click pitch decay time (ms, 5-100ms)
    pub osc2_level: f32,       // Click oscillator level (0.0-1.0)

    // Amplitude Envelope
    pub amp_attack: f32,  // Attack time (ms, 0.1-10ms)
    pub amp_decay: f32,   // Decay time (ms, 50-2000ms)
    pub amp_sustain: f32, // Sustain level (0.0-1.0, usually 0 for kicks)
    pub amp_release: f32, // Release time (ms, 10-500ms)

    // Filter
    pub filter_cutoff: f32,     // Lowpass cutoff frequency (Hz, 50-20000Hz)
    pub filter_resonance: f32,  // Filter resonance (0.0-1.0)
    pub filter_env_amount: f32, // Filter envelope modulation amount (-1.0 to 1.0)
    pub filter_env_decay: f32,  // Filter envelope decay time (ms, 10-500ms)

    // Distortion/Saturation
    pub distortion_amount: f32, // Distortion/saturation amount (0.0-1.0)
    pub distortion_type: DistortionType, // Type of distortion

    // Master
    pub master_level: f32,         // Master output level (0.0-1.0)
    pub velocity_sensitivity: f32, // How much velocity affects amplitude (0.0-1.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistortionType {
    Soft,     // Soft clipping/saturation
    Hard,     // Hard clipping
    Tube,     // Tube-style warmth
    Foldback, // Foldback distortion
}

impl Default for KickParams {
    fn default() -> Self {
        Self {
            // Osc 1 - Body (classic 808-style pitch sweep)
            osc1_pitch_start: 150.0,
            osc1_pitch_end: 55.0,
            osc1_pitch_decay: 100.0,
            osc1_level: 0.8,

            // Osc 2 - Click (transient definition)
            osc2_pitch_start: 3000.0,
            osc2_pitch_end: 200.0,
            osc2_pitch_decay: 20.0,
            osc2_level: 0.3,

            // Amplitude envelope (punchy, no sustain)
            amp_attack: 0.5,
            amp_decay: 300.0,
            amp_sustain: 0.0,
            amp_release: 50.0,

            // Filter (keeps low end, removes harshness)
            filter_cutoff: 8000.0,
            filter_resonance: 0.2,
            filter_env_amount: 0.3,
            filter_env_decay: 150.0,

            // Distortion (subtle warmth)
            distortion_amount: 0.15,
            distortion_type: DistortionType::Soft,

            // Master
            master_level: 0.8,
            velocity_sensitivity: 0.5,
        }
    }
}

impl KickParams {
    /// Create a new instance with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Classic 808-style kick preset
    pub fn preset_808() -> Self {
        Self {
            osc1_pitch_start: 180.0,
            osc1_pitch_end: 50.0,
            osc1_pitch_decay: 120.0,
            osc1_level: 0.9,
            osc2_pitch_start: 2500.0,
            osc2_pitch_end: 150.0,
            osc2_pitch_decay: 15.0,
            osc2_level: 0.2,
            amp_attack: 0.3,
            amp_decay: 400.0,
            amp_sustain: 0.0,
            amp_release: 50.0,
            filter_cutoff: 6000.0,
            filter_resonance: 0.1,
            filter_env_amount: 0.2,
            filter_env_decay: 100.0,
            distortion_amount: 0.1,
            distortion_type: DistortionType::Soft,
            master_level: 0.8,
            velocity_sensitivity: 0.3,
        }
    }

    /// Hard, punchy techno kick preset
    pub fn preset_techno() -> Self {
        Self {
            osc1_pitch_start: 200.0,
            osc1_pitch_end: 60.0,
            osc1_pitch_decay: 80.0,
            osc1_level: 0.85,
            osc2_pitch_start: 4000.0,
            osc2_pitch_end: 250.0,
            osc2_pitch_decay: 12.0,
            osc2_level: 0.4,
            amp_attack: 0.2,
            amp_decay: 200.0,
            amp_sustain: 0.0,
            amp_release: 30.0,
            filter_cutoff: 10000.0,
            filter_resonance: 0.3,
            filter_env_amount: 0.5,
            filter_env_decay: 120.0,
            distortion_amount: 0.35,
            distortion_type: DistortionType::Hard,
            master_level: 0.85,
            velocity_sensitivity: 0.6,
        }
    }

    /// Deep, sub-bass kick preset
    pub fn preset_sub() -> Self {
        Self {
            osc1_pitch_start: 120.0,
            osc1_pitch_end: 40.0,
            osc1_pitch_decay: 200.0,
            osc1_level: 1.0,
            osc2_pitch_start: 1500.0,
            osc2_pitch_end: 100.0,
            osc2_pitch_decay: 25.0,
            osc2_level: 0.15,
            amp_attack: 1.0,
            amp_decay: 600.0,
            amp_sustain: 0.0,
            amp_release: 100.0,
            filter_cutoff: 5000.0,
            filter_resonance: 0.15,
            filter_env_amount: 0.1,
            filter_env_decay: 200.0,
            distortion_amount: 0.05,
            distortion_type: DistortionType::Tube,
            master_level: 0.9,
            velocity_sensitivity: 0.4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = KickParams::default();
        assert!(params.osc1_pitch_start > params.osc1_pitch_end);
        assert!(params.amp_attack < params.amp_decay);
        assert_eq!(params.amp_sustain, 0.0); // Kicks should have no sustain
    }

    #[test]
    fn test_presets() {
        let kick_808 = KickParams::preset_808();
        let kick_techno = KickParams::preset_techno();
        let kick_sub = KickParams::preset_sub();

        // All should have pitch sweeps (start > end)
        assert!(kick_808.osc1_pitch_start > kick_808.osc1_pitch_end);
        assert!(kick_techno.osc1_pitch_start > kick_techno.osc1_pitch_end);
        assert!(kick_sub.osc1_pitch_start > kick_sub.osc1_pitch_end);

        // Sub should be the lowest starting pitch
        assert!(kick_sub.osc1_pitch_start < kick_808.osc1_pitch_start);
        assert!(kick_sub.osc1_pitch_start < kick_techno.osc1_pitch_start);
    }
}
