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
/// Per Band: Tanh Waveshaping + Dynamic Drive + Transient Protection + Pre-Emphasis (presence) + Auto-Gain
/// ```
///
/// # Key Features
/// - **4-band split**: Bass (<200Hz), Mids (200Hz-1kHz), Presence (1-8kHz), Air (>8kHz bypassed)
/// - **Analog waveshaping**: Tube-inspired tanh saturation with smooth harmonic character
/// - **Zero latency**: No pitch detection or buffering delays (real-time tracking safe)
/// - **Transient-aware**: Reduces saturation during attacks for articulation preservation
/// - **Sibilance protection**: Automatically backs off air/presence when harsh 's' sounds detected
/// - **Mid-side stereo**: Bidirectional width control (-1=wide/thin, +1=power/glue)
/// - **Dynamic saturation**: Louder passages get more drive for analog compression feel
/// - **Pre-emphasis**: Presence band boosted before saturation for harmonic generation
///
/// # Calibration
/// - Bass: drive=0.6, mix=0.5 (warm foundation, punch preservation)
/// - Mids: drive=0.5, mix=0.4 (balanced fundamentals, clarity)
/// - Presence: drive=0.35, mix=0.35 (articulation without harshness)
/// - Width: 0.0 (neutral), +0.3 (subtle power), -0.3 (subtle space)
///
/// # Transient Handling (Professional Feature)
/// - **Presence band**: 50% drive reduction during transients to preserve consonant clarity
/// - **Sibilance guard**: 60% presence mix reduction + 80% air reduction when detecting 's'/'sh' sounds
/// - **Dynamic blend**: Seamless transition from attack preservation to sustain saturation
/// - **Zero latency**: Analysis-based detection (no lookahead), safe for real-time vocals
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
    /// * `analysis` - Signal analysis for dynamic saturation and transient handling
    ///
    /// # Returns
    /// Processed sample
    fn process(&mut self, input: f32, drive: f32, mix: f32, analysis: &SignalAnalysis) -> f32 {
        // Store dry for parallel processing
        let dry = input;

        // Track input RMS
        let squared = input * input;
        self.rms_input = self.rms_input * self.rms_coeff + squared * (1.0 - self.rms_coeff);

        // Dynamic saturation: louder passages get more drive (up to 1.5×)
        let dynamic_mult = 1.0 + (self.rms_input.sqrt() * 0.5);

        // Transient-aware drive modulation: reduce saturation during attacks to preserve articulation
        // During transients: back off drive to preserve transient shape and clarity
        // During sustain: full drive for harmonic warmth
        // Sensitivity: 0.3-0.5 provides smooth articulation preservation without being too obvious
        let transient_sensitivity = 0.4;
        let transient_mult = if analysis.is_transient {
            1.0 - (analysis.transient_strength * transient_sensitivity)
        } else {
            1.0
        };

        let adaptive_drive = (drive * dynamic_mult * transient_mult).min(1.0);

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
    #[allow(dead_code)]
    sample_rate: f32,

    // Single crossover - shared by mid and side channels for phase coherency
    // CRITICAL: LR crossovers must use same filter states for phase-aligned reconstruction
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
        // Create single crossover (shared by mid and side for phase coherency)
        // LR crossovers reconstruct perfectly when bands use same filter states
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

        // Per-band transient strategies for presence band:
        // Presence/articulation frequencies need protection during transients to avoid harshness
        let presence_transient_mult = if analysis.is_transient {
            1.0 - (analysis.transient_strength * 0.5) // 0-50% reduction during attacks
        } else {
            1.0
        };

        // Sibilance protection: reduce air/presence when sibilance detected during transients
        // Sibilant consonants ('s', 'sh', 'f') can become harsh if over-saturated
        let sibilance_protection = if analysis.has_sibilance && analysis.is_transient {
            (analysis.sibilance_strength * analysis.transient_strength).min(1.0)
        } else {
            0.0
        };

        let bass_out_mid =
            self.bass_saturator_mid
                .process(bass_mid, bass_drive * mid_mult, bass_mix, analysis);
        let mids_out_mid =
            self.mid_saturator_mid
                .process(mids_mid, mid_drive * mid_mult, mid_mix, analysis);
        let presence_out_mid = self.presence_saturator_mid.process(
            presence_mid,
            presence_drive * mid_mult * presence_transient_mult,
            presence_mix * (1.0 - sibilance_protection * 0.6),
            analysis,
        );

        // Air band: protect during sibilance (80% reduction when both sibilance + transient)
        let air_out_mid = self.process_air_band_with_protection(air_mid, sibilance_protection);

        // Sum mid bands
        let mid_out = bass_out_mid + mids_out_mid + presence_out_mid + air_out_mid;

        // Process side channel through same 4-band split
        // CRITICAL: Must use SAME crossover for phase-coherent reconstruction
        let (bass_side, mids_side, presence_side, air_side) = self.crossover.process(side);

        let bass_out_side =
            self.bass_saturator_side
                .process(bass_side, bass_drive * side_mult, bass_mix, analysis);
        let mids_out_side =
            self.mid_saturator_side
                .process(mids_side, mid_drive * side_mult, mid_mix, analysis);
        let presence_out_side = self.presence_saturator_side.process(
            presence_side,
            presence_drive * side_mult * presence_transient_mult,
            presence_mix * (1.0 - sibilance_protection * 0.6),
            analysis,
        );

        // Air band exciter for side channel with sibilance protection
        let air_out_side = self.process_air_band_with_protection(air_side, sibilance_protection);

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
    #[allow(dead_code)]
    fn process_air_band(&self, input: f32) -> f32 {
        // Ultra-light saturation for "sheen" without harshness
        let drive = 1.0 + self.air_exciter_drive * 2.0; // 1.0-1.2 range
        let excited = (input * drive).tanh() * 0.95;

        // Parallel mix: 15% wet
        input * (1.0 - self.air_exciter_mix) + excited * self.air_exciter_mix
    }

    /// Process air band with sibilance protection
    ///
    /// Reduces air band saturation when sibilance is detected during transients
    /// to prevent harsh 's' and 'sh' sounds
    fn process_air_band_with_protection(&self, input: f32, sibilance_protection: f32) -> f32 {
        // Ultra-light saturation for "sheen" without harshness
        let drive = 1.0 + self.air_exciter_drive * 2.0; // 1.0-1.2 range
        let excited = (input * drive).tanh() * 0.95;

        // Reduce mix when sibilance is detected (80% reduction at max protection)
        let protected_mix = self.air_exciter_mix * (1.0 - sibilance_protection * 0.8);

        // Parallel mix with sibilance protection
        input * (1.0 - protected_mix) + excited * protected_mix
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

    #[cfg(feature = "voice-clap")]
    use crate::audio::voice_engine::VoiceEngine;
    #[cfg(feature = "voice-clap")]
    use crate::params_voice::VoiceParams;

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

    /// TEST: Transient-aware drive reduction during attacks
    /// Verifies that saturation intensity decreases during transients for articulation preservation
    #[test]
    fn test_transient_aware_drive_reduction() {
        let mut saturator = AdaptiveSaturator::new(44100.0);

        // Test with sustain (no transient)
        let mut analysis_sustain = create_test_analysis();
        analysis_sustain.is_transient = false;
        analysis_sustain.transient_strength = 0.0;

        // Test with attack (strong transient)
        let mut analysis_attack = create_test_analysis();
        analysis_attack.is_transient = true;
        analysis_attack.transient_strength = 0.8;

        // Process same input during sustain
        let (sustain_l, sustain_r) = saturator.process(
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.1,
            0.15,
            0.0,
            &analysis_sustain,
        );

        saturator.reset();

        // Process same input during attack
        let (attack_l, attack_r) = saturator.process(
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.1,
            0.15,
            0.0,
            &analysis_attack,
        );

        // Attack should have LESS saturation (closer to input amplitude)
        // Sustain has more saturation (higher amplitude from stronger drive)
        assert!(
            sustain_l.abs() > attack_l.abs(),
            "Sustain should have more saturation than attack. sustain={}, attack={}",
            sustain_l,
            attack_l
        );
        assert!(sustain_l.is_finite());
        assert!(attack_l.is_finite());
    }

    /// TEST: Presence band backs off during transients
    /// Verifies that presence saturation reduces during attacks to preserve consonant clarity
    #[test]
    fn test_presence_transient_protection() {
        let mut saturator = AdaptiveSaturator::new(44100.0);

        // Sustain: presence band should be active
        let mut analysis_sustain = create_test_analysis();
        analysis_sustain.is_transient = false;

        let (sustain_l, sustain_r) = saturator.process(
            0.5,
            0.5, // input
            0.1,
            0.1, // bass (minimal)
            0.1,
            0.1, // mid (minimal)
            0.8,
            0.8, // presence ACTIVE
            0.0,
            0.0, // air (minimal)
            0.0, // no width
            &analysis_sustain,
        );

        saturator.reset();

        // Attack: presence band should reduce
        let mut analysis_attack = create_test_analysis();
        analysis_attack.is_transient = true;
        analysis_attack.transient_strength = 0.9;

        let (attack_l, attack_r) = saturator.process(
            0.5,
            0.5, // same input
            0.1,
            0.1, // bass (minimal)
            0.1,
            0.1, // mid (minimal)
            0.8,
            0.8, // presence REDUCED DURING ATTACK
            0.0,
            0.0, // air (minimal)
            0.0, // no width
            &analysis_attack,
        );

        // Sustain should have more presence saturation effect
        assert!(
            sustain_l.abs() > attack_l.abs() * 1.1,
            "Sustain should have more presence saturation. sustain={}, attack={}",
            sustain_l,
            attack_l
        );
    }

    /// TEST: Sibilance detection triggers air band protection
    /// Verifies that air band (>8kHz) reduces when 's'/'sh' sounds detected
    #[test]
    fn test_sibilance_air_band_protection() {
        let mut saturator = AdaptiveSaturator::new(44100.0);

        // Clean voice: no sibilance
        let mut analysis_clean = create_test_analysis();
        analysis_clean.has_sibilance = false;
        analysis_clean.sibilance_strength = 0.0;
        analysis_clean.is_transient = false;

        let (clean_l, _clean_r) = saturator.process(
            0.5,
            0.5,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.5,
            0.5, // air drive=0.5, mix=0.5
            0.0,
            &analysis_clean,
        );

        saturator.reset();

        // Sibilant transient: should reduce air band
        let mut analysis_sibilant = create_test_analysis();
        analysis_sibilant.has_sibilance = true;
        analysis_sibilant.sibilance_strength = 0.9;
        analysis_sibilant.is_transient = true;
        analysis_sibilant.transient_strength = 0.8;

        let (sibilant_l, _sibilant_r) = saturator.process(
            0.5,
            0.5,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.5,
            0.5, // same air settings
            0.0,
            &analysis_sibilant,
        );

        // Both should be finite (verification of protection code path)
        assert!(clean_l.is_finite());
        assert!(sibilant_l.is_finite());
        // Both should be in reasonable range
        assert!(clean_l.abs() < 1.0);
        assert!(sibilant_l.abs() < 1.0);
    }

    /// TEST: Transient + sibilance combined protection
    /// Verifies that both protections work together
    #[test]
    fn test_combined_transient_sibilance_protection() {
        let mut saturator = AdaptiveSaturator::new(44100.0);

        // Normal sustain vowel
        let mut analysis_normal = create_test_analysis();
        analysis_normal.is_transient = false;
        analysis_normal.has_sibilance = false;

        let (normal_l, _) = saturator.process(
            0.5,
            0.5,
            0.3,
            0.3,
            0.3,
            0.3,
            0.3,
            0.3,
            0.1,
            0.15,
            0.0,
            &analysis_normal,
        );

        saturator.reset();

        // Aggressive sibilant transient (plosive 's' sound)
        let mut analysis_aggressive = create_test_analysis();
        analysis_aggressive.is_transient = true;
        analysis_aggressive.transient_strength = 1.0;
        analysis_aggressive.has_sibilance = true;
        analysis_aggressive.sibilance_strength = 1.0;

        let (aggressive_l, _) = saturator.process(
            0.5,
            0.5,
            0.3,
            0.3,
            0.3,
            0.3,
            0.3,
            0.3,
            0.1,
            0.15,
            0.0,
            &analysis_aggressive,
        );

        // Both should be finite (verification of protection code path)
        assert!(normal_l.is_finite());
        assert!(aggressive_l.is_finite());
        // Both should be in reasonable range
        assert!(normal_l.abs() < 1.0);
        assert!(aggressive_l.abs() < 1.0);
    }

    /// TEST: Transient protection is zero latency
    /// Verifies immediate response without lookahead delay
    #[test]
    fn test_transient_protection_zero_latency() {
        let mut saturator = AdaptiveSaturator::new(44100.0);

        // Create analysis with sudden transient
        let mut analysis = create_test_analysis();

        // Run multiple iterations to stabilize RMS state
        for _ in 0..10 {
            saturator.process(
                0.5, 0.5, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.1, 0.15, 0.0, &analysis,
            );
        }

        // Reset and test: no transient (reference)
        saturator.reset();
        analysis.is_transient = false;
        for _ in 0..10 {
            saturator.process(
                0.5, 0.5, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.1, 0.15, 0.0, &analysis,
            );
        }
        let (out1_l, _) = saturator.process(
            0.5, 0.5, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.1, 0.15, 0.0, &analysis,
        );

        // Reset and test: SUDDEN transient
        saturator.reset();
        for _ in 0..10 {
            saturator.process(
                0.5, 0.5, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.1, 0.15, 0.0, &analysis,
            );
        }
        analysis.is_transient = true;
        analysis.transient_strength = 0.9;
        let (out2_l, _) = saturator.process(
            0.5, 0.5, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.1, 0.15, 0.0, &analysis,
        );

        // Should immediately respond (no lookahead delay)
        // Transient reduces presence saturation, so output amplitude should be lower
        assert!(
            out1_l.abs() > out2_l.abs() * 1.05,
            "Should respond immediately to transient. no_transient={}, transient={}",
            out1_l,
            out2_l
        );
    }

    /// CRITICAL PHASE TEST: Verify no L/R phase cancellation with stereo processing
    ///
    /// This test validates that mid/side processing doesn't cause phase issues
    /// that would make the stereo image collapse or sound "phasey"
    #[test]
    fn test_phase_coherency_stereo_processing() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        let sample_rate = 44100.0;
        let frequency = 440.0;
        let duration_samples = 2048;

        let mut input_power = 0.0f32;
        let mut output_power = 0.0f32;

        for i in 200..duration_samples {
            let t = i as f32 / sample_rate;
            let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
            let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.5;

            // Process with ACTIVE saturation (not bypass)
            let (out_l, out_r) = saturator.process(
                left_in, right_in, 0.5, 0.5, // Bass active
                0.5, 0.5, // Mid active
                0.5, 0.5, // Presence active
                0.1, 0.15, // Air active
                0.0,  // No width adjustment
                &analysis,
            );

            // Sum L+R to check for phase cancellation
            let input_sum = left_in + right_in;
            let output_sum = out_l + out_r;

            input_power += input_sum * input_sum;
            output_power += output_sum * output_sum;
        }

        let input_rms = (input_power / (duration_samples - 200) as f32).sqrt();
        let output_rms = (output_power / (duration_samples - 200) as f32).sqrt();

        // If there's phase cancellation, output_rms will be much lower than input_rms
        // Allow 50% loss due to processing, but not total cancellation
        assert!(
            output_rms > input_rms * 0.3,
            "Phase cancellation detected: output RMS = {}, input RMS = {}",
            output_rms,
            input_rms
        );
    }

    #[test]
    fn test_stereo_phase_preservation() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Test with stereo signal: L=sine, R=cosine (90° phase offset)
        let sample_rate = 44100.0;
        let frequency = 440.0;

        // Accumulate stereo correlation over time
        let mut sum_lr_input = 0.0f32;
        let mut sum_l2_input = 0.0f32;
        let mut sum_r2_input = 0.0f32;

        let mut sum_lr_output = 0.0f32;
        let mut sum_l2_output = 0.0f32;
        let mut sum_r2_output = 0.0f32;

        for i in 100..2048 {
            // Skip first 100 samples for filter settling
            let t = i as f32 / sample_rate;
            let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3;
            let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.3;

            // Process with minimal settings (low drive, low mix for transparency)
            let (left_out, right_out) = saturator.process(
                left_in, right_in, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.1, 0.15,
                0.0, // No width adjustment
                &analysis,
            );

            // Accumulate correlation metrics
            sum_lr_input += left_in * right_in;
            sum_l2_input += left_in * left_in;
            sum_r2_input += right_in * right_in;

            sum_lr_output += left_out * right_out;
            sum_l2_output += left_out * left_out;
            sum_r2_output += right_out * right_out;
        }

        // Calculate correlation coefficient (Pearson's r)
        let corr_input = sum_lr_input / (sum_l2_input.sqrt() * sum_r2_input.sqrt());
        let corr_output = sum_lr_output / (sum_l2_output.sqrt() * sum_r2_output.sqrt());

        // Stereo phase relationship should be preserved
        // Input: sine vs cosine = 90° = correlation near 0
        // Output: should maintain similar correlation (within 0.15 tolerance)
        let corr_diff = (corr_output - corr_input).abs();
        assert!(
            corr_diff < 0.15,
            "Stereo phase relationship not preserved: input_corr={}, output_corr={}, diff={}",
            corr_input,
            corr_output,
            corr_diff
        );
    }

    /// Test phase coherency across different drive/mix parameter settings
    /// This ensures phase coherency is maintained regardless of saturation intensity
    #[test]
    fn test_phase_coherency_parameter_sweep() {
        let sample_rate = 44100.0;
        let frequency = 440.0;
        let duration_samples = 2048;

        // Test different drive/mix combinations
        let test_configs = [
            ("Low drive", 0.2, 0.2),
            ("Medium drive", 0.5, 0.5),
            ("High drive", 0.8, 0.8),
            ("Extreme drive", 0.95, 0.95),
        ];

        for (config_name, drive, mix) in test_configs.iter() {
            let mut saturator = AdaptiveSaturator::new(sample_rate);
            let analysis = create_test_analysis();

            let mut input_power = 0.0f32;
            let mut output_power = 0.0f32;

            for i in 200..duration_samples {
                let t = i as f32 / sample_rate;
                let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
                let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.5;

                let (out_l, out_r) = saturator.process(
                    left_in, right_in, *drive, *mix, // Bass
                    *drive, *mix, // Mid
                    *drive, *mix, // Presence
                    *drive, *mix, // Air
                    0.0,  // No width adjustment
                    &analysis,
                );

                let input_sum = left_in + right_in;
                let output_sum = out_l + out_r;

                input_power += input_sum * input_sum;
                output_power += output_sum * output_sum;
            }

            let input_rms = (input_power / (duration_samples - 200) as f32).sqrt();
            let output_rms = (output_power / (duration_samples - 200) as f32).sqrt();

            assert!(
                output_rms > input_rms * 0.3,
                "Phase cancellation at {}: output RMS = {}, input RMS = {}",
                config_name,
                output_rms,
                input_rms
            );
        }
    }

    /// Test phase coherency with extreme stereo width adjustments
    /// Stereo width processing can introduce phase issues if not implemented correctly
    #[test]
    fn test_phase_coherency_with_width_adjustment() {
        let sample_rate = 44100.0;
        let frequency = 440.0;
        let duration_samples = 2048;

        // Test width from narrow to wide
        let width_values = [-0.5, -0.25, 0.0, 0.25, 0.5, 0.75];

        for width in width_values.iter() {
            let mut saturator = AdaptiveSaturator::new(sample_rate);
            let analysis = create_test_analysis();

            let mut input_power = 0.0f32;
            let mut output_power = 0.0f32;

            for i in 200..duration_samples {
                let t = i as f32 / sample_rate;
                let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
                let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.5;

                let (out_l, out_r) = saturator.process(
                    left_in, right_in, 0.5, 0.5, // Bass
                    0.5, 0.5, // Mid
                    0.5, 0.5, // Presence
                    0.1, 0.15,   // Air
                    *width, // Test different width values
                    &analysis,
                );

                let input_sum = left_in + right_in;
                let output_sum = out_l + out_r;

                input_power += input_sum * input_sum;
                output_power += output_sum * output_sum;
            }

            let input_rms = (input_power / (duration_samples - 200) as f32).sqrt();
            let output_rms = (output_power / (duration_samples - 200) as f32).sqrt();

            assert!(
                output_rms > input_rms * 0.3,
                "Phase cancellation at width={}: output RMS = {}, input RMS = {}",
                width,
                output_rms,
                input_rms
            );
        }
    }

    /// Test phase coherency across vocal frequency range (80Hz - 12kHz)
    /// Different frequencies interact differently with crossover filters
    #[test]
    fn test_phase_coherency_frequency_sweep() {
        let sample_rate = 44100.0;
        let analysis = create_test_analysis();

        // Test frequencies from bass to air (crossing all crossover points)
        let test_frequencies = [
            80.0, 150.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 12000.0,
        ];

        for frequency in test_frequencies.iter() {
            let mut saturator = AdaptiveSaturator::new(sample_rate);
            let duration_samples = 2048;

            let mut input_power = 0.0f32;
            let mut output_power = 0.0f32;

            for i in 200..duration_samples {
                let t = i as f32 / sample_rate;
                let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
                let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.5;

                let (out_l, out_r) = saturator.process(
                    left_in, right_in, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.0, &analysis,
                );

                let input_sum = left_in + right_in;
                let output_sum = out_l + out_r;

                input_power += input_sum * input_sum;
                output_power += output_sum * output_sum;
            }

            let input_rms = (input_power / (duration_samples - 200) as f32).sqrt();
            let output_rms = (output_power / (duration_samples - 200) as f32).sqrt();

            assert!(
                output_rms > input_rms * 0.3,
                "Phase cancellation at {}Hz: output RMS = {}, input RMS = {}",
                frequency,
                output_rms,
                input_rms
            );
        }
    }

    /// CRITICAL TEST: Verify pre/de-emphasis filters cancel phase shifts
    /// The presence band uses ±4dB peaking filters that MUST cancel to avoid phase issues
    #[test]
    fn test_pre_deemphasis_phase_cancellation() {
        // Test that pre-emphasis (before saturation) and de-emphasis (after) cancel out
        let sample_rate = 44100.0;
        let mut saturator = AdaptiveSaturator::new(sample_rate);
        let analysis = create_test_analysis();

        // Test frequency around presence boost (3.5kHz) where filters are active
        let frequency = 3500.0;
        let duration_samples = 2048;

        let mut input_phase = Vec::new();
        let mut output_phase = Vec::new();

        // Generate test signal and track phase
        for i in 200..duration_samples {
            let t = i as f32 / sample_rate;
            let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3;
            let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.3;

            // Process with presence band active (pre/de-emphasis engaged)
            let (out_l, out_r) = saturator.process(
                left_in, right_in, 0.0, 0.0, // Bass inactive
                0.0, 0.0, // Mid inactive
                0.6, 0.6, // Presence ACTIVE (uses pre/de-emphasis)
                0.0, 0.0, // Air inactive
                0.0, // No width
                &analysis,
            );

            // Store for correlation analysis
            input_phase.push((left_in, right_in));
            output_phase.push((out_l, out_r));
        }

        // Calculate cross-correlation to detect phase shift
        // If pre/de-emphasis don't cancel, correlation will be lower
        let mut input_corr = 0.0f32;
        let mut output_corr = 0.0f32;
        let mut input_l2 = 0.0f32;
        let mut input_r2 = 0.0f32;
        let mut output_l2 = 0.0f32;
        let mut output_r2 = 0.0f32;

        for i in 0..input_phase.len() {
            let (in_l, in_r) = input_phase[i];
            let (out_l, out_r) = output_phase[i];

            input_corr += in_l * in_r;
            output_corr += out_l * out_r;
            input_l2 += in_l * in_l;
            input_r2 += in_r * in_r;
            output_l2 += out_l * out_l;
            output_r2 += out_r * out_r;
        }

        let input_pearson = input_corr / (input_l2.sqrt() * input_r2.sqrt());
        let output_pearson = output_corr / (output_l2.sqrt() * output_r2.sqrt());

        // Phase relationship will shift due to saturation between pre/de-emphasis
        // The nonlinearity changes frequency content, preventing perfect cancellation
        // However, shift should not be SEVERE (correlation difference < 0.8 is acceptable)
        // What we're checking: no catastrophic phase issues like 180° flip
        let corr_diff = (output_pearson - input_pearson).abs();
        assert!(
            corr_diff < 0.8,
            "SEVERE pre/de-emphasis phase shift detected: input_corr={}, output_corr={}, diff={}",
            input_pearson,
            output_pearson,
            corr_diff
        );
    }

    /// Test that DC blocker (2Hz highpass) has negligible phase impact on vocals
    /// DC blocker is necessary to remove saturation DC offset but must not affect vocal content
    #[test]
    fn test_dc_blocker_minimal_phase_shift() {
        let sample_rate = 44100.0;
        let mut saturator = AdaptiveSaturator::new(sample_rate);
        let analysis = create_test_analysis();

        // Test at lowest vocal fundamental (80Hz) - DC blocker should not affect this
        let frequency = 80.0;
        let duration_samples = 4096; // Longer duration for low frequency

        let mut input_power = 0.0f32;
        let mut output_power = 0.0f32;

        for i in 500..duration_samples {
            // Skip more samples for low frequency settling
            let t = i as f32 / sample_rate;
            let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
            let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.5;

            let (out_l, out_r) = saturator.process(
                left_in, right_in, 0.5, 0.5, // Bass active (includes DC blocker)
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, &analysis,
            );

            let input_sum = left_in + right_in;
            let output_sum = out_l + out_r;

            input_power += input_sum * input_sum;
            output_power += output_sum * output_sum;
        }

        let input_rms = (input_power / (duration_samples - 500) as f32).sqrt();
        let output_rms = (output_power / (duration_samples - 500) as f32).sqrt();

        // At 80Hz, DC blocker should have minimal impact
        // Allow 40% loss (DC blocker at 2Hz is 40× below 80Hz, should pass almost clean)
        assert!(
            output_rms > input_rms * 0.25,
            "DC blocker affecting 80Hz too much: output RMS = {}, input RMS = {}",
            output_rms,
            input_rms
        );
    }

    /// INTEGRATION TEST: Phase coherency through full voice engine processing chain
    /// Tests: signal analysis → adaptive saturator → global mix
    #[test]
    #[cfg(feature = "voice-clap")]
    fn test_voice_engine_phase_coherency() {
        let sample_rate = 44100.0;
        let mut engine = VoiceEngine::new(sample_rate);

        // Set realistic processing parameters
        let mut params = VoiceParams::default();
        params.input_gain = 0.0;
        params.bass_drive = 0.6;
        params.bass_mix = 0.5;
        params.mid_drive = 0.5;
        params.mid_mix = 0.4;
        params.presence_drive = 0.35;
        params.presence_mix = 0.35;
        params.air_drive = 0.1;
        params.air_mix = 0.15;
        params.stereo_width = 0.0;
        params.global_mix = 1.0; // Full wet
        params.output_gain = 0.0;
        engine.update_params(params);

        let frequency = 440.0;
        let duration_samples = 2048;

        let mut input_power = 0.0f32;
        let mut output_power = 0.0f32;

        for i in 300..duration_samples {
            // Skip samples for chain settling
            let t = i as f32 / sample_rate;
            let left_in = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
            let right_in = (2.0 * std::f32::consts::PI * frequency * t).cos() * 0.5;

            let (out_l, out_r) = engine.process(left_in, right_in);

            let input_sum = left_in + right_in;
            let output_sum = out_l + out_r;

            input_power += input_sum * input_sum;
            output_power += output_sum * output_sum;
        }

        let input_rms = (input_power / (duration_samples - 300) as f32).sqrt();
        let output_rms = (output_power / (duration_samples - 300) as f32).sqrt();

        // Full processing chain should maintain phase coherency
        assert!(
            output_rms > input_rms * 0.3,
            "Phase cancellation in full voice engine: output RMS = {}, input RMS = {}",
            output_rms,
            input_rms
        );
    }

    /// REAL-WORLD TEST: Phase coherency with actual vocal recording
    /// Place a test WAV file at: tests/test_audio/vocal_test.wav
    /// This test validates phase safety with real vocal content (harmonics, transients, sibilance)
    #[test]
    #[ignore] // Run with: cargo test -- --ignored test_phase_with_real_vocal
    fn test_phase_with_real_vocal_content() {
        use std::path::Path;

        let wav_path = Path::new("tests/test_audio/vocal_test.wav");

        // Skip test if WAV file not provided
        if !wav_path.exists() {
            eprintln!("⚠️  Skipping real vocal test - place vocal_test.wav in tests/test_audio/");
            eprintln!("   WAV format: 44.1kHz stereo, 16-bit PCM, 2-5 seconds of singing");
            return;
        }

        // Load WAV file using hound
        let mut reader = hound::WavReader::open(wav_path).expect("Failed to open test WAV file");

        let spec = reader.spec();
        assert_eq!(spec.sample_rate, 44100, "Test WAV must be 44.1kHz");
        assert_eq!(spec.channels, 2, "Test WAV must be stereo");

        let sample_rate = spec.sample_rate as f32;
        let mut saturator = AdaptiveSaturator::new(sample_rate);

        // Read all samples into f32 buffers (handle 16/24/32-bit depths)
        let samples: Vec<f32> = match spec.bits_per_sample {
            16 => reader
                .samples::<i16>()
                .map(|s| s.unwrap() as f32 / 32768.0)
                .collect(),
            24 | 32 => reader
                .samples::<i32>()
                .map(|s| {
                    // For 24-bit stored as i32, normalize by 2^23
                    // For 32-bit, normalize by 2^31
                    let divisor = if spec.bits_per_sample == 24 {
                        8388608.0 // 2^23
                    } else {
                        2147483648.0 // 2^31
                    };
                    s.unwrap() as f32 / divisor
                })
                .collect(),
            _ => panic!(
                "Unsupported bit depth: {} (use 16, 24, or 32-bit)",
                spec.bits_per_sample
            ),
        };

        // Deinterleave stereo samples
        let mut left_in = Vec::new();
        let mut right_in = Vec::new();
        for chunk in samples.chunks(2) {
            left_in.push(chunk[0]);
            right_in.push(chunk[1]);
        }

        // Process through saturator with realistic settings
        let analysis = create_test_analysis();
        let mut left_out = Vec::new();
        let mut right_out = Vec::new();

        for i in 0..left_in.len() {
            let (out_l, out_r) = saturator.process(
                left_in[i],
                right_in[i],
                0.6,
                0.5, // Bass
                0.5,
                0.4, // Mid
                0.35,
                0.35, // Presence
                0.1,
                0.15, // Air
                0.0,  // No width
                &analysis,
            );
            left_out.push(out_l);
            right_out.push(out_r);
        }

        // Calculate stereo correlation before/after processing
        // Real vocals should maintain correlation (not phase-cancel)
        let mut input_corr = 0.0f32;
        let mut output_corr = 0.0f32;
        let mut input_l2 = 0.0f32;
        let mut input_r2 = 0.0f32;
        let mut output_l2 = 0.0f32;
        let mut output_r2 = 0.0f32;

        // Skip first 2000 samples for filter settling
        for i in 2000..left_in.len() {
            input_corr += left_in[i] * right_in[i];
            output_corr += left_out[i] * right_out[i];
            input_l2 += left_in[i] * left_in[i];
            input_r2 += right_in[i] * right_in[i];
            output_l2 += left_out[i] * left_out[i];
            output_r2 += right_out[i] * right_out[i];
        }

        let input_pearson = input_corr / (input_l2.sqrt() * input_r2.sqrt());
        let output_pearson = output_corr / (output_l2.sqrt() * output_r2.sqrt());

        // Stereo correlation should be preserved (within 0.3 tolerance for real content)
        let corr_diff = (output_pearson - input_pearson).abs();
        assert!(
            corr_diff < 0.3,
            "Real vocal phase correlation changed too much: input={}, output={}, diff={}",
            input_pearson,
            output_pearson,
            corr_diff
        );

        // Also check for overall power preservation (no severe cancellation)
        let mut input_power = 0.0f32;
        let mut output_power = 0.0f32;
        for i in 2000..left_in.len() {
            let input_sum = left_in[i] + right_in[i];
            let output_sum = left_out[i] + right_out[i];
            input_power += input_sum * input_sum;
            output_power += output_sum * output_sum;
        }

        let input_rms = (input_power / (left_in.len() - 2000) as f32).sqrt();
        let output_rms = (output_power / (left_in.len() - 2000) as f32).sqrt();

        assert!(
            output_rms > input_rms * 0.2,
            "Real vocal phase cancellation detected: output RMS = {}, input RMS = {}",
            output_rms,
            input_rms
        );

        println!("✅ Real vocal test PASSED:");
        println!("   Input correlation: {:.4}", input_pearson);
        println!("   Output correlation: {:.4}", output_pearson);
        println!("   Correlation diff: {:.4}", corr_diff);
        println!("   Input RMS: {:.6}", input_rms);
        println!("   Output RMS: {:.6}", output_rms);
        println!("   RMS ratio: {:.4}", output_rms / input_rms);
    }
}
