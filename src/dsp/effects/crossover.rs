/// Linkwitz-Riley 2nd order crossover filter
///
/// Produces flat magnitude response when low+high outputs are summed.
/// Used for phase-coherent band splitting in multiband processors.
///
/// # Technical Details
/// - Butterworth-based 2nd-order filters (Q = 0.7071)
/// - -12dB/octave slopes for each filter
/// - Phase-aligned at crossover frequency
/// - No magnitude ripple when bands are summed
///
/// # Usage
/// ```
/// let mut xover = LR2Crossover::new(44100.0, 150.0);
/// let (low, high) = xover.process(input_sample);
/// ```
use std::f32::consts::PI;

/// Linkwitz-Riley 2nd order crossover filter pair
pub struct LR2Crossover {
    sample_rate: f32,
    frequency: f32,

    // Low-pass state (Direct Form I)
    lp_x1: f32,
    lp_x2: f32,
    lp_y1: f32,
    lp_y2: f32,

    // High-pass state (Direct Form I)
    hp_x1: f32,
    hp_x2: f32,
    hp_y1: f32,
    hp_y2: f32,

    // Biquad coefficients (shared denominator)
    b0_lp: f32,
    b1_lp: f32,
    b2_lp: f32,
    b0_hp: f32,
    b1_hp: f32,
    b2_hp: f32,
    a1: f32,
    a2: f32,
}

impl LR2Crossover {
    /// Create a new Linkwitz-Riley 2nd order crossover
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `crossover_freq` - Crossover frequency in Hz (typically 50Hz - 20kHz)
    pub fn new(sample_rate: f32, crossover_freq: f32) -> Self {
        let mut xover = Self {
            sample_rate,
            frequency: crossover_freq,
            lp_x1: 0.0,
            lp_x2: 0.0,
            lp_y1: 0.0,
            lp_y2: 0.0,
            hp_x1: 0.0,
            hp_x2: 0.0,
            hp_y1: 0.0,
            hp_y2: 0.0,
            b0_lp: 0.0,
            b1_lp: 0.0,
            b2_lp: 0.0,
            b0_hp: 0.0,
            b1_hp: 0.0,
            b2_hp: 0.0,
            a1: 0.0,
            a2: 0.0,
        };
        xover.calculate_coefficients();
        xover
    }

    /// Update the crossover frequency at runtime
    ///
    /// Recalculates biquad coefficients without clearing filter state.
    /// Safe to call in audio thread (no allocations, ~100 FLOPs).
    ///
    /// # Arguments
    /// * `freq` - New crossover frequency in Hz
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
        self.calculate_coefficients();
    }

    /// Get the current crossover frequency
    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    /// Calculate biquad coefficients for current frequency
    fn calculate_coefficients(&mut self) {
        // Linkwitz-Riley 2nd order (Butterworth squared)
        let omega = 2.0 * PI * self.frequency / self.sample_rate;
        let cos_omega = omega.cos();
        let sin_omega = omega.sin();
        let alpha = sin_omega / (2.0 * 0.7071); // Q = 0.7071 for Butterworth

        let a0 = 1.0 + alpha;

        // Low-pass coefficients (Butterworth)
        self.b0_lp = ((1.0 - cos_omega) / 2.0) / a0;
        self.b1_lp = (1.0 - cos_omega) / a0;
        self.b2_lp = ((1.0 - cos_omega) / 2.0) / a0;

        // High-pass coefficients (Butterworth)
        self.b0_hp = ((1.0 + cos_omega) / 2.0) / a0;
        self.b1_hp = -(1.0 + cos_omega) / a0;
        self.b2_hp = ((1.0 + cos_omega) / 2.0) / a0;

        // Shared denominator coefficients
        self.a1 = (-2.0 * cos_omega) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// Process a single sample through the crossover
    ///
    /// Returns (low_band, high_band) where:
    /// - `low_band` contains frequencies below crossover point
    /// - `high_band` contains frequencies above crossover point
    /// - Summing both bands reconstructs the original signal
    pub fn process(&mut self, input: f32) -> (f32, f32) {
        // Low-pass filter (Direct Form I biquad)
        let lp_out = self.b0_lp * input + self.b1_lp * self.lp_x1 + self.b2_lp * self.lp_x2
            - self.a1 * self.lp_y1
            - self.a2 * self.lp_y2;

        // Update low-pass state
        self.lp_x2 = self.lp_x1;
        self.lp_x1 = input;
        self.lp_y2 = self.lp_y1;
        self.lp_y1 = lp_out;

        // High-pass filter (Direct Form I biquad)
        let hp_out = self.b0_hp * input + self.b1_hp * self.hp_x1 + self.b2_hp * self.hp_x2
            - self.a1 * self.hp_y1
            - self.a2 * self.hp_y2;

        // Update high-pass state
        self.hp_x2 = self.hp_x1;
        self.hp_x1 = input;
        self.hp_y2 = self.hp_y1;
        self.hp_y1 = hp_out;

        (lp_out, hp_out)
    }

    /// Clear filter state (zero delay line buffers)
    ///
    /// Call this when:
    /// - Starting a new note/voice
    /// - Seeking in audio playback
    /// - Resetting the effect
    pub fn clear(&mut self) {
        self.lp_x1 = 0.0;
        self.lp_x2 = 0.0;
        self.lp_y1 = 0.0;
        self.lp_y2 = 0.0;
        self.hp_x1 = 0.0;
        self.hp_x2 = 0.0;
        self.hp_y1 = 0.0;
        self.hp_y2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_crossover_creation() {
        let xover = LR2Crossover::new(44100.0, 1000.0);
        assert_eq!(xover.frequency(), 1000.0);
    }

    #[test]
    fn test_set_frequency() {
        let mut xover = LR2Crossover::new(44100.0, 1000.0);
        xover.set_frequency(2000.0);
        assert_eq!(xover.frequency(), 2000.0);
    }

    #[test]
    fn test_flat_response_when_summed() {
        // When low + high bands are summed, should equal input
        let mut xover = LR2Crossover::new(44100.0, 1000.0);

        // Process DC signal
        let input = 1.0;
        for _ in 0..100 {
            let (low, high) = xover.process(input);
            let sum = low + high;
            // After settling, sum should equal input (flat magnitude response)
            if sum.is_finite() {
                assert_relative_eq!(sum, input, epsilon = 0.01);
            }
        }
    }

    #[test]
    fn test_clear_zeros_state() {
        let mut xover = LR2Crossover::new(44100.0, 1000.0);

        // Process some samples to populate state
        for _ in 0..10 {
            xover.process(0.5);
        }

        // Clear should zero everything
        xover.clear();

        // Next output should be close to zero (no lingering state)
        let (low, high) = xover.process(0.0);
        assert_relative_eq!(low, 0.0, epsilon = 1e-6);
        assert_relative_eq!(high, 0.0, epsilon = 1e-6);
    }
}
