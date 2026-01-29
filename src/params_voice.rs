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
/// **Total: 18 parameters**
/// - Input/Output (2): input_gain, output_gain
/// - Attack Enhancer (1): transient_attack
/// - De-Esser (3): de_esser_amount, de_esser_threshold, de_esser_listen_hf
/// - Bass (2): bass_drive, bass_mix
/// - Mids (2): mid_drive, mid_mix
/// - Presence (2): presence_drive, presence_mix
/// - Air (2): air_drive, air_mix
/// - Stereo (1): stereo_width
/// - Global (1): global_mix
/// - Limiter (2): limiter_threshold, limiter_release
use serde::{Deserialize, Serialize};

/// Professional vocal processing with zero-latency dynamics chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceParams {
    // === Input/Output (2 params) ===
    /// Input gain in dB (-12dB to +12dB)
    pub input_gain: f32,
    /// Output gain in dB (-12dB to +12dB)
    pub output_gain: f32,

    // === Attack Enhancer (1 param) ===
    /// Attack gain adjustment (-1.0 to +1.0, negative=soften, positive=punch)
    pub transient_attack: f32,

    // === De-Esser (3 params) ===
    /// De-esser amount (0.0-1.0). 0.0 is a bit-perfect bypass.
    pub de_esser_amount: f32,
    /// De-esser threshold (0.0-1.0). Higher = less sensitive.
    pub de_esser_threshold: f32,
    /// Debug: listen to the reduction delta (what is being removed).
    pub de_esser_listen_hf: bool,

    // === Bass Band (2 params) ===
    /// Bass drive amount (0.0-1.0)
    pub bass_drive: f32,
    /// Bass dry/wet mix (0.0-1.0)
    pub bass_mix: f32,

    // === Mids Band (2 params) ===
    /// Mids drive amount (0.0-1.0)
    pub mid_drive: f32,
    /// Mids dry/wet mix (0.0-1.0)
    pub mid_mix: f32,

    // === Presence Band (2 params) ===
    /// Presence drive amount (0.0-1.0)
    pub presence_drive: f32,
    /// Presence dry/wet mix (0.0-1.0)
    pub presence_mix: f32,

    // === Air Band (2 params) ===
    /// Air exciter drive amount (0.0-1.0)
    pub air_drive: f32,
    /// Air exciter dry/wet mix (0.0-1.0)
    pub air_mix: f32,

    // === Stereo (1 param) ===
    /// Stereo width control (-1.0 to +1.0)
    /// - -1.0: Maximum width (saturate sides more)
    /// - 0.0: Neutral (equal processing)
    /// - +1.0: Maximum power (saturate mid more)
    pub stereo_width: f32,

    // === Global Mix (1 param) ===
    /// Master wet/dry blend (0.0-1.0)
    /// - 0.0: 100% dry (bypass)
    /// - 1.0: 100% wet (full effect)
    pub global_mix: f32,

    // === Adaptive Compression Limiter (2 params) ===
    /// Limiter threshold in dB (-20.0 to 0.0)
    pub limiter_threshold: f32,
    /// Limiter release time in ms (50-500ms)
    pub limiter_release: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            // Input/Output - unity gain
            input_gain: 0.0,
            output_gain: 0.0,

            // Attack Enhancer - neutral (no effect)
            transient_attack: 0.0, // No attack boost/cut

            // De-Esser - off by default (amount==0), conservative threshold
            de_esser_amount: 0.0,
            de_esser_threshold: 0.6,
            de_esser_listen_hf: false,

            // Bass - warm foundation
            bass_drive: 0.6,
            bass_mix: 0.5,

            // Mids - balanced fundamentals
            mid_drive: 0.5,
            mid_mix: 0.4,

            // Presence - clarity without harshness
            presence_drive: 0.35,
            presence_mix: 0.35,

            // Air - subtle high-frequency enhancement
            air_drive: 0.1,
            air_mix: 0.15,

            // Stereo - neutral
            stereo_width: 0.0,

            // Global Mix - 100% wet (full effect)
            global_mix: 1.0,

            // Limiter - safety ceiling
            limiter_threshold: -6.0, // Start limiting at -6dB
            limiter_release: 200.0,  // 200ms release time
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
    /// Test preset: Gentle saturation
    pub fn test_gentle() -> Self {
        Self {
            input_gain: 0.0,
            output_gain: 0.0,
            transient_attack: 0.0,
            de_esser_amount: 0.0,
            de_esser_threshold: 0.6,
            de_esser_listen_hf: false,
            bass_drive: 0.3,
            bass_mix: 0.3,
            mid_drive: 0.25,
            mid_mix: 0.25,
            presence_drive: 0.2,
            presence_mix: 0.2,
            air_drive: 0.05,
            air_mix: 0.1,
            stereo_width: 0.0,
            global_mix: 1.0,
            limiter_threshold: -8.0,
            limiter_release: 250.0,
        }
    }

    /// Test preset: Moderate saturation (default)
    pub fn test_moderate() -> Self {
        Self::default()
    }

    /// Test preset: Aggressive saturation
    pub fn test_aggressive() -> Self {
        Self {
            input_gain: 3.0, // Hot input
            output_gain: 0.0,
            transient_attack: 0.5,
            de_esser_amount: 0.35,
            de_esser_threshold: 0.5,
            de_esser_listen_hf: false,
            bass_drive: 0.9,
            bass_mix: 0.7,
            mid_drive: 0.8,
            mid_mix: 0.6,
            presence_drive: 0.6,
            presence_mix: 0.5,
            air_drive: 0.2,
            air_mix: 0.3,
            stereo_width: 0.3,
            global_mix: 1.0,
            limiter_threshold: -3.0,
            limiter_release: 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = VoiceParams::default();
        assert_eq!(params.input_gain, 0.0);
        assert_eq!(params.output_gain, 0.0);
        assert_eq!(params.de_esser_amount, 0.0);
        assert_eq!(params.bass_drive, 0.6);
        assert_eq!(params.stereo_width, 0.0);
    }

    #[test]
    fn test_test_presets() {
        let gentle = VoiceParams::test_gentle();
        assert_eq!(gentle.bass_drive, 0.3);

        let moderate = VoiceParams::test_moderate();
        assert_eq!(moderate.bass_drive, 0.6);

        let aggressive = VoiceParams::test_aggressive();
        assert_eq!(aggressive.bass_drive, 0.9);
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

        // Verify parameters are within expected ranges
        assert!(params.input_gain >= -12.0 && params.input_gain <= 12.0);
        assert!(params.output_gain >= -12.0 && params.output_gain <= 12.0);
        assert!(params.de_esser_amount >= 0.0 && params.de_esser_amount <= 1.0);
        assert!(params.de_esser_threshold >= 0.0 && params.de_esser_threshold <= 1.0);
        assert!(params.bass_drive >= 0.0 && params.bass_drive <= 1.0);
        assert!(params.mid_mix >= 0.0 && params.mid_mix <= 1.0);
        assert!(params.stereo_width >= -1.0 && params.stereo_width <= 1.0);
    }
}
