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
            attack_time: 0.01,   // 10ms
            decay_time: 0.1,     // 100ms
            sustain_level: 0.7,  // 70%
            release_time: 0.2,   // 200ms
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

    /// Update increment values based on current parameters
    fn update_increments(&mut self) {
        let attack_samples = self.attack_time * self.sample_rate;
        self.attack_increment = 1.0 / attack_samples;
        
        let decay_samples = self.decay_time * self.sample_rate;
        self.decay_increment = (1.0 - self.sustain_level) / decay_samples;
        
        let release_samples = self.release_time * self.sample_rate;
        self.release_increment = 1.0 / release_samples;
    }

    /// Trigger the envelope (note on)
    pub fn note_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
        // Don't reset current_level to allow for retriggering
    }

    /// Release the envelope (note off)
    pub fn note_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
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
                self.current_level += self.attack_increment;
                if self.current_level >= 1.0 {
                    self.current_level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                self.current_level -= self.decay_increment;
                if self.current_level <= self.sustain_level {
                    self.current_level = self.sustain_level;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                self.current_level = self.sustain_level;
            }
            EnvelopeStage::Release => {
                self.current_level -= self.release_increment;
                if self.current_level <= 0.0 {
                    self.current_level = 0.0;
                    self.stage = EnvelopeStage::Idle;
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
        assert_eq!(stages_seen, vec![
            EnvelopeStage::Attack,
            EnvelopeStage::Decay,
            EnvelopeStage::Sustain,
            EnvelopeStage::Release,
            EnvelopeStage::Idle,
        ]);
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
                samples_to_peak >= expected_samples.saturating_sub(tolerance) &&
                samples_to_peak <= expected_samples + tolerance,
                "At {}Hz: expected ~{} samples, got {}",
                rate, expected_samples, samples_to_peak
            );
        }
    }
}
