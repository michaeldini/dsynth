/// Voice Enhancement Engine
///
/// Real-time audio processing chain for vocal enhancement:
/// 1. Stereo input → mono sum for pitch detection
/// 2. Pitch detection (YIN algorithm, 1024 sample buffer, adaptive smoothing)
/// 3. Noise gate (background noise removal)
/// 4. Parametric EQ (4-band vocal shaping)
/// 5. Compressor (dynamics control)
/// 6. De-Esser (sibilance reduction)
/// 7. Sub oscillator (pitch-tracked bass enhancement with amplitude ramping)
/// 8. Ring modulator (pitch-tracked harmonically-related robotic effects)
/// 9. Pitch-controlled filter sweep (talking synthesizer effect)
/// 10. Exciter (harmonic enhancement)
/// 11. Vocal doubler (stereo double-tracking effect)
/// 12. Vocal choir (multi-voice ensemble effect)
/// 13. Multiband distortion (frequency-dependent saturation)
/// 14. Lookahead limiter (safety ceiling)
/// 15. Dry/wet mix and stereo output
use crate::dsp::effects::compressor::Compressor;
use crate::dsp::effects::de_esser::DeEsser;
use crate::dsp::effects::exciter::Exciter;
use crate::dsp::effects::noise_gate::NoiseGate;
use crate::dsp::effects::parametric_eq::ParametricEQ;
use crate::dsp::effects::ring_modulator::RingModulator;
use crate::dsp::effects::vocal_choir::VocalChoir;
use crate::dsp::effects::MultibandDistortion;
use crate::dsp::filter::BiquadFilter;
use crate::dsp::lookahead_limiter::LookAheadLimiter;
use crate::dsp::oscillator::Oscillator;
use crate::dsp::pitch_detector::{PitchDetector, PITCH_BUFFER_SIZE};
use crate::dsp::pitch_quantizer::{PitchQuantizer, RootNote, ScaleType};
use crate::params::{FilterType, Waveform};
use crate::params_voice::{RingModWaveform, SubOscWaveform, VoiceParams};

/// Voice enhancement engine
pub struct VoiceEngine {
    /// Sample rate in Hz
    sample_rate: f32,

    /// Pitch detector (mono sum of stereo input)
    pitch_detector: PitchDetector,

    /// Pitch quantizer for auto-tune/pitch correction
    pitch_quantizer: PitchQuantizer,

    /// Current detected pitch in Hz (raw from YIN)
    detected_pitch: f32,

    /// Quantized/corrected pitch in Hz (after scale snapping)
    corrected_pitch: f32,

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

    /// Harmonizer oscillator 2 (major 3rd by default)
    harm2_oscillator: Oscillator,
    harm2_target_pitch: f32,
    harm2_amplitude: f32,
    harm2_target_amplitude: f32,

    /// Harmonizer oscillator 3 (perfect 5th by default)
    harm3_oscillator: Oscillator,
    harm3_target_pitch: f32,
    harm3_amplitude: f32,
    harm3_target_amplitude: f32,

    /// Harmonizer oscillator 4 (octave up by default)
    harm4_oscillator: Oscillator,
    harm4_target_pitch: f32,
    harm4_amplitude: f32,
    harm4_target_amplitude: f32,

    /// Ring modulator for harmonically-related robotic effects
    ring_modulator: RingModulator,

    /// Pitch-controlled filter sweep (talking synthesizer effect)
    filter_follow: BiquadFilter,

    /// Exciter for harmonic enhancement
    exciter: Exciter,

    /// Vocal doubler for double-tracking effect
    doubler: crate::dsp::effects::VocalDoubler,

    /// Vocal choir for multi-voice ensemble effect
    choir: VocalChoir,

    /// Multiband distortion for frequency-dependent saturation
    multiband_distortion: MultibandDistortion,

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
            pitch_quantizer: PitchQuantizer::new(sample_rate),
            detected_pitch: 100.0, // Default to ~100Hz
            corrected_pitch: 100.0, // Default to ~100Hz
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
            harm2_oscillator: {
                let mut osc = Oscillator::new(sample_rate);
                osc.set_waveform(Waveform::Sine);
                osc
            },
            harm2_target_pitch: 100.0,
            harm2_amplitude: 0.0,
            harm2_target_amplitude: 0.0,
            harm3_oscillator: {
                let mut osc = Oscillator::new(sample_rate);
                osc.set_waveform(Waveform::Sine);
                osc
            },
            harm3_target_pitch: 100.0,
            harm3_amplitude: 0.0,
            harm3_target_amplitude: 0.0,
            harm4_oscillator: {
                let mut osc = Oscillator::new(sample_rate);
                osc.set_waveform(Waveform::Sine);
                osc
            },
            harm4_target_pitch: 100.0,
            harm4_amplitude: 0.0,
            harm4_target_amplitude: 0.0,
            ring_modulator: RingModulator::new(sample_rate, 200.0), // Default 200Hz carrier
            filter_follow: {
                let mut filter = BiquadFilter::new(sample_rate);
                filter.set_filter_type(FilterType::Lowpass);
                filter.set_cutoff(8000.0);
                filter.set_resonance(1.0);
                filter
            },
            exciter: Exciter::new(sample_rate),
            doubler: crate::dsp::effects::VocalDoubler::new(sample_rate),
            choir: VocalChoir::new(sample_rate),
            multiband_distortion: MultibandDistortion::new(sample_rate),
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
        let old_sub_waveform = self.params.sub_waveform;
        let old_harm2_waveform = self.params.harm2_waveform;
        let old_harm3_waveform = self.params.harm3_waveform;
        let old_harm4_waveform = self.params.harm4_waveform;
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

        // Update pitch quantizer (auto-tune) settings
        let scale_type = match self.params.pitch_correction_scale {
            1 => ScaleType::Major,
            2 => ScaleType::Minor,
            3 => ScaleType::Pentatonic,
            4 => ScaleType::MinorPentatonic,
            _ => ScaleType::Chromatic, // 0 or invalid = chromatic
        };
        self.pitch_quantizer.set_scale_type(scale_type);
        self.pitch_quantizer
            .set_root_note(RootNote(self.params.pitch_correction_root.clamp(0, 11)));
        self.pitch_quantizer
            .set_retune_speed(self.params.pitch_correction_speed);
        self.pitch_quantizer
            .set_correction_amount(self.params.pitch_correction_amount);

        // Update sub oscillator waveform if changed
        if self.params.sub_waveform != old_sub_waveform {
            let waveform = match self.params.sub_waveform {
                SubOscWaveform::Sine => Waveform::Sine,
                SubOscWaveform::Triangle => Waveform::Triangle,
                SubOscWaveform::Square => Waveform::Square,
                SubOscWaveform::Saw => Waveform::Saw,
            };
            self.sub_oscillator.set_waveform(waveform);
        }

        // Update harmonizer 2 waveform if changed
        if self.params.harm2_waveform != old_harm2_waveform {
            let waveform = match self.params.harm2_waveform {
                SubOscWaveform::Sine => Waveform::Sine,
                SubOscWaveform::Triangle => Waveform::Triangle,
                SubOscWaveform::Square => Waveform::Square,
                SubOscWaveform::Saw => Waveform::Saw,
            };
            self.harm2_oscillator.set_waveform(waveform);
        }

        // Update harmonizer 3 waveform if changed
        if self.params.harm3_waveform != old_harm3_waveform {
            let waveform = match self.params.harm3_waveform {
                SubOscWaveform::Sine => Waveform::Sine,
                SubOscWaveform::Triangle => Waveform::Triangle,
                SubOscWaveform::Square => Waveform::Square,
                SubOscWaveform::Saw => Waveform::Saw,
            };
            self.harm3_oscillator.set_waveform(waveform);
        }

        // Update harmonizer 4 waveform if changed
        if self.params.harm4_waveform != old_harm4_waveform {
            let waveform = match self.params.harm4_waveform {
                SubOscWaveform::Sine => Waveform::Sine,
                SubOscWaveform::Triangle => Waveform::Triangle,
                SubOscWaveform::Square => Waveform::Square,
                SubOscWaveform::Saw => Waveform::Saw,
            };
            self.harm4_oscillator.set_waveform(waveform);
        }

        // Update ring modulator
        let ring_waveform = match self.params.ring_mod_waveform {
            RingModWaveform::Sine => crate::dsp::effects::ring_modulator::Waveform::Sine,
            RingModWaveform::Triangle => crate::dsp::effects::ring_modulator::Waveform::Triangle,
            RingModWaveform::Square => crate::dsp::effects::ring_modulator::Waveform::Square,
            RingModWaveform::Saw => crate::dsp::effects::ring_modulator::Waveform::Saw,
        };
        self.ring_modulator.set_waveform(ring_waveform);
        self.ring_modulator.set_depth(self.params.ring_mod_depth);
        self.ring_modulator.set_mix(self.params.ring_mod_mix);

        // Update exciter
        self.exciter.set_drive(self.params.exciter_amount);
        self.exciter.set_frequency(self.params.exciter_frequency);
        // Note: exciter_harmonics parameter is not used (Exciter doesn't have this control)
        self.exciter.set_mix(self.params.exciter_mix);

        // Update vocal doubler
        self.doubler.set_delay_time(self.params.doubler_delay);
        self.doubler.set_detune(self.params.doubler_detune);
        self.doubler
            .set_stereo_width(self.params.doubler_stereo_width);
        self.doubler.set_mix(self.params.doubler_mix);

        // Update vocal choir
        self.choir.set_num_voices(self.params.choir_num_voices);
        self.choir.set_detune_amount(self.params.choir_detune);
        self.choir
            .set_delay_spread(self.params.choir_delay_spread);
        self.choir
            .set_stereo_spread(self.params.choir_stereo_spread);
        self.choir.set_mix(self.params.choir_mix);

        // Update multiband distortion
        self.multiband_distortion
            .set_low_mid_freq(self.params.mb_dist_low_mid_freq);
        self.multiband_distortion
            .set_mid_high_freq(self.params.mb_dist_mid_high_freq);
        self.multiband_distortion
            .set_drive_low(self.params.mb_dist_drive_low);
        self.multiband_distortion
            .set_drive_mid(self.params.mb_dist_drive_mid);
        self.multiband_distortion
            .set_drive_high(self.params.mb_dist_drive_high);
        self.multiband_distortion
            .set_gain_low(self.params.mb_dist_gain_low);
        self.multiband_distortion
            .set_gain_mid(self.params.mb_dist_gain_mid);
        self.multiband_distortion
            .set_gain_high(self.params.mb_dist_gain_high);
        self.multiband_distortion
            .set_mix(self.params.mb_dist_mix);
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
        // Debug: Check input
        use std::io::Write;
        if !input_left.is_finite() || !input_right.is_finite() {
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN DETECTED] Input: left={}, right={}",
                input_left,
                input_right
            );
        }

        // Apply input gain
        let input_gain = VoiceParams::db_to_gain(self.params.input_gain);
        let mut left = input_left * input_gain;
        let mut right = input_right * input_gain;

        // Debug: Check after input gain
        if !left.is_finite() || !right.is_finite() {
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN DETECTED] After input gain ({}): left={}, right={}",
                input_gain,
                left,
                right
            );
        }

        // Store dry signal for later mixing
        let dry_left = left;
        let dry_right = right;

        // 1. Noise Gate (FIRST - remove noise before pitch detection)
        let (left_gated, right_gated) = self.noise_gate.process(left, right);
        left = left_gated;
        right = right_gated;

        // Debug: Check after noise gate
        if !left.is_finite() || !right.is_finite() {
            use std::io::Write;
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN] After noise gate: left={}, right={}",
                left,
                right
            );
        }

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

                // Apply pitch correction / auto-tune (quantize to scale)
                // This corrects the pitch before feeding to harmonizers
                self.corrected_pitch = if self.params.pitch_correction_enable {
                    self.pitch_quantizer.quantize(self.detected_pitch)
                } else {
                    self.detected_pitch // No correction - pass through raw pitch
                };

                // Adaptive pitch smoothing: fast attack on big jumps, slow smoothing on small wobbles
                // This prevents beating between harmonizers while maintaining responsive tracking
                // NOTE: Smoothing happens AFTER quantization for cleaner auto-tune effect
                let pitch_delta = (self.corrected_pitch - self.smoothed_pitch).abs();
                let adaptive_alpha = if pitch_delta > 20.0 {
                    0.3 // Fast attack on big jumps (>20Hz)
                } else {
                    0.9 // Slow smoothing on small wobbles (<20Hz)
                };
                
                self.smoothed_pitch =
                    adaptive_alpha * self.corrected_pitch + (1.0 - adaptive_alpha) * self.smoothed_pitch;
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

        // Calculate sub oscillator target amplitude based on pitch confidence and enable
        self.sub_target_amplitude = if self.params.sub_enable
            && self.pitch_confidence >= self.params.pitch_confidence_threshold
        {
            self.params.sub_level
        } else {
            0.0 // Fade out sub when disabled or no confident pitch detected
        };

        // Calculate harmonizer 2 target pitch (semitone shift: +4 = major 3rd)
        let harm2_multiplier = 2.0_f32.powf(self.params.harm2_semitones / 12.0);
        self.harm2_target_pitch = self.smoothed_pitch * harm2_multiplier;
        self.harm2_oscillator.set_frequency(self.harm2_target_pitch);
        self.harm2_target_amplitude = if self.params.harm2_enable
            && self.pitch_confidence >= self.params.pitch_confidence_threshold
        {
            self.params.harm2_level
        } else {
            0.0
        };

        // Calculate harmonizer 3 target pitch (semitone shift: +7 = perfect 5th)
        let harm3_multiplier = 2.0_f32.powf(self.params.harm3_semitones / 12.0);
        self.harm3_target_pitch = self.smoothed_pitch * harm3_multiplier;
        self.harm3_oscillator.set_frequency(self.harm3_target_pitch);
        self.harm3_target_amplitude = if self.params.harm3_enable
            && self.pitch_confidence >= self.params.pitch_confidence_threshold
        {
            self.params.harm3_level
        } else {
            0.0
        };

        // Calculate harmonizer 4 target pitch (semitone shift: +12 = octave up)
        let harm4_multiplier = 2.0_f32.powf(self.params.harm4_semitones / 12.0);
        self.harm4_target_pitch = self.smoothed_pitch * harm4_multiplier;
        self.harm4_oscillator.set_frequency(self.harm4_target_pitch);
        self.harm4_target_amplitude = if self.params.harm4_enable
            && self.pitch_confidence >= self.params.pitch_confidence_threshold
        {
            self.params.harm4_level
        } else {
            0.0
        };

        // Update ring modulator carrier frequency (harmonic ratio × detected pitch)
        if self.params.ring_mod_enable {
            let carrier_freq = self.smoothed_pitch * self.params.ring_mod_harmonic;
            self.ring_modulator.set_frequency(carrier_freq);
        }

        // 3. Parametric EQ
        let (left_eq, right_eq) = self.parametric_eq.process(left, right);
        left = left_eq;
        right = right_eq;

        // Debug: Check after EQ
        if !left.is_finite() || !right.is_finite() {
            use std::io::Write;
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN] After EQ: left={}, right={}",
                left,
                right
            );
        }

        // Apply EQ master gain
        let eq_master_gain = VoiceParams::db_to_gain(self.params.eq_master_gain);
        left *= eq_master_gain;
        right *= eq_master_gain;

        // Debug: Check after EQ master gain
        if !left.is_finite() || !right.is_finite() {
            use std::io::Write;
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN] After EQ master gain ({}): left={}, right={}",
                eq_master_gain,
                left,
                right
            );
        }

        // 4. Compressor
        let (left_comp, right_comp) = self.compressor.process(left, right);
        left = left_comp;
        right = right_comp;

        // Debug: Check after compressor
        if !left.is_finite() || !right.is_finite() {
            use std::io::Write;
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN] After compressor: left={}, right={}",
                left,
                right
            );
        }

        // 5. De-Esser
        let (left_deess, right_deess) = self.de_esser.process(left, right);
        left = left_deess * self.params.deess_amount + left * (1.0 - self.params.deess_amount);
        right = right_deess * self.params.deess_amount + right * (1.0 - self.params.deess_amount);

        // 6. Sub Oscillator (with amplitude ramping to avoid clicks)
        if self.params.sub_enable {
            let ramp_samples = (self.params.sub_ramp_time * 0.001 * self.sample_rate).max(1.0);
            let ramp_step = (self.sub_target_amplitude - self.sub_amplitude) / ramp_samples;
            self.sub_amplitude += ramp_step;
            self.sub_amplitude = self.sub_amplitude.clamp(0.0, 1.0);

            let sub_sample = self.sub_oscillator.process() * self.sub_amplitude;
            left += sub_sample;
            right += sub_sample;
        }

        // 6b. Harmonizer Oscillator 2 (major 3rd)
        if self.params.harm2_enable {
            let ramp_samples = (self.params.harm2_ramp_time * 0.001 * self.sample_rate).max(1.0);
            let ramp_step = (self.harm2_target_amplitude - self.harm2_amplitude) / ramp_samples;
            self.harm2_amplitude += ramp_step;
            self.harm2_amplitude = self.harm2_amplitude.clamp(0.0, 1.0);

            let harm2_sample = self.harm2_oscillator.process() * self.harm2_amplitude;
            left += harm2_sample;
            right += harm2_sample;
        }

        // 6c. Harmonizer Oscillator 3 (perfect 5th)
        if self.params.harm3_enable {
            let ramp_samples = (self.params.harm3_ramp_time * 0.001 * self.sample_rate).max(1.0);
            let ramp_step = (self.harm3_target_amplitude - self.harm3_amplitude) / ramp_samples;
            self.harm3_amplitude += ramp_step;
            self.harm3_amplitude = self.harm3_amplitude.clamp(0.0, 1.0);

            let harm3_sample = self.harm3_oscillator.process() * self.harm3_amplitude;
            left += harm3_sample;
            right += harm3_sample;
        }

        // 6d. Harmonizer Oscillator 4 (octave up)
        if self.params.harm4_enable {
            let ramp_samples = (self.params.harm4_ramp_time * 0.001 * self.sample_rate).max(1.0);
            let ramp_step = (self.harm4_target_amplitude - self.harm4_amplitude) / ramp_samples;
            self.harm4_amplitude += ramp_step;
            self.harm4_amplitude = self.harm4_amplitude.clamp(0.0, 1.0);

            let harm4_sample = self.harm4_oscillator.process() * self.harm4_amplitude;
            left += harm4_sample;
            right += harm4_sample;
        }

        // Debug: Check before exciter
        if !left.is_finite() || !right.is_finite() {
            use std::io::Write;
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN DETECTED] BEFORE exciter (after harmonizer oscs): left={}, right={}",
                left,
                right
            );
        }

        // 7. Ring Modulator (pitch-tracked harmonically-related modulation)
        if self.params.ring_mod_enable {
            let (left_ring, right_ring) = self.ring_modulator.process(left, right);
            left = left_ring;
            right = right_ring;
        }

        // 7b. Pitch-Controlled Filter Sweep (talking synthesizer effect)
        if self.params.filter_follow_enable {
            // Store dry signal for mixing
            let dry_left = left;
            let dry_right = right;
            
            // Map detected pitch to filter cutoff
            // Use a narrower pitch range that better matches typical singing/speaking
            // Most vocals stay in 100-400Hz range, not the full 80-800Hz potential
            const MIN_VOCAL_PITCH: f32 = 100.0;  // Male low notes
            const MAX_VOCAL_PITCH: f32 = 500.0;  // Female/falsetto high notes
            
            let pitch_normalized = ((self.smoothed_pitch - MIN_VOCAL_PITCH) 
                / (MAX_VOCAL_PITCH - MIN_VOCAL_PITCH)).clamp(0.0, 1.0);
            
            // Apply tracking amount (sensitivity): >1.0 exaggerates the response
            // For amount > 1.5, use squared exaggeration for even more drama
            let exaggerated_pitch = if self.params.filter_follow_amount > 1.5 {
                // Very exaggerated: combine power curve with squaring
                let pow_curve = pitch_normalized.powf(1.0 / self.params.filter_follow_amount.max(0.1));
                pow_curve * pow_curve  // Square it for extreme response
            } else {
                // Normal exaggeration
                pitch_normalized.powf(1.0 / self.params.filter_follow_amount.max(0.1))
            };
            
            let cutoff_hz = self.params.filter_follow_min_freq 
                + exaggerated_pitch * (self.params.filter_follow_max_freq - self.params.filter_follow_min_freq);
            
            self.filter_follow.set_cutoff(cutoff_hz);
            self.filter_follow.set_resonance(self.params.filter_follow_resonance);
            
            let wet_left = self.filter_follow.process(left);
            let wet_right = self.filter_follow.process(right);
            
            // Mix dry and wet
            left = dry_left * (1.0 - self.params.filter_follow_mix) + wet_left * self.params.filter_follow_mix;
            right = dry_right * (1.0 - self.params.filter_follow_mix) + wet_right * self.params.filter_follow_mix;
        }

        // 8. Exciter (harmonic enhancement)
        // Update frequency based on pitch tracking if enabled
        if self.params.exciter_follow_enable {
            if self.pitch_confidence >= self.params.pitch_confidence_threshold {
                // Calculate target frequency as multiple of detected pitch
                let target_freq = self.smoothed_pitch * self.params.exciter_follow_amount;
                // Clamp to valid exciter range (2-12kHz)
                let clamped_freq = target_freq.clamp(2000.0, 12000.0);
                self.exciter.set_frequency(clamped_freq);
            }
        } else {
            // Use static frequency parameter when tracking disabled
            self.exciter.set_frequency(self.params.exciter_frequency);
        }
        let (left_excited, right_excited) = self.exciter.process(left, right);
        left = left_excited;
        right = right_excited;

        // Debug: Check after exciter
        if !left.is_finite() || !right.is_finite() {
            use std::io::Write;
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN DETECTED] After exciter: left={}, right={}",
                left,
                right
            );
        }

        // 9. Vocal Doubler (stereo double-tracking effect)
        if self.params.doubler_enable {
            let (left_doubled, right_doubled) = self.doubler.process(left, right);
            left = left_doubled;
            right = right_doubled;
        }

        // 10. Vocal Choir (multi-voice ensemble effect)
        if self.params.choir_enable {
            let (left_choir, right_choir) = self.choir.process(left, right);
            left = left_choir;
            right = right_choir;
        }

        // 11. Multiband Distortion (frequency-dependent saturation)
        if self.params.mb_dist_enable {
            let (left_dist, right_dist) = self.multiband_distortion.process_stereo(left, right);
            left = left_dist;
            right = right_dist;
        }

        // 12. Lookahead Limiter (safety ceiling at 0dB)
        let (left_limited, right_limited) = self.limiter.process(left, right);
        left = left_limited;
        right = right_limited;

        // Debug: Check after limiter
        if !left.is_finite() || !right.is_finite() {
            use std::io::Write;
            let _ = writeln!(
                &mut std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/dsynth_voice_debug.log")
                    .unwrap(),
                "[NaN DETECTED] After limiter: left={}, right={}",
                left,
                right
            );
        }

        // 13. Dry/Wet Mix
        left = left * self.params.dry_wet + dry_left * (1.0 - self.params.dry_wet);
        right = right * self.params.dry_wet + dry_right * (1.0 - self.params.dry_wet);

        // 14. Output Gain
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
        self.detected_pitch = 100.0;
        self.corrected_pitch = 100.0;
        self.smoothed_pitch = 100.0;
        self.pitch_confidence = 0.0;
        self.pitch_detection_counter = 0;
        self.sub_amplitude = 0.0;
        self.sub_target_amplitude = 0.0;
        self.harm2_amplitude = 0.0;
        self.harm2_target_amplitude = 0.0;
        self.harm3_amplitude = 0.0;
        self.harm3_target_amplitude = 0.0;
        self.harm4_amplitude = 0.0;
        self.harm4_target_amplitude = 0.0;

        self.pitch_detector = PitchDetector::new(self.sample_rate);
        self.pitch_quantizer.reset();
        self.noise_gate.reset();
        self.parametric_eq.reset();
        self.compressor.reset();
        self.de_esser.reset();
        self.sub_oscillator = {
            let mut osc = Oscillator::new(self.sample_rate);
            osc.set_waveform(Waveform::Sine);
            osc
        };
        self.harm2_oscillator = {
            let mut osc = Oscillator::new(self.sample_rate);
            osc.set_waveform(Waveform::Sine);
            osc
        };
        self.harm3_oscillator = {
            let mut osc = Oscillator::new(self.sample_rate);
            osc.set_waveform(Waveform::Sine);
            osc
        };
        self.harm4_oscillator = {
            let mut osc = Oscillator::new(self.sample_rate);
            osc.set_waveform(Waveform::Sine);
            osc
        };
        self.ring_modulator = RingModulator::new(self.sample_rate, 200.0);
        self.filter_follow = {
            let mut filter = BiquadFilter::new(self.sample_rate);
            filter.set_filter_type(FilterType::Lowpass);
            filter.set_cutoff(8000.0);
            filter.set_resonance(1.0);
            filter
        };
        self.exciter.reset();
        self.doubler.reset();
        self.choir.reset();
        self.multiband_distortion.clear();
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
        let params = VoiceParams {
            sub_level: 0.0,        // Disable sub for this test
            gate_threshold: -80.0, // Very low threshold
            ..Default::default()
        };
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
        let params = VoiceParams {
            pitch_confidence_threshold: 0.5,
            ..Default::default()
        };
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
        let params = VoiceParams {
            sub_level: 0.5,
            sub_ramp_time: 10.0,             // 10ms ramp
            pitch_confidence_threshold: 0.0, // Always enable sub
            ..Default::default()
        };
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
        let params = VoiceParams {
            dry_wet: 0.0,
            ..Default::default()
        };
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
        let params = VoiceParams {
            dry_wet: 1.0,
            ..Default::default()
        };
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

        let params = VoiceParams {
            gate_threshold: -40.0,
            comp_ratio: 8.0,
            sub_level: 0.7,
            ..Default::default()
        };

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
