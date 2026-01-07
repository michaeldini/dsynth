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
    #[serde(default)]
    pub distortion_enabled: bool, // Master enable for distortion (default false)

    // Master
    pub master_level: f32,         // Master output level (0.0-1.0)
    pub velocity_sensitivity: f32, // How much velocity affects amplitude (0.0-1.0)
    pub key_tracking: f32,         // Key tracking amount (0.0-1.0, 0=no tracking, 1=full chromatic)

    // Multiband Compression - Crossovers
    pub mb_xover_low: f32,  // Low crossover frequency (50-500Hz, default 150Hz)
    pub mb_xover_high: f32, // High crossover frequency (400-2000Hz, default 800Hz)

    // Multiband Compression - Sub Band (40-150Hz)
    pub mb_sub_threshold: f32, // Threshold in dB (-60 to 0, default -20dB)
    pub mb_sub_ratio: f32,     // Compression ratio (1.0-20.0, default 4.0)
    pub mb_sub_attack: f32,    // Attack time in ms (0.1-1000, default 5.0)
    pub mb_sub_release: f32,   // Release time in ms (1-5000, default 100.0)
    pub mb_sub_gain: f32,      // Post-compression gain (0.0-2.0, default 1.0)
    pub mb_sub_bypass: bool,   // Bypass sub band (default false)

    // Multiband Compression - Body Band (150-800Hz)
    pub mb_body_threshold: f32, // Threshold in dB (-60 to 0, default -15dB)
    pub mb_body_ratio: f32,     // Compression ratio (1.0-20.0, default 3.0)
    pub mb_body_attack: f32,    // Attack time in ms (0.1-1000, default 10.0)
    pub mb_body_release: f32,   // Release time in ms (1-5000, default 150.0)
    pub mb_body_gain: f32,      // Post-compression gain (0.0-2.0, default 1.0)
    pub mb_body_bypass: bool,   // Bypass body band (default false)

    // Multiband Compression - Click Band (800Hz+)
    pub mb_click_threshold: f32, // Threshold in dB (-60 to 0, default -10dB)
    pub mb_click_ratio: f32,     // Compression ratio (1.0-20.0, default 2.0)
    pub mb_click_attack: f32,    // Attack time in ms (0.1-1000, default 0.5)
    pub mb_click_release: f32,   // Release time in ms (1-5000, default 50.0)
    pub mb_click_gain: f32,      // Post-compression gain (0.0-2.0, default 1.0)
    pub mb_click_bypass: bool,   // Bypass click band (default false)

    // Multiband Compression - Global
    pub mb_mix: f32,      // Wet/dry mix (0.0-1.0, default 1.0)
    pub mb_enabled: bool, // Master enable (default true)

    // Exciter (high-frequency transient enhancement)
    pub exciter_frequency: f32, // High-pass cutoff (2000-12000Hz, default 4000Hz)
    pub exciter_drive: f32,     // Harmonic drive (0.0-1.0, default 0.3)
    pub exciter_mix: f32,       // Wet/dry mix (0.0-1.0, default 0.3)
    #[serde(default)]
    pub exciter_enabled: bool, // Master enable for exciter (default false)

    // Transient Shaper
    pub transient_attack_boost: f32, // Attack boost (0.0-1.0, default 0.3)
    pub transient_sustain_reduction: f32, // Sustain reduction (0.0-1.0, default 0.2)
    #[serde(default)]
    pub transient_enabled: bool, // Master enable for transient shaper (default false)

    // Clipper (brick-wall limiting)
    pub clipper_enabled: bool,  // Enable clipper (default false)
    pub clipper_threshold: f32, // Clipping threshold (0.7-1.0, default 0.95)
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

            // Distortion (OFF by default)
            distortion_amount: 0.0,
            distortion_type: DistortionType::Soft,
            distortion_enabled: false,

            // Master
            master_level: 0.8,
            velocity_sensitivity: 0.5,
            key_tracking: 0.0, // Default: no key tracking (preserve current behavior)

            // Multiband Compression (OFF by default)
            mb_xover_low: 150.0,
            mb_xover_high: 800.0,
            mb_sub_threshold: -20.0,
            mb_sub_ratio: 4.0,
            mb_sub_attack: 5.0,
            mb_sub_release: 100.0,
            mb_sub_gain: 1.0,
            mb_sub_bypass: false,
            mb_body_threshold: -15.0,
            mb_body_ratio: 3.0,
            mb_body_attack: 10.0,
            mb_body_release: 150.0,
            mb_body_gain: 1.0,
            mb_body_bypass: false,
            mb_click_threshold: -10.0,
            mb_click_ratio: 2.0,
            mb_click_attack: 0.5,
            mb_click_release: 50.0,
            mb_click_gain: 1.0,
            mb_click_bypass: false,
            mb_mix: 1.0,
            mb_enabled: false,

            // Exciter (OFF by default)
            exciter_frequency: 4000.0,
            exciter_drive: 0.0,
            exciter_mix: 0.0,
            exciter_enabled: false,

            // Transient Shaper (OFF by default)
            transient_attack_boost: 0.0,
            transient_sustain_reduction: 0.0,
            transient_enabled: false,

            // Clipper - Disabled by default
            clipper_enabled: false,
            clipper_threshold: 0.95,
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
            distortion_enabled: true,
            master_level: 0.8,
            velocity_sensitivity: 0.3,
            key_tracking: 0.0,
            mb_xover_low: 150.0,
            mb_xover_high: 800.0,
            mb_sub_threshold: -20.0,
            mb_sub_ratio: 4.0,
            mb_sub_attack: 5.0,
            mb_sub_release: 100.0,
            mb_sub_gain: 1.0,
            mb_sub_bypass: false,
            mb_body_threshold: -15.0,
            mb_body_ratio: 3.0,
            mb_body_attack: 10.0,
            mb_body_release: 150.0,
            mb_body_gain: 1.0,
            mb_body_bypass: false,
            mb_click_threshold: -10.0,
            mb_click_ratio: 2.0,
            mb_click_attack: 0.5,
            mb_click_release: 50.0,
            mb_click_gain: 1.0,
            mb_click_bypass: false,
            mb_mix: 1.0,
            mb_enabled: true,
            exciter_frequency: 4000.0,
            exciter_drive: 0.3,
            exciter_mix: 0.3,
            exciter_enabled: true,
            transient_attack_boost: 0.3,
            transient_sustain_reduction: 0.2,
            transient_enabled: true,
            clipper_enabled: false,
            clipper_threshold: 0.95,
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
            distortion_enabled: true,
            master_level: 0.85,
            velocity_sensitivity: 0.6,
            key_tracking: 0.0,
            mb_xover_low: 150.0,
            mb_xover_high: 800.0,
            mb_sub_threshold: -18.0,
            mb_sub_ratio: 5.0,
            mb_sub_attack: 3.0,
            mb_sub_release: 80.0,
            mb_sub_gain: 1.1,
            mb_sub_bypass: false,
            mb_body_threshold: -12.0,
            mb_body_ratio: 4.0,
            mb_body_attack: 5.0,
            mb_body_release: 100.0,
            mb_body_gain: 1.0,
            mb_body_bypass: false,
            mb_click_threshold: -8.0,
            mb_click_ratio: 2.5,
            mb_click_attack: 0.3,
            mb_click_release: 40.0,
            mb_click_gain: 1.2,
            mb_click_bypass: false,
            mb_mix: 1.0,
            mb_enabled: true,
            exciter_frequency: 5000.0,
            exciter_drive: 0.5,
            exciter_mix: 0.4,
            exciter_enabled: true,
            transient_attack_boost: 0.5,
            transient_sustain_reduction: 0.4,
            transient_enabled: true,
            clipper_enabled: true,
            clipper_threshold: 0.85,
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
            distortion_enabled: true,
            master_level: 0.9,
            velocity_sensitivity: 0.4,
            key_tracking: 0.0,
            mb_xover_low: 100.0,
            mb_xover_high: 600.0,
            mb_sub_threshold: -24.0,
            mb_sub_ratio: 6.0,
            mb_sub_attack: 10.0,
            mb_sub_release: 150.0,
            mb_sub_gain: 1.3,
            mb_sub_bypass: false,
            mb_body_threshold: -18.0,
            mb_body_ratio: 2.5,
            mb_body_attack: 15.0,
            mb_body_release: 200.0,
            mb_body_gain: 0.9,
            mb_body_bypass: false,
            mb_click_threshold: -12.0,
            mb_click_ratio: 1.5,
            mb_click_attack: 1.0,
            mb_click_release: 60.0,
            mb_click_gain: 0.7,
            mb_click_bypass: false,
            mb_mix: 1.0,
            mb_enabled: true,
            exciter_frequency: 3000.0,
            exciter_drive: 0.2,
            exciter_mix: 0.2,
            exciter_enabled: true,
            transient_attack_boost: 0.2,
            transient_sustain_reduction: 0.1,
            transient_enabled: true,
            clipper_enabled: false,
            clipper_threshold: 0.95,
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

        // Effects should be OFF by default so the user knows what's shaping the sound.
        assert!(!params.distortion_enabled);
        assert!(!params.mb_enabled);
        assert!(!params.exciter_enabled);
        assert!(!params.transient_enabled);
        assert!(!params.clipper_enabled);
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
