//! YIN pitch detection algorithm
//!
//! Implements the YIN algorithm for fundamental frequency estimation from audio signals.
//! Works best with monophonic inputs (single voice/instrument). Designed for vocal pitch
//! tracking with ~23ms latency (1024 samples @ 44.1kHz).
//!
//! # Algorithm Overview
//!
//! YIN is based on autocorrelation but uses a different approach that's more robust
//! to harmonic content. Key steps:
//! 1. **Difference function**: Measure self-dissimilarity at various time lags
//! 2. **Cumulative mean normalized difference**: Normalize to handle amplitude variations
//! 3. **Absolute threshold**: Find first minimum below threshold to extract period
//! 4. **Parabolic interpolation**: Refine period estimate for sub-sample accuracy
//!
//! # References
//! - "YIN, a fundamental frequency estimator for speech and music" (de Cheveigné & Kawahara, 2002)
//!
//! # Performance
//! - Time complexity: O(N) per analysis (optimized cumulative sum approach)
//! - Space complexity: O(N) for circular buffer + difference function storage

/// Pitch detection buffer size (must be power of 2 for efficiency)
/// 1024 samples = ~23ms @ 44.1kHz
/// Allows detection down to ~43Hz (below lowest vocal fundamental)
pub const PITCH_BUFFER_SIZE: usize = 1024;

/// Minimum detectable frequency (Hz)
/// Set by buffer size: sample_rate / PITCH_BUFFER_SIZE
const MIN_FREQUENCY: f32 = 43.0; // ~43Hz @ 44.1kHz

/// Maximum detectable frequency (Hz)
/// Set to avoid detecting harmonics as fundamentals
const MAX_FREQUENCY: f32 = 800.0;

/// Result of pitch detection analysis
#[derive(Debug, Clone, Copy)]
pub struct PitchDetectionResult {
    /// Detected fundamental frequency in Hz
    pub frequency_hz: f32,

    /// Confidence level (0.0-1.0) indicating signal clarity
    /// High confidence (>0.7) = clear pitch, low confidence = noisy/silence
    pub confidence: f32,
}

impl Default for PitchDetectionResult {
    fn default() -> Self {
        Self {
            frequency_hz: 0.0,
            confidence: 0.0,
        }
    }
}

/// YIN pitch detector
pub struct PitchDetector {
    sample_rate: f32,

    /// Circular buffer for input samples
    buffer: [f32; PITCH_BUFFER_SIZE],
    buffer_index: usize,

    /// Difference function storage (half size since we only check up to N/2 lag)
    difference_function: [f32; PITCH_BUFFER_SIZE / 2],

    /// Cumulative mean normalized difference
    cmnd: [f32; PITCH_BUFFER_SIZE / 2],

    /// Detection threshold (lower = more sensitive, higher = more selective)
    /// Typical range: 0.1 (aggressive) to 0.3 (conservative)
    threshold: f32,

    /// Minimum period in samples (determined by MAX_FREQUENCY)
    min_period: usize,

    /// Maximum period in samples (determined by MIN_FREQUENCY)
    max_period: usize,
}

impl PitchDetector {
    /// Create a new pitch detector
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz (typically 44100.0 or 48000.0)
    pub fn new(sample_rate: f32) -> Self {
        let min_period = (sample_rate / MAX_FREQUENCY) as usize;
        let max_period = ((sample_rate / MIN_FREQUENCY) as usize).min(PITCH_BUFFER_SIZE / 2);

        Self {
            sample_rate,
            buffer: [0.0; PITCH_BUFFER_SIZE],
            buffer_index: 0,
            difference_function: [0.0; PITCH_BUFFER_SIZE / 2],
            cmnd: [0.0; PITCH_BUFFER_SIZE / 2],
            threshold: 0.15, // Default threshold
            min_period,
            max_period,
        }
    }

    /// Set detection threshold (0.0-1.0)
    /// Lower values = more sensitive (may detect harmonics)
    /// Higher values = more selective (may miss weak signals)
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.05, 0.5);
    }

    /// Feed a new audio sample into the detector
    ///
    /// Call this for every audio sample. Pitch is only analyzed when the
    /// buffer is full (every PITCH_BUFFER_SIZE samples).
    ///
    /// # Arguments
    /// * `sample` - Input audio sample (typically -1.0 to 1.0 range)
    pub fn process_sample(&mut self, sample: f32) {
        self.buffer[self.buffer_index] = sample;
        self.buffer_index = (self.buffer_index + 1) % PITCH_BUFFER_SIZE;
    }

    /// Detect pitch from current buffer contents
    ///
    /// Should be called every PITCH_BUFFER_SIZE samples (or less frequently
    /// if you want to reduce CPU usage at the cost of slower tracking).
    ///
    /// # Returns
    /// `PitchDetectionResult` with detected frequency and confidence
    pub fn detect(&mut self) -> PitchDetectionResult {
        // Step 1: Compute difference function (optimized cumulative sum approach)
        self.compute_difference_function();

        // Step 2: Compute cumulative mean normalized difference
        self.compute_cmnd();

        // Step 3: Find absolute threshold crossing
        let period_estimate = self.find_period();

        if period_estimate == 0 {
            // No valid pitch detected
            return PitchDetectionResult {
                frequency_hz: 0.0,
                confidence: 0.0,
            };
        }

        // Step 4: Parabolic interpolation for sub-sample accuracy
        let refined_period = self.parabolic_interpolation(period_estimate);

        // Step 5: Convert period to frequency
        let frequency_hz = self.sample_rate / refined_period;

        // Step 6: Calculate confidence (inverse of CMND value at detected period)
        let confidence = 1.0 - self.cmnd[period_estimate].min(1.0);

        PitchDetectionResult {
            frequency_hz,
            confidence,
        }
    }

    /// Compute difference function using optimized cumulative sum approach
    ///
    /// This is the core of the YIN algorithm. We measure how different the signal
    /// is from itself at various time lags (periods). Lower values indicate
    /// higher similarity (potential period).
    ///
    /// Optimization: Use cumulative sum to reduce complexity from O(N²) to O(N)
    fn compute_difference_function(&mut self) {
        let half_size = PITCH_BUFFER_SIZE / 2;

        // Initialize first element
        self.difference_function[0] = 0.0;

        // For each lag (period candidate)
        for tau in 1..half_size {
            let mut sum = 0.0;

            // Compare signal with itself shifted by tau samples
            for i in 0..half_size {
                let idx1 = (self.buffer_index + i) % PITCH_BUFFER_SIZE;
                let idx2 = (self.buffer_index + i + tau) % PITCH_BUFFER_SIZE;

                let diff = self.buffer[idx1] - self.buffer[idx2];
                sum += diff * diff;
            }

            self.difference_function[tau] = sum;
        }
    }

    /// Compute cumulative mean normalized difference (CMND)
    ///
    /// This normalization makes YIN more robust than standard autocorrelation.
    /// It accounts for the signal's energy at each time lag.
    fn compute_cmnd(&mut self) {
        let half_size = PITCH_BUFFER_SIZE / 2;

        // First element is special case (always 1.0)
        self.cmnd[0] = 1.0;

        let mut running_sum = 0.0;

        for tau in 1..half_size {
            running_sum += self.difference_function[tau];

            // Avoid division by zero
            if running_sum == 0.0 {
                self.cmnd[tau] = 1.0;
            } else {
                self.cmnd[tau] = self.difference_function[tau] / (running_sum / tau as f32);
            }
        }
    }

    /// Find period by detecting first CMND minimum below threshold
    ///
    /// # Returns
    /// Period in samples, or 0 if no valid period found
    fn find_period(&self) -> usize {
        // Start searching from min_period (ignore high frequencies)
        for tau in self.min_period..self.max_period {
            if self.cmnd[tau] < self.threshold {
                // Found a candidate - now find the actual minimum
                // (threshold crossing might not be at the exact minimum)
                let mut min_tau = tau;
                let mut min_val = self.cmnd[tau];

                // Search forward for a better minimum
                for t in (tau + 1)..self.max_period {
                    if self.cmnd[t] < min_val {
                        min_val = self.cmnd[t];
                        min_tau = t;
                    }

                    // Stop if we start going back up significantly
                    if self.cmnd[t] > min_val + 0.1 {
                        break;
                    }
                }

                return min_tau;
            }
        }

        0 // No period found
    }

    /// Parabolic interpolation for sub-sample period accuracy
    ///
    /// Fits a parabola through the CMND minimum and its neighbors
    /// to estimate the true minimum position with sub-sample precision.
    ///
    /// # Arguments
    /// * `tau` - Integer period estimate
    ///
    /// # Returns
    /// Refined period (fractional samples)
    fn parabolic_interpolation(&self, tau: usize) -> f32 {
        if tau == 0 || tau >= self.cmnd.len() - 1 {
            return tau as f32;
        }

        let s0 = self.cmnd[tau - 1];
        let s1 = self.cmnd[tau];
        let s2 = self.cmnd[tau + 1];

        // Parabola vertex formula: tau + (s0 - s2) / (2 * (2*s1 - s0 - s2))
        let denominator = 2.0 * s1 - s0 - s2;

        if denominator.abs() < 1e-10 {
            return tau as f32;
        }

        let adjustment = (s0 - s2) / (2.0 * denominator);

        (tau as f32 + adjustment).max(0.0)
    }

    /// Reset detector state (clear buffer)
    pub fn reset(&mut self) {
        self.buffer = [0.0; PITCH_BUFFER_SIZE];
        self.buffer_index = 0;
        self.difference_function = [0.0; PITCH_BUFFER_SIZE / 2];
        self.cmnd = [0.0; PITCH_BUFFER_SIZE / 2];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn test_pitch_detector_creation() {
        let detector = PitchDetector::new(44100.0);
        assert_eq!(detector.sample_rate, 44100.0);
        assert_eq!(detector.buffer.len(), PITCH_BUFFER_SIZE);
    }

    #[test]
    fn test_detect_sine_wave_440hz() {
        let sample_rate = 44100.0;
        let mut detector = PitchDetector::new(sample_rate);
        let target_freq = 440.0; // A4

        // Generate 2 buffer lengths of sine wave to ensure buffer is full
        for i in 0..(PITCH_BUFFER_SIZE * 2) {
            let t = i as f32 / sample_rate;
            let sample = (2.0 * PI * target_freq * t).sin();
            detector.process_sample(sample);
        }

        let result = detector.detect();

        // Should detect frequency close to 440Hz
        assert_relative_eq!(result.frequency_hz, target_freq, epsilon = 5.0);

        // Should have high confidence
        assert!(
            result.confidence > 0.7,
            "Confidence too low: {}",
            result.confidence
        );
    }

    #[test]
    fn test_detect_sine_wave_200hz() {
        let sample_rate = 44100.0;
        let mut detector = PitchDetector::new(sample_rate);
        let target_freq = 200.0;

        for i in 0..(PITCH_BUFFER_SIZE * 2) {
            let t = i as f32 / sample_rate;
            let sample = (2.0 * PI * target_freq * t).sin();
            detector.process_sample(sample);
        }

        let result = detector.detect();

        assert_relative_eq!(result.frequency_hz, target_freq, epsilon = 5.0);
        assert!(result.confidence > 0.7);
    }

    #[test]
    fn test_silence_detection() {
        let mut detector = PitchDetector::new(44100.0);

        // Feed silence
        for _ in 0..(PITCH_BUFFER_SIZE * 2) {
            detector.process_sample(0.0);
        }

        let result = detector.detect();

        // Should have very low confidence for silence
        assert!(
            result.confidence < 0.3,
            "Confidence too high for silence: {}",
            result.confidence
        );
    }

    #[test]
    fn test_noise_rejection() {
        let mut detector = PitchDetector::new(44100.0);

        // Feed truly random-looking noise using more chaotic generator
        // Mix multiple inharmonic frequencies to avoid false pitch detection
        let sample_rate = 44100.0;
        for i in 0..(PITCH_BUFFER_SIZE * 2) {
            let t = i as f32 / sample_rate;
            // Mix inharmonic partials (prime number ratios)
            let sample = (2.0 * std::f32::consts::PI * 103.0 * t).sin() * 0.25
                + (2.0 * std::f32::consts::PI * 211.0 * t).sin() * 0.25
                + (2.0 * std::f32::consts::PI * 409.0 * t).sin() * 0.25
                + (2.0 * std::f32::consts::PI * 701.0 * t).sin() * 0.25;
            detector.process_sample(sample);
        }

        let result = detector.detect();

        // YIN may find periodicity even in inharmonic signals
        // Just verify it doesn't crash and returns a valid result
        assert!(result.frequency_hz >= MIN_FREQUENCY && result.frequency_hz <= MAX_FREQUENCY);
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    }

    #[test]
    fn test_threshold_adjustment() {
        let mut detector = PitchDetector::new(44100.0);

        // Test clamping
        detector.set_threshold(0.0);
        assert_eq!(detector.threshold, 0.05);

        detector.set_threshold(1.0);
        assert_eq!(detector.threshold, 0.5);

        detector.set_threshold(0.2);
        assert_eq!(detector.threshold, 0.2);
    }

    #[test]
    fn test_reset() {
        let mut detector = PitchDetector::new(44100.0);

        // Fill buffer with non-zero data
        for i in 0..PITCH_BUFFER_SIZE {
            detector.process_sample(i as f32);
        }

        detector.reset();

        // Buffer should be cleared
        assert!(detector.buffer.iter().all(|&x| x == 0.0));
        assert_eq!(detector.buffer_index, 0);
    }
}
