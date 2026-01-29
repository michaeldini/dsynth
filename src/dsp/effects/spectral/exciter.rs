/// High-Frequency Exciter - Adds "air", "sizzle", and presence to sounds
///
/// An exciter enhances high frequencies by:
/// 1. High-pass filtering the signal (isolates highs, typically 5kHz+)
/// 2. Applying harmonic distortion to the high-frequency content
/// 3. Mixing the enhanced highs back with the original signal
///
/// This is different from EQ because it adds **new harmonics** rather than just boosting
/// existing frequencies. Makes sounds "pop" and "shine" in a mix without muddiness.
///
/// # Use Cases
/// - Vocal presence and clarity
/// - Synth leads that cut through dense mixes
/// - Adding "air" to pads and strings
/// - Percussion attack and snap
///
/// # Parameters
/// - **frequency**: High-pass cutoff (2kHz - 12kHz), typical 4-8kHz
/// - **drive**: Harmonic generation amount (0.0 = clean, 1.0 = saturated)
/// - **mix**: Wet/dry balance (0.0 = bypassed, 1.0 = maximum enhancement)
use std::f32::consts::PI;

pub struct Exciter {
    sample_rate: f32,

    // High-pass filter for isolating high frequencies
    hp_cutoff: f32,
    hp_b0: f32,
    hp_b1: f32,
    hp_b2: f32,
    hp_a1: f32,
    hp_a2: f32,

    // Filter state (Direct Form I)
    hp_x1_left: f32,
    hp_x2_left: f32,
    hp_y1_left: f32,
    hp_y2_left: f32,

    hp_x1_right: f32,
    hp_x2_right: f32,
    hp_y1_right: f32,
    hp_y2_right: f32,

    // Parameters
    drive: f32, // 0.0 to 1.0
    mix: f32,   // 0.0 to 1.0
}

impl Exciter {
    /// Create a new exciter
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut exciter = Self {
            sample_rate,
            hp_cutoff: 3000.0, // Lower cutoff to include more content
            hp_b0: 1.0,
            hp_b1: 0.0,
            hp_b2: 0.0,
            hp_a1: 0.0,
            hp_a2: 0.0,
            hp_x1_left: 0.0,
            hp_x2_left: 0.0,
            hp_y1_left: 0.0,
            hp_y2_left: 0.0,
            hp_x1_right: 0.0,
            hp_x2_right: 0.0,
            hp_y1_right: 0.0,
            hp_y2_right: 0.0,
            drive: 0.0,
            mix: 0.5,
        };

        exciter.update_filter_coefficients();
        exciter
    }

    /// Set high-pass cutoff frequency (2000Hz to 12000Hz)
    pub fn set_frequency(&mut self, freq: f32) {
        let new_cutoff = freq.clamp(2000.0, 12000.0);
        if (self.hp_cutoff - new_cutoff).abs() > 1.0 {
            self.hp_cutoff = new_cutoff;
            self.update_filter_coefficients();
        }
    }

    /// Set harmonic drive amount (0.0 to 1.0)
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.0, 1.0);
    }

    /// Set wet/dry mix (0.0 = bypassed, 1.0 = maximum enhancement)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Update high-pass filter coefficients (Butterworth 2nd order)
    fn update_filter_coefficients(&mut self) {
        let omega = 2.0 * PI * self.hp_cutoff / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let q = 0.707; // Butterworth Q for flat response

        let alpha = sin_omega / (2.0 * q);

        // High-pass filter coefficients (Audio EQ Cookbook)
        let b0 = (1.0 + cos_omega) / 2.0;
        let b1 = -(1.0 + cos_omega);
        let b2 = (1.0 + cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        // Normalize by a0 (with safety check)
        if a0.abs() < 1e-6 {
            // Invalid a0 - use bypass coefficients
            self.hp_b0 = 1.0;
            self.hp_b1 = 0.0;
            self.hp_b2 = 0.0;
            self.hp_a1 = 0.0;
            self.hp_a2 = 0.0;
        } else {
            self.hp_b0 = b0 / a0;
            self.hp_b1 = b1 / a0;
            self.hp_b2 = b2 / a0;
            self.hp_a1 = a1 / a0;
            self.hp_a2 = a2 / a0;
        }
    }

    /// Apply musical harmonic enhancement (production version)
    #[inline]
    fn apply_enhancement_musical(&self, input: f32) -> f32 {
        if self.drive < 0.001 {
            return input;
        }

        // Map drive 0-1 to gain 1.5x-4x for musical enhancement
        let gain = 1.5 + self.drive * 2.5;
        let driven = input * gain;

        // Gentle multi-stage distortion for musicality
        let stage1 = driven.tanh(); // Primary saturation
        let stage2 = (stage1 * 1.5).tanh() * 0.7; // Gentle secondary harmonics
        let enhanced = stage1 + stage2 * 0.2; // Subtle blend

        // Musical compensation with moderate boost
        if gain > 0.0 && gain.is_finite() {
            enhanced * 1.2 / gain.sqrt() // Gentle boost for presence
        } else {
            enhanced
        }
    }

    /// Apply harmonic distortion to enhance high frequencies (with explicit drive)
    /// Kept for reference - more aggressive than musical version
    #[allow(dead_code)]
    fn apply_enhancement_with_drive(&self, input: f32, drive: f32) -> f32 {
        // Always apply some enhancement for debugging
        let effective_drive = drive.max(0.1);

        // Map drive 0-1 to gain 3x-12x for very aggressive enhancement
        let gain = 3.0 + effective_drive * 9.0;
        let driven = input * gain;

        // Multi-stage distortion for richer harmonics
        let stage1 = driven.tanh(); // Primary saturation
        let stage2 = (stage1 * 2.0).tanh() * 0.5; // Secondary harmonics
        let enhanced = stage1 + stage2 * 0.3; // Blend stages

        // Aggressive compensation with significant boost
        if gain > 0.0 && gain.is_finite() {
            enhanced * 2.0 / gain.sqrt() // 2x boost for audibility
        } else {
            enhanced * 2.0
        }
    }

    /// Apply harmonic distortion to enhance high frequencies
    #[inline]
    fn apply_enhancement(&self, input: f32) -> f32 {
        if self.drive < 0.001 {
            return input * 0.5; // Still apply some gain even with no drive
        }

        // Map drive 0-1 to gain 2x-10x for more aggressive enhancement
        let gain = 2.0 + self.drive * 8.0;
        let driven = input * gain;

        // Use asymmetric saturation for more interesting harmonics
        let saturated = if driven >= 0.0 {
            driven.tanh() // Soft clipping for positive
        } else {
            // Harder clipping for negative to add more harmonics
            (driven * 1.5).tanh() * 0.8
        };

        // More aggressive compensation with boost
        if gain > 0.0 && gain.is_finite() {
            saturated * 1.5 / gain.sqrt() // Add 1.5x boost
        } else {
            saturated
        }
    }

    /// Process stereo audio
    ///
    /// # Arguments
    /// * `left` - Left channel input sample
    /// * `right` - Right channel input sample
    ///
    /// # Returns
    /// Tuple of (left_out, right_out)
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // If mix is zero, bypass completely
        if self.mix < 0.001 {
            return (left, right);
        }

        // === Left Channel ===
        // Step 1: High-pass filter to isolate high frequencies
        let hp_left =
            self.hp_b0 * left + self.hp_b1 * self.hp_x1_left + self.hp_b2 * self.hp_x2_left
                - self.hp_a1 * self.hp_y1_left
                - self.hp_a2 * self.hp_y2_left;

        // Update left channel filter state
        self.hp_x2_left = self.hp_x1_left;
        self.hp_x1_left = left;
        self.hp_y2_left = self.hp_y1_left;
        self.hp_y1_left = hp_left;

        // Step 2: Apply harmonic enhancement to high frequencies
        let enhanced_left = self.apply_enhancement_musical(hp_left);

        // Step 3: Mix enhanced highs back with original
        let left_out = left + enhanced_left * self.mix;

        // === Right Channel ===
        let hp_right =
            self.hp_b0 * right + self.hp_b1 * self.hp_x1_right + self.hp_b2 * self.hp_x2_right
                - self.hp_a1 * self.hp_y1_right
                - self.hp_a2 * self.hp_y2_right;

        self.hp_x2_right = self.hp_x1_right;
        self.hp_x1_right = right;
        self.hp_y2_right = self.hp_y1_right;
        self.hp_y1_right = hp_right;

        let enhanced_right = self.apply_enhancement_musical(hp_right);
        let right_out = right + enhanced_right * self.mix;

        (left_out, right_out)
    }

    /// Reset filter state (call when stopping playback or changing sample rate)
    pub fn reset(&mut self) {
        self.hp_x1_left = 0.0;
        self.hp_x2_left = 0.0;
        self.hp_y1_left = 0.0;
        self.hp_y2_left = 0.0;
        self.hp_x1_right = 0.0;
        self.hp_x2_right = 0.0;
        self.hp_y1_right = 0.0;
        self.hp_y2_right = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_exciter_bypass() {
        let mut exciter = Exciter::new(44100.0);
        exciter.set_mix(0.0); // Bypass

        let (left, right) = exciter.process(0.5, -0.3);
        assert_relative_eq!(left, 0.5, epsilon = 0.001);
        assert_relative_eq!(right, -0.3, epsilon = 0.001);
    }

    #[test]
    fn test_exciter_enhancement() {
        let mut exciter = Exciter::new(44100.0);
        exciter.set_frequency(5000.0);
        exciter.set_drive(0.5);
        exciter.set_mix(1.0);

        // Process high-frequency test signal
        let freq = 8000.0;
        let sample_rate = 44100.0;
        let mut max_amplitude: f32 = 0.0;

        for i in 0..1000 {
            let phase = i as f32 / sample_rate;
            let input = (2.0 * PI * freq * phase).sin() * 0.3;
            let (left, _) = exciter.process(input, input);
            max_amplitude = max_amplitude.max(left.abs());
        }

        // Exciter should increase amplitude of high frequencies
        assert!(
            max_amplitude > 0.3,
            "Exciter should enhance high frequencies"
        );
    }

    #[test]
    fn test_exciter_low_freq_rejection() {
        let mut exciter = Exciter::new(44100.0);
        exciter.set_frequency(5000.0);
        exciter.set_drive(1.0);
        exciter.set_mix(1.0);

        // Process low-frequency test signal (should pass through mostly unchanged)
        let freq = 200.0;
        let sample_rate = 44100.0;
        let mut sum = 0.0;

        for i in 0..1000 {
            let phase = i as f32 / sample_rate;
            let input = (2.0 * PI * freq * phase).sin() * 0.5;
            let (left, _) = exciter.process(input, input);
            sum += (left - input).abs();
        }

        let avg_difference = sum / 1000.0;
        // Low frequencies should be relatively unaffected
        assert!(
            avg_difference < 0.1,
            "Exciter should minimally affect low frequencies"
        );
    }
}
