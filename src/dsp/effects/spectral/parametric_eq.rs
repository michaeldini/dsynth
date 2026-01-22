/// Parametric EQ with 4 bands
///
/// Professional-quality 4-band parametric equalizer for vocal shaping.
/// Each band has independent frequency, gain, and Q controls.
///
/// # Band Configuration
/// - **Band 1**: Low shelf (typically 80Hz) - control low-end rumble/body
/// - **Band 2**: Peaking bell (typically 400Hz) - shape lower mids
/// - **Band 3**: Peaking bell (typically 3kHz) - control presence/clarity
/// - **Band 4**: High shelf (typically 8kHz) - adjust air/brightness
use crate::dsp::filters::filter::BiquadFilter;
use crate::params::FilterType;

/// Number of EQ bands
const NUM_BANDS: usize = 4;

/// Single EQ band with full parameter control
pub struct EQBand {
    filter: BiquadFilter,
    enabled: bool,
    frequency: f32,
    gain_db: f32,
    q_factor: f32,
    band_type: FilterType,
}

impl EQBand {
    /// Create a new EQ band
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `band_type` - Filter type (Peaking, LowShelf, or HighShelf)
    /// * `frequency` - Center/corner frequency in Hz
    /// * `gain_db` - Gain/boost in dB (±12dB typical)
    /// * `q_factor` - Q/bandwidth (0.1-10, where 1.0 is moderate width)
    pub fn new(
        sample_rate: f32,
        band_type: FilterType,
        frequency: f32,
        gain_db: f32,
        q_factor: f32,
    ) -> Self {
        let mut filter = BiquadFilter::new(sample_rate);
        filter.set_filter_type(band_type);
        filter.set_cutoff(frequency);
        filter.set_gain_db(gain_db);
        filter.set_resonance(q_factor);

        Self {
            filter,
            enabled: true,
            frequency,
            gain_db,
            q_factor,
            band_type,
        }
    }

    /// Enable or disable this band
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set center/corner frequency
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.clamp(20.0, 20000.0);
        self.filter.set_cutoff(self.frequency);
    }

    /// Set gain in dB
    pub fn set_gain_db(&mut self, gain_db: f32) {
        self.gain_db = gain_db.clamp(-12.0, 12.0);
        self.filter.set_gain_db(self.gain_db);
    }

    /// Set Q factor (bandwidth)
    pub fn set_q_factor(&mut self, q_factor: f32) {
        self.q_factor = q_factor.clamp(0.1, 10.0);
        self.filter.set_resonance(self.q_factor);
    }

    /// Process one sample through this band
    pub fn process(&mut self, input: f32) -> f32 {
        if self.enabled {
            self.filter.process(input)
        } else {
            input
        }
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.filter.reset();
        self.filter.set_filter_type(self.band_type);
    }
}

/// 4-band parametric equalizer
pub struct ParametricEQ {
    bands: [EQBand; NUM_BANDS],
}

impl ParametricEQ {
    /// Create a new 4-band parametric EQ with default vocal settings
    ///
    /// # Default Configuration
    /// - Band 1: Low shelf @ 80Hz, 0dB, Q=1.0
    /// - Band 2: Bell @ 400Hz, 0dB, Q=1.0 (lower mids)
    /// - Band 3: Bell @ 3kHz, 0dB, Q=1.0 (presence)
    /// - Band 4: High shelf @ 8kHz, 0dB, Q=1.0 (air)
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        Self {
            bands: [
                EQBand::new(sample_rate, FilterType::LowShelf, 80.0, 0.0, 1.0),
                EQBand::new(sample_rate, FilterType::Peaking, 400.0, 0.0, 1.0),
                EQBand::new(sample_rate, FilterType::Peaking, 3000.0, 0.0, 1.0),
                EQBand::new(sample_rate, FilterType::HighShelf, 8000.0, 0.0, 1.0),
            ],
        }
    }

    /// Get mutable reference to a specific band
    ///
    /// # Arguments
    /// * `band_index` - Band number (0-3)
    pub fn get_band_mut(&mut self, band_index: usize) -> Option<&mut EQBand> {
        if band_index < NUM_BANDS {
            Some(&mut self.bands[band_index])
        } else {
            None
        }
    }

    /// Set parameters for a specific band
    ///
    /// # Arguments
    /// * `band_index` - Band number (0-3)
    /// * `frequency` - Center/corner frequency in Hz
    /// * `gain_db` - Gain in dB (±12dB)
    /// * `q_factor` - Q/bandwidth (0.1-10)
    pub fn set_band(&mut self, band_index: usize, frequency: f32, gain_db: f32, q_factor: f32) {
        if let Some(band) = self.get_band_mut(band_index) {
            band.set_frequency(frequency);
            band.set_gain_db(gain_db);
            band.set_q_factor(q_factor);
        }
    }

    /// Enable/disable a specific band
    pub fn set_band_enabled(&mut self, band_index: usize, enabled: bool) {
        if let Some(band) = self.get_band_mut(band_index) {
            band.set_enabled(enabled);
        }
    }

    /// Process one stereo sample pair through all bands
    ///
    /// Bands are processed in series (cascaded filters).
    ///
    /// # Arguments
    /// * `left` - Left channel input
    /// * `right` - Right channel input
    ///
    /// # Returns
    /// Tuple of (left, right) output samples
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let mut out_left = left;
        let mut out_right = right;

        // Process through all bands in series
        for band in &mut self.bands {
            out_left = band.process(out_left);
            out_right = band.process(out_right);
        }

        (out_left, out_right)
    }

    /// Reset all band states
    pub fn reset(&mut self) {
        for band in &mut self.bands {
            band.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_parametric_eq_creation() {
        let eq = ParametricEQ::new(44100.0);
        assert_eq!(eq.bands.len(), NUM_BANDS);
    }

    #[test]
    fn test_band_default_frequencies() {
        let eq = ParametricEQ::new(44100.0);

        // Check default frequencies
        assert_eq!(eq.bands[0].frequency, 80.0);
        assert_eq!(eq.bands[1].frequency, 400.0);
        assert_eq!(eq.bands[2].frequency, 3000.0);
        assert_eq!(eq.bands[3].frequency, 8000.0);
    }

    #[test]
    fn test_band_default_types() {
        let eq = ParametricEQ::new(44100.0);

        // Check filter types
        assert_eq!(eq.bands[0].band_type, FilterType::LowShelf);
        assert_eq!(eq.bands[1].band_type, FilterType::Peaking);
        assert_eq!(eq.bands[2].band_type, FilterType::Peaking);
        assert_eq!(eq.bands[3].band_type, FilterType::HighShelf);
    }

    #[test]
    fn test_set_band_parameters() {
        let mut eq = ParametricEQ::new(44100.0);

        eq.set_band(1, 500.0, 3.0, 2.0);

        assert_eq!(eq.bands[1].frequency, 500.0);
        assert_eq!(eq.bands[1].gain_db, 3.0);
        assert_eq!(eq.bands[1].q_factor, 2.0);
    }

    #[test]
    fn test_band_enable_disable() {
        let mut eq = ParametricEQ::new(44100.0);

        // Disable band 0
        eq.set_band_enabled(0, false);
        assert!(!eq.bands[0].enabled);

        // Enable it again
        eq.set_band_enabled(0, true);
        assert!(eq.bands[0].enabled);
    }

    #[test]
    fn test_process_flat_eq() {
        let mut eq = ParametricEQ::new(44100.0);

        // All bands at 0dB = should pass signal through unchanged
        let input = 0.5;
        let (left, right) = eq.process(input, input);

        // With 0dB gain, output should be close to input
        // (some phase shift is expected from filters)
        assert_relative_eq!(left, input, epsilon = 0.01);
        assert_relative_eq!(right, input, epsilon = 0.01);
    }

    #[test]
    fn test_process_boost() {
        let mut eq = ParametricEQ::new(44100.0);
        let sample_rate = 44100.0;

        // Boost mid frequencies significantly
        eq.set_band(2, 3000.0, 6.0, 1.0);

        // Generate a 3kHz sine wave and let filter settle
        let freq = 3000.0;
        let mut max_output: f32 = 0.0;

        for i in 0..5000 {
            // Run longer to allow filter settling
            let t = i as f32 / sample_rate;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.1;
            let (left, _) = eq.process(input, input);
            if i > 1000 {
                // Skip initial settling time
                max_output = max_output.max(left.abs());
            }
        }

        // With +6dB boost, output should be roughly 2× input (6dB ≈ ×2)
        // Allow tolerance for filter settling and frequency response
        assert!(
            max_output > 0.12,
            "Boost should increase signal amplitude (got {})",
            max_output
        );
    }

    #[test]
    fn test_process_cut() {
        let mut eq = ParametricEQ::new(44100.0);
        let sample_rate = 44100.0;

        // Cut high frequencies
        eq.set_band(3, 8000.0, -6.0, 1.0);

        // Generate an 8kHz sine wave and let filter settle
        let freq = 8000.0;
        let mut max_output: f32 = 0.0;

        for i in 0..5000 {
            // Run longer to allow filter settling
            let t = i as f32 / sample_rate;
            let input = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.1;
            let (left, _) = eq.process(input, input);
            if i > 1000 {
                // Skip initial settling time
                max_output = max_output.max(left.abs());
            }
        }

        // With -6dB cut, output should be roughly 0.5× input
        assert!(
            max_output < 0.095,
            "Cut should reduce signal amplitude (got {})",
            max_output
        );
    }

    #[test]
    fn test_disabled_band_bypass() {
        let mut eq = ParametricEQ::new(44100.0);

        // Set band 1 to extreme boost
        eq.set_band(1, 400.0, 12.0, 10.0);

        // Disable it
        eq.set_band_enabled(1, false);

        let input = 0.5;
        let (left, _) = eq.process(input, input);

        // Disabled band should not boost
        // (other bands are flat so output ≈ input)
        assert_relative_eq!(left, input, epsilon = 0.01);
    }

    #[test]
    fn test_frequency_clamping() {
        let mut eq = ParametricEQ::new(44100.0);

        // Try to set frequency out of range
        eq.set_band(0, 10.0, 0.0, 1.0);
        assert_eq!(eq.bands[0].frequency, 20.0); // Clamped to min

        eq.set_band(0, 30000.0, 0.0, 1.0);
        assert_eq!(eq.bands[0].frequency, 20000.0); // Clamped to max
    }

    #[test]
    fn test_gain_clamping() {
        let mut eq = ParametricEQ::new(44100.0);

        eq.set_band(1, 400.0, -20.0, 1.0);
        assert_eq!(eq.bands[1].gain_db, -12.0); // Clamped to min

        eq.set_band(1, 400.0, 20.0, 1.0);
        assert_eq!(eq.bands[1].gain_db, 12.0); // Clamped to max
    }

    #[test]
    fn test_q_clamping() {
        let mut eq = ParametricEQ::new(44100.0);

        eq.set_band(2, 3000.0, 0.0, 0.01);
        assert_eq!(eq.bands[2].q_factor, 0.1); // Clamped to min

        eq.set_band(2, 3000.0, 0.0, 100.0);
        assert_eq!(eq.bands[2].q_factor, 10.0); // Clamped to max
    }

    #[test]
    fn test_reset() {
        let mut eq = ParametricEQ::new(44100.0);

        // Process some signal to build up filter state
        for _ in 0..1000 {
            eq.process(0.5, 0.5);
        }

        eq.reset();

        // After reset, processing DC should give DC output
        let (left, right) = eq.process(1.0, 1.0);
        assert_relative_eq!(left, 1.0, epsilon = 0.01);
        assert_relative_eq!(right, 1.0, epsilon = 0.01);
    }

    #[test]
    fn test_stereo_processing() {
        let mut eq = ParametricEQ::new(44100.0);

        // Different L/R inputs should produce different outputs
        let (left_out, right_out) = eq.process(0.3, 0.7);

        // Outputs should be different (stereo processing)
        assert_ne!(left_out, right_out);
    }
}
