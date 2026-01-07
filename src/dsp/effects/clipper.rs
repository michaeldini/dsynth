/// Brick-wall clipper for maximum loudness
///
/// Hard clips signal at adjustable threshold for aggressive limiting.
/// Unlike soft saturation (tanh, etc.), this provides true brick-wall
/// limiting at the cost of harmonic distortion.
///
/// # Use Cases
/// - **Kick drums**: Maximize perceived loudness for competitive mixes
/// - **Final stage limiting**: Ensure output never exceeds threshold
/// - **Creative distortion**: Hard clipping adds aggressive harmonics
///
/// # Technical Details
/// - Zero latency (no lookahead)
/// - Instantaneous clipping (no attack/release)
/// - Symmetric clipping (positive and negative peaks treated equally)
/// - Can introduce aliasing at high frequencies (acceptable for kicks)

/// Hard clipper with adjustable threshold
pub struct Clipper {
    threshold: f32,
    enabled: bool,
}

impl Clipper {
    /// Create a new clipper
    ///
    /// # Arguments
    /// * `threshold` - Maximum allowed amplitude (0.0-1.0, default 0.95)
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
            enabled: true,
        }
    }

    /// Set the clipping threshold
    ///
    /// # Arguments
    /// * `threshold` - Maximum allowed amplitude (0.0-1.0)
    ///   - 1.0 = no clipping
    ///   - 0.95 = typical "safety" limiting
    ///   - 0.7 = aggressive loudness maximization
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get the current threshold
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Enable or disable clipping
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if clipping is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Process a single sample (mono)
    ///
    /// Clips the input to [-threshold, +threshold] range.
    ///
    /// # Arguments
    /// * `input` - Input sample
    ///
    /// # Returns
    /// Clipped output sample
    #[inline]
    pub fn process(&self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        input.clamp(-self.threshold, self.threshold)
    }

    /// Process a stereo sample pair
    ///
    /// # Arguments
    /// * `left` - Left channel input
    /// * `right` - Right channel input
    ///
    /// # Returns
    /// (left_out, right_out) - Clipped stereo pair
    #[inline]
    pub fn process_stereo(&self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }

        (
            left.clamp(-self.threshold, self.threshold),
            right.clamp(-self.threshold, self.threshold),
        )
    }

    /// Reset clipper state (no-op for stateless clipper)
    pub fn reset(&mut self) {
        // Clipper is stateless, nothing to reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_clipper_creation() {
        let clipper = Clipper::new(0.95);
        assert_relative_eq!(clipper.threshold(), 0.95, epsilon = 1e-6);
        assert!(clipper.is_enabled());
    }

    #[test]
    fn test_threshold_clamping() {
        // Test that threshold is clamped to valid range
        let clipper = Clipper::new(1.5);
        assert_relative_eq!(clipper.threshold(), 1.0, epsilon = 1e-6);

        let clipper = Clipper::new(-0.5);
        assert_relative_eq!(clipper.threshold(), 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_no_clipping_below_threshold() {
        let clipper = Clipper::new(0.95);

        // Signals below threshold should pass through unchanged
        assert_relative_eq!(clipper.process(0.5), 0.5, epsilon = 1e-6);
        assert_relative_eq!(clipper.process(-0.5), -0.5, epsilon = 1e-6);
        assert_relative_eq!(clipper.process(0.9), 0.9, epsilon = 1e-6);
    }

    #[test]
    fn test_clipping_above_threshold() {
        let clipper = Clipper::new(0.8);

        // Positive peaks should be clipped
        assert_relative_eq!(clipper.process(1.0), 0.8, epsilon = 1e-6);
        assert_relative_eq!(clipper.process(0.9), 0.8, epsilon = 1e-6);

        // Negative peaks should be clipped
        assert_relative_eq!(clipper.process(-1.0), -0.8, epsilon = 1e-6);
        assert_relative_eq!(clipper.process(-0.9), -0.8, epsilon = 1e-6);
    }

    #[test]
    fn test_symmetric_clipping() {
        let clipper = Clipper::new(0.7);

        // Clipping should be symmetric
        let positive = clipper.process(1.0);
        let negative = clipper.process(-1.0);

        assert_relative_eq!(positive, 0.7, epsilon = 1e-6);
        assert_relative_eq!(negative, -0.7, epsilon = 1e-6);
        assert_relative_eq!(positive, -negative, epsilon = 1e-6);
    }

    #[test]
    fn test_stereo_processing() {
        let clipper = Clipper::new(0.8);

        let (left, right) = clipper.process_stereo(1.0, -1.0);
        assert_relative_eq!(left, 0.8, epsilon = 1e-6);
        assert_relative_eq!(right, -0.8, epsilon = 1e-6);

        let (left, right) = clipper.process_stereo(0.5, 0.5);
        assert_relative_eq!(left, 0.5, epsilon = 1e-6);
        assert_relative_eq!(right, 0.5, epsilon = 1e-6);
    }

    #[test]
    fn test_disabled_bypass() {
        let mut clipper = Clipper::new(0.5);
        clipper.set_enabled(false);

        // When disabled, input should pass through unchanged
        assert_relative_eq!(clipper.process(1.0), 1.0, epsilon = 1e-6);
        assert_relative_eq!(clipper.process(-1.0), -1.0, epsilon = 1e-6);

        let (left, right) = clipper.process_stereo(1.5, -1.5);
        assert_relative_eq!(left, 1.5, epsilon = 1e-6);
        assert_relative_eq!(right, -1.5, epsilon = 1e-6);
    }

    #[test]
    fn test_threshold_update() {
        let mut clipper = Clipper::new(0.9);

        // Initial clipping
        assert_relative_eq!(clipper.process(1.0), 0.9, epsilon = 1e-6);

        // Update threshold
        clipper.set_threshold(0.7);
        assert_relative_eq!(clipper.process(1.0), 0.7, epsilon = 1e-6);

        // Disable clipping
        clipper.set_threshold(1.0);
        assert_relative_eq!(clipper.process(1.0), 1.0, epsilon = 1e-6);
    }
}
