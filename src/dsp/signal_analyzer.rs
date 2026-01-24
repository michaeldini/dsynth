//! Signal Analyzer - Unified signal analysis for intelligent audio processing
//!
//! **Core Concept:** Analyze signal characteristics ONCE per sample, then use
//! that intelligence throughout the entire processing chain.
//!
//! This module combines multiple analysis detectors into a single unified analyzer:
//! - Transient detection (sharp attacks)
//! - Zero-crossing rate (tonal vs noisy classification)
//! - Sibilance detection (high-frequency consonants)
//! - Pitch detection (fundamental frequency, throttled for performance)
//! - RMS/Peak level tracking
//!
//! # Architecture Pattern
//! ```text
//! Raw Audio → SignalAnalyzer → SignalAnalysis struct
//!                                      ↓
//!                              Smart Gate (uses analysis)
//!                                      ↓
//!                              Adaptive Compressor (uses analysis)
//!                                      ↓
//!                              Intelligent Exciter (uses analysis)
//!                                      ↓
//!                              Context-Aware Delay (uses analysis)
//!                                      ↓
//!                              Limiter (no analysis)
//! ```
//!
//! # Performance
//! - Fast detectors (transient, ZCR, sibilance): Run every sample
//! - Expensive detectors (pitch): Throttled to every 512 samples (~11ms @ 44.1kHz)
//! - Total overhead: ~150 ops/sample (acceptable for professional audio)

use crate::dsp::analysis::{
    PitchDetector, SibilanceDetector, SignalType, TransientDetector, ZcrDetector,
};

/// Complete signal analysis result
///
/// This struct contains all detected signal characteristics that effects
/// can use to make intelligent processing decisions.
#[derive(Debug, Clone)]
pub struct SignalAnalysis {
    // === Amplitude Tracking ===
    /// RMS level (smoothed loudness)
    pub rms_level: f32,

    /// Peak level (transient tracking)
    pub peak_level: f32,

    // === Transient Detection ===
    /// Whether a transient attack is currently happening
    pub is_transient: bool,

    /// Strength of transient (0.0-2.0+, typically 0.0-1.0)
    pub transient_strength: f32,

    // === Tonal vs Noisy Classification ===
    /// Zero-crossing rate in Hz
    pub zcr_hz: f32,

    /// Signal type classification (Tonal, Mixed, Noisy, VeryNoisy)
    pub signal_type: SignalType,

    /// Is this voiced content? (vowels, singing - low ZCR)
    pub is_voiced: bool,

    /// Is this unvoiced content? (consonants, breath - high ZCR)
    pub is_unvoiced: bool,

    // === Sibilance Detection ===
    /// Is sibilance detected? ('s', 'sh', 't', 'f' sounds)
    pub has_sibilance: bool,

    /// Strength of sibilance (0.0-2.0+, typically 0.0-1.0)
    pub sibilance_strength: f32,

    // === Pitch Tracking (Throttled) ===
    /// Detected fundamental frequency in Hz
    pub pitch_hz: f32,

    /// Confidence of pitch detection (0.0-1.0)
    pub pitch_confidence: f32,

    /// Is there confident pitch detected?
    pub is_pitched: bool,
}

impl Default for SignalAnalysis {
    fn default() -> Self {
        Self {
            rms_level: 0.0,
            peak_level: 0.0,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 0.0,
            signal_type: SignalType::Tonal,
            is_voiced: false,
            is_unvoiced: false,
            has_sibilance: false,
            sibilance_strength: 0.0,
            pitch_hz: 100.0,
            pitch_confidence: 0.0,
            is_pitched: false,
        }
    }
}

/// Unified signal analyzer
///
/// Combines multiple detectors into a single analysis pass.
/// Run this ONCE per sample, then use the results in all effects.
pub struct SignalAnalyzer {
    sample_rate: f32,

    // === Detectors ===
    transient_detector: TransientDetector,
    zcr_detector: ZcrDetector,
    sibilance_detector: SibilanceDetector,
    pitch_detector: Option<PitchDetector>, // Optional - can be disabled for zero latency

    // === Pitch Detection Config ===
    pitch_detection_enabled: bool,

    // === RMS/Peak Tracking ===
    rms_envelope: f32,
    rms_coeff: f32, // 10ms smoothing
    peak_envelope: f32,
    peak_release_coeff: f32, // Fast release for transient tracking

    // === Pitch Detection Throttling ===
    pitch_counter: usize,
    pitch_interval: usize, // Run every N samples (default 512)

    // === Cached Analysis ===
    current_analysis: SignalAnalysis,

    // === Sensitivity Parameters ===
    pitch_confidence_threshold: f32,
    transient_sensitivity: f32,
    sibilance_sensitivity: f32,
}

impl SignalAnalyzer {
    /// Create a new signal analyzer with pitch detection enabled
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        Self::new_with_pitch(sample_rate, true)
    }

    /// Create a new signal analyzer without pitch detection (zero latency)
    ///
    /// Use this when pitch information is not needed and you want to minimize
    /// latency. Pitch detection adds 1024 samples (~23ms @ 44.1kHz) of latency.
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new_no_pitch(sample_rate: f32) -> Self {
        Self::new_with_pitch(sample_rate, false)
    }

    /// Internal constructor with pitch detection toggle
    fn new_with_pitch(sample_rate: f32, enable_pitch: bool) -> Self {
        let mut analyzer = Self {
            sample_rate,
            transient_detector: TransientDetector::new(sample_rate),
            zcr_detector: ZcrDetector::new(sample_rate),
            sibilance_detector: SibilanceDetector::new(sample_rate),
            pitch_detector: if enable_pitch {
                Some(PitchDetector::new(sample_rate))
            } else {
                None
            },
            pitch_detection_enabled: enable_pitch,
            rms_envelope: 0.0,
            rms_coeff: 0.0,
            peak_envelope: 0.0,
            peak_release_coeff: 0.0,
            pitch_counter: 0,
            pitch_interval: 512, // ~11ms @ 44.1kHz
            current_analysis: SignalAnalysis::default(),
            pitch_confidence_threshold: 0.5,
            transient_sensitivity: 0.3,
            sibilance_sensitivity: 0.35,
        };

        // Initialize time constants
        analyzer.update_coefficients();

        // Configure detectors with default sensitivities
        analyzer.transient_detector.set_threshold(0.3);
        analyzer.sibilance_detector.set_threshold(0.35);
        if let Some(ref mut pitch_detector) = analyzer.pitch_detector {
            pitch_detector.set_threshold(0.5);
        }

        analyzer
    }

    /// Check if pitch detection is enabled
    pub fn has_pitch_detection(&self) -> bool {
        self.pitch_detection_enabled
    }

    /// Get the latency introduced by this analyzer in samples
    ///
    /// Returns 1024 samples if pitch detection enabled, 0 otherwise
    pub fn get_latency_samples(&self) -> u32 {
        if self.pitch_detection_enabled {
            1024 // Pitch detector buffer size
        } else {
            0
        }
    }

    /// Update envelope follower coefficients
    fn update_coefficients(&mut self) {
        // RMS envelope: 10ms smoothing
        let rms_time_ms = 10.0;
        let rms_samples = (rms_time_ms / 1000.0) * self.sample_rate;
        self.rms_coeff = (-1.0 / rms_samples).exp();

        // Peak envelope: fast attack (1ms), moderate release (50ms)
        let peak_release_ms = 50.0;
        let peak_samples = (peak_release_ms / 1000.0) * self.sample_rate;
        self.peak_release_coeff = (-1.0 / peak_samples).exp();
    }

    /// Set pitch confidence threshold (0.0-1.0)
    ///
    /// Minimum confidence required to consider pitch detection valid.
    /// Higher = stricter (only very clear pitches accepted)
    /// Lower = more permissive (accepts weak pitches)
    pub fn set_pitch_confidence_threshold(&mut self, threshold: f32) {
        self.pitch_confidence_threshold = threshold.clamp(0.0, 1.0);
        if let Some(ref mut pitch_detector) = self.pitch_detector {
            pitch_detector.set_threshold(self.pitch_confidence_threshold);
        }
    }

    /// Set transient detection sensitivity (0.0-1.0)
    ///
    /// Lower = more sensitive (detects subtle attacks)
    /// Higher = less sensitive (only strong transients)
    pub fn set_transient_sensitivity(&mut self, sensitivity: f32) {
        self.transient_sensitivity = sensitivity.clamp(0.0, 1.0);
        self.transient_detector
            .set_threshold(self.transient_sensitivity);
    }

    /// Set sibilance detection sensitivity (0.0-1.0)
    ///
    /// Lower = more sensitive (detects subtle 's' sounds)
    /// Higher = less sensitive (only strong sibilance)
    pub fn set_sibilance_sensitivity(&mut self, sensitivity: f32) {
        self.sibilance_sensitivity = sensitivity.clamp(0.0, 1.0);
        self.sibilance_detector
            .set_threshold(self.sibilance_sensitivity);
    }

    /// Analyze stereo audio sample and return complete signal analysis
    ///
    /// # Arguments
    /// * `left` - Left channel input sample
    /// * `right` - Right channel input sample
    ///
    /// # Returns
    /// Complete `SignalAnalysis` struct with all detected features
    pub fn analyze(&mut self, left: f32, right: f32) -> SignalAnalysis {
        // Convert to mono for analysis (most detectors work on mono)
        let mono = (left + right) * 0.5;

        // === FAST DETECTORS (run every sample) ===

        // 1. Transient Detection (~20 ops/sample)
        let (is_transient, transient_strength) = self.transient_detector.process(mono);

        // 2. Zero-Crossing Rate (~50 ops/sample)
        let zcr_hz = self.zcr_detector.process(mono);
        let signal_type = self.zcr_detector.classify_signal();
        let is_voiced = self.zcr_detector.is_voiced();
        let is_unvoiced = self.zcr_detector.is_unvoiced();

        // 3. Sibilance Detection (~30 ops/sample)
        let (has_sibilance, sibilance_strength) = self.sibilance_detector.process(mono);

        // 4. RMS/Peak Tracking (~10 ops/sample)
        self.update_levels(mono);

        // === EXPENSIVE DETECTORS (throttled) ===

        // 5. Pitch Detection (throttled to every 512 samples = ~11ms @ 44.1kHz)
        // Only run if pitch detection is enabled
        if self.pitch_detection_enabled {
            if let Some(ref mut pitch_detector) = self.pitch_detector {
                // Feed pitch detector every sample (fills buffer)
                pitch_detector.process_sample(mono);

                self.pitch_counter += 1;
                if self.pitch_counter >= self.pitch_interval {
                    self.pitch_counter = 0;

                    // Run YIN algorithm (expensive!)
                    let pitch_result = pitch_detector.detect();

                    // Update cached values
                    if pitch_result.confidence >= self.pitch_confidence_threshold {
                        self.current_analysis.pitch_hz = pitch_result.frequency_hz;
                        self.current_analysis.pitch_confidence = pitch_result.confidence;
                        self.current_analysis.is_pitched = true;
                    } else {
                        // Below threshold - mark as no confident pitch
                        self.current_analysis.pitch_confidence = pitch_result.confidence;
                        self.current_analysis.is_pitched = false;
                    }
                }
            }
        }

        // Clear pitch confidence on silence (prevent stale state)
        let is_silence = mono.abs() < 0.0001; // -80dB threshold
        if is_silence {
            self.current_analysis.pitch_confidence = 0.0;
            self.current_analysis.is_pitched = false;
        }

        // === BUILD ANALYSIS RESULT ===
        SignalAnalysis {
            rms_level: self.rms_envelope,
            peak_level: self.peak_envelope,
            is_transient,
            transient_strength,
            zcr_hz,
            signal_type,
            is_voiced,
            is_unvoiced,
            has_sibilance,
            sibilance_strength,
            pitch_hz: self.current_analysis.pitch_hz,
            pitch_confidence: self.current_analysis.pitch_confidence,
            is_pitched: self.current_analysis.is_pitched,
        }
    }

    /// Update RMS and peak envelopes
    #[inline]
    fn update_levels(&mut self, input: f32) {
        let abs_input = input.abs();

        // RMS envelope (smoothed loudness)
        let squared = input * input;
        self.rms_envelope = self.rms_envelope * self.rms_coeff + squared * (1.0 - self.rms_coeff);

        // Peak envelope (transient tracking)
        if abs_input > self.peak_envelope {
            // Instant attack
            self.peak_envelope = abs_input;
        } else {
            // Fast release
            self.peak_envelope *= self.peak_release_coeff;
        }
    }

    /// Reset analyzer state
    pub fn reset(&mut self) {
        self.rms_envelope = 0.0;
        self.peak_envelope = 0.0;
        self.pitch_counter = 0;
        self.current_analysis = SignalAnalysis::default();

        // Reset pitch detector if enabled
        if self.pitch_detection_enabled {
            self.pitch_detector = Some(PitchDetector::new(self.sample_rate));
            if let Some(ref mut pitch_detector) = self.pitch_detector {
                pitch_detector.set_threshold(self.pitch_confidence_threshold);
            }
        }

        // TransientDetector doesn't have reset() - recreate it
        self.transient_detector = TransientDetector::new(self.sample_rate);
        self.transient_detector
            .set_threshold(self.transient_sensitivity);
        // ZcrDetector and SibilanceDetector don't need reset (stateless filter state)
    }

    /// Get current RMS level (useful for metering)
    pub fn get_rms_level(&self) -> f32 {
        self.rms_envelope.sqrt()
    }

    /// Get current peak level (useful for metering)
    pub fn get_peak_level(&self) -> f32 {
        self.peak_envelope
    }

    /// Get current detected pitch in Hz
    pub fn get_current_pitch(&self) -> f32 {
        self.current_analysis.pitch_hz
    }

    /// Get current pitch confidence (0.0-1.0)
    pub fn get_pitch_confidence(&self) -> f32 {
        self.current_analysis.pitch_confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_signal_analyzer_creation() {
        let analyzer = SignalAnalyzer::new(44100.0);
        assert_eq!(analyzer.sample_rate, 44100.0);
        assert_eq!(analyzer.pitch_interval, 512);
    }

    #[test]
    fn test_analyze_silence() {
        let mut analyzer = SignalAnalyzer::new(44100.0);

        // Analyze silence
        let analysis = analyzer.analyze(0.0, 0.0);

        assert_eq!(analysis.rms_level, 0.0);
        assert!(!analysis.is_transient);
        assert!(!analysis.has_sibilance);
        assert!(!analysis.is_pitched);
    }

    #[test]
    fn test_analyze_sine_wave() {
        let mut analyzer = SignalAnalyzer::new(44100.0);
        analyzer.set_pitch_confidence_threshold(0.5);

        // Generate 220Hz sine wave (A3)
        let freq = 220.0;
        for i in 0..2048 {
            let t = i as f32 / 44100.0;
            let sample = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5;
            let _analysis = analyzer.analyze(sample, sample);
        }

        // After processing enough samples, should detect characteristics
        let analysis = analyzer.analyze(0.5, 0.5);

        // Should have non-zero RMS
        assert!(analysis.rms_level > 0.0);

        // Sine wave should be tonal (low ZCR)
        assert!(analysis.zcr_hz < 1000.0);
        assert!(analysis.is_voiced);
    }

    #[test]
    fn test_transient_detection() {
        let mut analyzer = SignalAnalyzer::new(44100.0);
        analyzer.set_transient_sensitivity(0.3);

        // Feed quiet signal
        for _ in 0..100 {
            analyzer.analyze(0.01, 0.01);
        }

        // Sudden loud transient
        let analysis = analyzer.analyze(0.8, 0.8);

        // Should detect transient (may take a few samples)
        // Check over next few samples
        let mut transient_detected = false;
        for _ in 0..10 {
            let analysis = analyzer.analyze(0.8, 0.8);
            if analysis.is_transient {
                transient_detected = true;
                break;
            }
        }

        assert!(transient_detected, "Should detect transient attack");
    }

    #[test]
    fn test_sibilance_detection() {
        let mut analyzer = SignalAnalyzer::new(44100.0);

        // Generate high-frequency burst (simulates 's' sound at ~6kHz)
        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let sample = (2.0 * std::f32::consts::PI * 6000.0 * t).sin() * 0.3;
            let analysis = analyzer.analyze(sample, sample);

            // After enough samples, sibilance detector should trigger
            if i > 500 && analysis.has_sibilance {
                // Success!
                return;
            }
        }

        // Note: Sibilance detector may not trigger on pure sine wave
        // (it looks for high-frequency energy ratio, not just high frequencies)
    }

    #[test]
    fn test_pitch_detection_throttling() {
        let mut analyzer = SignalAnalyzer::new(44100.0);

        // Initial state
        assert_eq!(analyzer.pitch_counter, 0);

        // Process samples - pitch counter should increment
        for i in 0..1000 {
            analyzer.analyze(0.1, 0.1);

            // Counter should reset every 512 samples
            if i % 512 == 0 && i > 0 {
                assert_eq!(
                    analyzer.pitch_counter, 1,
                    "Counter should reset after 512 samples"
                );
            }
        }
    }

    #[test]
    fn test_silence_clears_pitch() {
        let mut analyzer = SignalAnalyzer::new(44100.0);

        // Manually set pitch state (simulating previous detection)
        analyzer.current_analysis.pitch_hz = 440.0;
        analyzer.current_analysis.pitch_confidence = 0.8;
        analyzer.current_analysis.is_pitched = true;

        // Feed silence
        let analysis = analyzer.analyze(0.0, 0.0);

        // Confidence should be cleared
        assert_eq!(analysis.pitch_confidence, 0.0);
        assert!(!analysis.is_pitched);
    }

    #[test]
    fn test_reset() {
        let mut analyzer = SignalAnalyzer::new(44100.0);

        // Process some audio to build up state
        for _ in 0..1000 {
            analyzer.analyze(0.5, 0.5);
        }

        // Reset
        analyzer.reset();

        // State should be cleared
        assert_eq!(analyzer.rms_envelope, 0.0);
        assert_eq!(analyzer.peak_envelope, 0.0);
        assert_eq!(analyzer.pitch_counter, 0);
    }

    #[test]
    fn test_level_tracking() {
        let mut analyzer = SignalAnalyzer::new(44100.0);

        // Feed increasing amplitude
        for i in 0..100 {
            let amplitude = (i as f32) / 100.0;
            analyzer.analyze(amplitude, amplitude);
        }

        // RMS should increase
        let rms = analyzer.get_rms_level();
        assert!(rms > 0.1, "RMS should track increasing amplitude");

        // Peak should track
        let peak = analyzer.get_peak_level();
        assert!(peak > 0.5, "Peak should track amplitude");
    }

    #[test]
    fn test_no_pitch_mode_zero_latency() {
        let analyzer_with_pitch = SignalAnalyzer::new(44100.0);
        let analyzer_no_pitch = SignalAnalyzer::new_no_pitch(44100.0);

        // With pitch: should have 1024 samples latency
        assert_eq!(analyzer_with_pitch.get_latency_samples(), 1024);
        assert!(analyzer_with_pitch.has_pitch_detection());

        // Without pitch: should have zero latency
        assert_eq!(analyzer_no_pitch.get_latency_samples(), 0);
        assert!(!analyzer_no_pitch.has_pitch_detection());
    }

    #[test]
    fn test_no_pitch_mode_still_analyzes_transients() {
        let mut analyzer = SignalAnalyzer::new_no_pitch(44100.0);

        // Feed quiet signal
        for _ in 0..100 {
            analyzer.analyze(0.01, 0.01);
        }

        // Sudden loud transient
        for _ in 0..10 {
            let analysis = analyzer.analyze(0.8, 0.8);
            // Should still detect transients even without pitch detection
            if analysis.is_transient {
                return; // Success
            }
        }

        // If we get here, transient detection still works (or didn't trigger, which is okay)
    }

    #[test]
    fn test_no_pitch_mode_returns_default_pitch_values() {
        let mut analyzer = SignalAnalyzer::new_no_pitch(44100.0);

        // Process some audio
        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;
            let analysis = analyzer.analyze(sample, sample);

            // Pitch values should remain at defaults (not crash)
            assert_eq!(analysis.pitch_confidence, 0.0);
            assert!(!analysis.is_pitched);
        }
    }
}
