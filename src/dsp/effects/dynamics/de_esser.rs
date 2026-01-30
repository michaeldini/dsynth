/// Intelligent De-Esser - Zero Latency, Dynamic EQ (Band-Pass)
///
/// Design goals:
/// - **Zero latency**: no lookahead, no buffering
/// - **Audible + controllable**: targets typical sibilance region directly
/// - **Dynamic reduction**: envelope follower + compressor-style gain computer
///
/// Implementation:
/// - Extract a sibilance band via band-pass.
/// - Compute gain from a detector envelope.
/// - Subtract the reduced portion of that band from the full-band signal.
///
/// Notes:
/// - `amount == 0.0` is a bit-perfect bypass.
/// - Threshold is normalized 0..1 where **higher = less sensitive**.
use crate::dsp::filters::filter::BiquadFilter;
use crate::dsp::signal_analyzer::SignalAnalysis;
use crate::params::FilterType;

pub struct DeEsser {
    sample_rate: f32,
    sibilance_center_hz: f32,

    // Detector band-pass (mono, stereo-linked).
    detector_bp: BiquadFilter,

    // Dynamic EQ cut: two cascaded high-shelf filters per channel.
    // This is much more audible than a narrow bell cut and is great for debugging.
    shelf_left_1: BiquadFilter,
    shelf_left_2: BiquadFilter,
    shelf_right_1: BiquadFilter,
    shelf_right_2: BiquadFilter,

    // Detector + gain smoothing
    hf_env: f32,
    gain: f32,

    // Metering (for debugging / UI)
    meter_env_db: f32,
    meter_gain_reduction_db: f32,
}

impl DeEsser {
    pub fn new(sample_rate: f32) -> Self {
        // Typical vocal sibilance energy is often ~4-8kHz depending on the voice/mic.
        // A dynamic EQ approach centered here is usually much more audible than "HF split".
        let sibilance_center_hz = 6500.0;

        let mut detector_bp = BiquadFilter::new(sample_rate);
        detector_bp.set_cutoff_update_interval(1);
        detector_bp.set_filter_type(FilterType::Bandpass);
        detector_bp.set_cutoff(sibilance_center_hz);
        detector_bp.set_bandwidth(1.4);

        let mut shelf_left_1 = BiquadFilter::new(sample_rate);
        shelf_left_1.set_cutoff_update_interval(1);
        shelf_left_1.set_filter_type(FilterType::HighShelf);
        shelf_left_1.set_cutoff(sibilance_center_hz);
        shelf_left_1.set_resonance(0.707);
        shelf_left_1.set_gain_db(0.0);

        let mut shelf_left_2 = BiquadFilter::new(sample_rate);
        shelf_left_2.set_cutoff_update_interval(1);
        shelf_left_2.set_filter_type(FilterType::HighShelf);
        shelf_left_2.set_cutoff(sibilance_center_hz);
        shelf_left_2.set_resonance(0.707);
        shelf_left_2.set_gain_db(0.0);

        let mut shelf_right_1 = BiquadFilter::new(sample_rate);
        shelf_right_1.set_cutoff_update_interval(1);
        shelf_right_1.set_filter_type(FilterType::HighShelf);
        shelf_right_1.set_cutoff(sibilance_center_hz);
        shelf_right_1.set_resonance(0.707);
        shelf_right_1.set_gain_db(0.0);

        let mut shelf_right_2 = BiquadFilter::new(sample_rate);
        shelf_right_2.set_cutoff_update_interval(1);
        shelf_right_2.set_filter_type(FilterType::HighShelf);
        shelf_right_2.set_cutoff(sibilance_center_hz);
        shelf_right_2.set_resonance(0.707);
        shelf_right_2.set_gain_db(0.0);

        Self {
            sample_rate,
            sibilance_center_hz,
            detector_bp,
            shelf_left_1,
            shelf_left_2,
            shelf_right_1,
            shelf_right_2,
            hf_env: 0.0,
            gain: 1.0,

            meter_env_db: -120.0,
            meter_gain_reduction_db: 0.0,
        }
    }

    /// Process stereo sample and return both processed audio and HF reduction delta.
    ///
    /// Parameters:
    /// - `threshold`: 0.0-1.0 (higher = less sensitive)
    /// - `amount`: 0.0-1.0 (0 = bypass)
    ///
    /// Returns:
    /// - `((out_l, out_r), (delta_l, delta_r))`
    ///
    /// Where `delta` is the removed part of the HF band:
    /// $$\Delta = x - y$$
    ///
    /// This is the best signal for a "Listen" mode and avoids false positives caused by
    /// subtractive EQ differences outside the target band.
    pub fn process(
        &mut self,
        left: f32,
        right: f32,
        threshold: f32,
        amount: f32,
        analysis: &SignalAnalysis,
    ) -> ((f32, f32), (f32, f32)) {
        let amount = amount.clamp(0.0, 1.0);
        if amount <= 0.0 {
            return ((left, right), (0.0, 0.0));
        }

        let threshold = threshold.clamp(0.0, 1.0);

        // Internal detection (stereo-linked).
        let mono = 0.5 * (left + right);
        let detector_in = self.detector_bp.process(mono).abs();

        // Detector envelope.
        let det_attack_s = 0.0002; // 0.2ms
        let det_release_s = 0.050; // 50ms
        let det_attack_coeff = (-1.0_f32 / (det_attack_s * self.sample_rate)).exp();
        let det_release_coeff = (-1.0_f32 / (det_release_s * self.sample_rate)).exp();

        if detector_in > self.hf_env {
            self.hf_env = detector_in + (self.hf_env - detector_in) * det_attack_coeff;
        } else {
            self.hf_env = detector_in + (self.hf_env - detector_in) * det_release_coeff;
        }

        // Gain computer.
        let eps = 1.0e-12;
        let env_db = 20.0 * (self.hf_env.max(eps)).log10();

        self.meter_env_db = env_db;

        // Map threshold so 1.0 is effectively "never triggers".
        // Use a curve so mid values (e.g. ~0.5) are still quite sensitive.
        // This makes Amount=100% clearly audible for auditioning.
        let threshold_db = -50.0 + threshold.powf(2.5) * 40.0; // [-50..-10] dB

        // Use a much higher ratio range so small overshoots create strong attenuation.
        let ratio = 2.0 + amount * 8.0; // 2:1 .. 10:1

        let _ = analysis;

        let over_db = (env_db - threshold_db).max(0.0);
        let reduced_over_db = over_db / ratio;
        let gain_db = -(over_db - reduced_over_db);

        // Moderate maximum reduction for a "pro" range.
        let max_reduction_db = 6.0 + 18.0 * amount; // 6..24 dB
        let gain_db = gain_db.clamp(-max_reduction_db, 0.0);
        let target_gain = 10.0_f32.powf(gain_db / 20.0);

        // Gain smoothing.
        let gain_attack_s = 0.0010;
        let gain_release_s = 0.080;
        let gain_attack_coeff = (-1.0_f32 / (gain_attack_s * self.sample_rate)).exp();
        let gain_release_coeff = (-1.0_f32 / (gain_release_s * self.sample_rate)).exp();

        if target_gain < self.gain {
            self.gain = target_gain + (self.gain - target_gain) * gain_attack_coeff;
        } else {
            self.gain = target_gain + (self.gain - target_gain) * gain_release_coeff;
        }

        // Meter gain reduction (positive dB).
        self.meter_gain_reduction_db = -20.0 * (self.gain.max(eps)).log10();

        // Apply dynamic cut via high-shelf filters.
        // Map computed GR (positive dB) directly into shelf cut amount.
        let gr_db = self.meter_gain_reduction_db.clamp(0.0, 48.0);
        let shelf_scale = 0.35 + 0.35 * amount; // 0.35..0.70
        let stage_gain_db = (-(gr_db * shelf_scale)).clamp(-24.0, 0.0);

        self.shelf_left_1.set_gain_db(stage_gain_db);
        self.shelf_left_2.set_gain_db(stage_gain_db);
        self.shelf_right_1.set_gain_db(stage_gain_db);
        self.shelf_right_2.set_gain_db(stage_gain_db);

        let mut out_l = self.shelf_left_2.process(self.shelf_left_1.process(left));
        let mut out_r = self
            .shelf_right_2
            .process(self.shelf_right_1.process(right));

        // Debug overkill: apply additional full-band ducking keyed by sibilance.
        // This makes the effect very obvious at Amount=100% while we validate behavior.
        // (We can dial this back or remove once the behavior is confirmed.)
        let duck_db = (self.meter_gain_reduction_db * amount * 0.7).clamp(0.0, 24.0);
        if duck_db > 0.0 {
            let duck_gain = 10.0_f32.powf(-duck_db / 20.0);
            out_l *= duck_gain;
            out_r *= duck_gain;
        }

        let delta_l = left - out_l;
        let delta_r = right - out_r;

        ((out_l, out_r), (delta_l, delta_r))
    }

    /// Current detector envelope in dBFS.
    pub fn meter_env_db(&self) -> f32 {
        self.meter_env_db
    }

    /// Current gain reduction amount (positive dB).
    pub fn meter_gain_reduction_db(&self) -> f32 {
        self.meter_gain_reduction_db
    }

    pub fn reset(&mut self) {
        self.detector_bp.reset();
        self.shelf_left_1.reset();
        self.shelf_left_2.reset();
        self.shelf_right_1.reset();
        self.shelf_right_2.reset();
        self.hf_env = 0.0;
        self.gain = 1.0;

        self.meter_env_db = -120.0;
        self.meter_gain_reduction_db = 0.0;
    }

    pub fn set_crossover_frequency(&mut self, freq_hz: f32) {
        // Backwards-compatible API: now controls the dynamic-EQ center frequency.
        let freq_hz = freq_hz.clamp(3000.0, 10000.0);
        self.sibilance_center_hz = freq_hz;
        self.detector_bp.set_cutoff(freq_hz);
        self.shelf_left_1.set_cutoff(freq_hz);
        self.shelf_left_2.set_cutoff(freq_hz);
        self.shelf_right_1.set_cutoff(freq_hz);
        self.shelf_right_2.set_cutoff(freq_hz);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analysis_with_strength(strength: f32) -> SignalAnalysis {
        let mut analysis = SignalAnalysis::default();
        analysis.has_sibilance = strength > 0.01;
        analysis.sibilance_strength = strength;
        analysis
    }

    #[test]
    fn test_bypass_amount_zero_is_bit_perfect() {
        let mut de_esser = DeEsser::new(44100.0);
        let analysis = analysis_with_strength(1.0);

        let ((out_l, out_r), _) = de_esser.process(0.1234, -0.4321, 0.8, 0.0, &analysis);
        assert_eq!(out_l, 0.1234);
        assert_eq!(out_r, -0.4321);
    }

    #[test]
    fn test_no_reduction_when_threshold_is_max() {
        let sample_rate = 44100.0;
        let mut de_esser = DeEsser::new(sample_rate);
        let analysis = analysis_with_strength(1.0);

        let threshold = 1.0; // least sensitive
        let amount = 1.0;
        let freq = 6000.0;

        let mut max_error: f32 = 0.0;
        for i in 0..4096 {
            let t = i as f32 / sample_rate;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.1;
            let ((out_l, _), _) = de_esser.process(input, input, threshold, amount, &analysis);
            max_error = max_error.max((out_l - input).abs());
        }

        assert!(max_error < 0.02);
    }

    #[test]
    fn test_reduces_high_frequency_energy() {
        let sample_rate = 44100.0;
        let mut de_esser = DeEsser::new(sample_rate);
        let analysis = analysis_with_strength(1.0);

        let threshold = 0.0; // most sensitive
        let amount = 1.0;
        let freq = 8000.0;

        // Warm-up to settle filters/envelopes.
        for i in 0..2048 {
            let t = i as f32 / sample_rate;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.8;
            let (_, _) = de_esser.process(input, input, threshold, amount, &analysis);
        }

        let mut in_sum = 0.0;
        let mut out_sum = 0.0;
        for i in 0..4096 {
            let t = (i + 2048) as f32 / sample_rate;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.8;
            let ((out_l, _), _) = de_esser.process(input, input, threshold, amount, &analysis);
            in_sum += input * input;
            out_sum += out_l * out_l;
        }

        let in_rms = (in_sum / 4096.0_f32).sqrt();
        let out_rms = (out_sum / 4096.0_f32).sqrt();

        assert!(out_rms < in_rms * 0.8);
    }

    #[test]
    fn test_reduces_sibilance_region_energy_5khz() {
        let sample_rate = 44100.0;
        let mut de_esser = DeEsser::new(sample_rate);
        let analysis = analysis_with_strength(1.0);

        let threshold = 0.0; // most sensitive
        let amount = 1.0;
        let freq = 5000.0;

        // Warm-up to settle filters/envelopes.
        for i in 0..2048 {
            let t = i as f32 / sample_rate;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.8;
            let (_, _) = de_esser.process(input, input, threshold, amount, &analysis);
        }

        let mut in_sum = 0.0;
        let mut out_sum = 0.0;
        for i in 0..4096 {
            let t = (i + 2048) as f32 / sample_rate;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.8;
            let ((out_l, _), _) = de_esser.process(input, input, threshold, amount, &analysis);
            in_sum += input * input;
            out_sum += out_l * out_l;
        }

        let in_rms = (in_sum / 4096.0_f32).sqrt();
        let out_rms = (out_sum / 4096.0_f32).sqrt();

        assert!(out_rms < in_rms * 0.8);
    }
}
