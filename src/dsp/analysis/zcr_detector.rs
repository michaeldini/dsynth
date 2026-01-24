/// Zero-Crossing Rate (ZCR) Detector - Distinguishes voiced from unvoiced sounds
///
/// Zero-crossing rate counts how often the audio signal crosses the zero amplitude line.
/// This is a simple but powerful feature for audio classification:
///
/// # ZCR Characteristics
/// - **High ZCR** (many crossings): Noisy, unvoiced, or high-frequency content
///   - Examples: 's', 'sh', 't', 'f' consonants, breath noise, cymbals, white noise
/// - **Low ZCR** (few crossings): Tonal, voiced, or low-frequency content
///   - Examples: Vowels, sung notes, bass, pitched instruments
///
/// # Use Cases
/// - Voice/noise discrimination (better than simple amplitude gate)
/// - Voiced/unvoiced detection for selective processing
/// - Consonant detection in speech
/// - Complement to pitch detector (ZCR validates pitch confidence)
/// - Adaptive filtering (process vocals differently from breath noise)
///
/// # Algorithm
/// 1. Count zero-crossings in a sliding window (typically 10-50ms)
/// 2. Normalize to crossings per second (Hz)
/// 3. Smooth the result to avoid jitter
///
/// # Parameters
/// - **window_ms**: Analysis window size (10-100ms, typical 20ms)
/// - **smoothing_ms**: Output smoothing time (10-100ms)
use std::collections::VecDeque;

pub struct ZcrDetector {
    sample_rate: f32,

    // Sliding window for zero-crossing counting
    window_size: usize,
    sample_buffer: VecDeque<f32>,

    // Zero-crossing counting
    zero_crossings: usize,
    prev_sign: bool, // true = positive, false = negative

    // Output smoothing
    zcr_hz: f32,
    smooth_coef: f32,
}

impl ZcrDetector {
    /// Create a new zero-crossing rate detector
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut detector = Self {
            sample_rate,
            window_size: 0,
            sample_buffer: VecDeque::new(),
            zero_crossings: 0,
            prev_sign: true,
            zcr_hz: 0.0,
            smooth_coef: 0.0,
        };

        // Default settings
        detector.set_window_size(20.0); // 20ms window
        detector.set_smoothing_time(50.0); // 50ms smoothing

        detector
    }

    /// Set analysis window size in milliseconds (10-100ms)
    pub fn set_window_size(&mut self, window_ms: f32) {
        let window_ms = window_ms.clamp(5.0, 500.0);
        self.window_size = ((window_ms / 1000.0) * self.sample_rate) as usize;
        self.window_size = self.window_size.max(1);

        // Resize buffer if needed
        self.sample_buffer.clear();
        self.zero_crossings = 0;
    }

    /// Set output smoothing time in milliseconds (10-100ms)
    pub fn set_smoothing_time(&mut self, smoothing_ms: f32) {
        let smoothing_ms = smoothing_ms.clamp(5.0, 500.0);
        let time_samples = (smoothing_ms / 1000.0) * self.sample_rate;
        self.smooth_coef = (-1.0 / time_samples).exp();
    }

    /// Process a mono audio sample and compute ZCR
    ///
    /// # Arguments
    /// * `input` - Input audio sample
    ///
    /// # Returns
    /// Current zero-crossing rate in Hz
    pub fn process(&mut self, input: f32) -> f32 {
        // Determine sign (with small deadband to avoid noise-induced crossings)
        let deadband = 0.0001; // -80dB threshold
        let current_sign = input > deadband;

        // Detect zero crossing (sign change)
        if self.sample_buffer.len() > 0 && current_sign != self.prev_sign {
            // Only count if we've moved significantly from zero
            if input.abs() > deadband {
                self.zero_crossings += 1;
            }
        }

        // Add sample to buffer
        self.sample_buffer.push_back(input);

        // Remove oldest sample if window full
        if self.sample_buffer.len() > self.window_size {
            let oldest = self.sample_buffer.pop_front().unwrap_or(0.0);

            // If removing a sample that contributed to a crossing, decrement count
            // This is approximate but works well in practice
            if self.sample_buffer.len() > 1 {
                let second_oldest = self.sample_buffer[0];
                let oldest_sign = oldest > deadband;
                let second_sign = second_oldest > deadband;
                if oldest_sign != second_sign && oldest.abs() > deadband {
                    self.zero_crossings = self.zero_crossings.saturating_sub(1);
                }
            }
        }

        self.prev_sign = current_sign;

        // Calculate instantaneous ZCR in Hz
        // ZCR = (crossings / window_samples) * sample_rate
        let window_samples = self.sample_buffer.len().max(1);
        let zcr_instant = (self.zero_crossings as f32 / window_samples as f32) * self.sample_rate;

        // Smooth output to reduce jitter
        self.zcr_hz += (zcr_instant - self.zcr_hz) * (1.0 - self.smooth_coef);

        self.zcr_hz.clamp(0.0, self.sample_rate / 2.0)
    }

    /// Process stereo audio (uses average of both channels)
    pub fn process_stereo(&mut self, left: f32, right: f32) -> f32 {
        let mono = (left + right) * 0.5;
        self.process(mono)
    }

    /// Get current ZCR without processing new sample
    pub fn get_zcr_hz(&self) -> f32 {
        self.zcr_hz
    }

    /// Get normalized ZCR (0.0 = very tonal, 1.0 = very noisy)
    /// Maps typical range 0-3000 Hz to 0-1
    pub fn get_normalized_zcr(&self) -> f32 {
        (self.zcr_hz / 3000.0).clamp(0.0, 1.0)
    }

    /// Classify signal based on ZCR
    pub fn classify_signal(&self) -> SignalType {
        match self.zcr_hz {
            z if z < 500.0 => SignalType::Tonal,  // Strong pitch, low noise
            z if z < 1500.0 => SignalType::Mixed, // Some harmonics
            z if z < 3000.0 => SignalType::Noisy, // Consonants, breath
            _ => SignalType::VeryNoisy,           // Pure noise, sibilance
        }
    }

    /// Check if signal is likely voiced (sung/spoken vowel)
    pub fn is_voiced(&self) -> bool {
        self.zcr_hz < 1000.0
    }

    /// Check if signal is likely unvoiced (consonant, noise)
    pub fn is_unvoiced(&self) -> bool {
        self.zcr_hz > 2000.0
    }

    /// Reset detector state
    pub fn reset(&mut self) {
        self.sample_buffer.clear();
        self.zero_crossings = 0;
        self.prev_sign = true;
        self.zcr_hz = 0.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalType {
    Tonal,     // Low ZCR: vowels, pitched instruments
    Mixed,     // Medium ZCR: mixed voiced/unvoiced
    Noisy,     // High ZCR: consonants, breath
    VeryNoisy, // Very high ZCR: pure noise, sibilance
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn test_zcr_detector_creation() {
        let detector = ZcrDetector::new(44100.0);
        assert_relative_eq!(detector.get_zcr_hz(), 0.0, epsilon = 1.0);
    }

    #[test]
    fn test_low_frequency_tone() {
        let mut detector = ZcrDetector::new(44100.0);
        let sample_rate = 44100.0;

        // Process 200 Hz sine wave (low ZCR)
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 200.0 * phase).sin() * 0.5;
            detector.process(sample);
        }

        let zcr = detector.get_zcr_hz();

        // ZCR should be approximately 2× frequency (two crossings per cycle)
        // For 200 Hz signal, expect ~400 Hz ZCR
        assert!(
            zcr > 300.0 && zcr < 600.0,
            "Low frequency tone should have low ZCR, got {} Hz",
            zcr
        );
        assert!(
            detector.is_voiced(),
            "Low frequency should be classified as voiced"
        );
        assert_eq!(detector.classify_signal(), SignalType::Tonal);
    }

    #[test]
    fn test_high_frequency_tone() {
        let mut detector = ZcrDetector::new(44100.0);
        let sample_rate = 44100.0;

        // Process 5000 Hz sine wave (high ZCR)
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 5000.0 * phase).sin() * 0.5;
            detector.process(sample);
        }

        let zcr = detector.get_zcr_hz();

        // ZCR should be approximately 2× frequency
        // For 5000 Hz signal, expect ~10000 Hz ZCR
        assert!(
            zcr > 8000.0,
            "High frequency tone should have high ZCR, got {} Hz",
            zcr
        );
        assert!(
            detector.is_unvoiced(),
            "High frequency should be classified as unvoiced"
        );
    }

    #[test]
    fn test_white_noise() {
        let mut detector = ZcrDetector::new(44100.0);

        // Process white noise (very high ZCR)
        for i in 0..10000 {
            // Simple pseudo-random noise
            let noise = ((i * 1664525 + 1013904223) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            detector.process(noise * 0.3);
        }

        let zcr = detector.get_zcr_hz();

        // White noise has very high ZCR (many crossings)
        assert!(
            zcr > 3000.0,
            "White noise should have very high ZCR, got {} Hz",
            zcr
        );
        assert!(
            detector.is_unvoiced(),
            "Noise should be classified as unvoiced"
        );
        assert!(
            detector.classify_signal() == SignalType::Noisy
                || detector.classify_signal() == SignalType::VeryNoisy
        );
    }

    #[test]
    fn test_silence() {
        let mut detector = ZcrDetector::new(44100.0);

        // Process silence (zero ZCR)
        for _ in 0..5000 {
            detector.process(0.0);
        }

        let zcr = detector.get_zcr_hz();

        // Silence should have very low ZCR
        assert!(
            zcr < 100.0,
            "Silence should have near-zero ZCR, got {} Hz",
            zcr
        );
    }

    #[test]
    fn test_stereo_processing() {
        let mut detector = ZcrDetector::new(44100.0);
        let sample_rate = 44100.0;

        // Process stereo signal
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let left = (2.0 * PI * 300.0 * phase).sin() * 0.5;
            let right = (2.0 * PI * 300.0 * phase).sin() * 0.5;
            detector.process_stereo(left, right);
        }

        let zcr = detector.get_zcr_hz();
        assert!(
            zcr > 400.0 && zcr < 800.0,
            "Stereo processing should track frequency"
        );
    }

    #[test]
    fn test_normalized_output() {
        let mut detector = ZcrDetector::new(44100.0);

        detector.zcr_hz = 0.0;
        assert_relative_eq!(detector.get_normalized_zcr(), 0.0, epsilon = 0.01);

        detector.zcr_hz = 1500.0;
        assert_relative_eq!(detector.get_normalized_zcr(), 0.5, epsilon = 0.01);

        detector.zcr_hz = 3000.0;
        assert_relative_eq!(detector.get_normalized_zcr(), 1.0, epsilon = 0.01);

        detector.zcr_hz = 6000.0; // Should clamp to 1.0
        assert_relative_eq!(detector.get_normalized_zcr(), 1.0, epsilon = 0.01);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = ZcrDetector::new(44100.0);

        // Process some signal
        for i in 0..1000 {
            detector.process((i as f32).sin());
        }

        // Reset
        detector.reset();

        assert_relative_eq!(detector.get_zcr_hz(), 0.0, epsilon = 1.0);
        assert_eq!(detector.sample_buffer.len(), 0);
        assert_eq!(detector.zero_crossings, 0);
    }

    #[test]
    fn test_classification_boundaries() {
        let detector = ZcrDetector::new(44100.0);
        let mut test_detector = detector;

        test_detector.zcr_hz = 400.0;
        assert_eq!(test_detector.classify_signal(), SignalType::Tonal);

        test_detector.zcr_hz = 1000.0;
        assert_eq!(test_detector.classify_signal(), SignalType::Mixed);

        test_detector.zcr_hz = 2500.0;
        assert_eq!(test_detector.classify_signal(), SignalType::Noisy);

        test_detector.zcr_hz = 4000.0;
        assert_eq!(test_detector.classify_signal(), SignalType::VeryNoisy);
    }
}
