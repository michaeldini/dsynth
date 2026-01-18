use std::f32::consts::PI;

/// Kaiser window windowed-sinc FIR downsampler for 4:1 decimation
/// Uses ~20 taps with Kaiser window (β=8.5) for high stopband attenuation
pub struct Downsampler {
    taps: usize,
    coefficients: Vec<f32>,
    buffer: Vec<f32>,
    buffer_index: usize,
}

impl Downsampler {
    /// Create a new downsampler for 4:1 decimation
    ///
    /// # Arguments
    /// * `taps` - Number of filter taps (should be multiple of 4, recommended ~20)
    pub fn new(taps: usize) -> Self {
        assert!(
            taps >= 4 && taps.is_multiple_of(4),
            "Taps must be >= 4 and multiple of 4"
        );

        let coefficients = Self::calculate_kaiser_sinc_coefficients(taps, 8.5);

        Self {
            taps,
            coefficients,
            buffer: vec![0.0; taps],
            buffer_index: 0,
        }
    }

    /// Calculate windowed-sinc filter coefficients with Kaiser window
    fn calculate_kaiser_sinc_coefficients(taps: usize, beta: f32) -> Vec<f32> {
        let mut coeffs = Vec::with_capacity(taps);
        let cutoff = 0.25; // Cutoff at 1/4 of input sample rate (for 4:1 decimation)
        let center = (taps - 1) as f32 / 2.0;

        // Calculate Kaiser window and sinc coefficients
        for i in 0..taps {
            let x = i as f32 - center;

            // Sinc function
            let sinc = if x.abs() < 1e-6 {
                2.0 * PI * cutoff
            } else {
                (2.0 * PI * cutoff * x).sin() / x
            };

            // Kaiser window
            let alpha = (i as f32 - center) / center;
            let window = Self::kaiser_window(alpha, beta);

            coeffs.push(sinc * window);
        }

        // Normalize coefficients
        let sum: f32 = coeffs.iter().sum();
        for coeff in coeffs.iter_mut() {
            *coeff /= sum;
        }

        coeffs
    }

    /// Modified Bessel function of the first kind (I0)
    /// Used for Kaiser window calculation
    fn bessel_i0(x: f32) -> f32 {
        let mut sum = 1.0;
        let mut term = 1.0;
        let x_squared = x * x / 4.0;

        for i in 1..20 {
            term *= x_squared / (i as f32 * i as f32);
            sum += term;
            if term < 1e-9 {
                break;
            }
        }

        sum
    }

    /// Kaiser window function
    fn kaiser_window(alpha: f32, beta: f32) -> f32 {
        let arg = beta * (1.0 - alpha * alpha).sqrt();
        Self::bessel_i0(arg) / Self::bessel_i0(beta)
    }

    /// Process 4 input samples and produce 1 output sample
    ///
    /// # Arguments
    /// * `samples` - Array of 4 samples at 4× sample rate
    ///
    /// # Returns
    /// Single downsampled output sample
    pub fn process(&mut self, samples: [f32; 4]) -> f32 {
        // Insert 4 samples into ring buffer
        for &sample in &samples {
            self.buffer[self.buffer_index] = sample;
            self.buffer_index = (self.buffer_index + 1) % self.taps;
        }

        // Convolve with FIR coefficients
        let mut output = 0.0;
        let mut buf_idx = self.buffer_index;

        for &coeff in &self.coefficients {
            buf_idx = if buf_idx == 0 {
                self.taps - 1
            } else {
                buf_idx - 1
            };
            output += self.buffer[buf_idx] * coeff;
        }

        output
    }

    /// Reset the downsampler state
    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.buffer_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_downsampler_creation() {
        let ds = Downsampler::new(20);
        assert_eq!(ds.taps, 20);
        assert_eq!(ds.coefficients.len(), 20);
        assert_eq!(ds.buffer.len(), 20);
    }

    #[test]
    fn test_kaiser_window_properties() {
        // Kaiser window should be 1.0 at center and decrease towards edges
        let beta = 8.5;
        let center_value = Downsampler::kaiser_window(0.0, beta);
        let edge_value = Downsampler::kaiser_window(1.0, beta);

        assert_relative_eq!(center_value, 1.0, epsilon = 0.001);
        assert!(edge_value < center_value);
        assert!(edge_value > 0.0);
    }

    #[test]
    fn test_coefficient_normalization() {
        let ds = Downsampler::new(20);
        let sum: f32 = ds.coefficients.iter().sum();
        assert_relative_eq!(sum, 1.0, epsilon = 0.001);
    }

    #[test]
    fn test_dc_signal_preservation() {
        let mut ds = Downsampler::new(20);

        // Feed DC signal (constant value)
        let dc_value = 0.5;
        let mut output_sum = 0.0;
        let iterations = 50;

        for _ in 0..iterations {
            let output = ds.process([dc_value; 4]);
            output_sum += output;
        }

        let average_output = output_sum / iterations as f32;

        // After settling, DC output should match DC input
        assert_relative_eq!(average_output, dc_value, epsilon = 0.05);
    }

    #[test]
    fn test_passband_signal() {
        let mut ds = Downsampler::new(20);

        // Test with low-frequency sine wave (well within passband)
        // At 4× sample rate with freq = 0.05 (normalized), after downsampling freq = 0.2
        let sample_rate_4x = 176400.0; // 44100 * 4
        let freq = 1000.0; // 1kHz signal
        let normalized_freq = freq / sample_rate_4x;

        let mut max_output: f32 = 0.0;

        for i in 0..200 {
            let phase_base = 2.0 * PI * normalized_freq * (i * 4) as f32;
            let samples = [
                (phase_base).sin(),
                (phase_base + 2.0 * PI * normalized_freq).sin(),
                (phase_base + 2.0 * PI * normalized_freq * 2.0).sin(),
                (phase_base + 2.0 * PI * normalized_freq * 3.0).sin(),
            ];

            let output = ds.process(samples);
            max_output = max_output.max(output.abs());
        }

        // Passband signal should pass through with minimal attenuation
        assert!(max_output > 0.8, "Max output was {}", max_output);
    }

    #[test]
    fn test_stopband_rejection() {
        let mut ds = Downsampler::new(20);

        // Test with high-frequency signal that should be rejected
        // Nyquist after downsampling is 0.5, so test with freq near original Nyquist (0.5 * 4 = 2.0)
        let normalized_freq = 0.45; // Close to Nyquist of 4× rate

        let mut max_output: f32 = 0.0;

        for i in 0..200 {
            let phase_base = 2.0 * PI * normalized_freq * (i * 4) as f32;
            let samples = [
                (phase_base).sin(),
                (phase_base + 2.0 * PI * normalized_freq).sin(),
                (phase_base + 2.0 * PI * normalized_freq * 2.0).sin(),
                (phase_base + 2.0 * PI * normalized_freq * 3.0).sin(),
            ];

            let output = ds.process(samples);
            max_output = max_output.max(output.abs());
        }

        // High-frequency signal should be heavily attenuated
        assert!(
            max_output < 0.1,
            "Max output was {} (should be < 0.1)",
            max_output
        );
    }

    #[test]
    fn test_reset() {
        let mut ds = Downsampler::new(20);

        // Process some samples
        ds.process([1.0, 0.5, 0.25, 0.125]);

        // Reset
        ds.reset();

        // Buffer should be cleared
        assert!(ds.buffer.iter().all(|&x| x == 0.0));
        assert_eq!(ds.buffer_index, 0);
    }
}
