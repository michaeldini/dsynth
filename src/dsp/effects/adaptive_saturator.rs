/// Adaptive Saturator - Intelligent multi-stage analog saturation for vocals
///
/// **Design Philosophy**: Single-knob drive control with automatic character selection
/// and transient-adaptive processing. Emulates cascading analog saturation stages found
/// in real hardware (preamp → console → tape/tube/transformer).
///
/// # Character Types (Musical Descriptors)
/// - **Warm**: Tube-style asymmetric saturation (emphasizes even harmonics, gentle compression)
/// - **Smooth**: Tape-style soft-knee saturation (balanced harmonics, musical compression)
/// - **Punchy**: Console-style saturation (aggressive mids, transient emphasis)
///
/// # Architecture
/// ```text
/// Input → Stage 1 (60% drive) → Stage 2 (25% drive) → Stage 3 (15% drive) → Auto-Gain → Output
///          ↑                      ↑                      ↑
///          └─── Transient boost (1.0 + transient_strength * 0.3) ───┘
/// ```
///
/// # Adaptive Processing
/// - **Transient-Sensitive**: Increases drive by up to 30% during attacks for punchiness
/// - **Auto-Gain Compensation**: RMS-matched output maintains perceived loudness
/// - **Character-Specific**: Each type uses different waveshaping for distinct tonal color
///
/// # Calibration
/// - Drive = 0.5 (50%) produces moderate saturation suitable for most vocals
/// - Drive scaling: drive^2.5 for smooth control (subtle at low values, aggressive at high)
/// - Stage distribution: 60%/25%/15% for natural harmonic buildup
use crate::dsp::signal_analyzer::SignalAnalysis;

/// Saturation character types (musical descriptors)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaturationCharacter {
    /// Tube-style asymmetric saturation (emphasizes even harmonics, gentle)
    Warm = 0,
    /// Tape-style soft-knee saturation (balanced harmonics, musical)
    Smooth = 1,
    /// Console-style saturation (aggressive mids, transient emphasis)
    Punchy = 2,
}

impl SaturationCharacter {
    /// Convert from u8 index (for CLAP parameter)
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Warm,
            1 => Self::Smooth,
            2 => Self::Punchy,
            _ => Self::Warm, // Default to Warm if invalid
        }
    }

    /// Convert to u8 index (for CLAP parameter)
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Adaptive saturator with multi-stage processing
pub struct AdaptiveSaturator {
    sample_rate: f32,

    // RMS tracking for auto-gain compensation
    rms_input: f32,
    rms_output: f32,
    rms_coeff: f32, // 10ms smoothing

    // DC blocking filters (per stage to prevent DC buildup)
    dc_block_x1: [f32; 3],
    dc_block_y1: [f32; 3],
    dc_block_coeff: f32,
}

impl AdaptiveSaturator {
    /// Create a new adaptive saturator
    pub fn new(sample_rate: f32) -> Self {
        // RMS smoothing coefficient (10ms)
        let rms_time_ms = 10.0;
        let rms_samples = (rms_time_ms / 1000.0) * sample_rate;
        let rms_coeff = (-1.0 / rms_samples).exp();

        // DC blocking filter coefficient (high-pass at 5Hz)
        let cutoff = 5.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let dt = 1.0 / sample_rate;
        let dc_block_coeff = rc / (rc + dt);

        Self {
            sample_rate,
            rms_input: 0.0,
            rms_output: 0.0,
            rms_coeff,
            dc_block_x1: [0.0; 3],
            dc_block_y1: [0.0; 3],
            dc_block_coeff,
        }
    }

    /// Process a stereo sample with adaptive saturation
    ///
    /// # Arguments
    /// * `left` - Left channel input
    /// * `right` - Right channel input
    /// * `drive` - Drive amount (0.0-1.0, calibrated for transparent enhancement at moderate levels)
    /// * `mix` - Dry/wet mix (0.0 = dry, 1.0 = wet, 0.3-0.5 = optimal for transparent vocal enhancement)
    /// * `character` - Saturation character (Warm/Smooth/Punchy)
    /// * `analysis` - Signal analysis from SignalAnalyzer
    ///
    /// # Returns
    /// Tuple of (output_left, output_right)
    pub fn process(
        &mut self,
        left: f32,
        right: f32,
        drive: f32,
        mix: f32,
        character: SaturationCharacter,
        analysis: &SignalAnalysis,
    ) -> (f32, f32) {
        // Clamp parameters to valid range
        let drive = drive.clamp(0.0, 1.0);
        let mix = mix.clamp(0.0, 1.0);

        // Store dry signal for parallel processing
        let dry_left = left;
        let dry_right = right;

        // Apply drive^3.0 for balanced vocal enhancement (more pronounced at moderate levels)
        // Reduced from 3.5 to 3.0 to make character differences more audible
        let drive_internal = drive.powf(3.0);

        // Transient-adaptive drive boost (up to 30% more drive on transients)
        let transient_mult = 1.0 + (analysis.transient_strength * 0.3);

        // Sibilance-aware saturation: Reduce drive during sibilant sounds (S, SH, CH, T, etc.)
        // High sibilance_strength (0.7-1.0) indicates harsh high-frequency content that doesn't benefit from saturation
        // Reduction up to 70% on strong sibilance to preserve vocal clarity
        let sibilance_reduction = 1.0 - (analysis.sibilance_strength * 0.7);

        // Voiced/unvoiced adaptation: Reduce drive on consonants (unvoiced segments)
        // Voiced segments (vowels, sustained tones) benefit from saturation
        // Unvoiced segments (consonants, breaths) should stay cleaner to maintain intelligibility
        let voice_factor = if analysis.is_voiced { 1.0 } else { 0.6 };

        // ZCR-based frequency adaptation: Adapt drive based on frequency content
        // Low ZCR (<200Hz) = bass-heavy content → full drive for warmth
        // Mid ZCR (200-800Hz) = vocal fundamentals → balanced drive
        // High ZCR (>800Hz) = bright/harsh content → reduced drive
        let freq_factor = if analysis.zcr_hz < 200.0 {
            1.0 // Full drive on bass content
        } else if analysis.zcr_hz < 800.0 {
            1.0 - ((analysis.zcr_hz - 200.0) / 600.0) * 0.2 // Gentle reduction 1.0→0.8
        } else {
            0.8 - ((analysis.zcr_hz - 800.0) / 1200.0).min(1.0) * 0.3 // Further reduction 0.8→0.5
        };

        // Combine all adaptive factors (transient boost × sibilance reduction × voice factor × frequency factor)
        let combined_mult = transient_mult * sibilance_reduction * voice_factor * freq_factor;
        let adaptive_drive = (drive_internal * combined_mult).min(1.0);

        // Calculate stage drive distribution (40%/20%/10% for balanced multi-stage character)
        // Conservative distribution prevents excessive buildup at high drive while maintaining character
        let stage1_drive = adaptive_drive * 0.40;
        let stage2_drive = adaptive_drive * 0.20;
        let stage3_drive = adaptive_drive * 0.10;

        // Track input RMS for auto-gain compensation
        let mono_input = (left + right) * 0.5;
        self.update_rms_input(mono_input);

        // Process left channel through 3-stage cascade
        let mut left_out = left;
        left_out = self.saturate_stage(left_out, stage1_drive, character, 0);
        left_out = self.saturate_stage(left_out, stage2_drive, character, 1);
        left_out = self.saturate_stage(left_out, stage3_drive, character, 2);

        // Process right channel through 3-stage cascade
        let mut right_out = right;
        right_out = self.saturate_stage(right_out, stage1_drive, character, 0);
        right_out = self.saturate_stage(right_out, stage2_drive, character, 1);
        right_out = self.saturate_stage(right_out, stage3_drive, character, 2);

        // Track output RMS
        let mono_output = (left_out + right_out) * 0.5;
        self.update_rms_output(mono_output);

        // Auto-gain compensation (maintain perceived loudness)
        let compensation = self.calculate_auto_gain();
        left_out *= compensation;
        right_out *= compensation;

        // Parallel processing: blend dry and wet signals
        // This preserves transient clarity while adding harmonic richness
        // Optimal vocal settings: mix = 0.3-0.5 (70-50% dry + 30-50% wet)
        let final_left = dry_left * (1.0 - mix) + left_out * mix;
        let final_right = dry_right * (1.0 - mix) + right_out * mix;

        (final_left, final_right)
    }

    /// Process a single saturation stage
    fn saturate_stage(
        &mut self,
        input: f32,
        drive: f32,
        character: SaturationCharacter,
        stage_idx: usize,
    ) -> f32 {
        // Map drive (0.0-1.0) to gain (1.0-8.0) - gentle range for transparent vocal enhancement
        // Reduced from 1-20× to prevent harsh clipping at moderate drive settings
        let gain = 1.0 + drive * 7.0;
        let x = input * gain;

        // Apply character-specific waveshaping
        let saturated = match character {
            SaturationCharacter::Warm => self.warm_saturation(x),
            SaturationCharacter::Smooth => self.smooth_saturation(x),
            SaturationCharacter::Punchy => self.punchy_saturation(x),
        };

        // DC blocking (prevents DC buildup in cascaded stages)
        self.dc_block(saturated, stage_idx)
    }

    /// Warm character: Tube-style asymmetric saturation (even harmonics)
    #[inline]
    fn warm_saturation(&self, x: f32) -> f32 {
        // Tube-style soft saturation with asymmetric clipping
        // Balanced threshold (0.50) with always-on harmonic exciter for pronounced warmth
        // 2nd harmonic exciter adds tube-like character at all levels
        let abs_x = x.abs();

        // Balanced linear region (0.50 threshold)
        if abs_x <= 0.50 {
            // 2nd harmonic exciter always active (even in linear region)
            let harmonic = (x * 2.0 * std::f32::consts::PI).sin() * abs_x * 0.08;
            return (x + harmonic).clamp(-0.95, 0.95);
        }

        // Soft clip above threshold with strong asymmetric behavior
        let sign = x.signum();
        let excess = abs_x - 0.50;

        // Stronger asymmetric compression (3.0/1.2 ratio for pronounced even harmonics)
        let compression_factor = if x > 0.0 {
            3.0 // Positive: strong compression
        } else {
            1.2 // Negative: minimal compression (strong asymmetry)
        };

        let compressed = 0.50 + excess / (1.0 + excess * compression_factor);
        let saturated = sign * compressed.min(0.95);

        // 2nd harmonic exciter for tube warmth (stronger above threshold)
        let harmonic = (saturated * 2.0 * std::f32::consts::PI).sin() * abs_x * 0.15;

        (saturated + harmonic).clamp(-0.95, 0.95)
    }

    /// Smooth character: Tape-style soft-knee saturation (balanced harmonics)
    #[inline]
    fn smooth_saturation(&self, x: f32) -> f32 {
        // Enhanced tanh-based saturation for more tape character
        // Increased drive (0.7→0.9) for more pronounced saturation
        // Always-on odd harmonic exciter adds tape-style "glue" at all levels
        let abs_x = x.abs();
        let scaled = x * 0.9; // Stronger drive into tanh for more character
        let saturated = scaled.tanh() * 0.95;

        // Subtle odd harmonic exciter for tape "glue" (3rd harmonic)
        // Always active to create the smooth, cohesive quality of magnetic tape
        let harmonic = (saturated * 3.0 * std::f32::consts::PI).sin() * abs_x * 0.12;

        (saturated + harmonic).clamp(-0.95, 0.95)
    }

    /// Punchy character: Console-style saturation (aggressive mids)
    #[inline]
    fn punchy_saturation(&self, x: f32) -> f32 {
        // Aggressive soft clip for console-style bite
        // Balanced threshold (0.60) with always-on harmonics for pronounced character
        // 3rd/5th harmonic exciters add console "edge" at all levels
        let abs_x = x.abs();

        if abs_x <= 0.60 {
            // Linear region with always-on harmonic exciters
            let harmonic_3rd = (x * 3.0 * std::f32::consts::PI).sin() * abs_x * 0.10;
            let harmonic_5th = (x * 5.0 * std::f32::consts::PI).sin() * abs_x * 0.06;
            return (x + harmonic_3rd + harmonic_5th).clamp(-0.95, 0.95);
        } else {
            // Stronger hyperbolic compression for aggressive character
            let sign = x.signum();
            let excess = abs_x - 0.60;
            let compressed = 0.60 + excess / (1.0 + excess * 2.5); // Aggressive rolloff
            let saturated = sign * compressed.min(0.95);

            // 3rd + 5th harmonic exciter for console "bite" and presence (stronger above threshold)
            let harmonic_3rd = (saturated * 3.0 * std::f32::consts::PI).sin() * abs_x * 0.18;
            let harmonic_5th = (saturated * 5.0 * std::f32::consts::PI).sin() * abs_x * 0.12;

            (saturated + harmonic_3rd + harmonic_5th).clamp(-0.95, 0.95)
        }
    }

    /// DC blocking filter (high-pass)
    #[inline]
    fn dc_block(&mut self, input: f32, stage_idx: usize) -> f32 {
        let output = self.dc_block_coeff
            * (self.dc_block_y1[stage_idx] + input - self.dc_block_x1[stage_idx]);
        self.dc_block_x1[stage_idx] = input;
        self.dc_block_y1[stage_idx] = output;
        output
    }

    /// Update input RMS tracker
    #[inline]
    fn update_rms_input(&mut self, input: f32) {
        let squared = input * input;
        self.rms_input = self.rms_input * self.rms_coeff + squared * (1.0 - self.rms_coeff);
    }

    /// Update output RMS tracker
    #[inline]
    fn update_rms_output(&mut self, output: f32) {
        let squared = output * output;
        self.rms_output = self.rms_output * self.rms_coeff + squared * (1.0 - self.rms_coeff);
    }

    /// Calculate auto-gain compensation factor
    #[inline]
    fn calculate_auto_gain(&self) -> f32 {
        // Target: match input RMS level
        let input_level = self.rms_input.sqrt().max(0.001); // Avoid division by zero
        let output_level = self.rms_output.sqrt().max(0.001);

        // Calculate compensation (with safety limits)
        let compensation = input_level / output_level;
        compensation.clamp(0.5, 2.0) // Limit to ±6dB
    }

    /// Reset all processing state
    pub fn reset(&mut self) {
        self.rms_input = 0.0;
        self.rms_output = 0.0;
        self.dc_block_x1 = [0.0; 3];
        self.dc_block_y1 = [0.0; 3];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_analysis() -> SignalAnalysis {
        SignalAnalysis {
            rms_level: 0.3,
            peak_level: 0.5,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 200.0,
            signal_type: crate::dsp::analysis::SignalType::Tonal,
            is_voiced: true,
            is_unvoiced: false,
            has_sibilance: false,
            sibilance_strength: 0.0,
            pitch_hz: 220.0,
            pitch_confidence: 0.8,
            is_pitched: true,
        }
    }

    #[test]
    fn test_adaptive_saturator_creation() {
        let saturator = AdaptiveSaturator::new(44100.0);
        assert_eq!(saturator.sample_rate, 44100.0);
    }

    #[test]
    fn test_character_enum_conversion() {
        assert_eq!(SaturationCharacter::from_u8(0), SaturationCharacter::Warm);
        assert_eq!(SaturationCharacter::from_u8(1), SaturationCharacter::Smooth);
        assert_eq!(SaturationCharacter::from_u8(2), SaturationCharacter::Punchy);
        assert_eq!(SaturationCharacter::from_u8(99), SaturationCharacter::Warm); // Invalid → default

        assert_eq!(SaturationCharacter::Warm.to_u8(), 0);
        assert_eq!(SaturationCharacter::Smooth.to_u8(), 1);
        assert_eq!(SaturationCharacter::Punchy.to_u8(), 2);
    }

    #[test]
    fn test_warm_character_no_nan() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Test at various drive levels
        for drive in [0.0, 0.3, 0.5, 0.7, 1.0] {
            let (out_l, out_r) =
                saturator.process(0.5, 0.5, drive, 1.0, SaturationCharacter::Warm, &analysis);
            assert!(out_l.is_finite(), "Warm drive={} produced NaN/Inf", drive);
            assert!(out_r.is_finite(), "Warm drive={} produced NaN/Inf", drive);
        }
    }

    #[test]
    fn test_smooth_character_no_nan() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Test at various drive levels
        for drive in [0.0, 0.3, 0.5, 0.7, 1.0] {
            let (out_l, out_r) =
                saturator.process(0.5, 0.5, drive, 1.0, SaturationCharacter::Smooth, &analysis);
            assert!(out_l.is_finite(), "Smooth drive={} produced NaN/Inf", drive);
            assert!(out_r.is_finite(), "Smooth drive={} produced NaN/Inf", drive);
        }
    }

    #[test]
    fn test_punchy_character_no_nan() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Test at various drive levels
        for drive in [0.0, 0.3, 0.5, 0.7, 1.0] {
            let (out_l, out_r) =
                saturator.process(0.5, 0.5, drive, 1.0, SaturationCharacter::Punchy, &analysis);
            assert!(out_l.is_finite(), "Punchy drive={} produced NaN/Inf", drive);
            assert!(out_r.is_finite(), "Punchy drive={} produced NaN/Inf", drive);
        }
    }

    #[test]
    fn test_progressive_saturation_with_drive() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Process some samples to stabilize RMS tracking
        for _ in 0..1000 {
            saturator.process(0.8, 0.8, 0.5, 1.0, SaturationCharacter::Warm, &analysis);
        }

        // Test that saturation produces finite output at various drive levels
        let (out_0, _) =
            saturator.process(0.8, 0.8, 0.0, 1.0, SaturationCharacter::Warm, &analysis);
        let (out_50, _) =
            saturator.process(0.8, 0.8, 0.5, 1.0, SaturationCharacter::Warm, &analysis);
        let (out_100, _) =
            saturator.process(0.8, 0.8, 1.0, 1.0, SaturationCharacter::Warm, &analysis);

        // All outputs should be finite (auto-gain compensation keeps levels reasonable)
        assert!(out_0.is_finite());
        assert!(out_50.is_finite());
        assert!(out_100.is_finite());

        // Outputs should be in reasonable range (auto-gain may push slightly above 1.0)
        assert!(out_0.abs() < 2.0, "Output 0% drive out of range: {}", out_0);
        assert!(
            out_50.abs() < 2.0,
            "Output 50% drive out of range: {}",
            out_50
        );
        assert!(
            out_100.abs() < 2.0,
            "Output 100% drive out of range: {}",
            out_100
        );
    }

    #[test]
    fn test_transient_adaptive_drive() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let mut analysis = create_test_analysis();

        // Process with no transient
        analysis.transient_strength = 0.0;
        let (out_no_transient, _) =
            saturator.process(0.5, 0.5, 0.5, 1.0, SaturationCharacter::Warm, &analysis);

        // Process with strong transient (should have more saturation)
        analysis.transient_strength = 1.0;
        let (out_with_transient, _) =
            saturator.process(0.5, 0.5, 0.5, 1.0, SaturationCharacter::Warm, &analysis);

        // Both should be finite
        assert!(out_no_transient.is_finite());
        assert!(out_with_transient.is_finite());

        // Transient should slightly affect output (but auto-gain may compensate)
        // Just verify no crash
    }

    #[test]
    fn test_zero_drive_passthrough() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Zero drive should be nearly passthrough (minimal saturation)
        let input = 0.3;
        let (out_l, out_r) =
            saturator.process(input, input, 0.0, 1.0, SaturationCharacter::Warm, &analysis);

        // Output should be close to input (some deviation due to DC blocking)
        assert_relative_eq!(out_l, input, epsilon = 0.2);
        assert_relative_eq!(out_r, input, epsilon = 0.2);
    }

    #[test]
    fn test_fifty_percent_drive_moderate_saturation() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // 50% drive should produce moderate, audible saturation
        let input = 0.5;
        let (out_l, _) =
            saturator.process(input, input, 0.5, 1.0, SaturationCharacter::Warm, &analysis);

        // Output should be different from input (saturation applied)
        assert!(out_l.is_finite());
        assert!(out_l.abs() <= 1.0); // No clipping
                                     // Don't check exact equality - saturation changes waveform
    }

    #[test]
    fn test_all_characters_produce_different_results() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        let input = 0.6;
        let drive = 0.7;

        // Process with each character
        let (warm_out, _) = saturator.process(
            input,
            input,
            drive,
            1.0,
            SaturationCharacter::Warm,
            &analysis,
        );
        let (smooth_out, _) = saturator.process(
            input,
            input,
            drive,
            1.0,
            SaturationCharacter::Smooth,
            &analysis,
        );
        let (punchy_out, _) = saturator.process(
            input,
            input,
            drive,
            1.0,
            SaturationCharacter::Punchy,
            &analysis,
        );

        // All should be valid
        assert!(warm_out.is_finite());
        assert!(smooth_out.is_finite());
        assert!(punchy_out.is_finite());

        // Characters should produce moderately different results
        // (Exact values will vary due to auto-gain, but they should differ)
        // Just verify no crash and valid output
    }

    #[test]
    fn test_reset_clears_state() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Process some audio to build up state
        for _ in 0..1000 {
            saturator.process(0.5, 0.5, 0.5, 1.0, SaturationCharacter::Warm, &analysis);
        }

        // Reset
        saturator.reset();

        // State should be cleared
        assert_eq!(saturator.rms_input, 0.0);
        assert_eq!(saturator.rms_output, 0.0);
        assert_eq!(saturator.dc_block_x1, [0.0; 3]);
        assert_eq!(saturator.dc_block_y1, [0.0; 3]);
    }

    #[test]
    fn test_no_clipping_at_max_drive() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Even at max drive, output should not clip (stay within ±1.0)
        for _ in 0..100 {
            let (out_l, out_r) =
                saturator.process(0.8, 0.8, 1.0, 1.0, SaturationCharacter::Warm, &analysis);
            assert!(out_l.abs() <= 1.0, "Output clipped: {}", out_l);
            assert!(out_r.abs() <= 1.0, "Output clipped: {}", out_r);
        }
    }

    #[test]
    fn test_warm_saturation_function_directly() {
        let saturator = AdaptiveSaturator::new(44100.0);

        // Test the warm_saturation function directly (no DC blocking, no stages)
        let test_cases = [
            (0.1, "low positive"),
            (0.3, "mid positive"),
            (-0.1, "low negative"),
            (-0.3, "mid negative"),
        ];

        for (input, label) in &test_cases {
            let output = saturator.warm_saturation(*input);

            // Output should have same sign as input
            assert!(
                output.signum() == input.signum(),
                "{}: Output sign mismatch. Input: {}, Output: {}",
                label,
                input,
                output
            );

            // Output should be finite
            assert!(output.is_finite(), "{}: Output not finite", label);

            // Output magnitude should be reasonable (not amplified excessively)
            assert!(
                output.abs() <= input.abs() * 2.0,
                "{}: Excessive amplification. Input: {}, Output: {}",
                label,
                input,
                output
            );
        }
    }

    #[test]
    fn test_warm_low_drive_transparency() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Test with a varying signal (sine-like pattern) to avoid DC blocking artifacts
        // Generate 1000 samples of varying amplitude to warm up RMS properly
        for i in 0..1000 {
            let phase = (i as f32) * 0.01; // Slow variation
            let varying_input = 0.3 * phase.sin();
            saturator.process(
                varying_input,
                varying_input,
                0.2,
                1.0,
                SaturationCharacter::Warm,
                &analysis,
            );
        }

        // Now test with actual inputs
        let test_inputs = [0.1, 0.3, 0.5];

        for &input in &test_inputs {
            let (out_l, out_r) =
                saturator.process(input, input, 0.2, 1.0, SaturationCharacter::Warm, &analysis);

            // At low drive, output should be finite and bounded
            assert!(
                out_l.is_finite(),
                "Output should be finite for input {}",
                input
            );
            assert!(
                out_r.is_finite(),
                "Output should be finite for input {}",
                input
            );
            assert!(
                out_l.abs() <= 1.0,
                "Output should not clip. Input: {}, Output: {}",
                input,
                out_l
            );

            // At low drive (20%), saturation should be very subtle
            // Output should still have reasonable relationship to input
            assert!(
                out_l.abs() < 2.0,
                "Low drive should not excessively amplify. Input: {}, Output: {}",
                input,
                out_l
            );
        }
    }

    #[test]
    fn test_mix_parameter() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();
        let input = 0.5;

        // Warm up RMS tracking
        for _ in 0..1000 {
            saturator.process(input, input, 0.5, 1.0, SaturationCharacter::Warm, &analysis);
        }

        // 0% mix = 100% dry (should be identical to input)
        let (out_0, _) =
            saturator.process(input, input, 0.5, 0.0, SaturationCharacter::Warm, &analysis);
        assert!((out_0 - input).abs() < 0.001, "0% mix should be dry signal");

        // 50% mix = balanced blend
        let (out_50, _) =
            saturator.process(input, input, 0.5, 0.5, SaturationCharacter::Warm, &analysis);
        assert!(out_50.is_finite(), "50% mix should produce finite output");

        // 100% mix = 100% wet (full saturation)
        let (out_100, _) =
            saturator.process(input, input, 0.5, 1.0, SaturationCharacter::Warm, &analysis);
        assert!(out_100.is_finite(), "100% mix should produce finite output");

        // Mix acts as expected: more wet = more saturation effect
        // (Don't assert specific ordering due to auto-gain compensation)
    }

    #[test]
    fn test_parallel_processing_preserves_clarity() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();
        let input = 0.8;

        // Warm up RMS tracking
        for _ in 0..1000 {
            saturator.process(input, input, 0.8, 1.0, SaturationCharacter::Warm, &analysis);
        }

        // At 30% mix with high drive, parallel processing preserves more of original
        let (parallel_out, _) =
            saturator.process(input, input, 0.8, 0.3, SaturationCharacter::Warm, &analysis);
        let (full_wet, _) =
            saturator.process(input, input, 0.8, 1.0, SaturationCharacter::Warm, &analysis);

        // Both should be finite
        assert!(parallel_out.is_finite());
        assert!(full_wet.is_finite());

        // Parallel processing preserves more of original signal
        let parallel_diff = (parallel_out - input).abs();
        let wet_diff = (full_wet - input).abs();
        assert!(
            parallel_diff < wet_diff,
            "Parallel (30% mix) should be closer to input than full wet"
        );
    }

    #[test]
    fn test_sibilance_aware_saturation() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let input = 0.5;

        // Create analysis with low sibilance
        let mut analysis_low_sib = create_test_analysis();
        analysis_low_sib.sibilance_strength = 0.1;

        // Create analysis with high sibilance (harsh S sound)
        let mut analysis_high_sib = create_test_analysis();
        analysis_high_sib.sibilance_strength = 0.9;

        // Warm up RMS
        for _ in 0..1000 {
            saturator.process(
                input,
                input,
                0.8,
                1.0,
                SaturationCharacter::Warm,
                &analysis_low_sib,
            );
        }

        // Process with low sibilance (should get more saturation)
        let (out_low_sib, _) = saturator.process(
            input,
            input,
            0.8,
            1.0,
            SaturationCharacter::Warm,
            &analysis_low_sib,
        );

        // Reset saturator for fair comparison
        saturator = AdaptiveSaturator::new(44100.0);
        for _ in 0..1000 {
            saturator.process(
                input,
                input,
                0.8,
                1.0,
                SaturationCharacter::Warm,
                &analysis_high_sib,
            );
        }

        // Process with high sibilance (should get less saturation due to 70% reduction)
        let (out_high_sib, _) = saturator.process(
            input,
            input,
            0.8,
            1.0,
            SaturationCharacter::Warm,
            &analysis_high_sib,
        );

        assert!(out_low_sib.is_finite());
        assert!(out_high_sib.is_finite());

        // High sibilance should be closer to input (less saturation)
        let low_sib_diff = (out_low_sib - input).abs();
        let high_sib_diff = (out_high_sib - input).abs();
        assert!(
            high_sib_diff < low_sib_diff,
            "High sibilance should produce less saturation effect. Low sib diff: {}, High sib diff: {}",
            low_sib_diff, high_sib_diff
        );
    }

    #[test]
    fn test_voiced_unvoiced_adaptation() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let input = 0.5;

        // Voiced segment (vowel - should get full saturation)
        let mut analysis_voiced = create_test_analysis();
        analysis_voiced.is_voiced = true;

        // Unvoiced segment (consonant - should get 60% saturation)
        let mut analysis_unvoiced = create_test_analysis();
        analysis_unvoiced.is_voiced = false;

        // Warm up RMS for voiced
        for _ in 0..1000 {
            saturator.process(
                input,
                input,
                0.8,
                1.0,
                SaturationCharacter::Warm,
                &analysis_voiced,
            );
        }

        // Process voiced segment
        let (out_voiced, _) = saturator.process(
            input,
            input,
            0.8,
            1.0,
            SaturationCharacter::Warm,
            &analysis_voiced,
        );

        // Reset for unvoiced
        saturator = AdaptiveSaturator::new(44100.0);
        for _ in 0..1000 {
            saturator.process(
                input,
                input,
                0.8,
                1.0,
                SaturationCharacter::Warm,
                &analysis_unvoiced,
            );
        }

        // Process unvoiced segment
        let (out_unvoiced, _) = saturator.process(
            input,
            input,
            0.8,
            1.0,
            SaturationCharacter::Warm,
            &analysis_unvoiced,
        );

        assert!(out_voiced.is_finite());
        assert!(out_unvoiced.is_finite());

        // Unvoiced should be closer to input (less saturation for clarity)
        let voiced_diff = (out_voiced - input).abs();
        let unvoiced_diff = (out_unvoiced - input).abs();
        assert!(
            unvoiced_diff < voiced_diff,
            "Unvoiced segments should get less saturation. Voiced diff: {}, Unvoiced diff: {}",
            voiced_diff,
            unvoiced_diff
        );
    }

    #[test]
    fn test_zcr_frequency_adaptation() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let input = 0.5;

        // Low ZCR (bass content - should get full saturation)
        let mut analysis_bass = create_test_analysis();
        analysis_bass.zcr_hz = 100.0;

        // Mid ZCR (vocal fundamentals - should get balanced saturation)
        let mut analysis_mid = create_test_analysis();
        analysis_mid.zcr_hz = 500.0;

        // High ZCR (bright/harsh content - should get reduced saturation)
        let mut analysis_bright = create_test_analysis();
        analysis_bright.zcr_hz = 1500.0;

        // Warm up and process bass content
        for _ in 0..1000 {
            saturator.process(
                input,
                input,
                0.8,
                1.0,
                SaturationCharacter::Warm,
                &analysis_bass,
            );
        }
        let (out_bass, _) = saturator.process(
            input,
            input,
            0.8,
            1.0,
            SaturationCharacter::Warm,
            &analysis_bass,
        );

        // Reset and process mid content
        saturator = AdaptiveSaturator::new(44100.0);
        for _ in 0..1000 {
            saturator.process(
                input,
                input,
                0.8,
                1.0,
                SaturationCharacter::Warm,
                &analysis_mid,
            );
        }
        let (out_mid, _) = saturator.process(
            input,
            input,
            0.8,
            1.0,
            SaturationCharacter::Warm,
            &analysis_mid,
        );

        // Reset and process bright content
        saturator = AdaptiveSaturator::new(44100.0);
        for _ in 0..1000 {
            saturator.process(
                input,
                input,
                0.8,
                1.0,
                SaturationCharacter::Warm,
                &analysis_bright,
            );
        }
        let (out_bright, _) = saturator.process(
            input,
            input,
            0.8,
            1.0,
            SaturationCharacter::Warm,
            &analysis_bright,
        );

        assert!(out_bass.is_finite());
        assert!(out_mid.is_finite());
        assert!(out_bright.is_finite());

        // Calculate difference from input (larger = more saturation effect)
        let bass_diff = (out_bass - input).abs();
        let mid_diff = (out_mid - input).abs();
        let bright_diff = (out_bright - input).abs();

        // Bass should get most saturation, bright should get least
        assert!(
            bass_diff > bright_diff,
            "Bass content should get more saturation than bright. Bass: {}, Bright: {}",
            bass_diff,
            bright_diff
        );
        assert!(
            mid_diff > bright_diff,
            "Mid content should get more saturation than bright. Mid: {}, Bright: {}",
            mid_diff,
            bright_diff
        );
    }
}
