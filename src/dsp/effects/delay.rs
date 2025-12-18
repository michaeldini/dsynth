/// Stereo ping-pong delay effect
///
/// This is a classic delay effect where the delayed signal bounces between left
/// and right channels, creating a spatial "ping-pong" effect. Each repeat alternates
/// channels, which sounds great on synth leads and pads.
///
/// # Architecture
/// - Two delay lines (left and right) with cross-feedback
/// - Feedback controls how many repeats (0.0 = single echo, 0.9 = many repeats)
/// - Delay time in milliseconds (1ms to 2000ms / 2 seconds)
/// - Wet/dry mix control
///
/// # Parameters
/// - **time_ms**: Delay time in milliseconds (1.0 to 2000.0)
/// - **feedback**: Amount of repeats (0.0 to 0.95)
/// - **wet**: Delay signal level (0.0 = dry, 1.0 = full wet)
/// - **dry**: Direct signal level (0.0 = none, 1.0 = full dry)
///
/// # Real-Time Safety
/// Delay buffer is pre-allocated to maximum size (2 seconds at sample rate).
/// No allocations happen during `process()`.

const MAX_DELAY_MS: f32 = 2000.0;

/// Stereo ping-pong delay processor
pub struct StereoDelay {
    sample_rate: f32,
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_index: usize,
    max_samples: usize,

    // Parameters
    time_ms: f32,
    delay_samples: usize,
    feedback: f32,
    wet: f32,
    dry: f32,
}

impl StereoDelay {
    /// Create a new stereo delay
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0)
    ///
    /// # Pre-allocation
    /// Allocates enough memory for 2 seconds of delay at the given sample rate.
    /// For 44.1kHz, this is ~88,200 samples per channel (~353KB total).
    pub fn new(sample_rate: f32) -> Self {
        let max_samples = (sample_rate * MAX_DELAY_MS / 1000.0) as usize;

        Self {
            sample_rate,
            buffer_l: vec![0.0; max_samples],
            buffer_r: vec![0.0; max_samples],
            write_index: 0,
            max_samples,
            time_ms: 500.0,
            delay_samples: (sample_rate * 0.5) as usize, // 500ms default
            feedback: 0.3,
            wet: 0.3,
            dry: 0.7,
        }
    }

    /// Set delay time in milliseconds (1.0 to 2000.0)
    pub fn set_time(&mut self, time_ms: f32) {
        self.time_ms = time_ms.clamp(1.0, MAX_DELAY_MS);
        self.delay_samples = (self.sample_rate * self.time_ms / 1000.0) as usize;
        self.delay_samples = self.delay_samples.min(self.max_samples - 1);
    }

    /// Set feedback amount (0.0 to 0.95)
    /// Higher values create more repeats but can become unstable above 0.95
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95);
    }

    /// Set wet level (0.0 to 1.0)
    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet.clamp(0.0, 1.0);
    }

    /// Set dry level (0.0 to 1.0)
    pub fn set_dry(&mut self, dry: f32) {
        self.dry = dry.clamp(0.0, 1.0);
    }

    /// Process a stereo sample pair
    ///
    /// # Arguments
    /// * `input_l` - Left channel input
    /// * `input_r` - Right channel input
    ///
    /// # Returns
    /// Tuple of (left_output, right_output)
    ///
    /// # Ping-Pong Behavior
    /// The left input feeds the left delay, which crosses to the right delay via feedback.
    /// Similarly, right input feeds right delay, which crosses to left delay.
    /// This creates the characteristic ping-pong bouncing effect.
    pub fn process(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        // Calculate read position (delay samples back from write position)
        let read_index = if self.write_index >= self.delay_samples {
            self.write_index - self.delay_samples
        } else {
            self.max_samples - (self.delay_samples - self.write_index)
        };

        // Read delayed samples
        let delayed_l = self.buffer_l[read_index];
        let delayed_r = self.buffer_r[read_index];

        // Ping-pong: L delay feeds R, R delay feeds L (cross-feedback)
        self.buffer_l[self.write_index] = input_l + delayed_r * self.feedback;
        self.buffer_r[self.write_index] = input_r + delayed_l * self.feedback;

        // Advance write position
        self.write_index = (self.write_index + 1) % self.max_samples;

        // Mix wet and dry
        let output_l = input_l * self.dry + delayed_l * self.wet;
        let output_r = input_r * self.dry + delayed_r * self.wet;

        (output_l, output_r)
    }

    /// Clear delay buffers
    pub fn clear(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_delay_creation() {
        let delay = StereoDelay::new(44100.0);
        assert_eq!(delay.sample_rate, 44100.0);
        assert_eq!(delay.time_ms, 500.0);
        assert_eq!(delay.feedback, 0.3);
    }

    #[test]
    fn test_delay_parameters() {
        let mut delay = StereoDelay::new(44100.0);

        delay.set_time(250.0);
        assert_eq!(delay.time_ms, 250.0);

        delay.set_feedback(0.5);
        assert_eq!(delay.feedback, 0.5);

        delay.set_wet(0.8);
        assert_eq!(delay.wet, 0.8);

        delay.set_dry(0.2);
        assert_eq!(delay.dry, 0.2);
    }

    #[test]
    fn test_delay_parameter_clamping() {
        let mut delay = StereoDelay::new(44100.0);

        delay.set_time(5000.0); // Over max
        assert_eq!(delay.time_ms, MAX_DELAY_MS);

        delay.set_time(0.5); // Under min
        assert_eq!(delay.time_ms, 1.0);

        delay.set_feedback(1.5); // Too high
        assert_eq!(delay.feedback, 0.95);

        delay.set_feedback(-0.5); // Negative
        assert_eq!(delay.feedback, 0.0);
    }

    #[test]
    fn test_delay_dry_passthrough() {
        let mut delay = StereoDelay::new(44100.0);
        delay.set_wet(0.0);
        delay.set_dry(1.0);

        let (out_l, out_r) = delay.process(0.5, -0.5);

        // With dry=1.0 and wet=0.0, output equals input
        assert_relative_eq!(out_l, 0.5, epsilon = 0.001);
        assert_relative_eq!(out_r, -0.5, epsilon = 0.001);
    }

    #[test]
    fn test_delay_timing() {
        let sample_rate = 44100.0;
        let mut delay = StereoDelay::new(sample_rate);
        
        // Set 100ms delay
        delay.set_time(100.0);
        delay.set_wet(1.0);
        delay.set_dry(0.0);
        delay.set_feedback(0.0); // No repeats for this test

        // Send impulse on left channel
        delay.process(1.0, 0.0);

        // Process silence for 100ms - 1 sample
        let samples_100ms = (sample_rate * 0.1) as usize;
        for _ in 1..samples_100ms {
            let (out_l, _out_r) = delay.process(0.0, 0.0);
            // Should be silent until delay time reached
            assert_relative_eq!(out_l, 0.0, epsilon = 0.001);
        }

        // At exactly 100ms, we should see the delayed impulse
        let (out_l, _out_r) = delay.process(0.0, 0.0);
        assert!(out_l.abs() > 0.5, "Delayed signal should appear after delay time");
    }

    #[test]
    fn test_delay_feedback_repeats() {
        let mut delay = StereoDelay::new(44100.0);
        delay.set_time(10.0); // Short delay for fast testing
        delay.set_feedback(0.5);
        delay.set_wet(1.0);
        delay.set_dry(0.0);

        // Send impulse
        delay.process(1.0, 0.0);

        let mut peaks_found = 0;
        let mut prev_sample: f32 = 0.0;

        // Check for multiple peaks (repeats)
        for _ in 0..2000 {
            let (out_l, _) = delay.process(0.0, 0.0);
            
            // Detect peak (rising edge crosses threshold)
            if out_l.abs() > 0.05 && prev_sample.abs() < 0.05 {
                peaks_found += 1;
            }
            prev_sample = out_l;
        }

        // Should have multiple repeats with feedback
        assert!(peaks_found >= 2, "Delay should produce multiple repeats with feedback, found {}", peaks_found);
    }

    #[test]
    fn test_delay_ping_pong() {
        let mut delay = StereoDelay::new(44100.0);
        delay.set_time(10.0); // Short delay
        delay.set_feedback(0.7);
        delay.set_wet(1.0);
        delay.set_dry(0.0);

        // Send impulse on LEFT channel only
        delay.process(1.0, 0.0);

        let delay_samples = delay.delay_samples;
        
        // Process until we see the signal in right channel (ping-pong)
        let mut found_ping_pong = false;
        for _ in 0..(delay_samples * 3) {
            let (_out_l, out_r) = delay.process(0.0, 0.0);
            // Look for signal in right channel (cross-feedback from left)
            if out_r.abs() > 0.05 {
                found_ping_pong = true;
                break;
            }
        }
        
        assert!(found_ping_pong, "Delayed signal should ping-pong to right channel");
    }

    #[test]
    fn test_delay_clear() {
        let mut delay = StereoDelay::new(44100.0);
        delay.set_feedback(0.5);

        // Send impulse
        delay.process(1.0, 1.0);

        // Clear
        delay.clear();

        // Process silence - should be completely silent
        for _ in 0..1000 {
            let (out_l, out_r) = delay.process(0.0, 0.0);
            assert_relative_eq!(out_l, 0.0, epsilon = 0.0001);
            assert_relative_eq!(out_r, 0.0, epsilon = 0.0001);
        }
    }

    #[test]
    fn test_delay_stability() {
        let mut delay = StereoDelay::new(44100.0);
        delay.set_feedback(0.9); // High feedback

        // Process continuous input for a long time
        for _ in 0..44100 {
            let (out_l, out_r) = delay.process(0.1, 0.1);

            // Should not blow up
            assert!(out_l.abs() < 10.0, "Delay became unstable (left)");
            assert!(out_r.abs() < 10.0, "Delay became unstable (right)");
            assert!(out_l.is_finite(), "Delay produced NaN/inf (left)");
            assert!(out_r.is_finite(), "Delay produced NaN/inf (right)");
        }
    }
}
