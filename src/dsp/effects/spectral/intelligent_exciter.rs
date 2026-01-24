//! Intelligent Exciter - Pitch-tracked harmonic enhancement using signal analysis
//!
//! Unlike traditional exciters that add generic high-frequency harmonics, this exciter:
//!
//! - **Pitch-tracked**: Generates harmonics at exact multiples of detected pitch (2×, 3×, 4×)
//! - **Content-aware**: Only processes voiced content (singing, speech)
//! - **Bypasses sibilance**: 's', 'sh', 't', 'f' sounds pass through unmodified
//! - **Adaptive**: Automatically adjusts to signal characteristics
//!
//! # How It Works
//! 1. Receives `SignalAnalysis` with detected pitch and signal type
//! 2. If voiced content with confident pitch → generates harmonics
//! 3. If sibilance or unvoiced → bypasses (already has high-frequency content)
//! 4. Blends harmonics with dry signal based on mix parameter
//!
//! # Parameters
//! Just TWO parameters (all intelligence is automatic):
//! - `amount` (0.0-1.0): How much harmonic content to generate
//! - `mix` (0.0-1.0): Wet/dry balance
//!
//! # Harmonic Generation
//! ```text
//! Detected pitch: 220Hz (A3)
//! Generated harmonics:
//!   2× = 440Hz (octave above)
//!   3× = 660Hz (perfect fifth above octave)
//!   4× = 880Hz (two octaves above)
//! 
//! Result: Rich, musical harmonics that enhance presence without harshness
//! ```
//!
//! # Signal Flow
//! ```text
//! Input → Analyze
//!           ↓
//!      Is Voiced? → NO → Bypass (return dry)
//!           ↓ YES
//!      Has Sibilance? → YES → Bypass (already bright)
//!           ↓ NO
//!      Has Pitch? → NO → Bypass (no fundamental to track)
//!           ↓ YES
//!      Generate Harmonics (2×, 3×, 4× pitch)
//!           ↓
//!      Blend with dry signal (mix parameter)
//!           ↓
//!      Output
//! ```

use crate::dsp::signal_analyzer::SignalAnalysis;
use std::f32::consts::PI;

/// Intelligent exciter with pitch-tracked harmonic generation
pub struct IntelligentExciter {
    sample_rate: f32,

    /// Parameters
    amount: f32, // 0.0-1.0: harmonic generation amount
    mix: f32,    // 0.0-1.0: wet/dry balance

    /// Oscillator phases for harmonic generation
    phase_2x: f32, // 2× fundamental (octave)
    phase_3x: f32, // 3× fundamental (perfect fifth above octave)
    phase_4x: f32, // 4× fundamental (two octaves)

    /// Current pitch being tracked
    current_pitch_hz: f32,

    /// Phase increments (calculated from pitch)
    phase_inc_2x: f32,
    phase_inc_3x: f32,
    phase_inc_4x: f32,
}

impl IntelligentExciter {
    /// Create a new intelligent exciter
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            amount: 0.3,           // Default 30% harmonic content
            mix: 0.5,              // Default 50% wet/dry
            phase_2x: 0.0,
            phase_3x: 0.0,
            phase_4x: 0.0,
            current_pitch_hz: 0.0,
            phase_inc_2x: 0.0,
            phase_inc_3x: 0.0,
            phase_inc_4x: 0.0,
        }
    }

    /// Set harmonic amount (0.0-1.0)
    ///
    /// Controls how much harmonic content is generated.
    /// - 0.0 = no harmonics
    /// - 0.5 = moderate enhancement
    /// - 1.0 = maximum harmonics
    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
    }

    /// Set wet/dry mix (0.0-1.0)
    ///
    /// Controls blend between dry signal and harmonically enhanced signal.
    /// - 0.0 = bypass (dry only)
    /// - 0.5 = equal mix
    /// - 1.0 = wet only (maximum enhancement)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Update phase increments when pitch changes
    fn update_pitch(&mut self, pitch_hz: f32) {
        if (self.current_pitch_hz - pitch_hz).abs() < 1.0 {
            // Pitch hasn't changed significantly - keep phases continuous
            return;
        }

        self.current_pitch_hz = pitch_hz;

        // Calculate phase increments for each harmonic
        // phase_inc = frequency / sample_rate
        self.phase_inc_2x = (pitch_hz * 2.0) / self.sample_rate;
        self.phase_inc_3x = (pitch_hz * 3.0) / self.sample_rate;
        self.phase_inc_4x = (pitch_hz * 4.0) / self.sample_rate;
    }

    /// Generate harmonic content at 2×, 3×, 4× fundamental
    #[inline]
    fn generate_harmonics(&mut self) -> f32 {
        // Generate sine waves at harmonic frequencies
        let harmonic_2x = (self.phase_2x * 2.0 * PI).sin();
        let harmonic_3x = (self.phase_3x * 2.0 * PI).sin();
        let harmonic_4x = (self.phase_4x * 2.0 * PI).sin();

        // Advance phases
        self.phase_2x += self.phase_inc_2x;
        self.phase_3x += self.phase_inc_3x;
        self.phase_4x += self.phase_inc_4x;

        // Wrap phases (keep in 0.0-1.0 range)
        if self.phase_2x >= 1.0 {
            self.phase_2x -= 1.0;
        }
        if self.phase_3x >= 1.0 {
            self.phase_3x -= 1.0;
        }
        if self.phase_4x >= 1.0 {
            self.phase_4x -= 1.0;
        }

        // Mix harmonics with decreasing amplitude (natural harmonic rolloff)
        // 2× = full, 3× = 70%, 4× = 50%
        (harmonic_2x + harmonic_3x * 0.7 + harmonic_4x * 0.5) / 2.2 // Normalize
    }

    /// Process one stereo sample pair with signal analysis
    ///
    /// # Arguments
    /// * `left` - Left channel input sample
    /// * `right` - Right channel input sample
    /// * `analysis` - Pre-computed signal analysis
    ///
    /// # Returns
    /// Tuple of (left, right) output samples
    pub fn process(&mut self, left: f32, right: f32, analysis: &SignalAnalysis) -> (f32, f32) {
        // === STEP 1: Decide Whether to Process ===

        // Bypass if sibilance detected (already has high-frequency content)
        if analysis.has_sibilance {
            return (left, right);
        }

        // Bypass if not voiced content (noise, unvoiced consonants)
        if !analysis.is_voiced {
            return (left, right);
        }

        // Bypass if no confident pitch (can't track harmonics without fundamental)
        if !analysis.is_pitched || analysis.pitch_confidence < 0.5 {
            return (left, right);
        }

        // === STEP 2: Update Pitch Tracking ===
        self.update_pitch(analysis.pitch_hz);

        // === STEP 3: Generate Harmonics ===
        let harmonics = self.generate_harmonics();

        // Scale by amount parameter and input level
        let input_level = (left.abs() + right.abs()) * 0.5;
        let scaled_harmonics = harmonics * self.amount * input_level;

        // === STEP 4: Apply Stereo Enhancement ===
        // Add harmonics to both channels but with slight phase offset for width
        let left_harmonics = scaled_harmonics;
        let right_harmonics = scaled_harmonics * 0.9; // Slightly reduced on right

        // === STEP 5: Mix Wet/Dry ===
        let wet_left = left + left_harmonics;
        let wet_right = right + right_harmonics;

        let output_left = left * (1.0 - self.mix) + wet_left * self.mix;
        let output_right = right * (1.0 - self.mix) + wet_right * self.mix;

        (output_left, output_right)
    }

    /// Reset exciter state
    pub fn reset(&mut self) {
        self.phase_2x = 0.0;
        self.phase_3x = 0.0;
        self.phase_4x = 0.0;
        self.current_pitch_hz = 0.0;
        self.phase_inc_2x = 0.0;
        self.phase_inc_3x = 0.0;
        self.phase_inc_4x = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::SignalType;

    fn create_voiced_analysis(pitch_hz: f32) -> SignalAnalysis {
        SignalAnalysis {
            rms_level: 0.1,
            peak_level: 0.1,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 400.0,
            signal_type: SignalType::Tonal,
            is_voiced: true,
            is_unvoiced: false,
            has_sibilance: false,
            sibilance_strength: 0.0,
            pitch_hz,
            pitch_confidence: 0.8,
            is_pitched: true,
        }
    }

    #[test]
    fn test_intelligent_exciter_creation() {
        let exciter = IntelligentExciter::new(44100.0);
        assert_eq!(exciter.sample_rate, 44100.0);
        assert_eq!(exciter.amount, 0.3);
        assert_eq!(exciter.mix, 0.5);
    }

    #[test]
    fn test_bypasses_sibilance() {
        let mut exciter = IntelligentExciter::new(44100.0);
        exciter.set_amount(1.0);
        exciter.set_mix(1.0);

        let mut analysis = create_voiced_analysis(220.0);
        analysis.has_sibilance = true;
        analysis.sibilance_strength = 0.8;

        let input = 0.5;
        let (out_l, out_r) = exciter.process(input, input, &analysis);

        // Should bypass completely (sibilance already has high frequencies)
        assert_eq!(out_l, input);
        assert_eq!(out_r, input);
    }

    #[test]
    fn test_bypasses_unvoiced() {
        let mut exciter = IntelligentExciter::new(44100.0);
        exciter.set_amount(1.0);
        exciter.set_mix(1.0);

        let mut analysis = create_voiced_analysis(220.0);
        analysis.is_voiced = false;
        analysis.is_unvoiced = true;

        let input = 0.5;
        let (out_l, out_r) = exciter.process(input, input, &analysis);

        // Should bypass unvoiced content
        assert_eq!(out_l, input);
        assert_eq!(out_r, input);
    }

    #[test]
    fn test_bypasses_unpitched() {
        let mut exciter = IntelligentExciter::new(44100.0);
        exciter.set_amount(1.0);
        exciter.set_mix(1.0);

        let mut analysis = create_voiced_analysis(220.0);
        analysis.is_pitched = false;
        analysis.pitch_confidence = 0.2;

        let input = 0.5;
        let (out_l, out_r) = exciter.process(input, input, &analysis);

        // Should bypass when pitch confidence is low
        assert_eq!(out_l, input);
        assert_eq!(out_r, input);
    }

    #[test]
    fn test_generates_harmonics_for_voiced() {
        let mut exciter = IntelligentExciter::new(44100.0);
        exciter.set_amount(0.5);
        exciter.set_mix(1.0); // 100% wet

        let analysis = create_voiced_analysis(220.0); // A3

        let input = 0.5;

        // Process several samples to establish harmonic generation
        for _ in 0..100 {
            exciter.process(input, input, &analysis);
        }

        let (out_l, out_r) = exciter.process(input, input, &analysis);

        // Output should differ from input (harmonics added)
        assert_ne!(out_l, input);
        assert_ne!(out_r, input);

        // Output should be in reasonable range
        assert!(out_l.abs() < 2.0);
        assert!(out_r.abs() < 2.0);
    }

    #[test]
    fn test_mix_parameter() {
        let mut exciter = IntelligentExciter::new(44100.0);
        exciter.set_amount(0.5);

        let analysis = create_voiced_analysis(220.0);
        let input = 0.5;

        // 0% mix = bypass
        exciter.set_mix(0.0);
        let (out_dry, _) = exciter.process(input, input, &analysis);
        assert_eq!(out_dry, input);

        // Reset phase to compare fairly
        exciter.reset();

        // 100% mix = full effect
        exciter.set_mix(1.0);
        for _ in 0..100 {
            exciter.process(input, input, &analysis);
        }
        let (out_wet, _) = exciter.process(input, input, &analysis);
        assert_ne!(out_wet, input); // Should be different with harmonics
    }

    #[test]
    fn test_amount_parameter() {
        let mut exciter = IntelligentExciter::new(44100.0);
        exciter.set_mix(1.0);

        let analysis = create_voiced_analysis(220.0);
        let input = 0.5;

        // Low amount = subtle effect
        exciter.set_amount(0.1);
        for _ in 0..100 {
            exciter.process(input, input, &analysis);
        }
        let (out_low, _) = exciter.process(input, input, &analysis);

        // Reset
        exciter.reset();

        // High amount = strong effect
        exciter.set_amount(0.9);
        for _ in 0..100 {
            exciter.process(input, input, &analysis);
        }
        let (out_high, _) = exciter.process(input, input, &analysis);

        // High amount should produce stronger effect
        let diff_low = (out_low - input).abs();
        let diff_high = (out_high - input).abs();

        assert!(diff_high > diff_low, "High amount should have stronger effect");
    }

    #[test]
    fn test_pitch_tracking_updates() {
        let mut exciter = IntelligentExciter::new(44100.0);

        let analysis1 = create_voiced_analysis(220.0); // A3
        exciter.process(0.5, 0.5, &analysis1);

        let pitch1 = exciter.current_pitch_hz;
        assert!((pitch1 - 220.0).abs() < 1.0);

        // Change pitch significantly
        let analysis2 = create_voiced_analysis(440.0); // A4
        exciter.process(0.5, 0.5, &analysis2);

        let pitch2 = exciter.current_pitch_hz;
        assert!((pitch2 - 440.0).abs() < 1.0);

        // Pitch should have updated
        assert_ne!(pitch1, pitch2);
    }

    #[test]
    fn test_reset() {
        let mut exciter = IntelligentExciter::new(44100.0);

        let analysis = create_voiced_analysis(220.0);

        // Process some audio to build up state
        for _ in 0..1000 {
            exciter.process(0.5, 0.5, &analysis);
        }

        exciter.reset();

        assert_eq!(exciter.phase_2x, 0.0);
        assert_eq!(exciter.phase_3x, 0.0);
        assert_eq!(exciter.phase_4x, 0.0);
        assert_eq!(exciter.current_pitch_hz, 0.0);
    }

    #[test]
    fn test_stereo_width() {
        let mut exciter = IntelligentExciter::new(44100.0);
        exciter.set_amount(0.5);
        exciter.set_mix(1.0);

        let analysis = create_voiced_analysis(220.0);
        let input = 0.5;

        // Process to establish harmonics
        for _ in 0..100 {
            exciter.process(input, input, &analysis);
        }

        let (out_l, out_r) = exciter.process(input, input, &analysis);

        // Left and right should differ slightly (stereo enhancement)
        assert_ne!(out_l, out_r);

        // But should be close (not wildly different)
        let diff = (out_l - out_r).abs();
        assert!(diff < 0.2, "Stereo difference should be subtle");
    }
}
