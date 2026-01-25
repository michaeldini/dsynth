/// Voice Saturation Parameters - MINIMAL ANALOG EMULATION
///
/// **New Design: Minimal analog saturation plugin for vocals**
///
/// Simple processing chain with adaptive multi-stage saturation:
/// 1. Input Gain
/// 2. **Signal Analysis** (transient, ZCR, sibilance - NO PITCH for zero latency)
/// 3. **Adaptive Saturator** (3-stage cascaded saturation with character selection + parallel processing)
///    - Drive: Single knob controls saturation amount (0-100%)
///    - Character: Warm/Smooth/Punchy (musical descriptors)
///    - Mix: Dry/wet blend for transparent enhancement (0-100%)
///    - Auto-gain compensation maintains perceived loudness
///    - Transient-adaptive: More saturation on attacks
/// 4. Output Gain
///
/// **Total: 5 parameters** (input_gain, saturation_character, saturation_drive, saturation_mix, output_gain)
use serde::{Deserialize, Serialize};

/// Simplified parameter set for analog saturation with parallel processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceParams {
    // === Input/Output (2 params) ===
    /// Input gain in dB (-12dB to +12dB)
    pub input_gain: f32,
    /// Output gain in dB (-12dB to +12dB)
    pub output_gain: f32,

    // === Saturation (3 params) ===
    /// Saturation character selection (0=Warm, 1=Smooth, 2=Punchy)
    /// - Warm: Tube-style asymmetric saturation (even harmonics, gentle)
    /// - Smooth: Tape-style soft-knee saturation (balanced harmonics)
    /// - Punchy: Console-style saturation (aggressive mids, transient emphasis)
    pub saturation_character: u8,

    /// Saturation drive amount (0.0-1.0)
    /// - 0.0: No saturation (clean passthrough)
    /// - 0.5: Moderate saturation (suitable for most vocals)
    /// - 1.0: Aggressive saturation (maximum warmth/color)
    /// Internally scaled as drive^3.5 for gentle transparent control
    pub saturation_drive: f32,

    /// Dry/wet mix (0.0-1.0)
    /// - 0.0: 100% dry (bypass, but analysis still active)
    /// - 0.3-0.5: Optimal for transparent vocal enhancement (parallel saturation)
    /// - 1.0: 100% wet (full saturation, no dry signal)
    /// Parallel processing preserves transient clarity while adding harmonic richness
    pub saturation_mix: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            // Input/Output - unity gain
            input_gain: 0.0,
            output_gain: 0.0,

            // Saturation - moderate settings
            saturation_character: 0, // Warm (tube-style)
            saturation_drive: 0.5,   // 50% drive = moderate saturation
            saturation_mix: 0.4,     // 40% wet = balanced parallel saturation
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
    /// Test preset: Gentle saturation (30% drive)
    pub fn test_gentle() -> Self {
        Self {
            input_gain: 0.0,
            output_gain: 0.0,
            saturation_character: 0, // Warm
            saturation_drive: 0.3,   // Gentle
            saturation_mix: 0.3,     // 30% wet = subtle enhancement
        }
    }

    /// Test preset: Moderate saturation (50% drive) - target calibration
    pub fn test_moderate() -> Self {
        Self {
            input_gain: 0.0,
            output_gain: 0.0,
            saturation_character: 1, // Smooth
            saturation_drive: 0.5,   // Moderate (target)
            saturation_mix: 0.5,     // 50% wet = balanced blend
        }
    }

    /// Test preset: Aggressive saturation (80% drive)
    pub fn test_aggressive() -> Self {
        Self {
            input_gain: 3.0, // Hot input
            output_gain: 0.0,
            saturation_character: 2, // Punchy
            saturation_drive: 0.8,   // Aggressive
            saturation_mix: 1.0,     // 100% wet = full saturation
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
        assert_eq!(params.saturation_character, 0);
        assert_eq!(params.saturation_drive, 0.5);
    }

    #[test]
    fn test_test_presets() {
        let gentle = VoiceParams::test_gentle();
        assert_eq!(gentle.saturation_drive, 0.3);

        let moderate = VoiceParams::test_moderate();
        assert_eq!(moderate.saturation_drive, 0.5);
        assert_eq!(moderate.saturation_character, 1);

        let aggressive = VoiceParams::test_aggressive();
        assert_eq!(aggressive.saturation_drive, 0.8);
        assert_eq!(aggressive.saturation_character, 2);
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

        // Verify simplified parameters are within expected ranges
        assert!(params.input_gain >= -12.0 && params.input_gain <= 12.0);
        assert!(params.output_gain >= -12.0 && params.output_gain <= 12.0);
        assert!(params.saturation_character <= 2);
        assert!(params.saturation_drive >= 0.0 && params.saturation_drive <= 1.0);
    }
}
