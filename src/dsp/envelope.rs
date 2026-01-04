/// ADSR envelope generator with sample-rate-aware time calculations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Envelope {
    sample_rate: f32,
    stage: EnvelopeStage,
    current_level: f32,

    // Parameters (in seconds)
    attack_time: f32,
    decay_time: f32,
    sustain_level: f32,
    release_time: f32,

    // Curve parameters (-1.0 to +1.0)
    attack_curve: f32,
    decay_curve: f32,
    release_curve: f32,

    // Linear progress tracking for curve application
    linear_progress: f32,

    // Release start level (level when note_off was called)
    release_start_level: f32,

    // Computed increments per sample
    attack_increment: f32,
    decay_increment: f32,
    release_increment: f32,
}

impl Envelope {
    /// Create a new ADSR envelope
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut env = Self {
            sample_rate,
            stage: EnvelopeStage::Idle,
            current_level: 0.0,
            attack_time: 0.01,  // 10ms
            decay_time: 0.1,    // 100ms
            sustain_level: 0.7, // 70%
            release_time: 0.2,  // 200ms
            attack_curve: 0.0,
            decay_curve: 0.0,
            release_curve: 0.0,
            linear_progress: 0.0,
            release_start_level: 0.0,
            attack_increment: 0.0,
            decay_increment: 0.0,
            release_increment: 0.0,
        };
        env.update_increments();
        env
    }

    /// Set attack time in seconds
    pub fn set_attack(&mut self, time: f32) {
        self.attack_time = time.max(0.001); // Minimum 1ms
        self.update_increments();
    }

    /// Set decay time in seconds
    pub fn set_decay(&mut self, time: f32) {
        self.decay_time = time.max(0.001);
        self.update_increments();
    }

    /// Set sustain level (0.0 to 1.0)
    pub fn set_sustain(&mut self, level: f32) {
        self.sustain_level = level.clamp(0.0, 1.0);
    }

    /// Set release time in seconds
    pub fn set_release(&mut self, time: f32) {
        self.release_time = time.max(0.001);
        self.update_increments();
    }

    /// Set attack curve (-1.0 = logarithmic, 0.0 = linear, +1.0 = exponential)
    pub fn set_attack_curve(&mut self, curve: f32) {
        self.attack_curve = curve.clamp(-1.0, 1.0);
    }

    /// Set decay curve (-1.0 = logarithmic, 0.0 = linear, +1.0 = exponential)
    pub fn set_decay_curve(&mut self, curve: f32) {
        self.decay_curve = curve.clamp(-1.0, 1.0);
    }

    /// Set release curve (-1.0 = logarithmic, 0.0 = linear, +1.0 = exponential)
    pub fn set_release_curve(&mut self, curve: f32) {
        self.release_curve = curve.clamp(-1.0, 1.0);
    }

    /// Update increment values based on current parameters
    fn update_increments(&mut self) {
        let attack_samples = self.attack_time * self.sample_rate;
        self.attack_increment = 1.0 / attack_samples;

        let decay_samples = self.decay_time * self.sample_rate;
        self.decay_increment = 1.0 / decay_samples; // Normalized 0-1 progress

        let release_samples = self.release_time * self.sample_rate;
        self.release_increment = 1.0 / release_samples;
    }

    /// Apply curve transformation to linear progress
    /// 
    /// # Arguments
    /// * `progress` - Linear progress from 0.0 to 1.0
    /// * `curve` - Curve amount: -1.0 (logarithmic) to +1.0 (exponential)
    /// 
    /// Returns curved value from 0.0 to 1.0
    fn apply_curve(&self, progress: f32, curve: f32) -> f32 {
        if curve.abs() < 0.01 {
            // Linear (no curve)
            progress
        } else if curve > 0.0 {
            // Exponential (fast→slow): use fractional power < 1
            // powf(1.0 - curve * 0.67) gives smooth transition
            // curve=0.5 → powf(0.67), curve=1.0 → powf(0.33)
            // This creates fast initial rise that slows near the end
            progress.powf(1.0 - curve * 0.67)
        } else {
            // Logarithmic (slow→fast): use power > 1
            // This creates slow initial rise that accelerates near the end
            progress.powf(1.0 - curve * 0.67)
        }
    }

    /// Trigger the envelope (note on)
    pub fn note_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.linear_progress = 0.0;
        // Don't reset current_level to allow for retriggering
    }

    /// Release the envelope (note off)
    pub fn note_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.release_start_level = self.current_level;
            self.linear_progress = 0.0;
            self.stage = EnvelopeStage::Release;
        }
    }

    /// Process one sample and return envelope value
    pub fn process(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => {
                self.current_level = 0.0;
            }
            EnvelopeStage::Attack => {
                self.linear_progress += self.attack_increment;
                if self.linear_progress >= 1.0 {
                    self.current_level = 1.0;
                    self.linear_progress = 0.0;
                    self.stage = EnvelopeStage::Decay;
                } else {
                    // Apply curve to linear progress
                    self.current_level = self.apply_curve(self.linear_progress, self.attack_curve);
                }
            }
            EnvelopeStage::Decay => {
                self.linear_progress += self.decay_increment;
                if self.linear_progress >= 1.0 {
                    self.current_level = self.sustain_level;
                    self.linear_progress = 0.0;
                    self.stage = EnvelopeStage::Sustain;
                } else {
                    // Apply curve: start at 1.0, end at sustain_level
                    let curved_progress = self.apply_curve(self.linear_progress, self.decay_curve);
                    self.current_level = 1.0 - curved_progress * (1.0 - self.sustain_level);
                }
            }
            EnvelopeStage::Sustain => {
                self.current_level = self.sustain_level;
            }
            EnvelopeStage::Release => {
                self.linear_progress += self.release_increment;
                if self.linear_progress >= 1.0 {
                    self.current_level = 0.0;
                    self.linear_progress = 0.0;
                    self.stage = EnvelopeStage::Idle;
                } else {
                    // Apply curve: start at release_start_level, end at 0.0
                    let curved_progress = self.apply_curve(self.linear_progress, self.release_curve);
                    self.current_level = self.release_start_level * (1.0 - curved_progress);
                }
            }
        }

        self.current_level
    }

    /// Get current envelope stage
    pub fn stage(&self) -> EnvelopeStage {
        self.stage
    }

    /// Check if envelope is active (not idle)
    pub fn is_active(&self) -> bool {
        self.stage != EnvelopeStage::Idle
    }

    /// Get current level
    pub fn level(&self) -> f32 {
        self.current_level
    }

    /// Reset envelope to idle state
    pub fn reset(&mut self) {
        self.stage = EnvelopeStage::Idle;
        self.current_level = 0.0;
    }

    /// Reset current_level to zero for clean note retriggering.
    /// This should be called before note_on() when a voice is stolen/reused
    /// to prevent the envelope from starting at a non-zero level (which would
    /// cause an audible click/pop).
    pub fn reset_level(&mut self) {
        self.current_level = 0.0;
        self.linear_progress = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_envelope_creation() {
        let env = Envelope::new(44100.0);
        assert_eq!(env.sample_rate, 44100.0);
        assert_eq!(env.stage, EnvelopeStage::Idle);
        assert_eq!(env.current_level, 0.0);
    }

    #[test]
    fn test_attack_stage() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.1); // 100ms attack
        env.note_on();

        assert_eq!(env.stage(), EnvelopeStage::Attack);

        // Process attack phase
        let attack_samples = (0.1 * 44100.0) as usize;
        let mut reached_peak = false;

        for _ in 0..attack_samples + 100 {
            let level = env.process();
            if level >= 1.0 {
                reached_peak = true;
                break;
            }
        }

        assert!(reached_peak, "Should reach peak during attack");
    }

    #[test]
    fn test_decay_to_sustain() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.01);
        env.set_decay(0.1);
        env.set_sustain(0.7);
        env.note_on();

        // Process through attack
        for _ in 0..(0.01 * 44100.0) as usize + 100 {
            env.process();
            if env.stage() == EnvelopeStage::Decay {
                break;
            }
        }

        assert_eq!(env.stage(), EnvelopeStage::Decay);

        // Process through decay
        for _ in 0..(0.1 * 44100.0) as usize + 100 {
            env.process();
            if env.stage() == EnvelopeStage::Sustain {
                break;
            }
        }

        assert_eq!(env.stage(), EnvelopeStage::Sustain);
        assert_relative_eq!(env.level(), 0.7, epsilon = 0.01);
    }

    #[test]
    fn test_sustain_holds() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.01);
        env.set_decay(0.01);
        env.set_sustain(0.5);
        env.note_on();

        // Process to sustain
        for _ in 0..10000 {
            env.process();
            if env.stage() == EnvelopeStage::Sustain {
                break;
            }
        }

        // Sustain should hold steady
        let level_before = env.level();
        for _ in 0..1000 {
            env.process();
        }
        let level_after = env.level();

        assert_relative_eq!(level_before, level_after, epsilon = 0.001);
        assert_eq!(env.stage(), EnvelopeStage::Sustain);
    }

    #[test]
    fn test_release_stage() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.01);
        env.set_decay(0.01);
        env.set_sustain(0.6);
        env.set_release(0.1);
        env.note_on();

        // Process to sustain
        for _ in 0..10000 {
            env.process();
            if env.stage() == EnvelopeStage::Sustain {
                break;
            }
        }

        // Trigger release
        env.note_off();
        assert_eq!(env.stage(), EnvelopeStage::Release);

        // Process through release
        let mut reached_zero = false;
        for _ in 0..(0.15 * 44100.0) as usize {
            let level = env.process();
            if level <= 0.0 {
                reached_zero = true;
                break;
            }
        }

        assert!(reached_zero, "Should reach zero during release");
        assert_eq!(env.stage(), EnvelopeStage::Idle);
    }

    #[test]
    fn test_full_adsr_cycle() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.02);
        env.set_decay(0.05);
        env.set_sustain(0.7);
        env.set_release(0.1);

        // Start idle
        assert_eq!(env.stage(), EnvelopeStage::Idle);
        assert!(!env.is_active());

        // Trigger note
        env.note_on();
        assert!(env.is_active());

        // Track stage transitions
        let mut stages_seen = Vec::new();
        let mut sustain_samples = 0;

        for _ in 0..30000 {
            env.process();
            let current_stage = env.stage();
            if stages_seen.is_empty() || stages_seen.last() != Some(&current_stage) {
                stages_seen.push(current_stage);
            }

            // Count sustain samples and release after a while
            if current_stage == EnvelopeStage::Sustain {
                sustain_samples += 1;
                if sustain_samples > 100 {
                    env.note_off();
                }
            }

            if current_stage == EnvelopeStage::Idle && stages_seen.len() > 1 {
                break;
            }
        }

        // Should have seen all stages
        assert_eq!(
            stages_seen,
            vec![
                EnvelopeStage::Attack,
                EnvelopeStage::Decay,
                EnvelopeStage::Sustain,
                EnvelopeStage::Release,
                EnvelopeStage::Idle,
            ]
        );
    }

    #[test]
    fn test_note_off_during_attack() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.1);
        env.set_release(0.05);
        env.note_on();

        // Process part of attack
        for _ in 0..1000 {
            env.process();
        }

        assert_eq!(env.stage(), EnvelopeStage::Attack);
        let level_at_release = env.level();
        assert!(level_at_release > 0.0 && level_at_release < 1.0);

        // Release during attack
        env.note_off();
        assert_eq!(env.stage(), EnvelopeStage::Release);

        // Should decay to zero
        for _ in 0..10000 {
            env.process();
        }

        assert_eq!(env.stage(), EnvelopeStage::Idle);
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn test_retriggering() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.05);
        env.set_decay(0.05);
        env.set_sustain(0.6);
        env.note_on();

        // Process to middle of attack
        for _ in 0..1000 {
            env.process();
        }

        let level_before_retrigger = env.level();
        assert!(level_before_retrigger > 0.0 && level_before_retrigger < 1.0);

        // Retrigger
        env.note_on();
        assert_eq!(env.stage(), EnvelopeStage::Attack);

        // Should continue from current level, not reset to 0
        assert_relative_eq!(env.level(), level_before_retrigger, epsilon = 0.01);
    }

    #[test]
    fn test_parameter_ranges() {
        let mut env = Envelope::new(44100.0);

        // Test minimum times
        env.set_attack(0.0);
        assert!(env.attack_time >= 0.001);

        env.set_decay(0.0);
        assert!(env.decay_time >= 0.001);

        env.set_release(0.0);
        assert!(env.release_time >= 0.001);

        // Test sustain clamping
        env.set_sustain(-0.5);
        assert_eq!(env.sustain_level, 0.0);

        env.set_sustain(1.5);
        assert_eq!(env.sustain_level, 1.0);

        env.set_sustain(0.5);
        assert_eq!(env.sustain_level, 0.5);
    }

    #[test]
    fn test_reset() {
        let mut env = Envelope::new(44100.0);
        env.note_on();

        for _ in 0..100 {
            env.process();
        }

        env.reset();
        assert_eq!(env.stage(), EnvelopeStage::Idle);
        assert_eq!(env.level(), 0.0);
        assert!(!env.is_active());
    }

    #[test]
    fn test_sample_rate_independence() {
        // Test that envelope timing is correct at different sample rates
        let test_rates = vec![44100.0, 48000.0, 96000.0];

        for rate in test_rates {
            let mut env = Envelope::new(rate);
            env.set_attack(0.05); // 50ms
            env.note_on();

            let expected_samples = (0.05 * rate) as usize;
            let mut samples_to_peak = 0;

            for i in 0..expected_samples + 1000 {
                let level = env.process();
                if level >= 1.0 {
                    samples_to_peak = i;
                    break;
                }
            }

            // Should reach peak within 10% of expected time
            let tolerance = (expected_samples as f32 * 0.1) as usize;
            assert!(
                samples_to_peak >= expected_samples.saturating_sub(tolerance)
                    && samples_to_peak <= expected_samples + tolerance,
                "At {}Hz: expected ~{} samples, got {}",
                rate,
                expected_samples,
                samples_to_peak
            );
        }
    }

    #[test]
    fn test_apply_curve_linear() {
        let env = Envelope::new(44100.0);
        // Linear curve (curve = 0.0) should return input unchanged
        assert_relative_eq!(env.apply_curve(0.0, 0.0), 0.0, epsilon = 0.001);
        assert_relative_eq!(env.apply_curve(0.5, 0.0), 0.5, epsilon = 0.001);
        assert_relative_eq!(env.apply_curve(1.0, 0.0), 1.0, epsilon = 0.001);
    }

    #[test]
    fn test_apply_curve_exponential() {
        let env = Envelope::new(44100.0);
        // Exponential curve (curve = 1.0) should produce fast→slow behavior
        // At 50% progress, output should be > 50% (fast initial rise)
        let result = env.apply_curve(0.5, 1.0);
        assert!(
            result > 0.5,
            "Exponential curve at 50% progress should be > 50%, got {}",
            result
        );

        // At 25% progress, should already be significantly higher
        let result_early = env.apply_curve(0.25, 1.0);
        assert!(
            result_early > 0.35,
            "Exponential curve at 25% progress should be > 35%, got {}",
            result_early
        );
    }

    #[test]
    fn test_apply_curve_logarithmic() {
        let env = Envelope::new(44100.0);
        // Logarithmic curve (curve = -1.0) should produce slow→fast behavior
        // At 50% progress, output should be < 50% (slow initial rise)
        let result = env.apply_curve(0.5, -1.0);
        assert!(
            result < 0.5,
            "Logarithmic curve at 50% progress should be < 50%, got {}",
            result
        );

        // At 75% progress, should accelerate and be close to end
        let result_late = env.apply_curve(0.75, -1.0);
        assert!(
            result_late > 0.5 && result_late < 0.85,
            "Logarithmic curve at 75% progress should be between 50-85%, got {}",
            result_late
        );
    }

    #[test]
    fn test_apply_curve_bounds() {
        let env = Envelope::new(44100.0);
        // Test extreme curve values stay within 0.0-1.0
        for curve in [-1.0, -0.5, 0.0, 0.5, 1.0].iter() {
            for progress in [0.0, 0.25, 0.5, 0.75, 1.0].iter() {
                let result = env.apply_curve(*progress, *curve);
                assert!(
                    result >= 0.0 && result <= 1.0,
                    "Curve {} at progress {} produced out-of-bounds result {}",
                    curve,
                    progress,
                    result
                );
            }
        }
    }

    #[test]
    fn test_envelope_with_exponential_attack_curve() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.1); // 100ms
        env.set_attack_curve(1.0); // Exponential (fast→slow)
        env.note_on();

        // With exponential curve, should reach 50% level faster than 50% of attack time
        let half_attack_samples = (0.05 * 44100.0) as usize;
        let mut level_at_half_time = 0.0;

        for i in 0..half_attack_samples {
            level_at_half_time = env.process();
            if i == half_attack_samples - 1 {
                break;
            }
        }

        // Exponential attack should be > 50% at halfway point
        assert!(
            level_at_half_time > 0.5,
            "Exponential attack should reach > 50% at halfway point, got {}",
            level_at_half_time
        );
    }

    #[test]
    fn test_envelope_with_logarithmic_decay_curve() {
        let mut env = Envelope::new(44100.0);
        env.set_attack(0.001); // Very short attack
        env.set_decay(0.2);    // 200ms decay
        env.set_sustain(0.3);
        env.set_decay_curve(-1.0); // Logarithmic (slow→fast, hangs at high levels)
        env.note_on();

        // Process through attack to reach decay stage
        for _ in 0..(0.001 * 44100.0) as usize + 100 {
            env.process();
            if env.stage() == EnvelopeStage::Decay {
                break;
            }
        }

        // At halfway through decay, logarithmic should still be > 65%
        // (hangs at high levels longer)
        let half_decay_samples = (0.1 * 44100.0) as usize;
        let mut level_at_half_decay = 0.0;

        for i in 0..half_decay_samples {
            level_at_half_decay = env.process();
            if i == half_decay_samples - 1 {
                break;
            }
        }

        // Logarithmic decay should hang above 65% (linear would be at ~65%)
        assert!(
            level_at_half_decay > 0.65,
            "Logarithmic decay should hang above 65% at halfway point, got {}",
            level_at_half_decay
        );
    }
}
