/// Attack Enhancer - Zero Latency, Transient-Aware
///
/// Professional attack enhancer that boosts or suppresses transients (consonants,
/// plosives, percussive elements) while leaving non-transient content untouched.
/// Uses pre-computed transient detection from SignalAnalyzer for zero-latency operation.
///
/// # Design Philosophy
/// - **Analysis-driven**: Uses `analysis.is_transient` + `transient_strength` (no lookahead)
/// - **Zero latency**: Envelope-based gain modulation with immediate response
/// - **Transient-only processing**: Only modulates during detected transients
/// - **Fixed 0.15 sensitivity**: Low threshold for consistent triggering on imperfect vocals
///
/// # How It Works
/// ```text
/// Input → Fast Envelope (1ms) → Transient Detection → Gain Modulation (attack param)
///       → Output (non-transients pass through unaffected)
/// ```
///
/// **Attack boost/cut**: When `analysis.is_transient == true`, apply attack gain
/// **Pass-through**: When no transient detected, output = input (unity gain)
///
/// # Parameters (1 total)
/// - `attack`: -1.0 to +1.0 (negative = soften transients, 0 = neutral, positive = punch)
///
/// # Use Cases
/// - Attack +0.5: Emphasize consonants/plosives for clarity
/// - Attack -0.4: Smooth harsh transients for warmth
/// - Attack +0.7: Maximum punch on percussive elements
use crate::dsp::signal_analyzer::SignalAnalysis;

/// Attack enhancer for transient processing
pub struct TransientShaper {
    #[allow(dead_code)]
    sample_rate: f32,

    /// Fixed transient sensitivity threshold (0.15 = low for imperfect vocals)
    sensitivity: f32,

    /// Fast envelope follower for attack detection (1ms)
    fast_env_left: f32,
    fast_env_right: f32,
    fast_coeff: f32,

    /// Previous transient state for edge detection
    prev_transient: bool,

    /// Gain smoothing to prevent clicks (5ms)
    target_gain_left: f32,
    target_gain_right: f32,
    current_gain_left: f32,
    current_gain_right: f32,
    gain_smooth_coeff: f32,
}

impl TransientShaper {
    /// Create new transient shaper
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        // Fast envelope: 1ms attack for transient tracking
        let fast_time_ms = 1.0;
        let fast_samples = (fast_time_ms / 1000.0) * sample_rate;
        let fast_coeff = (-1.0 / fast_samples).exp();

        // Gain smoothing: 5ms to prevent clicks during transitions
        let gain_smooth_ms = 5.0;
        let gain_smooth_samples = (gain_smooth_ms / 1000.0) * sample_rate;
        let gain_smooth_coeff = (-1.0 / gain_smooth_samples).exp();

        Self {
            sample_rate,
            sensitivity: 0.15, // Lowered from 0.4 for consistent triggering on imperfect vocals
            fast_env_left: 0.0,
            fast_env_right: 0.0,
            fast_coeff,
            prev_transient: false,
            target_gain_left: 1.0,
            target_gain_right: 1.0,
            current_gain_left: 1.0,
            current_gain_right: 1.0,
            gain_smooth_coeff,
        }
    }

    /// Process stereo sample with attack enhancement
    ///
    /// # Arguments
    /// * `left/right` - Input samples
    /// * `attack` - Attack gain adjustment (-1.0 to +1.0)
    /// * `analysis` - Pre-computed transient detection
    ///
    /// # Returns
    /// Tuple of (left_out, right_out)
    pub fn process(
        &mut self,
        left: f32,
        right: f32,
        attack: f32,
        analysis: &SignalAnalysis,
    ) -> (f32, f32) {
        // Bypass if attack is neutral
        if attack.abs() < 0.001 {
            return (left, right);
        }

        let attack = attack.clamp(-1.0, 1.0);

        // Update envelope followers
        let abs_left = left.abs();
        let abs_right = right.abs();

        // Fast envelope (tracks peaks/transients)
        self.fast_env_left =
            self.fast_env_left * self.fast_coeff + abs_left * (1.0 - self.fast_coeff);
        self.fast_env_right =
            self.fast_env_right * self.fast_coeff + abs_right * (1.0 - self.fast_coeff);

        // Determine if we're in transient phase using SignalAnalysis
        let is_transient = analysis.is_transient && analysis.transient_strength >= self.sensitivity;

        // Calculate gain multiplier - only modulate during transients
        let (gain_mult_left, gain_mult_right) = if is_transient {
            // TRANSIENT PHASE: Apply attack gain
            // Convert -1 to +1 → 0.1× to 3.0× gain range
            // attack=-1.0 → 0.1× (soften), attack=0.0 → 1.0× (neutral), attack=+1.0 → 3.0× (punch)
            let attack_gain = if attack >= 0.0 {
                1.0 + attack * 2.0 // 0 to +1 → 1.0 to 3.0
            } else {
                1.0 + attack * 0.9 // -1 to 0 → 0.1 to 1.0
            };

            // Modulate by transient strength for smooth transitions
            let strength_factor = analysis.transient_strength.min(1.0);
            let final_gain = 1.0 + (attack_gain - 1.0) * strength_factor;

            (final_gain, final_gain)
        } else {
            // NON-TRANSIENT: Pass through at unity gain (no processing)
            (1.0, 1.0)
        };

        // Smooth gain changes to prevent clicks
        self.target_gain_left = gain_mult_left;
        self.target_gain_right = gain_mult_right;

        self.current_gain_left = self.current_gain_left * self.gain_smooth_coeff
            + self.target_gain_left * (1.0 - self.gain_smooth_coeff);
        self.current_gain_right = self.current_gain_right * self.gain_smooth_coeff
            + self.target_gain_right * (1.0 - self.gain_smooth_coeff);

        // Apply gain
        let out_left = left * self.current_gain_left;
        let out_right = right * self.current_gain_right;

        // Update state
        self.prev_transient = is_transient;

        (out_left, out_right)
    }

    /// Reset all processing state
    pub fn reset(&mut self) {
        self.fast_env_left = 0.0;
        self.fast_env_right = 0.0;
        self.prev_transient = false;
        self.target_gain_left = 1.0;
        self.target_gain_right = 1.0;
        self.current_gain_left = 1.0;
        self.current_gain_right = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_analysis() -> SignalAnalysis {
        SignalAnalysis {
            rms_level: 0.3,
            peak_level: 0.5,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 200.0,
            signal_type: crate::dsp::analysis::SignalType::Tonal,
            is_voiced: true,
            is_unvoiced: false,
            has_sibilance: false,
            sibilance_strength: 0.0,
            pitch_hz: 100.0,
            pitch_confidence: 0.0,
            is_pitched: false,
        }
    }

    #[test]
    fn test_shaper_creation() {
        let shaper = TransientShaper::new(44100.0);
        assert_eq!(shaper.sample_rate, 44100.0);
        assert_eq!(shaper.sensitivity, 0.15);
    }

    #[test]
    fn test_bypass_when_neutral() {
        let mut shaper = TransientShaper::new(44100.0);
        let analysis = create_test_analysis();

        // Attack at 0.0 should bypass
        let (out_l, out_r) = shaper.process(0.5, 0.5, 0.0, &analysis);

        assert!((out_l - 0.5).abs() < 0.001);
        assert!((out_r - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_attack_boost() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();

        // Strong transient detected
        analysis.is_transient = true;
        analysis.transient_strength = 0.8;

        // Build up envelope state
        for _ in 0..100 {
            shaper.process(0.5, 0.5, 0.7, &analysis);
        }

        let (out_l, _) = shaper.process(0.5, 0.5, 0.7, &analysis);

        // Attack boost should increase level
        assert!(out_l > 0.5);
        assert!(out_l.is_finite());
    }

    #[test]
    fn test_attack_cut() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();

        analysis.is_transient = true;
        analysis.transient_strength = 0.8;

        // Build up state
        for _ in 0..100 {
            shaper.process(0.5, 0.5, -0.7, &analysis);
        }

        let (out_l, _) = shaper.process(0.5, 0.5, -0.7, &analysis);

        // Attack cut should decrease level
        assert!(out_l < 0.5);
        assert!(out_l.is_finite());
    }

    #[test]
    fn test_non_transient_passthrough() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();

        // No transient (should pass through at unity gain)
        analysis.is_transient = false;

        // Build up state
        for _ in 0..100 {
            shaper.process(0.5, 0.5, 0.7, &analysis);
        }

        let (out_l, _) = shaper.process(0.5, 0.5, 0.7, &analysis);

        // Should pass through with minimal change (unity gain)
        assert!((out_l - 0.5).abs() < 0.1);
        assert!(out_l.is_finite());
    }

    #[test]
    fn test_transient_to_non_transient_transition() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();

        // Start with transient
        analysis.is_transient = true;
        analysis.transient_strength = 0.8;

        for _ in 0..100 {
            shaper.process(0.5, 0.5, 0.5, &analysis);
        }

        let (attack_out, _) = shaper.process(0.5, 0.5, 0.5, &analysis);

        // Switch to non-transient
        analysis.is_transient = false;
        analysis.transient_strength = 0.0;

        // Allow gain smoothing to converge (5ms = ~220 samples @ 44.1kHz)
        for _ in 0..250 {
            shaper.process(0.5, 0.5, 0.5, &analysis);
        }

        let (non_transient_out, _) = shaper.process(0.5, 0.5, 0.5, &analysis);

        // Attack should be boosted, non-transient should pass through
        assert!(attack_out > 0.5);
        assert!((non_transient_out - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_sensitivity_threshold() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();

        // Transient above 0.15 sensitivity threshold
        analysis.is_transient = true;
        analysis.transient_strength = 0.3; // Above 0.15 sensitivity

        for _ in 0..100 {
            shaper.process(0.5, 0.5, 0.7, &analysis);
        }

        let (out, _) = shaper.process(0.5, 0.5, 0.7, &analysis);

        // Should apply attack gain (above threshold)
        assert!(out > 0.5);
    }

    /// ZERO-LATENCY TEST: Immediate response to transient flag change
    /// Note: Gain smoothing (5ms) means full effect takes time, but detection is immediate
    #[test]
    fn test_zero_latency_transient_response() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();

        // Start with no transient
        analysis.is_transient = false;
        for _ in 0..200 {
            shaper.process(0.5, 0.5, 0.7, &analysis);
        }

        let (non_transient_out, _) = shaper.process(0.5, 0.5, 0.7, &analysis);

        // IMMEDIATELY flip to transient
        analysis.is_transient = true;
        analysis.transient_strength = 0.8;

        // Allow a few samples for gain smoothing to start responding
        for _ in 0..50 {
            shaper.process(0.5, 0.5, 0.7, &analysis);
        }

        let (transient_out, _) = shaper.process(0.5, 0.5, 0.7, &analysis);

        // Should respond (detection is zero-latency, gain smoothing takes time)
        // After 50 samples, should be clearly different
        assert!(
            transient_out > non_transient_out * 1.1,
            "Must respond to transient flag change (detection is immediate, gain smooths over 5ms)"
        );
    }

    /// ZERO-LATENCY TEST: Impulse response
    #[test]
    fn test_zero_latency_impulse() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.is_transient = true;
        analysis.transient_strength = 0.8;

        // Send silence
        for _ in 0..100 {
            shaper.process(0.0, 0.0, 0.7, &analysis);
        }

        // Send impulse
        let (out_l, out_r) = shaper.process(1.0, 1.0, 0.7, &analysis);

        // Must respond on same sample (zero latency)
        assert!(
            out_l.abs() > 0.5,
            "Must respond immediately to impulse, got {}",
            out_l
        );
        assert!(out_r.abs() > 0.5);
    }

    /// PHASE COHERENCY TEST: Stereo sum should not cancel
    #[test]
    fn test_phase_coherency_stereo() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.is_transient = true;
        analysis.transient_strength = 0.7;

        let sample_rate = 44100.0;
        let frequency = 440.0;
        let duration_samples = 2048;

        let mut input_power = 0.0f32;
        let mut output_power = 0.0f32;

        for i in 100..duration_samples {
            let t = i as f32 / sample_rate;
            let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
            let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.5;

            let (out_l, out_r) = shaper.process(left_in, right_in, 0.5, &analysis);

            let input_sum = left_in + right_in;
            let output_sum = out_l + out_r;

            input_power += input_sum * input_sum;
            output_power += output_sum * output_sum;
        }

        let input_rms = (input_power / (duration_samples - 100) as f32).sqrt();
        let output_rms = (output_power / (duration_samples - 100) as f32).sqrt();

        // Output should maintain reasonable power (no phase cancellation)
        assert!(
            output_rms > input_rms * 0.3,
            "Phase cancellation detected: output RMS = {}, input RMS = {}",
            output_rms,
            input_rms
        );
    }

    #[test]
    fn test_parameter_clamping() {
        let mut shaper = TransientShaper::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.is_transient = true;
        analysis.transient_strength = 0.8;

        // Test extreme parameter values
        let (out_l, out_r) = shaper.process(
            1.0, 1.0, 5.0, // Out of range
            &analysis,
        );

        assert!(out_l.is_finite());
        assert!(out_r.is_finite());
    }
}
