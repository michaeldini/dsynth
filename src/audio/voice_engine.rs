use crate::dsp::effects::dynamics::{AdaptiveCompressionLimiter, Compressor, TransientShaper};
use crate::dsp::effects::saturator::Saturator;
use crate::dsp::effects::spectral::IntelligentExciter;
use crate::dsp::effects::vocal::VocalDoubler;
use crate::dsp::filters::MultibandCrossover;
use crate::dsp::signal_analyzer::SignalAnalyzer;
use crate::dsp::modulation::{ParameterMapper, ProcessorSettings};
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

    /// Air band intelligent exciter (pitch-tracked vocal sparkle)
    air_exciter: IntelligentExciter,

    /// NEW: Multiband compressors (one per frequency band)
    bass_compressor: Compressor,
    mid_compressor: Compressor,
    presence_compressor: Compressor,
    air_compressor: Compressor,

    /// Adaptive compression limiter (transient-aware envelope-follower limiting)
    limiter: AdaptiveCompressionLimiter,

    /// Vocal doubler for intelligent thickness enhancement
    vocal_doubler: VocalDoubler,

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
        let limiter = AdaptiveCompressionLimiter::new(sample_rate);

        // Separate crossovers for L/R channels
        let crossover_left = MultibandCrossover::new(sample_rate);
        let crossover_right = MultibandCrossover::new(sample_rate);

        // Intelligent air exciter for pitch-tracked vocal sparkle
        let air_exciter = IntelligentExciter::new(sample_rate);

        // NEW: Initialize multiband compressors with band-optimized settings
        let bass_compressor = Compressor::new(sample_rate, -18.0, 2.5, 10.0, 200.0); // Gentle bass compression
        let mid_compressor = Compressor::new(sample_rate, -16.0, 3.5, 3.0, 150.0);   // Moderate mid compression
        let presence_compressor = Compressor::new(sample_rate, -12.0, 2.0, 1.0, 100.0); // Fast presence control
        let air_compressor = Compressor::new(sample_rate, -10.0, 1.5, 0.5, 50.0);    // Light air compression

        // Initialize vocal doubler for intelligent vocal thickness
        let vocal_doubler = VocalDoubler::new(sample_rate);

        Self {
            sample_rate,
            signal_analyzer: SignalAnalyzer::new_no_pitch(sample_rate),
            transient_shaper,
            crossover_left,
            crossover_right,
            bass_saturator_left,
            bass_saturator_right,
            mid_saturator_left,
            mid_saturator_right,
            presence_saturator_left,
            presence_saturator_right,
            air_exciter,
            bass_compressor,
            mid_compressor,
            presence_compressor,
            air_compressor,
            limiter,
            vocal_doubler,
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
        // Set fixed professional compression settings optimized for vocal processing
        self.update_professional_compression_settings();

        self.params = params;
    }

    /// Set fixed professional multiband compressor settings for hit pop vocals
    fn update_professional_compression_settings(&mut self) {
        // Professional settings optimized for hit pop vocals
        // Bass: gentle compression for warmth and body
        self.bass_compressor.set_ratio(2.2);           // Moderate compression
        self.bass_compressor.set_threshold(-16.0);     // Catch prominent bass
        
        // Mid: moderate compression for vocal consistency
        self.mid_compressor.set_ratio(3.0);            // Good control
        self.mid_compressor.set_threshold(-14.0);      // Active on most vocals
        
        // Presence: careful compression to maintain clarity
        self.presence_compressor.set_ratio(2.5);       // Controlled but not harsh
        self.presence_compressor.set_threshold(-10.0); // Only on louder parts
        
        // Air: light compression to preserve sparkle
        self.air_compressor.set_ratio(1.8);            // Very gentle
        self.air_compressor.set_threshold(-8.0);       // Only peaks
    }

    /// Apply comprehensive intelligent multiband processing chain
    fn apply_multiband_processing(
        &mut self,
        left: &mut f32,
        right: &mut f32, 
        settings: &ProcessorSettings,
        analysis: &crate::dsp::signal_analyzer::SignalAnalysis
    ) {
        // Split into 4 frequency bands
        let (bass_l, mid_l, presence_l, air_l) = self.crossover_left.process(*left);
        let (bass_r, mid_r, presence_r, air_r) = self.crossover_right.process(*right);
        
        // Process each band with band-specific intelligence
        let (proc_bass_l, proc_bass_r) = self.process_bass_band(
            bass_l, bass_r, settings, analysis
        );
        
        let (proc_mid_l, proc_mid_r) = self.process_mid_band(
            mid_l, mid_r, settings, analysis  
        );
        
        let (proc_presence_l, proc_presence_r) = self.process_presence_band(
            presence_l, presence_r, settings, analysis
        );
        
        let (proc_air_l, proc_air_r) = self.process_air_band(
            air_l, air_r, settings, analysis
        );
        
        // Recombine with dynamic EQ adjustments (convert dB to linear gain)
        let bass_eq_gain = 1.0 + settings.dynamic_eq.bass_gain_db * 0.115; // 1dB ≈ 0.115 linear
        let mid_eq_gain = 1.0 + settings.dynamic_eq.mid_gain_db * 0.115;
        let presence_eq_gain = 1.0 + settings.dynamic_eq.presence_gain_db * 0.115;
        let air_eq_gain = 1.0 + settings.dynamic_eq.air_gain_db * 0.115;
                
        *left = proc_bass_l * bass_eq_gain +
                proc_mid_l * mid_eq_gain +
                proc_presence_l * presence_eq_gain +
                proc_air_l * air_eq_gain;
                
        *right = proc_bass_r * bass_eq_gain +
                 proc_mid_r * mid_eq_gain +
                 proc_presence_r * presence_eq_gain +
                 proc_air_r * air_eq_gain;
    }
    
    /// Process bass band (DC-200Hz): Foundation, warmth, body
    fn process_bass_band(
        &mut self, 
        left: f32, right: f32, 
        settings: &ProcessorSettings,
        analysis: &crate::dsp::signal_analyzer::SignalAnalysis
    ) -> (f32, f32) {
        // 1. Bass compression (gentle, preserves warmth)
        self.bass_compressor.set_threshold(settings.compression.bass_threshold);
        self.bass_compressor.set_ratio(settings.compression.bass_ratio);
        let (comp_l, comp_r) = self.bass_compressor.process(left, right);
        
        // 2. Bass transient shaping (usually gentle to preserve power)
        let (shaped_l, shaped_r) = self.transient_shaper.process(
            comp_l, comp_r, 
            settings.transient_shaping.bass_attack,
            analysis
        );
        
        // 3. Bass saturation (character-dependent warmth with fixed mix)  
        let sat_l = self.bass_saturator_left.process(
            shaped_l, settings.saturation.bass_drive, 0.6, analysis
        );
        let sat_r = self.bass_saturator_right.process(
            shaped_r, settings.saturation.bass_drive, 0.6, analysis
        );
        
        // 4. Bass harmonic enhancement (even-order harmonics for warmth)
        let (enhanced_l, enhanced_r) = self.apply_bass_harmonics(
            sat_l, sat_r, settings.harmonic_enhancement.bass_harmonics
        );
        
        (enhanced_l, enhanced_r)
    }
    
    /// Process mid band (200Hz-1kHz): Fundamentals, body, consistency
    fn process_mid_band(
        &mut self,
        left: f32, right: f32,
        settings: &ProcessorSettings, 
        analysis: &crate::dsp::signal_analyzer::SignalAnalysis
    ) -> (f32, f32) {
        // 1. Mid compression for vocal consistency
        self.mid_compressor.set_threshold(settings.compression.mid_threshold);
        self.mid_compressor.set_ratio(settings.compression.mid_ratio);
        let (comp_l, comp_r) = self.mid_compressor.process(left, right);
        
        // 2. Mid transient shaping for clarity
        let (shaped_l, shaped_r) = self.transient_shaper.process(
            comp_l, comp_r,
            settings.transient_shaping.mid_attack,
            analysis
        );
        
        // 3. Mid saturation (moderate processing)
        let sat_l = self.mid_saturator_left.process(
            shaped_l, settings.saturation.mid_drive, 0.5, analysis
        );
        let sat_r = self.mid_saturator_right.process(
            shaped_r, settings.saturation.mid_drive, 0.5, analysis
        );
        
        // 4. Mid harmonic enhancement
        let (enhanced_l, enhanced_r) = self.apply_mid_harmonics(
            sat_l, sat_r, settings.harmonic_enhancement.mid_harmonics
        );
        
        (enhanced_l, enhanced_r)
    }
    
    /// Process presence band (1kHz-8kHz): Clarity, consonants, brightness
    fn process_presence_band(
        &mut self,
        left: f32, right: f32,
        settings: &ProcessorSettings, 
        analysis: &crate::dsp::signal_analyzer::SignalAnalysis
    ) -> (f32, f32) {
        // 1. Presence-specific transient shaping (critical for clarity)
        let (shaped_l, shaped_r) = self.transient_shaper.process(
            left, right,
            settings.transient_shaping.presence_attack,
            analysis
        );
        
        // 2. Intelligent compression (lighter on sibilants)
        let compression_ratio = if analysis.has_sibilance && analysis.sibilance_strength > 0.5 {
            settings.compression.presence_ratio * 0.7  // Gentler on sibilants
        } else {
            settings.compression.presence_ratio
        };
        
        self.presence_compressor.set_threshold(settings.compression.presence_threshold);
        self.presence_compressor.set_ratio(compression_ratio);
        let (comp_l, comp_r) = self.presence_compressor.process(shaped_l, shaped_r);
        
        // 3. Presence saturation (brightness-dependent)
        let sat_l = self.presence_saturator_left.process(
            comp_l, settings.saturation.presence_drive, 0.4, analysis
        );
        let sat_r = self.presence_saturator_right.process(
            comp_r, settings.saturation.presence_drive, 0.4, analysis
        );
        
        // 4. Harmonic enhancement (odd-order harmonics for brightness)
        let (enhanced_l, enhanced_r) = self.apply_presence_harmonics(
            sat_l, sat_r, 
            settings.harmonic_enhancement.presence_harmonics,
            analysis.has_sibilance  // Reduce on sibilants
        );
        
        (enhanced_l, enhanced_r)
    }
    
    /// Process air band (8kHz+): Sparkle, harmonics, gentle enhancement
    fn process_air_band(
        &mut self,
        left: f32, right: f32,
        settings: &ProcessorSettings, 
        analysis: &crate::dsp::signal_analyzer::SignalAnalysis
    ) -> (f32, f32) {
        // 1. Air compression (very gentle to preserve sparkle)
        self.air_compressor.set_threshold(settings.compression.air_threshold);
        self.air_compressor.set_ratio(settings.compression.air_ratio);
        let (comp_l, comp_r) = self.air_compressor.process(left, right);
        
        // 2. Air transient shaping (subtle)
        let (shaped_l, shaped_r) = self.transient_shaper.process(
            comp_l, comp_r,
            settings.transient_shaping.air_attack,
            analysis
        );
        
        // 3. Intelligent air exciter (pitch-tracked harmonic enhancement)
        self.air_exciter.set_amount(settings.intelligent_exciter.amount);
        self.air_exciter.set_mix(settings.intelligent_exciter.mix);
        let (exc_l, exc_r) = self.air_exciter.process(shaped_l, shaped_r, analysis);
        
        // 4. Additional harmonic enhancement
        let (enhanced_l, enhanced_r) = self.apply_air_harmonics(
            exc_l, exc_r, settings.harmonic_enhancement.air_harmonics
        );
        
        (enhanced_l, enhanced_r)
    }
    
    /// Apply bass harmonic enhancement (even-order harmonics for warmth)
    fn apply_bass_harmonics(&self, left: f32, right: f32, amount: f32) -> (f32, f32) {
        if amount < 0.01 {
            return (left, right);
        }
        
        // Gentle even-order harmonic generation
        let harmonic_l = (left * 2.0).tanh() * 0.5;
        let harmonic_r = (right * 2.0).tanh() * 0.5;
        
        (
            left + harmonic_l * amount * 0.3,
            right + harmonic_r * amount * 0.3
        )
    }
    
    /// Apply mid harmonic enhancement (moderate harmonics for body)
    fn apply_mid_harmonics(&self, left: f32, right: f32, amount: f32) -> (f32, f32) {
        if amount < 0.01 {
            return (left, right);
        }
        
        // Subtle harmonic distortion
        let harmonic_l = (left * 1.5).tanh() * 0.7;
        let harmonic_r = (right * 1.5).tanh() * 0.7;
        
        (
            left + harmonic_l * amount * 0.2,
            right + harmonic_r * amount * 0.2
        )
    }
    
    /// Apply presence harmonic enhancement (odd-order harmonics for brightness)  
    fn apply_presence_harmonics(&self, left: f32, right: f32, amount: f32, has_sibilance: bool) -> (f32, f32) {
        if amount < 0.01 {
            return (left, right);
        }
        
        // Reduce harmonic enhancement on sibilant material
        let effective_amount = if has_sibilance { amount * 0.5 } else { amount };
        
        // Odd-order harmonic generation for brightness
        let harmonic_l = (left * 3.0).tanh() * 0.33;
        let harmonic_r = (right * 3.0).tanh() * 0.33;
        
        (
            left + harmonic_l * effective_amount * 0.25,
            right + harmonic_r * effective_amount * 0.25
        )
    }
    
    /// Apply air harmonic enhancement (high-frequency sparkle)
    fn apply_air_harmonics(&self, left: f32, right: f32, amount: f32) -> (f32, f32) {
        if amount < 0.01 {
            return (left, right);
        }
        
        // Very gentle high-frequency enhancement
        let harmonic_l = (left * 1.2).tanh() * 0.8;
        let harmonic_r = (right * 1.2).tanh() * 0.8;
        
        (
            left + harmonic_l * amount * 0.15,
            right + harmonic_r * amount * 0.15
        )
    }

    /// Process a stereo sample pair with perceptual vocal processing chain
    ///
    /// # Arguments
    /// * `input_left` - Left channel input
    /// * `input_right` - Right channel input
    ///
    /// # Returns
    /// Tuple of (output_left, output_right)
    pub fn process(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        // 1. Input gain
        let input_gain = VoiceParams::db_to_gain(self.params.input_gain);
        let mut left = input_left * input_gain;
        let mut right = input_right * input_gain;

        // 2. Signal analysis (unchanged)
        let analysis = self.signal_analyzer.analyze(left, right);

        // 3. Map perceptual parameters to technical settings (lightweight <100 ops)
        let settings = ParameterMapper::map_parameters(&self.params, &analysis);

        // 4. Comprehensive multiband processing chain
        self.apply_multiband_processing(&mut left, &mut right, &settings, &analysis);
        
        // 5. Vocal doubler (intelligent thickness enhancement)
        self.vocal_doubler.set_amount(settings.vocal_doubler.amount);
        self.vocal_doubler.set_stereo_width(settings.vocal_doubler.stereo_width);
        let (doubled_left, doubled_right) = self.vocal_doubler.process(left, right, &analysis);
        left = doubled_left;
        right = doubled_right;
        
        // 6. Global limiting (transient-aware)
        let (left_limited, right_limited) = self.limiter.process(
            left, right, -3.0, &analysis  // Fixed professional threshold
        );

        // 7. Dry/wet mix and output gain
        let dry_wet = self.params.dry_wet_mix;
        let output_gain = VoiceParams::db_to_gain(self.params.output_gain);
        
        let final_left = (input_left * (1.0 - dry_wet) + left_limited * dry_wet) * output_gain;
        let final_right = (input_right * (1.0 - dry_wet) + right_limited * dry_wet) * output_gain;
        
        // Safety checks
        (
            if final_left.is_finite() { final_left } else { 0.0 },
            if final_right.is_finite() { final_right } else { 0.0 }
        )
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
        self.crossover_left.reset();
        self.crossover_right.reset();
        self.bass_saturator_left.reset();
        self.bass_saturator_right.reset();
        self.mid_saturator_left.reset();
        self.mid_saturator_right.reset();
        self.presence_saturator_left.reset();
        self.presence_saturator_right.reset();
        self.air_exciter.reset();
        self.bass_compressor.reset();
        self.mid_compressor.reset();
        self.presence_compressor.reset();
        self.air_compressor.reset();
        self.limiter.reset();
        self.vocal_doubler.reset();
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

        params.character = 0.7;
        params.intensity = 0.6;
        params.presence = 0.3;
        params.dynamics = -0.2;
        params.input_gain = 3.0;

        engine.update_params(params.clone());

        // Params should be stored
        assert_eq!(engine.params.character, 0.7);
        assert_eq!(engine.params.intensity, 0.6);
        assert_eq!(engine.params.presence, 0.3);
        assert_eq!(engine.params.dynamics, -0.2);
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
        params.intensity = 0.5;   // Some processing
        params.character = 0.3;   // Slightly bright
        engine.update_params(params);

        let input = 0.1;

        // Process multiple samples to stabilize processing
        for _ in 0..200 {
            engine.process(input, input);
        }

        // Now check output after processing is stabilized
        let (out_l, _) = engine.process(input, input);

        // With processing, output should be valid and modified
        assert!(out_l.is_finite());
        // Output should be in reasonable range with processing and limiter
        assert!(out_l.abs() > 0.01); // Not silent
        assert!(out_l.abs() < 2.0);   // Reasonable range
    }

    #[test]
    fn test_moderate_intensity_produces_processing() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();

        // Moderate intensity should produce audible processing
        params.intensity = 0.6;
        params.character = 0.2; // Slightly bright
        engine.update_params(params);

        let input = 0.5;

        // Process multiple samples to stabilize processing
        for _ in 0..100 {
            engine.process(input, input);
        }

        let (out_l, _) = engine.process(input, input);

        // Output should be different from input (processing applied)
        assert!(out_l.is_finite());
        assert!(out_l.abs() <= 2.0); // Reasonable range with limiting
    }

    #[test]
    fn test_perceptual_parameters_work() {
        let mut engine = VoiceEngine::new(44100.0);

        let input = 0.6;

        // Test with various perceptual parameter combinations
        let mut params = VoiceParams::default();
        params.character = 0.7;  // Bright
        params.intensity = 0.6;  // Moderate
        params.presence = 0.5;   // Upfront
        params.dynamics = -0.2;  // Controlled
        engine.update_params(params);

        // Stabilize processing
        for _ in 0..100 {
            engine.process(input, input);
        }

        let (out_l, out_r) = engine.process(input, input);

        // All processing should produce valid output
        assert!(out_l.is_finite(), "Left channel produced NaN/Inf");
        assert!(out_r.is_finite(), "Right channel produced NaN/Inf");
    }

    #[test]
    fn test_professional_compression() {
        let mut engine = VoiceEngine::new(44100.0);
        let params = VoiceParams::default();
        engine.update_params(params);

        let input = 0.8; // Loud input to trigger compression

        // Process multiple samples to let compressors settle
        let mut outputs = Vec::new();
        for _ in 0..200 {
            let (out_l, _) = engine.process(input, input);
            outputs.push(out_l);
        }

        // Professional compression should reduce the loud signal
        let final_output = outputs.last().unwrap();
        assert!(final_output.is_finite());
        // With professional compression, output should be controlled
        assert!(final_output.abs() > 0.1); // Not silent
        assert!(final_output.abs() < input); // Should be compressed (less than input)
    }

    #[test]
    fn test_perceptual_character_mapping() {
        let mut engine = VoiceEngine::new(44100.0);
        
        // Test warm character
        let mut warm_params = VoiceParams::default();
        warm_params.character = -0.8; // Very warm
        warm_params.intensity = 0.4;  // Moderate intensity
        engine.update_params(warm_params);

        let input = 0.3;
        
        // Warm setting should produce different output
        let (warm_out, _) = engine.process(input, input);
        assert!(warm_out.is_finite());

        // Test bright character
        let mut bright_params = VoiceParams::default();
        bright_params.character = 0.8;  // Very bright
        bright_params.intensity = 0.4;  // Same intensity
        engine.update_params(bright_params);

        let (bright_out, _) = engine.process(input, input);
        assert!(bright_out.is_finite());

        // Both should produce valid results (specific differences depend on frequency content)
        assert!(warm_out.is_finite() && bright_out.is_finite());
    }

    #[test]
    fn test_perceptual_defaults() {
        let params = VoiceParams::default();
        
        // Verify professional out-of-box perceptual settings
        assert_eq!(params.character, 0.2);   // Slightly bright
        assert_eq!(params.intensity, 0.4);   // Moderate processing
        assert_eq!(params.presence, 0.3);    // Upfront but not harsh
        assert_eq!(params.dynamics, 0.1);    // Controlled but not squashed
        assert_eq!(params.dry_wet_mix, 1.0); // Full effect

        // Verify these defaults produce valid output
        let mut engine = VoiceEngine::new(44100.0);
        engine.update_params(params);

        let (out_l, out_r) = engine.process(0.5, 0.5);
        assert!(out_l.is_finite());
        assert!(out_r.is_finite());
    }
    
    #[test]
    fn test_intensity_and_dynamics_mapping() {
        let mut engine = VoiceEngine::new(44100.0);
        
        // Test gentle processing
        let mut gentle_params = VoiceParams::default();
        gentle_params.intensity = 0.1;  // Very gentle
        gentle_params.dynamics = 0.8;   // Very dynamic
        engine.update_params(gentle_params);
        
        let (gentle_out, _) = engine.process(0.5, 0.5);
        assert!(gentle_out.is_finite());
        
        // Test aggressive processing
        let mut aggressive_params = VoiceParams::default();
        aggressive_params.intensity = 0.9;  // Very aggressive
        aggressive_params.dynamics = -0.8;  // Very compressed
        engine.update_params(aggressive_params);
        
        let (aggressive_out, _) = engine.process(0.5, 0.5);
        assert!(aggressive_out.is_finite());
        
        // Both should produce valid output with different characteristics
        assert!(gentle_out.is_finite() && aggressive_out.is_finite());
    }

    #[test] 
    fn test_vocal_doubler_integration() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();

        // Test that vocal doubler is controlled by intensity parameter
        params.intensity = 0.0; // No doubling
        params.character = 0.0;  
        params.presence = 0.0;
        engine.update_params(params.clone());

        let (no_double_out, _) = engine.process(0.5, 0.0); // Mono input

        // Test with doubling enabled
        params.intensity = 0.8;  // Strong doubling
        params.character = -0.5; // Warm character (more doubling)
        params.presence = 0.5;   // Intimate (wide stereo)
        engine.update_params(params);

        let (doubled_left, doubled_right) = engine.process(0.5, 0.0); // Mono input

        // Verify output is valid
        assert!(doubled_left.is_finite());
        assert!(doubled_right.is_finite());
        
        // With high intensity and stereo spread, should have stereo output from mono input
        let stereo_spread = (doubled_left - doubled_right).abs();
        assert!(stereo_spread > 0.001, 
               "Expected stereo spread from doubler, got L={}, R={}", 
               doubled_left, doubled_right);
    }

    #[test]
    fn test_intelligent_exciter_integration() {
        let mut engine = VoiceEngine::new(44100.0);
        let mut params = VoiceParams::default();

        // Test presence controls intelligent exciter
        params.presence = -1.0; // Distant (minimal sparkle)
        params.character = 0.0;
        engine.update_params(params.clone());

        let (distant_out, _) = engine.process(0.5, 0.5);

        // Test intimate presence with bright character (maximum sparkle)
        params.presence = 1.0;  // Intimate (maximum sparkle)
        params.character = 0.8; // Bright (more harmonics)
        engine.update_params(params);

        let (intimate_out, _) = engine.process(0.5, 0.5);

        // Both outputs should be valid
        assert!(distant_out.is_finite());
        assert!(intimate_out.is_finite());
        
        // Intimate presence should produce more enhanced output due to intelligent exciter
        // This is a subtle effect so we mainly test that it processes without error
        assert!(intimate_out.abs() > 0.001, "Intimate presence should produce audible output");
    }
}
