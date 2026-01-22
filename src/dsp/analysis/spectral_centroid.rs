/// Spectral Centroid Detector - Measures "brightness" or timbral character
///
/// The spectral centroid is the **center of mass** of the frequency spectrum.
/// Think of it as the "average frequency" weighted by amplitude.
///
/// # Interpretation
/// - **Low centroid** (500-2000 Hz): Dark, warm, bass-heavy sound
/// - **Medium centroid** (2000-5000 Hz): Balanced, natural timbre
/// - **High centroid** (5000+ Hz): Bright, airy, treble-heavy sound
///
/// # Algorithm
/// Uses a simplified **spectral estimation** via filterbank:
/// 1. Split signal into frequency bands (low, mid-low, mid-high, high)
/// 2. Measure energy in each band using bandpass filters
/// 3. Calculate weighted average: centroid = Σ(freq * energy) / Σ(energy)
///
/// This is faster than FFT for real-time use and sufficient for brightness tracking.
///
/// # Use Cases
/// - Adaptive EQ (brighten dark signals, darken bright ones)
/// - Exciter amount control (less exciter for already-bright sounds)
/// - De-esser threshold adaptation (track sibilance energy)
/// - Timbral matching between tracks
/// - Voice quality analysis (nasal vs chest voice has different centroids)
///
/// # Parameters
/// - **smoothing_ms**: Time constant for centroid smoothing (10-100ms)
use std::f32::consts::PI;

/// Simple 2nd-order bandpass filter for spectral analysis
struct BandpassFilter {
    // Filter coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    // State variables (Direct Form I)
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BandpassFilter {
    fn new(sample_rate: f32, center_freq: f32, q: f32) -> Self {
        let mut filter = Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };

        filter.update_coefficients(sample_rate, center_freq, q);
        filter
    }

    fn update_coefficients(&mut self, sample_rate: f32, center_freq: f32, q: f32) {
        let omega = 2.0 * PI * center_freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        // Bandpass filter (constant 0 dB peak gain)
        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        // Normalize
        if a0.abs() > 1e-6 {
            self.b0 = b0 / a0;
            self.b1 = b1 / a0;
            self.b2 = b2 / a0;
            self.a1 = a1 / a0;
            self.a2 = a2 / a0;
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

pub struct SpectralCentroid {
    sample_rate: f32,

    // Filterbank for spectral analysis
    // Each band represents a frequency range
    band_low: BandpassFilter,      // ~500 Hz (bass/low-mids)
    band_mid_low: BandpassFilter,  // ~1500 Hz (mid-range)
    band_mid_high: BandpassFilter, // ~4000 Hz (presence)
    band_high: BandpassFilter,     // ~8000 Hz (brilliance)

    // Band center frequencies (Hz)
    freq_low: f32,
    freq_mid_low: f32,
    freq_mid_high: f32,
    freq_high: f32,

    // Energy tracking for each band
    energy_low: f32,
    energy_mid_low: f32,
    energy_mid_high: f32,
    energy_high: f32,

    // Energy envelope coefficients (smoothing)
    energy_attack_coef: f32,
    energy_release_coef: f32,

    // Centroid smoothing
    centroid_smooth_coef: f32,
    centroid_hz: f32,
}

impl SpectralCentroid {
    /// Create a new spectral centroid detector
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        // Define frequency bands (logarithmically spaced for perceptual relevance)
        let freq_low = 500.0;
        let freq_mid_low = 1500.0;
        let freq_mid_high = 4000.0;
        let freq_high = 8000.0;

        let q = 1.5; // Moderate Q for reasonable bandwidth

        let mut detector = Self {
            sample_rate,
            band_low: BandpassFilter::new(sample_rate, freq_low, q),
            band_mid_low: BandpassFilter::new(sample_rate, freq_mid_low, q),
            band_mid_high: BandpassFilter::new(sample_rate, freq_mid_high, q),
            band_high: BandpassFilter::new(sample_rate, freq_high, q),
            freq_low,
            freq_mid_low,
            freq_mid_high,
            freq_high,
            energy_low: 0.0,
            energy_mid_low: 0.0,
            energy_mid_high: 0.0,
            energy_high: 0.0,
            energy_attack_coef: 0.0,
            energy_release_coef: 0.0,
            centroid_smooth_coef: 0.0,
            centroid_hz: 0.0,
        };

        // Default smoothing times
        detector.set_energy_time_constant(10.0); // 10ms energy tracking
        detector.set_smoothing_time(50.0); // 50ms centroid smoothing

        detector
    }

    /// Set energy envelope time constant (faster = more responsive, 5-50ms typical)
    pub fn set_energy_time_constant(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(1.0, 200.0);
        let coef = self.ms_to_coefficient(time_ms);
        self.energy_attack_coef = coef;
        self.energy_release_coef = coef;
    }

    /// Set centroid smoothing time (10-100ms, default 50ms)
    pub fn set_smoothing_time(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(5.0, 500.0);
        self.centroid_smooth_coef = self.ms_to_coefficient(time_ms);
    }

    /// Convert milliseconds to exponential smoothing coefficient
    fn ms_to_coefficient(&self, time_ms: f32) -> f32 {
        let time_samples = (time_ms / 1000.0) * self.sample_rate;
        (-1.0 / time_samples).exp()
    }

    /// Process a mono audio sample and compute spectral centroid
    ///
    /// # Arguments
    /// * `input` - Input audio sample
    ///
    /// # Returns
    /// Current spectral centroid in Hz (typically 500-10000 Hz)
    pub fn process(&mut self, input: f32) -> f32 {
        // Filter input through each band
        let band_low_out = self.band_low.process(input);
        let band_mid_low_out = self.band_mid_low.process(input);
        let band_mid_high_out = self.band_mid_high.process(input);
        let band_high_out = self.band_high.process(input);

        // Measure instantaneous energy (squared amplitude) in each band
        let energy_low_instant = band_low_out * band_low_out;
        let energy_mid_low_instant = band_mid_low_out * band_mid_low_out;
        let energy_mid_high_instant = band_mid_high_out * band_mid_high_out;
        let energy_high_instant = band_high_out * band_high_out;

        // Smooth energies with attack/release envelopes
        Self::update_energy(
            &mut self.energy_low,
            energy_low_instant,
            self.energy_attack_coef,
            self.energy_release_coef,
        );
        Self::update_energy(
            &mut self.energy_mid_low,
            energy_mid_low_instant,
            self.energy_attack_coef,
            self.energy_release_coef,
        );
        Self::update_energy(
            &mut self.energy_mid_high,
            energy_mid_high_instant,
            self.energy_attack_coef,
            self.energy_release_coef,
        );
        Self::update_energy(
            &mut self.energy_high,
            energy_high_instant,
            self.energy_attack_coef,
            self.energy_release_coef,
        );

        // Calculate spectral centroid (weighted average of frequencies)
        let total_energy =
            self.energy_low + self.energy_mid_low + self.energy_mid_high + self.energy_high;

        if total_energy > 1e-6 {
            // Weighted sum: Σ(frequency * energy)
            let weighted_sum = self.freq_low * self.energy_low
                + self.freq_mid_low * self.energy_mid_low
                + self.freq_mid_high * self.energy_mid_high
                + self.freq_high * self.energy_high;

            // Centroid = weighted sum / total energy
            let centroid_instant = weighted_sum / total_energy;

            // Smooth centroid for stability
            self.centroid_hz +=
                (centroid_instant - self.centroid_hz) * (1.0 - self.centroid_smooth_coef);
        } else {
            // No signal - decay centroid toward neutral value
            self.centroid_hz += (2000.0 - self.centroid_hz) * (1.0 - self.centroid_smooth_coef);
        }

        self.centroid_hz.clamp(100.0, 15000.0)
    }

    /// Update energy tracker with attack/release envelope
    fn update_energy(
        energy: &mut f32,
        instant_energy: f32,
        attack_coef: f32,
        release_coef: f32,
    ) {
        if instant_energy > *energy {
            // Attack
            *energy += (instant_energy - *energy) * (1.0 - attack_coef);
        } else {
            // Release
            *energy *= release_coef;
        }
    }

    /// Process stereo audio (uses average of both channels)
    pub fn process_stereo(&mut self, left: f32, right: f32) -> f32 {
        let mono = (left + right) * 0.5;
        self.process(mono)
    }

    /// Get current centroid without processing new sample
    pub fn get_centroid_hz(&self) -> f32 {
        self.centroid_hz
    }

    /// Get normalized brightness (0.0 = dark, 1.0 = bright)
    /// Maps centroid range ~500-8000 Hz to 0-1
    pub fn get_brightness(&self) -> f32 {
        let min_freq = 500.0;
        let max_freq = 8000.0;
        ((self.centroid_hz - min_freq) / (max_freq - min_freq)).clamp(0.0, 1.0)
    }

    /// Classify timbre based on centroid
    pub fn get_timbre_description(&self) -> &'static str {
        match self.centroid_hz {
            f if f < 1000.0 => "Very Dark/Bassy",
            f if f < 2000.0 => "Dark/Warm",
            f if f < 3500.0 => "Balanced/Natural",
            f if f < 6000.0 => "Bright/Present",
            _ => "Very Bright/Airy",
        }
    }

    /// Reset detector state
    pub fn reset(&mut self) {
        self.band_low.reset();
        self.band_mid_low.reset();
        self.band_mid_high.reset();
        self.band_high.reset();
        self.energy_low = 0.0;
        self.energy_mid_low = 0.0;
        self.energy_mid_high = 0.0;
        self.energy_high = 0.0;
        self.centroid_hz = 2000.0; // Reset to neutral value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn test_spectral_centroid_creation() {
        let detector = SpectralCentroid::new(44100.0);
        let centroid = detector.get_centroid_hz();
        assert!(
            centroid > 0.0 && centroid < 15000.0,
            "Initial centroid should be in valid range"
        );
    }

    #[test]
    fn test_low_frequency_signal() {
        let mut detector = SpectralCentroid::new(44100.0);
        let sample_rate = 44100.0;

        // Process low-frequency sine wave (300 Hz)
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 300.0 * phase).sin() * 0.5;
            detector.process(sample);
        }

        let centroid = detector.get_centroid_hz();
        let brightness = detector.get_brightness();

        // Low frequency should produce low centroid
        assert!(
            centroid < 2000.0,
            "Low frequency signal should have low centroid, got {}",
            centroid
        );
        assert!(
            brightness < 0.4,
            "Low frequency signal should have low brightness"
        );
    }

    #[test]
    fn test_high_frequency_signal() {
        let mut detector = SpectralCentroid::new(44100.0);
        let sample_rate = 44100.0;

        // Process high-frequency sine wave (6000 Hz)
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 6000.0 * phase).sin() * 0.5;
            detector.process(sample);
        }

        let centroid = detector.get_centroid_hz();
        let brightness = detector.get_brightness();

        // High frequency should produce high centroid
        assert!(
            centroid > 4000.0,
            "High frequency signal should have high centroid, got {}",
            centroid
        );
        assert!(
            brightness > 0.5,
            "High frequency signal should have high brightness"
        );
    }

    #[test]
    fn test_broadband_signal() {
        let mut detector = SpectralCentroid::new(44100.0);
        let sample_rate = 44100.0;

        // Process broadband signal (mixture of frequencies)
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let low = (2.0 * PI * 500.0 * phase).sin() * 0.3;
            let high = (2.0 * PI * 5000.0 * phase).sin() * 0.3;
            let sample = low + high;
            detector.process(sample);
        }

        let centroid = detector.get_centroid_hz();

        // Broadband should produce medium centroid
        assert!(
            centroid > 1500.0 && centroid < 5000.0,
            "Broadband signal should have medium centroid, got {}",
            centroid
        );
    }

    #[test]
    fn test_stereo_processing() {
        let mut detector = SpectralCentroid::new(44100.0);
        let sample_rate = 44100.0;

        // Process stereo high-frequency signal
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let left = (2.0 * PI * 6000.0 * phase).sin() * 0.5;
            let right = (2.0 * PI * 6000.0 * phase).sin() * 0.5;
            detector.process_stereo(left, right);
        }

        let centroid = detector.get_centroid_hz();
        assert!(
            centroid > 4000.0,
            "Stereo processing should track high frequencies"
        );
    }

    #[test]
    fn test_silence_decay() {
        let mut detector = SpectralCentroid::new(44100.0);
        let sample_rate = 44100.0;

        // First process high-frequency signal
        for i in 0..2000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 7000.0 * phase).sin() * 0.5;
            detector.process(sample);
        }

        let centroid_before = detector.get_centroid_hz();

        // Then process silence
        for _ in 0..5000 {
            detector.process(0.0);
        }

        let centroid_after = detector.get_centroid_hz();

        // Centroid should decay toward neutral value
        assert!(
            centroid_after < centroid_before,
            "Centroid should decay during silence"
        );
        assert!(
            centroid_after > 1000.0 && centroid_after < 4000.0,
            "Centroid should settle near neutral value"
        );
    }

    #[test]
    fn test_timbre_classification() {
        let mut detector = SpectralCentroid::new(44100.0);

        // Test different frequency regions
        detector.centroid_hz = 800.0;
        assert!(
            detector.get_timbre_description().contains("Dark")
                || detector.get_timbre_description().contains("Bassy")
        );

        detector.centroid_hz = 2500.0;
        assert!(detector.get_timbre_description().contains("Balanced"));

        detector.centroid_hz = 7000.0;
        assert!(detector.get_timbre_description().contains("Bright"));
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = SpectralCentroid::new(44100.0);

        // Process signal
        for i in 0..1000 {
            detector.process((i as f32).sin() * 0.5);
        }

        // Reset
        detector.reset();

        // Energy should be cleared
        assert_relative_eq!(detector.energy_low, 0.0, epsilon = 0.001);
        assert_relative_eq!(detector.energy_mid_low, 0.0, epsilon = 0.001);
        assert_relative_eq!(detector.energy_mid_high, 0.0, epsilon = 0.001);
        assert_relative_eq!(detector.energy_high, 0.0, epsilon = 0.001);
    }
}
