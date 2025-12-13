/// Shared waveform generation logic for oscillators and LFOs
/// Provides both scalar and SIMD implementations of common waveforms
use crate::params::Waveform;
use std::f32::consts::PI;

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
                sample >= -1.0 && sample <= 1.0,
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
                sample >= -1.0 && sample <= 1.0,
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
                sample >= -1.0 && sample <= 1.0,
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
}
