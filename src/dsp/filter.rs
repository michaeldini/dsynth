use crate::params::FilterType;
use std::f32::consts::PI;

/// Biquad filter with coefficient clamping for stability
/// Implements lowpass, highpass, and bandpass filters using Audio EQ Cookbook formulas
pub struct BiquadFilter {
    sample_rate: f32,
    filter_type: FilterType,
    cutoff: f32,
    resonance: f32,

    // Biquad coefficients
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

impl BiquadFilter {
    /// Create a new biquad filter
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut filter = Self {
            sample_rate,
            filter_type: FilterType::Lowpass,
            cutoff: 1000.0,
            resonance: 0.707,
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
        filter.update_coefficients();
        filter
    }

    /// Set filter type
    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        if self.filter_type != filter_type {
            self.filter_type = filter_type;
            self.update_coefficients();
        }
    }

    /// Set cutoff frequency in Hz (20 - 20000)
    pub fn set_cutoff(&mut self, cutoff: f32) {
        let clamped = cutoff.clamp(20.0, self.sample_rate * 0.49);
        if self.cutoff != clamped {
            self.cutoff = clamped;
            self.update_coefficients();
        }
    }

    /// Set resonance (Q factor, 0.5 - 10.0)
    pub fn set_resonance(&mut self, resonance: f32) {
        let clamped = resonance.clamp(0.5, 10.0);
        if self.resonance != clamped {
            self.resonance = clamped;
            self.update_coefficients();
        }
    }

    /// Update biquad coefficients based on current parameters
    /// Uses Audio EQ Cookbook formulas with stability clamping
    fn update_coefficients(&mut self) {
        let omega = 2.0 * PI * self.cutoff / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * self.resonance);

        let (mut b0, mut b1, mut b2, a0, mut a1, mut a2) = match self.filter_type {
            FilterType::Lowpass => {
                let b1_temp = 1.0 - cos_omega;
                let b0_temp = b1_temp / 2.0;
                let b2_temp = b0_temp;
                let a0_temp = 1.0 + alpha;
                let a1_temp = -2.0 * cos_omega;
                let a2_temp = 1.0 - alpha;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
            FilterType::Highpass => {
                let b0_temp = (1.0 + cos_omega) / 2.0;
                let b1_temp = -(1.0 + cos_omega);
                let b2_temp = (1.0 + cos_omega) / 2.0;
                let a0_temp = 1.0 + alpha;
                let a1_temp = -2.0 * cos_omega;
                let a2_temp = 1.0 - alpha;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
            FilterType::Bandpass => {
                let b0_temp = alpha;
                let b1_temp = 0.0;
                let b2_temp = -alpha;
                let a0_temp = 1.0 + alpha;
                let a1_temp = -2.0 * cos_omega;
                let a2_temp = 1.0 - alpha;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
        };

        // Normalize by a0
        b0 /= a0;
        b1 /= a0;
        b2 /= a0;
        a1 /= a0;
        a2 /= a0;

        // Clamp coefficients for stability
        // Prevent excessive values that could cause instability
        b0 = Self::clamp_coefficient(b0, 3.0);
        b1 = Self::clamp_coefficient(b1, 3.0);
        b2 = Self::clamp_coefficient(b2, 3.0);
        a1 = Self::clamp_coefficient(a1, 2.0);
        a2 = Self::clamp_coefficient(a2, 1.0);

        self.b0 = b0;
        self.b1 = b1;
        self.b2 = b2;
        self.a1 = a1;
        self.a2 = a2;
    }

    /// Clamp coefficient to prevent instability
    fn clamp_coefficient(value: f32, max_abs: f32) -> f32 {
        value.clamp(-max_abs, max_abs)
    }

    /// Process one sample through the filter with optional drive
    pub fn process(&mut self, input: f32) -> f32 {
        // Direct Form I implementation
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // Update state
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    /// Process with drive/saturation for warmth and harmonics
    pub fn process_with_drive(&mut self, input: f32, drive: f32) -> f32 {
        // Apply pre-filter drive
        let driven = input * drive;

        // Soft clipping using tanh for warmth
        let saturated = if drive > 1.0 {
            driven.tanh() / drive.tanh()
        } else {
            driven
        };

        // Process through filter
        let output = self.b0 * saturated + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // Update state
        self.x2 = self.x1;
        self.x1 = saturated;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_filter_creation() {
        let filter = BiquadFilter::new(44100.0);
        assert_eq!(filter.sample_rate, 44100.0);
        assert_eq!(filter.filter_type, FilterType::Lowpass);
    }

    #[test]
    fn test_cutoff_clamping() {
        let mut filter = BiquadFilter::new(44100.0);

        // Too low
        filter.set_cutoff(10.0);
        assert_eq!(filter.cutoff, 20.0);

        // Too high (above Nyquist)
        filter.set_cutoff(25000.0);
        assert!(filter.cutoff < 22050.0);

        // Valid
        filter.set_cutoff(1000.0);
        assert_eq!(filter.cutoff, 1000.0);
    }

    #[test]
    fn test_resonance_clamping() {
        let mut filter = BiquadFilter::new(44100.0);

        filter.set_resonance(0.1);
        assert_eq!(filter.resonance, 0.5);

        filter.set_resonance(20.0);
        assert_eq!(filter.resonance, 10.0);

        filter.set_resonance(2.0);
        assert_eq!(filter.resonance, 2.0);
    }

    #[test]
    fn test_coefficient_stability() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_cutoff(20000.0);
        filter.set_resonance(10.0);

        // Coefficients should be within stable bounds
        assert!(filter.a1.abs() <= 2.0);
        assert!(filter.a2.abs() <= 1.0);
        assert!(filter.b0.abs() <= 3.0);
        assert!(filter.b1.abs() <= 3.0);
        assert!(filter.b2.abs() <= 3.0);
    }

    #[test]
    fn test_lowpass_dc_signal() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_filter_type(FilterType::Lowpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.707);

        // DC signal (0 Hz) should pass through lowpass
        let mut output_sum = 0.0;
        for _ in 0..100 {
            output_sum += filter.process(1.0);
        }

        let average = output_sum / 100.0;
        assert_relative_eq!(average, 1.0, epsilon = 0.1);
    }

    #[test]
    fn test_highpass_dc_rejection() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_filter_type(FilterType::Highpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.707);

        // DC signal should be rejected by highpass
        let mut output_sum = 0.0;
        for _ in 0..100 {
            output_sum += filter.process(1.0);
        }

        let average = output_sum / 100.0;
        assert!(
            average.abs() < 0.2,
            "DC should be rejected, got {}",
            average
        );
    }

    #[test]
    fn test_lowpass_frequency_response() {
        let sample_rate = 44100.0;
        let mut filter = BiquadFilter::new(sample_rate);
        filter.set_filter_type(FilterType::Lowpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.707);

        // Test passband (100 Hz - well below cutoff)
        filter.reset();
        let passband_freq = 100.0;
        let mut max_passband: f32 = 0.0;

        for i in 0..1000 {
            let input = (2.0 * PI * passband_freq * i as f32 / sample_rate).sin();
            let output = filter.process(input);
            max_passband = max_passband.max(output.abs());
        }

        // Test stopband (10000 Hz - well above cutoff)
        filter.reset();
        let stopband_freq = 10000.0;
        let mut max_stopband: f32 = 0.0;

        for i in 0..1000 {
            let input = (2.0 * PI * stopband_freq * i as f32 / sample_rate).sin();
            let output = filter.process(input);
            max_stopband = max_stopband.max(output.abs());
        }

        // Passband should have higher amplitude than stopband
        assert!(
            max_passband > max_stopband * 5.0,
            "Passband {} should be >> stopband {}",
            max_passband,
            max_stopband
        );
    }

    #[test]
    fn test_highpass_frequency_response() {
        let sample_rate = 44100.0;
        let mut filter = BiquadFilter::new(sample_rate);
        filter.set_filter_type(FilterType::Highpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.707);

        // Test stopband (100 Hz - well below cutoff)
        filter.reset();
        let stopband_freq = 100.0;
        let mut max_stopband: f32 = 0.0;

        for i in 0..1000 {
            let input = (2.0 * PI * stopband_freq * i as f32 / sample_rate).sin();
            let output = filter.process(input);
            max_stopband = max_stopband.max(output.abs());
        }

        // Test passband (5000 Hz - well above cutoff)
        filter.reset();
        let passband_freq = 5000.0;
        let mut max_passband: f32 = 0.0;

        for i in 0..1000 {
            let input = (2.0 * PI * passband_freq * i as f32 / sample_rate).sin();
            let output = filter.process(input);
            max_passband = max_passband.max(output.abs());
        }

        // Passband should have higher amplitude than stopband
        assert!(
            max_passband > max_stopband * 5.0,
            "Passband {} should be >> stopband {}",
            max_passband,
            max_stopband
        );
    }

    #[test]
    fn test_bandpass_response() {
        let sample_rate = 44100.0;
        let mut filter = BiquadFilter::new(sample_rate);
        filter.set_filter_type(FilterType::Bandpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(2.0);

        // Test center frequency (should pass)
        filter.reset();
        let center_freq = 1000.0;
        let mut max_center: f32 = 0.0;

        for i in 0..1000 {
            let input = (2.0 * PI * center_freq * i as f32 / sample_rate).sin();
            let output = filter.process(input);
            max_center = max_center.max(output.abs());
        }

        // Test low frequency (should reject)
        filter.reset();
        let low_freq = 100.0;
        let mut max_low: f32 = 0.0;

        for i in 0..1000 {
            let input = (2.0 * PI * low_freq * i as f32 / sample_rate).sin();
            let output = filter.process(input);
            max_low = max_low.max(output.abs());
        }

        // Test high frequency (should reject)
        filter.reset();
        let high_freq = 10000.0;
        let mut max_high: f32 = 0.0;

        for i in 0..1000 {
            let input = (2.0 * PI * high_freq * i as f32 / sample_rate).sin();
            let output = filter.process(input);
            max_high = max_high.max(output.abs());
        }

        // Center should be strongest
        assert!(max_center > max_low * 3.0);
        assert!(max_center > max_high * 3.0);
    }

    #[test]
    fn test_reset() {
        let mut filter = BiquadFilter::new(44100.0);

        // Process some samples
        for _ in 0..10 {
            filter.process(1.0);
        }

        // Reset
        filter.reset();

        assert_eq!(filter.x1, 0.0);
        assert_eq!(filter.x2, 0.0);
        assert_eq!(filter.y1, 0.0);
        assert_eq!(filter.y2, 0.0);
    }

    #[test]
    fn test_extreme_parameters_stability() {
        let mut filter = BiquadFilter::new(44100.0);

        // Try extreme combinations
        let test_cases = vec![
            (FilterType::Lowpass, 20.0, 10.0),
            (FilterType::Lowpass, 20000.0, 10.0),
            (FilterType::Highpass, 20.0, 10.0),
            (FilterType::Highpass, 20000.0, 10.0),
            (FilterType::Bandpass, 100.0, 10.0),
        ];

        for (ftype, cutoff, res) in test_cases {
            filter.set_filter_type(ftype);
            filter.set_cutoff(cutoff);
            filter.set_resonance(res);
            filter.reset();

            // Process sine wave - should not explode
            for i in 0..100 {
                let input = (2.0 * PI * 440.0 * i as f32 / 44100.0).sin();
                let output = filter.process(input);

                assert!(output.is_finite(), "Output should be finite");
                assert!(output.abs() < 100.0, "Output {} shouldn't explode", output);
            }
        }
    }
}
