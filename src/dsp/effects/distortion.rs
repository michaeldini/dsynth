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
/// - **drive**: Input gain before distortion (0.0 to 1.0 maps to 1x to 100x)
/// - **mix**: Wet/dry balance (0.0 = dry, 1.0 = full wet)
/// - **algorithm**: Choice of distortion curve (tanh, soft clip, hard clip, cubic)
///
/// # Distortion Types
/// - **Tanh**: Smooth, tube-like saturation (most musical)
/// - **Soft Clip**: Gentle compression then hard limit
/// - **Hard Clip**: Brick-wall limiting (harsh, digital)
/// - **Cubic**: Subtle harmonic enhancement (adds 3rd harmonic)
/// - **Foldback**: Wave folding for complex, metallic harmonics (West Coast style)
/// - **Asymmetric**: Tube-style asymmetric clipping (adds even harmonics)
/// - **SineShaper**: Sine-based smooth musical distortion
/// - **Bitcrush**: Bit depth reduction for lo-fi/retro sound
/// - **Diode**: Models diode clipper circuit (guitar pedal style)
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistortionType {
    Tanh,
    SoftClip,
    HardClip,
    Cubic,
    Foldback,
    Asymmetric,
    SineShaper,
    Bitcrush,
    Diode,
}

/// Distortion/waveshaper processor with true stereo processing
pub struct Distortion {
    // DC blocking filters (high-pass at ~10Hz) - separate L/R for true stereo
    dc_block_x1_l: f32,
    dc_block_y1_l: f32,
    dc_block_x1_r: f32,
    dc_block_y1_r: f32,
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
            dc_block_x1_l: 0.0,
            dc_block_y1_l: 0.0,
            dc_block_x1_r: 0.0,
            dc_block_y1_r: 0.0,
            dc_block_coeff,
            drive: 0.0,
            mix: 0.5,
            dist_type: DistortionType::Tanh,
        }
    }

    /// Set drive amount (0.0 to 1.0)
    /// Maps to 1x to 100x gain internally
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
                // Most gentle - asymptotically approaches ±1
                x.tanh()
            }
            DistortionType::SoftClip => {
                // Piecewise polynomial soft clipper - more aggressive than tanh
                // Transitions smoothly from linear to clipped at ±0.5
                let abs_x = x.abs();
                if abs_x <= 0.5 {
                    x
                } else if abs_x <= 1.5 {
                    let sign = x.signum();
                    let scaled = (abs_x - 0.5) / 1.0; // 0.0 to 1.0 range
                    sign * (0.5 + (1.0 - scaled * scaled) * 0.5)
                } else {
                    x.signum()
                }
            }
            DistortionType::HardClip => {
                // Hard brick-wall clipping at ±0.7 for more aggressive sound
                // Creates harsh, digital-sounding harmonics
                x.clamp(-0.7, 0.7)
            }
            DistortionType::Cubic => {
                // Cubic waveshaper with stronger 3rd harmonic content
                // Creates more obvious harmonic distortion
                let x_clamped = x.clamp(-2.0, 2.0);
                if x_clamped.abs() < 1.5 {
                    x_clamped - 0.4 * x_clamped * x_clamped * x_clamped
                } else {
                    x_clamped.signum() * 0.6
                }
            }
            DistortionType::Foldback => {
                // Wave folding - reflects signal when exceeding threshold
                // Creates complex, metallic harmonics (West Coast synthesis)
                let threshold = 1.0;
                let range = 4.0 * threshold;
                let mut folded = x % range;

                // Normalize to -2*threshold to 2*threshold
                if folded > 2.0 * threshold {
                    folded -= range;
                } else if folded < -2.0 * threshold {
                    folded += range;
                }

                // Fold back when exceeding threshold
                if folded > threshold {
                    2.0 * threshold - folded
                } else if folded < -threshold {
                    -2.0 * threshold - folded
                } else {
                    folded
                }
            }
            DistortionType::Asymmetric => {
                // Asymmetric clipping - models vacuum tube behavior
                // Compresses positive peaks more than negative (adds even harmonics)
                if x > 0.0 {
                    x / (1.0 + 0.8 * x) // Stronger compression on positive halfwave
                } else {
                    x / (1.0 + 0.3 * x.abs()) // Gentler on negative halfwave
                }
            }
            DistortionType::SineShaper => {
                // Sine-based waveshaping - very smooth and musical
                // Less harsh than polynomial methods
                let clamped = x.clamp(-PI / 2.0, PI / 2.0);
                clamped.sin() * 1.5 // Scale up slightly for more presence
            }
            DistortionType::Bitcrush => {
                // Reduce bit depth for lo-fi/digital/retro sound
                // Quantizes signal to fewer discrete levels
                let bits = 4.0; // Effective bit depth (adjustable)
                let steps = 2.0_f32.powf(bits);
                let quantized = (x * steps).round() / steps;
                quantized.clamp(-1.0, 1.0)
            }
            DistortionType::Diode => {
                // Models diode clipper circuit (guitar pedal style)
                // Soft knee followed by hard limiting
                let threshold = 0.6;
                if x.abs() < threshold {
                    x // Linear passthrough below threshold
                } else {
                    // Compress signal above threshold (10% of overshoot)
                    x.signum() * (threshold + (x.abs() - threshold) * 0.1)
                }
            }
        }
    }

    /// DC blocking filter (high-pass) - static function for a single channel
    #[inline]
    fn dc_block(coeff: f32, input: f32, x1: &mut f32, y1: &mut f32) -> f32 {
        let output = coeff * (*y1 + input - *x1);
        *x1 = input;
        *y1 = output;
        output
    }

    /// Process a single sample (uses left channel DC blocking state)
    ///
    /// # Arguments
    /// * `input` - Input sample
    ///
    /// # Returns
    /// Distorted output sample
    ///
    /// **Note**: For true stereo processing, use `process_stereo()` which maintains
    /// separate DC blocking state for each channel.
    pub fn process(&mut self, input: f32) -> f32 {
        // Map drive (0.0 to 1.0) to gain (1.0 to 50.0) - reduced from 100x for more control
        let gain = 1.0 + self.drive * 49.0;

        // Apply distortion
        let distorted = self.apply_distortion(input, gain);

        // Less aggressive compensation to preserve distortion character
        let compensated = distorted / (1.0 + self.drive * 0.3);

        // DC blocking (prevents DC offset from asymmetric distortion)
        let blocked = Self::dc_block(
            self.dc_block_coeff,
            compensated,
            &mut self.dc_block_x1_l,
            &mut self.dc_block_y1_l,
        );

        // Mix wet and dry
        input * (1.0 - self.mix) + blocked * self.mix
    }

    /// Process a stereo sample pair with independent L/R DC blocking
    ///
    /// # Arguments
    /// * `input_l` - Left channel input
    /// * `input_r` - Right channel input
    ///
    /// # Returns
    /// Tuple of (left_output, right_output)
    ///
    /// **True Stereo**: Each channel maintains independent DC blocking state,
    /// preserving stereo imaging and preventing cross-channel artifacts.
    pub fn process_stereo(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        // Map drive (0.0 to 1.0) to gain (1.0 to 50.0)
        let gain = 1.0 + self.drive * 49.0;

        // Process left channel
        let distorted_l = self.apply_distortion(input_l, gain);
        let compensated_l = distorted_l / (1.0 + self.drive * 0.3);
        let blocked_l = Self::dc_block(
            self.dc_block_coeff,
            compensated_l,
            &mut self.dc_block_x1_l,
            &mut self.dc_block_y1_l,
        );
        let out_l = input_l * (1.0 - self.mix) + blocked_l * self.mix;

        // Process right channel
        let distorted_r = self.apply_distortion(input_r, gain);
        let compensated_r = distorted_r / (1.0 + self.drive * 0.3);
        let blocked_r = Self::dc_block(
            self.dc_block_coeff,
            compensated_r,
            &mut self.dc_block_x1_r,
            &mut self.dc_block_y1_r,
        );
        let out_r = input_r * (1.0 - self.mix) + blocked_r * self.mix;

        (out_l, out_r)
    }

    /// Clear filter state for both channels
    pub fn clear(&mut self) {
        self.dc_block_x1_l = 0.0;
        self.dc_block_y1_l = 0.0;
        self.dc_block_x1_r = 0.0;
        self.dc_block_y1_r = 0.0;
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
        assert!(
            low_out.abs() < 1.0,
            "Low amplitude output should be reasonable: {}",
            low_out
        );

        // High amplitude signal should be compressed/clipped
        dist.clear();
        let high_out = dist.process(2.0);
        assert!(
            high_out.abs() < 1.5,
            "Distortion should compress/limit signal"
        );
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
        assert!(
            (out_tanh - out_hard).abs() > 0.00001,
            "Different distortion types should produce different outputs: tanh={}, hard={}, diff={}",
            out_tanh,
            out_hard,
            (out_tanh - out_hard).abs()
        );
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

        // DC blocker state should be reset (check both channels)
        assert_eq!(dist.dc_block_x1_l, 0.0);
        assert_eq!(dist.dc_block_y1_l, 0.0);
        assert_eq!(dist.dc_block_x1_r, 0.0);
        assert_eq!(dist.dc_block_y1_r, 0.0);
    }

    #[test]
    fn test_distortion_foldback() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::Foldback);
        dist.set_drive(0.8);
        dist.set_mix(1.0);

        // Foldback should create folded waveform
        let output = dist.process(0.5);
        assert!(output.is_finite(), "Foldback should produce valid output");
    }

    #[test]
    fn test_distortion_asymmetric() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::Asymmetric);
        dist.set_drive(0.5);
        dist.set_mix(1.0);

        // Asymmetric should treat positive and negative differently
        let pos_out = dist.process(0.5);
        dist.clear();
        let neg_out = dist.process(-0.5);

        // Absolute values should differ due to asymmetry
        assert!(
            (pos_out.abs() - neg_out.abs()).abs() > 0.01,
            "Asymmetric should produce different positive/negative responses"
        );
    }

    #[test]
    fn test_distortion_sine_shaper() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::SineShaper);
        dist.set_drive(0.6);
        dist.set_mix(1.0);

        let output = dist.process(0.3);
        assert!(output.is_finite(), "SineShaper should produce valid output");
    }

    #[test]
    fn test_distortion_bitcrush() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::Bitcrush);
        dist.set_drive(0.3);
        dist.set_mix(1.0);

        // Bitcrush should quantize the signal
        let output = dist.process(0.123456);
        assert!(output.is_finite(), "Bitcrush should produce valid output");
    }

    #[test]
    fn test_distortion_diode() {
        let mut dist = Distortion::new(44100.0);
        dist.set_type(DistortionType::Diode);
        dist.set_drive(0.1); // Very low drive to keep signal below threshold
        dist.set_mix(1.0);

        // Low input with low drive should pass through relatively unchanged
        let low_out = dist.process(0.1);
        assert!(
            (low_out - 0.1).abs() < 0.5,
            "Diode should pass low signals relatively clean"
        );
    }

    #[test]
    fn test_all_distortion_types_compile() {
        let sample_rate = 44100.0;
        let input = 0.5;
        let gain = 10.0;

        // Just verify all types compile and produce valid output
        let types = vec![
            DistortionType::Tanh,
            DistortionType::SoftClip,
            DistortionType::HardClip,
            DistortionType::Cubic,
            DistortionType::Foldback,
            DistortionType::Asymmetric,
            DistortionType::SineShaper,
            DistortionType::Bitcrush,
            DistortionType::Diode,
        ];

        for dist_type in types {
            let mut dist = Distortion::new(sample_rate);
            dist.set_type(dist_type);
            let output = dist.apply_distortion(input, gain);
            assert!(
                output.is_finite(),
                "Distortion type {:?} produced invalid output: {}",
                dist_type,
                output
            );
        }
    }
}
