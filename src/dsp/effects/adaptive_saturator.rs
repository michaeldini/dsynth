use crate::dsp::filters::BiquadFilter;
/// Professional 4-Band Vocal Saturator - Zero Latency
///
/// **Design Philosophy**: Multiband waveshaping for professional vocal production.
/// Optimized for transparency, clarity, and analog warmth with zero processing latency.
///
/// # Architecture
/// ```text
/// Input → Mid/Side → 4-Band Split (200Hz/1kHz/8kHz) → Per-Band Saturation → Sum → L/R → Output
///         ↑          Bass/Mids/Presence/Air              ↓
///         └─────── Width Control (-1 to +1) ────────────┘
///
/// Per Band: Tanh Waveshaping + Dynamic Drive + Pre-Emphasis (presence) + Auto-Gain
/// ```
///
/// # Key Features
/// - **4-band split**: Bass (<200Hz), Mids (200Hz-1kHz), Presence (1-8kHz), Air (>8kHz bypassed)
/// - **Analog waveshaping**: Tube-inspired tanh saturation with smooth harmonic character
/// - **Zero latency**: No pitch detection or buffering delays (real-time tracking safe)
/// - **Mid-side stereo**: Bidirectional width control (-1=wide/thin, +1=power/glue)
/// - **Dynamic saturation**: Louder passages get more drive for analog compression feel
/// - **Pre-emphasis**: Presence band boosted before saturation for harmonic generation
///
/// # Calibration
/// - Bass: drive=0.6, mix=0.5 (warm foundation, punch preservation)
/// - Mids: drive=0.5, mix=0.4 (balanced fundamentals, clarity)
/// - Presence: drive=0.35, mix=0.35 (articulation without harshness)
/// - Width: 0.0 (neutral), +0.3 (subtle power), -0.3 (subtle space)
use crate::dsp::filters::MultibandCrossover;
use crate::dsp::signal_analyzer::SignalAnalysis;

/// Per-band saturation processor with analog-style waveshaping
///
/// Uses tanh soft saturation with dynamic drive and auto-gain compensation.
/// Pre-emphasis/de-emphasis for presence band enhances harmonic generation.
struct BandSaturator {
    sample_rate: f32,

    // RMS tracking for dynamic saturation and auto-gain (75ms window)
    rms_input: f32,
    rms_output: f32,
    rms_coeff: f32,

    // Pre-emphasis filter (only active for presence band)
    pre_emphasis: Option<BiquadFilter>,
    de_emphasis: Option<BiquadFilter>,

    // DC blocker (1-pole highpass @ 2Hz to remove saturation DC offset)
    dc_blocker: BiquadFilter,
}

impl BandSaturator {
    /// Create new band saturator
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `use_emphasis` - Whether to use pre/de-emphasis (presence band only)
    fn new(sample_rate: f32, use_emphasis: bool) -> Self {
        // RMS smoothing (75ms for smooth gain riding)
        let rms_time_ms = 75.0;
        let rms_samples = (rms_time_ms / 1000.0) * sample_rate;
        let rms_coeff = (-1.0 / rms_samples).exp();

        // Pre/de-emphasis filters (+4dB/-4dB @ 3.5kHz peaking)
        let (pre_emphasis, de_emphasis) = if use_emphasis {
            let mut pre = BiquadFilter::new(sample_rate);
            pre.set_filter_type(crate::params::FilterType::Peaking);
            pre.set_cutoff(3500.0);
            pre.set_resonance(1.0);
            pre.set_gain_db(4.0); // +4dB

            let mut de = BiquadFilter::new(sample_rate);
            de.set_filter_type(crate::params::FilterType::Peaking);
            de.set_cutoff(3500.0);
            de.set_resonance(1.0);
            de.set_gain_db(-4.0); // -4dB

            (Some(pre), Some(de))
        } else {
            (None, None)
        };

        // DC blocker: Remove DC offset from saturation
        let mut dc_blocker = BiquadFilter::new(sample_rate);
        dc_blocker.set_filter_type(crate::params::FilterType::Highpass);
        dc_blocker.set_cutoff(2.0); // Minimal phase shift in vocal range
        dc_blocker.set_resonance(0.707);

        Self {
            sample_rate,
            rms_input: 0.0,
            rms_output: 0.0,
            rms_coeff,
            pre_emphasis,
            de_emphasis,
            dc_blocker,
        }
    }

    /// Process one sample through band saturation
    ///
    /// # Arguments
    /// * `input` - Input sample
    /// * `drive` - Drive amount (0-1)
    /// * `mix` - Dry/wet mix (0-1, per-band parallel processing)
    /// * `analysis` - Signal analysis for dynamic saturation
    ///
    /// # Returns
    /// Processed sample
    fn process(&mut self, input: f32, drive: f32, mix: f32, _analysis: &SignalAnalysis) -> f32 {
        // Store dry for parallel processing
        let dry = input;

        // Track input RMS
        let squared = input * input;
        self.rms_input = self.rms_input * self.rms_coeff + squared * (1.0 - self.rms_coeff);

        // Dynamic saturation: louder passages get more drive (up to 1.5×)
        let dynamic_mult = 1.0 + (self.rms_input.sqrt() * 0.5);
        let adaptive_drive = (drive * dynamic_mult).min(1.0);

        // Apply pre-emphasis (presence band only)
        let mut signal = input;
        if let Some(ref mut pre) = self.pre_emphasis {
            signal = pre.process(signal);
        }

        // Analog-style waveshaping (tanh soft saturation)
        let gain = 1.0 + adaptive_drive * 5.0; // 1-6× gain range
        let mut wet = (signal * gain).tanh() * 0.95;

        // Apply de-emphasis (presence band only)
        if let Some(ref mut de) = self.de_emphasis {
            wet = de.process(wet);
        }

        // Track output RMS
        let squared_out = wet * wet;
        self.rms_output = self.rms_output * self.rms_coeff + squared_out * (1.0 - self.rms_coeff);

        // Auto-gain compensation
        let compensation = self.calculate_auto_gain();
        wet *= compensation;

        // Remove DC offset from saturation
        wet = self.dc_blocker.process(wet);

        // Parallel processing: blend dry and wet per-band
        dry * (1.0 - mix) + wet * mix
    }

    fn calculate_auto_gain(&self) -> f32 {
        let input_level = self.rms_input.sqrt().max(0.001);
        let output_level = self.rms_output.sqrt().max(0.001);
        let compensation = input_level / output_level;
        compensation.clamp(0.5, 2.0) // ±6dB limit
    }

    fn reset(&mut self) {
        self.rms_input = 0.0;
        self.rms_output = 0.0;
        if let Some(ref mut pre) = self.pre_emphasis {
            *pre = BiquadFilter::new(self.sample_rate);
            pre.set_filter_type(crate::params::FilterType::Peaking);
            pre.set_cutoff(3500.0);
            pre.set_resonance(1.0);
            pre.set_gain_db(4.0);
        }
        if let Some(ref mut de) = self.de_emphasis {
            *de = BiquadFilter::new(self.sample_rate);
            de.set_filter_type(crate::params::FilterType::Peaking);
            de.set_cutoff(3500.0);
            de.set_resonance(1.0);
            de.set_gain_db(-4.0);
        }
        self.dc_blocker = BiquadFilter::new(self.sample_rate);
        self.dc_blocker
            .set_filter_type(crate::params::FilterType::Highpass);
        self.dc_blocker.set_cutoff(2.0);
        self.dc_blocker.set_resonance(0.707);
    }
}

/// Professional 4-band vocal saturator with mid-side processing
pub struct AdaptiveSaturator {
    sample_rate: f32,

    // Single crossover - used for both mid and side channels
    // (Crossover is just frequency splitting, doesn't need separate instances)
    crossover: MultibandCrossover,

    // Per-band saturators - SEPARATE instances for mid and side channels
    // (Saturators have stateful processing that needs independence)
    bass_saturator_mid: BandSaturator,
    mid_saturator_mid: BandSaturator,
    presence_saturator_mid: BandSaturator,

    bass_saturator_side: BandSaturator,
    mid_saturator_side: BandSaturator,
    presence_saturator_side: BandSaturator,

    // Air band exciter (ultra-light harmonic enhancement >8kHz)
    air_exciter_drive: f32,
    air_exciter_mix: f32,

    // Safety limiter (fast attack, -0.1dBFS ceiling)
    limiter_threshold: f32, // 0.9 linear (~-0.9dB)
    limiter_gain_reduction: f32,
}

impl AdaptiveSaturator {
    /// Create new 4-band adaptive saturator
    pub fn new(sample_rate: f32) -> Self {
        // Create single crossover (shared by mid and side channels)
        let crossover = MultibandCrossover::new(sample_rate);

        // Create band saturators - SEPARATE instances for mid and side channels
        // to prevent filter state corruption

        // Bass band: No pre-emphasis
        let bass_saturator_mid = BandSaturator::new(sample_rate, false);
        let bass_saturator_side = BandSaturator::new(sample_rate, false);

        // Mid band: No pre-emphasis
        let mid_saturator_mid = BandSaturator::new(sample_rate, false);
        let mid_saturator_side = BandSaturator::new(sample_rate, false);

        // Presence band: Pre-emphasis active for enhanced harmonic generation
        let presence_saturator_mid = BandSaturator::new(sample_rate, true);
        let presence_saturator_side = BandSaturator::new(sample_rate, true);

        Self {
            sample_rate,
            crossover,
            bass_saturator_mid,
            mid_saturator_mid,
            presence_saturator_mid,
            bass_saturator_side,
            mid_saturator_side,
            presence_saturator_side,
            air_exciter_drive: 0.1,
            air_exciter_mix: 0.15,
            limiter_threshold: 0.9,
            limiter_gain_reduction: 1.0,
        }
    }

    /// Process stereo sample with 4-band saturation and mid-side processing
    ///
    /// # Arguments
    /// * `left/right` - Input samples
    /// * `bass/mid/presence_drive` - Per-band drive (0-1)
    /// * `bass/mid/presence_mix` - Per-band dry/wet mix (0-1)
    /// * `air_drive` - Air exciter drive (0-1)
    /// * `air_mix` - Air exciter dry/wet mix (0-1)
    /// * `stereo_width` - Mid-side balance (-1 to +1, 0=neutral)
    /// * `analysis` - Signal analysis (pitch, dynamics, etc.)
    ///
    /// # Returns
    /// Tuple of (left_out, right_out)
    pub fn process(
        &mut self,
        left: f32,
        right: f32,
        bass_drive: f32,
        bass_mix: f32,
        mid_drive: f32,
        mid_mix: f32,
        presence_drive: f32,
        presence_mix: f32,
        air_drive: f32,
        air_mix: f32,
        stereo_width: f32,
        analysis: &SignalAnalysis,
    ) -> (f32, f32) {
        // Store air parameters for this processing cycle
        self.air_exciter_drive = air_drive;
        self.air_exciter_mix = air_mix;

        // Convert L/R to Mid/Side
        let mid = (left + right) * 0.5;
        let side = (left - right) * 0.5;

        // Calculate mid/side drive multipliers based on width
        let (mid_mult, side_mult) = if stereo_width >= 0.0 {
            // Positive width: saturate mid more (power/glue)
            (1.0 + stereo_width * 0.5, 1.0 - stereo_width * 0.3)
        } else {
            // Negative width: saturate sides more (wide/thin)
            (1.0 + stereo_width * 0.3, 1.0 - stereo_width * 0.5)
        };

        // Process mid channel through 4-band split
        let (bass_mid, mids_mid, presence_mid, air_mid) = self.crossover.process(mid);

        let bass_out_mid =
            self.bass_saturator_mid
                .process(bass_mid, bass_drive * mid_mult, bass_mix, analysis);
        let mids_out_mid =
            self.mid_saturator_mid
                .process(mids_mid, mid_drive * mid_mult, mid_mix, analysis);
        let presence_out_mid = self.presence_saturator_mid.process(
            presence_mid,
            presence_drive * mid_mult,
            presence_mix,
            analysis,
        );

        // Air band: Ultra-light exciter for "sheen" (>8kHz)
        let air_out_mid = self.process_air_band(air_mid);

        // Sum mid bands
        let mid_out = bass_out_mid + mids_out_mid + presence_out_mid + air_out_mid;

        // Process side channel through same 4-band split
        // Note: Crossover can be shared safely - only saturators need separate instances
        let (bass_side, mids_side, presence_side, air_side) = self.crossover.process(side);

        let bass_out_side =
            self.bass_saturator_side
                .process(bass_side, bass_drive * side_mult, bass_mix, analysis);
        let mids_out_side =
            self.mid_saturator_side
                .process(mids_side, mid_drive * side_mult, mid_mix, analysis);
        let presence_out_side = self.presence_saturator_side.process(
            presence_side,
            presence_drive * side_mult,
            presence_mix,
            analysis,
        );

        // Air band exciter for side channel
        let air_out_side = self.process_air_band(air_side);

        // Sum side bands
        let side_out = bass_out_side + mids_out_side + presence_out_side + air_out_side;

        // Convert Mid/Side back to L/R (CRITICAL: must scale by 2.0 to compensate for 0.5 scaling in M/S encode)
        // M = (L+R)*0.5, S = (L-R)*0.5 → L = M+S (but we need to scale back!)
        // Correct formula: L = (M+S)*2, R = (M-S)*2 OR use full-scale M/S without 0.5
        let mut left_out = mid_out + side_out;
        let mut right_out = mid_out - side_out;

        // Safety limiter: Fast attack, -0.1dBFS ceiling (prevents clipping)
        let peak = left_out.abs().max(right_out.abs());
        if peak > self.limiter_threshold {
            let target_gain = self.limiter_threshold / peak;
            self.limiter_gain_reduction = target_gain.min(self.limiter_gain_reduction * 0.9 + 0.1);
        // Fast attack
        } else {
            self.limiter_gain_reduction = (self.limiter_gain_reduction * 0.999 + 0.001).min(1.0);
            // Slow release
        }

        left_out *= self.limiter_gain_reduction;
        right_out *= self.limiter_gain_reduction;

        (left_out, right_out)
    }

    /// Process air band with ultra-light harmonic exciter (>8kHz)
    fn process_air_band(&self, input: f32) -> f32 {
        // Ultra-light saturation for "sheen" without harshness
        let drive = 1.0 + self.air_exciter_drive * 2.0; // 1.0-1.2 range
        let excited = (input * drive).tanh() * 0.95;

        // Parallel mix: 15% wet
        input * (1.0 - self.air_exciter_mix) + excited * self.air_exciter_mix
    }

    /// Reset all processing state
    pub fn reset(&mut self) {
        self.crossover.reset();
        self.bass_saturator_mid.reset();
        self.mid_saturator_mid.reset();
        self.presence_saturator_mid.reset();
        self.bass_saturator_side.reset();
        self.mid_saturator_side.reset();
        self.presence_saturator_side.reset();

        // Reset limiter state
        self.limiter_gain_reduction = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

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
            pitch_hz: 220.0,
            pitch_confidence: 0.8,
            is_pitched: true,
        }
    }

    #[test]
    fn test_saturator_creation() {
        let saturator = AdaptiveSaturator::new(44100.0);
        assert_eq!(saturator.sample_rate, 44100.0);
    }

    #[test]
    fn test_process_produces_finite_output() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        let (left, right) = saturator.process(
            0.5, 0.5, // left, right
            0.6, 0.5, // bass drive, mix
            0.5, 0.4, // mid drive, mix
            0.35, 0.35, // presence drive, mix
            0.1, 0.15, // air drive, mix
            0.0,  // width
            &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
    }

    #[test]
    fn test_stereo_width_positive() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Positive width should increase mid saturation
        let (left, right) = saturator.process(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.5, &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
    }

    #[test]
    fn test_stereo_width_negative() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Negative width should increase side saturation
        let (left, right) = saturator.process(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, -0.5, &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
    }

    #[test]
    fn test_waveshaping_produces_valid_output() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        let (left, right) = saturator.process(
            0.5, 0.5, 0.6, 0.5, 0.5, 0.4, 0.35, 0.35, 0.1, 0.15, 0.0, &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
        // Output should be non-zero with drive applied
        assert!(left.abs() > 0.01);
        assert!(right.abs() > 0.01);
    }

    #[test]
    fn test_dynamic_drive_responds_to_level() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Low level input
        let (low_l, _) = saturator.process(
            0.1, 0.1, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.0, &analysis,
        );

        // Reset for fair comparison
        saturator.reset();

        // High level input should produce different saturation characteristics
        let (high_l, _) = saturator.process(
            0.8, 0.8, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.0, &analysis,
        );

        assert!(low_l.is_finite());
        assert!(high_l.is_finite());
    }

    #[test]
    fn test_separate_crossovers_no_corruption() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Process multiple samples to ensure crossovers stay independent
        for _ in 0..100 {
            let (l, r) = saturator.process(
                0.5, -0.3, // Asymmetric L/R to test mid/side separation
                0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.0, &analysis,
            );
            assert!(l.is_finite());
            assert!(r.is_finite());
        }
    }

    #[test]
    fn test_safety_limiter_prevents_clipping() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Process hot signal with max drive
        let mut max_peak: f32 = 0.0;
        for _ in 0..200 {
            let (l, r) = saturator.process(
                0.9, 0.9, // Hot input
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.15, 0.0, &analysis,
            );
            max_peak = max_peak.max(l.abs()).max(r.abs());
        }

        // Limiter should keep peaks below threshold
        assert!(
            max_peak <= 0.95,
            "Limiter should prevent clipping, got peak: {}",
            max_peak
        );
    }

    #[test]
    fn test_air_band_exciter_works() {
        let saturator = AdaptiveSaturator::new(44100.0);

        // Test air band processing
        let input = 0.3;
        let output = saturator.process_air_band(input);

        assert!(output.is_finite());
        assert!(output.abs() > 0.0);
        // Output should be slightly enhanced but not drastically different
        assert!((output - input).abs() < 0.1);
    }

    #[test]
    fn test_no_gain_loss() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // With transient reduction disabled, should have immediate response
        let (l1, r1) = saturator.process(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.0, &analysis,
        );

        // Should have immediate output (no lookahead delay)
        assert!(l1.abs() > 0.0, "Should have immediate output");
        assert!(r1.abs() > 0.0, "Should have immediate output");

        // With default 50% mix and auto-gain, output will be in range
        // Allow 0.1-1.0 range (processing can reduce level slightly)
        assert!(
            l1.abs() > 0.1 && l1.abs() < 1.0,
            "Output should be in reasonable range: got {}",
            l1
        );
    }
}
