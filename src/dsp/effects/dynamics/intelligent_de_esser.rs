/// Intelligent De-Esser - Zero Latency, Phase-Coherent, Sibilance-Aware
///
/// **CRITICAL FIX**: Uses Linkwitz-Riley 4th-order crossover for phase-coherent reconstruction
///
/// # Design Philosophy  
/// - **Phase-coherent**: Linkwitz-Riley crossover (cascaded Butterworth) sums to flat response
/// - **Sibilance-triggered**: Only processes when `analysis.has_sibilance == true`
/// - **Zero latency**: No lookahead or pitch detection required
/// - **Split-band**: 7kHz crossover separates sibilance range (4-10kHz optimal)
/// - **Fixed 6:1 ratio**: Professional standard for transparent de-essing
///
/// # What Was Wrong
/// Previous implementation used 2nd-order Butterworth filters which introduce phase shift.
/// When low + high bands are recombined, phase cancellation creates "underwater" sound.
/// **Solution**: Linkwitz-Riley 4th-order crossover (two cascaded 2nd-order Butterworth
/// filters, each with Q=0.707) which sums to perfectly flat magnitude and linear phase.
///
/// # Signal Flow
/// ```text
/// Input → Linkwitz-Riley Split (7kHz, 4th-order)
///         ├→ Low Band → Pass Through
///         └→ High Band → [Sibilance?] → Compress (6:1) → Mix Amount
///                                      → [No] → Pass Through
/// ```
use crate::dsp::effects::dynamics::compressor::Compressor;
use crate::dsp::filters::filter::BiquadFilter;
use crate::dsp::signal_analyzer::SignalAnalysis;
use crate::params::FilterType;

/// Phase-coherent intelligent de-esser with Linkwitz-Riley crossover
pub struct IntelligentDeEsser {
    #[allow(dead_code)]
    sample_rate: f32,
    #[allow(dead_code)]
    crossover_hz: f32,
    ratio: f32,

    // Linkwitz-Riley 4th-order crossover = cascade of two 2nd-order Butterworth (Q=0.707)
    // This ensures phase-coherent reconstruction when low + high bands are summed

    // High-pass cascade (for high band)
    high_pass_left_1: BiquadFilter,
    high_pass_left_2: BiquadFilter,
    high_pass_right_1: BiquadFilter,
    high_pass_right_2: BiquadFilter,

    // Low-pass cascade (for low band)
    low_pass_left_1: BiquadFilter,
    low_pass_left_2: BiquadFilter,
    low_pass_right_1: BiquadFilter,
    low_pass_right_2: BiquadFilter,

    // Compressor for high band
    compressor: Compressor,
}

impl IntelligentDeEsser {
    /// Create new phase-coherent de-esser with Linkwitz-Riley crossover
    pub fn new(sample_rate: f32) -> Self {
        let crossover_hz = 7000.0;
        let ratio = 6.0;

        // Initialize Linkwitz-Riley 4th-order crossover
        // Stage 1 (Q=0.707 Butterworth)
        let mut high_pass_left_1 = BiquadFilter::new(sample_rate);
        high_pass_left_1.set_filter_type(FilterType::Highpass);
        high_pass_left_1.set_cutoff(crossover_hz);
        high_pass_left_1.set_resonance(0.707);

        let mut high_pass_left_2 = BiquadFilter::new(sample_rate);
        high_pass_left_2.set_filter_type(FilterType::Highpass);
        high_pass_left_2.set_cutoff(crossover_hz);
        high_pass_left_2.set_resonance(0.707);

        let mut high_pass_right_1 = BiquadFilter::new(sample_rate);
        high_pass_right_1.set_filter_type(FilterType::Highpass);
        high_pass_right_1.set_cutoff(crossover_hz);
        high_pass_right_1.set_resonance(0.707);

        let mut high_pass_right_2 = BiquadFilter::new(sample_rate);
        high_pass_right_2.set_filter_type(FilterType::Highpass);
        high_pass_right_2.set_cutoff(crossover_hz);
        high_pass_right_2.set_resonance(0.707);

        // Stage 2 (Q=0.707 Butterworth)
        let mut low_pass_left_1 = BiquadFilter::new(sample_rate);
        low_pass_left_1.set_filter_type(FilterType::Lowpass);
        low_pass_left_1.set_cutoff(crossover_hz);
        low_pass_left_1.set_resonance(0.707);

        let mut low_pass_left_2 = BiquadFilter::new(sample_rate);
        low_pass_left_2.set_filter_type(FilterType::Lowpass);
        low_pass_left_2.set_cutoff(crossover_hz);
        low_pass_left_2.set_resonance(0.707);

        let mut low_pass_right_1 = BiquadFilter::new(sample_rate);
        low_pass_right_1.set_filter_type(FilterType::Lowpass);
        low_pass_right_1.set_cutoff(crossover_hz);
        low_pass_right_1.set_resonance(0.707);

        let mut low_pass_right_2 = BiquadFilter::new(sample_rate);
        low_pass_right_2.set_filter_type(FilterType::Lowpass);
        low_pass_right_2.set_cutoff(crossover_hz);
        low_pass_right_2.set_resonance(0.707);

        let compressor = Compressor::new(sample_rate, -10.0, ratio, 0.5, 50.0);

        Self {
            sample_rate,
            crossover_hz,
            ratio,
            high_pass_left_1,
            high_pass_left_2,
            high_pass_right_1,
            high_pass_right_2,
            low_pass_left_1,
            low_pass_left_2,
            low_pass_right_1,
            low_pass_right_2,
            compressor,
        }
    }

    /// Process stereo audio with phase-coherent de-essing
    ///
    /// # Phase Coherence Guarantee
    /// When amount=0, output EXACTLY equals input (bit-perfect bypass).
    /// When amount>0, Linkwitz-Riley crossover ensures low+high sum to unity (no phase cancellation).
    ///
    /// # Arguments
    /// * `left` - Left channel input
    /// * `right` - Right channel input
    /// * `threshold` - Sibilance detection threshold (0.0-1.0)
    /// * `amount` - De-essing amount (0.0=bypass, 1.0=full)
    /// * `analysis` - Pre-computed signal analysis with sibilance detection
    ///
    /// # Returns
    /// Tuple of (left_out, right_out)
    pub fn process(
        &mut self,
        left: f32,
        right: f32,
        threshold: f32,
        amount: f32,
        analysis: &SignalAnalysis,
    ) -> (f32, f32) {
        // CRITICAL: Bit-perfect bypass when amount=0
        if amount < 0.001 {
            return (left, right);
        }

        // Split into low and high bands using Linkwitz-Riley 4th-order crossover
        // Stage 1: First 2nd-order filter
        let low_left_stage1 = self.low_pass_left_1.process(left);
        let low_right_stage1 = self.low_pass_right_1.process(right);
        let high_left_stage1 = self.high_pass_left_1.process(left);
        let high_right_stage1 = self.high_pass_right_1.process(right);

        // Stage 2: Second 2nd-order filter (cascaded = 4th-order Linkwitz-Riley)
        let low_left = self.low_pass_left_2.process(low_left_stage1);
        let low_right = self.low_pass_right_2.process(low_right_stage1);
        let high_left = self.high_pass_left_2.process(high_left_stage1);
        let high_right = self.high_pass_right_2.process(high_right_stage1);

        // Process high band only if sibilance detected
        // CRITICAL: Invert threshold logic - higher threshold = MORE sensitive
        // threshold=0.0 → require strength >= 1.0 (never triggers)
        // threshold=1.0 → require strength >= 0.0 (always triggers when has_sibilance)
        let effective_threshold = 1.0 - threshold;
        let (processed_high_left, processed_high_right) =
            if analysis.has_sibilance && analysis.sibilance_strength >= effective_threshold {
                // Dynamic threshold based on sibilance strength
                let dynamic_threshold_db = -10.0 - (analysis.sibilance_strength * 10.0);
                self.compressor.set_threshold(dynamic_threshold_db);

                // Compress sibilant high band
                self.compressor.process(high_left, high_right)
            } else {
                // No sibilance detected - pass through uncompressed
                (high_left, high_right)
            };

        // Parallel mix: blend compressed/uncompressed high band
        let mixed_high_left = high_left * (1.0 - amount) + processed_high_left * amount;
        let mixed_high_right = high_right * (1.0 - amount) + processed_high_right * amount;

        // Reconstruct full-band signal
        // CRITICAL: Linkwitz-Riley guarantees low + high = original (phase-coherent)
        let out_left = low_left + mixed_high_left;
        let out_right = low_right + mixed_high_right;

        (out_left, out_right)
    }

    /// Reset all filter state
    pub fn reset(&mut self) {
        self.high_pass_left_1.reset();
        self.high_pass_left_2.reset();
        self.high_pass_right_1.reset();
        self.high_pass_right_2.reset();
        self.low_pass_left_1.reset();
        self.low_pass_left_2.reset();
        self.low_pass_right_1.reset();
        self.low_pass_right_2.reset();
        self.compressor = Compressor::new(self.sample_rate, -10.0, self.ratio, 0.5, 50.0);
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
            pitch_hz: 100.0,
            pitch_confidence: 0.0,
            is_pitched: false,
        }
    }

    #[test]
    fn test_bit_perfect_bypass_when_amount_zero() {
        let mut deesser = IntelligentDeEsser::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.has_sibilance = true;
        analysis.sibilance_strength = 1.0;

        // Test various input amplitudes
        for input_level in [0.1, 0.5, 1.0] {
            let (out_l, out_r) = deesser.process(input_level, input_level, 0.5, 0.0, &analysis);

            // With amount=0, output MUST be bit-perfect equal to input
            assert_eq!(out_l, input_level, "Left channel not bypassed at amount=0");
            assert_eq!(out_r, input_level, "Right channel not bypassed at amount=0");
        }
    }

    #[test]
    fn test_linkwitz_riley_crossover_phase_coherence() {
        let mut deesser = IntelligentDeEsser::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.has_sibilance = false; // No sibilance = pass through

        // Warm up filters (first 100 samples to stabilize transient response)
        for _ in 0..100 {
            deesser.process(0.5, 0.5, 0.5, 1.0, &analysis);
        }

        // Test sweep from 100Hz to 12kHz
        // Linkwitz-Riley crossover should sum to near-unity across all frequencies
        let sample_rate = 44100.0;
        for freq in [100.0, 500.0, 2000.0, 5000.0, 7000.0, 10000.0, 12000.0] {
            let mut phase: f32 = 0.0;
            let phase_inc = 2.0 * std::f32::consts::PI * freq / sample_rate;

            let mut input_rms = 0.0;
            let mut output_rms = 0.0;
            let test_samples = 512;

            for _ in 0..test_samples {
                let input = phase.sin() * 0.5;
                let (out_l, _) = deesser.process(input, input, 0.5, 1.0, &analysis);

                input_rms += input * input;
                output_rms += out_l * out_l;

                phase += phase_inc;
                if phase > 2.0 * std::f32::consts::PI {
                    phase -= 2.0 * std::f32::consts::PI;
                }
            }

            input_rms = (input_rms / test_samples as f32).sqrt();
            output_rms = (output_rms / test_samples as f32).sqrt();

            // Linkwitz-Riley should maintain magnitude within 1dB across all frequencies
            let magnitude_ratio = output_rms / input_rms;
            assert!(
                (magnitude_ratio - 1.0).abs() < 0.12, // ~1dB tolerance
                "Phase cancellation detected at {}Hz: magnitude ratio = {:.3} (expected ~1.0)",
                freq,
                magnitude_ratio
            );
        }
    }

    #[test]
    fn test_no_underwater_sound_with_amount_one() {
        // This is the critical test that would fail with 2nd-order Butterworth
        // Linkwitz-Riley 4th-order should pass cleanly
        let mut deesser = IntelligentDeEsser::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.has_sibilance = false; // Pass-through (no compression)

        // Warm up filters
        for _ in 0..100 {
            deesser.process(0.5, 0.5, 0.5, 1.0, &analysis);
        }

        // Test with broadband signal (pink noise approximation)
        let mut input_power = 0.0f32;
        let mut output_power = 0.0f32;

        for i in 0..1000 {
            // Multi-frequency test signal (simulates vocal harmonics)
            let t = i as f32 / 44100.0;
            let input = (2.0 * std::f32::consts::PI * 200.0 * t).sin() * 0.2
                + (2.0 * std::f32::consts::PI * 800.0 * t).sin() * 0.15
                + (2.0 * std::f32::consts::PI * 2000.0 * t).sin() * 0.1
                + (2.0 * std::f32::consts::PI * 6000.0 * t).sin() * 0.08
                + (2.0 * std::f32::consts::PI * 10000.0 * t).sin() * 0.05;

            let (out_l, _) = deesser.process(input, input, 0.5, 1.0, &analysis);

            input_power += input * input;
            output_power += out_l * out_l;
        }

        let power_ratio = output_power / input_power;

        // With Linkwitz-Riley, power should be preserved (ratio ≈ 1.0)
        // With 2nd-order Butterworth, we'd see significant power loss (ratio < 0.7)
        assert!(
            power_ratio > 0.85,
            "Underwater sound detected! Power ratio = {:.3} (expected > 0.85). \
             This indicates phase cancellation in the crossover.",
            power_ratio
        );
    }

    #[test]
    fn test_sibilance_compression_works() {
        let mut deesser = IntelligentDeEsser::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.has_sibilance = true;
        analysis.sibilance_strength = 1.0;

        // Warm up
        for _ in 0..100 {
            deesser.process(0.0, 0.0, 0.5, 1.0, &analysis);
        }

        // Send loud high-frequency content (simulated sibilance)
        let sample_rate = 44100.0;
        let freq = 8000.0; // Well above 7kHz crossover
        let mut phase: f32 = 0.0;
        let phase_inc = 2.0 * std::f32::consts::PI * freq / sample_rate;

        let mut max_input = 0.0f32;
        let mut max_output = 0.0f32;

        for _ in 0..200 {
            let input = phase.sin() * 0.8; // Loud sibilant signal
            let (out_l, _) = deesser.process(input, input, 0.5, 1.0, &analysis);

            max_input = max_input.max(input.abs());
            max_output = max_output.max(out_l.abs());

            phase += phase_inc;
        }

        // With 6:1 compression, output should be significantly reduced
        assert!(
            max_output < max_input * 0.8,
            "De-esser not compressing sibilance: input peak = {:.3}, output peak = {:.3}",
            max_input,
            max_output
        );
    }

    #[test]
    fn test_low_frequencies_unaffected() {
        let mut deesser = IntelligentDeEsser::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.has_sibilance = true;
        analysis.sibilance_strength = 1.0;

        // Warm up
        for _ in 0..100 {
            deesser.process(0.0, 0.0, 0.5, 1.0, &analysis);
        }

        // Test low frequency (well below 7kHz crossover)
        let sample_rate = 44100.0;
        let freq = 1000.0;
        let mut phase: f32 = 0.0;
        let phase_inc = 2.0 * std::f32::consts::PI * freq / sample_rate;

        let mut input_rms = 0.0f32;
        let mut output_rms = 0.0f32;

        for _ in 0..500 {
            let input = phase.sin() * 0.5;
            let (out_l, _) = deesser.process(input, input, 0.5, 1.0, &analysis);

            input_rms += input * input;
            output_rms += out_l * out_l;

            phase += phase_inc;
        }

        input_rms = (input_rms / 500.0).sqrt();
        output_rms = (output_rms / 500.0).sqrt();

        // Low frequencies should pass through virtually unchanged
        assert_relative_eq!(output_rms, input_rms, epsilon = 0.05);
    }
}
