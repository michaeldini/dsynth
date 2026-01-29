/// Professional crossover filters for audio frequency splitting
///
/// Provides both single-point and multiband crossover implementations
/// using simple RC filter mathematics for guaranteed unity gain reconstruction.

/// Single crossover point using simple RC filter mathematics
///
/// Implements a first-order RC lowpass filter with perfect complementary highpass.
/// The highpass output is computed as `input - lowpass`, ensuring mathematically
/// perfect reconstruction when both outputs are summed.
pub struct SingleCrossover {
    lp_state: f32,
    alpha: f32,
}

impl SingleCrossover {
    /// Create a new single crossover at the specified frequency
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `frequency` - Crossover frequency in Hz
    pub fn new(sample_rate: f32, frequency: f32) -> Self {
        // RC filter coefficient: alpha = dt / (RC + dt)
        let dt = 1.0 / sample_rate;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * frequency);
        let alpha = dt / (rc + dt);

        Self {
            lp_state: 0.0,
            alpha,
        }
    }

    /// Process sample, returns (lowpass_out, highpass_out)
    ///
    /// Guaranteed: lowpass + highpass = input (perfect reconstruction)
    pub fn process(&mut self, input: f32) -> (f32, f32) {
        // Simple 1-pole lowpass: y[n] = alpha * x[n] + (1-alpha) * y[n-1]
        self.lp_state = self.alpha * input + (1.0 - self.alpha) * self.lp_state;
        let lowpass = self.lp_state;

        // Perfect complement: highpass = input - lowpass
        let highpass = input - lowpass;

        (lowpass, highpass)
    }

    /// Reset filter state to zero
    pub fn reset(&mut self) {
        self.lp_state = 0.0;
    }
}

/// 4-band multiband crossover with fixed frequencies optimized for vocals
///
/// Uses cascading architecture where each crossover point produces complementary
/// LP and HP outputs that sum to the input. This guarantees unity gain when
/// all bands are summed together.
///
/// # Frequency Bands
/// - Bass: DC → 200Hz (fundamental vocal energy)
/// - Mid: 200Hz → 1kHz (vowel formants)  
/// - Presence: 1kHz → 8kHz (consonants and clarity)
/// - Air: 8kHz → Nyquist (breath and shimmer)
pub struct MultibandCrossover {
    // Three crossover points, cascaded using simple RC filters:
    // Input → xover_200 → (bass=LP, HP→xover_1k → (mids=LP, HP→xover_8k → (presence=LP, air=HP)))
    xover_200: SingleCrossover, // Split: bass vs everything else
    xover_1k: SingleCrossover,  // Split: mids vs high content
    xover_8k: SingleCrossover,  // Split: presence vs air
}

impl MultibandCrossover {
    /// Create a new 4-band crossover with fixed frequencies optimized for vocals
    pub fn new(sample_rate: f32) -> Self {
        Self {
            xover_200: SingleCrossover::new(sample_rate, 200.0),
            xover_1k: SingleCrossover::new(sample_rate, 1000.0),
            xover_8k: SingleCrossover::new(sample_rate, 8000.0),
        }
    }

    /// Process sample into 4 frequency bands
    ///
    /// # Returns
    /// Tuple of (bass, mid, presence, air) where each band contains
    /// the frequency content within its designated range.
    ///
    /// Guaranteed: bass + mid + presence + air = input
    #[inline]
    pub fn process(&mut self, input: f32) -> (f32, f32, f32, f32) {
        // Cascading crossover architecture
        let (bass, above_200) = self.xover_200.process(input);
        let (mid, above_1k) = self.xover_1k.process(above_200);
        let (presence, air) = self.xover_8k.process(above_1k);

        (bass, mid, presence, air)
    }

    /// Reset all filter states
    pub fn reset(&mut self) {
        self.xover_200.reset();
        self.xover_1k.reset();
        self.xover_8k.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_crossover_reconstruction() {
        let mut crossover = SingleCrossover::new(44100.0, 1000.0);

        // Test unity reconstruction over various inputs
        let test_values = [0.0, 0.5, -0.3, 1.0, -1.0];

        for &input in &test_values {
            let (low, high) = crossover.process(input);
            let reconstructed = low + high;

            // Should reconstruct perfectly (within floating point precision)
            assert!(
                (reconstructed - input).abs() < 1e-6,
                "Reconstruction failed: input={}, low={}, high={}, sum={}",
                input,
                low,
                high,
                reconstructed
            );
        }
    }

    #[test]
    fn test_multiband_reconstruction() {
        let mut crossover = MultibandCrossover::new(44100.0);

        // Test with various input signals
        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let input = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;

            let (bass, mid, presence, air) = crossover.process(input);
            let reconstructed = bass + mid + presence + air;

            // Should reconstruct perfectly
            assert!(
                (reconstructed - input).abs() < 1e-5,
                "Multiband reconstruction failed at sample {}: input={}, sum={}",
                i,
                input,
                reconstructed
            );
        }
    }

    #[test]
    fn test_frequency_separation() {
        let sample_rate = 44100.0;
        let mut crossover = MultibandCrossover::new(sample_rate);

        // Test DC (should go to bass after settling)
        crossover.reset();
        // Process DC for settling time
        for _ in 0..100 {
            crossover.process(1.0);
        }
        let (bass, _, _, _) = crossover.process(1.0);
        assert!(bass > 0.5, "DC should be in bass band after settling");

        // Test high frequency (should go mostly to air after settling)
        crossover.reset();
        for i in 0..1000 {
            // Let filters settle
            let t = i as f32 / sample_rate;
            let high_freq = (2.0 * std::f32::consts::PI * 10000.0 * t).sin();
            crossover.process(high_freq);
        }

        let t = 1000.0 / sample_rate;
        let high_freq = (2.0 * std::f32::consts::PI * 10000.0 * t).sin();
        let (bass, mid, presence, air) = crossover.process(high_freq);

        // 10kHz should be primarily in air band
        assert!(
            air.abs() > bass.abs() && air.abs() > mid.abs(),
            "10kHz should be strongest in air band: bass={}, mid={}, presence={}, air={}",
            bass,
            mid,
            presence,
            air
        );
    }
}
