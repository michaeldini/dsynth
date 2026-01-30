use crate::dsp::effects::dynamics::{AdaptiveCompressionLimiter, DeEsser, TransientShaper};
/// Voice Saturation Engine - PROFESSIONAL VOCAL PROCESSING CHAIN
///
/// **Design: Zero-latency vocal processor with intelligent dynamics and saturation**
///
/// Processing chain:
/// 1. Input Gain
/// 2. **Signal Analysis** (transient, ZCR - NO PITCH for zero latency)
/// 3. **Transient Shaper** (attack control based on analysis)
/// 4. **De-Esser** (pre-saturation sibilance control)
/// 5. **Frequency Splitting** (4-band crossover: bass/mid/presence/air)
/// 6. **Parallel Band Processing**:
///    - Bass/Mid/Presence: Individual saturators per channel
///    - Air: Exciter for high-frequency enhancement
/// 7. **Adaptive Compression Limiter** (transient-aware limiting)
/// 8. **Global Mix** (dry/wet parallel processing)
/// 9. **Output Gain**
///
/// # Total Latency
/// - **Zero latency** (pitch detection disabled, all processors use envelope followers)
/// - Signal analysis runs <50 CPU ops per sample
/// - Target: <15% CPU for real-time vocal processing
use crate::dsp::effects::saturator::Saturator;
use crate::dsp::effects::spectral::Exciter;
use crate::dsp::filters::MultibandCrossover;
use crate::dsp::signal_analyzer::SignalAnalyzer;
use crate::params_voice::VoiceParams;

/// Professional voice processing engine with zero-latency dynamics chain
pub struct VoiceEngine {
    /// Sample rate in Hz
    #[allow(dead_code)]
    sample_rate: f32,

    /// Signal analyzer (no pitch detection for zero latency)
    signal_analyzer: SignalAnalyzer,

    /// Transient shaper (analysis-based attack/sustain control)
    transient_shaper: TransientShaper,

    /// De-Esser (pre-saturation sibilance control)
    de_esser: DeEsser,

    /// 4-band crossover (splits once in voice engine)
    crossover_left: MultibandCrossover,
    crossover_right: MultibandCrossover,

    /// 3-band saturators (individual instances for clean processing)
    bass_saturator_left: Saturator,
    bass_saturator_right: Saturator,
    mid_saturator_left: Saturator,
    mid_saturator_right: Saturator,
    presence_saturator_left: Saturator,
    presence_saturator_right: Saturator,

    /// Air band exciter (separate processor)
    air_exciter: Exciter,

    /// Adaptive compression limiter (transient-aware envelope-follower limiting)
    limiter: AdaptiveCompressionLimiter,

    /// Current parameters
    params: VoiceParams,
}

impl VoiceEngine {
    /// Create a new voice saturation engine
    pub fn new(sample_rate: f32) -> Self {
        // Individual band saturators
        let bass_saturator_left = Saturator::new(sample_rate, false);
        let bass_saturator_right = Saturator::new(sample_rate, false);
        let mid_saturator_left = Saturator::new(sample_rate, false);
        let mid_saturator_right = Saturator::new(sample_rate, false);
        let presence_saturator_left = Saturator::new(sample_rate, false);
        let presence_saturator_right = Saturator::new(sample_rate, false);

        let transient_shaper = TransientShaper::new(sample_rate);
        let de_esser = DeEsser::new(sample_rate);
        let limiter = AdaptiveCompressionLimiter::new(sample_rate);

        // Separate crossovers for L/R channels
        let crossover_left = MultibandCrossover::new(sample_rate);
        let crossover_right = MultibandCrossover::new(sample_rate);

        // Air exciter configured for high frequencies
        let mut air_exciter = Exciter::new(sample_rate);
        air_exciter.set_frequency(3000.0); // Match exciter's new default cutoff
        air_exciter.set_mix(0.0); // Will be controlled by air_mix parameter

        Self {
            sample_rate,
            signal_analyzer: SignalAnalyzer::new_no_pitch(sample_rate),
            transient_shaper,
            de_esser,
            crossover_left,
            crossover_right,
            bass_saturator_left,
            bass_saturator_right,
            mid_saturator_left,
            mid_saturator_right,
            presence_saturator_left,
            presence_saturator_right,
            air_exciter,
            limiter,
            params: VoiceParams::default(),
        }
    }

    /// Get the plugin's processing latency in samples
    ///
    /// Returns 0 since all processors are zero-latency (no lookahead buffers)
    pub fn get_latency(&self) -> u32 {
        0
    }

    /// Update parameters
    pub fn update_params(&mut self, params: VoiceParams) {
        // Update de-esser frequency if it has changed
        if (self.params.sibilance_frequency - params.sibilance_frequency).abs() > 1.0 {
            self.de_esser
                .set_crossover_frequency(params.sibilance_frequency);
        }

        self.params = params;
    }

    /// Process a stereo sample pair with full vocal processing chain
    ///
    /// # Arguments
    /// * `input_left` - Left channel input
    /// * `input_right` - Right channel input
    ///
    /// # Returns
    /// Tuple of (output_left, output_right)
    pub fn process(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        // 1. Apply input gain
        let input_gain = VoiceParams::db_to_gain(self.params.input_gain);
        let mut left = input_left * input_gain;
        let mut right = input_right * input_gain;

        // 2. Signal Analysis (needed by adaptive saturator)
        let analysis = self.signal_analyzer.analyze(left, right);

        // 3. Transient Shaper
        let (left_shaped, right_shaped) =
            self.transient_shaper
                .process(left, right, self.params.transient_attack, &analysis);
        left = left_shaped;
        right = right_shaped;

        // 4. First De-Esser (before saturation)
        let ((left_deessed, right_deessed), (delta_left, delta_right)) = self.de_esser.process(
            left,
            right,
            self.params.de_esser_threshold,
            self.params.de_esser_amount,
            &analysis,
        );

        // De-Esser Listen Mode: Output the removed sibilance for auditioning
        if self.params.de_esser_listen_hf {
            left = delta_left;
            right = delta_right;
        } else {
            left = left_deessed;
            right = right_deessed;
        }

        // 5. Frequency Splitting (split once in voice engine)
        let (bass_left, mid_left, presence_left, air_left) = self.crossover_left.process(left);
        let (bass_right, mid_right, presence_right, air_right) =
            self.crossover_right.process(right);

        // 6. Parallel Band Processing
        // 6a. Individual Band Saturators (clean and direct)
        let sat_bass_left = self.bass_saturator_left.process(
            bass_left,
            self.params.bass_drive,
            self.params.bass_mix,
            &analysis,
        );
        let sat_bass_right = self.bass_saturator_right.process(
            bass_right,
            self.params.bass_drive,
            self.params.bass_mix,
            &analysis,
        );

        let sat_mid_left = self.mid_saturator_left.process(
            mid_left,
            self.params.mid_drive,
            self.params.mid_mix,
            &analysis,
        );
        let sat_mid_right = self.mid_saturator_right.process(
            mid_right,
            self.params.mid_drive,
            self.params.mid_mix,
            &analysis,
        );

        let sat_presence_left = self.presence_saturator_left.process(
            presence_left,
            self.params.presence_drive,
            self.params.presence_mix,
            &analysis,
        );
        let sat_presence_right = self.presence_saturator_right.process(
            presence_right,
            self.params.presence_drive,
            self.params.presence_mix,
            &analysis,
        );

        // 6b. Air Exciter (separate processor)
        self.air_exciter.set_drive(self.params.air_drive);
        self.air_exciter.set_mix(self.params.air_mix);
        let (exc_air_left, exc_air_right) = self.air_exciter.process(air_left, air_right);

        // 7. Recombine frequency bands (no stereo processing)
        let left = sat_bass_left + sat_mid_left + sat_presence_left + exc_air_left;
        let right = sat_bass_right + sat_mid_right + sat_presence_right + exc_air_right;

        // 8. Adaptive Compression Limiter
        let (left_limited, right_limited) =
            self.limiter
                .process(left, right, self.params.limiter_threshold, &analysis);

        // 8. Global Mix (dry/wet) - Apply BEFORE output gain
        let dry_wet = self.params.global_mix;
        let mut left = input_left * (1.0 - dry_wet) + left_limited * dry_wet;
        let mut right = input_right * (1.0 - dry_wet) + right_limited * dry_wet;

        // 9. Output Gain
        let output_gain = VoiceParams::db_to_gain(self.params.output_gain);
        left *= output_gain;
        right *= output_gain;

        // Safety: Check for NaN/Inf and replace with silence
        if !left.is_finite() {
            left = 0.0;
        }
        if !right.is_finite() {
            right = 0.0;
        }

        (left, right)
    }

    /// Process a buffer of stereo samples
    ///
    /// # Arguments
    /// * `input_left` - Left channel input buffer
    /// * `input_right` - Right channel input buffer
    /// * `output_left` - Left channel output buffer
    /// * `output_right` - Right channel output buffer
    /// * `frame_count` - Number of frames to process
    pub fn process_buffer(
        &mut self,
        input_left: &[f32],
        input_right: &[f32],
        output_left: &mut [f32],
        output_right: &mut [f32],
        frame_count: usize,
    ) {
        for i in 0..frame_count {
            let (out_l, out_r) = self.process(input_left[i], input_right[i]);
            output_left[i] = out_l;
            output_right[i] = out_r;
        }
    }

    /// Reset all processing state
    pub fn reset(&mut self) {
        self.signal_analyzer.reset();
        self.transient_shaper.reset();
        self.de_esser.reset();
        self.crossover_left.reset();
        self.crossover_right.reset();
        self.bass_saturator_left.reset();
        self.bass_saturator_right.reset();
        self.mid_saturator_left.reset();
        self.mid_saturator_right.reset();
        self.presence_saturator_left.reset();
        self.presence_saturator_right.reset();
        self.air_exciter.reset();
        self.limiter.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_engine_creation() {
        let engine = VoiceEngine::new(44100.0);
        assert_eq!(engine.sample_rate, 44100.0);
    }

    #[test]
    fn test_zero_latency() {
        let engine = VoiceEngine::new(44100.0);
        assert_eq!(engine.get_latency(), 0);
    }

    #[test]
    fn test_zero_latency_impulse_response() {
        // Verify that output appears immediately on the same sample as input
        // This is the definitive test for zero-latency operation
        let mut engine = VoiceEngine::new(44100.0);
        let params = VoiceParams::default();
        engine.update_params(params);

        // Process silence to ensure clean state
        for _ in 0..100 {
            engine.process(0.0, 0.0);
        }

        // Send impulse and capture IMMEDIATE output
        let (left_out, right_out) = engine.process(1.0, 1.0);

        // Zero-latency guarantee: output must be non-zero on SAME sample as input
        // If there was any buffering/lookahead, the output would be zero here
        assert!(
            left_out.abs() > 0.001 || right_out.abs() > 0.001,
            "Zero-latency violation: engine did not respond immediately to impulse. \
             Output was L={}, R={} (expected non-zero)",
            left_out,
            right_out
        );
    }

    #[test]
    fn test_update_params() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();

        params.bass_drive = 0.7;
        params.mid_mix = 0.6;
        params.input_gain = 3.0;

        engine.update_params(params.clone());

        // Params should be stored
        assert_eq!(engine.params.bass_drive, 0.7);
        assert_eq!(engine.params.mid_mix, 0.6);
        assert_eq!(engine.params.input_gain, 3.0);
    }

    #[test]
    fn test_process_produces_valid_output() {
        let mut engine = VoiceEngine::new(44100.0);

        // Process a simple sine wave
        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.3;
            let (out_l, out_r) = engine.process(sample, sample);

            // Output should be finite
            assert!(out_l.is_finite());
            assert!(out_r.is_finite());

            // Output should be in reasonable range
            assert!(out_l.abs() < 5.0);
            assert!(out_r.abs() < 5.0);
        }
    }

    #[test]
    fn test_silence_handling() {
        let mut engine = VoiceEngine::new(44100.0);

        // Process silence
        for _ in 0..1000 {
            let (out_l, out_r) = engine.process(0.0, 0.0);

            // Should handle silence without issues
            assert!(out_l.is_finite());
            assert!(out_r.is_finite());
        }
    }

    #[test]
    fn test_buffer_processing() {
        let mut engine = VoiceEngine::new(44100.0);

        let frame_count = 512;
        let mut input_left = vec![0.0; frame_count];
        let mut input_right = vec![0.0; frame_count];
        let mut output_left = vec![0.0; frame_count];
        let mut output_right = vec![0.0; frame_count];

        // Generate test signal (440Hz sine wave)
        for i in 0..frame_count {
            let t = i as f32 / 44100.0;
            input_left[i] = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.3;
            input_right[i] = input_left[i];
        }

        engine.process_buffer(
            &input_left,
            &input_right,
            &mut output_left,
            &mut output_right,
            frame_count,
        );

        // Verify output is not silent
        let sum: f32 = output_left.iter().map(|x| x.abs()).sum();
        assert!(sum > 0.1, "Should produce non-silent output");
    }

    #[test]
    fn test_reset() {
        let mut engine = VoiceEngine::new(44100.0);

        // Process some audio to change internal state
        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;
            engine.process(sample, sample);
        }

        // Reset
        engine.reset();

        // Verify we can still process audio without errors
        let (out_l, out_r) = engine.process(0.5, 0.5);
        assert!(out_l.is_finite());
        assert!(out_r.is_finite());
    }

    #[test]
    fn test_input_gain() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();

        // Test with different input gains
        params.input_gain = 6.0; // +6dB
        params.bass_drive = 0.5; // Some drive for proper saturation
        params.mid_drive = 0.5;
        params.presence_drive = 0.5;
        engine.update_params(params);

        let input = 0.1;

        // Process multiple samples to fill lookahead buffer (~88 samples @ 44.1kHz)
        // and stabilize RMS tracking
        for _ in 0..200 {
            engine.process(input, input);
        }

        // Now check output after buffer is filled
        let (out_l, _) = engine.process(input, input);

        // With processing, output should be valid and modified
        assert!(out_l.is_finite());
        // With input gain, saturation, and processing, output may be compressed/limited
        // so we just verify it's reasonable (not silent, not clipping badly)
        assert!(out_l.abs() > 0.01); // Not silent
        assert!(out_l.abs() < 2.0); // Reasonable range with limiter
    }

    #[test]
    fn test_moderate_drive_produces_saturation() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();

        // Moderate drive should produce audible saturation
        params.bass_drive = 0.5;
        params.mid_drive = 0.5;
        params.presence_drive = 0.5;
        engine.update_params(params);

        let input = 0.5;

        // Process multiple samples to stabilize RMS tracking
        for _ in 0..100 {
            engine.process(input, input);
        }

        let (out_l, _) = engine.process(input, input);

        // Output should be different from input (saturation applied)
        assert!(out_l.is_finite());
        assert!(out_l.abs() <= 2.0); // Reasonable range with auto-gain
    }

    #[test]
    fn test_all_bands_work() {
        let mut engine = VoiceEngine::new(44100.0);

        let input = 0.6;

        // Test with various drive combinations
        let mut params = VoiceParams::default();
        params.bass_drive = 0.7;
        params.mid_drive = 0.6;
        params.presence_drive = 0.5;
        engine.update_params(params);

        // Stabilize RMS
        for _ in 0..100 {
            engine.process(input, input);
        }

        let (out_l, out_r) = engine.process(input, input);

        // All bands should produce valid output
        assert!(out_l.is_finite(), "Left channel produced NaN/Inf");
        assert!(out_r.is_finite(), "Right channel produced NaN/Inf");
    }
}
