/// Noise gate effect
///
/// Reduces or eliminates audio signal when it falls below a threshold. Useful for:
/// - Removing background noise during vocal silence
/// - Cleaning up microphone handling noise
/// - Tightening drum recordings
///
/// # Features
/// - RMS envelope follower for smooth tracking
/// - Attack/release/hold timing controls
/// - Expansion ratio (like a compressor in reverse)
/// - Hysteresis to prevent chattering
use std::f32::consts::PI;

/// Noise gate with RMS envelope detection
pub struct NoiseGate {
    sample_rate: f32,
    
    /// Gate parameters
    threshold_db: f32,  // -80dB to -20dB
    ratio: f32,         // 1:1 to 10:1 (expansion ratio)
    attack_ms: f32,     // 0.1-50ms
    release_ms: f32,    // 10-1000ms
    hold_ms: f32,       // 0-500ms (hold gate open after signal drops below threshold)
    
    /// Envelope follower state (separate for L/R)
    rms_envelope_left: f32,
    rms_envelope_right: f32,
    
    /// Gate state
    gain: f32,           // Current gate gain (0.0-1.0)
    hold_counter: usize, // Samples remaining in hold phase
    
    /// Time constants (converted from ms to coefficients)
    attack_coeff: f32,
    release_coeff: f32,
    rms_coeff: f32, // RMS envelope follower time constant
    
    /// Hysteresis (prevent chattering near threshold)
    hysteresis_db: f32,
}

impl NoiseGate {
    /// Create a new noise gate
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut gate = Self {
            sample_rate,
            threshold_db: -40.0,
            ratio: 4.0,
            attack_ms: 1.0,
            release_ms: 100.0,
            hold_ms: 50.0,
            rms_envelope_left: 0.0,
            rms_envelope_right: 0.0,
            gain: 1.0,
            hold_counter: 0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            rms_coeff: 0.0,
            hysteresis_db: 3.0, // 3dB hysteresis
        };
        
        gate.update_coefficients();
        gate
    }
    
    /// Set threshold in dB
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db.clamp(-80.0, -20.0);
    }
    
    /// Set expansion ratio (1:1 = no gating, 10:1 = aggressive)
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(1.0, 10.0);
    }
    
    /// Set attack time in milliseconds
    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack_ms = attack_ms.clamp(0.1, 50.0);
        self.attack_coeff = Self::ms_to_coeff(self.attack_ms, self.sample_rate);
    }
    
    /// Set release time in milliseconds
    pub fn set_release(&mut self, release_ms: f32) {
        self.release_ms = release_ms.clamp(10.0, 1000.0);
        self.release_coeff = Self::ms_to_coeff(self.release_ms, self.sample_rate);
    }
    
    /// Set hold time in milliseconds
    pub fn set_hold(&mut self, hold_ms: f32) {
        self.hold_ms = hold_ms.clamp(0.0, 500.0);
    }
    
    /// Convert milliseconds to exponential decay coefficient
    ///
    /// Used for smooth attack/release envelopes
    fn ms_to_coeff(time_ms: f32, sample_rate: f32) -> f32 {
        if time_ms <= 0.0 {
            return 0.0;
        }
        
        // Calculate coefficient for exponential smoothing
        // Reaches ~63% of target in time_ms milliseconds
        let time_sec = time_ms / 1000.0;
        let samples = time_sec * sample_rate;
        
        (-1.0 / samples).exp()
    }
    
    /// Update all time-based coefficients
    fn update_coefficients(&mut self) {
        self.attack_coeff = Self::ms_to_coeff(self.attack_ms, self.sample_rate);
        self.release_coeff = Self::ms_to_coeff(self.release_ms, self.sample_rate);
        
        // RMS envelope: fast enough to track transients but smooth enough to average
        self.rms_coeff = Self::ms_to_coeff(10.0, self.sample_rate); // 10ms RMS window
    }
    
    /// Convert dB to linear gain
    #[inline]
    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }
    
    /// Convert linear level to dB
    #[inline]
    fn linear_to_db(linear: f32) -> f32 {
        20.0 * linear.max(1e-10).log10()
    }
    
    /// Process one stereo sample pair
    ///
    /// # Arguments
    /// * `left` - Left channel input sample
    /// * `right` - Right channel input sample
    ///
    /// # Returns
    /// Tuple of (left, right) output samples
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Step 1: Update RMS envelope followers
        let left_squared = left * left;
        let right_squared = right * right;
        
        self.rms_envelope_left = self.rms_envelope_left * self.rms_coeff 
            + left_squared * (1.0 - self.rms_coeff);
        self.rms_envelope_right = self.rms_envelope_right * self.rms_coeff 
            + right_squared * (1.0 - self.rms_coeff);
        
        // Step 2: Calculate input level (dB) - use max of L/R for linked stereo gating
        let rms_left = self.rms_envelope_left.sqrt();
        let rms_right = self.rms_envelope_right.sqrt();
        let input_level_linear = rms_left.max(rms_right);
        let input_level_db = Self::linear_to_db(input_level_linear);
        
        // Step 3: Determine target gain based on threshold with hysteresis
        let target_gain = if input_level_db > self.threshold_db {
            // Signal above threshold (with hysteresis) - gate is open
            self.hold_counter = (self.hold_ms * self.sample_rate / 1000.0) as usize;
            1.0
        } else if input_level_db < self.threshold_db - self.hysteresis_db || self.hold_counter == 0 {
            // Signal well below threshold and hold expired - calculate expansion gain
            let diff_db = input_level_db - self.threshold_db;
            let gain_reduction_db = diff_db * (1.0 - 1.0 / self.ratio);
            Self::db_to_linear(gain_reduction_db).max(0.0)
        } else {
            // In hold phase - keep gate open
            if self.hold_counter > 0 {
                self.hold_counter -= 1;
            }
            1.0
        };
        
        // Step 4: Smooth gain changes (attack/release)
        let coeff = if target_gain > self.gain {
            self.attack_coeff  // Opening gate (attack)
        } else {
            self.release_coeff // Closing gate (release)
        };
        
        self.gain = self.gain * coeff + target_gain * (1.0 - coeff);
        
        // Step 5: Apply gain to input
        (left * self.gain, right * self.gain)
    }
    
    /// Reset gate state
    pub fn reset(&mut self) {
        self.rms_envelope_left = 0.0;
        self.rms_envelope_right = 0.0;
        self.gain = 1.0;
        self.hold_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_noise_gate_creation() {
        let gate = NoiseGate::new(44100.0);
        assert_eq!(gate.sample_rate, 44100.0);
        assert_eq!(gate.threshold_db, -40.0);
        assert_eq!(gate.ratio, 4.0);
    }
    
    #[test]
    fn test_threshold_clamping() {
        let mut gate = NoiseGate::new(44100.0);
        
        gate.set_threshold(-100.0);
        assert_eq!(gate.threshold_db, -80.0);
        
        gate.set_threshold(-10.0);
        assert_eq!(gate.threshold_db, -20.0);
        
        gate.set_threshold(-50.0);
        assert_eq!(gate.threshold_db, -50.0);
    }
    
    #[test]
    fn test_ratio_clamping() {
        let mut gate = NoiseGate::new(44100.0);
        
        gate.set_ratio(0.5);
        assert_eq!(gate.ratio, 1.0);
        
        gate.set_ratio(20.0);
        assert_eq!(gate.ratio, 10.0);
        
        gate.set_ratio(4.0);
        assert_eq!(gate.ratio, 4.0);
    }
    
    #[test]
    fn test_gate_passes_loud_signal() {
        let mut gate = NoiseGate::new(44100.0);
        gate.set_threshold(-40.0);
        gate.set_attack(0.1);
        gate.set_release(100.0);
        
        // Feed loud signal (should pass through)
        let loud_signal = 0.5; // ~-6dB
        let mut output_sum = 0.0;
        
        for _ in 0..1000 {
            let (left, right) = gate.process(loud_signal, loud_signal);
            output_sum += (left.abs() + right.abs()) / 2.0;
        }
        
        // Output should be significant (gate open)
        assert!(output_sum > 100.0, "Gate should pass loud signal");
    }
    
    #[test]
    fn test_gate_blocks_quiet_signal() {
        let mut gate = NoiseGate::new(44100.0);
        gate.set_threshold(-40.0);
        gate.set_attack(0.1);
        gate.set_release(10.0); // Fast release for test
        gate.set_hold(0.0); // No hold
        
        // Feed quiet signal (should be blocked)
        let quiet_signal = 0.001; // ~-60dB
        let mut output_sum = 0.0;
        
        // Process enough samples for release to take effect
        for _ in 0..5000 {
            let (left, right) = gate.process(quiet_signal, quiet_signal);
            output_sum += (left.abs() + right.abs()) / 2.0;
        }
        
        // Output should be much less than input (gate closed)
        let input_sum = quiet_signal * 5000.0 * 2.0;
        assert!(output_sum < input_sum * 0.5, "Gate should reduce quiet signal");
    }
    
    #[test]
    fn test_hold_time() {
        let mut gate = NoiseGate::new(44100.0);
        gate.set_threshold(-40.0);
        gate.set_attack(0.1);
        gate.set_release(10.0);
        gate.set_hold(100.0); // 100ms hold
        
        // Open gate with loud signal
        for _ in 0..1000 {
            gate.process(0.5, 0.5);
        }
        
        // Switch to quiet signal
        let quiet = 0.001;
        let mut outputs = Vec::new();
        
        for _ in 0..10000 {
            let (left, _) = gate.process(quiet, quiet);
            outputs.push(left.abs());
        }
        
        // During hold period, output should stay relatively high
        let early_avg = outputs[0..1000].iter().sum::<f32>() / 1000.0;
        
        // After hold + release, output should be lower
        let late_avg = outputs[8000..9000].iter().sum::<f32>() / 1000.0;
        
        assert!(early_avg > late_avg, "Hold should delay gate closing");
    }
    
    #[test]
    fn test_db_conversion() {
        assert_relative_eq!(NoiseGate::db_to_linear(0.0), 1.0, epsilon = 0.001);
        assert_relative_eq!(NoiseGate::db_to_linear(-6.0), 0.5, epsilon = 0.01);
        assert_relative_eq!(NoiseGate::db_to_linear(-20.0), 0.1, epsilon = 0.001);
        
        assert_relative_eq!(NoiseGate::linear_to_db(1.0), 0.0, epsilon = 0.001);
        assert_relative_eq!(NoiseGate::linear_to_db(0.5), -6.0, epsilon = 0.1);
    }
    
    #[test]
    fn test_reset() {
        let mut gate = NoiseGate::new(44100.0);
        
        // Process some signal to build up state
        for _ in 0..1000 {
            gate.process(0.5, 0.5);
        }
        
        gate.reset();
        
        assert_eq!(gate.rms_envelope_left, 0.0);
        assert_eq!(gate.rms_envelope_right, 0.0);
        assert_eq!(gate.gain, 1.0);
        assert_eq!(gate.hold_counter, 0);
    }
    
    #[test]
    fn test_stereo_linked() {
        let mut gate = NoiseGate::new(44100.0);
        gate.set_threshold(-40.0);
        
        // Left channel loud, right channel quiet
        // Gate should stay open (linked mode uses max of L/R)
        for _ in 0..1000 {
            gate.process(0.5, 0.001);
        }
        
        // Both channels should have same gain applied
        let (left, right) = gate.process(0.5, 0.001);
        let left_gain = left / 0.5;
        let right_gain = right / 0.001;
        
        assert_relative_eq!(left_gain, right_gain, epsilon = 0.01);
    }
}
