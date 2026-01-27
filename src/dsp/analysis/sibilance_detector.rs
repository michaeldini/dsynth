/// Sibilance Detector - Detects 's', 'sh', 't', 'f' consonants in vocals
///
/// Sibilance refers to harsh, high-frequency hissing sounds in speech:
/// - **'s'** sounds (as in "see", "pass")
/// - **'sh'** sounds (as in "she", "push")
/// - **'t'** sounds (as in "tea", "bit")
/// - **'f'** sounds (as in "free", "off")
///
/// # Characteristics of Sibilance
/// - **Frequency range**: 4-10 kHz (peak energy around 6-8 kHz)
/// - **Transient nature**: Brief bursts of high-frequency energy
/// - **High zero-crossing rate**: Many zero crossings (noisy character)
/// - **Distinct from sustained high frequencies**: Different from cymbals/hi-hats
///
/// # Algorithm
/// This detector uses a **multi-feature approach**:
/// 1. **High-pass filter** (4 kHz) to isolate sibilant frequency range
/// 2. **Energy tracking** in sibilant band with fast attack/release
/// 3. **High-frequency ratio**: Compare high-band energy to full-band energy
/// 4. **Transient detection**: Sibilance has sharp attacks
///
/// # Use Cases
/// - Smart de-essing (only reduce sibilance when actually present)
/// - Adaptive threshold de-esser (track sibilance level dynamically)
/// - Consonant emphasis/suppression
/// - Vocal intelligibility enhancement
/// - Sibilance-triggered effects
///
/// # Parameters
/// - **threshold**: Detection sensitivity (0.0-1.0, typical 0.3-0.5)
/// - **attack_ms**: Energy tracker attack time (1-10ms)
/// - **release_ms**: Energy tracker release time (20-100ms)
use std::f32::consts::PI;

pub struct SibilanceDetector {
    sample_rate: f32,

    // High-pass filter for sibilant frequency range (4kHz+)
    hp_cutoff: f32,
    hp_b0: f32,
    hp_b1: f32,
    hp_b2: f32,
    hp_a1: f32,
    hp_a2: f32,
    hp_x1: f32,
    hp_x2: f32,
    hp_y1: f32,
    hp_y2: f32,

    // Energy tracking
    sibilant_energy: f32,  // Energy in high-frequency band
    full_band_energy: f32, // Energy in full spectrum
    energy_attack_coef: f32,
    energy_release_coef: f32,

    // Detection state
    threshold: f32,
    sibilance_detected: bool,
    sibilance_strength: f32, // 0.0 to 1.0+
}

impl SibilanceDetector {
    /// Create a new sibilance detector
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut detector = Self {
            sample_rate,
            hp_cutoff: 4000.0,
            hp_b0: 1.0,
            hp_b1: 0.0,
            hp_b2: 0.0,
            hp_a1: 0.0,
            hp_a2: 0.0,
            hp_x1: 0.0,
            hp_x2: 0.0,
            hp_y1: 0.0,
            hp_y2: 0.0,
            sibilant_energy: 0.0,
            full_band_energy: 0.0,
            energy_attack_coef: 0.0,
            energy_release_coef: 0.0,
            threshold: 0.35, // Default threshold
            sibilance_detected: false,
            sibilance_strength: 0.0,
        };

        detector.update_filter_coefficients();
        detector.set_attack_time(3.0); // Fast attack to catch sibilance onset
        detector.set_release_time(50.0); // Moderate release

        detector
    }

    /// Set high-pass cutoff frequency (3000-8000 Hz, default 4000 Hz)
    pub fn set_cutoff_frequency(&mut self, freq: f32) {
        let new_cutoff = freq.clamp(2000.0, 12000.0);
        if (self.hp_cutoff - new_cutoff).abs() > 10.0 {
            self.hp_cutoff = new_cutoff;
            self.update_filter_coefficients();
        }
    }

    /// Set detection threshold (0.0 = very sensitive, 1.0 = only strong sibilance)
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Set energy tracker attack time (1-20ms, fast to catch sibilance)
    pub fn set_attack_time(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(0.5, 100.0);
        let time_samples = (time_ms / 1000.0) * self.sample_rate;
        self.energy_attack_coef = (-1.0 / time_samples).exp();
    }

    /// Set energy tracker release time (20-200ms)
    pub fn set_release_time(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(5.0, 500.0);
        let time_samples = (time_ms / 1000.0) * self.sample_rate;
        self.energy_release_coef = (-1.0 / time_samples).exp();
    }

    /// Update high-pass filter coefficients (Butterworth 2nd order)
    fn update_filter_coefficients(&mut self) {
        let omega = 2.0 * PI * self.hp_cutoff / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let q = 0.707; // Butterworth Q

        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 + cos_omega) / 2.0;
        let b1 = -(1.0 + cos_omega);
        let b2 = (1.0 + cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        if a0.abs() > 1e-6 {
            self.hp_b0 = b0 / a0;
            self.hp_b1 = b1 / a0;
            self.hp_b2 = b2 / a0;
            self.hp_a1 = a1 / a0;
            self.hp_a2 = a2 / a0;
        }
    }

    /// Process a mono audio sample and detect sibilance
    ///
    /// # Arguments
    /// * `input` - Input audio sample
    ///
    /// # Returns
    /// Tuple of (sibilance_detected: bool, strength: f32)
    pub fn process(&mut self, input: f32) -> (bool, f32) {
        // Step 1: Filter to isolate high-frequency sibilant range
        let hp_output = self.hp_b0 * input + self.hp_b1 * self.hp_x1 + self.hp_b2 * self.hp_x2
            - self.hp_a1 * self.hp_y1
            - self.hp_a2 * self.hp_y2;

        // Update filter state
        self.hp_x2 = self.hp_x1;
        self.hp_x1 = input;
        self.hp_y2 = self.hp_y1;
        self.hp_y1 = hp_output;

        // Step 2: Measure energy in sibilant band vs full band
        let sibilant_energy_instant = hp_output * hp_output;
        let full_band_energy_instant = input * input;

        // Smooth energies with attack/release
        Self::update_energy(
            &mut self.sibilant_energy,
            sibilant_energy_instant,
            self.energy_attack_coef,
            self.energy_release_coef,
        );
        Self::update_energy(
            &mut self.full_band_energy,
            full_band_energy_instant,
            self.energy_attack_coef,
            self.energy_release_coef,
        );

        // Step 3: Calculate sibilance ratio
        // High ratio = lots of high-frequency energy relative to total
        if self.full_band_energy > 1e-6 {
            self.sibilance_strength = (self.sibilant_energy / self.full_band_energy).sqrt();
        } else {
            self.sibilance_strength = 0.0;
        }

        // Clamp to reasonable range
        self.sibilance_strength = self.sibilance_strength.clamp(0.0, 2.0);

        // Step 4: Detect sibilance based on threshold
        self.sibilance_detected = self.sibilance_strength > self.threshold;

        (self.sibilance_detected, self.sibilance_strength)
    }

    /// Update energy tracker with attack/release
    fn update_energy(energy: &mut f32, instant_energy: f32, attack_coef: f32, release_coef: f32) {
        if instant_energy > *energy {
            // Attack
            *energy += (instant_energy - *energy) * (1.0 - attack_coef);
        } else {
            // Release
            *energy += (instant_energy - *energy) * (1.0 - release_coef);
        }
    }

    /// Process stereo audio (uses maximum of both channels)
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (bool, f32) {
        let max_input = left.abs().max(right.abs());
        self.process(max_input)
    }

    /// Get current sibilance detection state
    pub fn is_sibilance_detected(&self) -> bool {
        self.sibilance_detected
    }

    /// Get current sibilance strength (0.0 to ~2.0)
    pub fn get_sibilance_strength(&self) -> f32 {
        self.sibilance_strength
    }

    /// Get suggested de-esser gain reduction (0.0 = full reduction, 1.0 = no reduction)
    /// Maps sibilance strength to gain reduction curve
    pub fn get_suggested_gain_reduction(&self) -> f32 {
        if !self.sibilance_detected {
            return 1.0; // No reduction
        }

        // Soft knee gain reduction curve
        let excess = (self.sibilance_strength - self.threshold).max(0.0);
        let reduction_amount = (excess * 2.0).min(1.0); // Map to 0-1

        // Return gain multiplier (0.0 = silence, 1.0 = unity)
        1.0 - reduction_amount * 0.7 // Max 70% reduction (-10 dB)
    }

    /// Reset detector state
    pub fn reset(&mut self) {
        self.hp_x1 = 0.0;
        self.hp_x2 = 0.0;
        self.hp_y1 = 0.0;
        self.hp_y2 = 0.0;
        self.sibilant_energy = 0.0;
        self.full_band_energy = 0.0;
        self.sibilance_detected = false;
        self.sibilance_strength = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn test_sibilance_detector_creation() {
        let detector = SibilanceDetector::new(44100.0);
        assert!(!detector.is_sibilance_detected());
        assert_relative_eq!(detector.get_sibilance_strength(), 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_detects_high_frequency_burst() {
        let mut detector = SibilanceDetector::new(44100.0);
        detector.set_threshold(0.3);
        let sample_rate = 44100.0;

        // Start with silence
        for _ in 0..500 {
            detector.process(0.0);
        }

        // High-frequency burst (sibilance simulation)
        let mut detected = false;
        for i in 0..200 {
            let phase = i as f32 / sample_rate;
            let sibilance = (2.0 * PI * 7000.0 * phase).sin() * 0.5;
            let (is_sibilant, _) = detector.process(sibilance);
            if is_sibilant {
                detected = true;
                break;
            }
        }

        assert!(detected, "Should detect high-frequency burst as sibilance");
    }

    #[test]
    fn test_ignores_low_frequency() {
        let mut detector = SibilanceDetector::new(44100.0);
        detector.set_threshold(0.3);
        let sample_rate = 44100.0;

        // Low-frequency tone (not sibilance)
        for i in 0..2000 {
            let phase = i as f32 / sample_rate;
            let low_freq = (2.0 * PI * 500.0 * phase).sin() * 0.8;
            detector.process(low_freq);
        }

        let strength = detector.get_sibilance_strength();

        // Low frequency should not trigger sibilance detection
        assert!(
            strength < 0.5,
            "Low frequency should not be detected as sibilance, got strength {}",
            strength
        );
        assert!(
            !detector.is_sibilance_detected(),
            "Low frequency should not trigger detection"
        );
    }

    #[test]
    fn test_broadband_with_sibilant_peak() {
        let mut detector = SibilanceDetector::new(44100.0);
        detector.set_threshold(0.4);
        let sample_rate = 44100.0;

        // Broadband signal with strong high-frequency component (like 's' sound)
        for i in 0..2000 {
            let phase = i as f32 / sample_rate;
            let low = (2.0 * PI * 300.0 * phase).sin() * 0.3; // Fundamental
            let high = (2.0 * PI * 6000.0 * phase).sin() * 0.6; // Sibilance
            let mixed = low + high;
            detector.process(mixed);
        }

        let strength = detector.get_sibilance_strength();

        // Should detect sibilant character
        assert!(
            strength > 0.3,
            "Broadband with sibilant peak should be detected, got strength {}",
            strength
        );
    }

    #[test]
    fn test_threshold_sensitivity() {
        let sample_rate = 44100.0;

        // Low threshold (sensitive)
        let mut detector_sensitive = SibilanceDetector::new(sample_rate);
        detector_sensitive.set_threshold(0.2);

        // High threshold (strict)
        let mut detector_strict = SibilanceDetector::new(sample_rate);
        detector_strict.set_threshold(0.7);

        // Warm up both
        for _ in 0..500 {
            detector_sensitive.process(0.0);
            detector_strict.process(0.0);
        }

        // Moderate high-frequency content
        let mut sensitive_detected = false;
        let mut strict_detected = false;

        for i in 0..500 {
            let phase = i as f32 / sample_rate;
            let test_signal = (2.0 * PI * 6000.0 * phase).sin() * 0.4;

            let (sens, _) = detector_sensitive.process(test_signal);
            let (strict, _) = detector_strict.process(test_signal);

            if sens {
                sensitive_detected = true;
            }
            if strict {
                strict_detected = true;
            }
        }

        assert!(
            sensitive_detected,
            "Low threshold should detect moderate sibilance"
        );
        assert!(
            !strict_detected,
            "High threshold should NOT detect moderate sibilance"
        );
    }

    #[test]
    fn test_gain_reduction_suggestions() {
        let mut detector = SibilanceDetector::new(44100.0);

        // No sibilance detected
        detector.sibilance_detected = false;
        assert_relative_eq!(detector.get_suggested_gain_reduction(), 1.0, epsilon = 0.01);

        // Mild sibilance
        detector.sibilance_detected = true;
        detector.sibilance_strength = 0.5;
        let mild_reduction = detector.get_suggested_gain_reduction();
        assert!(mild_reduction < 1.0 && mild_reduction > 0.5);

        // Strong sibilance
        detector.sibilance_strength = 1.5;
        let strong_reduction = detector.get_suggested_gain_reduction();
        assert!(
            strong_reduction < mild_reduction,
            "Stronger sibilance should suggest more reduction"
        );
    }

    #[test]
    fn test_stereo_processing() {
        let mut detector = SibilanceDetector::new(44100.0);
        detector.set_threshold(0.3);
        let sample_rate = 44100.0;

        // Silence baseline
        for _ in 0..500 {
            detector.process_stereo(0.0, 0.0);
        }

        // Sibilance on left channel only
        let mut detected = false;
        for i in 0..500 {
            let phase = i as f32 / sample_rate;
            let left = (2.0 * PI * 7000.0 * phase).sin() * 0.6;
            let (d, _) = detector.process_stereo(left, 0.0);
            if d {
                detected = true;
                break;
            }
        }

        assert!(detected, "Should detect sibilance in stereo signal");
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = SibilanceDetector::new(44100.0);

        // Process sibilant signal
        for i in 0..1000 {
            let sample = (i as f32 / 100.0).sin() * 0.5;
            detector.process(sample);
        }

        // Reset
        detector.reset();

        assert_relative_eq!(detector.sibilant_energy, 0.0, epsilon = 0.001);
        assert_relative_eq!(detector.full_band_energy, 0.0, epsilon = 0.001);
        assert!(!detector.is_sibilance_detected());
        assert_relative_eq!(detector.get_sibilance_strength(), 0.0, epsilon = 0.001);
    }
}
