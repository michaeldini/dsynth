use crate::dsp::downsampler::Downsampler;
use crate::dsp::waveform;
use crate::params::Waveform;

#[cfg(feature = "simd")]
use std::simd::{StdFloat, cmp::SimdPartialOrd, f32x4};

/// Sample-rate-agnostic oscillator with 4× oversampling for anti-aliasing
pub struct Oscillator {
    oversample_rate: f32,
    phase: f32,
    phase_increment: f32,
    downsampler: Downsampler,
    waveform: Waveform,
    initial_phase: f32, // Initial phase offset for unison
    shape: f32,         // Wave shaping amount (-1.0 to 1.0)
}

impl Oscillator {
    /// Create a new oscillator
    ///
    /// # Arguments
    /// * `sample_rate` - Target sample rate (e.g., 44100.0)
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

    /// Set initial phase offset (0.0 to 1.0)
    pub fn set_phase(&mut self, phase: f32) {
        self.initial_phase = phase.clamp(0.0, 1.0);
    }

    /// Set the frequency in Hz
    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_increment = freq / self.oversample_rate;
    }

    /// Set the waveform type
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Set the wave shaping amount (-1.0 to 1.0)
    pub fn set_shape(&mut self, shape: f32) {
        self.shape = shape.clamp(-1.0, 1.0);
    }

    /// Apply wave shaping to samples based on waveform type
    #[cfg(feature = "simd")]
    fn apply_wave_shaping(&self, samples: f32x4) -> f32x4 {
        let shape_amount = f32x4::splat(self.shape.abs());
        match self.waveform {
            Waveform::Sine => {
                // Sine: add harmonics via soft clipping/saturation
                let drive = f32x4::splat(1.0 + self.shape.abs() * 3.0);
                let driven = samples * drive;
                // Soft clip using tanh approximation
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

    /// Generate one output sample (processes 4× oversampled internally)
    #[cfg(feature = "simd")]
    pub fn process(&mut self) -> f32 {
        // SIMD-optimized version
        // Generate 4 phase values at once
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
                // shape: -1.0 = 10% duty, 0.0 = 50% duty (square), 1.0 = 90% duty
                let pulse_width = 0.5 + self.shape * 0.4; // Maps to 0.1 - 0.9 range
                let threshold = f32x4::splat(pulse_width);
                let one = f32x4::splat(1.0);
                let neg_one = f32x4::splat(-1.0);
                phases.simd_lt(threshold).select(one, neg_one)
            }
            _ => waveform::generate_simd(phases, self.waveform),
        };

        // Apply wave shaping if shape parameter is non-zero
        if self.shape != 0.0 && self.waveform != Waveform::Pulse {
            samples = self.apply_wave_shaping(samples);
        }

        // Advance phase by 4 increments
        self.phase += 4.0 * self.phase_increment;
        while self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Convert SIMD to array and downsample
        self.downsampler.process(samples.to_array())
    }

    /// Generate one output sample (processes 4× oversampled internally)
    #[cfg(not(feature = "simd"))]
    pub fn process(&mut self) -> f32 {
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

            // Apply wave shaping if not Pulse (Pulse uses shape for PWM)
            if self.shape != 0.0 && self.waveform != Waveform::Pulse {
                *sample = self.apply_wave_shaping_scalar(*sample);
            }

            // Advance phase
            self.phase += self.phase_increment;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        // Downsample from 4× to 1×
        self.downsampler.process(oversampled)
    }

    /// Apply wave shaping to scalar sample (non-SIMD version)
    #[cfg(not(feature = "simd"))]
    fn apply_wave_shaping_scalar(&self, sample: f32) -> f32 {
        let shape_amount = self.shape.abs();
        match self.waveform {
            Waveform::Sine => {
                // Add harmonics via soft clipping
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

    /// Reset oscillator state
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
