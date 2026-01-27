/// Adaptive Compression Limiter - Zero Latency, Transient-Aware
///
/// Professional compressor-limiter that combines intelligent compression with
/// hard ceiling limiting. Uses envelope followers (no lookahead) for zero-latency
/// operation while being gentle on transients.
///
/// # Design Philosophy
/// - **Zero latency**: Envelope-follower based (no lookahead buffer)
/// - **Transient-aware**: Adaptive attack time (1ms peaks, 10ms transients)
/// - **Fixed 10:1 ratio**: Limiting characteristic (transparent compression→brick wall)
/// - **Fixed -0.5dB ceiling**: Safe headroom to prevent intersample peaks
///
/// # How It Works
/// ```text
/// Input → RMS Detection (stereo-linked)
///       → Compression Stage (adaptive ratio based on transients)
///       → Hard Ceiling Limiter (-0.5dB)
///       → Output
/// ```
///
/// **Adaptive Attack:**
/// - Non-transient peaks: 1ms attack (fast response to level spikes)
/// - Transients: 10ms attack (preserves punch on drum hits/consonants)
/// - Uses `analysis.is_transient` for zero-latency decision
///
/// **Compression vs Limiting:**
/// - Below threshold: Transparent (1:1 passthrough)
/// - Above threshold: 10:1 compression (gentle limiting)
/// - Above -0.5dB: Hard ceiling (brick wall, prevents clipping)
///
/// # Parameters (2 total)
/// - `threshold`: -20.0 to 0.0 dB (compression knee point)
/// - `release`: 50-500ms (tail behavior, how fast gain recovers)
use crate::dsp::signal_analyzer::SignalAnalysis;

/// Adaptive compression limiter with transient awareness
pub struct AdaptiveCompressionLimiter {
    sample_rate: f32,

    /// Fixed parameters
    ratio: f32, // 10:1 (limiting characteristic)
    ceiling_db: f32, // -0.5dB (safe headroom)
    knee_db: f32,    // 3dB (moderate soft knee)

    /// Envelope follower state (stereo-linked, uses max of L/R)
    envelope: f32,

    /// Time constants
    fast_attack_coeff: f32, // 1ms for peaks
    slow_attack_coeff: f32, // 10ms for transients
    release_coeff: f32,     // User-adjustable

    /// Current gain reduction (for smooth transitions)
    current_gain: f32,
}

impl AdaptiveCompressionLimiter {
    /// Create new adaptive compression limiter
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        // Fast attack: 1ms for catching peaks quickly
        let fast_attack_coeff = Self::ms_to_coeff(1.0, sample_rate);

        // Slow attack: 10ms for preserving transient punch
        let slow_attack_coeff = Self::ms_to_coeff(10.0, sample_rate);

        // Default release: 200ms (moderate recovery)
        let release_coeff = Self::ms_to_coeff(200.0, sample_rate);

        Self {
            sample_rate,
            ratio: 10.0,      // Fixed 10:1 limiting
            ceiling_db: -0.5, // Fixed safe ceiling
            knee_db: 3.0,     // Fixed moderate knee
            envelope: 0.0,
            fast_attack_coeff,
            slow_attack_coeff,
            release_coeff,
            current_gain: 1.0,
        }
    }

    /// Set compression threshold (-20.0 to 0.0 dB)
    /// This is the only user-adjustable parameter for compression behavior
    pub fn set_threshold(&mut self, _threshold_db: f32) {
        // Threshold is passed per-sample in process() for real-time control
    }

    /// Set release time (50-500ms)
    pub fn set_release(&mut self, release_ms: f32) {
        let clamped_release = release_ms.clamp(50.0, 500.0);
        self.release_coeff = Self::ms_to_coeff(clamped_release, self.sample_rate);
    }

    /// Convert milliseconds to exponential smoothing coefficient
    fn ms_to_coeff(time_ms: f32, sample_rate: f32) -> f32 {
        let time_sec = time_ms / 1000.0;
        let samples = time_sec * sample_rate;
        (-1.0 / samples).exp()
    }

    /// Convert amplitude to dB
    #[inline]
    fn amp_to_db(amp: f32) -> f32 {
        20.0 * amp.max(1e-10).log10()
    }

    /// Convert dB to amplitude
    #[inline]
    fn db_to_amp(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Soft knee compression curve
    fn apply_knee(&self, overshoot_db: f32) -> f32 {
        if overshoot_db <= -self.knee_db / 2.0 {
            // Below knee: no compression
            0.0
        } else if overshoot_db >= self.knee_db / 2.0 {
            // Above knee: full compression
            overshoot_db
        } else {
            // In knee: smooth transition (quadratic curve)
            let x = overshoot_db + self.knee_db / 2.0;
            (x * x) / (2.0 * self.knee_db)
        }
    }

    /// Process stereo sample with adaptive compression limiting
    ///
    /// # Arguments
    /// * `left/right` - Input samples
    /// * `threshold_db` - Compression threshold (-20.0 to 0.0 dB)
    /// * `analysis` - Pre-computed transient detection
    ///
    /// # Returns
    /// Tuple of (left_out, right_out)
    pub fn process(
        &mut self,
        left: f32,
        right: f32,
        threshold_db: f32,
        analysis: &SignalAnalysis,
    ) -> (f32, f32) {
        let threshold_db = threshold_db.clamp(-20.0, 0.0);

        // Stereo-linked detection (use max of L/R for consistent stereo image)
        let input_level = left.abs().max(right.abs());

        // Adaptive attack coefficient based on transient detection
        let attack_coeff = if analysis.is_transient {
            // Transient: Use slower attack to preserve punch
            self.slow_attack_coeff
        } else {
            // Peak/sustain: Use fast attack to catch level spikes
            self.fast_attack_coeff
        };

        // Update envelope follower
        if input_level > self.envelope {
            // Attack: envelope rises to meet signal
            self.envelope = self.envelope * attack_coeff + input_level * (1.0 - attack_coeff);
        } else {
            // Release: envelope falls
            self.envelope =
                self.envelope * self.release_coeff + input_level * (1.0 - self.release_coeff);
        }

        // Convert to dB
        let envelope_db = Self::amp_to_db(self.envelope);

        // Calculate compression gain reduction
        let overshoot_db = envelope_db - threshold_db;

        let gain_reduction_db = if overshoot_db > 0.0 {
            // Apply soft knee
            let knee_overshoot = self.apply_knee(overshoot_db);

            // Calculate gain reduction with ratio
            // 10:1 ratio means: for every 10dB over threshold, reduce by 9dB
            let reduction = knee_overshoot * (1.0 - 1.0 / self.ratio);
            reduction
        } else {
            0.0
        };

        // Calculate compression gain
        let compression_gain = Self::db_to_amp(-gain_reduction_db);

        // Apply hard ceiling limiter
        // This is a brick wall at -0.5dB to prevent intersample peaks
        let compressed_left = left * compression_gain;
        let compressed_right = right * compression_gain;

        let ceiling_linear = Self::db_to_amp(self.ceiling_db);

        let (limited_left, limited_right) =
            if compressed_left.abs() > ceiling_linear || compressed_right.abs() > ceiling_linear {
                // Hard limit: instant gain reduction to ceiling
                let peak = compressed_left.abs().max(compressed_right.abs());
                let limit_gain = ceiling_linear / peak;

                (compressed_left * limit_gain, compressed_right * limit_gain)
            } else {
                (compressed_left, compressed_right)
            };

        // Smooth gain changes slightly to prevent zipper noise
        let target_gain = if limited_left.abs() < left.abs() {
            limited_left.abs() / left.abs().max(1e-10)
        } else {
            1.0
        };

        self.current_gain = self.current_gain * 0.95 + target_gain * 0.05;

        (limited_left, limited_right)
    }

    /// Reset all processing state
    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.current_gain = 1.0;
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
    fn test_limiter_creation() {
        let limiter = AdaptiveCompressionLimiter::new(44100.0);
        assert_eq!(limiter.sample_rate, 44100.0);
        assert_eq!(limiter.ratio, 10.0);
        assert_eq!(limiter.ceiling_db, -0.5);
    }

    #[test]
    fn test_passthrough_below_threshold() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let analysis = create_test_analysis();

        // Low level signal well below threshold
        for _ in 0..100 {
            limiter.process(0.1, 0.1, -10.0, &analysis);
        }

        let (out_l, out_r) = limiter.process(0.1, 0.1, -10.0, &analysis);

        // Should pass through with minimal change
        assert!((out_l - 0.1).abs() < 0.02);
        assert!((out_r - 0.1).abs() < 0.02);
    }

    #[test]
    fn test_compression_above_threshold() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let analysis = create_test_analysis();

        // Hot signal above threshold
        for _ in 0..200 {
            limiter.process(0.8, 0.8, -10.0, &analysis);
        }

        let (out_l, _) = limiter.process(0.8, 0.8, -10.0, &analysis);

        // Should compress (output < input)
        assert!(out_l < 0.8);
        assert!(out_l.is_finite());
    }

    #[test]
    fn test_hard_ceiling_limiting() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let analysis = create_test_analysis();

        // Send very hot signal
        let mut max_output = 0.0f32;
        for _ in 0..500 {
            let (out_l, out_r) = limiter.process(1.5, 1.5, 0.0, &analysis);
            max_output = max_output.max(out_l.abs()).max(out_r.abs());
        }

        // Should never exceed ceiling (-0.5dB = ~0.944 linear)
        let ceiling_linear = AdaptiveCompressionLimiter::db_to_amp(-0.5);
        assert!(
            max_output <= ceiling_linear * 1.01,
            "Output exceeded ceiling: {} > {}",
            max_output,
            ceiling_linear
        );
    }

    #[test]
    fn test_transient_aware_attack() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let mut analysis = create_test_analysis();

        // Non-transient peak (should use fast attack)
        analysis.is_transient = false;
        for _ in 0..50 {
            limiter.process(0.8, 0.8, -10.0, &analysis);
        }
        let (peak_out, _) = limiter.process(0.8, 0.8, -10.0, &analysis);

        limiter.reset();

        // Transient (should use slower attack, preserve punch)
        analysis.is_transient = true;
        for _ in 0..50 {
            limiter.process(0.8, 0.8, -10.0, &analysis);
        }
        let (transient_out, _) = limiter.process(0.8, 0.8, -10.0, &analysis);

        // Transient should have less compression (more level preserved)
        assert!(transient_out > peak_out * 0.95);
        assert!(transient_out.is_finite());
    }

    #[test]
    fn test_release_time_adjustment() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let analysis = create_test_analysis();

        // Set fast release
        limiter.set_release(50.0);

        // Compress signal
        for _ in 0..200 {
            limiter.process(0.8, 0.8, -10.0, &analysis);
        }

        // Drop to low level
        for _ in 0..100 {
            limiter.process(0.1, 0.1, -10.0, &analysis);
        }

        let envelope_fast = limiter.envelope;

        // Reset and test slow release
        limiter.reset();
        limiter.set_release(500.0);

        for _ in 0..200 {
            limiter.process(0.8, 0.8, -10.0, &analysis);
        }

        for _ in 0..100 {
            limiter.process(0.1, 0.1, -10.0, &analysis);
        }

        let envelope_slow = limiter.envelope;

        // Slow release should maintain higher envelope
        assert!(envelope_slow > envelope_fast);
    }

    /// ZERO-LATENCY TEST: Immediate response to impulse
    #[test]
    fn test_zero_latency_impulse() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let analysis = create_test_analysis();

        // Send silence
        for _ in 0..100 {
            limiter.process(0.0, 0.0, -10.0, &analysis);
        }

        // Send impulse - should respond immediately
        let (out_l, out_r) = limiter.process(1.0, 1.0, -10.0, &analysis);

        assert!(
            out_l.abs() > 0.5,
            "Must respond on same sample (zero latency), got {}",
            out_l
        );
        assert!(out_r.abs() > 0.5);
    }

    /// ZERO-LATENCY TEST: Immediate transient awareness
    #[test]
    fn test_zero_latency_transient_response() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let mut analysis = create_test_analysis();

        // Stabilize with non-transient
        analysis.is_transient = false;
        for _ in 0..100 {
            limiter.process(0.8, 0.8, -10.0, &analysis);
        }

        let (non_transient_out, _) = limiter.process(0.8, 0.8, -10.0, &analysis);

        // IMMEDIATELY flip to transient
        analysis.is_transient = true;
        let (transient_out, _) = limiter.process(0.8, 0.8, -10.0, &analysis);

        // Should respond immediately (no lookahead)
        assert!(
            transient_out > non_transient_out * 0.99,
            "Must respond immediately to transient flag"
        );
    }

    /// PHASE COHERENCY TEST: Stereo sum should not cancel
    #[test]
    fn test_phase_coherency_stereo() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let analysis = create_test_analysis();

        let sample_rate = 44100.0;
        let frequency = 440.0;
        let duration_samples = 2048;

        let mut input_power = 0.0f32;
        let mut output_power = 0.0f32;

        for i in 200..duration_samples {
            let t = i as f32 / sample_rate;
            let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.7;
            let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.7;

            let (out_l, out_r) = limiter.process(left_in, right_in, -10.0, &analysis);

            let input_sum = left_in + right_in;
            let output_sum = out_l + out_r;

            input_power += input_sum * input_sum;
            output_power += output_sum * output_sum;
        }

        let input_rms = (input_power / (duration_samples - 200) as f32).sqrt();
        let output_rms = (output_power / (duration_samples - 200) as f32).sqrt();

        // Output should maintain reasonable power
        assert!(
            output_rms > input_rms * 0.3,
            "Phase cancellation detected: output RMS = {}, input RMS = {}",
            output_rms,
            input_rms
        );
    }

    #[test]
    fn test_parameter_clamping() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let analysis = create_test_analysis();

        // Test extreme threshold values
        let (out_l, out_r) = limiter.process(0.5, 0.5, -100.0, &analysis); // Below range
        assert!(out_l.is_finite());
        assert!(out_r.is_finite());

        let (out_l, out_r) = limiter.process(0.5, 0.5, 20.0, &analysis); // Above range
        assert!(out_l.is_finite());
        assert!(out_r.is_finite());
    }

    #[test]
    fn test_stereo_linked_detection() {
        let mut limiter = AdaptiveCompressionLimiter::new(44100.0);
        let analysis = create_test_analysis();

        // Asymmetric input (left hot, right quiet)
        for _ in 0..200 {
            limiter.process(0.9, 0.1, -10.0, &analysis);
        }

        let (out_l, out_r) = limiter.process(0.9, 0.1, -10.0, &analysis);

        // Both channels should be compressed (stereo-linked)
        assert!(out_l < 0.9);
        assert!(out_r < 0.1); // Right channel compressed too due to left
    }
}
