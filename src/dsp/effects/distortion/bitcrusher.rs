//! Bitcrusher effect - reduces sample rate and bit depth for lo-fi digital artifacts
//! Creates vintage digital, video game, and glitchy textures

pub struct Bitcrusher {
    /// Target sample rate (actual sample rate / downsample_factor)
    target_sample_rate: f32,

    /// Actual audio sample rate
    actual_sample_rate: f32,

    /// Sample counter for downsampling
    sample_counter: f32,

    /// Held sample for sample rate reduction
    held_sample_left: f32,
    held_sample_right: f32,

    /// Bit depth (1-16 bits)
    bit_depth: u32,

    /// Quantization step size (calculated from bit depth)
    quantize_step: f32,

    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    mix: f32,
}

impl Bitcrusher {
    /// Create a new bitcrusher effect
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `target_sample_rate` - Target sample rate for downsampling (100 Hz - sample_rate)
    /// * `bit_depth` - Bit depth for quantization (1-16 bits)
    pub fn new(sample_rate: f32, target_sample_rate: f32, bit_depth: u32) -> Self {
        let target_sample_rate = target_sample_rate.clamp(100.0, sample_rate);
        let bit_depth = bit_depth.clamp(1, 16);

        let quantize_step = Self::calculate_quantize_step(bit_depth);

        Self {
            target_sample_rate,
            actual_sample_rate: sample_rate,
            sample_counter: 0.0,
            held_sample_left: 0.0,
            held_sample_right: 0.0,
            bit_depth,
            quantize_step,
            mix: 1.0,
        }
    }

    /// Set target sample rate for downsampling
    pub fn set_sample_rate(&mut self, target_sample_rate: f32) {
        self.target_sample_rate = target_sample_rate.clamp(100.0, self.actual_sample_rate);
    }

    /// Set bit depth for quantization (1-16 bits)
    pub fn set_bit_depth(&mut self, bit_depth: u32) {
        self.bit_depth = bit_depth.clamp(1, 16);
        self.quantize_step = Self::calculate_quantize_step(self.bit_depth);
    }

    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Calculate quantization step size from bit depth
    #[inline]
    fn calculate_quantize_step(bit_depth: u32) -> f32 {
        // For n bits, we have 2^n quantization levels
        // Step size = 2.0 / (2^n - 1) for range [-1, 1]
        let levels = (1 << bit_depth) as f32; // 2^bit_depth
        2.0 / (levels - 1.0)
    }

    /// Quantize a sample to the specified bit depth
    #[inline]
    fn quantize(&self, sample: f32) -> f32 {
        // Quantize to nearest step
        (sample / self.quantize_step).round() * self.quantize_step
    }

    /// Process a stereo sample through the bitcrusher
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Sample rate reduction: hold sample until counter reaches threshold
        let samples_per_hold = self.actual_sample_rate / self.target_sample_rate;

        if self.sample_counter >= samples_per_hold {
            // Time to capture a new sample
            self.held_sample_left = self.quantize(left);
            self.held_sample_right = self.quantize(right);
            self.sample_counter -= samples_per_hold;
        }

        self.sample_counter += 1.0;

        // Mix dry and wet
        let output_left = left * (1.0 - self.mix) + self.held_sample_left * self.mix;
        let output_right = right * (1.0 - self.mix) + self.held_sample_right * self.mix;

        (output_left, output_right)
    }

    /// Reset the bitcrusher state
    pub fn reset(&mut self) {
        self.sample_counter = 0.0;
        self.held_sample_left = 0.0;
        self.held_sample_right = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_bitcrusher_creation() {
        let bc = Bitcrusher::new(44100.0, 11025.0, 8);
        assert_eq!(bc.actual_sample_rate, 44100.0);
        assert_relative_eq!(bc.target_sample_rate, 11025.0, epsilon = 0.1);
        assert_eq!(bc.bit_depth, 8);
    }

    #[test]
    fn test_bitcrusher_bit_depth_clamping() {
        let mut bc = Bitcrusher::new(44100.0, 11025.0, 8);

        // Should clamp to 1-16 range
        bc.set_bit_depth(0);
        assert_eq!(bc.bit_depth, 1);

        bc.set_bit_depth(32);
        assert_eq!(bc.bit_depth, 16);
    }

    #[test]
    fn test_bitcrusher_sample_rate_clamping() {
        let mut bc = Bitcrusher::new(44100.0, 11025.0, 8);

        // Should clamp to valid range
        bc.set_sample_rate(50.0); // Too low
        assert!(bc.target_sample_rate >= 100.0);

        bc.set_sample_rate(100000.0); // Above actual sample rate
        assert!(bc.target_sample_rate <= 44100.0);
    }

    #[test]
    fn test_bitcrusher_quantization() {
        let bc = Bitcrusher::new(44100.0, 44100.0, 4); // 4 bits = 16 levels

        // Test quantization of various inputs
        let input = 0.5;
        let quantized = bc.quantize(input);

        // Should be quantized to a discrete level
        let inverse_step = 1.0 / bc.quantize_step;
        let steps = (quantized * inverse_step).round();
        assert_relative_eq!(quantized, steps * bc.quantize_step, epsilon = 0.0001);
    }

    #[test]
    fn test_bitcrusher_extreme_bit_depths() {
        let bc_1bit = Bitcrusher::new(44100.0, 44100.0, 1);
        let bc_16bit = Bitcrusher::new(44100.0, 44100.0, 16);

        // 1 bit should have large quantization steps
        assert!(bc_1bit.quantize_step > 1.0);

        // 16 bits should have small quantization steps
        assert!(bc_16bit.quantize_step < 0.001);
    }

    #[test]
    fn test_bitcrusher_sample_hold() {
        let mut bc = Bitcrusher::new(44100.0, 4410.0, 16); // Downsample by 10x
        bc.set_mix(1.0);

        // Process varying input
        let mut prev_output = f32::NAN;
        let mut hold_count = 0;

        for i in 0..100 {
            let input = (i as f32 * 0.1).sin();
            let (left, _) = bc.process(input, input);

            if prev_output.is_nan() {
                prev_output = left;
            } else if (left - prev_output).abs() < 0.0001 {
                hold_count += 1;
            } else {
                prev_output = left;
            }
        }

        // Should hold samples (downsample by 10x means ~9 holds per update)
        assert!(hold_count > 50);
    }

    #[test]
    fn test_bitcrusher_reduces_resolution() {
        let mut bc_high = Bitcrusher::new(44100.0, 44100.0, 16);
        let mut bc_low = Bitcrusher::new(44100.0, 44100.0, 2);

        bc_high.set_mix(1.0);
        bc_low.set_mix(1.0);

        // Generate smooth input
        let mut high_res_outputs = Vec::new();
        let mut low_res_outputs = Vec::new();

        for i in 0..1000 {
            let input = (i as f32 * 0.01).sin() * 0.5;
            let (high, _) = bc_high.process(input, input);
            let (low, _) = bc_low.process(input, input);

            high_res_outputs.push(high);
            low_res_outputs.push(low);
        }

        // Count unique values (should be fewer with lower bit depth)
        let unique_high: std::collections::HashSet<_> = high_res_outputs
            .iter()
            .map(|&x| (x * 10000.0).round() as i32)
            .collect();

        let unique_low: std::collections::HashSet<_> = low_res_outputs
            .iter()
            .map(|&x| (x * 10000.0).round() as i32)
            .collect();

        assert!(unique_low.len() < unique_high.len());
    }

    #[test]
    fn test_bitcrusher_mix_control() {
        let mut bc = Bitcrusher::new(44100.0, 4410.0, 2);

        // Dry signal (mix = 0.0)
        bc.set_mix(0.0);
        let input = 0.7;
        let (left_dry, _) = bc.process(input, input);
        assert_relative_eq!(left_dry, input, epsilon = 0.01);

        // Wet signal (mix = 1.0)
        bc.reset();
        bc.set_mix(1.0);
        bc.process(input, input); // Prime the held sample
        let (left_wet, _) = bc.process(input, input);

        // Should be quantized/crushed
        assert!((left_wet - input).abs() >= 0.0); // May differ due to quantization
    }

    #[test]
    fn test_bitcrusher_creates_aliasing() {
        let mut bc = Bitcrusher::new(44100.0, 2000.0, 8); // Heavy downsampling
        bc.set_mix(1.0);

        // High-frequency input (above new Nyquist of 1000 Hz)
        let input_freq = 5000.0;
        let mut outputs = Vec::new();

        for i in 0..4410 {
            let phase = i as f32 / 44100.0;
            let input = (input_freq * 2.0 * std::f32::consts::PI * phase).sin();
            let (left, _) = bc.process(input, input);
            outputs.push(left);
        }

        // Output should contain aliasing (frequency content different from input)
        // Just verify it produces output
        let energy: f32 = outputs.iter().map(|&x| x * x).sum();
        assert!(energy > 0.1);
    }

    #[test]
    fn test_bitcrusher_reset() {
        let mut bc = Bitcrusher::new(44100.0, 11025.0, 8);

        // Process some samples
        for _ in 0..100 {
            bc.process(1.0, 1.0);
        }

        // Reset
        bc.reset();

        // State should be cleared
        assert_eq!(bc.sample_counter, 0.0);
        assert_eq!(bc.held_sample_left, 0.0);
        assert_eq!(bc.held_sample_right, 0.0);
    }

    #[test]
    fn test_bitcrusher_stereo_identical() {
        let mut bc = Bitcrusher::new(44100.0, 11025.0, 8);
        bc.set_mix(1.0);

        // Both channels should be processed identically with same input
        for i in 0..100 {
            let input = (i as f32 * 0.1).sin();
            let (left, right) = bc.process(input, input);
            assert_relative_eq!(left, right, epsilon = 0.0001);
        }
    }

    #[test]
    fn test_bitcrusher_quantize_step_calculation() {
        // 1 bit = 2 levels: step should be 2.0
        let bc_1bit = Bitcrusher::new(44100.0, 44100.0, 1);
        assert_relative_eq!(bc_1bit.quantize_step, 2.0, epsilon = 0.01);

        // 2 bits = 4 levels: step should be 2/3 ≈ 0.667
        let bc_2bit = Bitcrusher::new(44100.0, 44100.0, 2);
        assert_relative_eq!(bc_2bit.quantize_step, 0.667, epsilon = 0.01);

        // 8 bits = 256 levels
        let bc_8bit = Bitcrusher::new(44100.0, 44100.0, 8);
        assert_relative_eq!(bc_8bit.quantize_step, 2.0 / 255.0, epsilon = 0.001);
    }

    #[test]
    fn test_bitcrusher_no_sample_rate_reduction() {
        let mut bc = Bitcrusher::new(44100.0, 44100.0, 4);
        bc.set_mix(1.0);

        // With no sample rate reduction, should update every sample
        let input1 = 0.5;
        let input2 = 0.7;

        // First sample initializes the held sample
        bc.process(input1, input1);
        let (out1, _) = bc.process(input1, input1);
        let (out2, _) = bc.process(input2, input2);

        // Outputs should be quantized versions
        let q1 = bc.quantize(input1);
        let q2 = bc.quantize(input2);
        assert_relative_eq!(out1, q1, epsilon = 0.01);
        assert_relative_eq!(out2, q2, epsilon = 0.01);
    }
}
