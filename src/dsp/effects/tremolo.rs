/// Tremolo effect - amplitude modulation with LFO
/// Creates rhythmic volume pulsing
use std::f32::consts::PI;

pub struct Tremolo {
    /// Sample rate for frequency calculations
    sample_rate: f32,

    /// LFO phase accumulator
    lfo_phase: f32,

    /// LFO rate in Hz
    lfo_rate: f32,

    /// Modulation depth (0.0 to 1.0)
    depth: f32,

    /// LFO waveform type
    waveform: Waveform,

    /// Stereo phase offset (0-1, creates stereo width)
    stereo_phase: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Triangle,
    Square,
    Saw,
}

impl Tremolo {
    /// Create a new tremolo effect
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `lfo_rate` - LFO rate in Hz (0.1-20 Hz typical)
    pub fn new(sample_rate: f32, lfo_rate: f32) -> Self {
        Self {
            sample_rate,
            lfo_phase: 0.0,
            lfo_rate: lfo_rate.clamp(0.01, 50.0),
            depth: 1.0,
            waveform: Waveform::Sine,
            stereo_phase: 0.0,
        }
    }

    /// Set LFO rate in Hz
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.lfo_rate = rate_hz.clamp(0.01, 50.0);
    }

    /// Set modulation depth (0.0 to 1.0)
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    /// Set LFO waveform
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Set stereo phase offset (0.0 to 1.0, 0.5 = 180 degrees)
    pub fn set_stereo_phase(&mut self, phase: f32) {
        self.stereo_phase = phase.clamp(0.0, 1.0);
    }

    /// Reset LFO phase to initial state (called when tempo sync mode changes)
    pub fn reset_phase(&mut self) {
        self.lfo_phase = 0.0;
    }

    /// Process a stereo sample through the tremolo
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Generate LFO values for left and right channels
        let lfo_left = self.generate_lfo(self.lfo_phase);
        let lfo_right = self.generate_lfo(self.lfo_phase + self.stereo_phase);

        // Advance LFO phase
        self.lfo_phase += self.lfo_rate / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        // Convert LFO (-1 to 1) to amplitude modulation (0 to 1)
        // With depth control: amp = 1 - depth/2 + lfo * depth/2
        // This keeps the average amplitude at 1.0 - depth/2
        let amp_left = 1.0 - self.depth * 0.5 + lfo_left * self.depth * 0.5;
        let amp_right = 1.0 - self.depth * 0.5 + lfo_right * self.depth * 0.5;

        // Apply amplitude modulation
        let output_left = left * amp_left;
        let output_right = right * amp_right;

        (output_left, output_right)
    }

    /// Generate LFO sample based on waveform
    #[inline]
    fn generate_lfo(&self, phase: f32) -> f32 {
        let phase = phase % 1.0;

        match self.waveform {
            Waveform::Sine => (phase * 2.0 * PI).sin(),
            Waveform::Triangle => {
                if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                }
            }
            Waveform::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Saw => 2.0 * phase - 1.0,
        }
    }

    /// Reset the tremolo state
    pub fn reset(&mut self) {
        self.lfo_phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_tremolo_creation() {
        let tremolo = Tremolo::new(44100.0, 5.0);
        assert_eq!(tremolo.sample_rate, 44100.0);
        assert_relative_eq!(tremolo.lfo_rate, 5.0, epsilon = 0.1);
    }

    #[test]
    fn test_tremolo_rate_clamping() {
        let mut tremolo = Tremolo::new(44100.0, 5.0);

        tremolo.set_rate(0.001); // Too low
        assert!(tremolo.lfo_rate >= 0.01);

        tremolo.set_rate(100.0); // Too high
        assert!(tremolo.lfo_rate <= 50.0);
    }

    #[test]
    fn test_tremolo_modulates_amplitude() {
        let mut tremolo = Tremolo::new(44100.0, 1.0); // 1 Hz
        tremolo.set_depth(1.0);

        let input = 1.0;
        let mut outputs = Vec::new();

        // Collect one full cycle
        for _ in 0..44100 {
            let (left, _) = tremolo.process(input, input);
            outputs.push(left);
        }

        // Output should vary significantly
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!(max - min > 0.5); // Significant amplitude modulation
        assert!(min >= 0.0); // Should not go negative with positive input
        assert!(max <= 1.0); // Should not exceed input
    }

    #[test]
    fn test_tremolo_depth_control() {
        let mut tremolo_no_depth = Tremolo::new(44100.0, 5.0);
        tremolo_no_depth.set_depth(0.0);

        let mut tremolo_full_depth = Tremolo::new(44100.0, 5.0);
        tremolo_full_depth.set_depth(1.0);

        let input = 0.8;

        // No depth should pass signal through
        let (left_no_depth, _) = tremolo_no_depth.process(input, input);
        assert_relative_eq!(left_no_depth, input, epsilon = 0.01);

        // Full depth should modulate (but exact value depends on phase)
        tremolo_full_depth.lfo_phase = 0.25; // Peak of sine wave
        let (left_full_depth, _) = tremolo_full_depth.process(input, input);
        assert!(left_full_depth > 0.0 && left_full_depth <= input);
    }

    #[test]
    fn test_tremolo_sine_wave() {
        let mut tremolo = Tremolo::new(44100.0, 10.0);
        tremolo.set_waveform(Waveform::Sine);
        tremolo.set_depth(1.0);

        let mut outputs = Vec::new();
        for _ in 0..441 {
            // 0.01 seconds
            let (left, _) = tremolo.process(1.0, 1.0);
            outputs.push(left);
        }

        // Should create smooth modulation
        let diffs: Vec<f32> = outputs.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
        let max_diff = diffs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        // Sine should have relatively smooth changes
        assert!(max_diff < 0.5);
    }

    #[test]
    fn test_tremolo_square_wave() {
        let mut tremolo = Tremolo::new(44100.0, 10.0);
        tremolo.set_waveform(Waveform::Square);
        tremolo.set_depth(1.0);

        let mut outputs = Vec::new();
        for _ in 0..4410 {
            // Full cycle at 10 Hz
            let (left, _) = tremolo.process(1.0, 1.0);
            outputs.push(left);
        }

        // Square wave should vary between high and low levels
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        // Should have significant variation
        assert!(max - min > 0.3);
    }

    #[test]
    fn test_tremolo_triangle_wave() {
        let mut tremolo = Tremolo::new(44100.0, 10.0);
        tremolo.set_waveform(Waveform::Triangle);
        tremolo.set_depth(1.0);

        let mut outputs = Vec::new();
        for _ in 0..4410 {
            // Full cycle
            let (left, _) = tremolo.process(1.0, 1.0);
            outputs.push(left);
        }

        // Should modulate
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!(max - min > 0.2);
    }

    #[test]
    fn test_tremolo_saw_wave() {
        let mut tremolo = Tremolo::new(44100.0, 10.0);
        tremolo.set_waveform(Waveform::Saw);
        tremolo.set_depth(1.0);

        let mut outputs = Vec::new();
        for _ in 0..4410 {
            // Full cycle
            let (left, _) = tremolo.process(1.0, 1.0);
            outputs.push(left);
        }

        // Should modulate
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!(max - min > 0.2);
    }

    #[test]
    fn test_tremolo_stereo_phase() {
        let mut tremolo = Tremolo::new(44100.0, 5.0);
        tremolo.set_stereo_phase(0.5); // 180 degrees
        tremolo.set_depth(1.0);

        let input = 1.0;
        let mut left_outputs = Vec::new();
        let mut right_outputs = Vec::new();

        for _ in 0..4410 {
            // Full cycle at 5 Hz
            let (left, right) = tremolo.process(input, input);
            left_outputs.push(left);
            right_outputs.push(right);
        }

        // Left and right should be modulated differently
        // Check that they're not always equal
        let diff: f32 = left_outputs
            .iter()
            .zip(right_outputs.iter())
            .map(|(l, r)| (l - r).abs())
            .sum();

        // Significant stereo difference with phase offset
        assert!(diff > 10.0);
    }

    #[test]
    fn test_tremolo_reset() {
        let mut tremolo = Tremolo::new(44100.0, 5.0);

        // Process some samples
        for _ in 0..100 {
            tremolo.process(1.0, 1.0);
        }

        // Reset
        tremolo.reset();

        // Phase should be cleared
        assert_eq!(tremolo.lfo_phase, 0.0);
    }

    #[test]
    fn test_tremolo_phase_wrap() {
        let mut tremolo = Tremolo::new(44100.0, 5.0);

        // Process many samples - phase should stay bounded
        for _ in 0..10000 {
            tremolo.process(1.0, 1.0);
            assert!(tremolo.lfo_phase >= 0.0 && tremolo.lfo_phase < 1.0);
        }
    }

    #[test]
    fn test_tremolo_no_stereo_phase() {
        let mut tremolo = Tremolo::new(44100.0, 5.0);
        tremolo.set_stereo_phase(0.0); // No phase offset
        tremolo.set_depth(1.0);

        // Both channels should be identical
        for _ in 0..100 {
            let (left, right) = tremolo.process(1.0, 1.0);
            assert_relative_eq!(left, right, epsilon = 0.001);
        }
    }

    #[test]
    fn test_tremolo_preserves_silence() {
        let mut tremolo = Tremolo::new(44100.0, 5.0);
        tremolo.set_depth(1.0);

        // Silence should remain silence
        for _ in 0..100 {
            let (left, right) = tremolo.process(0.0, 0.0);
            assert_eq!(left, 0.0);
            assert_eq!(right, 0.0);
        }
    }
}
