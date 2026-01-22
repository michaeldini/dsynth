/// Flanger effect using short delay line with LFO modulation
/// Creates metallic, jet-like sweeping sounds similar to tape flanging
use std::f32::consts::PI;

const MAX_DELAY_MS: f32 = 15.0; // Maximum delay time in milliseconds

pub struct Flanger {
    /// Sample rate for time calculations
    sample_rate: f32,

    /// Delay buffer (circular buffer)
    delay_buffer_left: Vec<f32>,
    delay_buffer_right: Vec<f32>,

    /// Write position in delay buffer
    write_pos: usize,

    /// LFO phase accumulator
    lfo_phase: f32,

    /// LFO rate in Hz
    lfo_rate: f32,

    /// Minimum delay time in samples
    min_delay_samples: f32,

    /// Maximum delay time in samples
    max_delay_samples: f32,

    /// Feedback amount (-0.95 to 0.95)
    feedback: f32,

    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    mix: f32,

    /// Stereo phase offset (0-1, creates stereo width)
    stereo_phase: f32,
}

impl Flanger {
    /// Create a new flanger effect
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `min_delay_ms` - Minimum delay time in milliseconds (0.5-5ms typical)
    /// * `max_delay_ms` - Maximum delay time in milliseconds (5-15ms typical)
    /// * `lfo_rate` - LFO rate in Hz (0.1-1.0 Hz typical)
    pub fn new(sample_rate: f32, min_delay_ms: f32, max_delay_ms: f32, lfo_rate: f32) -> Self {
        let min_delay_ms = min_delay_ms.clamp(0.1, MAX_DELAY_MS);
        let max_delay_ms = max_delay_ms.clamp(min_delay_ms, MAX_DELAY_MS);

        // Allocate delay buffer for maximum delay time
        let buffer_size = ((MAX_DELAY_MS / 1000.0) * sample_rate).ceil() as usize + 1;

        Self {
            sample_rate,
            delay_buffer_left: vec![0.0; buffer_size],
            delay_buffer_right: vec![0.0; buffer_size],
            write_pos: 0,
            lfo_phase: 0.0,
            lfo_rate: lfo_rate.clamp(0.01, 20.0),
            min_delay_samples: (min_delay_ms / 1000.0) * sample_rate,
            max_delay_samples: (max_delay_ms / 1000.0) * sample_rate,
            feedback: 0.5,
            mix: 0.5,
            stereo_phase: 0.25, // 90-degree phase offset for stereo
        }
    }

    /// Set LFO rate in Hz
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.lfo_rate = rate_hz.clamp(0.01, 20.0);
    }

    /// Set delay range in milliseconds
    pub fn set_delay_range(&mut self, min_ms: f32, max_ms: f32) {
        let min_ms = min_ms.clamp(0.1, MAX_DELAY_MS);
        let max_ms = max_ms.clamp(min_ms, MAX_DELAY_MS);

        self.min_delay_samples = (min_ms / 1000.0) * self.sample_rate;
        self.max_delay_samples = (max_ms / 1000.0) * self.sample_rate;
    }

    /// Set feedback amount (-0.95 to 0.95)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-0.95, 0.95);
    }

    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Set stereo phase offset (0.0 to 1.0, 0.25 = 90 degrees)
    pub fn set_stereo_phase(&mut self, phase: f32) {
        self.stereo_phase = phase.clamp(0.0, 1.0);
    }

    /// Reset LFO phase to initial state (called when tempo sync mode changes)
    pub fn reset_phase(&mut self) {
        self.lfo_phase = 0.0;
    }

    /// Process a stereo sample through the flanger
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Generate LFO modulation (sine wave for smooth sweep)
        let lfo_left = (self.lfo_phase * 2.0 * PI).sin();
        let lfo_right = ((self.lfo_phase + self.stereo_phase) * 2.0 * PI).sin();

        // Advance LFO phase
        self.lfo_phase += self.lfo_rate / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        // Calculate modulated delay times (linear interpolation)
        let delay_range = self.max_delay_samples - self.min_delay_samples;
        let delay_left = self.min_delay_samples + delay_range * (lfo_left * 0.5 + 0.5);
        let delay_right = self.min_delay_samples + delay_range * (lfo_right * 0.5 + 0.5);

        // Process left channel inline
        let read_pos_float_left = self.write_pos as f32 - delay_left;
        let read_pos_float_left = if read_pos_float_left < 0.0 {
            read_pos_float_left + self.delay_buffer_left.len() as f32
        } else {
            read_pos_float_left
        };

        let read_pos_int_left = read_pos_float_left.floor() as usize % self.delay_buffer_left.len();
        let read_pos_next_left = (read_pos_int_left + 1) % self.delay_buffer_left.len();
        let frac_left = read_pos_float_left - read_pos_float_left.floor();

        let delayed_sample_left = self.delay_buffer_left[read_pos_int_left] * (1.0 - frac_left)
            + self.delay_buffer_left[read_pos_next_left] * frac_left;

        self.delay_buffer_left[self.write_pos] = left + delayed_sample_left * self.feedback;
        let wet_left = delayed_sample_left;

        // Process right channel inline
        let read_pos_float_right = self.write_pos as f32 - delay_right;
        let read_pos_float_right = if read_pos_float_right < 0.0 {
            read_pos_float_right + self.delay_buffer_right.len() as f32
        } else {
            read_pos_float_right
        };

        let read_pos_int_right =
            read_pos_float_right.floor() as usize % self.delay_buffer_right.len();
        let read_pos_next_right = (read_pos_int_right + 1) % self.delay_buffer_right.len();
        let frac_right = read_pos_float_right - read_pos_float_right.floor();

        let delayed_sample_right = self.delay_buffer_right[read_pos_int_right] * (1.0 - frac_right)
            + self.delay_buffer_right[read_pos_next_right] * frac_right;

        self.delay_buffer_right[self.write_pos] = right + delayed_sample_right * self.feedback;
        let wet_right = delayed_sample_right;

        // Mix dry and wet signals
        let output_left = left * (1.0 - self.mix) + wet_left * self.mix;
        let output_right = right * (1.0 - self.mix) + wet_right * self.mix;

        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.delay_buffer_left.len();

        (output_left, output_right)
    }

    /// Reset the flanger state
    pub fn reset(&mut self) {
        self.lfo_phase = 0.0;
        self.write_pos = 0;
        self.delay_buffer_left.fill(0.0);
        self.delay_buffer_right.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_flanger_creation() {
        let flanger = Flanger::new(44100.0, 1.0, 10.0, 0.5);
        assert_eq!(flanger.sample_rate, 44100.0);
        assert!(!flanger.delay_buffer_left.is_empty());
        assert_eq!(
            flanger.delay_buffer_left.len(),
            flanger.delay_buffer_right.len()
        );
    }

    #[test]
    fn test_flanger_delay_range_clamping() {
        let flanger = Flanger::new(44100.0, 0.5, 20.0, 0.5);

        // Max should be clamped to MAX_DELAY_MS
        let expected_max = (MAX_DELAY_MS / 1000.0) * 44100.0;
        assert_relative_eq!(flanger.max_delay_samples, expected_max, epsilon = 1.0);
    }

    #[test]
    fn test_flanger_processes_audio() {
        let mut flanger = Flanger::new(44100.0, 1.0, 5.0, 0.25);
        flanger.set_mix(0.5);

        // Test with impulse
        let (left, right) = flanger.process(1.0, 1.0);

        // Should produce output
        assert!(left.abs() > 0.0);
        assert!(right.abs() > 0.0);

        // Process more samples - should show flanging effect
        for _ in 0..100 {
            let (l, r) = flanger.process(0.0, 0.0);
            // Delayed impulse should appear
            if l.abs() > 0.01 || r.abs() > 0.01 {
                break;
            }
        }
    }

    #[test]
    fn test_flanger_lfo_modulation() {
        let mut flanger = Flanger::new(44100.0, 1.0, 10.0, 1.0); // 1 Hz LFO
        flanger.set_mix(1.0); // Full wet

        // Generate constant input and collect outputs
        let mut outputs = Vec::new();
        for i in 0..44100 {
            let input = if i == 0 { 1.0 } else { 0.0 }; // Impulse at start
            let (left, _) = flanger.process(input, input);
            outputs.push(left);
        }

        // Output should vary significantly over time due to LFO modulation
        let min = outputs
            .iter()
            .skip(100)
            .cloned()
            .fold(f32::INFINITY, f32::min);
        let max = outputs
            .iter()
            .skip(100)
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max - min > 0.01);
    }

    #[test]
    fn test_flanger_feedback() {
        let mut flanger_no_fb = Flanger::new(44100.0, 2.0, 8.0, 0.5);
        flanger_no_fb.set_feedback(0.0);
        flanger_no_fb.set_mix(1.0);

        let mut flanger_with_fb = Flanger::new(44100.0, 2.0, 8.0, 0.5);
        flanger_with_fb.set_feedback(0.7);
        flanger_with_fb.set_mix(1.0);

        // Process impulse and several subsequent samples
        flanger_no_fb.process(1.0, 1.0);
        flanger_with_fb.process(1.0, 1.0);

        let mut sum_no_fb = 0.0;
        let mut sum_with_fb = 0.0;

        for _ in 0..1000 {
            let (l_no, _) = flanger_no_fb.process(0.0, 0.0);
            let (l_with, _) = flanger_with_fb.process(0.0, 0.0);
            sum_no_fb += l_no.abs();
            sum_with_fb += l_with.abs();
        }

        // With feedback should have more sustained energy
        assert!(sum_with_fb > sum_no_fb);
    }

    #[test]
    fn test_flanger_mix_control() {
        let mut flanger = Flanger::new(44100.0, 1.0, 5.0, 0.5);

        // Dry signal (mix = 0.0)
        flanger.set_mix(0.0);
        let (left_dry, _) = flanger.process(0.8, 0.8);
        assert_relative_eq!(left_dry, 0.8, epsilon = 0.01);

        // Wet signal (mix = 1.0)
        flanger.reset();
        flanger.set_mix(1.0);
        let (left_wet, _) = flanger.process(0.8, 0.8);
        // Should be different due to initial delay (likely near zero)
        assert!((left_wet - 0.8).abs() > 0.1);
    }

    #[test]
    fn test_flanger_stereo_phase() {
        let mut flanger = Flanger::new(44100.0, 1.0, 10.0, 0.5);
        flanger.set_stereo_phase(0.5); // 180-degree phase difference
        flanger.set_mix(1.0);

        // Process impulse
        flanger.process(1.0, 1.0);

        // Collect some outputs
        let mut left_outputs = Vec::new();
        let mut right_outputs = Vec::new();

        for _ in 0..1000 {
            let (l, r) = flanger.process(0.0, 0.0);
            left_outputs.push(l);
            right_outputs.push(r);
        }

        // Left and right should be different due to phase offset
        let diff: f32 = left_outputs
            .iter()
            .zip(right_outputs.iter())
            .map(|(l, r)| (l - r).abs())
            .sum();

        assert!(diff > 0.1); // Significant stereo difference
    }

    #[test]
    fn test_flanger_reset() {
        let mut flanger = Flanger::new(44100.0, 1.0, 5.0, 0.5);

        // Process some samples
        for _ in 0..100 {
            flanger.process(1.0, 1.0);
        }

        // Reset
        flanger.reset();

        // State should be cleared
        assert_eq!(flanger.lfo_phase, 0.0);
        assert_eq!(flanger.write_pos, 0);
        assert!(flanger.delay_buffer_left.iter().all(|&x| x == 0.0));
        assert!(flanger.delay_buffer_right.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_flanger_fractional_delay() {
        let mut flanger = Flanger::new(44100.0, 1.0, 5.0, 0.1);
        flanger.set_mix(1.0);

        // Process impulse
        flanger.process(1.0, 1.0);

        // The delayed signal should appear smoothly due to interpolation
        let mut found_peak = false;
        for _ in 0..500 {
            let (left, _) = flanger.process(0.0, 0.0);
            if left.abs() > 0.3 {
                found_peak = true;
                break;
            }
        }

        assert!(found_peak);
    }

    #[test]
    fn test_flanger_rate_limits() {
        let mut flanger = Flanger::new(44100.0, 1.0, 5.0, 0.5);

        // Should clamp extremely low rates
        flanger.set_rate(0.001);
        assert!(flanger.lfo_rate >= 0.01);

        // Should clamp extremely high rates
        flanger.set_rate(100.0);
        assert!(flanger.lfo_rate <= 20.0);
    }
}
