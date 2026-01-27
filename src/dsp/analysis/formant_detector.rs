/// Formant Detector - Identifies vowel characteristics in vocal signals
///
/// Formants are **resonant peaks** in the frequency spectrum that define vowel sounds.
/// They are the most important feature for vowel identification:
///
/// # Formant Frequencies by Vowel
/// - **F1** (First formant): Tongue height (low frequency, 200-1000 Hz)
///   - High F1 = open vowels (ah, æ as in "cat")
///   - Low F1 = close vowels (ee, oo)
///
/// - **F2** (Second formant): Tongue front/back position (800-3000 Hz)
///   - High F2 = front vowels (ee as in "beet")
///   - Low F2 = back vowels (oo as in "boot")
///
/// - **F3** (Third formant): Lip rounding and voice quality (1500-4000 Hz)
///
/// # Example Formant Patterns (approximate)
/// - "ee" as in "beet": F1=280, F2=2250, F3=2900
/// - "ih" as in "bit":  F1=400, F2=1900, F3=2550
/// - "ah" as in "father": F1=750, F2=1200, F3=2400
/// - "oo" as in "boot": F1=300, F2=850, F3=2200
///
/// # Algorithm: Simplified LPC (Linear Predictive Coding)
/// True formant detection requires sophisticated LPC analysis with autocorrelation
/// and root-finding. This is a **simplified approach** using:
/// 1. Peak detection in smoothed spectrum (via filterbank)
/// 2. Tracking prominent peaks as formant estimates
/// 3. Temporal smoothing for stability
///
/// Note: This is a practical approximation. For research-grade formant analysis,
/// use dedicated LPC libraries or Praat.
///
/// # Use Cases
/// - Vowel identification and classification
/// - Formant shifting effects (gender/age transformation)
/// - Voice quality analysis (nasal vs oral resonance)
/// - Intelligent EQ (emphasize or reduce formant regions)
/// - Pitch correction (formants must stay constant when shifting pitch)
use std::f32::consts::PI;

/// Simple bandpass filter for formant region analysis
struct FormantFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl FormantFilter {
    fn new(sample_rate: f32, center_freq: f32, bandwidth: f32) -> Self {
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

        filter.update_coefficients(sample_rate, center_freq, bandwidth);
        filter
    }

    fn update_coefficients(&mut self, sample_rate: f32, center_freq: f32, bandwidth: f32) {
        let omega = 2.0 * PI * center_freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();

        // Calculate Q from bandwidth
        let q = center_freq / bandwidth;
        let alpha = sin_omega / (2.0 * q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

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

pub struct FormantDetector {
    sample_rate: f32,

    // Formant tracking filters
    // Sweep through expected formant ranges
    f1_filters: Vec<FormantFilter>, // F1 range: 200-1000 Hz
    f2_filters: Vec<FormantFilter>, // F2 range: 800-3000 Hz
    f3_filters: Vec<FormantFilter>, // F3 range: 1500-4000 Hz

    // Filter center frequencies
    f1_freqs: Vec<f32>,
    f2_freqs: Vec<f32>,
    f3_freqs: Vec<f32>,

    // Energy tracking per filter
    f1_energies: Vec<f32>,
    f2_energies: Vec<f32>,
    f3_energies: Vec<f32>,

    // Energy smoothing coefficient
    energy_smooth_coef: f32,

    // Detected formant frequencies
    f1_hz: f32,
    f2_hz: f32,
    f3_hz: f32,

    // Formant smoothing coefficient
    formant_smooth_coef: f32,
}

impl FormantDetector {
    /// Create a new formant detector
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        // Define frequency ranges for each formant
        // Using 5 filters per formant for reasonable resolution
        let f1_freqs = vec![250.0, 400.0, 550.0, 700.0, 900.0];
        let f2_freqs = vec![900.0, 1400.0, 1900.0, 2400.0, 2900.0];
        let f3_freqs = vec![2000.0, 2500.0, 3000.0, 3500.0, 4000.0];

        let bandwidth = 200.0; // Wide enough to catch formants

        // Create filterbanks
        let f1_filters: Vec<FormantFilter> = f1_freqs
            .iter()
            .map(|&freq| FormantFilter::new(sample_rate, freq, bandwidth))
            .collect();

        let f2_filters: Vec<FormantFilter> = f2_freqs
            .iter()
            .map(|&freq| FormantFilter::new(sample_rate, freq, bandwidth))
            .collect();

        let f3_filters: Vec<FormantFilter> = f3_freqs
            .iter()
            .map(|&freq| FormantFilter::new(sample_rate, freq, bandwidth))
            .collect();

        let mut detector = Self {
            sample_rate,
            f1_filters,
            f2_filters,
            f3_filters,
            f1_freqs: f1_freqs.clone(),
            f2_freqs: f2_freqs.clone(),
            f3_freqs: f3_freqs.clone(),
            f1_energies: vec![0.0; f1_freqs.len()],
            f2_energies: vec![0.0; f2_freqs.len()],
            f3_energies: vec![0.0; f3_freqs.len()],
            energy_smooth_coef: 0.0,
            f1_hz: 500.0, // Default neutral values
            f2_hz: 1500.0,
            f3_hz: 2500.0,
            formant_smooth_coef: 0.0,
        };

        detector.set_energy_smoothing(20.0); // 20ms energy tracking
        detector.set_formant_smoothing(100.0); // 100ms formant smoothing

        detector
    }

    /// Set energy smoothing time constant (faster = more responsive)
    pub fn set_energy_smoothing(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(5.0, 200.0);
        let time_samples = (time_ms / 1000.0) * self.sample_rate;
        self.energy_smooth_coef = (-1.0 / time_samples).exp();
    }

    /// Set formant frequency smoothing (prevents jitter)
    pub fn set_formant_smoothing(&mut self, time_ms: f32) {
        let time_ms = time_ms.clamp(10.0, 500.0);
        let time_samples = (time_ms / 1000.0) * self.sample_rate;
        self.formant_smooth_coef = (-1.0 / time_samples).exp();
    }

    /// Process a mono audio sample and detect formants
    ///
    /// # Arguments
    /// * `input` - Input audio sample
    ///
    /// # Returns
    /// Tuple of (F1, F2, F3) frequencies in Hz
    pub fn process(&mut self, input: f32) -> (f32, f32, f32) {
        // Process through F1 filterbank
        let energy_smooth_coef = self.energy_smooth_coef;
        for i in 0..self.f1_filters.len() {
            let output = self.f1_filters[i].process(input);
            let energy = output * output;
            Self::update_energy(&mut self.f1_energies[i], energy, energy_smooth_coef);
        }

        // Process through F2 filterbank
        for i in 0..self.f2_filters.len() {
            let output = self.f2_filters[i].process(input);
            let energy = output * output;
            Self::update_energy(&mut self.f2_energies[i], energy, energy_smooth_coef);
        }

        // Process through F3 filterbank
        for i in 0..self.f3_filters.len() {
            let output = self.f3_filters[i].process(input);
            let energy = output * output;
            Self::update_energy(&mut self.f3_energies[i], energy, energy_smooth_coef);
        }

        // Find peak energies (formant locations)
        let f1_peak = self.find_peak_frequency(&self.f1_energies, &self.f1_freqs);
        let f2_peak = self.find_peak_frequency(&self.f2_energies, &self.f2_freqs);
        let f3_peak = self.find_peak_frequency(&self.f3_energies, &self.f3_freqs);

        // Smooth formant frequencies
        self.f1_hz += (f1_peak - self.f1_hz) * (1.0 - self.formant_smooth_coef);
        self.f2_hz += (f2_peak - self.f2_hz) * (1.0 - self.formant_smooth_coef);
        self.f3_hz += (f3_peak - self.f3_hz) * (1.0 - self.formant_smooth_coef);

        (self.f1_hz, self.f2_hz, self.f3_hz)
    }

    /// Update energy tracker with smoothing
    fn update_energy(energy: &mut f32, instant_energy: f32, smooth_coef: f32) {
        *energy += (instant_energy - *energy) * (1.0 - smooth_coef);
    }

    /// Find frequency with maximum energy (peak detection)
    fn find_peak_frequency(&self, energies: &[f32], frequencies: &[f32]) -> f32 {
        let mut max_energy = 0.0;
        let mut peak_freq = frequencies[0];

        for i in 0..energies.len() {
            if energies[i] > max_energy {
                max_energy = energies[i];
                peak_freq = frequencies[i];
            }
        }

        peak_freq
    }

    /// Process stereo audio (uses mono sum)
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32, f32) {
        let mono = (left + right) * 0.5;
        self.process(mono)
    }

    /// Get current formant frequencies
    pub fn get_formants(&self) -> (f32, f32, f32) {
        (self.f1_hz, self.f2_hz, self.f3_hz)
    }

    /// Estimate vowel based on formant pattern (simplified classification)
    pub fn estimate_vowel(&self) -> VowelEstimate {
        // Very simplified vowel classification
        // Real vowel detection requires more sophisticated algorithms

        let f1 = self.f1_hz;
        let f2 = self.f2_hz;

        // Classify based on F1/F2 space
        match (f1, f2) {
            (f1, f2) if f1 < 400.0 && f2 > 2000.0 => VowelEstimate::EE, // "beet"
            (f1, f2) if f1 < 500.0 && f2 < 1200.0 => VowelEstimate::OO, // "boot"
            (f1, f2) if f1 > 650.0 && f2 > 1400.0 => VowelEstimate::AE, // "bat"
            (f1, f2) if f1 > 650.0 && f2 < 1400.0 => VowelEstimate::AH, // "father"
            (f1, f2) if f1 < 500.0 && f2 > 1400.0 && f2 < 2000.0 => VowelEstimate::EH, // "bet"
            _ => VowelEstimate::Unknown,
        }
    }

    /// Reset detector state
    pub fn reset(&mut self) {
        for filter in &mut self.f1_filters {
            filter.reset();
        }
        for filter in &mut self.f2_filters {
            filter.reset();
        }
        for filter in &mut self.f3_filters {
            filter.reset();
        }

        for energy in &mut self.f1_energies {
            *energy = 0.0;
        }
        for energy in &mut self.f2_energies {
            *energy = 0.0;
        }
        for energy in &mut self.f3_energies {
            *energy = 0.0;
        }

        self.f1_hz = 500.0;
        self.f2_hz = 1500.0;
        self.f3_hz = 2500.0;
    }
}

#[derive(Debug, PartialEq)]
pub enum VowelEstimate {
    EE, // "beet"
    IH, // "bit"
    EH, // "bet"
    AE, // "bat"
    AH, // "father"
    OO, // "boot"
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_formant_detector_creation() {
        let detector = FormantDetector::new(44100.0);
        let (f1, f2, f3) = detector.get_formants();

        // Should start with neutral formant values
        assert!(f1 > 0.0 && f1 < 2000.0);
        assert!(f2 > 0.0 && f2 < 4000.0);
        assert!(f3 > 0.0 && f3 < 5000.0);
    }

    #[test]
    fn test_process_single_frequency() {
        let mut detector = FormantDetector::new(44100.0);
        let sample_rate = 44100.0;

        // Process tone at F1-like frequency (500 Hz)
        for i in 0..10000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 500.0 * phase).sin() * 0.5;
            detector.process(sample);
        }

        let (f1, _, _) = detector.get_formants();

        // F1 should be near 500 Hz (within filterbank resolution)
        assert!(
            (f1 - 500.0).abs() < 300.0,
            "F1 should track 500 Hz input, got {}",
            f1
        );
    }

    #[test]
    fn test_formant_stability() {
        let mut detector = FormantDetector::new(44100.0);
        let sample_rate = 44100.0;

        // Process for a while
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 1000.0 * phase).sin() * 0.5;
            detector.process(sample);
        }

        let (f1_before, f2_before, f3_before) = detector.get_formants();

        // Continue processing same signal
        for i in 5000..10000 {
            let phase = i as f32 / sample_rate;
            let sample = (2.0 * PI * 1000.0 * phase).sin() * 0.5;
            detector.process(sample);
        }

        let (f1_after, f2_after, f3_after) = detector.get_formants();

        // Formants should be stable (not jumping around)
        assert!((f1_after - f1_before).abs() < 200.0, "F1 should be stable");
        assert!((f2_after - f2_before).abs() < 300.0, "F2 should be stable");
        assert!((f3_after - f3_before).abs() < 500.0, "F3 should be stable");
    }

    #[test]
    fn test_stereo_processing() {
        let mut detector = FormantDetector::new(44100.0);
        let sample_rate = 44100.0;

        // Process stereo signal
        for i in 0..5000 {
            let phase = i as f32 / sample_rate;
            let left = (2.0 * PI * 700.0 * phase).sin() * 0.5;
            let right = (2.0 * PI * 700.0 * phase).sin() * 0.5;
            detector.process_stereo(left, right);
        }

        let (f1, _, _) = detector.get_formants();
        assert!(f1 > 0.0, "Stereo processing should produce valid formants");
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = FormantDetector::new(44100.0);

        // Process some signal
        for i in 0..2000 {
            detector.process((i as f32).sin() * 0.5);
        }

        // Reset
        detector.reset();

        // Energies should be cleared
        for energy in &detector.f1_energies {
            assert_relative_eq!(*energy, 0.0, epsilon = 0.001);
        }
        for energy in &detector.f2_energies {
            assert_relative_eq!(*energy, 0.0, epsilon = 0.001);
        }
        for energy in &detector.f3_energies {
            assert_relative_eq!(*energy, 0.0, epsilon = 0.001);
        }
    }

    #[test]
    fn test_vowel_estimation_boundaries() {
        let mut detector = FormantDetector::new(44100.0);

        // Test EE vowel pattern (low F1, high F2)
        detector.f1_hz = 350.0;
        detector.f2_hz = 2300.0;
        assert_eq!(detector.estimate_vowel(), VowelEstimate::EE);

        // Test OO vowel pattern (low F1, low F2)
        detector.f1_hz = 350.0;
        detector.f2_hz = 900.0;
        assert_eq!(detector.estimate_vowel(), VowelEstimate::OO);

        // Test AH vowel pattern (high F1, low F2)
        detector.f1_hz = 750.0;
        detector.f2_hz = 1200.0;
        assert_eq!(detector.estimate_vowel(), VowelEstimate::AH);

        // Test AE vowel pattern (high F1, high F2)
        detector.f1_hz = 750.0;
        detector.f2_hz = 1700.0;
        assert_eq!(detector.estimate_vowel(), VowelEstimate::AE);
    }
}
