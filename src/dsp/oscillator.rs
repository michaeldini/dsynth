use crate::dsp::downsampler::Downsampler;
use crate::dsp::waveform;
use crate::params::Waveform;

#[cfg(feature = "simd")]
use std::simd::{StdFloat, cmp::SimdPartialOrd, f32x4};

/// A polyphonic-safe oscillator with 4× oversampling and anti-aliasing.
///
/// This oscillator is the core sound generation component of the synthesizer. It produces
/// basic waveforms (sine, square, sawtooth, triangle, pulse) at any frequency. The key
/// design feature is **4× oversampling with anti-aliasing** to prevent aliasing artifacts
/// that would otherwise occur at high frequencies.
///
/// ## Why 4× Oversampling?
///
/// When generating waveforms at high frequencies (e.g., a 10kHz sawtooth wave at 44.1kHz
/// sample rate), aliasing can occur. Aliasing happens because the waveform contains frequency
/// components that are too high for the sample rate to represent accurately (above the
/// Nyquist frequency of 22050Hz). This produces harsh, digital-sounding artifacts.
///
/// The solution: generate the waveform at 4× the target sample rate (176400Hz in this case),
/// then filter it back down to the target rate. This removes the high-frequency content that
/// would alias, producing smooth, analog-sounding oscillators even at high frequencies.
///
/// ## Supported Waveforms
///
/// - **Sine**: Pure, smooth fundamental tone with minimal harmonics
/// - **Square**: Bright, hollow tone with odd-numbered harmonics
/// - **Sawtooth**: Buzzy, bright tone with all harmonics
/// - **Triangle**: Less bright than square, with only odd harmonics but weaker
/// - **Pulse**: Square wave variant with variable pulse width (duty cycle)
///
/// ## Wave Shaping
///
/// The `shape` parameter allows morphing between waveforms or adding harmonic distortion:
/// - For sine: adds harmonic content via soft clipping
/// - For saw/triangle/square: morphs to adjacent waveforms
/// - For pulse: controls the pulse width (duty cycle) instead
///
/// ## Phase-Accurate Frequency Control
///
/// The oscillator uses phase accumulation rather than sample counting. At each step, it
/// adds a phase_increment (which equals frequency / sample_rate) to an accumulating phase
/// value. When phase reaches 1.0, it wraps back to 0.0. This is more accurate than trying
/// to hit exact sample counts and allows smooth frequency changes without clicks.
pub struct Oscillator {
    /// The oversampling sample rate (4× the target sample rate)
    /// For example, if target is 44100Hz, oversample_rate is 176400Hz
    oversample_rate: f32,

    /// Current oscillator phase (0.0 to 1.0)
    /// This represents the position within one complete oscillation cycle.
    /// 0.0 = start of cycle, 0.5 = middle, 1.0 = end (wraps back to 0.0)
    phase: f32,

    /// How much to increment phase each sample (frequency / oversample_rate)
    /// At 440Hz and 176400Hz oversample rate: 440 / 176400 ≈ 0.00249
    /// This means advancing about 0.249% through the waveform cycle per sample
    phase_increment: f32,

    /// Downsampler for filtering 4× oversampled signal back to target rate
    /// This is what prevents aliasing by removing high-frequency content
    downsampler: Downsampler,

    /// Current waveform type (sine, square, saw, etc.)
    waveform: Waveform,

    /// Initial phase offset (0.0 to 1.0) for unison/detune effects
    /// Allows multiple oscillators to start at different phases for richer sound
    initial_phase: f32,

    /// Wave shaping parameter (-1.0 to 1.0)
    /// Controls waveform morphing or harmonic content addition
    shape: f32,
}

impl Oscillator {
    /// Create a new oscillator tuned for a specific sample rate.
    ///
    /// # Arguments
    /// * `sample_rate` - The target sample rate in Hz (e.g., 44100.0)
    ///
    /// # Returns
    /// A new oscillator with:
    /// - Internal oversampling rate set to 4× the target rate
    /// - Phase initialized to 0.0
    /// - Default waveform (Sine)
    /// - No initial phase offset
    /// - No wave shaping
    ///
    /// # Example
    /// ```
    /// use dsynth::dsp::oscillator::Oscillator;
    ///
    /// let osc = Oscillator::new(44100.0);
    /// // osc is now set up for 44.1kHz audio, using 176.4kHz internally
    ///
    /// // Process a sample to verify it works
    /// // (internal state verification would require pub fields)
    /// ```
    pub fn new(sample_rate: f32) -> Self {
        Self {
            oversample_rate: sample_rate * 4.0,
            phase: 0.0,
            phase_increment: 0.0,
            downsampler: Downsampler::new(20),
            waveform: Waveform::Sine,
            initial_phase: 0.0,
            shape: 0.0,
        }
    }

    /// Set the initial phase offset for this oscillator.
    ///
    /// This is useful for creating "unison" effects where multiple detuned oscillators
    /// are stacked with different phases, creating a richer, wider sound. It can also
    /// be used for creative effects where different notes start at different points in
    /// the waveform cycle.
    ///
    /// # Arguments
    /// * `phase` - Phase offset from 0.0 to 1.0
    ///   - 0.0 = start of cycle
    ///   - 0.5 = middle (180° out of phase)
    ///   - 1.0 = full cycle (same as 0.0)
    ///
    /// Values outside [0.0, 1.0] are clamped to stay in range.
    pub fn set_phase(&mut self, phase: f32) {
        self.initial_phase = phase.clamp(0.0, 1.0);
    }

    /// Set the frequency for this oscillator in Hz.
    ///
    /// The frequency determines how fast the phase advances. Higher frequencies advance
    /// the phase faster, producing higher-pitched tones. The phase increment is calculated
    /// as: frequency / oversample_rate.
    ///
    /// # Arguments
    /// * `freq` - Frequency in Hz (e.g., 440.0 for concert A4)
    ///
    /// # Notes
    /// - Can be changed at any time for smooth frequency sweeps
    /// - Changing frequency doesn't reset the oscillator's current phase
    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_increment = freq / self.oversample_rate;
    }

    /// Set the waveform type (sine, square, saw, triangle, or pulse).
    ///
    /// Different waveforms have different harmonic content and tonal characteristics.
    /// Can be changed in real-time without artifacts.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Set the wave shaping amount (-1.0 to 1.0).
    ///
    /// Wave shaping behavior depends on the current waveform:
    /// - **Sine**: Adds harmonic content via soft clipping distortion
    /// - **Sawtooth**: Morphs towards triangle (reduces harshness)
    /// - **Square**: Applied as morphing (though pulse width is primary)
    /// - **Triangle**: Morphs towards sawtooth (adds sharpness)
    /// - **Pulse**: Controls pulse width (duty cycle): -1.0 = 10%, 0.0 = 50%, 1.0 = 90%
    ///
    /// Values outside [-1.0, 1.0] are clamped to stay in range.
    pub fn set_shape(&mut self, shape: f32) {
        self.shape = shape.clamp(-1.0, 1.0);
    }

    /// Apply wave shaping to an SIMD vector of samples (SIMD version).
    ///
    /// This function modulates the waveform based on the shape parameter and current
    /// waveform type. Different waveforms respond to shaping differently:
    ///
    /// - **Sine**: Uses soft clipping (tanh approximation) to add harmonics. The more
    ///   shape is applied, the more the wave is "driven" and clipped, adding overtones
    ///   for a richer, dirtier sound.
    ///
    /// - **Sawtooth**: Morphs towards triangle wave. This reduces the harsh high harmonics
    ///   of a sawtooth, making it sound smoother and more mellow.
    ///
    /// - **Triangle**: Morphs towards sawtooth, adding sharpness and brightness by
    ///   introducing more high-frequency content.
    ///
    /// The shaping is implemented using linear interpolation between the base waveform
    /// and the target waveform/effect.
    #[cfg(feature = "simd")]
    fn apply_wave_shaping(&self, samples: f32x4) -> f32x4 {
        let shape_amount = f32x4::splat(self.shape.abs());
        match self.waveform {
            Waveform::Sine => {
                // Sine: add harmonics via soft clipping/saturation
                // "Drive" the signal harder when shape is applied
                let drive = f32x4::splat(1.0 + self.shape.abs() * 3.0);
                let driven = samples * drive;
                // Soft clip using tanh approximation: tanh(x) ≈ x - x³/3
                // This creates smooth, musical distortion without harsh clipping
                let x2 = driven * driven;
                let x3 = x2 * driven;
                let tanh_approx = driven - x3 / f32x4::splat(3.0);
                tanh_approx / drive.sqrt()
            }
            Waveform::Saw => {
                // Saw: morph towards triangle (removes harsh harmonics)
                let triangle = f32x4::from_array([
                    if self.phase < 0.5 {
                        4.0 * self.phase - 1.0
                    } else {
                        -4.0 * self.phase + 3.0
                    },
                    if self.phase + self.phase_increment < 0.5 {
                        4.0 * (self.phase + self.phase_increment) - 1.0
                    } else {
                        -4.0 * (self.phase + self.phase_increment) + 3.0
                    },
                    if self.phase + 2.0 * self.phase_increment < 0.5 {
                        4.0 * (self.phase + 2.0 * self.phase_increment) - 1.0
                    } else {
                        -4.0 * (self.phase + 2.0 * self.phase_increment) + 3.0
                    },
                    if self.phase + 3.0 * self.phase_increment < 0.5 {
                        4.0 * (self.phase + 3.0 * self.phase_increment) - 1.0
                    } else {
                        -4.0 * (self.phase + 3.0 * self.phase_increment) + 3.0
                    },
                ]);
                samples * (f32x4::splat(1.0) - shape_amount) + triangle * shape_amount
            }
            Waveform::Square => {
                // Square: morph pulse width (but this is handled by Pulse waveform)
                samples
            }
            Waveform::Triangle => {
                // Triangle: add corners/sharpness (morph towards saw)
                let saw = f32x4::from_array([
                    2.0 * self.phase - 1.0,
                    2.0 * (self.phase + self.phase_increment) - 1.0,
                    2.0 * (self.phase + 2.0 * self.phase_increment) - 1.0,
                    2.0 * (self.phase + 3.0 * self.phase_increment) - 1.0,
                ]);
                samples * (f32x4::splat(1.0) - shape_amount) + saw * shape_amount
            }
            Waveform::Pulse => samples, // Pulse uses shape for PWM, not morphing
        }
    }

    /// Generate one output sample (processes 4× oversampled internally) - SIMD optimized version.
    ///
    /// This is the main processing function when the "simd" feature is enabled. It:
    /// 1. Generates 4 oversampled waveform samples simultaneously using SIMD
    /// 2. Applies optional wave shaping if needed
    /// 3. Downsamples the 4 samples back to 1 output sample
    /// 4. Returns the single anti-aliased sample
    ///
    /// ## Optimization: Shape Skipping
    ///
    /// If the shape parameter is very small (< 0.001), the expensive wave shaping calculations
    /// are skipped. This is a crucial optimization because wave shaping involves multiple
    /// multiplications and branching. For most presets where shape isn't heavily used, this
    /// early return provides significant CPU savings.
    ///
    /// ## SIMD Processing
    ///
    /// SIMD (Single Instruction Multiple Data) allows processing 4 float32 samples in parallel
    /// using a single CPU instruction, making oscillator generation roughly 4× faster when
    /// available. If the "simd" feature isn't enabled, a scalar version (below) is used instead.
    ///
    /// ## Phase Wrapping
    ///
    /// After advancing the phase by 4 increments (for the 4 oversampled samples), we check if
    /// phase >= 1.0 and wrap it back to the 0.0-1.0 range. This is a modulo operation that
    /// must be done correctly to avoid cumulative rounding errors.
    #[cfg(feature = "simd")]
    pub fn process(&mut self) -> f32 {
        // OPTIMIZATION: Early return if shape is effectively zero (skip expensive shaping)
        if self.shape.abs() < 0.001 && self.waveform != Waveform::Pulse {
            // Fast path: no wave shaping needed
            let phases = f32x4::from_array([
                self.phase,
                self.phase + self.phase_increment,
                self.phase + 2.0 * self.phase_increment,
                self.phase + 3.0 * self.phase_increment,
            ]);
            let samples = waveform::generate_simd(phases, self.waveform);
            self.phase += 4.0 * self.phase_increment;
            while self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            return self.downsampler.process([
                samples.as_array()[0],
                samples.as_array()[1],
                samples.as_array()[2],
                samples.as_array()[3],
            ]);
        }

        // SIMD-optimized version with wave shaping
        // Generate 4 phase values at once, each offset by one oversampled increment
        let phases = f32x4::from_array([
            self.phase,
            self.phase + self.phase_increment,
            self.phase + 2.0 * self.phase_increment,
            self.phase + 3.0 * self.phase_increment,
        ]);

        // Generate samples based on waveform using SIMD
        let mut samples = match self.waveform {
            Waveform::Pulse => {
                // Pulse width is controlled by shape parameter
                // Maps shape (-1.0 to 1.0) to pulse width (10% to 90%)
                // At shape=0, pulse width is 50% (square wave)
                let pulse_width = 0.5 + self.shape * 0.4; // Maps to 0.1 - 0.9 range
                let threshold = f32x4::splat(pulse_width);
                let one = f32x4::splat(1.0);
                let neg_one = f32x4::splat(-1.0);
                // Use SIMD comparison: if phase < pulse_width, output 1.0, else -1.0
                phases.simd_lt(threshold).select(one, neg_one)
            }
            _ => waveform::generate_simd(phases, self.waveform),
        };

        // Apply wave shaping if shape parameter is non-zero
        if self.shape != 0.0 && self.waveform != Waveform::Pulse {
            samples = self.apply_wave_shaping(samples);
        }

        // Advance phase by 4 increments (one for each oversampled sample)
        self.phase += 4.0 * self.phase_increment;
        // Wrap phase back into 0.0-1.0 range if it exceeded 1.0
        while self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Convert SIMD vector to array and downsample 4 samples to 1
        self.downsampler.process(samples.to_array())
    }

    /// Generate one output sample (processes 4× oversampled internally) - Scalar version.
    ///
    /// This is the fallback version when the "simd" feature is not enabled. It does the same
    /// thing as the SIMD version but processes samples one at a time instead of 4 in parallel.
    /// It's slower but works on any CPU and doesn't require nightly Rust.
    ///
    /// The algorithm is identical to the SIMD version, just implemented with scalar floating
    /// point operations instead of vector operations.
    #[cfg(not(feature = "simd"))]
    pub fn process(&mut self) -> f32 {
        // OPTIMIZATION: Early return if shape is effectively zero (skip expensive shaping)
        if self.shape.abs() < 0.001 && self.waveform != Waveform::Pulse {
            // Fast path: no wave shaping needed, just generate base waveform
            let mut oversampled = [0.0; 4];
            for sample in &mut oversampled {
                *sample = waveform::generate_scalar(self.phase, self.waveform);
                self.phase += self.phase_increment;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
            }
            return self.downsampler.process(oversampled);
        }

        // Generate 4 oversampled samples (one per iteration)
        let mut oversampled = [0.0; 4];

        for sample in &mut oversampled {
            *sample = match self.waveform {
                Waveform::Pulse => {
                    // Pulse width controlled by shape: -1.0 = 10% duty, 0.0 = 50%, 1.0 = 90%
                    let pulse_width = 0.5 + self.shape * 0.4;
                    if self.phase < pulse_width { 1.0 } else { -1.0 }
                }
                _ => waveform::generate_scalar(self.phase, self.waveform),
            };

            // Apply wave shaping if not Pulse (Pulse uses shape for PWM, not morphing)
            if self.shape != 0.0 && self.waveform != Waveform::Pulse {
                *sample = self.apply_wave_shaping_scalar(*sample);
            }

            // Advance phase for next sample
            self.phase += self.phase_increment;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        // Downsample from 4× back to 1× output sample
        self.downsampler.process(oversampled)
    }

    /// Apply wave shaping to a scalar sample (non-SIMD version).
    ///
    /// This implements the same wave shaping logic as apply_wave_shaping() but operates on
    /// a single floating point value instead of a SIMD vector. It's called per-sample in the
    /// scalar processing path.
    #[cfg(not(feature = "simd"))]
    fn apply_wave_shaping_scalar(&self, sample: f32) -> f32 {
        let shape_amount = self.shape.abs();
        match self.waveform {
            Waveform::Sine => {
                // Add harmonics via soft clipping (tanh approximation: x - x³/3)
                let drive = 1.0 + shape_amount * 3.0;
                let driven = sample * drive;
                let tanh_approx = driven - (driven * driven * driven) / 3.0;
                tanh_approx / drive.sqrt()
            }
            Waveform::Saw => {
                // Morph towards triangle
                let triangle = if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    -4.0 * self.phase + 3.0
                };
                sample * (1.0 - shape_amount) + triangle * shape_amount
            }
            Waveform::Square => sample,
            Waveform::Triangle => {
                // Add sharpness (morph towards saw)
                let saw = 2.0 * self.phase - 1.0;
                sample * (1.0 - shape_amount) + saw * shape_amount
            }
            Waveform::Pulse => sample,
        }
    }

    /// Reset the oscillator to its initial state.
    ///
    /// This clears the phase accumulator and the internal downsampler state. Useful when:
    /// - A note ends and the voice is being reused for a new note
    /// - You want to ensure the oscillator starts cleanly without any state from previous processing
    ///
    /// Note: This does NOT reset frequency, waveform, or shape parameters. It only clears
    /// the running state.
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.downsampler.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_oscillator_creation() {
        let osc = Oscillator::new(44100.0);
        // assert_eq!(osc.sample_rate, 44100.0);
        assert_eq!(osc.oversample_rate, 176400.0);
        assert_eq!(osc.phase, 0.0);
    }

    #[test]
    fn test_frequency_setting() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Phase increment should be freq / oversample_rate
        let expected_increment = 440.0 / 176400.0;
        assert_relative_eq!(osc.phase_increment, expected_increment, epsilon = 1e-6);
    }

    #[test]
    fn test_sine_wave_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sine);
        osc.set_frequency(1000.0);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(osc.process());
        }

        // Find max and min values (should approach ±1.0 for sine wave)
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max > 0.9, "Max value {} should be close to 1.0", max);
        assert!(
            max < 1.1,
            "Max value {} should not exceed 1.0 significantly",
            max
        );
        assert!(min < -0.9, "Min value {} should be close to -1.0", min);
        assert!(
            min > -1.1,
            "Min value {} should not be below -1.0 significantly",
            min
        );
    }

    #[test]
    fn test_saw_wave_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Saw);
        osc.set_frequency(1000.0);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(osc.process());
        }

        // Verify we get a range of values
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        // Oversampling and filtering will reduce the amplitude slightly
        assert!(max > 0.5, "Max value {} should be substantial", max);
        assert!(min < -0.5, "Min value {} should be substantial", min);
    }

    #[test]
    fn test_square_wave_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Square);
        osc.set_frequency(1000.0);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(osc.process());
        }

        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        // Square wave after filtering will have reduced amplitude
        assert!(max > 0.4, "Max value {} should be substantial", max);
        assert!(min < -0.4, "Min value {} should be substantial", min);
    }

    #[test]
    fn test_triangle_wave_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Triangle);
        osc.set_frequency(1000.0);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(osc.process());
        }

        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max > 0.8, "Max value {} should be close to 1.0", max);
        assert!(min < -0.8, "Min value {} should be close to -1.0", min);
    }

    #[test]
    fn test_waveform_switching() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);

        osc.set_waveform(Waveform::Sine);
        // Process a few samples to let filter settle
        for _ in 0..20 {
            osc.process();
        }
        let sine_sample = osc.process();

        osc.reset();
        osc.set_waveform(Waveform::Saw);
        for _ in 0..20 {
            osc.process();
        }
        let saw_sample = osc.process();

        // Different waveforms should produce different outputs after settling
        assert!((sine_sample - saw_sample).abs() > 0.001);
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(10000.0); // High frequency to cause wrapping quickly
        osc.set_waveform(Waveform::Sine);

        // Process many samples
        for _ in 0..1000 {
            osc.process();
        }

        // Phase should still be in valid range [0, 1)
        assert!(osc.phase >= 0.0 && osc.phase < 1.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Sine);

        // Process some samples
        for _ in 0..10 {
            osc.process();
        }

        // Reset should clear phase
        osc.reset();
        assert_eq!(osc.phase, 0.0);
    }

    #[test]
    fn test_dc_offset() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Sine);

        let mut sum = 0.0;
        let num_samples = 4410; // One full second at 44.1kHz / 10

        for _ in 0..num_samples {
            sum += osc.process();
        }

        let average = sum / num_samples as f32;

        // Sine wave should have near-zero DC offset
        assert!(
            average.abs() < 0.01,
            "DC offset {} should be near zero",
            average
        );
    }

    #[test]
    fn test_aliasing_reduction() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Saw);
        osc.set_frequency(5000.0); // High frequency sawtooth

        let mut samples = Vec::new();
        for _ in 0..1000 {
            samples.push(osc.process());
        }

        // With 4× oversampling and proper downsampling, high-frequency content
        // should be filtered out. This is a basic check that output isn't clipping
        // or producing obviously wrong values
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max < 1.5, "Output shouldn't clip excessively");
        assert!(min > -1.5, "Output shouldn't clip excessively");
    }
}
