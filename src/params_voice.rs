/// Voice Enhancer Parameters - INTELLIGENT ARCHITECTURE v2.0
///
/// **New Design: "Analyze Once, Use Everywhere"**
///
/// Simplified processing chain with intelligent signal-adaptive effects:
/// 1. Input Gain
/// 2. **Signal Analysis** (runs all detectors once: transient, ZCR, sibilance, pitch)
/// 3. **Smart Gate** (auto-adapts to transients/sibilance) - 1 parameter
/// 4. **Adaptive Compressor** (pitch-responsive, transient-aware) - 4 parameters
/// 5. **Intelligent Exciter** (pitch-tracked harmonics, bypasses sibilance) - 2 parameters
/// 6. Lookahead Limiter (safety ceiling)
/// 7. Output Gain & Dry/Wet Mix
///
/// **Total: ~12 parameters** (down from 70+)
use serde::{Deserialize, Serialize};

/// Simplified parameter set for intelligent voice enhancement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceParams {
    // === Input/Output (2 params) ===
    pub input_gain: f32,  // -12dB to +12dB
    pub output_gain: f32, // -12dB to +12dB

    // === Signal Analysis (1 param) ===
    /// Minimum pitch confidence (0.0-1.0) for pitch tracking
    /// Used by analyzer and all pitch-dependent effects
    pub pitch_confidence_threshold: f32,

    // === Smart Gate (2 params) ===
    /// Enable/disable smart gate
    pub gate_enable: bool,
    /// Threshold in dB (-80 to -20)
    /// Gate automatically adapts based on signal type:
    /// - Transients: -18dB offset (more sensitive)
    /// - Sibilance: -12dB offset
    /// - Pitched vocals: -6dB offset
    /// - Voiced content: -3dB offset
    /// - Unvoiced: uses this threshold as-is
    pub gate_threshold: f32,

    // === Adaptive Compressor (5 params) ===
    /// Enable/disable adaptive compressor
    pub comp_enable: bool,
    /// Threshold in dB (-40 to 0)
    /// Automatically adjusted based on detected pitch
    pub comp_threshold: f32,
    /// Compression ratio (1.0 to 20.0)
    /// Automatically reduced by 40% during transients
    pub comp_ratio: f32,
    /// Attack time in ms (0.1 to 100)
    /// Overridden to 2ms fast attack during transients
    pub comp_attack: f32,
    /// Release time in ms (10 to 1000)
    pub comp_release: f32,

    // === Intelligent Exciter (3 params) ===
    /// Enable/disable intelligent exciter
    pub exciter_enable: bool,
    /// Harmonic generation amount (0.0 to 1.0)
    /// Exciter automatically:
    /// - Tracks detected pitch
    /// - Generates harmonics at 2×, 3×, 4× frequency
    /// - Bypasses sibilance content
    /// - Only processes voiced content
    pub exciter_amount: f32,
    /// Wet/dry mix (0.0 to 1.0)
    pub exciter_mix: f32,

    // === De-Esser (2 params) ===
    /// Enable/disable de-esser
    pub deess_enable: bool,
    /// Sibilance reduction amount in dB (0.0 to 12.0)
    /// Automatically reduces harsh "s", "t", "ch" sounds
    pub deess_amount: f32,

    // === Smart Delay (4 params) ===
    /// Enable/disable smart delay
    pub delay_enable: bool,
    /// Delay time in milliseconds (50.0 to 500.0)
    /// Creates space and depth in sustained vocal content
    pub delay_time: f32,
    /// Feedback amount (0.0 to 0.8)
    /// Controls how many times the delay repeats
    pub delay_feedback: f32,
    /// Wet/dry mix (0.0 to 1.0)
    /// Smart delay automatically:
    /// - Reduces mix to 0% during transients (preserves attack)
    /// - Applies full mix to sustained content (adds space)
    pub delay_mix: f32,
    /// Transient sensitivity (0.0 to 1.0)
    /// - 0.0 = ignore transients, always apply delay
    /// - 0.5 = moderate sensitivity (default)
    /// - 1.0 = very sensitive, reduce delay even on weak transients
    pub delay_sensitivity: f32,

    // === Master (1 param) ===
    /// Overall dry/wet mix (0.0 = bypass, 1.0 = fully processed)
    pub dry_wet: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            // Input/Output - unity gain
            input_gain: 0.0,
            output_gain: 0.0,

            // Signal Analysis
            pitch_confidence_threshold: 0.6, // 60% confidence threshold

            // Smart Gate - moderate threshold
            gate_enable: true,
            gate_threshold: -50.0, // -50dB threshold (auto-adapts based on content)

            // Adaptive Compressor - optimized for vocals
            comp_enable: true,
            comp_threshold: -18.0, // Catch peaks without over-compressing
            comp_ratio: 3.5,       // Moderate compression (auto-gentler on transients)
            comp_attack: 8.0,      // Fast enough for transients (overridden to 2ms on transients)
            comp_release: 150.0,   // Musical release

            // Intelligent Exciter - subtle enhancement
            exciter_enable: true,
            exciter_amount: 0.3, // 30% harmonic generation (auto-bypasses sibilance)
            exciter_mix: 0.3,    // 30% wet (auto-tracks pitch for harmonics)

            // De-Esser - moderate sibilance control
            deess_enable: true,
            deess_amount: 6.0, // 6dB reduction on sibilance

            // Smart Delay - subtle depth
            delay_enable: false,    // Off by default
            delay_time: 120.0,      // 120ms delay (musical eighth note @ 125 BPM)
            delay_feedback: 0.3,    // 30% feedback (2-3 repeats)
            delay_mix: 0.4,         // 40% wet (auto-adapts to bypass transients)
            delay_sensitivity: 0.5, // Moderate sensitivity

            // Master - 100% wet (fully processed)
            dry_wet: 1.0,
        }
    }
}

impl VoiceParams {
    /// Create a new VoiceParams with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Preset: "Clean Vocal" - transparent processing with light enhancement
    pub fn preset_clean_vocal() -> Self {
        Self {
            input_gain: 0.0,
            output_gain: 0.0,

            pitch_confidence_threshold: 0.7, // Stricter pitch detection

            // Gentle gate
            gate_enable: true,
            gate_threshold: -55.0, // More sensitive (will auto-adapt)

            // Light compression
            comp_enable: true,
            comp_threshold: -20.0, // Less aggressive
            comp_ratio: 2.5,       // Gentler ratio
            comp_attack: 10.0,     // Slightly slower
            comp_release: 120.0,   // Quicker release

            // Very light exciter
            exciter_enable: true,
            exciter_amount: 0.2, // Subtle
            exciter_mix: 0.25,   // Light blend

            // Light de-essing
            deess_enable: true,
            deess_amount: 4.0, // Gentle reduction

            // No delay (keep natural)
            delay_enable: false,
            delay_time: 100.0,
            delay_feedback: 0.2,
            delay_mix: 0.3,
            delay_sensitivity: 0.5, // Moderate (doesn't matter when disabled)

            dry_wet: 1.0,
        }
    }

    /// Preset: "Radio Voice" - aggressive processing for broadcast sound
    pub fn preset_radio_voice() -> Self {
        Self {
            input_gain: 3.0, // Hot input level
            output_gain: 0.0,

            pitch_confidence_threshold: 0.5, // More permissive

            // Aggressive gate
            gate_enable: true,
            gate_threshold: -45.0, // Tighter gate

            // Heavy compression
            comp_enable: true,
            comp_threshold: -16.0, // Lower threshold
            comp_ratio: 6.0,       // Aggressive ratio
            comp_attack: 5.0,      // Fast attack
            comp_release: 80.0,    // Quick release (pumpy)

            // Strong exciter
            exciter_enable: true,
            exciter_amount: 0.6, // Strong presence
            exciter_mix: 0.5,    // Prominent blend

            // Aggressive de-essing
            deess_enable: true,
            deess_amount: 8.0, // Strong reduction

            // Moderate delay for broadcast depth
            delay_enable: true,
            delay_time: 150.0,      // 150ms delay
            delay_feedback: 0.25,   // 25% feedback (subtle repeats)
            delay_mix: 0.35,        // 35% wet (auto-bypasses transients)
            delay_sensitivity: 0.6, // More sensitive - preserve clarity

            dry_wet: 1.0,
        }
    }

    /// Preset: "Deep Bass Enhancement" - emphasizes low-frequency content
    pub fn preset_deep_bass() -> Self {
        Self {
            input_gain: 0.0,
            output_gain: 0.0,

            pitch_confidence_threshold: 0.5, // Track low pitches

            // Moderate gate
            gate_enable: true,
            gate_threshold: -50.0,

            // Gentle compression (preserve dynamics)
            comp_enable: true,
            comp_threshold: -20.0,
            comp_ratio: 2.0,   // Light compression
            comp_attack: 15.0, // Slow attack (preserve transients)
            comp_release: 200.0,

            // Moderate exciter (let pitch-tracked harmonics do the work)
            exciter_enable: true,
            exciter_amount: 0.4, // Moderate harmonics at 2×, 3×, 4× pitch
            exciter_mix: 0.4,

            // Moderate de-essing
            deess_enable: true,
            deess_amount: 6.0,

            // Strong delay for spacious sound
            delay_enable: true,
            delay_time: 180.0,      // 180ms delay (longer for depth)
            delay_feedback: 0.4,    // 40% feedback (more repeats)
            delay_mix: 0.5,         // 50% wet (auto-bypasses transients)
            delay_sensitivity: 0.3, // Less sensitive = more delay on vocals

            dry_wet: 1.0,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = VoiceParams::default();
        assert_eq!(params.input_gain, 0.0);
        assert_eq!(params.output_gain, 0.0);
        assert_eq!(params.gate_threshold, -50.0);
        assert_eq!(params.dry_wet, 1.0);
    }

    #[test]
    fn test_preset_clean_vocal() {
        let params = VoiceParams::preset_clean_vocal();
        assert!(params.gate_threshold < 0.0);
        assert!(params.comp_ratio >= 1.0);
    }

    #[test]
    fn test_preset_radio_voice() {
        let params = VoiceParams::preset_radio_voice();
        assert!(params.comp_ratio > 5.0); // Heavy compression
    }

    #[test]
    fn test_preset_deep_bass() {
        let params = VoiceParams::preset_deep_bass();
        // Just verify it creates without error
        assert!(params.comp_ratio >= 1.0);
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
        assert!(params.gate_threshold >= -80.0 && params.gate_threshold <= -20.0);
        assert!(params.comp_ratio >= 1.0 && params.comp_ratio <= 20.0);
        assert!(params.dry_wet >= 0.0 && params.dry_wet <= 1.0);
    }
}
