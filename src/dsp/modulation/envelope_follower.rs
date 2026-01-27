/// RMS/Peak Envelope Follower - Tracks signal amplitude over time
///
/// An envelope follower measures the **instantaneous loudness** of an audio signal.
/// It provides smooth, continuous amplitude tracking that follows the natural shape
/// of sounds (attack, sustain, decay, release).
///
/// # Two Measurement Types
///
/// ## 1. RMS (Root Mean Square) - "Average Power"
/// - Measures **energy/loudness** (closer to human perception)
/// - Calculated as: √(average of squared samples)
/// - Better for: dynamics processing, auto-leveling, loudness matching
/// - More accurate representation of perceived volume
///
/// ## 2. Peak - "Maximum Amplitude"
/// - Measures **absolute maximum** over analysis window
/// - Tracks highest absolute value
/// - Better for: limiters, peak detection, transient emphasis
/// - Catches brief spikes that RMS might smooth out
///
/// # Use Cases
/// - Auto-gain/leveling (maintain consistent volume)
/// - Dynamics visualization (meters, waveform display)
/// - Gate threshold adaptation (intelligent noise gate)
/// - Compressor/limiter sidechain input
/// - Ducking effects (lower music when voice present)
/// - Vocal rider (automatic fader)
///
/// # Parameters
/// - **attack_ms**: How fast to respond to increases (1-100ms)
/// - **release_ms**: How fast to decay when signal drops (10-500ms)
/// - **mode**: RMS or Peak measurement
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeMode {
    RMS,  // Root Mean Square (energy-based, perceptual)
    Peak, // Peak absolute value
}

pub struct EnvelopeFollower {
    sample_rate: f32,
    mode: EnvelopeMode,

    // Envelope state
    envelope: f32,

    // Attack/Release coefficients
    attack_coef: f32,
    release_coef: f32,

    // RMS calculation buffer (for true RMS)
    rms_window_size: usize,
    rms_buffer: VecDeque<f32>, // Stores squared samples
    rms_sum: f32,              // Running sum of squared samples
}

impl EnvelopeFollower {
    /// Create a new envelope follower
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `mode` - RMS or Peak measurement
    pub fn new(sample_rate: f32, mode: EnvelopeMode) -> Self {
        let mut follower = Self {
            sample_rate,
            mode,
            envelope: 0.0,
            attack_coef: 0.0,
            release_coef: 0.0,
            rms_window_size: 0,
            rms_buffer: VecDeque::new(),
            rms_sum: 0.0,
        };

        // Default time constants
        follower.set_attack_time(10.0); // 10ms attack
        follower.set_release_time(100.0); // 100ms release
        follower.set_rms_window_size(10.0); // 10ms RMS window

        follower
    }

    /// Set envelope mode (RMS or Peak)
    pub fn set_mode(&mut self, mode: EnvelopeMode) {
        self.mode = mode;
    }

    /// Set attack time in milliseconds (1-100ms)
    /// Attack = how fast envelope rises when signal increases
    pub fn set_attack_time(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(0.1, 500.0);
        self.attack_coef = self.ms_to_coefficient(time_ms);
    }

    /// Set release time in milliseconds (10-500ms)
    /// Release = how fast envelope falls when signal decreases
    pub fn set_release_time(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(1.0, 5000.0);
        self.release_coef = self.ms_to_coefficient(time_ms);
    }

    /// Set RMS window size in milliseconds (for RMS mode only, 5-50ms typical)
    pub fn set_rms_window_size(&mut self, window_ms: f32) {
        let window_ms = window_ms.clamp(1.0, 200.0);
        self.rms_window_size = ((window_ms / 1000.0) * self.sample_rate) as usize;
        self.rms_window_size = self.rms_window_size.max(1);

        // Clear buffer when window size changes
        self.rms_buffer.clear();
        self.rms_sum = 0.0;
    }

    /// Convert milliseconds to exponential smoothing coefficient
    fn ms_to_coefficient(&self, time_ms: f32) -> f32 {
        let time_samples = (time_ms / 1000.0) * self.sample_rate;
        (-1.0 / time_samples).exp()
    }

    /// Process a mono audio sample and return envelope value
    ///
    /// # Arguments
    /// * `input` - Input audio sample
    ///
    /// # Returns
    /// Current envelope value (0.0 to 1.0+)
    pub fn process(&mut self, input: f32) -> f32 {
        let target = match self.mode {
            EnvelopeMode::RMS => self.calculate_rms(input),
            EnvelopeMode::Peak => input.abs(),
        };

        // Apply attack/release smoothing
        if target > self.envelope {
            // Attack: signal increasing
            self.envelope += (target - self.envelope) * (1.0 - self.attack_coef);
        } else {
            // Release: signal decreasing
            self.envelope += (target - self.envelope) * (1.0 - self.release_coef);
        }

        self.envelope
    }

    /// Calculate RMS (Root Mean Square) over sliding window
    fn calculate_rms(&mut self, input: f32) -> f32 {
        let squared = input * input;

        // Add new squared sample to buffer
        self.rms_buffer.push_back(squared);
        self.rms_sum += squared;

        // Remove oldest sample if window full
        if self.rms_buffer.len() > self.rms_window_size {
            if let Some(oldest) = self.rms_buffer.pop_front() {
                self.rms_sum -= oldest;
            }
        }

        // RMS = sqrt(average of squared samples)
        let window_size = self.rms_buffer.len().max(1);
        let mean_square = self.rms_sum / window_size as f32;
        mean_square.sqrt()
    }

    /// Process stereo audio (uses maximum of both channels for unified envelope)
    pub fn process_stereo(&mut self, left: f32, right: f32) -> f32 {
        let max_input = left.abs().max(right.abs());
        self.process(max_input)
    }

    /// Get current envelope value without processing new sample
    pub fn get_envelope(&self) -> f32 {
        self.envelope
    }

    /// Get envelope in dB (-inf to 0)
    pub fn get_envelope_db(&self) -> f32 {
        if self.envelope < 1e-6 {
            -120.0 // Minimum dB
        } else {
            20.0 * self.envelope.log10()
        }
    }

    /// Check if signal is above threshold (for gating applications)
    pub fn is_above_threshold(&self, threshold_linear: f32) -> bool {
        self.envelope > threshold_linear
    }

    /// Get gain reduction factor for auto-leveling
    /// Tries to maintain target level by returning a gain multiplier
    pub fn get_auto_gain(&self, target_level: f32) -> f32 {
        if self.envelope < 1e-6 {
            1.0 // No signal, unity gain
        } else {
            (target_level / self.envelope).clamp(0.1, 10.0) // Limit gain range
        }
    }

    /// Reset envelope follower state
    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.rms_buffer.clear();
        self.rms_sum = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn test_envelope_follower_creation() {
        let follower = EnvelopeFollower::new(44100.0, EnvelopeMode::RMS);
        assert_relative_eq!(follower.get_envelope(), 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_rms_mode_tracks_amplitude() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::RMS);
        follower.set_attack_time(5.0); // Fast attack
        follower.set_release_time(50.0);

        // Process constant amplitude signal
        for _ in 0..5000 {
            follower.process(0.5);
        }

        let envelope = follower.get_envelope();

        // RMS should converge close to input amplitude
        assert!(
            (envelope - 0.5).abs() < 0.1,
            "RMS envelope should track amplitude, got {}",
            envelope
        );
    }

    #[test]
    fn test_peak_mode_tracks_amplitude() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::Peak);
        follower.set_attack_time(5.0);
        follower.set_release_time(50.0);

        // Process constant amplitude signal
        for _ in 0..5000 {
            follower.process(0.7);
        }

        let envelope = follower.get_envelope();

        // Peak should converge to absolute value
        assert!(
            (envelope - 0.7).abs() < 0.1,
            "Peak envelope should track amplitude, got {}",
            envelope
        );
    }

    #[test]
    fn test_attack_response() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::Peak);
        follower.set_attack_time(10.0); // 10ms attack

        // Start from silence
        for _ in 0..100 {
            follower.process(0.0);
        }

        // Sudden increase
        let mut envelope_after_attack = 0.0;
        for _ in 0..500 {
            envelope_after_attack = follower.process(1.0);
        }

        // Should rise quickly but not instantly
        assert!(
            envelope_after_attack > 0.5,
            "Envelope should rise during attack"
        );
        assert!(
            envelope_after_attack < 1.01,
            "Envelope should not overshoot significantly"
        );
    }

    #[test]
    fn test_release_response() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::Peak);
        follower.set_attack_time(5.0);
        follower.set_release_time(50.0);

        // Build up envelope
        for _ in 0..2000 {
            follower.process(1.0);
        }

        let envelope_at_peak = follower.get_envelope();

        // Drop to silence
        for _ in 0..2000 {
            follower.process(0.0);
        }

        let envelope_after_release = follower.get_envelope();

        // Should decay
        assert!(
            envelope_after_release < envelope_at_peak * 0.5,
            "Envelope should decay during release"
        );
    }

    #[test]
    fn test_rms_vs_peak_difference() {
        let sample_rate = 44100.0;

        let mut rms_follower = EnvelopeFollower::new(sample_rate, EnvelopeMode::RMS);
        rms_follower.set_attack_time(10.0);
        rms_follower.set_release_time(100.0);

        let mut peak_follower = EnvelopeFollower::new(sample_rate, EnvelopeMode::Peak);
        peak_follower.set_attack_time(10.0);
        peak_follower.set_release_time(100.0);

        // Process sine wave (RMS should be ~0.707× peak)
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 440.0 * phase).sin();
            rms_follower.process(sample);
            peak_follower.process(sample);
        }

        let rms_env = rms_follower.get_envelope();
        let peak_env = peak_follower.get_envelope();

        // RMS of sine wave ≈ 0.707 × peak
        assert!(
            rms_env < peak_env,
            "RMS should be less than peak for sine wave"
        );
        assert!(
            (rms_env / peak_env - 0.707).abs() < 0.15,
            "RMS/Peak ratio should be ~0.707 for sine, got {}",
            rms_env / peak_env
        );
    }

    #[test]
    fn test_stereo_processing() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::Peak);
        follower.set_attack_time(5.0);

        // Process stereo signal (different amplitudes)
        for _ in 0..2000 {
            follower.process_stereo(0.3, 0.8); // Right channel louder
        }

        let envelope = follower.get_envelope();

        // Should track louder channel (0.8)
        assert!(
            (envelope - 0.8).abs() < 0.1,
            "Stereo envelope should track louder channel"
        );
    }

    #[test]
    fn test_db_conversion() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::Peak);

        follower.envelope = 1.0;
        assert_relative_eq!(follower.get_envelope_db(), 0.0, epsilon = 0.1);

        follower.envelope = 0.5;
        let db = follower.get_envelope_db();
        assert!(db < 0.0 && db > -10.0, "0.5 amplitude ≈ -6dB");

        follower.envelope = 0.0;
        assert!(
            follower.get_envelope_db() < -100.0,
            "Silence should be very low dB"
        );
    }

    #[test]
    fn test_threshold_detection() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::RMS);

        // Build envelope above threshold
        for _ in 0..2000 {
            follower.process(0.5);
        }

        assert!(
            follower.is_above_threshold(0.3),
            "Should be above threshold"
        );
        assert!(
            !follower.is_above_threshold(0.7),
            "Should be below threshold"
        );
    }

    #[test]
    fn test_auto_gain() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::RMS);

        // Process signal at 0.5 amplitude
        for _ in 0..2000 {
            follower.process(0.5);
        }

        let gain = follower.get_auto_gain(0.7); // Target 0.7

        // Gain should be > 1.0 to boost 0.5 → 0.7
        assert!(gain > 1.0, "Auto-gain should boost low signal");
        assert!(
            (0.5 * gain - 0.7).abs() < 0.2,
            "Auto-gain should reach target"
        );
    }

    #[test]
    fn test_reset_clears_state() {
        let mut follower = EnvelopeFollower::new(44100.0, EnvelopeMode::RMS);

        // Build up envelope
        for _ in 0..2000 {
            follower.process(0.8);
        }

        // Reset
        follower.reset();

        assert_relative_eq!(follower.get_envelope(), 0.0, epsilon = 0.001);
        assert_eq!(follower.rms_buffer.len(), 0);
        assert_relative_eq!(follower.rms_sum, 0.0, epsilon = 0.001);
    }
}
