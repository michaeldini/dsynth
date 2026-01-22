pub struct TransientDetector {
    sample_rate: f32,

    // Fast envelope (follows attacks closely)
    fast_attack_coef: f32,
    fast_release_coef: f32,
    fast_envelope: f32,

    // Slow envelope (tracks average level)
    slow_attack_coef: f32,
    slow_release_coef: f32,
    slow_envelope: f32,

    // Detection parameters
    threshold: f32, // Sensitivity (0.0 to 1.0)

    // Output state
    transient_detected: bool,
    transient_strength: f32, // How strong the transient is (0.0 to 1.0+)
}

impl TransientDetector {
    /// Create a new transient detector
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut detector = Self {
            sample_rate,
            fast_attack_coef: 0.0,
            fast_release_coef: 0.0,
            fast_envelope: 0.0,
            slow_attack_coef: 0.0,
            slow_release_coef: 0.0,
            slow_envelope: 0.0,
            threshold: 0.3,
            transient_detected: false,
            transient_strength: 0.0,
        };

        // Default time constants
        detector.set_fast_attack_time(5.0); // 5ms fast attack
        detector.set_slow_attack_time(50.0); // 50ms slow attack
        detector.set_release_time(100.0); // 100ms release

        detector
    }

    /// Set fast envelope attack time (1-20ms, typical 5ms)
    pub fn set_fast_attack_time(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(0.1, 100.0);
        self.fast_attack_coef = self.ms_to_coefficient(time_ms);
    }

    /// Set slow envelope attack time (20-100ms, typical 50ms)
    pub fn set_slow_attack_time(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(1.0, 500.0);
        self.slow_attack_coef = self.ms_to_coefficient(time_ms);
    }

    /// Set release time for both envelopes (50-200ms)
    pub fn set_release_time(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(10.0, 1000.0);
        let coef = self.ms_to_coefficient(time_ms);
        self.fast_release_coef = coef;
        self.slow_release_coef = coef;
    }

    /// Set detection threshold (0.0 = very sensitive, 1.0 = only strong transients)
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Convert milliseconds to envelope coefficient
    /// Uses exponential decay: y(t) = e^(-1/(time * sample_rate))
    fn ms_to_coefficient(&self, time_ms: f32) -> f32 {
        let time_samples = (time_ms / 1000.0) * self.sample_rate;
        (-1.0 / time_samples).exp()
    }

    /// Process a mono audio sample and detect transients
    ///
    /// # Arguments
    /// * `input` - Input audio sample
    ///
    /// # Returns
    /// Tuple of (transient_detected: bool, strength: f32)
    pub fn process(&mut self, input: f32) -> (bool, f32) {
        // Get absolute value (envelope detection works on amplitude)
        let abs_input = input.abs();

        // Update fast envelope (follows attacks closely)
        if abs_input > self.fast_envelope {
            // Attack: fast response
            self.fast_envelope += (abs_input - self.fast_envelope) * (1.0 - self.fast_attack_coef);
        } else {
            // Release: slower response
            self.fast_envelope *= self.fast_release_coef;
        }

        // Update slow envelope (tracks average level)
        if abs_input > self.slow_envelope {
            // Attack: slower response than fast envelope
            self.slow_envelope += (abs_input - self.slow_envelope) * (1.0 - self.slow_attack_coef);
        } else {
            // Release
            self.slow_envelope *= self.slow_release_coef;
        }

        // Calculate transient strength (how much fast exceeds slow)
        // When fast >> slow, we have a transient (sudden attack)
        self.transient_strength = (self.fast_envelope - self.slow_envelope).max(0.0);

        // Normalize by slow envelope to get relative strength
        // (prevents loud sustained notes from triggering)
        if self.slow_envelope > 0.001 {
            self.transient_strength /= self.slow_envelope.max(0.001);
        }

        // Clamp to reasonable range
        self.transient_strength = self.transient_strength.clamp(0.0, 2.0);

        // Detect transient based on threshold
        self.transient_detected = self.transient_strength > self.threshold;

        (self.transient_detected, self.transient_strength)
    }

    /// Process stereo audio (uses maximum of both channels)
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (bool, f32) {
        let max_input = left.abs().max(right.abs());
        self.process(max_input)
    }

    /// Get current transient state without processing new sample
    pub fn is_transient_detected(&self) -> bool {
        self.transient_detected
    }

    /// Get current transient strength (0.0 to ~2.0)
    pub fn get_transient_strength(&self) -> f32 {
        self.transient_strength
    }

    /// Reset detector state
    pub fn reset(&mut self) {
        self.fast_envelope = 0.0;
        self.slow_envelope = 0.0;
        self.transient_detected = false;
        self.transient_strength = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_transient_detector_creation() {
        let detector = TransientDetector::new(44100.0);
        assert!(!detector.is_transient_detected());
        assert_relative_eq!(detector.get_transient_strength(), 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_detects_sudden_attack() {
        let mut detector = TransientDetector::new(44100.0);
        detector.set_threshold(0.3);

        // Feed silence first to establish baseline
        for _ in 0..100 {
            detector.process(0.0);
        }

        // Sudden attack (transient)
        let mut detected = false;
        for _ in 0..50 {
            let (is_transient, _) = detector.process(0.8);
            if is_transient {
                detected = true;
                break;
            }
        }

        assert!(detected, "Should detect sudden attack as transient");
    }

    #[test]
    fn test_ignores_slow_rise() {
        let mut detector = TransientDetector::new(44100.0);
        detector.set_threshold(0.3);

        // Slow fade-in (not a transient)
        let mut max_strength: f32 = 0.0;
        for i in 0..1000 {
            let amplitude = (i as f32 / 1000.0) * 0.8; // Linear fade 0→0.8
            let (_, strength) = detector.process(amplitude);
            max_strength = max_strength.max(strength);
        }

        // Slow rise should produce low transient strength
        assert!(
            max_strength < 0.5,
            "Slow rise should not produce strong transient signal"
        );
    }

    #[test]
    fn test_threshold_sensitivity() {
        let sample_rate = 44100.0;

        // Test with low threshold (sensitive)
        let mut detector_sensitive = TransientDetector::new(sample_rate);
        detector_sensitive.set_threshold(0.1);

        // Test with high threshold (less sensitive)
        let mut detector_strict = TransientDetector::new(sample_rate);
        detector_strict.set_threshold(0.8);

        // Warm up both
        for _ in 0..100 {
            detector_sensitive.process(0.0);
            detector_strict.process(0.0);
        }

        // Moderate attack
        let mut sensitive_detected = false;
        let mut strict_detected = false;

        for _ in 0..50 {
            let (sens, _) = detector_sensitive.process(0.5);
            let (strict, _) = detector_strict.process(0.5);

            if sens {
                sensitive_detected = true;
            }
            if strict {
                strict_detected = true;
            }
        }

        assert!(
            sensitive_detected,
            "Low threshold should detect moderate attack"
        );
        assert!(
            !strict_detected,
            "High threshold should NOT detect moderate attack"
        );
    }

    #[test]
    fn test_stereo_processing() {
        let mut detector = TransientDetector::new(44100.0);
        detector.set_threshold(0.3);

        // Silence baseline
        for _ in 0..100 {
            detector.process_stereo(0.0, 0.0);
        }

        // Attack on left channel only
        let (detected, _) = detector.process_stereo(0.8, 0.0);

        // Should eventually detect (may take a few samples)
        let mut detected_within_window = detected;
        for _ in 0..20 {
            let (d, _) = detector.process_stereo(0.8, 0.0);
            if d {
                detected_within_window = true;
                break;
            }
        }

        assert!(detected_within_window, "Should detect transient in stereo");
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = TransientDetector::new(44100.0);

        // Process some signal
        for _ in 0..100 {
            detector.process(0.5);
        }

        // Reset
        detector.reset();

        assert_relative_eq!(detector.get_transient_strength(), 0.0, epsilon = 0.001);
        assert!(!detector.is_transient_detected());
    }

    #[test]
    fn test_sustained_note_no_transient() {
        let mut detector = TransientDetector::new(44100.0);
        detector.set_threshold(0.3);

        // Initial attack
        for _ in 0..100 {
            detector.process(0.8);
        }

        // Now sustain at same level (no new transient)
        let mut false_positives = 0;
        for _ in 100..500 {
            let (detected, _) = detector.process(0.8);
            if detected {
                false_positives += 1;
            }
        }

        // Should have very few false positives during sustain
        assert!(
            false_positives < 50,
            "Sustained note should not continuously trigger transient detection"
        );
    }
}
