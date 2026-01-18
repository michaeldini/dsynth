/// Voice Enhancement Engine
///
/// Real-time audio processing chain for vocal enhancement:
/// 1. Stereo input → mono sum for pitch detection
/// 2. Pitch detection (YIN algorithm, 1024 sample buffer)
/// 3. Noise gate (background noise removal)
/// 4. Parametric EQ (4-band vocal shaping)
/// 5. Compressor (dynamics control)
/// 6. De-Esser (sibilance reduction)
/// 7. Sub oscillator (pitch-tracked bass enhancement with amplitude ramping)
/// 8. Exciter (harmonic enhancement)
/// 9. Lookahead limiter (safety ceiling)
/// 10. Dry/wet mix and stereo output
use crate::dsp::effects::compressor::Compressor;
use crate::dsp::effects::de_esser::DeEsser;
use crate::dsp::effects::exciter::Exciter;
use crate::dsp::effects::noise_gate::NoiseGate;
use crate::dsp::effects::parametric_eq::ParametricEQ;
use crate::dsp::lookahead_limiter::LookAheadLimiter;
use crate::dsp::oscillator::Oscillator;
use crate::dsp::pitch_detector::{PitchDetector, PITCH_BUFFER_SIZE};
use crate::params::Waveform;
use crate::params_voice::{SubOscWaveform, VoiceParams};

/// Voice enhancement engine
pub struct VoiceEngine {
    /// Sample rate in Hz
    sample_rate: f32,

    /// Pitch detector (mono sum of stereo input)
    pitch_detector: PitchDetector,

    /// Current detected pitch in Hz
    detected_pitch: f32,

    /// Smoothed pitch for sub oscillator (prevents clicks)
    smoothed_pitch: f32,

    /// Confidence of pitch detection (0.0-1.0)
    pitch_confidence: f32,

    /// Pitch detection counter (run detection every N samples to save CPU)
    pitch_detection_counter: usize,

    /// Pitch detection interval (run every N samples)
    pitch_detection_interval: usize,

    /// Noise gate for background noise removal
    noise_gate: NoiseGate,

    /// 4-band parametric EQ for vocal shaping
    parametric_eq: ParametricEQ,

    /// Compressor for dynamics control
    compressor: Compressor,

    /// De-esser for sibilance reduction
    de_esser: DeEsser,

    /// Sub oscillator for bass enhancement
    sub_oscillator: Oscillator,

    /// Target pitch for sub oscillator (smoothed + octave shifted)
    sub_target_pitch: f32,

    /// Current amplitude for sub oscillator (for ramping)
    sub_amplitude: f32,

    /// Target amplitude for sub oscillator
    sub_target_amplitude: f32,

    /// Exciter for harmonic enhancement
    exciter: Exciter,

    /// Lookahead limiter (safety ceiling)
    limiter: LookAheadLimiter,

    /// Current parameters
    params: VoiceParams,
}

impl VoiceEngine {
    /// Create a new voice enhancement engine
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            pitch_detector: PitchDetector::new(sample_rate),
            detected_pitch: 100.0, // Default to ~100Hz
            smoothed_pitch: 100.0,
            pitch_confidence: 0.0,
            pitch_detection_counter: 0,
            pitch_detection_interval: 512, // Run pitch detection every 512 samples (~11ms @ 44.1kHz)
            noise_gate: NoiseGate::new(sample_rate),
            parametric_eq: ParametricEQ::new(sample_rate),
            compressor: Compressor::new(sample_rate, -20.0, 3.0, 10.0, 100.0),
            de_esser: DeEsser::new(sample_rate),
            sub_oscillator: {
                let mut osc = Oscillator::new(sample_rate);
                osc.set_waveform(Waveform::Sine);
                osc
            },
            sub_target_pitch: 50.0,
            sub_amplitude: 0.0,
            sub_target_amplitude: 0.0,
            exciter: Exciter::new(sample_rate),
            limiter: LookAheadLimiter::new(sample_rate, 5.0, 0.99, 0.5, 50.0), // 5ms lookahead, 0.99 threshold
            params: VoiceParams::default(),
        }
    }

    /// Get the plugin's processing latency in samples
    ///
    /// Total latency = pitch detector buffer (1024) + limiter lookahead (~220)
    pub fn get_latency(&self) -> u32 {
        PITCH_BUFFER_SIZE as u32 + self.limiter.get_latency_samples() as u32
    }

    /// Update parameters
    pub fn update_params(&mut self, params: VoiceParams) {
        // Store params
        let old_waveform = self.params.sub_waveform;
        self.params = params;

        // Update noise gate
        self.noise_gate.set_threshold(self.params.gate_threshold);
        self.noise_gate.set_ratio(self.params.gate_ratio);
        self.noise_gate.set_attack(self.params.gate_attack);
        self.noise_gate.set_release(self.params.gate_release);
        self.noise_gate.set_hold(self.params.gate_hold);

        // Update parametric EQ
        self.parametric_eq.set_band(
            0,
            self.params.eq_band1_freq,
            self.params.eq_band1_gain,
            self.params.eq_band1_q,
        );
        self.parametric_eq.set_band(
            1,
            self.params.eq_band2_freq,
            self.params.eq_band2_gain,
            self.params.eq_band2_q,
        );
        self.parametric_eq.set_band(
            2,
            self.params.eq_band3_freq,
            self.params.eq_band3_gain,
            self.params.eq_band3_q,
        );
        self.parametric_eq.set_band(
            3,
            self.params.eq_band4_freq,
            self.params.eq_band4_gain,
            self.params.eq_band4_q,
        );

        // Update compressor
        self.compressor.set_threshold(self.params.comp_threshold);
        self.compressor.set_ratio(self.params.comp_ratio);
        self.compressor.set_attack(self.params.comp_attack);
        self.compressor.set_release(self.params.comp_release);
        self.compressor.set_knee(self.params.comp_knee);
        self.compressor
            .set_makeup_gain(self.params.comp_makeup_gain);

        // Update de-esser
        self.de_esser.set_threshold(self.params.deess_threshold);
        self.de_esser.set_frequency(self.params.deess_frequency);
        self.de_esser.set_ratio(self.params.deess_ratio);

        // Update pitch detector threshold
        self.pitch_detector
            .set_threshold(self.params.pitch_confidence_threshold);

        // Update sub oscillator waveform if changed
        if self.params.sub_waveform != old_waveform {
            let waveform = match self.params.sub_waveform {
                SubOscWaveform::Sine => Waveform::Sine,
                SubOscWaveform::Triangle => Waveform::Triangle,
                SubOscWaveform::Square => Waveform::Square,
                SubOscWaveform::Saw => Waveform::Saw,
            };
            self.sub_oscillator.set_waveform(waveform);
        }

        // Update exciter
        self.exciter.set_drive(self.params.exciter_amount);
        self.exciter.set_frequency(self.params.exciter_frequency);
        // Note: exciter_harmonics parameter is not used (Exciter doesn't have this control)
        self.exciter.set_mix(self.params.exciter_mix);
    }

    /// Process a stereo sample pair
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

        // 1. Noise Gate (FIRST - remove noise before pitch detection)
        let (left_gated, right_gated) = self.noise_gate.process(left, right);
        left = left_gated;
        right = right_gated;

        // 2. Pitch Detection (mono sum) - now runs on GATED signal
        // This prevents the sub oscillator from tracking noise/silence
        let mono = (left + right) * 0.5;
        self.pitch_detector.process_sample(mono);

        // Detect silence to clear stale pitch confidence
        let is_silence = mono.abs() < 0.0001; // -80dB threshold
        if is_silence {
            self.pitch_confidence = 0.0; // Clear confidence immediately on silence
        }

        // Only run detection every N samples (default 512 = ~11ms @ 44.1kHz)
        self.pitch_detection_counter += 1;
        if self.pitch_detection_counter >= self.pitch_detection_interval {
            self.pitch_detection_counter = 0;

            let pitch_result = self.pitch_detector.detect();

            // Update detected pitch with confidence check
            if pitch_result.confidence >= self.params.pitch_confidence_threshold {
                self.detected_pitch = pitch_result.frequency_hz;
                self.pitch_confidence = pitch_result.confidence;

                // Smooth pitch changes using exponential moving average
                let alpha = 1.0 - self.params.pitch_smoothing;
                self.smoothed_pitch =
                    alpha * self.detected_pitch + (1.0 - alpha) * self.smoothed_pitch;
            } else {
                // Below threshold - clear confidence to stop sub oscillator
                self.pitch_confidence = 0.0;
            }
        }

        // Calculate sub oscillator target pitch (octave shift)
        let octave_multiplier = 2.0_f32.powf(self.params.sub_octave);
        self.sub_target_pitch = self.smoothed_pitch * octave_multiplier;

        // Update sub oscillator frequency (continuous phase)
        self.sub_oscillator.set_frequency(self.sub_target_pitch);

        // Calculate sub oscillator target amplitude based on pitch confidence
        self.sub_target_amplitude =
            if self.pitch_confidence >= self.params.pitch_confidence_threshold {
                self.params.sub_level
            } else {
                0.0 // Fade out sub when no confident pitch detected
            };

        // 3. Parametric EQ
        let (left_eq, right_eq) = self.parametric_eq.process(left, right);
        left = left_eq;
        right = right_eq;

        // Apply EQ master gain
        let eq_master_gain = VoiceParams::db_to_gain(self.params.eq_master_gain);
        left *= eq_master_gain;
        right *= eq_master_gain;

        // 4. Compressor
        let (left_comp, right_comp) = self.compressor.process(left, right);
        left = left_comp;
        right = right_comp;

        // 5. De-Esser
        let (left_deess, right_deess) = self.de_esser.process(left, right);
        left = left_deess * self.params.deess_amount + left * (1.0 - self.params.deess_amount);
        right = right_deess * self.params.deess_amount + right * (1.0 - self.params.deess_amount);

        // 6. Sub Oscillator (with amplitude ramping to avoid clicks)
        let ramp_samples = (self.params.sub_ramp_time * 0.001 * self.sample_rate).max(1.0);
        let ramp_step = (self.sub_target_amplitude - self.sub_amplitude) / ramp_samples;
        self.sub_amplitude += ramp_step;
        self.sub_amplitude = self.sub_amplitude.clamp(0.0, 1.0);

        let sub_sample = self.sub_oscillator.process() * self.sub_amplitude;
        left += sub_sample;
        right += sub_sample;

        // 7. Exciter (harmonic enhancement)
        let (left_excited, right_excited) = self.exciter.process(left, right);
        left = left_excited;
        right = right_excited;

        // 8. Lookahead Limiter (safety ceiling at 0dB)
        let (left_limited, right_limited) = self.limiter.process(left, right);
        left = left_limited;
        right = right_limited;

        // 9. Dry/Wet Mix
        left = left * self.params.dry_wet + dry_left * (1.0 - self.params.dry_wet);
        right = right * self.params.dry_wet + dry_right * (1.0 - self.params.dry_wet);

        // 10. Output Gain
        let output_gain = VoiceParams::db_to_gain(self.params.output_gain);
        left *= output_gain;
        right *= output_gain;

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
        self.detected_pitch = 100.0;
        self.smoothed_pitch = 100.0;
        self.pitch_confidence = 0.0;
        self.pitch_detection_counter = 0;
        self.sub_amplitude = 0.0;
        self.sub_target_amplitude = 0.0;

        self.pitch_detector = PitchDetector::new(self.sample_rate);
        self.noise_gate.reset();
        self.parametric_eq.reset();
        self.compressor.reset();
        self.de_esser.reset();
        self.sub_oscillator = {
            let mut osc = Oscillator::new(self.sample_rate);
            osc.set_waveform(Waveform::Sine);
            osc
        };
        self.exciter.reset();
        // Don't reset limiter - it maintains its internal state across resets
    }

    /// Get current detected pitch in Hz
    pub fn get_detected_pitch(&self) -> f32 {
        self.detected_pitch
    }

    /// Get current pitch confidence (0.0-1.0)
    pub fn get_pitch_confidence(&self) -> f32 {
        self.pitch_confidence
    }

    /// Get current sub oscillator amplitude (for visualization)
    pub fn get_sub_amplitude(&self) -> f32 {
        self.sub_amplitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_engine_creation() {
        let engine = VoiceEngine::new(44100.0);
        assert_eq!(engine.sample_rate, 44100.0);
        assert_eq!(engine.detected_pitch, 100.0);
        assert_eq!(engine.pitch_confidence, 0.0);
    }

    #[test]
    fn test_voice_engine_latency() {
        let engine = VoiceEngine::new(44100.0);
        let latency = engine.get_latency();
        // Pitch buffer (1024) + limiter lookahead (~220)
        assert!(latency > 1024);
        assert!(latency < 1300);
    }

    #[test]
    fn test_process_silence() {
        let mut engine = VoiceEngine::new(44100.0);

        // Process silent samples
        for _ in 0..2048 {
            let (out_l, out_r) = engine.process(0.0, 0.0);
            assert!(
                out_l.abs() < 0.001,
                "Left output should be near zero for silence"
            );
            assert!(
                out_r.abs() < 0.001,
                "Right output should be near zero for silence"
            );
        }
    }

    #[test]
    fn test_process_audio_signal() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();
        params.sub_level = 0.0; // Disable sub for this test
        params.gate_threshold = -80.0; // Very low threshold
        engine.update_params(params);

        // Generate a 220Hz sine wave (A3)
        let freq = 220.0;
        let mut max_output = 0.0_f32;

        for i in 0..44100 {
            let t = i as f32 / 44100.0;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5;
            let (out_l, out_r) = engine.process(input, input);
            max_output = max_output.max(out_l.abs()).max(out_r.abs());
        }

        // Should produce audible output
        assert!(max_output > 0.1, "Should produce audible output");
        assert!(max_output <= 1.0, "Should not clip");
    }

    #[test]
    fn test_pitch_detection_integration() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();
        params.pitch_confidence_threshold = 0.5;
        engine.update_params(params);

        // Feed a stable 110Hz signal (A2)
        let freq = 110.0;
        for i in 0..(PITCH_BUFFER_SIZE * 4) {
            let t = i as f32 / 44100.0;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5;
            engine.process(input, input);
        }

        // After processing enough samples, should detect pitch
        let detected = engine.get_detected_pitch();
        let confidence = engine.get_pitch_confidence();

        // Should detect something in the ballpark (YIN may not be exact)
        assert!(
            detected > 80.0 && detected < 150.0,
            "Detected pitch should be near 110Hz"
        );
        assert!(confidence > 0.0, "Should have some confidence");
    }

    #[test]
    fn test_sub_oscillator_amplitude_ramping() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();
        params.sub_level = 0.5;
        params.sub_ramp_time = 10.0; // 10ms ramp
        params.pitch_confidence_threshold = 0.0; // Always enable sub
        engine.update_params(params);

        // Initial amplitude should be zero
        assert_eq!(engine.get_sub_amplitude(), 0.0);

        // Process samples and watch amplitude ramp up
        let mut last_amp = 0.0;
        for _ in 0..1000 {
            engine.process(0.1, 0.1);
            let current_amp = engine.get_sub_amplitude();
            assert!(current_amp >= last_amp, "Amplitude should ramp up smoothly");
            last_amp = current_amp;
        }

        // After enough samples, should reach target
        assert!(
            engine.get_sub_amplitude() > 0.4,
            "Should ramp to near target level"
        );
    }

    #[test]
    fn test_dry_wet_mixing() {
        let mut engine = VoiceEngine::new(44100.0);

        // Test 100% dry (bypass)
        let mut params = VoiceParams::default();
        params.dry_wet = 0.0;
        engine.update_params(params);

        let input = 0.5;
        let (out_l, out_r) = engine.process(input, input);
        assert!(
            (out_l - input).abs() < 0.01,
            "Dry signal should pass through unaffected"
        );
        assert!(
            (out_r - input).abs() < 0.01,
            "Dry signal should pass through unaffected"
        );

        // Test 100% wet
        let mut params = VoiceParams::default();
        params.dry_wet = 1.0;
        engine.update_params(params);
        engine.reset();

        // Process silence - wet signal will be processed (gated, compressed, etc.)
        let (out_l, out_r) = engine.process(0.0, 0.0);
        // Output should be near zero (processed silence)
        assert!(out_l.abs() < 0.01);
        assert!(out_r.abs() < 0.01);
    }

    #[test]
    fn test_parameter_update() {
        let mut engine = VoiceEngine::new(44100.0);

        let mut params = VoiceParams::default();
        params.gate_threshold = -40.0;
        params.comp_ratio = 8.0;
        params.sub_level = 0.7;

        engine.update_params(params.clone());

        // Verify parameters were applied (we can't directly check internal state,
        // but we can verify the engine doesn't crash and accepts the params)
        assert_eq!(engine.params.gate_threshold, -40.0);
        assert_eq!(engine.params.comp_ratio, 8.0);
        assert_eq!(engine.params.sub_level, 0.7);
    }

    #[test]
    fn test_buffer_processing() {
        let mut engine = VoiceEngine::new(44100.0);

        let frame_count = 512;
        let mut input_left = vec![0.0; frame_count];
        let mut input_right = vec![0.0; frame_count];
        let mut output_left = vec![0.0; frame_count];
        let mut output_right = vec![0.0; frame_count];

        // Generate test signal
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
        for _ in 0..1000 {
            engine.process(0.5, 0.5);
        }

        // Reset
        engine.reset();

        // Verify state is reset
        assert_eq!(engine.detected_pitch, 100.0);
        assert_eq!(engine.smoothed_pitch, 100.0);
        assert_eq!(engine.pitch_confidence, 0.0);
        assert_eq!(engine.sub_amplitude, 0.0);
    }

    #[test]
    fn test_waveform_change() {
        let mut engine = VoiceEngine::new(44100.0);

        let mut params = VoiceParams::default();

        // Change waveform to each type
        for waveform in [
            SubOscWaveform::Sine,
            SubOscWaveform::Triangle,
            SubOscWaveform::Square,
            SubOscWaveform::Saw,
        ] {
            params.sub_waveform = waveform;
            engine.update_params(params.clone());

            // Process a sample to ensure it doesn't crash
            let (out_l, out_r) = engine.process(0.1, 0.1);
            assert!(out_l.is_finite());
            assert!(out_r.is_finite());
        }
    }
}
