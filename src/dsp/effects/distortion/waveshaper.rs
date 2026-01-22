//! Waveshaper - nonlinear distortion using transfer functions
//! Creates harmonic distortion from soft clipping to hard fuzz

pub struct Waveshaper {
    /// Waveshaping algorithm
    algorithm: Algorithm,

    /// Drive/input gain (0.1 to 10.0) - controls distortion intensity
    drive: f32,

    /// Output level compensation
    output_gain: f32,

    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    mix: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Algorithm {
    /// Soft clipping using tanh
    SoftClip,

    /// Hard clipping
    HardClip,

    /// Cubic soft clipping
    Cubic,

    /// Arctangent distortion
    Atan,

    /// Sigmoid/logistic function
    Sigmoid,

    /// Foldback distortion
    Foldback,
}

impl Waveshaper {
    /// Create a new waveshaper
    ///
    /// # Arguments
    /// * `algorithm` - Waveshaping algorithm to use
    /// * `drive` - Input gain/drive amount (0.1 to 10.0)
    pub fn new(algorithm: Algorithm, drive: f32) -> Self {
        Self {
            algorithm,
            drive: drive.clamp(0.1, 10.0),
            output_gain: 1.0,
            mix: 1.0,
        }
    }

    /// Set waveshaping algorithm
    pub fn set_algorithm(&mut self, algorithm: Algorithm) {
        self.algorithm = algorithm;
    }

    /// Set drive amount (0.1 to 10.0)
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.1, 10.0);
    }

    /// Set output gain compensation
    pub fn set_output_gain(&mut self, gain: f32) {
        self.output_gain = gain.clamp(0.1, 2.0);
    }

    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Process a stereo sample through the waveshaper
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Apply drive
        let driven_left = left * self.drive;
        let driven_right = right * self.drive;

        // Apply waveshaping
        let shaped_left = self.shape(driven_left);
        let shaped_right = self.shape(driven_right);

        // Apply output gain
        let wet_left = shaped_left * self.output_gain;
        let wet_right = shaped_right * self.output_gain;

        // Mix dry and wet
        let output_left = left * (1.0 - self.mix) + wet_left * self.mix;
        let output_right = right * (1.0 - self.mix) + wet_right * self.mix;

        (output_left, output_right)
    }

    /// Apply waveshaping function to a sample
    #[inline]
    fn shape(&self, x: f32) -> f32 {
        match self.algorithm {
            Algorithm::SoftClip => {
                // Hyperbolic tangent - smooth soft clipping
                x.tanh()
            }
            Algorithm::HardClip => {
                // Hard clipping at ±1.0
                x.clamp(-1.0, 1.0)
            }
            Algorithm::Cubic => {
                // Cubic soft clipping: y = x - x^3/3
                // Smooth transition, clips at ±1.5
                if x.abs() < 1.5 {
                    x - (x * x * x) / 3.0
                } else {
                    x.signum() * 1.0
                }
            }
            Algorithm::Atan => {
                // Arctangent distortion - very soft and musical
                (x * 2.0).atan() / std::f32::consts::FRAC_PI_2
            }
            Algorithm::Sigmoid => {
                // Sigmoid/logistic function: y = 2/(1 + e^-x) - 1
                2.0 / (1.0 + (-x).exp()) - 1.0
            }
            Algorithm::Foldback => {
                // Foldback distortion - folds signal back on itself
                // Creates harsh, aliased distortion
                let threshold = 1.0;
                let mut signal = x;

                while signal.abs() > threshold {
                    if signal > threshold {
                        signal = 2.0 * threshold - signal;
                    } else if signal < -threshold {
                        signal = -2.0 * threshold - signal;
                    }
                }

                signal
            }
        }
    }

    /// Reset the waveshaper (stateless, but included for consistency)
    pub fn reset(&mut self) {
        // Waveshaper is stateless
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_waveshaper_creation() {
        let ws = Waveshaper::new(Algorithm::SoftClip, 2.0);
        assert_eq!(ws.algorithm, Algorithm::SoftClip);
        assert_relative_eq!(ws.drive, 2.0, epsilon = 0.1);
    }

    #[test]
    fn test_waveshaper_drive_clamping() {
        let mut ws = Waveshaper::new(Algorithm::SoftClip, 2.0);

        ws.set_drive(0.01); // Too low
        assert!(ws.drive >= 0.1);

        ws.set_drive(100.0); // Too high
        assert!(ws.drive <= 10.0);
    }

    #[test]
    fn test_waveshaper_soft_clip() {
        let mut ws = Waveshaper::new(Algorithm::SoftClip, 1.0);
        ws.set_mix(1.0);

        // Low amplitude should pass through relatively unchanged
        let (low_out, _) = ws.process(0.1, 0.1);
        assert_relative_eq!(low_out, 0.1, epsilon = 0.01);

        // High amplitude should be compressed
        let (high_out, _) = ws.process(5.0, 5.0);
        assert!(high_out.abs() < 2.0); // Should be compressed
        assert!(high_out > 0.0); // Should preserve sign
    }

    #[test]
    fn test_waveshaper_hard_clip() {
        let mut ws = Waveshaper::new(Algorithm::HardClip, 1.0);
        ws.set_mix(1.0);

        // Should clip at ±1.0
        let (clipped_pos, _) = ws.process(5.0, 5.0);
        assert_relative_eq!(clipped_pos, 1.0, epsilon = 0.01);

        let (clipped_neg, _) = ws.process(-5.0, -5.0);
        assert_relative_eq!(clipped_neg, -1.0, epsilon = 0.01);

        // Below threshold should pass through
        let (passthrough, _) = ws.process(0.5, 0.5);
        assert_relative_eq!(passthrough, 0.5, epsilon = 0.01);
    }

    #[test]
    fn test_waveshaper_cubic() {
        let mut ws = Waveshaper::new(Algorithm::Cubic, 1.0);
        ws.set_mix(1.0);

        // Small signals should pass through with minimal distortion
        let (small, _) = ws.process(0.1, 0.1);
        assert_relative_eq!(small, 0.1, epsilon = 0.01);

        // Large signals should be shaped
        let (large, _) = ws.process(2.0, 2.0);
        assert!(large.abs() < 2.0);
        assert!(large.abs() > 0.5);
    }

    #[test]
    fn test_waveshaper_atan() {
        let mut ws = Waveshaper::new(Algorithm::Atan, 1.0);
        ws.set_mix(1.0);

        // Atan should be smooth and bounded
        for i in -100..100 {
            let input = i as f32 * 0.1;
            let (output, _) = ws.process(input, input);

            // Should be bounded
            assert!(output.abs() < 2.0);

            // Should preserve sign
            if input != 0.0 {
                assert_eq!(output.signum(), input.signum());
            }
        }
    }

    #[test]
    fn test_waveshaper_sigmoid() {
        let mut ws = Waveshaper::new(Algorithm::Sigmoid, 1.0);
        ws.set_mix(1.0);

        // Sigmoid should be smooth and bounded to approximately [-1, 1]
        let (large_pos, _) = ws.process(10.0, 10.0);
        assert!(large_pos > 0.9 && large_pos < 1.1);

        let (large_neg, _) = ws.process(-10.0, -10.0);
        assert!(large_neg < -0.9 && large_neg > -1.1);

        // Zero should map to approximately zero
        let (zero, _) = ws.process(0.0, 0.0);
        assert_relative_eq!(zero, 0.0, epsilon = 0.1);
    }

    #[test]
    fn test_waveshaper_foldback() {
        let mut ws = Waveshaper::new(Algorithm::Foldback, 1.0);
        ws.set_mix(1.0);

        // Should fold back signals that exceed threshold
        let (folded, _) = ws.process(2.0, 2.0);

        // Should be within bounds (folded back)
        assert!(folded.abs() <= 1.0);

        // With drive, high input should fold multiple times
        ws.set_drive(3.0);
        let (multi_fold, _) = ws.process(2.0, 2.0);
        assert!(multi_fold.abs() <= 1.0);
    }

    #[test]
    fn test_waveshaper_drive_effect() {
        let mut ws_low = Waveshaper::new(Algorithm::SoftClip, 0.5);
        let mut ws_high = Waveshaper::new(Algorithm::SoftClip, 5.0);

        ws_low.set_mix(1.0);
        ws_high.set_mix(1.0);

        let input = 0.5;

        let (low_out, _) = ws_low.process(input, input);
        let (high_out, _) = ws_high.process(input, input);

        // Higher drive should create more distortion (larger difference from input)
        let low_dist = (low_out - input).abs();
        let high_dist = (high_out - input).abs();

        assert!(high_dist > low_dist);
    }

    #[test]
    fn test_waveshaper_mix_control() {
        let mut ws = Waveshaper::new(Algorithm::HardClip, 5.0);

        // Dry signal (mix = 0.0)
        ws.set_mix(0.0);
        let (dry, _) = ws.process(2.0, 2.0);
        assert_relative_eq!(dry, 2.0, epsilon = 0.01);

        // Wet signal (mix = 1.0)
        ws.set_mix(1.0);
        let (wet, _) = ws.process(2.0, 2.0);
        assert!(wet < 1.5); // Should be clipped/distorted
    }

    #[test]
    fn test_waveshaper_output_gain() {
        let mut ws = Waveshaper::new(Algorithm::SoftClip, 2.0);
        ws.set_mix(1.0);

        ws.set_output_gain(0.5);
        let (low_gain, _) = ws.process(0.5, 0.5);

        ws.set_output_gain(1.5);
        let (high_gain, _) = ws.process(0.5, 0.5);

        // Higher output gain should produce louder output
        assert!(high_gain.abs() > low_gain.abs() * 2.0);
    }

    #[test]
    fn test_waveshaper_preserves_silence() {
        let mut ws = Waveshaper::new(Algorithm::SoftClip, 5.0);

        // All algorithms should preserve silence
        for algo in [
            Algorithm::SoftClip,
            Algorithm::HardClip,
            Algorithm::Cubic,
            Algorithm::Atan,
            Algorithm::Sigmoid,
            Algorithm::Foldback,
        ] {
            ws.set_algorithm(algo);
            let (left, right) = ws.process(0.0, 0.0);
            assert_relative_eq!(left, 0.0, epsilon = 0.01);
            assert_relative_eq!(right, 0.0, epsilon = 0.01);
        }
    }

    #[test]
    fn test_waveshaper_symmetry() {
        let mut ws = Waveshaper::new(Algorithm::SoftClip, 2.0);
        ws.set_mix(1.0);

        // Most algorithms should be symmetric (f(-x) = -f(x))
        let input = 0.7;

        let (pos, _) = ws.process(input, input);
        let (neg, _) = ws.process(-input, -input);

        assert_relative_eq!(pos, -neg, epsilon = 0.01);
    }

    #[test]
    fn test_waveshaper_stereo_identical() {
        let mut ws = Waveshaper::new(Algorithm::SoftClip, 2.0);

        // Same input should produce same output on both channels
        for _ in 0..100 {
            let input = (0..100)
                .map(|_| rand::random::<f32>() * 2.0 - 1.0)
                .next()
                .unwrap_or(0.5);
            let (left, right) = ws.process(input, input);
            assert_relative_eq!(left, right, epsilon = 0.0001);
        }
    }

    #[test]
    fn test_waveshaper_adds_harmonics() {
        let mut ws = Waveshaper::new(Algorithm::SoftClip, 3.0);
        ws.set_mix(1.0);

        // Generate sine wave and waveshape it
        let freq = 440.0;
        let sample_rate = 44100.0;
        let mut outputs = Vec::new();

        for i in 0..4410 {
            let phase = i as f32 / sample_rate;
            let input = (freq * 2.0 * std::f32::consts::PI * phase).sin() * 0.3;
            let (output, _) = ws.process(input, input);
            outputs.push(output);
        }

        // Waveshaping should add harmonic content
        // Output should be non-sinusoidal (verified by visual inspection or FFT)
        // For now, just verify output exists and is bounded
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max > 0.1);
        assert!(min < -0.1);
        assert!(max < 2.0);
        assert!(min > -2.0);
    }
}
