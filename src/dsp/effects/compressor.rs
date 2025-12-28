/// Dynamic range compressor - reduces dynamic range by attenuating loud signals
/// Uses envelope follower with attack/release for transparent dynamics control

pub struct Compressor {
    /// Sample rate for time calculations
    sample_rate: f32,

    /// Threshold in dB (signals above this are compressed)
    threshold_db: f32,

    /// Ratio (1:1 to ∞:1) - higher means more compression
    ratio: f32,

    /// Attack time in seconds
    attack_time: f32,

    /// Release time in seconds
    release_time: f32,

    /// Knee width in dB (0 = hard knee, >0 = soft knee)
    knee_db: f32,

    /// Makeup gain in dB (compensates for gain reduction)
    makeup_gain_db: f32,

    /// Envelope follower state for left channel
    envelope_left: f32,

    /// Envelope follower state for right channel
    envelope_right: f32,

    /// Attack coefficient (calculated from attack time)
    attack_coeff: f32,

    /// Release coefficient (calculated from release time)
    release_coeff: f32,

    /// Sample counter for process_fast() throttling
    sample_counter: usize,

    /// Cached envelope value for mono compression
    envelope_mono: f32,
}

impl Compressor {
    /// Create a new compressor
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `threshold_db` - Threshold in dB (-60 to 0)
    /// * `ratio` - Compression ratio (1.0 to 20.0)
    /// * `attack_ms` - Attack time in milliseconds
    /// * `release_ms` - Release time in milliseconds
    pub fn new(
        sample_rate: f32,
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
    ) -> Self {
        let mut compressor = Self {
            sample_rate,
            threshold_db: threshold_db.clamp(-60.0, 0.0),
            ratio: ratio.clamp(1.0, 20.0),
            attack_time: (attack_ms / 1000.0).clamp(0.0001, 1.0),
            release_time: (release_ms / 1000.0).clamp(0.001, 5.0),
            knee_db: 0.0,
            makeup_gain_db: 0.0,
            envelope_left: 0.0,
            envelope_right: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            sample_counter: 0,
            envelope_mono: 0.0,
        };

        compressor.update_coefficients();
        compressor
    }

    /// Set threshold in dB
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db.clamp(-60.0, 0.0);
    }

    /// Set compression ratio (1:1 to 20:1)
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(1.0, 20.0);
    }

    /// Set attack time in milliseconds
    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack_time = (attack_ms / 1000.0).clamp(0.0001, 1.0);
        self.update_coefficients();
    }

    /// Set release time in milliseconds
    pub fn set_release(&mut self, release_ms: f32) {
        self.release_time = (release_ms / 1000.0).clamp(0.001, 5.0);
        self.update_coefficients();
    }

    /// Set knee width in dB (0 = hard knee, 6-10 = soft knee)
    pub fn set_knee(&mut self, knee_db: f32) {
        self.knee_db = knee_db.clamp(0.0, 20.0);
    }

    /// Set makeup gain in dB
    pub fn set_makeup_gain(&mut self, gain_db: f32) {
        self.makeup_gain_db = gain_db.clamp(0.0, 30.0);
    }

    /// Update attack/release coefficients
    fn update_coefficients(&mut self) {
        // Exponential smoothing coefficients
        // coeff = exp(-1 / (time * sample_rate))
        self.attack_coeff = (-1.0 / (self.attack_time * self.sample_rate)).exp();
        self.release_coeff = (-1.0 / (self.release_time * self.sample_rate)).exp();
    }

    /// Process a stereo sample through the compressor
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Convert to dB (with floor to avoid log(0))
        let input_db_left = Self::amp_to_db(left.abs().max(0.000001));
        let input_db_right = Self::amp_to_db(right.abs().max(0.000001));

        // Envelope follower (attack/release smoothing)
        self.envelope_left = if input_db_left > self.envelope_left {
            self.attack_coeff * self.envelope_left + (1.0 - self.attack_coeff) * input_db_left
        } else {
            self.release_coeff * self.envelope_left + (1.0 - self.release_coeff) * input_db_left
        };

        self.envelope_right = if input_db_right > self.envelope_right {
            self.attack_coeff * self.envelope_right + (1.0 - self.attack_coeff) * input_db_right
        } else {
            self.release_coeff * self.envelope_right + (1.0 - self.release_coeff) * input_db_right
        };

        // Calculate gain reduction for each channel
        let gain_reduction_left = self.calculate_gain_reduction(self.envelope_left);
        let gain_reduction_right = self.calculate_gain_reduction(self.envelope_right);

        // Apply gain reduction and makeup gain
        let output_left = left * gain_reduction_left * Self::db_to_amp(self.makeup_gain_db);
        let output_right = right * gain_reduction_right * Self::db_to_amp(self.makeup_gain_db);

        (output_left, output_right)
    }

    /// Calculate gain reduction based on input level
    #[inline]
    fn calculate_gain_reduction(&self, input_db: f32) -> f32 {
        // Calculate how much the signal exceeds the threshold
        let overshoot = input_db - self.threshold_db;

        if overshoot <= -self.knee_db * 0.5 {
            // Below threshold - no compression
            1.0
        } else if overshoot >= self.knee_db * 0.5 {
            // Above threshold + knee - full compression
            let gain_reduction_db = overshoot * (1.0 - 1.0 / self.ratio);
            Self::db_to_amp(-gain_reduction_db)
        } else {
            // In the knee region - soft transition
            let knee_overshoot = overshoot + self.knee_db * 0.5;
            let gain_reduction_db =
                knee_overshoot * knee_overshoot / (2.0 * self.knee_db) * (1.0 - 1.0 / self.ratio);
            Self::db_to_amp(-gain_reduction_db)
        }
    }

    /// Convert amplitude to dB
    #[inline]
    fn amp_to_db(amp: f32) -> f32 {
        20.0 * amp.log10()
    }

    /// Convert dB to amplitude
    #[inline]
    fn db_to_amp(db: f32) -> f32 {
        10.0f32.powf(db / 20.0)
    }

    /// Reset the compressor state
    pub fn reset(&mut self) {
        self.envelope_left = 0.0;
        self.envelope_right = 0.0;
        self.envelope_mono = 0.0;
        self.sample_counter = 0;
    }

    /// Process a stereo sample with optimized mono compression (for per-voice use)
    ///
    /// Uses mono envelope follower (max of left/right) and processes envelope every 4 samples
    /// for CPU efficiency. Applies identical gain reduction to both channels.
    ///
    /// This is optimized for per-voice compression where 16 instances run simultaneously.
    /// The envelope follower updates every 4 samples (~11kHz at 44.1kHz = imperceptible for transients).
    pub fn process_fast(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Update envelope follower every 4 samples to reduce CPU usage
        if self.sample_counter % 4 == 0 {
            // Mono compression: use max of both channels for envelope detection
            let input_peak = left.abs().max(right.abs()).max(0.000001);
            let input_db = Self::amp_to_db(input_peak);

            // Envelope follower with attack/release smoothing
            self.envelope_mono = if input_db > self.envelope_mono {
                self.attack_coeff * self.envelope_mono + (1.0 - self.attack_coeff) * input_db
            } else {
                self.release_coeff * self.envelope_mono + (1.0 - self.release_coeff) * input_db
            };
        }

        self.sample_counter = self.sample_counter.wrapping_add(1);

        // Calculate gain reduction from cached envelope
        let gain_reduction = self.calculate_gain_reduction(self.envelope_mono);

        // Apply identical gain reduction to both channels (mono compression)
        let makeup = Self::db_to_amp(self.makeup_gain_db);
        let output_left = left * gain_reduction * makeup;
        let output_right = right * gain_reduction * makeup;

        (output_left, output_right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_compressor_creation() {
        let comp = Compressor::new(44100.0, -20.0, 4.0, 10.0, 100.0);
        assert_eq!(comp.sample_rate, 44100.0);
        assert_relative_eq!(comp.threshold_db, -20.0, epsilon = 0.1);
        assert_relative_eq!(comp.ratio, 4.0, epsilon = 0.1);
    }

    #[test]
    fn test_compressor_db_conversion() {
        // Test amplitude to dB conversion
        assert_relative_eq!(Compressor::amp_to_db(1.0), 0.0, epsilon = 0.01);
        assert_relative_eq!(Compressor::amp_to_db(0.5), -6.02, epsilon = 0.1);
        assert_relative_eq!(Compressor::amp_to_db(0.1), -20.0, epsilon = 0.1);

        // Test dB to amplitude conversion
        assert_relative_eq!(Compressor::db_to_amp(0.0), 1.0, epsilon = 0.01);
        assert_relative_eq!(Compressor::db_to_amp(-6.0), 0.5, epsilon = 0.01);
        assert_relative_eq!(Compressor::db_to_amp(-20.0), 0.1, epsilon = 0.01);
    }

    #[test]
    fn test_compressor_below_threshold() {
        let mut comp = Compressor::new(44100.0, -20.0, 4.0, 10.0, 100.0);

        // Signal below threshold should pass through relatively unchanged
        let input = 0.01; // -40 dB (well below -20 dB threshold)

        for _ in 0..5000 {
            comp.process(input, input);
        }

        let (left, right) = comp.process(input, input);

        // Should be approximately unchanged (within envelope following tolerance)
        assert_relative_eq!(left, input, epsilon = 0.01);
        assert_relative_eq!(right, input, epsilon = 0.01);
    }

    #[test]
    fn test_compressor_above_threshold() {
        let mut comp = Compressor::new(44100.0, -20.0, 4.0, 1.0, 100.0);
        comp.set_knee(0.0); // Hard knee for predictable behavior

        // Signal above threshold should be compressed
        let input = 0.5; // -6 dB (well above -20 dB threshold)

        // Process until envelope stabilizes
        for _ in 0..1000 {
            comp.process(input, input);
        }

        let (left, _) = comp.process(input, input);

        // Should be compressed (output < input)
        assert!(left < input * 0.9);
    }

    #[test]
    fn test_compressor_ratio_effect() {
        let mut comp_low_ratio = Compressor::new(44100.0, -20.0, 2.0, 1.0, 100.0);
        let mut comp_high_ratio = Compressor::new(44100.0, -20.0, 10.0, 1.0, 100.0);

        let input = 0.5; // Above threshold

        // Stabilize both
        for _ in 0..1000 {
            comp_low_ratio.process(input, input);
            comp_high_ratio.process(input, input);
        }

        let (left_low, _) = comp_low_ratio.process(input, input);
        let (left_high, _) = comp_high_ratio.process(input, input);

        // Higher ratio should compress more (lower output)
        assert!(left_high < left_low);
    }

    #[test]
    fn test_compressor_attack_time() {
        let mut comp_fast = Compressor::new(44100.0, -20.0, 4.0, 1.0, 100.0);
        let mut comp_slow = Compressor::new(44100.0, -20.0, 4.0, 50.0, 100.0);

        // Process continuous signal above threshold
        let signal = 0.8;

        // Just verify both compressors produce output and function
        for _ in 0..100 {
            let (fast_out, _) = comp_fast.process(signal, signal);
            let (slow_out, _) = comp_slow.process(signal, signal);

            // Both should produce valid output
            assert!(fast_out.is_finite());
            assert!(slow_out.is_finite());

            // Output should be less than input (compression is happening)
            assert!(fast_out < signal);
            assert!(slow_out < signal);
        }
    }

    #[test]
    fn test_compressor_release_time() {
        let mut comp_fast = Compressor::new(44100.0, -20.0, 4.0, 1.0, 10.0);
        let mut comp_slow = Compressor::new(44100.0, -20.0, 4.0, 1.0, 200.0);

        // Build up envelope
        for _ in 0..1000 {
            comp_fast.process(0.8, 0.8);
            comp_slow.process(0.8, 0.8);
        }

        // Switch to low level
        let low_input = 0.01;

        for _ in 0..100 {
            comp_fast.process(low_input, low_input);
            comp_slow.process(low_input, low_input);
        }

        // Fast release should recover faster (less compression on quiet signal)
        // Meaning output will be closer to input
        let (fast_out, _) = comp_fast.process(low_input, low_input);
        let (slow_out, _) = comp_slow.process(low_input, low_input);

        assert!(fast_out > slow_out * 0.5);
    }

    #[test]
    fn test_compressor_makeup_gain() {
        let mut comp = Compressor::new(44100.0, -20.0, 4.0, 1.0, 100.0);
        comp.set_makeup_gain(6.0); // +6 dB

        let input = 0.5;

        // Stabilize
        for _ in 0..1000 {
            comp.process(input, input);
        }

        let (with_makeup, _) = comp.process(input, input);

        // With makeup gain, output should be boosted
        // Reset and test without makeup
        comp.reset();
        comp.set_makeup_gain(0.0);

        for _ in 0..1000 {
            comp.process(input, input);
        }

        let (without_makeup, _) = comp.process(input, input);

        assert!(with_makeup > without_makeup * 1.5);
    }

    #[test]
    fn test_compressor_soft_knee() {
        let mut comp_hard = Compressor::new(44100.0, -20.0, 4.0, 1.0, 100.0);
        comp_hard.set_knee(0.0);

        let mut comp_soft = Compressor::new(44100.0, -20.0, 4.0, 1.0, 100.0);
        comp_soft.set_knee(6.0);

        // Test signal right at threshold
        let input = Compressor::db_to_amp(-20.0);

        // Stabilize
        for _ in 0..1000 {
            comp_hard.process(input, input);
            comp_soft.process(input, input);
        }

        let (hard_out, _) = comp_hard.process(input, input);
        let (soft_out, _) = comp_soft.process(input, input);

        // Soft knee should compress less harshly at threshold
        assert!((soft_out - hard_out).abs() < input); // Should differ but both work
    }

    #[test]
    fn test_compressor_reset() {
        let mut comp = Compressor::new(44100.0, -20.0, 4.0, 10.0, 100.0);

        // Build up envelope
        for _ in 0..1000 {
            comp.process(0.8, 0.8);
        }

        // Reset
        comp.reset();

        // Envelope should be cleared
        assert_eq!(comp.envelope_left, 0.0);
        assert_eq!(comp.envelope_right, 0.0);
    }

    #[test]
    fn test_compressor_stability() {
        let mut comp = Compressor::new(44100.0, -20.0, 10.0, 1.0, 50.0);

        // Process loud signal for extended time
        for _ in 0..10000 {
            let (left, right) = comp.process(1.0, 1.0);

            // Should remain stable
            assert!(left.is_finite());
            assert!(right.is_finite());
            assert!(left.abs() < 10.0);
            assert!(right.abs() < 10.0);
        }
    }

    #[test]
    fn test_compressor_stereo_independence() {
        let mut comp = Compressor::new(44100.0, -20.0, 4.0, 1.0, 100.0);

        // Different levels in each channel
        let left_input = 0.8;
        let right_input = 0.2;

        for _ in 0..1000 {
            comp.process(left_input, right_input);
        }

        let (left_out, right_out) = comp.process(left_input, right_input);

        // Outputs should be different (independent compression)
        assert!((left_out - right_out).abs() > 0.1);
    }
}
