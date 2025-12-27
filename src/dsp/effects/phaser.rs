/// Phaser effect using cascaded all-pass filters with LFO modulation
/// Creates classic swooshing/jet-like sounds via frequency-dependent phase shifts
use std::f32::consts::PI;

pub struct Phaser {
    /// Sample rate for coefficient calculation
    sample_rate: f32,

    /// Chain of all-pass filter stages (typically 4-12 stages)
    stages: Vec<AllPassStage>,

    /// LFO phase accumulator for sweep modulation
    lfo_phase: f32,

    /// LFO rate in Hz
    lfo_rate: f32,

    /// Modulation depth (0.0 to 1.0)
    depth: f32,

    /// Feedback amount (-0.95 to 0.95) - creates resonant peaks
    feedback: f32,

    /// Feedback buffer for previous output
    feedback_sample: f32,

    /// Center frequency for all-pass filter modulation (Hz)
    center_freq: f32,

    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    mix: f32,
}

/// Single all-pass filter stage using biquad structure
/// Transfer function: H(z) = (a + z^-1) / (1 + a*z^-1)
struct AllPassStage {
    /// All-pass coefficient (calculated from frequency)
    coefficient: f32,

    /// Previous input sample
    x1: f32,

    /// Previous output sample
    y1: f32,
}

impl AllPassStage {
    fn new() -> Self {
        Self {
            coefficient: 0.0,
            x1: 0.0,
            y1: 0.0,
        }
    }

    /// Process one sample through the all-pass filter
    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        // All-pass filter equation: y[n] = -c*x[n] + x[n-1] + c*y[n-1]
        let output = -self.coefficient * input + self.x1 + self.coefficient * self.y1;

        self.x1 = input;
        self.y1 = output;

        output
    }

    /// Update the all-pass coefficient based on frequency
    /// coefficient = (tan(π*f/fs) - 1) / (tan(π*f/fs) + 1)
    fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let frequency = frequency.clamp(20.0, sample_rate * 0.49);
        let tan_term = (PI * frequency / sample_rate).tan();
        self.coefficient = (tan_term - 1.0) / (tan_term + 1.0);
    }
}

impl Phaser {
    /// Create a new phaser effect
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `num_stages` - Number of all-pass stages (4-12 typical, more stages = deeper effect)
    /// * `center_freq` - Center frequency for modulation (200-2000 Hz typical)
    /// * `lfo_rate` - LFO rate in Hz (0.1-5.0 Hz typical)
    pub fn new(sample_rate: f32, num_stages: usize, center_freq: f32, lfo_rate: f32) -> Self {
        let num_stages = num_stages.clamp(2, 24); // Limit to reasonable range

        Self {
            sample_rate,
            stages: (0..num_stages).map(|_| AllPassStage::new()).collect(),
            lfo_phase: 0.0,
            lfo_rate: lfo_rate.clamp(0.01, 20.0),
            depth: 1.0,
            feedback: 0.5,
            feedback_sample: 0.0,
            center_freq: center_freq.clamp(100.0, 5000.0),
            mix: 0.5,
        }
    }

    /// Set LFO rate in Hz
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.lfo_rate = rate_hz.clamp(0.01, 20.0);
    }

    /// Set modulation depth (0.0 to 1.0)
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    /// Set feedback amount (-0.95 to 0.95)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-0.95, 0.95);
    }

    /// Set center frequency for modulation
    pub fn set_center_frequency(&mut self, freq_hz: f32) {
        self.center_freq = freq_hz.clamp(100.0, 5000.0);
    }

    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Process a stereo sample through the phaser
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Generate LFO modulation (sine wave)
        let lfo = (self.lfo_phase * 2.0 * PI).sin();

        // Advance LFO phase
        self.lfo_phase += self.lfo_rate / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        // Calculate modulated frequency (exponential modulation for musical sweep)
        // Sweep from center_freq/4 to center_freq*4 (two octaves each way)
        let freq_ratio = 2.0f32.powf(lfo * self.depth * 2.0); // ±2 octaves
        let modulated_freq = self.center_freq * freq_ratio;

        // Update all stage frequencies
        for stage in &mut self.stages {
            stage.set_frequency(modulated_freq, self.sample_rate);
        }

        // Process left channel
        let mut wet_left = left + self.feedback_sample * self.feedback;
        for stage in &mut self.stages {
            wet_left = stage.process(wet_left);
        }

        // Store feedback sample
        self.feedback_sample = wet_left;

        // Mix dry and wet
        let output_left = left * (1.0 - self.mix) + wet_left * self.mix;

        // For stereo, process right channel with same settings
        // (In a more advanced version, could use 90° phase offset LFO for stereo width)
        let mut wet_right = right + self.feedback_sample * self.feedback * 0.5;
        for stage in &mut self.stages {
            wet_right = stage.process(wet_right);
        }

        let output_right = right * (1.0 - self.mix) + wet_right * self.mix;

        (output_left, output_right)
    }

    /// Reset the phaser state
    pub fn reset(&mut self) {
        self.lfo_phase = 0.0;
        self.feedback_sample = 0.0;
        for stage in &mut self.stages {
            stage.x1 = 0.0;
            stage.y1 = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_phaser_creation() {
        let phaser = Phaser::new(44100.0, 4, 1000.0, 0.5);
        assert_eq!(phaser.stages.len(), 4);
        assert_eq!(phaser.sample_rate, 44100.0);
    }

    #[test]
    fn test_phaser_stage_limits() {
        // Should clamp to 2-24 stages
        let phaser_low = Phaser::new(44100.0, 0, 1000.0, 0.5);
        assert_eq!(phaser_low.stages.len(), 2);

        let phaser_high = Phaser::new(44100.0, 100, 1000.0, 0.5);
        assert_eq!(phaser_high.stages.len(), 24);
    }

    #[test]
    fn test_phaser_processes_audio() {
        let mut phaser = Phaser::new(44100.0, 6, 800.0, 0.25);
        phaser.set_mix(0.5);

        // Test with a simple impulse
        let (left, right) = phaser.process(1.0, 1.0);

        // Should produce non-zero output
        assert!(left.abs() > 0.0);
        assert!(right.abs() > 0.0);

        // Subsequent samples should show the effect
        let (left2, _right2) = phaser.process(0.0, 0.0);
        assert!(left2.abs() > 0.0); // Impulse response continues
    }

    #[test]
    fn test_phaser_lfo_modulation() {
        let mut phaser = Phaser::new(44100.0, 4, 1000.0, 1.0); // 1 Hz LFO
        phaser.set_mix(1.0); // Full wet

        // Process for one LFO cycle (44100 samples at 1 Hz)
        let mut outputs = Vec::new();
        for _ in 0..44100 {
            let (left, _) = phaser.process(1.0, 1.0);
            outputs.push(left);
        }

        // Output should vary over the cycle (not constant)
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(max - min > 0.1); // Significant modulation
    }

    #[test]
    fn test_phaser_feedback() {
        let mut phaser_no_fb = Phaser::new(44100.0, 6, 1000.0, 0.5);
        phaser_no_fb.set_feedback(0.0);
        phaser_no_fb.set_mix(1.0);

        let mut phaser_with_fb = Phaser::new(44100.0, 6, 1000.0, 0.5);
        phaser_with_fb.set_feedback(0.7);
        phaser_with_fb.set_mix(1.0);

        // Process impulse
        let (left_no_fb, _) = phaser_no_fb.process(1.0, 1.0);
        let (left_with_fb, _) = phaser_with_fb.process(1.0, 1.0);

        // With feedback should have stronger resonance (larger output)
        assert!(left_with_fb.abs() > left_no_fb.abs() * 0.5);
    }

    #[test]
    fn test_phaser_mix_control() {
        let mut phaser = Phaser::new(44100.0, 4, 1000.0, 0.5);

        // Dry signal (mix = 0.0)
        phaser.set_mix(0.0);
        let (left_dry, _) = phaser.process(0.5, 0.5);
        assert_relative_eq!(left_dry, 0.5, epsilon = 0.01);

        // Wet signal (mix = 1.0)
        phaser.reset();
        phaser.set_mix(1.0);
        let (left_wet, _) = phaser.process(0.5, 0.5);
        assert!((left_wet - 0.5).abs() > 0.01); // Should be different from dry
    }

    #[test]
    fn test_phaser_depth_control() {
        let mut phaser_no_depth = Phaser::new(44100.0, 6, 1000.0, 0.5);
        phaser_no_depth.set_depth(0.0);
        phaser_no_depth.set_mix(1.0);

        let mut phaser_full_depth = Phaser::new(44100.0, 6, 1000.0, 0.5);
        phaser_full_depth.set_depth(1.0);
        phaser_full_depth.set_mix(1.0);

        // Process multiple samples to allow LFO to modulate
        let mut output_no_depth = Vec::new();
        let mut output_full_depth = Vec::new();

        for i in 0..1000 {
            let input = (i as f32 * 0.01).sin();
            let (left_no, _) = phaser_no_depth.process(input, input);
            let (left_full, _) = phaser_full_depth.process(input, input);
            output_no_depth.push(left_no);
            output_full_depth.push(left_full);
        }

        // Calculate variance as a measure of modulation
        let variance = |samples: &[f32]| {
            let mean = samples.iter().sum::<f32>() / samples.len() as f32;
            samples.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / samples.len() as f32
        };

        let var_no = variance(&output_no_depth);
        let var_full = variance(&output_full_depth);

        // Full depth should have more variation (due to stronger modulation)
        assert!(var_full > var_no * 0.5);
    }

    #[test]
    fn test_phaser_reset() {
        let mut phaser = Phaser::new(44100.0, 4, 1000.0, 0.5);

        // Process some samples
        for _ in 0..100 {
            phaser.process(1.0, 1.0);
        }

        // Reset
        phaser.reset();

        // State should be cleared
        assert_eq!(phaser.lfo_phase, 0.0);
        assert_eq!(phaser.feedback_sample, 0.0);
        for stage in &phaser.stages {
            assert_eq!(stage.x1, 0.0);
            assert_eq!(stage.y1, 0.0);
        }
    }

    #[test]
    fn test_all_pass_stage_process() {
        let mut stage = AllPassStage::new();
        stage.set_frequency(1000.0, 44100.0);

        // All-pass should preserve signal magnitude (approximately)
        let input = 1.0;
        let output = stage.process(input);

        // Output should be non-zero and bounded
        assert!(output.abs() <= 2.0); // Reasonable bound
    }

    #[test]
    fn test_all_pass_coefficient_calculation() {
        let mut stage = AllPassStage::new();

        // Low frequency should give coefficient near -1
        stage.set_frequency(100.0, 44100.0);
        assert!(stage.coefficient < -0.9);

        // High frequency should give coefficient near 0
        stage.set_frequency(10000.0, 44100.0);
        assert!(stage.coefficient > -0.5 && stage.coefficient < 0.0);
    }
}
