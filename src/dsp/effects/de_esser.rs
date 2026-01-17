/// De-esser effect
///
/// Reduces harsh sibilance (S, T, SH sounds) in vocal recordings.
/// Works by detecting high-frequency content and applying frequency-selective
/// compression only when sibilance is present.
///
/// # How It Works
/// 1. Split signal into detection path (high-shelf filter) and main path
/// 2. Analyze high-frequency energy to detect sibilance
/// 3. Apply compression only when sibilance exceeds threshold
/// 4. Compress only the high-frequency band to preserve overall tone
use crate::dsp::effects::compressor::Compressor;
use crate::dsp::filter::BiquadFilter;
use crate::params::FilterType;

/// De-esser with frequency-selective compression
pub struct DeEsser {
    sample_rate: f32,
    
    /// Detection frequency (typically 4kHz-10kHz for sibilance)
    frequency_hz: f32,
    
    /// Threshold in dB (sibilance above this level triggers compression)
    threshold_db: f32,
    
    /// Compression ratio (1:1 to 10:1, typically 3:1-6:1)
    ratio: f32,
    
    /// Enabled flag
    enabled: bool,
    
    /// High-shelf filter for sibilance detection
    /// This isolates the frequency range where sibilance occurs
    detector_filter_left: BiquadFilter,
    detector_filter_right: BiquadFilter,
    
    /// Split-band filters for processing
    /// High-pass: isolates sibilance band for compression
    /// Low-pass: passes low frequencies unchanged
    high_pass_left: BiquadFilter,
    high_pass_right: BiquadFilter,
    low_pass_left: BiquadFilter,
    low_pass_right: BiquadFilter,
    
    /// Compressor for sibilance band
    compressor: Compressor,
    
    /// Fixed Q factor for detection (moderate width catches typical sibilance)
    detection_q: f32,
}

impl DeEsser {
    /// Create a new de-esser
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let frequency_hz = 6000.0; // Default sibilance frequency
        let threshold_db = -15.0;
        let ratio = 4.0;
        let detection_q = 2.0; // Fixed moderate Q
        
        // Detector: high-shelf filter to isolate sibilance
        let mut detector_filter_left = BiquadFilter::new(sample_rate);
        detector_filter_left.set_filter_type(FilterType::HighShelf);
        detector_filter_left.set_cutoff(frequency_hz);
        detector_filter_left.set_gain_db(12.0); // Boost to emphasize sibilance in detection
        detector_filter_left.set_resonance(detection_q);
        
        let mut detector_filter_right = BiquadFilter::new(sample_rate);
        detector_filter_right.set_filter_type(FilterType::HighShelf);
        detector_filter_right.set_cutoff(frequency_hz);
        detector_filter_right.set_gain_db(12.0);
        detector_filter_right.set_resonance(detection_q);
        
        // Split-band filters
        let mut high_pass_left = BiquadFilter::new(sample_rate);
        high_pass_left.set_filter_type(FilterType::Highpass);
        high_pass_left.set_cutoff(frequency_hz);
        high_pass_left.set_resonance(0.707); // Butterworth response
        
        let mut high_pass_right = BiquadFilter::new(sample_rate);
        high_pass_right.set_filter_type(FilterType::Highpass);
        high_pass_right.set_cutoff(frequency_hz);
        high_pass_right.set_resonance(0.707);
        
        let mut low_pass_left = BiquadFilter::new(sample_rate);
        low_pass_left.set_filter_type(FilterType::Lowpass);
        low_pass_left.set_cutoff(frequency_hz);
        low_pass_left.set_resonance(0.707);
        
        let mut low_pass_right = BiquadFilter::new(sample_rate);
        low_pass_right.set_filter_type(FilterType::Lowpass);
        low_pass_right.set_cutoff(frequency_hz);
        low_pass_right.set_resonance(0.707);
        
        // Compressor for sibilance band (fast attack for transient sibilance)
        let compressor = Compressor::new(sample_rate, threshold_db, ratio, 0.5, 50.0);
        
        Self {
            sample_rate,
            frequency_hz,
            threshold_db,
            ratio,
            enabled: true,
            detector_filter_left,
            detector_filter_right,
            high_pass_left,
            high_pass_right,
            low_pass_left,
            low_pass_right,
            compressor,
            detection_q,
        }
    }
    
    /// Enable or disable de-esser
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Set sibilance detection frequency (4kHz-10kHz typical)
    pub fn set_frequency(&mut self, frequency_hz: f32) {
        self.frequency_hz = frequency_hz.clamp(4000.0, 10000.0);
        
        // Update detector filters
        self.detector_filter_left.set_cutoff(self.frequency_hz);
        self.detector_filter_right.set_cutoff(self.frequency_hz);
        
        // Update split-band filters
        self.high_pass_left.set_cutoff(self.frequency_hz);
        self.high_pass_right.set_cutoff(self.frequency_hz);
        self.low_pass_left.set_cutoff(self.frequency_hz);
        self.low_pass_right.set_cutoff(self.frequency_hz);
    }
    
    /// Set threshold in dB
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db.clamp(-40.0, 0.0);
        self.compressor.set_threshold(self.threshold_db);
    }
    
    /// Set compression ratio
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(1.0, 10.0);
        self.compressor.set_ratio(self.ratio);
    }
    
    /// Process one stereo sample pair
    ///
    /// # Arguments
    /// * `left` - Left channel input
    /// * `right` - Right channel input
    ///
    /// # Returns
    /// Tuple of (left, right) output samples
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        
        // Step 1: Detect sibilance using high-shelf filtered signal
        // (Not actually used for sidechain - we compress based on high-band energy directly)
        // This is here for potential future sidechain implementation
        let _detected_left = self.detector_filter_left.process(left);
        let _detected_right = self.detector_filter_right.process(right);
        
        // Step 2: Split signal into low and high bands
        let low_left = self.low_pass_left.process(left);
        let low_right = self.low_pass_right.process(right);
        
        let high_left = self.high_pass_left.process(left);
        let high_right = self.high_pass_right.process(right);
        
        // Step 3: Compress only the high band (where sibilance lives)
        let (compressed_high_left, compressed_high_right) = self.compressor.process(high_left, high_right);
        
        // Step 4: Recombine low (unchanged) and high (compressed) bands
        let output_left = low_left + compressed_high_left;
        let output_right = low_right + compressed_high_right;
        
        (output_left, output_right)
    }
    
    /// Reset all filter and compressor states
    pub fn reset(&mut self) {
        self.detector_filter_left.reset();
        self.detector_filter_right.reset();
        self.high_pass_left.reset();
        self.high_pass_right.reset();
        self.low_pass_left.reset();
        self.low_pass_right.reset();
        self.compressor.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f32::consts::PI;
    
    #[test]
    fn test_deesser_creation() {
        let de_esser = DeEsser::new(44100.0);
        assert_eq!(de_esser.sample_rate, 44100.0);
        assert_eq!(de_esser.frequency_hz, 6000.0);
        assert_eq!(de_esser.threshold_db, -15.0);
        assert!(de_esser.enabled);
    }
    
    #[test]
    fn test_frequency_clamping() {
        let mut de_esser = DeEsser::new(44100.0);
        
        de_esser.set_frequency(2000.0);
        assert_eq!(de_esser.frequency_hz, 4000.0); // Clamped to min
        
        de_esser.set_frequency(15000.0);
        assert_eq!(de_esser.frequency_hz, 10000.0); // Clamped to max
        
        de_esser.set_frequency(6000.0);
        assert_eq!(de_esser.frequency_hz, 6000.0);
    }
    
    #[test]
    fn test_threshold_clamping() {
        let mut de_esser = DeEsser::new(44100.0);
        
        de_esser.set_threshold(-50.0);
        assert_eq!(de_esser.threshold_db, -40.0); // Clamped to min
        
        de_esser.set_threshold(10.0);
        assert_eq!(de_esser.threshold_db, 0.0); // Clamped to max
    }
    
    #[test]
    fn test_ratio_clamping() {
        let mut de_esser = DeEsser::new(44100.0);
        
        de_esser.set_ratio(0.5);
        assert_eq!(de_esser.ratio, 1.0); // Clamped to min
        
        de_esser.set_ratio(20.0);
        assert_eq!(de_esser.ratio, 10.0); // Clamped to max
    }
    
    #[test]
    fn test_bypass_when_disabled() {
        let mut de_esser = DeEsser::new(44100.0);
        de_esser.set_enabled(false);
        
        let input = 0.5;
        let (left, right) = de_esser.process(input, input);
        
        // Should pass through unchanged when disabled
        assert_eq!(left, input);
        assert_eq!(right, input);
    }
    
    #[test]
    fn test_low_frequency_passthrough() {
        let mut de_esser = DeEsser::new(44100.0);
        de_esser.set_frequency(6000.0);
        de_esser.set_threshold(-20.0);
        de_esser.set_ratio(4.0);
        
        let sample_rate = 44100.0;
        let freq = 1000.0; // Well below de-esser frequency
        
        let mut input_sum = 0.0;
        let mut output_sum = 0.0;
        
        for i in 0..1000 {
            let t = i as f32 / sample_rate;
            let input = (2.0 * PI * freq * t).sin() * 0.5;
            let (left, _) = de_esser.process(input, input);
            
            input_sum += input.abs();
            output_sum += left.abs();
        }
        
        // Low frequencies should pass through relatively unchanged
        // (some phase shift and crossover artifacts expected)
        let ratio = output_sum / input_sum;
        assert!(ratio > 0.8 && ratio < 1.2, "Low freq should pass through, ratio: {}", ratio);
    }
    
    #[test]
    fn test_high_frequency_reduction() {
        let mut de_esser = DeEsser::new(44100.0);
        de_esser.set_frequency(6000.0);
        de_esser.set_threshold(-30.0); // Low threshold for aggressive de-essing
        de_esser.set_ratio(6.0);
        
        let sample_rate = 44100.0;
        let freq = 7000.0; // Above de-esser frequency (sibilance range)
        
        let mut input_peaks = Vec::new();
        let mut output_peaks = Vec::new();
        
        for i in 0..5000 {
            let t = i as f32 / sample_rate;
            let input = (2.0 * PI * freq * t).sin() * 0.3; // Loud sibilance
            let (left, _) = de_esser.process(input, input);
            
            if i > 1000 { // Skip transient settling
                input_peaks.push(input.abs());
                output_peaks.push(left.abs());
            }
        }
        
        let input_max = input_peaks.iter().cloned().fold(0.0, f32::max);
        let output_max = output_peaks.iter().cloned().fold(0.0, f32::max);
        
        // High-frequency content should be reduced
        assert!(output_max < input_max * 0.9, "De-esser should reduce high freq peaks");
    }
    
    #[test]
    fn test_enable_disable() {
        let mut de_esser = DeEsser::new(44100.0);
        
        // Enable
        de_esser.set_enabled(true);
        assert!(de_esser.enabled);
        
        // Disable
        de_esser.set_enabled(false);
        assert!(!de_esser.enabled);
    }
    
    #[test]
    fn test_reset() {
        let mut de_esser = DeEsser::new(44100.0);
        
        // Process some signal to build up state
        for _ in 0..1000 {
            de_esser.process(0.5, 0.5);
        }
        
        de_esser.reset();
        
        // After reset, DC input should give DC output (no filter memory)
        let (left, right) = de_esser.process(1.0, 1.0);
        
        // With compression and filtering, won't be exactly 1.0 but should be finite
        assert!(left.is_finite());
        assert!(right.is_finite());
    }
    
    #[test]
    fn test_stereo_processing() {
        let mut de_esser = DeEsser::new(44100.0);
        
        // Different inputs should produce different outputs
        let (left_out, right_out) = de_esser.process(0.3, 0.7);
        
        assert_ne!(left_out, right_out);
    }
}
