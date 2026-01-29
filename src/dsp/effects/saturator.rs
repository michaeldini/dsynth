use crate::dsp::signal_analyzer::SignalAnalysis;

/// Audio saturator for harmonic enhancement and warmth
///
/// Applies controlled nonlinear distortion using hyperbolic tangent waveshaping
/// to add harmonic content and musical saturation. Includes configurable drive
/// and dry/wet mixing for transparent to aggressive saturation effects.
pub struct Saturator {
    #[allow(dead_code)]
    sample_rate: f32,
}

impl Saturator {
    /// Create a new saturator instance
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `_use_emphasis` - Reserved for future pre/de-emphasis filtering (currently unused)
    pub fn new(sample_rate: f32, _use_emphasis: bool) -> Self {
        Self { sample_rate }
    }

    /// Process a single audio sample through the saturator
    ///
    /// # Arguments
    /// * `input` - Input audio sample (-1.0 to 1.0)
    /// * `drive` - Saturation drive amount (0.0 to 1.0)
    /// * `mix` - Dry/wet mix (0.0 = dry, 1.0 = wet)
    /// * `_analysis` - Signal analysis data (reserved for future adaptive behavior)
    ///
    /// # Returns
    /// * Processed audio sample with saturation applied
    pub fn process(&mut self, input: f32, drive: f32, mix: f32, _analysis: &SignalAnalysis) -> f32 {
        // Apply variable gain based on drive parameter (1x to 3x)
        let gain = 1.0 + drive * 2.0;

        // Hyperbolic tangent waveshaping with output scaling
        // tanh() naturally compresses signals above ±1, creating soft clipping
        let saturated = (input * gain).tanh() * 0.8;

        // Linear crossfade between dry and saturated signals
        input * (1.0 - mix) + saturated * mix
    }

    /// Reset internal state
    ///
    /// Currently no internal state to reset, but provided for consistency
    /// with other DSP modules and future stateful implementations.
    pub fn reset(&mut self) {
        // No internal state in current implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_analysis() -> SignalAnalysis {
        SignalAnalysis {
            rms_level: 0.3,
            peak_level: 0.5,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 200.0,
            signal_type: crate::dsp::analysis::SignalType::Tonal,
            is_voiced: true,
            is_unvoiced: false,
            has_sibilance: false,
            sibilance_strength: 0.0,
            pitch_hz: 220.0,
            pitch_confidence: 0.8,
            is_pitched: true,
        }
    }

    #[test]
    fn test_saturator_creation() {
        let saturator = Saturator::new(44100.0, false);
        assert_eq!(saturator.sample_rate, 44100.0);
    }

    #[test]
    fn test_bypass_when_mix_zero() {
        let mut saturator = Saturator::new(44100.0, false);
        let analysis = create_test_analysis();

        let input = 0.5;
        let output = saturator.process(input, 1.0, 0.0, &analysis);

        assert_eq!(output, input, "Should pass through unchanged when mix is 0");
    }

    #[test]
    fn test_saturation_with_drive() {
        let mut saturator = Saturator::new(44100.0, false);
        let analysis = create_test_analysis();

        let input = 0.5;
        let output = saturator.process(input, 1.0, 1.0, &analysis);

        assert!(output.is_finite());
        assert!(
            output != input,
            "Should modify signal when drive and mix are applied"
        );
    }

    #[test]
    fn test_dry_wet_mix() {
        let mut saturator = Saturator::new(44100.0, false);
        let analysis = create_test_analysis();

        let input = 0.5;

        let dry = saturator.process(input, 1.0, 0.0, &analysis);
        saturator.reset();
        let wet = saturator.process(input, 1.0, 1.0, &analysis);
        saturator.reset();
        let mixed = saturator.process(input, 1.0, 0.5, &analysis);

        assert_eq!(dry, input);
        assert!(wet != input);
        assert!(mixed != dry);
        assert!(mixed != wet);
    }

    #[test]
    fn test_produces_finite_output() {
        let mut saturator = Saturator::new(44100.0, false);
        let analysis = create_test_analysis();

        let test_values = [-1.0, -0.5, 0.0, 0.5, 1.0];

        for &input in &test_values {
            let output = saturator.process(input, 0.8, 0.6, &analysis);
            assert!(
                output.is_finite(),
                "Output should be finite for input {}",
                input
            );
        }
    }

    #[test]
    fn test_reset_clears_state() {
        let mut saturator = Saturator::new(44100.0, false);
        let analysis = create_test_analysis();

        // Process some audio to potentially build up state
        for _ in 0..100 {
            saturator.process(0.5, 0.8, 0.6, &analysis);
        }

        // Reset should clear any internal state
        saturator.reset();

        // After reset, behavior should be consistent
        let output1 = saturator.process(0.5, 0.8, 0.6, &analysis);
        saturator.reset();
        let output2 = saturator.process(0.5, 0.8, 0.6, &analysis);

        assert!(
            (output1 - output2).abs() < 0.001,
            "Reset should produce consistent behavior"
        );
    }
}
