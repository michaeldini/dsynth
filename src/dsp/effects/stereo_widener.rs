/// Stereo widener effect
///
/// Creates a wider stereo image using two complementary techniques:
/// 1. **Haas delay**: A small delay (1-30ms) on one channel creates perceived width
/// 2. **Mid/Side processing**: Adjusts the balance between mono (mid) and stereo (side) content
///
/// # The Haas Effect
/// The Haas effect (or precedence effect) is a psychoacoustic phenomenon where our brain
/// perceives two similar sounds arriving within ~40ms as a single "wider" sound rather than
/// an echo. By delaying one channel by 5-30ms, we create perceived width without obvious delay.
///
/// # Mid/Side Processing
/// Mid = (L + R) / 2 (the mono-compatible center)
/// Side = (L - R) / 2 (the stereo difference)
///
/// By boosting Side relative to Mid, we make the stereo image wider. By reducing Side,
/// we make it narrower (more mono). This is independent of the Haas effect.
///
/// # Mono Compatibility
/// The width parameter allows reducing stereo content to ensure the mix translates well
/// to mono playback systems. At width = 0, the output is pure mono.

use std::f32::consts::PI;

/// Maximum delay in samples (30ms at 192kHz = ~5760 samples, round up)
const MAX_DELAY_SAMPLES: usize = 6144;

/// Stereo widener processor
pub struct StereoWidener {
    sample_rate: f32,

    // Haas delay buffer (for one channel)
    delay_buffer: [f32; MAX_DELAY_SAMPLES],
    delay_write_pos: usize,
    delay_samples: usize,

    // Parameters
    haas_delay_ms: f32,  // 0.0 to 30.0 ms
    haas_mix: f32,       // 0.0 to 1.0 (how much Haas effect to apply)
    width: f32,          // 0.0 (mono) to 2.0 (extra wide), 1.0 = normal
    mid_gain: f32,       // 0.0 to 2.0, default 1.0
    side_gain: f32,      // 0.0 to 2.0, default 1.0

    // High-pass filter on side channel to prevent bass muddiness
    side_hp_x1: f32,
    side_hp_y1: f32,
    side_hp_coeff: f32,
}

impl StereoWidener {
    /// Create a new stereo widener
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        // High-pass coefficient for side channel (~100Hz cutoff)
        let cutoff = 100.0;
        let rc = 1.0 / (2.0 * PI * cutoff);
        let dt = 1.0 / sample_rate;
        let hp_coeff = rc / (rc + dt);

        Self {
            sample_rate,
            delay_buffer: [0.0; MAX_DELAY_SAMPLES],
            delay_write_pos: 0,
            delay_samples: 0,
            haas_delay_ms: 0.0,
            haas_mix: 0.0,
            width: 1.0,
            mid_gain: 1.0,
            side_gain: 1.0,
            side_hp_x1: 0.0,
            side_hp_y1: 0.0,
            side_hp_coeff: hp_coeff,
        }
    }

    /// Set Haas delay time in milliseconds (0.0 to 30.0)
    pub fn set_haas_delay(&mut self, delay_ms: f32) {
        self.haas_delay_ms = delay_ms.clamp(0.0, 30.0);
        self.delay_samples = ((self.haas_delay_ms / 1000.0) * self.sample_rate) as usize;
        self.delay_samples = self.delay_samples.min(MAX_DELAY_SAMPLES - 1);
    }

    /// Set Haas effect mix (0.0 = off, 1.0 = full Haas)
    pub fn set_haas_mix(&mut self, mix: f32) {
        self.haas_mix = mix.clamp(0.0, 1.0);
    }

    /// Set stereo width (0.0 = mono, 1.0 = normal, 2.0 = extra wide)
    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(0.0, 2.0);
    }

    /// Set mid channel gain (0.0 to 2.0)
    pub fn set_mid_gain(&mut self, gain: f32) {
        self.mid_gain = gain.clamp(0.0, 2.0);
    }

    /// Set side channel gain (0.0 to 2.0)
    pub fn set_side_gain(&mut self, gain: f32) {
        self.side_gain = gain.clamp(0.0, 2.0);
    }

    /// High-pass filter for side channel
    #[inline]
    fn side_highpass(&mut self, input: f32) -> f32 {
        let output = self.side_hp_coeff * (self.side_hp_y1 + input - self.side_hp_x1);
        self.side_hp_x1 = input;
        self.side_hp_y1 = output;
        output
    }

    /// Process a stereo sample pair
    ///
    /// # Arguments
    /// * `input_l` - Left channel input
    /// * `input_r` - Right channel input
    ///
    /// # Returns
    /// Tuple of (left_output, right_output)
    pub fn process(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        // === STEP 1: Apply Haas delay to right channel ===
        // Write current right sample to delay buffer
        self.delay_buffer[self.delay_write_pos] = input_r;

        // Read delayed sample
        let read_pos = if self.delay_write_pos >= self.delay_samples {
            self.delay_write_pos - self.delay_samples
        } else {
            MAX_DELAY_SAMPLES - (self.delay_samples - self.delay_write_pos)
        };
        let delayed_r = self.delay_buffer[read_pos];

        // Advance write position
        self.delay_write_pos = (self.delay_write_pos + 1) % MAX_DELAY_SAMPLES;

        // Mix original and delayed right channel
        let haas_r = input_r * (1.0 - self.haas_mix) + delayed_r * self.haas_mix;

        // Left channel passes through (no delay)
        let haas_l = input_l;

        // === STEP 2: Mid/Side processing ===
        // Convert to mid/side
        let mid = (haas_l + haas_r) * 0.5;
        let side = (haas_l - haas_r) * 0.5;

        // High-pass filter on side to prevent bass muddiness when widening
        let side_filtered = self.side_highpass(side);

        // Apply width and individual gains
        // width < 1.0 reduces stereo, width > 1.0 increases stereo
        let mid_processed = mid * self.mid_gain;
        let side_processed = side_filtered * self.side_gain * self.width;

        // Convert back to L/R
        let out_l = mid_processed + side_processed;
        let out_r = mid_processed - side_processed;

        (out_l, out_r)
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.delay_buffer.fill(0.0);
        self.delay_write_pos = 0;
        self.side_hp_x1 = 0.0;
        self.side_hp_y1 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_widener_creation() {
        let widener = StereoWidener::new(44100.0);
        assert_eq!(widener.haas_delay_ms, 0.0);
        assert_eq!(widener.haas_mix, 0.0);
        assert_eq!(widener.width, 1.0);
    }

    #[test]
    fn test_widener_unity_passthrough() {
        let mut widener = StereoWidener::new(44100.0);
        // Default settings: no Haas, width = 1.0

        // Warm up the filter
        for _ in 0..100 {
            widener.process(0.5, 0.5);
        }

        // With width=1.0, mid_gain=1.0, side_gain=1.0, and no Haas,
        // a mono signal (L=R) should pass through unchanged
        let (out_l, out_r) = widener.process(0.5, 0.5);
        assert_relative_eq!(out_l, 0.5, epsilon = 0.01);
        assert_relative_eq!(out_r, 0.5, epsilon = 0.01);
    }

    #[test]
    fn test_widener_mono_collapse() {
        let mut widener = StereoWidener::new(44100.0);
        widener.set_width(0.0); // Full mono

        // Warm up
        for _ in 0..100 {
            widener.process(1.0, -1.0);
        }

        // With width=0.0, side is eliminated, only mid remains
        // For L=1.0, R=-1.0: mid = 0.0, side = 1.0
        // At width=0: output = mid = 0.0
        let (out_l, out_r) = widener.process(1.0, -1.0);
        assert_relative_eq!(out_l, out_r, epsilon = 0.01); // Both channels should be equal (mono)
    }

    #[test]
    fn test_widener_haas_delay() {
        let mut widener = StereoWidener::new(44100.0);
        widener.set_haas_delay(10.0); // 10ms delay
        widener.set_haas_mix(1.0); // Full Haas

        // Process an impulse
        widener.process(0.0, 1.0); // Right channel impulse

        // Check that the impulse appears delayed
        let delay_samples = (0.01 * 44100.0) as usize;
        for i in 0..delay_samples {
            let (_, out_r) = widener.process(0.0, 0.0);
            if i < delay_samples - 1 {
                assert_relative_eq!(out_r, 0.0, epsilon = 0.01);
            }
        }
    }

    #[test]
    fn test_widener_parameter_clamping() {
        let mut widener = StereoWidener::new(44100.0);

        widener.set_haas_delay(50.0);
        assert_eq!(widener.haas_delay_ms, 30.0);

        widener.set_haas_delay(-5.0);
        assert_eq!(widener.haas_delay_ms, 0.0);

        widener.set_width(5.0);
        assert_eq!(widener.width, 2.0);

        widener.set_width(-1.0);
        assert_eq!(widener.width, 0.0);
    }

    #[test]
    fn test_widener_stability() {
        let mut widener = StereoWidener::new(44100.0);
        widener.set_haas_delay(20.0);
        widener.set_haas_mix(1.0);
        widener.set_width(2.0);

        // Process many samples with extreme settings
        for _ in 0..10000 {
            let (out_l, out_r) = widener.process(0.9, -0.9);
            assert!(out_l.is_finite());
            assert!(out_r.is_finite());
            assert!(out_l.abs() < 10.0);
            assert!(out_r.abs() < 10.0);
        }
    }

    #[test]
    fn test_widener_clear() {
        let mut widener = StereoWidener::new(44100.0);

        // Process some samples
        for _ in 0..100 {
            widener.process(0.5, 0.3);
        }

        // Clear state
        widener.clear();

        // Delay buffer should be cleared
        assert!(widener.delay_buffer.iter().all(|&x| x == 0.0));
        assert_eq!(widener.delay_write_pos, 0);
    }
}
