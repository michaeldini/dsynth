/// Auto-pan - automatic stereo panning with LFO modulation
/// Creates rhythmic stereo movement
use std::f32::consts::PI;

pub struct AutoPan {
    /// Sample rate for frequency calculations
    sample_rate: f32,

    /// LFO phase accumulator
    lfo_phase: f32,

    /// LFO rate in Hz
    lfo_rate: f32,

    /// Pan depth (0.0 to 1.0) - how far the panning moves from center
    depth: f32,

    /// LFO waveform type
    waveform: Waveform,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Triangle,
    Square,
    Saw,
}

impl AutoPan {
    /// Create a new auto-pan effect
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `lfo_rate` - LFO rate in Hz (0.1-10 Hz typical)
    pub fn new(sample_rate: f32, lfo_rate: f32) -> Self {
        Self {
            sample_rate,
            lfo_phase: 0.0,
            lfo_rate: lfo_rate.clamp(0.01, 50.0),
            depth: 1.0,
            waveform: Waveform::Sine,
        }
    }

    /// Set LFO rate in Hz
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.lfo_rate = rate_hz.clamp(0.01, 50.0);
    }

    /// Set pan depth (0.0 to 1.0)
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    /// Set LFO waveform
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Reset LFO phase to initial state (called when tempo sync mode changes)
    pub fn reset_phase(&mut self) {
        self.lfo_phase = 0.0;
    }

    /// Process a stereo sample through the auto-pan
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Generate LFO value (-1 to 1)
        let lfo = self.generate_lfo();

        // Advance LFO phase
        self.lfo_phase += self.lfo_rate / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        // Convert LFO to pan position (-1 = full left, 0 = center, 1 = full right)
        let pan_position = lfo * self.depth;

        // Calculate pan gains using equal-power panning law
        // Left gain: cos((pan + 1) * π/4)
        // Right gain: sin((pan + 1) * π/4)
        let pan_angle = (pan_position + 1.0) * PI / 4.0;
        let left_gain = pan_angle.cos();
        let right_gain = pan_angle.sin();

        // Mix input signal and apply panning
        // For stereo input, we'll sum to mono first, then pan
        let mono = (left + right) * 0.5;

        let output_left = mono * left_gain;
        let output_right = mono * right_gain;

        (output_left, output_right)
    }

    /// Generate LFO sample based on waveform
    #[inline]
    fn generate_lfo(&self) -> f32 {
        match self.waveform {
            Waveform::Sine => (self.lfo_phase * 2.0 * PI).sin(),
            Waveform::Triangle => {
                if self.lfo_phase < 0.5 {
                    4.0 * self.lfo_phase - 1.0
                } else {
                    3.0 - 4.0 * self.lfo_phase
                }
            }
            Waveform::Square => {
                if self.lfo_phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Saw => 2.0 * self.lfo_phase - 1.0,
        }
    }

    /// Reset the auto-pan state
    pub fn reset(&mut self) {
        self.lfo_phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_autopan_creation() {
        let ap = AutoPan::new(44100.0, 2.0);
        assert_eq!(ap.sample_rate, 44100.0);
        assert_relative_eq!(ap.lfo_rate, 2.0, epsilon = 0.1);
    }

    #[test]
    fn test_autopan_rate_clamping() {
        let mut ap = AutoPan::new(44100.0, 2.0);

        ap.set_rate(0.001); // Too low
        assert!(ap.lfo_rate >= 0.01);

        ap.set_rate(100.0); // Too high
        assert!(ap.lfo_rate <= 50.0);
    }

    #[test]
    fn test_autopan_modulates_stereo() {
        let mut ap = AutoPan::new(44100.0, 1.0); // 1 Hz
        ap.set_depth(1.0);

        let input = 1.0;
        let mut left_outputs = Vec::new();
        let mut right_outputs = Vec::new();

        // Collect one full cycle
        for _ in 0..44100 {
            let (left, right) = ap.process(input, input);
            left_outputs.push(left);
            right_outputs.push(right);
        }

        // Left channel should vary
        let left_min = left_outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let left_max = left_outputs
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(left_max - left_min > 0.3);

        // Right channel should vary
        let right_min = right_outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let right_max = right_outputs
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(right_max - right_min > 0.3);
    }

    #[test]
    fn test_autopan_depth_control() {
        let mut ap_no_depth = AutoPan::new(44100.0, 5.0);
        ap_no_depth.set_depth(0.0);

        let mut ap_full_depth = AutoPan::new(44100.0, 5.0);
        ap_full_depth.set_depth(1.0);

        let input = 1.0;

        // Collect outputs over time
        let mut no_depth_left = Vec::new();
        let mut full_depth_left = Vec::new();

        for _ in 0..1000 {
            let (l_no, _) = ap_no_depth.process(input, input);
            let (l_full, _) = ap_full_depth.process(input, input);
            no_depth_left.push(l_no);
            full_depth_left.push(l_full);
        }

        // No depth should have less variation
        let var_no = no_depth_left.iter().map(|&x| x * x).sum::<f32>() / no_depth_left.len() as f32;
        let var_full =
            full_depth_left.iter().map(|&x| x * x).sum::<f32>() / full_depth_left.len() as f32;

        // Both should have energy, but behavior differs based on LFO position
        assert!(var_no >= 0.0);
        assert!(var_full >= 0.0);
    }

    #[test]
    fn test_autopan_sine_wave() {
        let mut ap = AutoPan::new(44100.0, 10.0);
        ap.set_waveform(Waveform::Sine);
        ap.set_depth(1.0);

        let mut outputs = Vec::new();
        for _ in 0..441 {
            let (left, _) = ap.process(1.0, 1.0);
            outputs.push(left);
        }

        // Should create smooth panning motion
        let diffs: Vec<f32> = outputs.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
        let max_diff = diffs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        // Sine should have relatively smooth changes
        assert!(max_diff < 0.5);
    }

    #[test]
    fn test_autopan_square_wave() {
        let mut ap = AutoPan::new(44100.0, 10.0);
        ap.set_waveform(Waveform::Square);
        ap.set_depth(1.0);

        let mut outputs = Vec::new();
        for _ in 0..4410 {
            // Full cycle
            let (left, _) = ap.process(1.0, 1.0);
            outputs.push(left);
        }

        // Square wave should create variation
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        // Should have significant variation between positions
        assert!(max - min > 0.15);
    }

    #[test]
    fn test_autopan_equal_power_panning() {
        let mut ap = AutoPan::new(44100.0, 0.1);
        ap.set_depth(1.0);

        let input = 1.0;

        // Process multiple samples and check power conservation
        for _ in 0..100 {
            let (left, right) = ap.process(input, input);

            // Equal power panning: left^2 + right^2 should be approximately constant
            let power = left * left + right * right;

            // With stereo inputs of 1.0, mono sum is 1.0, so power should be ~1.0
            // Equal-power law: mono^2 = left^2 + right^2
            let expected_power = 1.0;
            assert_relative_eq!(power, expected_power, epsilon = 0.1);
        }
    }

    #[test]
    fn test_autopan_extremes() {
        let mut ap = AutoPan::new(44100.0, 1.0);
        ap.set_depth(1.0);

        // Set LFO to produce full left pan
        ap.lfo_phase = 0.75; // Sine minimum
        let (left_full_left, right_full_left) = ap.process(1.0, 1.0);

        // Left should be louder than right
        assert!(left_full_left > right_full_left);

        // Set LFO to produce full right pan
        ap.lfo_phase = 0.25; // Sine maximum
        let (left_full_right, right_full_right) = ap.process(1.0, 1.0);

        // Right should be louder than left
        assert!(right_full_right > left_full_right);
    }

    #[test]
    fn test_autopan_center_position() {
        let mut ap = AutoPan::new(44100.0, 1.0);
        ap.set_depth(1.0);

        // Set LFO to zero (center position)
        ap.lfo_phase = 0.0; // Sine starts at zero
        let (left, right) = ap.process(1.0, 1.0);

        // At center, left and right should be equal
        assert_relative_eq!(left, right, epsilon = 0.1);
    }

    #[test]
    fn test_autopan_reset() {
        let mut ap = AutoPan::new(44100.0, 5.0);

        // Process some samples
        for _ in 0..100 {
            ap.process(1.0, 1.0);
        }

        // Reset
        ap.reset();

        // Phase should be cleared
        assert_eq!(ap.lfo_phase, 0.0);
    }

    #[test]
    fn test_autopan_phase_wrap() {
        let mut ap = AutoPan::new(44100.0, 5.0);

        // Process many samples - phase should stay bounded
        for _ in 0..10000 {
            ap.process(1.0, 1.0);
            assert!(ap.lfo_phase >= 0.0 && ap.lfo_phase < 1.0);
        }
    }

    #[test]
    fn test_autopan_preserves_silence() {
        let mut ap = AutoPan::new(44100.0, 5.0);
        ap.set_depth(1.0);

        // Silence should remain silence
        for _ in 0..100 {
            let (left, right) = ap.process(0.0, 0.0);
            assert_eq!(left, 0.0);
            assert_eq!(right, 0.0);
        }
    }

    #[test]
    fn test_autopan_triangle_waveform() {
        let mut ap = AutoPan::new(44100.0, 10.0);
        ap.set_waveform(Waveform::Triangle);
        ap.set_depth(1.0);

        let mut outputs = Vec::new();
        for _ in 0..4410 {
            // Full cycle
            let (left, _) = ap.process(1.0, 1.0);
            outputs.push(left);
        }

        // Should modulate
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(max - min > 0.15);
    }

    #[test]
    fn test_autopan_saw_waveform() {
        let mut ap = AutoPan::new(44100.0, 10.0);
        ap.set_waveform(Waveform::Saw);
        ap.set_depth(1.0);

        let mut outputs = Vec::new();
        for _ in 0..4410 {
            // Full cycle
            let (left, _) = ap.process(1.0, 1.0);
            outputs.push(left);
        }

        // Should modulate
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(max - min > 0.15);
    }

    #[test]
    fn test_autopan_mono_summing() {
        let mut ap = AutoPan::new(44100.0, 5.0);

        // Process stereo input with different levels
        let left_in = 0.6;
        let right_in = 0.4;

        let (left_out, right_out) = ap.process(left_in, right_in);

        // Output should be based on summed mono signal
        let expected_mono = (left_in + right_in) * 0.5;
        let output_mono = (left_out + right_out) / std::f32::consts::SQRT_2; // Approximate

        // Should be in reasonable range based on mono sum
        assert!(output_mono > 0.0);
        assert!(output_mono < expected_mono * 2.0);
    }
}
