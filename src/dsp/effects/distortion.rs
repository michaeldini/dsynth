/// Waveshaper distortion effect
///
/// Distortion adds harmonics and saturation to the signal, making it sound "warmer"
/// or "grittier" depending on the amount. This is essential for lead synths, basses,
/// and adding character to otherwise clean sounds.
///
/// # Architecture
/// - Multiple waveshaping algorithms (tanh, soft clip, hard clip, cubic)
/// - Pre-gain (drive) stage boosts signal before distortion
/// - Post-gain stage compensates for level increase
/// - DC blocking filter prevents DC offset from asymmetric distortion
///
/// # Parameters
/// - **drive**: Input gain before distortion (0.0 to 1.0 maps to 1x to 20x)
/// - **mix**: Wet/dry balance (0.0 = dry, 1.0 = full wet)
/// - **algorithm**: Choice of distortion curve (tanh, soft clip, hard clip, cubic)
///
/// # Distortion Types
/// - **Tanh**: Smooth, tube-like saturation (most musical)
/// - **Soft Clip**: Gentle compression then hard limit
/// - **Hard Clip**: Brick-wall limiting (harsh, digital)
/// - **Cubic**: Subtle harmonic enhancement (adds 3rd harmonic)

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistortionType {
    Tanh,
    SoftClip,
    HardClip,
    Cubic,
}

/// Distortion/waveshaper processor
pub struct Distortion {
    // DC blocking filter (high-pass at ~10Hz)
    dc_block_x1: f32,
    dc_block_y1: f32,
    dc_block_coeff: f32,

    // Parameters
    drive: f32,
    mix: f32,
    dist_type: DistortionType,
}

impl Distortion {
    /// Create a new distortion processor
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (for DC blocking filter)
    pub fn new(sample_rate: f32) -> Self {
        // DC blocking filter coefficient (high-pass at ~10Hz)
        let cutoff = 10.0;
        let rc = 1.0 / (2.0 * PI * cutoff);
        let dt = 1.0 / sample_rate;
        let dc_block_coeff = rc / (rc + dt);

        Self {
            dc_block_x1: 0.0,
            dc_block_y1: 0.0,
            dc_block_coeff,
            drive: 0.0,
            mix: 0.5,
            dist_type: DistortionType::Tanh,
        }
    }

    /// Set drive amount (0.0 to 1.0)
    /// Maps to 1x to 20x gain internally
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.0, 1.0);
    }

    /// Set wet/dry mix (0.0 = dry, 1.0 = full wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Set distortion type
    pub fn set_type(&mut self, dist_type: DistortionType) {
        self.dist_type = dist_type;
    }

    /// Apply waveshaping based on selected algorithm
    fn apply_distortion(&self, input: f32, gain: f32) -> f32 {
        let x = input * gain;

        match self.dist_type {
            DistortionType::Tanh => {
                // Hyperbolic tangent - smooth, tube-like saturation
                x.tanh()
            }
            DistortionType::SoftClip => {
                // Soft clip with smooth transition
                if x > 1.0 {
                    2.0 / 3.0
                } else if x < -1.0 {
                    -2.0 / 3.0
                } else {
                    x - (x * x * x) / 3.0
                }
            }
            DistortionType::HardClip => {
                // Hard brick-wall clipping
                x.clamp(-1.0, 1.0)
            }
            DistortionType::Cubic => {
                // Cubic waveshaper (adds 3rd harmonic)
                if x.abs() < 1.0 {
                    x - 0.25 * x * x * x
                } else {
                    x.signum() * 0.75
                }
            }
        }
    }

    /// DC blocking filter (high-pass)
    fn dc_block(&mut self, input: f32) -> f32 {
        let output = self.dc_block_coeff * (self.dc_block_y1 + input - self.dc_block_x1);
        self.dc_block_x1 = input;
        self.dc_block_y1 = output;
        output
    }

    /// Process a single sample
    ///
    /// # Arguments
    /// * `input` - Input sample
    ///
    /// # Returns
    /// Distorted output sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Map drive (0.0 to 1.0) to gain (1.0 to 20.0)
        let gain = 1.0 + self.drive * 19.0;

        // Apply distortion
        let distorted = self.apply_distortion(input, gain);

        // Compensate for gain increase (rough approximation)
        let compensated = distorted / (1.0 + self.drive * 0.5);

        // DC blocking (prevents DC offset from asymmetric distortion)
        let blocked = self.dc_block(compensated);

        // Mix wet and dry
        input * (1.0 - self.mix) + blocked * self.mix
    }

    /// Process a stereo sample pair
    ///
    /// # Arguments
    /// * `input_l` - Left channel input
    /// * `input_r` - Right channel input
    ///
    /// # Returns
    /// Tuple of (left_output, right_output)
    pub fn process_stereo(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        let out_l = self.process(input_l);
        let out_r = self.process(input_r);
        (out_l, out_r)
    }

    /// Clear filter state
    pub fn clear(&mut self) {
        self.dc_block_x1 = 0.0;
        self.dc_block_y1 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_distortion_creation() {
        let dist = Distortion::new(44100.0);
        assert_eq!(dist.drive, 0.0);
        assert_eq!(dist.mix, 0.5);
    }

    #[test]
    fn test_distortion_parameters() {
        let mut dist = Distortion::new(44100.0);

        dist.set_drive(0.7);
        assert_eq!(dist.drive, 0.7);

        dist.set_mix(0.8);
        assert_eq!(dist.mix, 0.8);

        dist.set_type(DistortionType::HardClip);
        assert_eq!(dist.dist_type, DistortionType::HardClip);
    }

    #[test]
    fn test_distortion_parameter_clamping() {
        let mut dist = Distortion::new(44100.0);

        dist.set_drive(1.5);
        assert_eq!(dist.drive, 1.0);

        dist.set_drive(-0.5);
        assert_eq!(dist.drive, 0.0);

        dist.set_mix(2.0);
        assert_eq!(dist.mix, 1.0);
    }

    #[test]
    fn test_distortion_dry_passthrough() {
        let mut dist = Distortion::new(44100.0);
        dist.set_mix(0.0); // Fully dry

        let output = dist.process(0.5);

        // With mix=0.0, output should equal input
        assert_relative_eq!(output, 0.5, epsilon = 0.001);
    }

    #[test]
    fn test_distortion_adds_harmonics() {
        let mut dist = Distortion::new(44100.0);
        dist.set_drive(0.8);
        dist.set_mix(1.0); // Full wet

        // Low amplitude signal should pass relatively clean
        let low_out = dist.process(0.1);
        assert!(low_out.abs() < 1.0, "Low amplitude output should be reasonable: {}", low_out);

        // High amplitude signal should be compressed/clipped
        dist.clear();
        let high_out = dist.process(2.0);
        assert!(high_out.abs() < 1.5, "Distortion should compress/limit signal");
    }

    #[test]
    fn test_distortion_types() {
        let sample_rate = 44100.0;

        // Test that each type produces different results using raw distortion curves
        let mut dist_tanh = Distortion::new(sample_rate);
        dist_tanh.set_type(DistortionType::Tanh);

        let mut dist_hard = Distortion::new(sample_rate);
        dist_hard.set_type(DistortionType::HardClip);

        // Use apply_distortion directly to avoid DC blocker differences
        let gain = 10.0; // High gain to show clear difference
        let input = 0.5;
        
        let out_tanh = dist_tanh.apply_distortion(input, gain);
        let out_hard = dist_hard.apply_distortion(input, gain);

        // Different algorithms should produce different results (even tiny differences count)
        assert!((out_tanh - out_hard).abs() > 0.00001, 
            "Different distortion types should produce different outputs: tanh={}, hard={}, diff={}", 
            out_tanh, out_hard, (out_tanh - out_hard).abs());
    }

    #[test]
    fn test_distortion_tanh() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::Tanh);
        dist.set_drive(1.0); // Maximum drive
        dist.set_mix(1.0);

        // Large input should be bounded by tanh
        let output = dist.process(10.0);
        assert!(output.abs() < 2.0, "Tanh should limit output");
    }

    #[test]
    fn test_distortion_hard_clip() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::HardClip);
        dist.set_drive(0.1); // Low drive so input doesn't get multiplied too much
        dist.set_mix(1.0);

        // Input above 1.0 should be clipped
        let output = dist.process(0.6);
        assert!(output.abs() <= 1.0, "Hard clip should limit to ±1.0");
    }

    #[test]
    fn test_distortion_soft_clip() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::SoftClip);
        dist.set_drive(0.8);
        dist.set_mix(1.0);

        // Soft clip should compress smoothly
        let output = dist.process(1.0);
        assert!(output.abs() < 1.0, "Soft clip should compress signal");
    }

    #[test]
    fn test_distortion_cubic() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::Cubic);
        dist.set_drive(0.5);
        dist.set_mix(1.0);

        // Cubic adds subtle distortion
        let input = 0.5;
        let output = dist.process(input);
        
        // Output should be non-linear but not radically different
        assert!((output - input).abs() < 0.5);
    }

    #[test]
    fn test_distortion_stereo() {
        let mut dist = Distortion::new(44100.0);
        dist.set_drive(0.7);
        dist.set_mix(1.0);

        let (out_l, out_r) = dist.process_stereo(0.5, -0.5);

        // Both channels should be processed
        assert!(out_l > 0.0);
        assert!(out_r < 0.0);
    }

    #[test]
    fn test_distortion_stability() {
        let mut dist = Distortion::new(44100.0);
        dist.set_drive(1.0);
        dist.set_mix(1.0);

        // Process for a long time with various inputs
        for i in 0..44100 {
            let input = ((i as f32 * 440.0 * 2.0 * PI) / 44100.0).sin();
            let output = dist.process(input);

            assert!(output.is_finite(), "Distortion produced NaN/inf");
            assert!(output.abs() < 10.0, "Distortion became unstable");
        }
    }

    #[test]
    fn test_distortion_clear() {
        let mut dist = Distortion::new(44100.0);

        // Process some signal
        for _ in 0..100 {
            dist.process(0.5);
        }

        // Clear
        dist.clear();

        // DC blocker state should be reset
        assert_eq!(dist.dc_block_x1, 0.0);
        assert_eq!(dist.dc_block_y1, 0.0);
    }
}
