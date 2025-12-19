/// Shared waveform generation logic for oscillators and LFOs
/// Provides both scalar and SIMD implementations of common waveforms
use crate::params::Waveform;
use std::f32::consts::PI;

/// Fast xorshift32 PRNG for noise generation
/// Returns u32 in full range, caller converts to f32
#[inline]
pub fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

/// Convert u32 to f32 in range [-1.0, 1.0]
#[inline]
pub fn u32_to_f32_bipolar(x: u32) -> f32 {
    // Map [0, u32::MAX] to [-1.0, 1.0]
    (x as f32 / (u32::MAX as f32 / 2.0)) - 1.0
}

#[cfg(feature = "simd")]
use std::simd::{StdFloat, cmp::SimdPartialOrd, f32x4};

/// Generate a scalar waveform sample at a given normalized phase [0.0, 1.0)
///
/// # Arguments
/// * `phase` - Normalized phase in range [0.0, 1.0)
/// * `waveform` - Waveform type to generate
///
/// # Returns
/// Sample value in range [-1.0, 1.0]
pub fn generate_scalar(phase: f32, waveform: Waveform) -> f32 {
    match waveform {
        Waveform::Sine => (phase * 2.0 * PI).sin(),
        Waveform::Saw => 2.0 * phase - 1.0,
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Triangle => {
            if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                -4.0 * phase + 3.0
            }
        }
        Waveform::Pulse => {
            // For scalar pulse, default to 50% duty cycle (square)
            // Pulse width modulation is handled by the oscillator
            if phase < 0.5 { 1.0 } else { -1.0 }
        }
        Waveform::WhiteNoise | Waveform::PinkNoise => {
            // Noise generation requires stateful PRNG, handled by oscillator
            // This path shouldn't be called for noise waveforms
            0.0
        }
    }
}

/// Generate SIMD waveform samples at 4 given normalized phases
/// Only available when "simd" feature is enabled
#[cfg(feature = "simd")]
pub fn generate_simd(phases: f32x4, waveform: Waveform) -> f32x4 {
    match waveform {
        Waveform::Sine => {
            let two_pi = f32x4::splat(2.0 * PI);
            (phases * two_pi).sin()
        }
        Waveform::Saw => f32x4::splat(2.0) * phases - f32x4::splat(1.0),
        Waveform::Square => {
            let half = f32x4::splat(0.5);
            let one = f32x4::splat(1.0);
            let neg_one = f32x4::splat(-1.0);
            phases.simd_lt(half).select(one, neg_one)
        }
        Waveform::Triangle => {
            let half = f32x4::splat(0.5);
            let four = f32x4::splat(4.0);
            let neg_four = f32x4::splat(-4.0);
            let one = f32x4::splat(1.0);
            let three = f32x4::splat(3.0);

            let low_branch = four * phases - one;
            let high_branch = neg_four * phases + three;
            phases.simd_lt(half).select(low_branch, high_branch)
        }
        Waveform::Pulse => {
            // For SIMD pulse with default 50% duty cycle (square)
            let half = f32x4::splat(0.5);
            let one = f32x4::splat(1.0);
            let neg_one = f32x4::splat(-1.0);
            phases.simd_lt(half).select(one, neg_one)
        }
        Waveform::WhiteNoise | Waveform::PinkNoise => {
            // Noise generation requires stateful PRNG, handled by oscillator
            // This path shouldn't be called for noise waveforms
            f32x4::splat(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_range() {
        for i in 0..100 {
            let phase = (i as f32) / 100.0;
            let sample = generate_scalar(phase, Waveform::Sine);
            assert!(
                (-1.0..=1.0).contains(&sample),
                "Sine out of range at phase {}: {}",
                phase,
                sample
            );
        }
    }

    #[test]
    fn test_saw_range() {
        for i in 0..100 {
            let phase = (i as f32) / 100.0;
            let sample = generate_scalar(phase, Waveform::Saw);
            assert!(
                (-1.0..=1.0).contains(&sample),
                "Saw out of range at phase {}: {}",
                phase,
                sample
            );
        }
    }

    #[test]
    fn test_square_values() {
        assert_eq!(generate_scalar(0.0, Waveform::Square), 1.0);
        assert_eq!(generate_scalar(0.25, Waveform::Square), 1.0);
        assert_eq!(generate_scalar(0.49, Waveform::Square), 1.0);
        assert_eq!(generate_scalar(0.5, Waveform::Square), -1.0);
        assert_eq!(generate_scalar(0.75, Waveform::Square), -1.0);
    }

    #[test]
    fn test_triangle_range() {
        for i in 0..100 {
            let phase = (i as f32) / 100.0;
            let sample = generate_scalar(phase, Waveform::Triangle);
            assert!(
                (-1.0..=1.0).contains(&sample),
                "Triangle out of range at phase {}: {}",
                phase,
                sample
            );
        }
    }

    #[test]
    fn test_pulse_range() {
        for i in 0..100 {
            let phase = (i as f32) / 100.0;
            let sample = generate_scalar(phase, Waveform::Pulse);
            assert!(
                sample == 1.0 || sample == -1.0,
                "Pulse out of range at phase {}: {}",
                phase,
                sample
            );
        }
    }
    /// Test xorshift32 PRNG produces non-zero values
    #[test]
    fn test_xorshift32_non_zero() {
        let mut state = 1u32;
        for _ in 0..100 {
            let value = xorshift32(&mut state);
            assert_ne!(value, 0, "PRNG should not produce zero");
        }
    }

    /// Test xorshift32 has sufficient entropy (different values)
    #[test]
    fn test_xorshift32_entropy() {
        let mut state = 12345u32;
        let mut values = Vec::new();
        for _ in 0..100 {
            values.push(xorshift32(&mut state));
        }
        
        // Check that values are not all the same
        let first = values[0];
        let all_same = values.iter().all(|&v| v == first);
        assert!(!all_same, "PRNG should produce varying values");
    }

    /// Test u32_to_f32_bipolar produces values in [-1.0, 1.0]
    #[test]
    fn test_u32_to_f32_bipolar_range() {
        let mut state = 1u32;
        for _ in 0..1000 {
            let u = xorshift32(&mut state);
            let f = u32_to_f32_bipolar(u);
            assert!(
                (-1.0..=1.0).contains(&f),
                "Noise value out of range: {}",
                f
            );
        }
    }

    /// Test u32_to_f32_bipolar extremes
    #[test]
    fn test_u32_to_f32_bipolar_extremes() {
        // 0 should map to -1.0
        let min = u32_to_f32_bipolar(0);
        assert!((min - (-1.0)).abs() < 0.01, "Min should be near -1.0");
        
        // u32::MAX should map to ~1.0
        let max = u32_to_f32_bipolar(u32::MAX);
        assert!((max - 1.0).abs() < 0.01, "Max should be near 1.0");
    }}
