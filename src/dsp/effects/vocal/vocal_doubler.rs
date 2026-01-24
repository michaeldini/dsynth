//! Smart Vocal Doubler - Intelligent doubling that adapts to signal content
//!
//! Creates natural vocal doubling effect by adding delayed/detuned copies
//! that adapt based on signal characteristics:
//! - **Transients**: Minimal doubling (3ms, 20% mix) - preserves attack punch
//! - **Sibilance**: Light doubling (5ms, 40% mix) - avoids harsh "s" sounds
//! - **Pitched vocals**: Full doubling (12ms, 90% mix) - maximum thickness
//! - **Unvoiced**: Moderate doubling (8ms, 60% mix) - balanced
//!
//! Parameters are simplified to just 2 controls for user:
//! - **amount**: Overall doubling intensity (0.0-1.0)
//! - **stereo_width**: How much to spread doubling left/right (0.0-1.0)

use crate::dsp::SignalAnalysis;

const MAX_DELAY_MS: f32 = 50.0; // Maximum delay time (increased for audible doubling)
const SMOOTHING_MS: f32 = 10.0; // Smooth parameter changes over 10ms

/// Smart vocal doubler that adapts to signal content
pub struct VocalDoubler {
    sample_rate: f32,

    // User parameters
    amount: f32,       // 0.0-1.0 overall intensity
    stereo_width: f32, // 0.0-1.0 stereo spread

    // Adaptive state (smoothed)
    current_delay_ms: f32,
    current_mix: f32,

    // Delay buffers (stereo)
    delay_buffer_left: Vec<f32>,
    delay_buffer_right: Vec<f32>,
    write_pos: usize,

    // Smoothing
    delay_smoother: f32,
    mix_smoother: f32,
    smoothing_coeff: f32,
}

impl VocalDoubler {
    pub fn new(sample_rate: f32) -> Self {
        let max_delay_samples = ((MAX_DELAY_MS / 1000.0) * sample_rate).ceil() as usize;
        let smoothing_coeff = Self::calculate_smoothing_coeff(sample_rate, SMOOTHING_MS);

        Self {
            sample_rate,
            amount: 0.8,
            stereo_width: 0.7,
            current_delay_ms: 10.0,
            current_mix: 0.5,
            delay_buffer_left: vec![0.0; max_delay_samples],
            delay_buffer_right: vec![0.0; max_delay_samples],
            write_pos: 0,
            delay_smoother: 10.0,
            mix_smoother: 0.5,
            smoothing_coeff,
        }
    }

    fn calculate_smoothing_coeff(sample_rate: f32, time_ms: f32) -> f32 {
        (-1.0 / (sample_rate * time_ms / 1000.0)).exp()
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
    }

    pub fn set_stereo_width(&mut self, width: f32) {
        self.stereo_width = width.clamp(0.0, 1.0);
    }

    /// Process stereo sample with intelligent adaptation
    pub fn process(
        &mut self,
        left_in: f32,
        right_in: f32,
        analysis: &SignalAnalysis,
    ) -> (f32, f32) {
        // Determine target delay/mix based on signal content
        let (target_delay_ms, target_mix) = if analysis.is_transient {
            // Transients: minimal doubling to preserve attack
            (15.0, 0.3)
        } else if analysis.has_sibilance {
            // Sibilance: light doubling to avoid harsh "s"
            (20.0, 0.5)
        } else if analysis.is_pitched {
            // Pitched vocals: full doubling for thickness
            (35.0, 1.0)
        } else {
            // Unvoiced/other: moderate doubling
            (25.0, 0.7)
        };

        // Smooth the target values to prevent clicks
        self.delay_smoother +=
            (target_delay_ms - self.delay_smoother) * (1.0 - self.smoothing_coeff);
        self.mix_smoother += (target_mix - self.mix_smoother) * (1.0 - self.smoothing_coeff);

        // Scale by user amount parameter
        let final_mix = self.mix_smoother * self.amount;

        // Calculate delay in samples
        let delay_samples = ((self.delay_smoother / 1000.0) * self.sample_rate) as usize;
        let delay_samples = delay_samples.clamp(1, self.delay_buffer_left.len() - 1);

        // Write input to buffers
        self.delay_buffer_left[self.write_pos] = left_in;
        self.delay_buffer_right[self.write_pos] = right_in;

        // Calculate read position
        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            self.delay_buffer_left.len() + self.write_pos - delay_samples
        };

        // Read delayed samples
        let delayed_left = self.delay_buffer_left[read_pos];
        let delayed_right = self.delay_buffer_right[read_pos];

        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.delay_buffer_left.len();

        // Apply stereo width: swap delayed samples for stereo effect
        let left_delayed =
            delayed_left * (1.0 - self.stereo_width) + delayed_right * self.stereo_width;
        let right_delayed =
            delayed_right * (1.0 - self.stereo_width) + delayed_left * self.stereo_width;

        // Mix dry and wet
        let left_out = left_in * (1.0 - final_mix) + left_delayed * final_mix;
        let right_out = right_in * (1.0 - final_mix) + right_delayed * final_mix;

        (left_out, right_out)
    }

    pub fn reset(&mut self) {
        self.delay_buffer_left.fill(0.0);
        self.delay_buffer_right.fill(0.0);
        self.write_pos = 0;
        self.delay_smoother = 10.0;
        self.mix_smoother = 0.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let doubler = VocalDoubler::new(44100.0);
        assert_eq!(doubler.sample_rate, 44100.0);
        assert_eq!(doubler.amount, 0.8);
        assert_eq!(doubler.stereo_width, 0.7);
    }

    #[test]
    fn test_transient_minimal_doubling() {
        let mut doubler = VocalDoubler::new(44100.0);
        doubler.set_amount(1.0); // Full amount to test adaptation

        let analysis = SignalAnalysis {
            is_transient: true,
            transient_strength: 1.0,
            ..Default::default()
        };

        let input = 1.0;
        // Process to let smoothing settle towards transient values (3ms, 20%)
        for _ in 0..500 {
            doubler.process(input, input, &analysis);
        }

        let (left, right) = doubler.process(input, input, &analysis);

        // Transients should have minimal doubling (target mix ~0.2)
        // With full amount, we expect close to 0.2 mix
        assert!(left > 0.7, "Transients should preserve most of dry signal");
        assert!(right > 0.7, "Transients should preserve most of dry signal");
    }

    #[test]
    fn test_pitched_full_doubling() {
        let mut doubler = VocalDoubler::new(44100.0);
        doubler.set_amount(1.0); // Full amount

        let analysis = SignalAnalysis {
            is_pitched: true,
            pitch_hz: 200.0,
            pitch_confidence: 0.9,
            ..Default::default()
        };

        // Feed varying signal to fill buffer and see delay effect
        for i in 0..1000 {
            let input = (i as f32 * 0.1).sin();
            doubler.process(input, input, &analysis);
        }

        // Now feed constant 1.0 to see the doubling mix effect
        let input = 1.0;
        let (left, right) = doubler.process(input, input, &analysis);

        // Pitched content should have strong doubling (target mix ~0.9)
        // With varying buffer history, output should differ from current input
        // The delay buffer contains old values, so mixing with current input creates effect
        assert!(
            (left - input).abs() > 0.01,
            "Pitched content should show doubling effect"
        );
        assert!(
            (right - input).abs() > 0.01,
            "Pitched content should show doubling effect"
        );
    }

    #[test]
    fn test_sibilance_light_doubling() {
        let mut doubler = VocalDoubler::new(44100.0);
        doubler.set_amount(1.0); // Full amount

        let analysis = SignalAnalysis {
            has_sibilance: true,
            sibilance_strength: 0.8,
            ..Default::default()
        };

        let input = 1.0;
        // Process to let smoothing settle towards sibilance values (5ms, 40%)
        for _ in 0..500 {
            doubler.process(input, input, &analysis);
        }

        let (left, right) = doubler.process(input, input, &analysis);

        // Sibilance should have light doubling (target mix ~0.4)
        assert!(left > 0.5, "Sibilance should have moderate dry signal");
        assert!(right > 0.5, "Sibilance should have moderate dry signal");
    }

    #[test]
    fn test_stereo_width() {
        let mut doubler = VocalDoubler::new(44100.0);
        doubler.set_amount(1.0);
        doubler.set_stereo_width(1.0); // Maximum stereo spread

        let analysis = SignalAnalysis {
            is_pitched: true,
            ..Default::default()
        };

        // Feed different L/R inputs to test stereo spreading
        for _ in 0..1000 {
            doubler.process(1.0, 0.5, &analysis);
        }

        let (left, right) = doubler.process(1.0, 0.5, &analysis);

        // With width=1.0, delayed samples are fully swapped
        // Left and right outputs should differ due to stereo spreading
        assert!(
            (left - right).abs() > 0.01,
            "Stereo width should create L/R difference"
        );
    }
}
