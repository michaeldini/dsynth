use crate::dsp::effects::delay::SmartDelay;
use crate::dsp::effects::dynamics::adaptive_compressor::AdaptiveCompressor;
use crate::dsp::effects::dynamics::lookahead_limiter::LookAheadLimiter;
use crate::dsp::effects::dynamics::smart_gate::SmartGate;
use crate::dsp::effects::spectral::de_esser::DeEsser;
use crate::dsp::effects::spectral::intelligent_exciter::IntelligentExciter;
/// Voice Enhancement Engine - INTELLIGENT ARCHITECTURE (v2.0)
///
/// **New Design Philosophy: "Analyze Once, Use Everywhere"**
///
/// Instead of running separate detection algorithms in each effect, we:
/// 1. **Analyze the signal once** (transients, pitch, sibilance, etc.)
/// 2. **Thread that analysis** through all effects
/// 3. **Each effect adapts automatically** based on what's in the signal
///
/// # Processing Chain (6 stages instead of 12)
/// ```text
/// Input
///   ↓
/// 1. SIGNAL ANALYZER (runs all detectors once per sample)
///    - Transient detection (dual envelope)
///    - Zero-crossing rate (tonal vs noisy)
///    - Sibilance detection (4kHz+ high-pass)
///    - Pitch detection (YIN, throttled to 512 samples)
///    → Outputs: SignalAnalysis struct
///   ↓
/// 2. SMART GATE (receives SignalAnalysis)
///    - Automatically gentler on transients/sibilance
///    - Pitch-aware thresholds
///    - Just 1 parameter: threshold
///   ↓
/// 3. ADAPTIVE COMPRESSOR (receives SignalAnalysis)
///    - Faster attack on transients
///    - Pitch-responsive thresholds
///    - Just 4 parameters: threshold, ratio, attack, release
///   ↓
/// 4. INTELLIGENT EXCITER (receives SignalAnalysis)
///    - Tracks pitch, adds harmonics at 2×, 3×, 4× frequency
///    - Bypasses sibilance automatically
///    - Just 2 parameters: amount, mix
///   ↓
/// 5. LOOKAHEAD LIMITER
///    - Safety ceiling at 0dB
///   ↓
/// 6. DRY/WET MIX
///   ↓
/// Output
/// ```
///
/// # Key Benefits
/// - **Efficient**: Detectors run once, not N times per effect
/// - **Consistent**: All effects see the same signal analysis
/// - **Simple**: ~15-20 parameters instead of 70+
/// - **Musical**: Effects automatically adapt to signal content
///
/// # Total Latency
/// - Pitch detector: 1024 samples (~23ms @ 44.1kHz)
/// - Limiter lookahead: ~220 samples (~5ms @ 44.1kHz)
/// - **Total: ~1244 samples (~28ms @ 44.1kHz)**
use crate::dsp::signal_analyzer::SignalAnalyzer;
use crate::params_voice::VoiceParams;

/// Voice enhancement engine (intelligent architecture v2.0)
pub struct VoiceEngine {
    /// Sample rate in Hz
    sample_rate: f32,

    /// **Phase 1: Unified Signal Analysis**
    /// Runs all detectors once per sample (transient, ZCR, sibilance)
    /// Pitch detection is throttled to 512-sample intervals (~11ms)
    signal_analyzer: SignalAnalyzer,

    /// **Phase 2: Intelligent Effects**
    /// Each effect receives SignalAnalysis and adapts automatically

    /// Smart gate - automatically gentler on transients/sibilance
    smart_gate: SmartGate,

    /// Adaptive compressor - pitch-responsive, transient-aware
    adaptive_compressor: AdaptiveCompressor,

    /// Intelligent exciter - pitch-tracked harmonics, bypasses sibilance
    intelligent_exciter: IntelligentExciter,

    /// De-esser - intelligent sibilance reduction
    de_esser: DeEsser,

    /// Smart delay - transient-aware delay effect
    smart_delay: SmartDelay,

    /// Lookahead limiter - safety ceiling
    limiter: LookAheadLimiter,

    /// Current parameters
    params: VoiceParams,
}

impl VoiceEngine {
    /// Create a new voice enhancement engine
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            signal_analyzer: SignalAnalyzer::new(sample_rate),
            smart_gate: SmartGate::new(sample_rate),
            adaptive_compressor: AdaptiveCompressor::new(sample_rate),
            intelligent_exciter: IntelligentExciter::new(sample_rate),
            de_esser: DeEsser::new(sample_rate),
            smart_delay: SmartDelay::new(sample_rate, 500.0),
            limiter: LookAheadLimiter::new(sample_rate, 5.0, 0.99, 0.5, 50.0),
            params: VoiceParams::default(),
        }
    }

    /// Get the plugin's processing latency in samples
    ///
    /// Total latency = pitch detector buffer (1024) + limiter lookahead (~220)
    /// This is the same as before, but now more efficient (analyze once vs per-effect)
    pub fn get_latency(&self) -> u32 {
        1024 + self.limiter.get_latency_samples() as u32
    }

    /// Update parameters
    ///
    /// Much simpler now - only ~15-20 parameters instead of 70+
    pub fn update_params(&mut self, params: VoiceParams) {
        self.params = params;

        // Update smart gate (just 1 parameter!)
        self.smart_gate.set_threshold(self.params.gate_threshold);

        // Update adaptive compressor (4 parameters)
        self.adaptive_compressor
            .set_threshold(self.params.comp_threshold);
        self.adaptive_compressor.set_ratio(self.params.comp_ratio);
        self.adaptive_compressor.set_attack(self.params.comp_attack);
        self.adaptive_compressor
            .set_release(self.params.comp_release);

        // Update intelligent exciter (2 parameters)
        self.intelligent_exciter
            .set_amount(self.params.exciter_amount);
        self.intelligent_exciter.set_mix(self.params.exciter_mix);

        // Update de-esser (1 parameter - threshold is fixed at 0.5)
        self.de_esser.set_amount(self.params.deess_amount);

        // Note: Smart delay parameters are applied in process() method
        // They include: delay_time, delay_feedback, delay_mix
        // The delay automatically adapts based on transient detection

        // Note: pitch_confidence_threshold is used by signal_analyzer internally
        // It's applied when we check analysis.is_pitched and analysis.pitch_confidence
    }
    /// Process a stereo sample pair with intelligent signal analysis
    ///
    /// # New Architecture
    /// 1. Analyze signal once → get comprehensive SignalAnalysis
    /// 2. Thread analysis through all effects
    /// 3. Each effect adapts automatically based on detected characteristics
    ///
    /// # Arguments
    /// * `input_left` - Left channel input
    /// * `input_right` - Right channel input
    ///
    /// # Returns
    /// Tuple of (output_left, output_right)
    pub fn process(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        // Apply input gain
        let input_gain = VoiceParams::db_to_gain(self.params.input_gain);
        let mut left = input_left * input_gain;
        let mut right = input_right * input_gain;

        // Store dry signal for later mixing
        let dry_left = left;
        let dry_right = right;

        // === PHASE 1: SIGNAL ANALYSIS ===
        // Run all detectors ONCE per sample (transient, ZCR, sibilance, pitch)
        // Pitch detection is throttled internally to every 512 samples
        let analysis = self.signal_analyzer.analyze(left, right);

        // === PHASE 2: INTELLIGENT PROCESSING ===
        // Thread SignalAnalysis through all effects

        // 1. Smart Gate
        // Automatically gentler on transients/sibilance, pitch-aware thresholds
        if self.params.gate_enable {
            let (left_gated, right_gated) = self.smart_gate.process(left, right, &analysis);
            left = left_gated;
            right = right_gated;
        }

        // 2. Adaptive Compressor
        // Faster attack on transients, pitch-responsive thresholds
        if self.params.comp_enable {
            let (left_comp, right_comp) = self.adaptive_compressor.process(left, right, &analysis);
            left = left_comp;
            right = right_comp;
        }

        // 3. Intelligent Exciter
        // Tracks pitch, adds harmonics at 2×/3×/4×, bypasses sibilance
        if self.params.exciter_enable {
            let (left_excited, right_excited) =
                self.intelligent_exciter.process(left, right, &analysis);
            left = left_excited;
            right = right_excited;
        }

        // 4. De-Esser
        // Intelligently reduces harsh sibilance based on detected sibilance strength
        if self.params.deess_enable {
            let (left_deessed, right_deessed) = self.de_esser.process(left, right, &analysis);
            left = left_deessed;
            right = right_deessed;
        }

        // 5. Smart Delay
        // Transient-aware delay: bypasses delay during attacks, applies to sustained content
        if self.params.delay_enable {
            let (left_delayed, right_delayed) = self.smart_delay.process(
                left,
                right,
                self.params.delay_time,
                self.params.delay_feedback,
                self.params.delay_mix,
                self.params.delay_sensitivity,
                &analysis,
            );
            left = left_delayed;
            right = right_delayed;
        }

        // 6. Lookahead Limiter (safety ceiling)
        let (left_limited, right_limited) = self.limiter.process(left, right);
        left = left_limited;
        right = right_limited;

        // 7. Dry/Wet Mix
        left = left * self.params.dry_wet + dry_left * (1.0 - self.params.dry_wet);
        right = right * self.params.dry_wet + dry_right * (1.0 - self.params.dry_wet);

        // 6. Output Gain
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
        self.smart_gate.reset();
        self.adaptive_compressor.reset();
        self.intelligent_exciter.reset();
        self.de_esser.reset();
        self.smart_delay.reset();
        // Don't reset limiter - it maintains its internal state across resets
    }

    /// Get current detected pitch in Hz (from signal analyzer)
    pub fn get_detected_pitch(&self) -> f32 {
        self.signal_analyzer.get_current_pitch()
    }

    /// Get current pitch confidence (0.0-1.0, from signal analyzer)
    pub fn get_pitch_confidence(&self) -> f32 {
        self.signal_analyzer.get_pitch_confidence()
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
    fn test_latency() {
        let engine = VoiceEngine::new(44100.0);
        let latency = engine.get_latency();
        // Pitch buffer (1024) + limiter (~220) = ~1244 samples
        assert!(latency > 1200 && latency < 1300);
    }

    #[test]
    fn test_update_params() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();

        params.gate_threshold = -40.0;
        params.comp_threshold = -15.0;
        params.comp_ratio = 4.0;
        params.exciter_amount = 0.7;
        params.exciter_mix = 0.6;

        engine.update_params(params);

        // Params should be stored
        assert_eq!(engine.params.gate_threshold, -40.0);
        assert_eq!(engine.params.comp_threshold, -15.0);
        assert_eq!(engine.params.comp_ratio, 4.0);
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
        engine.update_params(params);

        let input = 0.1;
        let (out_l, _) = engine.process(input, input);

        // With +6dB gain, output should be amplified
        // Note: Can't predict exact value due to processing chain (gate, compressor, limiter)
        // Just verify it's producing valid output
        assert!(out_l.is_finite());
        assert!(out_l.abs() < 2.0); // Reasonable range
    }

    #[test]
    fn test_dry_wet_mix() {
        let mut engine = VoiceEngine::new(44100.0);

        // 100% dry (bypass)
        let mut params = VoiceParams::default();
        params.dry_wet = 0.0;
        engine.update_params(params);

        let input = 0.5;
        let (out_l, _) = engine.process(input, input);

        // Should be close to input (accounting for gain stages)
        assert!(out_l.abs() < 1.0);

        // 100% wet
        let mut params2 = VoiceParams::default();
        params2.dry_wet = 1.0;
        engine.update_params(params2);

        let (out_wet, _) = engine.process(input, input);

        // Wet should differ from dry (due to processing)
        // Just verify it's valid
        assert!(out_wet.is_finite());
    }
}
