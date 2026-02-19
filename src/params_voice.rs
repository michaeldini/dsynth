/// Voice Saturation Parameters - PROFESSIONAL VOCAL PROCESSING CHAIN
///
/// **Design: Zero-latency vocal processor with intelligent dynamics and saturation**
///
/// Processing chain:
/// 1. Input Gain
/// 2. **Signal Analysis** (transient, ZCR, sibilance - NO PITCH for zero latency)
/// 3. **Transient Shaper** (attack control based on analysis)
/// 4. **Split-Band De-Esser** (zero-latency, pre-saturation)
/// 5. **4-Band Multiband Saturator** (bass/mids/presence/air with mid-side processing)
/// 5. **Adaptive Compression Limiter** (transient-aware envelope-follower limiting)
/// 6. Global Mix (parallel processing)
/// 7. Output Gain
///
/// **Total: 19 parameters**
/// - Input/Output (2): input_gain, output_gain
/// - Attack Enhancer (1): transient_attack
/// - De-Esser (4): de_esser_amount, de_esser_threshold, sibilance_frequency, de_esser_listen_hf
/// - Bass (2): bass_drive, bass_mix
/// - Mids (2): mid_drive, mid_mix
/// - Presence (2): presence_drive, presence_mix
/// - Air (2): air_drive, air_mix
/// - Global (1): global_mix
/// - Vocal Character (1): vocal_character
/// - Limiter (2): limiter_threshold, limiter_release
use serde::{Deserialize, Serialize};

/// Professional vocal processing with perceptual controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceParams {
    // === PERCEPTUAL CONTROLS (4 core parameters) ===
    /// Character: -1.0 = warm/vintage, +1.0 = bright/modern
    pub character: f32,

    /// Intensity: 0.0 = transparent/gentle, 1.0 = saturated/aggressive
    pub intensity: f32,

    /// Presence: -1.0 = distant/laid-back, +1.0 = intimate/upfront
    pub presence: f32,

    /// Dynamics: -1.0 = controlled/compressed, +1.0 = punchy/dynamic
    pub dynamics: f32,

    // === I/O CONTROLS ===
    /// Input gain in dB (-12dB to +12dB)
    pub input_gain: f32,
    /// Output gain in dB (-12dB to +12dB)
    pub output_gain: f32,
    /// Dry/wet mix (0.0-1.0)
    pub dry_wet_mix: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            // Professional out-of-box settings for modern pop vocals
            character: 0.2, // Slightly bright
            intensity: 0.4, // Moderate processing
            presence: 0.3,  // Upfront but not harsh
            dynamics: 0.1,  // Controlled but not squashed

            // I/O defaults
            input_gain: 0.0,
            output_gain: 0.0,
            dry_wet_mix: 1.0, // Full effect
        }
    }
}

impl VoiceParams {
    /// Create a new VoiceParams with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert dB to linear gain
    pub fn db_to_gain(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Convert linear gain to dB
    pub fn gain_to_db(gain: f32) -> f32 {
        20.0 * gain.log10()
    }
}

// Test-only presets (for validation)
#[cfg(test)]
impl VoiceParams {
    /// Test preset: Warm and gentle
    pub fn test_warm() -> Self {
        Self {
            character: -0.7, // Very warm
            intensity: 0.2,  // Gentle processing
            presence: -0.3,  // Laid back
            dynamics: 0.3,   // Some punch
            input_gain: 0.0,
            output_gain: 0.0,
            dry_wet_mix: 1.0,
        }
    }

    /// Test preset: Bright and aggressive
    pub fn test_bright() -> Self {
        Self {
            character: 0.8, // Very bright
            intensity: 0.8, // Aggressive processing
            presence: 0.6,  // Very upfront
            dynamics: -0.2, // More controlled
            input_gain: 0.0,
            output_gain: 0.0,
            dry_wet_mix: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = VoiceParams::default();
        assert_eq!(params.character, 0.2);
        assert_eq!(params.intensity, 0.4);
        assert_eq!(params.presence, 0.3);
        assert_eq!(params.dynamics, 0.1);
        assert_eq!(params.input_gain, 0.0);
        assert_eq!(params.output_gain, 0.0);
        assert_eq!(params.dry_wet_mix, 1.0);
    }

    #[test]
    fn test_test_presets() {
        let warm = VoiceParams::test_warm();
        assert_eq!(warm.character, -0.7);
        assert_eq!(warm.intensity, 0.2);

        let bright = VoiceParams::test_bright();
        assert_eq!(bright.character, 0.8);
        assert_eq!(bright.intensity, 0.8);
    }

    #[test]
    fn test_db_to_gain() {
        assert!((VoiceParams::db_to_gain(0.0) - 1.0).abs() < 0.001);
        assert!((VoiceParams::db_to_gain(6.0) - 2.0).abs() < 0.01);
        assert!((VoiceParams::db_to_gain(-6.0) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_gain_to_db() {
        assert!((VoiceParams::gain_to_db(1.0) - 0.0).abs() < 0.001);
        assert!((VoiceParams::gain_to_db(2.0) - 6.0).abs() < 0.1);
        assert!((VoiceParams::gain_to_db(0.5) - (-6.0)).abs() < 0.1);
    }

    #[test]
    fn test_parameter_ranges() {
        let params = VoiceParams::default();

        // Verify perceptual parameters are within expected ranges
        assert!(params.character >= -1.0 && params.character <= 1.0);
        assert!(params.intensity >= 0.0 && params.intensity <= 1.0);
        assert!(params.presence >= -1.0 && params.presence <= 1.0);
        assert!(params.dynamics >= -1.0 && params.dynamics <= 1.0);
        assert!(params.input_gain >= -12.0 && params.input_gain <= 12.0);
        assert!(params.output_gain >= -12.0 && params.output_gain <= 12.0);
        assert!(params.dry_wet_mix >= 0.0 && params.dry_wet_mix <= 1.0);
    }
}
