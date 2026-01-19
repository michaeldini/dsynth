use crate::params::FilterType;
use std::f32::consts::PI;

/// Simplified biquad filter implementation
/// Implements lowpass, highpass, and bandpass filters using Audio EQ Cookbook formulas
pub struct BiquadFilter {
    sample_rate: f32,
    filter_type: FilterType,
    cutoff: f32,
    target_cutoff: f32,
    resonance: f32,
    bandwidth: f32, // Bandwidth in octaves for bandpass filter
    gain_db: f32,   // Gain in dB for peaking/shelf filters

    /// Throttle for expensive coefficient updates when cutoff is modulated at audio-rate.
    ///
    /// Updating biquad coefficients requires `sin`/`cos` and is relatively expensive.
    /// In DSynth we frequently modulate cutoff per-sample (LFO, envelope, key tracking),
    /// so we update coefficients at a small fixed interval and treat intermediate samples
    /// as holding the last coefficients.
    ///
    /// Default: every 4 samples (~0.09ms at 44.1kHz). This is typically inaudible but
    /// reduces CPU cost significantly when cutoff changes continuously.
    cutoff_update_interval: u8,
    cutoff_update_counter: u8,

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
    /// Flush denormals to zero to prevent CPU performance degradation
    #[inline(always)]
    fn flush_denormal(x: f32) -> f32 {
        if x.abs() < 1e-20 {
            0.0
        } else {
            x
        }
    }

    /// Create a new biquad filter
    pub fn new(sample_rate: f32) -> Self {
        const DEFAULT_CUTOFF_UPDATE_INTERVAL: u8 = 4;
        let mut filter = Self {
            sample_rate,
            filter_type: FilterType::Lowpass,
            cutoff: 1000.0,
            target_cutoff: 1000.0,
            resonance: 0.707,
            bandwidth: 1.0, // 1 octave default for bandpass
            gain_db: 0.0,   // 0dB default for peaking/shelf

            cutoff_update_interval: DEFAULT_CUTOFF_UPDATE_INTERVAL,
            cutoff_update_counter: 0,

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

    /// Set how often cutoff coefficient updates are allowed to run.
    ///
    /// - `1` means update every call (highest accuracy, highest CPU)
    /// - Larger values reduce CPU for audio-rate cutoff modulation
    pub fn set_cutoff_update_interval(&mut self, interval: u8) {
        self.cutoff_update_interval = interval.max(1);
        self.cutoff_update_counter = 0;
        // Apply any pending target immediately when tightening the interval.
        if (self.cutoff - self.target_cutoff).abs() > 0.01 {
            self.cutoff = self.target_cutoff;
            self.update_coefficients();
        }
    }

    /// Set filter type
    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        if self.filter_type != filter_type {
            self.filter_type = filter_type;
            self.cutoff_update_counter = 0;
            self.update_coefficients();
        }
    }

    /// Set cutoff frequency in Hz
    pub fn set_cutoff(&mut self, cutoff: f32) {
        let clamped = cutoff.clamp(20.0, self.sample_rate * 0.49);

        if (self.target_cutoff - clamped).abs() <= 0.01 {
            return;
        }
        self.target_cutoff = clamped;

        // If the cutoff jump is large, update immediately to keep UI/automation responsive.
        // For small continuous modulation, rate-limit expensive coefficient recomputation.
        let large_jump = (self.cutoff - self.target_cutoff).abs() > 100.0;

        if self.cutoff_update_interval == 1 || large_jump {
            self.cutoff = self.target_cutoff;
            self.cutoff_update_counter = 0;
            self.update_coefficients();
            return;
        }

        self.cutoff_update_counter = self.cutoff_update_counter.saturating_add(1);
        if self.cutoff_update_counter >= self.cutoff_update_interval {
            self.cutoff_update_counter = 0;
            self.cutoff = self.target_cutoff;
            self.update_coefficients();
        }
    }

    /// Set resonance (Q factor)
    pub fn set_resonance(&mut self, resonance: f32) {
        let clamped = resonance.clamp(0.5, 50.0);
        if (self.resonance - clamped).abs() > 0.01 {
            self.resonance = clamped;
            self.cutoff_update_counter = 0;
            self.update_coefficients();
        }
    }

    /// Set bandwidth in octaves (for bandpass filter)
    pub fn set_bandwidth(&mut self, bandwidth: f32) {
        let clamped = bandwidth.clamp(0.1, 4.0);
        if (self.bandwidth - clamped).abs() > 0.01 {
            self.bandwidth = clamped;
            if self.filter_type == FilterType::Bandpass {
                self.cutoff_update_counter = 0;
                self.update_coefficients();
            }
        }
    }

    /// Set gain in dB (for peaking/shelf filters)
    pub fn set_gain_db(&mut self, gain_db: f32) {
        let clamped = gain_db.clamp(-24.0, 24.0);
        if (self.gain_db - clamped).abs() > 0.01 {
            self.gain_db = clamped;
            if matches!(
                self.filter_type,
                FilterType::Peaking | FilterType::LowShelf | FilterType::HighShelf
            ) {
                self.cutoff_update_counter = 0;
                self.update_coefficients();
            }
        }
    }

    /// Update biquad coefficients based on current parameters
    fn update_coefficients(&mut self) {
        let omega = 2.0 * PI * self.cutoff / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();

        let (mut b0, mut b1, mut b2, a0, mut a1, mut a2) = match self.filter_type {
            FilterType::Lowpass => {
                // Standard lowpass using Q
                let alpha = sin_omega / (2.0 * self.resonance);
                let b1_temp = 1.0 - cos_omega;
                let b0_temp = b1_temp / 2.0;
                let b2_temp = b0_temp;
                let a0_temp = 1.0 + alpha;
                let a1_temp = -2.0 * cos_omega;
                let a2_temp = 1.0 - alpha;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
            FilterType::Highpass => {
                // Standard highpass using Q
                let alpha = sin_omega / (2.0 * self.resonance);
                let b0_temp = (1.0 + cos_omega) / 2.0;
                let b1_temp = -(1.0 + cos_omega);
                let b2_temp = (1.0 + cos_omega) / 2.0;
                let a0_temp = 1.0 + alpha;
                let a1_temp = -2.0 * cos_omega;
                let a2_temp = 1.0 - alpha;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
            FilterType::Peaking => {
                // Peaking EQ (bell filter) - Audio EQ Cookbook formula
                let a_coef = 10.0_f32.powf(self.gain_db / 40.0); // Square root of linear gain
                let alpha = sin_omega / (2.0 * self.resonance);
                let b0_temp = 1.0 + alpha * a_coef;
                let b1_temp = -2.0 * cos_omega;
                let b2_temp = 1.0 - alpha * a_coef;
                let a0_temp = 1.0 + alpha / a_coef;
                let a1_temp = -2.0 * cos_omega;
                let a2_temp = 1.0 - alpha / a_coef;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
            FilterType::LowShelf => {
                // Low shelf - Audio EQ Cookbook formula
                let a_coef = 10.0_f32.powf(self.gain_db / 40.0);
                let sqrt_arg = (a_coef + 1.0 / a_coef) * (1.0 / self.resonance - 1.0) + 2.0;
                // Safety: if sqrt argument is negative, return fallback values
                let alpha = if sqrt_arg < 0.0 {
                    0.0 // Fallback to no filtering
                } else {
                    sin_omega / 2.0 * sqrt_arg.sqrt()
                };
                let b0_temp = a_coef
                    * ((a_coef + 1.0) - (a_coef - 1.0) * cos_omega + 2.0 * a_coef.sqrt() * alpha);
                let b1_temp = 2.0 * a_coef * ((a_coef - 1.0) - (a_coef + 1.0) * cos_omega);
                let b2_temp = a_coef
                    * ((a_coef + 1.0) - (a_coef - 1.0) * cos_omega - 2.0 * a_coef.sqrt() * alpha);
                let a0_temp =
                    (a_coef + 1.0) + (a_coef - 1.0) * cos_omega + 2.0 * a_coef.sqrt() * alpha;
                let a1_temp = -2.0 * ((a_coef - 1.0) + (a_coef + 1.0) * cos_omega);
                let a2_temp =
                    (a_coef + 1.0) + (a_coef - 1.0) * cos_omega - 2.0 * a_coef.sqrt() * alpha;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
            FilterType::HighShelf => {
                // High shelf - Audio EQ Cookbook formula
                let a_coef = 10.0_f32.powf(self.gain_db / 40.0);
                let sqrt_arg = (a_coef + 1.0 / a_coef) * (1.0 / self.resonance - 1.0) + 2.0;
                // Safety: if sqrt argument is negative, return fallback values
                let alpha = if sqrt_arg < 0.0 {
                    0.0 // Fallback to no filtering
                } else {
                    sin_omega / 2.0 * sqrt_arg.sqrt()
                };
                let b0_temp = a_coef
                    * ((a_coef + 1.0) + (a_coef - 1.0) * cos_omega + 2.0 * a_coef.sqrt() * alpha);
                let b1_temp = -2.0 * a_coef * ((a_coef - 1.0) + (a_coef + 1.0) * cos_omega);
                let b2_temp = a_coef
                    * ((a_coef + 1.0) + (a_coef - 1.0) * cos_omega - 2.0 * a_coef.sqrt() * alpha);
                let a0_temp =
                    (a_coef + 1.0) - (a_coef - 1.0) * cos_omega + 2.0 * a_coef.sqrt() * alpha;
                let a1_temp = 2.0 * ((a_coef - 1.0) - (a_coef + 1.0) * cos_omega);
                let a2_temp =
                    (a_coef + 1.0) - (a_coef - 1.0) * cos_omega - 2.0 * a_coef.sqrt() * alpha;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
            FilterType::Bandpass => {
                // Bandpass using bandwidth in octaves (constant skirt gain)
                let bw = self.bandwidth;
                let alpha = sin_omega * ((2.0_f32.ln() / 2.0) * bw * omega / sin_omega).sinh();
                let b0_temp = sin_omega / 2.0; // or alpha for constant peak gain
                let b1_temp = 0.0;
                let b2_temp = -sin_omega / 2.0; // or -alpha
                let a0_temp = 1.0 + alpha;
                let a1_temp = -2.0 * cos_omega;
                let a2_temp = 1.0 - alpha;
                (b0_temp, b1_temp, b2_temp, a0_temp, a1_temp, a2_temp)
            }
        };

        // Normalize by a0 (with safety check for division by zero)
        if a0.abs() < 1e-10 || !a0.is_finite() {
            // Invalid a0 - use bypass coefficients to prevent NaN/Inf
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.b2 = 0.0;
            self.a1 = 0.0;
            self.a2 = 0.0;
            return;
        }

        b0 /= a0;
        b1 /= a0;
        b2 /= a0;
        a1 /= a0;
        a2 /= a0;

        // Final safety: if any coefficient is NaN/Inf, use bypass
        if !b0.is_finite() || !b1.is_finite() || !b2.is_finite()
            || !a1.is_finite() || !a2.is_finite()
        {
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.b2 = 0.0;
            self.a1 = 0.0;
            self.a2 = 0.0;
            return;
        }

        self.b0 = b0;
        self.b1 = b1;
        self.b2 = b2;
        self.a1 = a1;
        self.a2 = a2;
    }

    /// Process one sample through the filter
    pub fn process(&mut self, input: f32) -> f32 {
        // Direct Form I implementation
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // Update state with denormal flushing
        self.x2 = self.x1;
        self.x1 = Self::flush_denormal(input);
        self.y2 = self.y1;
        self.y1 = Self::flush_denormal(output);

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
        filter.set_cutoff_update_interval(1);

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
        filter.set_cutoff_update_interval(1);

        filter.set_resonance(0.1);
        assert_eq!(filter.resonance, 0.5);

        filter.set_resonance(100.0);
        assert_eq!(filter.resonance, 50.0);

        filter.set_resonance(2.0);
        assert_eq!(filter.resonance, 2.0);
    }

    #[test]
    fn test_bandwidth_clamping() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_cutoff_update_interval(1);

        filter.set_bandwidth(0.01);
        assert_eq!(filter.bandwidth, 0.1);

        filter.set_bandwidth(10.0);
        assert_eq!(filter.bandwidth, 4.0);

        filter.set_bandwidth(2.0);
        assert_eq!(filter.bandwidth, 2.0);
    }

    #[test]
    fn test_coefficient_stability() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_cutoff_update_interval(1);
        filter.set_cutoff(20000.0);
        filter.set_resonance(50.0);

        // Coefficients should be finite
        assert!(filter.a1.is_finite());
        assert!(filter.a2.is_finite());
        assert!(filter.b0.is_finite());
        assert!(filter.b1.is_finite());
        assert!(filter.b2.is_finite());
    }

    #[test]
    fn test_lowpass_dc_signal() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_cutoff_update_interval(1);
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
