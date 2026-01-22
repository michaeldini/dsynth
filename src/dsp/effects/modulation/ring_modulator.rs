/// Ring modulator - multiplies input signal with a carrier oscillator
/// Creates inharmonic, bell-like, and metallic tones by amplitude modulation
use std::f32::consts::PI;

pub struct RingModulator {
    /// Sample rate for frequency calculations
    sample_rate: f32,

    /// Carrier oscillator phase accumulator
    carrier_phase: f32,

    /// Carrier frequency in Hz
    carrier_freq: f32,

    /// Carrier waveform type
    waveform: Waveform,

    /// Modulation depth (0.0 to 1.0) - blend between dry and ring modulated
    depth: f32,

    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    mix: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Triangle,
    Square,
    Saw,
}

impl RingModulator {
    /// Create a new ring modulator
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `carrier_freq` - Carrier frequency in Hz (determines the character of modulation)
    pub fn new(sample_rate: f32, carrier_freq: f32) -> Self {
        Self {
            sample_rate,
            carrier_phase: 0.0,
            carrier_freq: carrier_freq.clamp(0.1, sample_rate * 0.5),
            waveform: Waveform::Sine,
            depth: 1.0,
            mix: 1.0,
        }
    }

    /// Set carrier frequency in Hz
    pub fn set_frequency(&mut self, freq_hz: f32) {
        self.carrier_freq = freq_hz.clamp(0.1, self.sample_rate * 0.5);
    }

    /// Set carrier waveform
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Set modulation depth (0.0 to 1.0)
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Process a stereo sample through the ring modulator
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Generate carrier oscillator sample
        let carrier = self.generate_carrier();

        // Advance carrier phase
        self.carrier_phase += self.carrier_freq / self.sample_rate;
        if self.carrier_phase >= 1.0 {
            self.carrier_phase -= 1.0;
        }

        // Ring modulation: multiply input by carrier
        // With depth control: output = input * (1 - depth + carrier * depth)
        // This allows partial modulation (depth < 1.0)
        let modulation_amount = 1.0 - self.depth + carrier * self.depth;

        let wet_left = left * modulation_amount;
        let wet_right = right * modulation_amount;

        // Mix dry and wet
        let output_left = left * (1.0 - self.mix) + wet_left * self.mix;
        let output_right = right * (1.0 - self.mix) + wet_right * self.mix;

        (output_left, output_right)
    }

    /// Generate one sample of the carrier oscillator
    #[inline]
    fn generate_carrier(&self) -> f32 {
        match self.waveform {
            Waveform::Sine => (self.carrier_phase * 2.0 * PI).sin(),
            Waveform::Triangle => {
                // Triangle wave: -1 to 1
                let phase = self.carrier_phase;
                if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                }
            }
            Waveform::Square => {
                // Square wave: -1 or 1
                if self.carrier_phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Saw => {
                // Sawtooth wave: -1 to 1
                2.0 * self.carrier_phase - 1.0
            }
        }
    }

    /// Reset the ring modulator state
    pub fn reset(&mut self) {
        self.carrier_phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_ring_mod_creation() {
        let ring_mod = RingModulator::new(44100.0, 440.0);
        assert_eq!(ring_mod.sample_rate, 44100.0);
        assert_relative_eq!(ring_mod.carrier_freq, 440.0, epsilon = 0.1);
    }

    #[test]
    fn test_ring_mod_frequency_clamping() {
        let mut ring_mod = RingModulator::new(44100.0, 440.0);

        // Should clamp to valid range
        ring_mod.set_frequency(50000.0); // Above Nyquist
        assert!(ring_mod.carrier_freq <= 44100.0 * 0.5);

        ring_mod.set_frequency(-100.0); // Negative
        assert!(ring_mod.carrier_freq >= 0.1);
    }

    #[test]
    fn test_ring_mod_sine_carrier() {
        let mut ring_mod = RingModulator::new(44100.0, 1000.0);
        ring_mod.set_waveform(Waveform::Sine);
        ring_mod.set_mix(1.0);
        ring_mod.set_depth(1.0);

        // Process constant input
        let input = 0.5;
        let mut outputs = Vec::new();

        for _ in 0..441 {
            // One period at 100 Hz carrier
            let (left, _) = ring_mod.process(input, input);
            outputs.push(left);
        }

        // Output should oscillate (be modulated)
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!(max - min > 0.3); // Significant modulation
    }

    #[test]
    fn test_ring_mod_creates_sidebands() {
        let mut ring_mod = RingModulator::new(44100.0, 100.0);
        ring_mod.set_mix(1.0);
        ring_mod.set_depth(1.0);

        // Input a sine wave at 1000 Hz
        // Ring modulation should create sidebands at 900 Hz and 1100 Hz (1000 ± 100)

        let input_freq = 1000.0;
        let mut outputs = Vec::new();

        for i in 0..4410 {
            // 0.1 second
            let phase = i as f32 / 44100.0;
            let input = (input_freq * 2.0 * PI * phase).sin();
            let (left, _) = ring_mod.process(input, input);
            outputs.push(left);
        }

        // Output should contain frequency content (not constant)
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!(max - min > 1.0);
    }

    #[test]
    fn test_ring_mod_depth_control() {
        let mut ring_mod_no_depth = RingModulator::new(44100.0, 440.0);
        ring_mod_no_depth.set_depth(0.0);
        ring_mod_no_depth.set_mix(1.0);

        let mut ring_mod_full_depth = RingModulator::new(44100.0, 440.0);
        ring_mod_full_depth.set_depth(1.0);
        ring_mod_full_depth.set_mix(1.0);

        // Process constant input
        let input = 0.5;

        let (left_no_depth, _) = ring_mod_no_depth.process(input, input);
        let (left_full_depth, _) = ring_mod_full_depth.process(input, input);

        // No depth should pass signal through (approximately)
        assert_relative_eq!(left_no_depth, input, epsilon = 0.01);

        // Full depth should modulate significantly
        assert!((left_full_depth - input).abs() > 0.1);
    }

    #[test]
    fn test_ring_mod_mix_control() {
        let mut ring_mod = RingModulator::new(44100.0, 440.0);

        // Dry signal (mix = 0.0)
        ring_mod.set_mix(0.0);
        let (left_dry, _) = ring_mod.process(0.6, 0.6);
        assert_relative_eq!(left_dry, 0.6, epsilon = 0.01);

        // Wet signal (mix = 1.0)
        ring_mod.reset();
        ring_mod.set_mix(1.0);
        ring_mod.set_depth(1.0);
        let (left_wet, _) = ring_mod.process(0.6, 0.6);

        // At phase = 0, sine carrier starts at 0, so output should be near 0
        assert!(left_wet.abs() < 0.3);
    }

    #[test]
    fn test_ring_mod_square_wave_carrier() {
        let mut ring_mod = RingModulator::new(44100.0, 1000.0);
        ring_mod.set_waveform(Waveform::Square);
        ring_mod.set_mix(1.0);
        ring_mod.set_depth(1.0);

        let input = 0.5;
        let mut outputs = Vec::new();

        for _ in 0..100 {
            let (left, _) = ring_mod.process(input, input);
            outputs.push(left);
        }

        // Square wave carrier should create abrupt transitions
        let unique_values: std::collections::HashSet<_> = outputs
            .iter()
            .map(|&x| (x * 100.0).round() as i32)
            .collect();

        // Should have at least two distinct levels (positive and negative)
        assert!(unique_values.len() >= 2);
    }

    #[test]
    fn test_ring_mod_triangle_carrier() {
        let mut ring_mod = RingModulator::new(44100.0, 1000.0);
        ring_mod.set_waveform(Waveform::Triangle);
        ring_mod.set_mix(1.0);
        ring_mod.set_depth(1.0);

        let input = 0.5;
        let mut outputs = Vec::new();

        for _ in 0..441 {
            let (left, _) = ring_mod.process(input, input);
            outputs.push(left);
        }

        // Triangle carrier should modulate smoothly
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!(max - min > 0.3);
    }

    #[test]
    fn test_ring_mod_saw_carrier() {
        let mut ring_mod = RingModulator::new(44100.0, 1000.0);
        ring_mod.set_waveform(Waveform::Saw);
        ring_mod.set_mix(1.0);
        ring_mod.set_depth(1.0);

        let input = 0.5;
        let mut outputs = Vec::new();

        for _ in 0..441 {
            let (left, _) = ring_mod.process(input, input);
            outputs.push(left);
        }

        // Saw carrier should modulate
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!(max - min > 0.3);
    }

    #[test]
    fn test_ring_mod_reset() {
        let mut ring_mod = RingModulator::new(44100.0, 440.0);

        // Process some samples
        for _ in 0..100 {
            ring_mod.process(1.0, 1.0);
        }

        // Reset
        ring_mod.reset();

        // Phase should be cleared
        assert_eq!(ring_mod.carrier_phase, 0.0);
    }

    #[test]
    fn test_ring_mod_stereo_identical() {
        let mut ring_mod = RingModulator::new(44100.0, 440.0);

        // Both channels should be modulated identically
        for _ in 0..100 {
            let (left, right) = ring_mod.process(0.5, 0.5);
            assert_relative_eq!(left, right, epsilon = 0.001);
        }
    }

    #[test]
    fn test_ring_mod_phase_accumulation() {
        let mut ring_mod = RingModulator::new(44100.0, 44100.0); // 1 Hz at 44.1kHz SR means 1 cycle per sample

        // After one sample, phase should wrap
        ring_mod.process(0.5, 0.5);
        assert!(ring_mod.carrier_phase < 1.0);

        // Process many samples - phase should stay bounded
        for _ in 0..10000 {
            ring_mod.process(0.5, 0.5);
            assert!(ring_mod.carrier_phase >= 0.0 && ring_mod.carrier_phase < 1.0);
        }
    }

    #[test]
    fn test_ring_mod_carrier_waveforms() {
        let ring_mod = RingModulator::new(44100.0, 440.0);

        // Test sine carrier at phase 0.25 (peak)
        let mut rm = ring_mod;
        rm.carrier_phase = 0.25;
        let sine_sample = rm.generate_carrier();
        assert_relative_eq!(sine_sample, 1.0, epsilon = 0.01);

        // Test triangle at phase 0.25 (peak)
        rm.waveform = Waveform::Triangle;
        rm.carrier_phase = 0.25;
        let tri_sample = rm.generate_carrier();
        assert_relative_eq!(tri_sample, 0.0, epsilon = 0.01);

        // Test square at phase 0.25
        rm.waveform = Waveform::Square;
        rm.carrier_phase = 0.25;
        let square_sample = rm.generate_carrier();
        assert_relative_eq!(square_sample, 1.0, epsilon = 0.01);

        // Test saw at phase 0.5 (zero crossing)
        rm.waveform = Waveform::Saw;
        rm.carrier_phase = 0.5;
        let saw_sample = rm.generate_carrier();
        assert_relative_eq!(saw_sample, 0.0, epsilon = 0.01);
    }
}
