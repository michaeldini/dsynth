//! Adaptive Compressor - Intelligent compression using signal analysis
//!
//! Unlike traditional static compressors, this compressor adapts its behavior
//! based on real-time signal analysis:
//!
//! - **Gentler on transients** - Preserves punch and attack of drum hits/consonants
//! - **Pitch-responsive** - Low-pitched notes get more compression than high-pitched
//! - **Content-aware attack** - Fast attack for transients, slower for sustained notes
//!
//! # How It Works
//! 1. Receives `SignalAnalysis` with pre-computed signal features
//! 2. Adapts compression parameters (ratio, attack) based on signal characteristics
//! 3. Applies intelligent gain reduction
//!
//! # Parameters
//! Standard 4 compressor params (all intelligence is automatic):
//! - `threshold_db` (-60dB to 0dB)
//! - `ratio` (1:1 to 20:1)
//! - `attack_ms` (0.1ms to 100ms)
//! - `release_ms` (10ms to 1000ms)
//!
//! # Adaptive Behavior
//! ```text
//! TRANSIENT DETECTED:
//!   effective_attack = min(attack, 2ms)  // Fast attack preserves punch
//!   effective_ratio = ratio * 0.6         // Gentler compression (40% reduction)
//!
//! PITCHED CONTENT (low notes):
//!   effective_threshold = threshold - pitch_offset  // More compression for bass
//!   // 80Hz = -6dB offset, 400Hz = 0dB offset, 800Hz = +3dB offset
//!
//! SUSTAINED CONTENT:
//!   effective_attack = attack             // Standard behavior
//!   effective_ratio = ratio
//! ```

use crate::dsp::signal_analyzer::SignalAnalysis;

/// Adaptive compressor with signal-aware intelligence
pub struct AdaptiveCompressor {
    sample_rate: f32,

    /// Base parameters (user-facing)
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,

    /// Soft knee width in dB
    knee_db: f32,

    /// Makeup gain in dB
    makeup_gain_db: f32,

    /// Envelope follower state (stereo-linked, uses max of L/R)
    envelope: f32,

    /// Time constants (calculated from ms values)
    attack_coeff: f32,
    release_coeff: f32,

    /// Fast attack coefficient for transients
    fast_attack_coeff: f32,
}

impl AdaptiveCompressor {
    /// Create a new adaptive compressor
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut comp = Self {
            sample_rate,
            threshold_db: -20.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            knee_db: 6.0, // Soft knee by default
            makeup_gain_db: 0.0,
            envelope: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            fast_attack_coeff: 0.0,
        };

        comp.update_coefficients();
        comp
    }

    /// Set threshold in dB (-60dB to 0dB)
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db.clamp(-60.0, 0.0);
    }

    /// Set compression ratio (1:1 to 20:1)
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(1.0, 20.0);
    }

    /// Set attack time in milliseconds (0.1ms to 100ms)
    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack_ms = attack_ms.clamp(0.1, 100.0);
        self.update_coefficients();
    }

    /// Set release time in milliseconds (10ms to 1000ms)
    pub fn set_release(&mut self, release_ms: f32) {
        self.release_ms = release_ms.clamp(10.0, 1000.0);
        self.update_coefficients();
    }

    /// Set knee width in dB (0 = hard knee, 6-10 = soft knee)
    pub fn set_knee(&mut self, knee_db: f32) {
        self.knee_db = knee_db.clamp(0.0, 20.0);
    }

    /// Set makeup gain in dB (0dB to +30dB)
    pub fn set_makeup_gain(&mut self, gain_db: f32) {
        self.makeup_gain_db = gain_db.clamp(0.0, 30.0);
    }

    /// Convert milliseconds to exponential smoothing coefficient
    fn ms_to_coeff(time_ms: f32, sample_rate: f32) -> f32 {
        let time_sec = time_ms / 1000.0;
        let samples = time_sec * sample_rate;
        (-1.0 / samples).exp()
    }

    /// Update time-based coefficients
    fn update_coefficients(&mut self) {
        self.attack_coeff = Self::ms_to_coeff(self.attack_ms, self.sample_rate);
        self.release_coeff = Self::ms_to_coeff(self.release_ms, self.sample_rate);

        // Fast attack for transients (fixed at 2ms)
        self.fast_attack_coeff = Self::ms_to_coeff(2.0, self.sample_rate);
    }

    /// Convert amplitude to dB
    #[inline]
    fn amp_to_db(amp: f32) -> f32 {
        20.0 * amp.max(1e-10).log10()
    }

    /// Convert dB to amplitude
    #[inline]
    fn db_to_amp(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
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
        // === STEP 1: Compute Adaptive Parameters ===

        // 1a. Adaptive Attack Time (transient detection)
        let effective_attack_coeff = if analysis.is_transient {
            // FAST attack for transients - preserves punch
            // Use pre-computed 2ms coefficient
            self.fast_attack_coeff
        } else {
            // Standard attack for sustained content
            self.attack_coeff
        };

        // 1b. Adaptive Ratio (transient detection)
        let effective_ratio = if analysis.is_transient {
            // GENTLER compression on transients
            // Reduce ratio by 40% to preserve attack dynamics
            self.ratio * 0.6
        } else {
            // Standard compression for sustained content
            self.ratio
        };

        // 1c. Adaptive Threshold (pitch-responsive)
        let effective_threshold = if analysis.is_pitched {
            // Pitch-responsive compression
            // Low pitches (80-200Hz) = more compression (lower threshold)
            // High pitches (400-800Hz) = less compression (higher threshold)

            // Logarithmic scaling gives more resolution in vocal range
            let pitch_hz = analysis.pitch_hz.clamp(80.0, 800.0);

            // Normalize: 80Hz = 0.0, 400Hz = 0.5, 800Hz = 1.0
            let pitch_norm = (pitch_hz.ln() - 80.0_f32.ln()) / (800.0_f32.ln() - 80.0_f32.ln());

            // Threshold offset: -6dB at 80Hz, 0dB at 400Hz, +3dB at 800Hz
            // This means bass gets compressed more, treble gets compressed less
            let pitch_offset = (pitch_norm - 0.5) * 9.0; // ±4.5dB range

            self.threshold_db + pitch_offset
        } else {
            // No pitch detected - use standard threshold
            self.threshold_db
        };

        // === STEP 2: Envelope Follower ===
        // Stereo-linked: use max of both channels
        let input_peak = left.abs().max(right.abs());
        let input_db = Self::amp_to_db(input_peak);

        // Update envelope with adaptive attack time
        self.envelope = if input_db > self.envelope {
            // Attack phase - use adaptive coefficient
            effective_attack_coeff * self.envelope + (1.0 - effective_attack_coeff) * input_db
        } else {
            // Release phase - always use standard release
            self.release_coeff * self.envelope + (1.0 - self.release_coeff) * input_db
        };

        // === STEP 3: Calculate Gain Reduction ===
        let gain_reduction =
            self.calculate_gain_reduction(self.envelope, effective_threshold, effective_ratio);

        // === STEP 4: Apply Compression ===
        let makeup = Self::db_to_amp(self.makeup_gain_db);
        let output_left = left * gain_reduction * makeup;
        let output_right = right * gain_reduction * makeup;

        (output_left, output_right)
    }

    /// Calculate gain reduction based on input level
    #[inline]
    fn calculate_gain_reduction(&self, input_db: f32, threshold: f32, ratio: f32) -> f32 {
        // Calculate how much the signal exceeds the threshold
        let overshoot = input_db - threshold;

        if overshoot <= -self.knee_db * 0.5 {
            // Below threshold - no compression
            1.0
        } else if overshoot >= self.knee_db * 0.5 {
            // Above threshold + knee - full compression
            let gain_reduction_db = overshoot * (1.0 - 1.0 / ratio);
            Self::db_to_amp(-gain_reduction_db)
        } else {
            // In the knee region - soft transition
            let knee_overshoot = overshoot + self.knee_db * 0.5;
            let gain_reduction_db =
                knee_overshoot * knee_overshoot / (2.0 * self.knee_db) * (1.0 - 1.0 / ratio);
            Self::db_to_amp(-gain_reduction_db)
        }
    }

    /// Reset compressor state
    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    /// Get current gain reduction in dB (for metering)
    pub fn get_gain_reduction_db(&self) -> f32 {
        let overshoot = self.envelope - self.threshold_db;
        if overshoot > 0.0 {
            -overshoot * (1.0 - 1.0 / self.ratio)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::SignalType;
    use approx::assert_relative_eq;

    fn create_default_analysis() -> SignalAnalysis {
        SignalAnalysis {
            rms_level: 0.1,
            peak_level: 0.1,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 500.0,
            signal_type: SignalType::Tonal,
            is_voiced: true,
            is_unvoiced: false,
            has_sibilance: false,
            sibilance_strength: 0.0,
            pitch_hz: 220.0,
            pitch_confidence: 0.0,
            is_pitched: false,
        }
    }

    #[test]
    fn test_adaptive_compressor_creation() {
        let comp = AdaptiveCompressor::new(44100.0);
        assert_eq!(comp.sample_rate, 44100.0);
        assert_eq!(comp.threshold_db, -20.0);
        assert_eq!(comp.ratio, 4.0);
    }

    #[test]
    fn test_transient_gentler_compression() {
        let mut comp = AdaptiveCompressor::new(44100.0);
        comp.set_threshold(-20.0);
        comp.set_ratio(8.0); // High ratio
        comp.set_attack(10.0);
        comp.set_release(100.0);

        // Test 1: Sustained content (normal compression)
        let mut analysis = create_default_analysis();
        analysis.is_transient = false;

        let mut sustained_outputs = Vec::new();
        for _ in 0..100 {
            let (out_l, _) = comp.process(0.6, 0.6, &analysis);
            sustained_outputs.push(out_l.abs());
        }
        let sustained_avg = sustained_outputs.iter().sum::<f32>() / sustained_outputs.len() as f32;

        // Reset for second test
        comp.reset();

        // Test 2: Transient (gentler compression)
        let mut analysis = create_default_analysis();
        analysis.is_transient = true;
        analysis.transient_strength = 0.8;

        let mut transient_outputs = Vec::new();
        for _ in 0..100 {
            let (out_l, _) = comp.process(0.6, 0.6, &analysis);
            transient_outputs.push(out_l.abs());
        }
        let transient_avg = transient_outputs.iter().sum::<f32>() / transient_outputs.len() as f32;

        // Transient should have LESS compression (higher output level)
        assert!(
            transient_avg > sustained_avg,
            "Transient should be compressed less. Transient={:.4}, Sustained={:.4}",
            transient_avg,
            sustained_avg
        );
    }

    #[test]
    fn test_pitch_responsive_compression() {
        let mut comp = AdaptiveCompressor::new(44100.0);
        comp.set_threshold(-20.0);
        comp.set_ratio(4.0);

        // Test 1: Low pitch (should get more compression)
        let mut analysis = create_default_analysis();
        analysis.is_pitched = true;
        analysis.pitch_hz = 100.0; // Low pitch
        analysis.pitch_confidence = 0.8;

        let mut low_pitch_outputs = Vec::new();
        for _ in 0..100 {
            let (out_l, _) = comp.process(0.5, 0.5, &analysis);
            low_pitch_outputs.push(out_l.abs());
        }
        let low_pitch_avg = low_pitch_outputs.iter().sum::<f32>() / low_pitch_outputs.len() as f32;

        // Reset
        comp.reset();

        // Test 2: High pitch (should get less compression)
        analysis.pitch_hz = 600.0; // High pitch

        let mut high_pitch_outputs = Vec::new();
        for _ in 0..100 {
            let (out_l, _) = comp.process(0.5, 0.5, &analysis);
            high_pitch_outputs.push(out_l.abs());
        }
        let high_pitch_avg =
            high_pitch_outputs.iter().sum::<f32>() / high_pitch_outputs.len() as f32;

        // High pitch should have LESS compression (higher output)
        assert!(
            high_pitch_avg > low_pitch_avg,
            "High pitch should be compressed less. High={:.4}, Low={:.4}",
            high_pitch_avg,
            low_pitch_avg
        );
    }

    #[test]
    fn test_fast_attack_on_transients() {
        let mut comp = AdaptiveCompressor::new(44100.0);
        comp.set_threshold(-20.0);
        comp.set_ratio(6.0);
        comp.set_attack(50.0); // Slow attack

        // Feed quiet signal to establish baseline
        let mut analysis = create_default_analysis();
        analysis.is_transient = false;
        for _ in 0..100 {
            comp.process(0.01, 0.01, &analysis);
        }

        // Now process loud transient - should use fast attack (2ms) and gentler ratio
        analysis.is_transient = true;

        // Process several samples to build up envelope with fast attack
        let mut outputs = Vec::new();
        for _ in 0..200 {
            let (out_l, _) = comp.process(0.6, 0.6, &analysis);
            outputs.push(out_l);
        }

        // Later samples should show compression (envelope has caught up)
        let final_output = outputs[outputs.len() - 1];

        // Transient gets gentler ratio (6 * 0.6 = 3.6)
        // Should compress but preserve more dynamics than sustained content
        assert!(
            final_output < 0.6,
            "Should apply compression to loud signal: {:.4}",
            final_output
        );
        assert!(final_output > 0.1, "But not crush it: {:.4}", final_output);
    }

    #[test]
    fn test_no_compression_below_threshold() {
        let mut comp = AdaptiveCompressor::new(44100.0);
        comp.set_threshold(-10.0); // High threshold
        comp.set_ratio(4.0);
        comp.set_knee(0.0); // Hard knee

        let analysis = create_default_analysis();

        // Feed quiet signal far below threshold
        let input = 0.001; // About -60dB (far below -10dB threshold)

        // Process to establish envelope
        for _ in 0..200 {
            comp.process(input, input, &analysis);
        }

        let (out_l, out_r) = comp.process(input, input, &analysis);

        // Should pass through with minimal change (far below threshold, outside knee)
        let difference = (out_l - input).abs();
        assert!(
            difference < 0.0005,
            "Output should be close to input. Input={:.6}, Output={:.6}, Diff={:.6}",
            input,
            out_l,
            difference
        );
    }

    #[test]
    fn test_makeup_gain() {
        let mut comp = AdaptiveCompressor::new(44100.0);
        comp.set_threshold(-70.0); // Extremely low threshold
        comp.set_ratio(1.5); // Very gentle ratio
        comp.set_knee(0.0);

        let analysis = create_default_analysis();
        let input = 0.1;

        // Test WITHOUT makeup gain
        comp.set_makeup_gain(0.0);
        for _ in 0..1000 {
            comp.process(input, input, &analysis);
        }
        let (out_without_makeup, _) = comp.process(input, input, &analysis);

        // Reset and test WITH makeup gain
        comp.reset();
        comp.set_makeup_gain(18.0); // +18dB = 8× linear
        for _ in 0..1000 {
            comp.process(input, input, &analysis);
        }
        let (out_with_makeup, _) = comp.process(input, input, &analysis);

        // With makeup gain should be significantly higher
        assert!(
            out_with_makeup > out_without_makeup * 4.0,
            "Makeup gain should significantly boost output. Without={:.4}, With={:.4}",
            out_without_makeup,
            out_with_makeup
        );
    }

    #[test]
    fn test_reset() {
        let mut comp = AdaptiveCompressor::new(44100.0);
        let analysis = create_default_analysis();

        // Build up envelope state
        for _ in 0..100 {
            comp.process(0.5, 0.5, &analysis);
        }

        comp.reset();

        assert_eq!(comp.envelope, 0.0);
    }

    #[test]
    fn test_stereo_linked() {
        let mut comp = AdaptiveCompressor::new(44100.0);
        comp.set_threshold(-20.0);
        comp.set_ratio(4.0);

        let analysis = create_default_analysis();

        // Feed asymmetric input (loud left, quiet right)
        for _ in 0..100 {
            comp.process(0.8, 0.1, &analysis);
        }

        // Both channels should get same compression (stereo-linked)
        let (out_l, out_r) = comp.process(0.8, 0.1, &analysis);

        // Gain reduction ratio should be similar (within 10%)
        let left_reduction = out_l / 0.8;
        let right_reduction = out_r / 0.1;

        assert_relative_eq!(left_reduction, right_reduction, epsilon = 0.1);
    }
}
