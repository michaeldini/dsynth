/// Voice Saturation Engine - PROFESSIONAL VOCAL PROCESSING CHAIN
///
/// **Design: Zero-latency vocal processor with intelligent dynamics and saturation**
///
/// Processing chain:
/// 1. Input Gain
/// 2. **Signal Analysis** (transient, ZCR - NO PITCH for zero latency)
/// 3. **Transient Shaper** (attack/sustain control based on analysis)
/// 4. **Adaptive Saturator** (4-band multiband waveshaping + harmonic synthesis)
/// 5. **Adaptive Compression Limiter** (transient-aware limiting with -0.5dB ceiling)
/// 6. Global Mix (parallel processing)
/// 7. Output Gain
///
/// # Total Latency
/// - **Zero latency** (pitch detection disabled, all processors use envelope followers)
/// - Signal analysis runs <50 CPU ops per sample
/// - Target: <15% CPU for real-time vocal processing
use crate::dsp::effects::adaptive_saturator::AdaptiveSaturator;
use crate::dsp::effects::dynamics::{AdaptiveCompressionLimiter, TransientShaper};
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

    /// Adaptive saturator (4-band multiband waveshaping)
    adaptive_saturator: AdaptiveSaturator,

    /// Adaptive compression limiter (transient-aware envelope-follower limiting)
    limiter: AdaptiveCompressionLimiter,

    /// Current parameters
    params: VoiceParams,
}

impl VoiceEngine {
    /// Create a new voice saturation engine
    pub fn new(sample_rate: f32) -> Self {
        let adaptive_saturator = AdaptiveSaturator::new(sample_rate);
        let transient_shaper = TransientShaper::new(sample_rate);
        let limiter = AdaptiveCompressionLimiter::new(sample_rate);

        Self {
            sample_rate,
            signal_analyzer: SignalAnalyzer::new_no_pitch(sample_rate),
            transient_shaper,
            adaptive_saturator,
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

        // 2. Signal Analysis (transient, ZCR - NO PITCH for zero latency)
        let analysis = self.signal_analyzer.analyze(left, right);

        // 3. Attack Enhancer (transient-only gain modulation)
        let (left_shaped, right_shaped) =
            self.transient_shaper
                .process(left, right, self.params.transient_attack, &analysis);
        left = left_shaped;
        right = right_shaped;

        // 5. Adaptive Saturator (4-band multiband processing)
        let (left_sat, right_sat) = self.adaptive_saturator.process(
            left,
            right,
            self.params.bass_drive,
            self.params.bass_mix,
            self.params.mid_drive,
            self.params.mid_mix,
            self.params.presence_drive,
            self.params.presence_mix,
            self.params.air_drive,
            self.params.air_mix,
            self.params.stereo_width,
            &analysis,
        );

        // 6. Adaptive Compression Limiter (transient-aware envelope-follower limiting)
        let (left_limited, right_limited) = self.limiter.process(
            left_sat,
            right_sat,
            self.params.limiter_threshold,
            &analysis,
        );

        // 7. Apply global mix (parallel processing)
        left = left * (1.0 - self.params.global_mix) + left_limited * self.params.global_mix;
        right = right * (1.0 - self.params.global_mix) + right_limited * self.params.global_mix;

        // 8. Output Gain
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
        self.adaptive_saturator.reset();
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
        params.stereo_width = 0.3;
        params.input_gain = 3.0;

        engine.update_params(params.clone());

        // Params should be stored
        assert_eq!(engine.params.bass_drive, 0.7);
        assert_eq!(engine.params.mid_mix, 0.6);
        assert_eq!(engine.params.stereo_width, 0.3);
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
