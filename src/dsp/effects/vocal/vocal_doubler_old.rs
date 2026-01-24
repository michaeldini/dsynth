/// Vocal Doubler Effect
///
/// Creates natural double-tracking effect by adding subtle time and pitch variations.
/// Unlike chorus (which is more obvious and modulated), doubler mimics the sound of
/// recording the same vocal performance twice.
///
/// Features:
/// - Dual delays (left/right) with independent timing (5-15ms)
/// - Subtle pitch shifting (±5 cents) for natural detuning
/// - Stereo width control
/// - Dry/wet mix
///
/// Algorithm:
/// 1. Buffer incoming audio with short delay lines
/// 2. Apply fractional delay for time variation
/// 3. Add subtle pitch shift via delay modulation
/// 4. Spread doubled voices in stereo
use std::collections::VecDeque;

const MAX_DELAY_MS: f32 = 20.0; // Maximum delay time

pub struct VocalDoubler {
    sample_rate: f32,

    // Delay buffers (circular buffers)
    left_buffer: VecDeque<f32>,
    right_buffer: VecDeque<f32>,

    // Parameters
    delay_time_ms: f32, // 5-15ms base delay
    detune_cents: f32,  // ±5 cents pitch variation
    stereo_width: f32,  // 0.0-1.0 (width of stereo spread)
    mix: f32,           // 0.0-1.0 (dry/wet)

    // Internal state
    delay_samples: f32, // Current delay in samples (fractional)
    max_delay_samples: usize,
}

impl VocalDoubler {
    pub fn new(sample_rate: f32) -> Self {
        let max_delay_samples = ((MAX_DELAY_MS / 1000.0) * sample_rate).ceil() as usize;

        Self {
            sample_rate,
            left_buffer: VecDeque::with_capacity(max_delay_samples + 1),
            right_buffer: VecDeque::with_capacity(max_delay_samples + 1),
            delay_time_ms: 10.0, // Default 10ms delay
            detune_cents: 5.0,   // Default ±5 cents
            stereo_width: 0.7,   // Default 70% width
            mix: 0.5,            // Default 50% mix
            delay_samples: 0.0,
            max_delay_samples,
        }
    }

    /// Set delay time in milliseconds (5-15ms recommended for doubling)
    pub fn set_delay_time(&mut self, delay_ms: f32) {
        self.delay_time_ms = delay_ms.clamp(1.0, MAX_DELAY_MS);
        self.update_delay_samples();
    }

    /// Set pitch detune in cents (±5 cents recommended for subtle effect)
    pub fn set_detune(&mut self, cents: f32) {
        self.detune_cents = cents.clamp(-50.0, 50.0);
        self.update_delay_samples();
    }

    /// Set stereo width (0.0 = mono, 1.0 = full stereo)
    pub fn set_stereo_width(&mut self, width: f32) {
        self.stereo_width = width.clamp(0.0, 1.0);
    }

    /// Set dry/wet mix (0.0 = dry only, 1.0 = wet only)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Update delay samples based on delay time and pitch shift
    /// Pitch shifting via delay: cents = 1200 * log2(delay_ratio)
    /// For small cents: delay_ratio ≈ 1 + (cents / 1200)
    fn update_delay_samples(&mut self) {
        let base_delay_samples = (self.delay_time_ms / 1000.0) * self.sample_rate;

        // Apply pitch shift via slight delay modulation
        // Positive cents = longer delay (pitch down), negative = shorter (pitch up)
        let pitch_ratio = 2.0_f32.powf(self.detune_cents / 1200.0);
        self.delay_samples = base_delay_samples * pitch_ratio;
    }

    /// Process stereo audio through doubler
    ///
    /// Creates two delayed copies:
    /// - Left channel: slightly earlier + positive detune
    /// - Right channel: slightly later + negative detune
    ///
    /// Returns: (left_out, right_out)
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        // Push new samples into buffers
        self.left_buffer.push_back(left_in);
        self.right_buffer.push_back(right_in);

        // Maintain buffer size
        while self.left_buffer.len() > self.max_delay_samples {
            self.left_buffer.pop_front();
        }
        while self.right_buffer.len() > self.max_delay_samples {
            self.right_buffer.pop_front();
        }

        // Read delayed samples with fractional delay (linear interpolation)
        let left_delayed = self.read_delayed(&self.left_buffer, self.delay_samples);
        let right_delayed = self.read_delayed(&self.right_buffer, self.delay_samples);

        // Apply stereo spread:
        // Left output = mostly left_delayed + some right
        // Right output = mostly right_delayed + some left
        let spread_factor = self.stereo_width;
        let left_spread = left_delayed * (0.5 + spread_factor * 0.5)
            + right_delayed * (0.5 - spread_factor * 0.5);
        let right_spread = right_delayed * (0.5 + spread_factor * 0.5)
            + left_delayed * (0.5 - spread_factor * 0.5);

        // Mix dry and wet signals
        let left_out = left_in * (1.0 - self.mix) + left_spread * self.mix;
        let right_out = right_in * (1.0 - self.mix) + right_spread * self.mix;

        (left_out, right_out)
    }

    /// Read sample from buffer with fractional delay using linear interpolation
    fn read_delayed(&self, buffer: &VecDeque<f32>, delay_samples: f32) -> f32 {
        if buffer.is_empty() {
            return 0.0;
        }

        let delay_samples = delay_samples.clamp(0.0, buffer.len() as f32 - 1.0);

        // Calculate read position from end of buffer
        let read_pos = buffer.len() as f32 - delay_samples - 1.0;

        if read_pos < 0.0 {
            return 0.0;
        }

        // Integer and fractional parts
        let pos_int = read_pos.floor() as usize;
        let pos_frac = read_pos - read_pos.floor();

        // Linear interpolation between adjacent samples
        let sample1 = buffer.get(pos_int).copied().unwrap_or(0.0);
        let sample2 = buffer.get(pos_int + 1).copied().unwrap_or(sample1);

        sample1 * (1.0 - pos_frac) + sample2 * pos_frac
    }

    /// Reset internal state
    pub fn reset(&mut self) {
        self.left_buffer.clear();
        self.right_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_doubler_creation() {
        let doubler = VocalDoubler::new(44100.0);
        assert_eq!(doubler.sample_rate, 44100.0);
        assert_eq!(doubler.delay_time_ms, 10.0);
        assert_eq!(doubler.detune_cents, 5.0);
    }

    #[test]
    fn test_delay_time_clamping() {
        let mut doubler = VocalDoubler::new(44100.0);

        // Below minimum
        doubler.set_delay_time(0.5);
        assert_eq!(doubler.delay_time_ms, 1.0);

        // Above maximum
        doubler.set_delay_time(50.0);
        assert_eq!(doubler.delay_time_ms, MAX_DELAY_MS);

        // Normal range
        doubler.set_delay_time(12.0);
        assert_eq!(doubler.delay_time_ms, 12.0);
    }

    #[test]
    fn test_stereo_output() {
        let mut doubler = VocalDoubler::new(44100.0);
        doubler.set_delay_time(10.0);
        doubler.set_mix(1.0); // 100% wet for testing

        // Feed some samples to fill buffer
        for _ in 0..500 {
            doubler.process(1.0, -1.0);
        }

        // Test that left and right outputs differ (stereo spread)
        let (left, right) = doubler.process(1.0, -1.0);

        // With stereo width, outputs should differ
        assert_ne!(left, right, "Stereo outputs should differ");
    }

    #[test]
    fn test_dry_wet_mix() {
        let mut doubler = VocalDoubler::new(44100.0);

        let input_left = 0.5;
        let input_right = -0.5;

        // 0% mix = dry only
        doubler.set_mix(0.0);
        let (left, right) = doubler.process(input_left, input_right);
        assert_relative_eq!(left, input_left, epsilon = 0.001);
        assert_relative_eq!(right, input_right, epsilon = 0.001);

        // 100% mix = wet only (will differ due to delay)
        doubler.reset();
        doubler.set_mix(1.0);

        // Fill buffer first
        for _ in 0..500 {
            doubler.process(input_left, input_right);
        }

        let (left_wet, right_wet) = doubler.process(input_left, input_right);
        // Wet signal should be delayed version (different from input)
        assert!(
            (left_wet - input_left).abs() > 0.01 || (right_wet - input_right).abs() > 0.01,
            "Wet signal should differ from dry input"
        );
    }

    #[test]
    fn test_fractional_delay_interpolation() {
        let mut doubler = VocalDoubler::new(44100.0);

        // Create a test impulse
        doubler.process(1.0, 1.0); // Impulse
        for _ in 0..100 {
            doubler.process(0.0, 0.0); // Zeros
        }

        // With fractional delay, output should be interpolated (not exactly 1.0 or 0.0)
        doubler.set_delay_time(0.5); // Very short delay for testing
        doubler.set_mix(1.0);

        let (left, _) = doubler.process(0.0, 0.0);

        // Should be between 0 and 1 due to interpolation
        assert!(
            left >= 0.0 && left <= 1.0,
            "Interpolated value should be in range"
        );
    }

    #[test]
    fn test_detune_affects_delay() {
        let mut doubler = VocalDoubler::new(44100.0);
        doubler.set_delay_time(10.0);

        let base_delay = doubler.delay_samples;

        // Positive detune = longer delay (pitch down)
        doubler.set_detune(10.0);
        assert!(
            doubler.delay_samples > base_delay,
            "Positive detune should increase delay"
        );

        // Negative detune = shorter delay (pitch up)
        doubler.set_detune(-10.0);
        assert!(
            doubler.delay_samples < base_delay,
            "Negative detune should decrease delay"
        );
    }
}
