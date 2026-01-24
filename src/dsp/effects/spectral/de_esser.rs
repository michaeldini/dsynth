/// De-Esser - Intelligent Sibilance Reduction
///
/// **Simplified de-esser that uses SignalAnalysis for detection**
///
/// Instead of running its own sibilance detection, this module relies on the
/// unified SignalAnalyzer to detect harsh "s", "t", "ch" sounds. This is more
/// efficient and consistent with the "analyze once, use everywhere" architecture.
///
/// # How It Works
/// 1. SignalAnalyzer detects sibilance strength (0.0-1.0)
/// 2. When sibilance exceeds threshold, reduce gain
/// 3. Reduction amount is proportional to sibilance strength
/// 4. Smooth gain changes to avoid artifacts
///
/// # Parameters
/// - `threshold` (0.0-1.0): Minimum sibilance strength to trigger de-essing
/// - `amount` (0.0-12.0 dB): Maximum gain reduction to apply
use crate::dsp::signal_analyzer::SignalAnalysis;

pub struct DeEsser {
    sample_rate: f32,

    /// Minimum sibilance strength (0.0-1.0) to trigger de-essing
    /// Default: 0.5 (moderate sensitivity)
    threshold: f32,

    /// Maximum gain reduction in dB (0.0-12.0)
    /// Default: 6.0 dB
    amount_db: f32,

    /// Smoothed gain reduction (prevents clicks)
    current_reduction: f32,

    /// Smoothing coefficient for gain changes
    smoothing: f32,
}

impl DeEsser {
    /// Create a new de-esser
    pub fn new(sample_rate: f32) -> Self {
        // Smoothing time constant: ~5ms (fast enough to catch sibilance, slow enough to avoid clicks)
        let smoothing_ms = 5.0;
        let smoothing = (-1.0 / (smoothing_ms * 0.001 * sample_rate)).exp();

        Self {
            sample_rate,
            threshold: 0.5,
            amount_db: 6.0,
            current_reduction: 1.0, // 1.0 = no reduction (linear gain)
            smoothing,
        }
    }

    /// Set sibilance detection threshold (0.0-1.0)
    /// Lower = more aggressive (catches weaker sibilance)
    /// Higher = less aggressive (only catches strong sibilance)
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Set maximum reduction amount in dB (0.0-12.0)
    pub fn set_amount(&mut self, amount_db: f32) {
        self.amount_db = amount_db.clamp(0.0, 12.0);
    }

    /// Process stereo audio with intelligent de-essing
    ///
    /// Uses sibilance strength from SignalAnalysis to dynamically reduce gain.
    /// Reduction is proportional to how much sibilance exceeds the threshold.
    ///
    /// # Arguments
    /// * `left` - Left channel input
    /// * `right` - Right channel input
    /// * `analysis` - Signal analysis data (contains sibilance_strength)
    ///
    /// # Returns
    /// Tuple of (left_out, right_out)
    pub fn process(&mut self, left: f32, right: f32, analysis: &SignalAnalysis) -> (f32, f32) {
        // Calculate how much sibilance exceeds threshold (0.0-1.0)
        let sibilance_excess = (analysis.sibilance_strength - self.threshold).max(0.0);
        let sibilance_factor = (sibilance_excess / (1.0 - self.threshold)).clamp(0.0, 1.0);

        // Calculate target gain reduction (linear, not dB)
        // sibilance_factor = 0.0 → no reduction (gain = 1.0)
        // sibilance_factor = 1.0 → full reduction (gain = 10^(-amount_db/20))
        let target_reduction_db = sibilance_factor * self.amount_db;
        let target_reduction_linear = Self::db_to_linear(-target_reduction_db);

        // Smooth the reduction to avoid clicks
        self.current_reduction = self.current_reduction * self.smoothing
            + target_reduction_linear * (1.0 - self.smoothing);

        // Apply reduction to both channels
        let left_out = left * self.current_reduction;
        let right_out = right * self.current_reduction;

        (left_out, right_out)
    }

    /// Reset internal state
    pub fn reset(&mut self) {
        self.current_reduction = 1.0;
    }

    /// Convert dB to linear gain
    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Get current gain reduction in dB (for metering)
    pub fn get_current_reduction_db(&self) -> f32 {
        -20.0 * self.current_reduction.log10().max(-60.0) // Clamp to -60dB floor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn make_analysis(sibilance: f32) -> SignalAnalysis {
        SignalAnalysis {
            is_transient: false,
            is_voiced: false,
            is_pitched: false,
            has_sibilance: sibilance > 0.5,
            rms_level: 0.1,
            peak_level: 0.2,
            transient_strength: 0.0,
            sibilance_strength: sibilance,
            zcr_hz: 1000.0,
            pitch_hz: 0.0,
            pitch_confidence: 0.0,
            ..Default::default()
        }
    }

    #[test]
    fn test_deesser_creation() {
        let deesser = DeEsser::new(44100.0);
        assert_eq!(deesser.sample_rate, 44100.0);
        assert_eq!(deesser.threshold, 0.5);
        assert_eq!(deesser.amount_db, 6.0);
    }

    #[test]
    fn test_no_reduction_below_threshold() {
        let mut deesser = DeEsser::new(44100.0);
        deesser.set_threshold(0.6);

        let analysis = make_analysis(0.3); // Below threshold
        let input = 1.0;

        // Process several samples to let smoothing settle
        for _ in 0..100 {
            deesser.process(input, input, &analysis);
        }

        let (left, right) = deesser.process(input, input, &analysis);

        // Should have minimal reduction (close to input)
        assert!((left - input).abs() < 0.1, "Should not reduce below threshold");
        assert!((right - input).abs() < 0.1, "Should not reduce below threshold");
    }

    #[test]
    fn test_reduction_above_threshold() {
        let mut deesser = DeEsser::new(44100.0);
        deesser.set_threshold(0.5);
        deesser.set_amount(6.0); // 6dB reduction

        let analysis = make_analysis(1.0); // Full sibilance
        let input = 1.0;

        // Process several samples to let smoothing settle
        for _ in 0..1000 {
            deesser.process(input, input, &analysis);
        }

        let (left, right) = deesser.process(input, input, &analysis);

        // Should have significant reduction
        assert!(left < input * 0.7, "Should reduce sibilance above threshold");
        assert!(right < input * 0.7, "Should reduce sibilance above threshold");
        assert!(left > 0.0, "Should not completely mute");
    }

    #[test]
    fn test_proportional_reduction() {
        let mut deesser = DeEsser::new(44100.0);
        deesser.set_threshold(0.5);
        deesser.set_amount(12.0);

        // Test moderate sibilance
        let analysis_moderate = make_analysis(0.7);
        for _ in 0..1000 {
            deesser.process(1.0, 1.0, &analysis_moderate);
        }
        let (moderate_left, _) = deesser.process(1.0, 1.0, &analysis_moderate);

        deesser.reset();

        // Test strong sibilance
        let analysis_strong = make_analysis(1.0);
        for _ in 0..1000 {
            deesser.process(1.0, 1.0, &analysis_strong);
        }
        let (strong_left, _) = deesser.process(1.0, 1.0, &analysis_strong);

        // Strong sibilance should have more reduction than moderate
        assert!(
            strong_left < moderate_left,
            "Stronger sibilance should have more reduction"
        );
    }

    #[test]
    fn test_stereo_processing() {
        let mut deesser = DeEsser::new(44100.0);
        let analysis = make_analysis(0.8);

        let (left, right) = deesser.process(0.5, -0.5, &analysis);

        // Both channels should be reduced by same factor
        assert_relative_eq!(left.abs(), right.abs(), epsilon = 0.001);
    }

    #[test]
    fn test_reset() {
        let mut deesser = DeEsser::new(44100.0);
        let analysis = make_analysis(1.0);

        // Process to build up reduction
        for _ in 0..1000 {
            deesser.process(1.0, 1.0, &analysis);
        }

        // Should have reduction active
        assert!(deesser.current_reduction < 0.9);

        // Reset
        deesser.reset();

        // Should be back to no reduction
        assert_eq!(deesser.current_reduction, 1.0);
    }

    #[test]
    fn test_amount_parameter() {
        let mut deesser = DeEsser::new(44100.0);
        deesser.set_threshold(0.5);

        let analysis = make_analysis(1.0);

        // Test with 3dB reduction
        deesser.set_amount(3.0);
        for _ in 0..1000 {
            deesser.process(1.0, 1.0, &analysis);
        }
        let (light_left, _) = deesser.process(1.0, 1.0, &analysis);

        deesser.reset();

        // Test with 12dB reduction
        deesser.set_amount(12.0);
        for _ in 0..1000 {
            deesser.process(1.0, 1.0, &analysis);
        }
        let (heavy_left, _) = deesser.process(1.0, 1.0, &analysis);

        // Higher amount should reduce more
        assert!(
            heavy_left < light_left,
            "Higher amount should reduce more"
        );
    }

    #[test]
    fn test_threshold_parameter() {
        let mut deesser = DeEsser::new(44100.0);
        deesser.set_amount(6.0);

        let analysis = make_analysis(0.7);

        // Low threshold (more aggressive)
        deesser.set_threshold(0.3);
        for _ in 0..1000 {
            deesser.process(1.0, 1.0, &analysis);
        }
        let (aggressive_left, _) = deesser.process(1.0, 1.0, &analysis);

        deesser.reset();

        // High threshold (less aggressive)
        deesser.set_threshold(0.8);
        for _ in 0..1000 {
            deesser.process(1.0, 1.0, &analysis);
        }
        let (gentle_left, _) = deesser.process(1.0, 1.0, &analysis);

        // Lower threshold should reduce more
        assert!(
            aggressive_left < gentle_left,
            "Lower threshold should reduce more"
        );
    }
}
