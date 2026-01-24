/// Smart Delay - Transient-aware delay effect for vocal enhancement
///
/// This module implements an intelligent delay that automatically adapts to signal characteristics:
/// - **Transients**: Delay mix reduced to 0% to preserve attack clarity
/// - **Non-transients**: Full delay applied for space and depth
/// - **Smooth transitions**: 10ms crossfades prevent audio artifacts
///
/// Uses SignalAnalysis.is_transient for real-time adaptation.
use crate::dsp::signal_analyzer::SignalAnalysis;

/// Smart delay with transient-aware mix control
pub struct SmartDelay {
    /// Circular delay buffers (stereo)
    delay_buffer_l: Vec<f32>,
    delay_buffer_r: Vec<f32>,

    /// Write position in delay buffer
    write_pos: usize,

    /// Maximum delay buffer size in samples
    max_delay_samples: usize,

    /// Sample rate
    sample_rate: f32,

    /// Current mix amount (smoothed, 0.0-1.0)
    current_mix: f32,

    /// Target mix amount based on transient detection
    target_mix: f32,

    /// Smoothing coefficient for mix changes (prevents clicks)
    mix_smooth_coeff: f32,

    /// Sensitivity to transients (0.0 = ignore transients, 1.0 = very sensitive)
    sensitivity: f32,
}

impl SmartDelay {
    /// Create a new SmartDelay
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    /// * `max_delay_ms` - Maximum delay time in milliseconds (determines buffer size)
    pub fn new(sample_rate: f32, max_delay_ms: f32) -> Self {
        // Allocate buffer for maximum delay time
        let max_delay_samples = (sample_rate * max_delay_ms / 1000.0).ceil() as usize;

        // Calculate smoothing coefficient for 30ms transitions (gentler than before)
        let smooth_time_samples = sample_rate * 0.030; // 30ms
        let mix_smooth_coeff = 1.0 - (-1.0 / smooth_time_samples).exp();

        Self {
            delay_buffer_l: vec![0.0; max_delay_samples],
            delay_buffer_r: vec![0.0; max_delay_samples],
            write_pos: 0,
            max_delay_samples,
            sample_rate,
            current_mix: 0.0,
            target_mix: 0.0,
            mix_smooth_coeff,
            sensitivity: 0.5, // Default moderate sensitivity
        }
    }

    /// Process a single stereo sample with smart delay
    ///
    /// # Arguments
    /// * `input_l` - Left channel input sample
    /// * `input_r` - Right channel input sample
    /// * `delay_time_ms` - Delay time in milliseconds (50-500ms recommended)
    /// * `feedback` - Feedback amount (0.0-0.8, higher = more repeats)
    /// * `mix` - Dry/wet mix (0.0-1.0, user-controlled base mix)
    /// * `sensitivity` - Transient sensitivity (0.0 = ignore transients, 1.0 = very sensitive)
    /// * `analysis` - Signal analysis data (used for transient detection)
    ///
    /// # Returns
    /// Tuple of (left_output, right_output)
    pub fn process(
        &mut self,
        input_l: f32,
        input_r: f32,
        delay_time_ms: f32,
        feedback: f32,
        mix: f32,
        sensitivity: f32,
        analysis: &SignalAnalysis,
    ) -> (f32, f32) {
        // Update sensitivity
        self.sensitivity = sensitivity.clamp(0.0, 1.0);

        // Calculate delay time in samples (clamp to valid range)
        let delay_samples = (self.sample_rate * delay_time_ms / 1000.0)
            .max(1.0)
            .min(self.max_delay_samples as f32 - 1.0) as usize;

        // Determine target mix based on transient strength (analog, not binary)
        // Use transient_strength (0.0-2.0+) and sensitivity to calculate reduction
        // Lower sensitivity = need stronger transient to reduce delay
        // Higher sensitivity = even weak transients reduce delay
        let transient_threshold = 0.3 + (1.0 - self.sensitivity) * 0.7; // Range: 0.3-1.0
        let transient_amount =
            ((analysis.transient_strength - transient_threshold) / 0.5).clamp(0.0, 1.0);

        // Reduce mix proportionally to transient strength
        // transient_amount 0.0 = full mix, 1.0 = no mix
        self.target_mix = mix * (1.0 - transient_amount);

        // Smooth current mix toward target (prevents clicks)
        self.current_mix += (self.target_mix - self.current_mix) * self.mix_smooth_coeff;

        // Calculate read position (wrap around buffer)
        let read_pos =
            (self.write_pos + self.max_delay_samples - delay_samples) % self.max_delay_samples;

        // Read delayed samples
        let delayed_l = self.delay_buffer_l[read_pos];
        let delayed_r = self.delay_buffer_r[read_pos];

        // Clamp feedback to safe range (prevent infinite buildup)
        let safe_feedback = feedback.clamp(0.0, 0.8);

        // Write input + feedback to delay buffer
        self.delay_buffer_l[self.write_pos] = input_l + delayed_l * safe_feedback;
        self.delay_buffer_r[self.write_pos] = input_r + delayed_r * safe_feedback;

        // Advance write position (circular buffer)
        self.write_pos = (self.write_pos + 1) % self.max_delay_samples;

        // Mix dry and wet signals using smoothed mix amount
        let output_l = input_l * (1.0 - self.current_mix) + delayed_l * self.current_mix;
        let output_r = input_r * (1.0 - self.current_mix) + delayed_r * self.current_mix;

        (output_l, output_r)
    }

    /// Reset delay buffers and state
    pub fn reset(&mut self) {
        self.delay_buffer_l.fill(0.0);
        self.delay_buffer_r.fill(0.0);
        self.write_pos = 0;
        self.current_mix = 0.0;
        self.target_mix = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_smart_delay_creation() {
        let delay = SmartDelay::new(44100.0, 500.0);
        assert_eq!(delay.max_delay_samples, 22050); // 500ms @ 44.1kHz
        assert_eq!(delay.write_pos, 0);
        assert_eq!(delay.current_mix, 0.0);
    }

    #[test]
    fn test_transient_bypasses_delay() {
        let mut delay = SmartDelay::new(44100.0, 500.0);

        // Create strong transient analysis (transient_strength = 1.5)
        let transient_analysis = SignalAnalysis {
            is_transient: true,
            transient_strength: 1.5, // Strong transient
            ..Default::default()
        };

        // Process with moderate sensitivity (0.5)
        // Strong transient should significantly reduce mix
        for _ in 0..500 {
            delay.process(1.0, 1.0, 100.0, 0.5, 1.0, 0.5, &transient_analysis);
        }

        // During strong transients with moderate sensitivity, mix should be reduced
        assert!(
            delay.current_mix < 0.3,
            "Mix should be low during strong transients, got {}",
            delay.current_mix
        );
    }

    #[test]
    fn test_non_transient_applies_delay() {
        let mut delay = SmartDelay::new(44100.0, 500.0);

        // Create sustained content analysis (low transient_strength)
        let sustained_analysis = SignalAnalysis {
            is_transient: false,
            transient_strength: 0.1, // Very low transient content
            ..Default::default()
        };

        // Process with moderate sensitivity (0.5)
        // Low transient_strength should allow near-full mix
        // Need enough samples for 30ms smoothing to converge (5000 samples ≈ 113ms)
        for _ in 0..5000 {
            delay.process(1.0, 1.0, 100.0, 0.5, 1.0, 0.5, &sustained_analysis);
        }

        // During sustained content, current_mix should approach user mix (1.0)
        // With 30ms smoothing, expect ~95% convergence after 5000 samples
        assert!(
            delay.current_mix > 0.90,
            "Mix should be near 1.0 during sustained content, got {}",
            delay.current_mix
        );
    }

    #[test]
    fn test_delay_time_accuracy() {
        let mut delay = SmartDelay::new(44100.0, 500.0);
        let delay_time_ms = 100.0; // 100ms delay
        let delay_samples = (44100.0 * delay_time_ms / 1000.0) as usize; // 4410 samples

        let analysis = SignalAnalysis {
            is_transient: false, // Not a transient, so mix will be applied
            ..Default::default()
        };

        // Feed constant signal to fill delay buffer
        for _ in 0..delay_samples + 500 {
            delay.process(1.0, 1.0, delay_time_ms, 0.0, 1.0, 0.5, &analysis);
        }

        // Now feed silence - delayed signal should still come through
        let mut found_delayed_signal = false;
        for _ in 0..delay_samples + 100 {
            let (out_l, _) = delay.process(0.0, 0.0, delay_time_ms, 0.0, 1.0, 0.5, &analysis);
            if out_l > 0.5 {
                found_delayed_signal = true;
                break;
            }
        }

        assert!(
            found_delayed_signal,
            "Delayed signal should appear after delay time"
        );
    }

    #[test]
    fn test_feedback_creates_repeats() {
        let mut delay = SmartDelay::new(44100.0, 500.0);
        let delay_time_ms = 50.0; // Short delay for testing
        let feedback = 0.5;

        let analysis = SignalAnalysis::default();

        // Feed single impulse
        delay.process(1.0, 1.0, delay_time_ms, feedback, 1.0, 0.5, &analysis);

        // Process enough samples to see multiple echoes
        let mut max_output = 0.0_f32;
        let delay_samples = (44100.0 * delay_time_ms / 1000.0) as usize;

        for _ in 0..delay_samples * 5 {
            let (out_l, _) = delay.process(0.0, 0.0, delay_time_ms, feedback, 1.0, 0.5, &analysis);
            max_output = max_output.max(out_l.abs());
        }

        // With feedback, we should see repeating echoes (max > 0.1)
        assert!(
            max_output > 0.1,
            "Feedback should create repeating echoes, max output: {}",
            max_output
        );
    }

    #[test]
    fn test_feedback_clamping() {
        let mut delay = SmartDelay::new(44100.0, 500.0);

        let analysis = SignalAnalysis::default();

        // Process with extreme feedback values (should be clamped internally)
        for _ in 0..1000 {
            let (out_l, out_r) = delay.process(0.5, 0.5, 100.0, 1.5, 1.0, 0.5, &analysis); // feedback > 1.0

            // Output should remain finite and reasonable
            assert!(
                out_l.is_finite() && out_l.abs() < 10.0,
                "Output should be finite and bounded"
            );
            assert!(
                out_r.is_finite() && out_r.abs() < 10.0,
                "Output should be finite and bounded"
            );
        }
    }

    #[test]
    fn test_stereo_processing() {
        let mut delay = SmartDelay::new(44100.0, 500.0);

        let analysis = SignalAnalysis {
            is_transient: false, // Not a transient, so delay will be applied
            ..Default::default()
        };

        // Feed different constant values to L/R channels to fill buffer
        let delay_samples = (44100.0 * 100.0 / 1000.0) as usize;
        for _ in 0..delay_samples + 500 {
            delay.process(1.0, 0.5, 100.0, 0.0, 1.0, 0.5, &analysis);
        }

        // Now check that delayed L and R are different
        let (out_l, out_r) = delay.process(0.0, 0.0, 100.0, 0.0, 1.0, 0.5, &analysis);

        // Stereo channels should be processed independently
        // Since we fed different values, delayed outputs should be different
        assert!(
            (out_l - out_r).abs() > 0.01,
            "Left and right channels should process independently, got L={} R={}",
            out_l,
            out_r
        );
    }

    #[test]
    fn test_reset_clears_state() {
        let mut delay = SmartDelay::new(44100.0, 500.0);

        let analysis = SignalAnalysis {
            is_transient: false,
            ..Default::default()
        };

        // Fill delay buffer with non-zero values
        for _ in 0..1000 {
            delay.process(1.0, 1.0, 100.0, 0.5, 1.0, 0.5, &analysis);
        }

        // Reset
        delay.reset();

        // Process silence - output should be near zero
        // Reset sets current_mix to 0, but process() will start ramping it up
        // So we expect very small values, not exactly zero
        let (out_l, out_r) = delay.process(0.0, 0.0, 100.0, 0.5, 1.0, 0.5, &analysis);

        assert_relative_eq!(out_l, 0.0, epsilon = 0.01);
        assert_relative_eq!(out_r, 0.0, epsilon = 0.01);
        // current_mix will start ramping up after reset
        assert!(delay.current_mix < 0.1, "Mix should be near 0 after reset");
    }
}
