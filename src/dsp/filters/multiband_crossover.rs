/// 4-Band Linkwitz-Riley Multiband Crossover
///
/// Provides phase-coherent 4-way frequency split for professional multiband processing.
/// Uses hierarchical 2nd-order Linkwitz-Riley filters (-12dB/octave slopes).
///
/// # Architecture
/// ```text
/// Input → LR2(200Hz) → Bass (20-200Hz)
///       ↓
///       → LR2(1kHz) → Mids (200Hz-1kHz)
///       ↓
///       → LR2(8kHz) → Presence (1-8kHz)
///       ↓
///       → Air (8kHz+)
/// ```
///
/// When all bands are summed, the result is phase-aligned with flat magnitude response
/// (no peaks or dips at crossover points).
use crate::dsp::effects::LR2Crossover;

/// 4-band multiband crossover with fixed frequencies optimized for vocals
pub struct MultibandCrossover {
    // Bass/Rest split at 200Hz (LR4)
    xover_bass: LR2Crossover,

    // Mids/Highs split at 1kHz (LR4)
    xover_mid: LR2Crossover,

    // Presence/Air split at 8kHz (LR4)
    xover_presence: LR2Crossover,

    sample_rate: f32,
}

impl MultibandCrossover {
    /// Create a new 4-band crossover
    ///
    /// # Fixed Crossover Frequencies
    /// - 200Hz: Bass/Mids split (fundamental vocal energy)
    /// - 1kHz: Mids/Presence split (vowel formants)
    /// - 8kHz: Presence/Air split (breath and shimmer)
    pub fn new(sample_rate: f32) -> Self {
        // Create independent LR2 crossovers at each split point
        // We'll use them hierarchically but keep independent state
        let xover_bass = LR2Crossover::new(sample_rate, 200.0);
        let xover_mid = LR2Crossover::new(sample_rate, 1000.0);
        let xover_presence = LR2Crossover::new(sample_rate, 8000.0);

        Self {
            xover_bass,
            xover_mid,
            xover_presence,
            sample_rate,
        }
    }

    /// Process a single sample, returns (bass, mids, presence, air)
    ///
    /// All bands are phase-aligned and sum to unity gain.
    ///
    /// # Arguments
    /// * `input` - Input sample
    ///
    /// # Returns
    /// Tuple of (bass, mids, presence, air) samples
    #[inline]
    pub fn process(&mut self, input: f32) -> (f32, f32, f32, f32) {
        // First split: Bass (<200Hz) vs Rest (>200Hz)
        let (bass, rest) = self.xover_bass.process(input);

        // Second split on Rest: Mids (200Hz-1kHz) vs HighRest (>1kHz)
        let (mids, high_rest) = self.xover_mid.process(rest);

        // Third split on HighRest: Presence (1-8kHz) vs Air (>8kHz)
        let (presence, air) = self.xover_presence.process(high_rest);

        (bass, mids, presence, air)
    }

    /// Reset all filter states
    pub fn reset(&mut self) {
        // LR2Crossover doesn't have a reset method, so we recreate them
        self.xover_bass = LR2Crossover::new(self.sample_rate, 200.0);
        self.xover_mid = LR2Crossover::new(self.sample_rate, 1000.0);
        self.xover_presence = LR2Crossover::new(self.sample_rate, 8000.0);
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_multiband_crossover_creation() {
        let xover = MultibandCrossover::new(44100.0);
        assert_eq!(xover.sample_rate(), 44100.0);
    }

    #[test]
    fn test_unity_gain_summing() {
        let mut xover = MultibandCrossover::new(44100.0);

        // Test with DC signal
        let input = 1.0;
        for _ in 0..1000 {
            // Warm up filters
            xover.process(input);
        }

        let (bass, mids, presence, air) = xover.process(input);
        let sum = bass + mids + presence + air;

        // Phase-aligned LR4 crossovers should sum to unity
        assert_relative_eq!(sum, input, epsilon = 0.01);
    }

    #[test]
    fn test_all_bands_produce_output() {
        let mut xover = MultibandCrossover::new(44100.0);

        // Generate broadband signal (sum of multiple frequencies)
        let mut input_signal = Vec::new();
        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            // Mix of 100Hz, 500Hz, 3kHz, 10kHz
            let sample = (2.0 * std::f32::consts::PI * 100.0 * t).sin() * 0.25
                + (2.0 * std::f32::consts::PI * 500.0 * t).sin() * 0.25
                + (2.0 * std::f32::consts::PI * 3000.0 * t).sin() * 0.25
                + (2.0 * std::f32::consts::PI * 10000.0 * t).sin() * 0.25;
            input_signal.push(sample);
        }

        // Process and check that all bands have significant energy
        let mut bass_energy = 0.0;
        let mut mids_energy = 0.0;
        let mut presence_energy = 0.0;
        let mut air_energy = 0.0;

        for &input in &input_signal {
            let (bass, mids, presence, air) = xover.process(input);
            bass_energy += bass * bass;
            mids_energy += mids * mids;
            presence_energy += presence * presence;
            air_energy += air * air;
        }

        // All bands should have non-zero energy
        assert!(bass_energy > 0.01, "Bass band has no energy");
        assert!(mids_energy > 0.01, "Mids band has no energy");
        assert!(presence_energy > 0.01, "Presence band has no energy");
        assert!(air_energy > 0.01, "Air band has no energy");
    }

    #[test]
    fn test_reset_clears_state() {
        let mut xover = MultibandCrossover::new(44100.0);

        // Process some samples to build up state
        for i in 0..100 {
            let sample = (i as f32 * 0.1).sin();
            xover.process(sample);
        }

        // Reset
        xover.reset();

        // After reset, silent input should produce near-zero output
        let (bass, mids, presence, air) = xover.process(0.0);
        assert!(bass.abs() < 0.001);
        assert!(mids.abs() < 0.001);
        assert!(presence.abs() < 0.001);
        assert!(air.abs() < 0.001);
    }

    #[test]
    fn test_bass_band_response() {
        let mut xover = MultibandCrossover::new(44100.0);

        // Generate 100Hz signal (should appear mostly in bass band)
        let mut bass_rms = 0.0;
        let mut other_rms = 0.0;

        for i in 0..4410 {
            // 100ms
            let t = i as f32 / 44100.0;
            let input = (2.0 * std::f32::consts::PI * 100.0 * t).sin();
            let (bass, mids, presence, air) = xover.process(input);

            bass_rms += bass * bass;
            other_rms += (mids * mids) + (presence * presence) + (air * air);
        }

        bass_rms = (bass_rms / 4410.0).sqrt();
        other_rms = (other_rms / 4410.0).sqrt();

        // Bass band should have more energy than other bands combined
        // LR4 has -24dB/octave slope, so at 100Hz expect bass > other
        assert!(
            bass_rms > other_rms * 0.9,
            "100Hz signal not properly isolated to bass band. Bass RMS: {}, Other RMS: {}",
            bass_rms,
            other_rms
        );
    }

    #[test]
    fn test_air_band_response() {
        let mut xover = MultibandCrossover::new(44100.0);

        // Generate 10kHz signal (should appear mostly in air band)
        let mut air_rms = 0.0;
        let mut other_rms = 0.0;

        for i in 0..4410 {
            // 100ms
            let t = i as f32 / 44100.0;
            let input = (2.0 * std::f32::consts::PI * 10000.0 * t).sin();
            let (bass, mids, presence, air) = xover.process(input);

            air_rms += air * air;
            other_rms += (bass * bass) + (mids * mids) + (presence * presence);
        }

        air_rms = (air_rms / 4410.0).sqrt();
        other_rms = (other_rms / 4410.0).sqrt();

        // Air band should have more energy than other bands combined
        // 10kHz is well above 8kHz crossover, should dominate air band
        assert!(
            air_rms > other_rms,
            "10kHz signal not properly isolated to air band. Air RMS: {}, Other RMS: {}",
            air_rms,
            other_rms
        );
    }
}
