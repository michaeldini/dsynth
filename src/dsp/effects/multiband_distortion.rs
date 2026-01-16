/// Multi-band distortion effect
///
/// Splits the audio into three frequency bands (bass/mid/high) and applies
/// independent saturation to each. This is essential for modern bass music
/// where you want "destroyed lows but clean highs" or vice versa.
///
/// # Architecture
/// - **Crossover filters**: 2nd-order Linkwitz-Riley (LR2) for phase-coherent band splitting
/// - **Three independent waveshapers**: Each band has its own drive and character
/// - **Band mixing**: Recombine with independent gain per band
///
/// # Why Multi-band?
/// Single-band distortion affects all frequencies equally. This means:
/// - Heavy bass distortion also makes highs harsh
/// - Trying to preserve highs means weak bass saturation
///
/// Multi-band solves this by processing each range independently:
/// - **Bass** (< 200Hz): Can be heavily saturated for "weight" without affecting clarity
/// - **Mids** (200Hz - 2kHz): The "body" of most sounds, moderate saturation
/// - **Highs** (> 2kHz): Often left cleaner for "air" and clarity
use std::f32::consts::PI;

/// Linkwitz-Riley 2nd order crossover filter pair
/// Produces flat magnitude response when low+high outputs are summed
struct LR2Crossover {
    // Low-pass state
    lp_x1: f32,
    lp_x2: f32,
    lp_y1: f32,
    lp_y2: f32,
    // High-pass state
    hp_x1: f32,
    hp_x2: f32,
    hp_y1: f32,
    hp_y2: f32,
    // Coefficients (shared for both LP and HP)
    b0_lp: f32,
    b1_lp: f32,
    b2_lp: f32,
    a1: f32,
    a2: f32,
    b0_hp: f32,
    b1_hp: f32,
    b2_hp: f32,
}

impl LR2Crossover {
    fn new(sample_rate: f32, crossover_freq: f32) -> Self {
        let mut xover = Self {
            lp_x1: 0.0,
            lp_x2: 0.0,
            lp_y1: 0.0,
            lp_y2: 0.0,
            hp_x1: 0.0,
            hp_x2: 0.0,
            hp_y1: 0.0,
            hp_y2: 0.0,
            b0_lp: 0.0,
            b1_lp: 0.0,
            b2_lp: 0.0,
            a1: 0.0,
            a2: 0.0,
            b0_hp: 0.0,
            b1_hp: 0.0,
            b2_hp: 0.0,
        };
        xover.set_frequency(sample_rate, crossover_freq);
        xover
    }

    fn set_frequency(&mut self, sample_rate: f32, freq: f32) {
        // Linkwitz-Riley 2nd order (Butterworth squared)
        let omega = 2.0 * PI * freq / sample_rate;
        let cos_omega = omega.cos();
        let sin_omega = omega.sin();
        let alpha = sin_omega / (2.0 * 0.7071); // Q = 0.7071 for Butterworth

        let a0 = 1.0 + alpha;

        // Low-pass coefficients
        self.b0_lp = ((1.0 - cos_omega) / 2.0) / a0;
        self.b1_lp = (1.0 - cos_omega) / a0;
        self.b2_lp = ((1.0 - cos_omega) / 2.0) / a0;

        // High-pass coefficients
        self.b0_hp = ((1.0 + cos_omega) / 2.0) / a0;
        self.b1_hp = -(1.0 + cos_omega) / a0;
        self.b2_hp = ((1.0 + cos_omega) / 2.0) / a0;

        // Shared denominator coefficients
        self.a1 = (-2.0 * cos_omega) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// Process input and return (low_band, high_band)
    fn process(&mut self, input: f32) -> (f32, f32) {
        // Low-pass
        let lp_out = self.b0_lp * input + self.b1_lp * self.lp_x1 + self.b2_lp * self.lp_x2
            - self.a1 * self.lp_y1
            - self.a2 * self.lp_y2;
        self.lp_x2 = self.lp_x1;
        self.lp_x1 = input;
        self.lp_y2 = self.lp_y1;
        self.lp_y1 = lp_out;

        // High-pass
        let hp_out = self.b0_hp * input + self.b1_hp * self.hp_x1 + self.b2_hp * self.hp_x2
            - self.a1 * self.hp_y1
            - self.a2 * self.hp_y2;
        self.hp_x2 = self.hp_x1;
        self.hp_x1 = input;
        self.hp_y2 = self.hp_y1;
        self.hp_y1 = hp_out;

        (lp_out, hp_out)
    }

    fn clear(&mut self) {
        self.lp_x1 = 0.0;
        self.lp_x2 = 0.0;
        self.lp_y1 = 0.0;
        self.lp_y2 = 0.0;
        self.hp_x1 = 0.0;
        self.hp_x2 = 0.0;
        self.hp_y1 = 0.0;
        self.hp_y2 = 0.0;
    }
}

/// Multi-band distortion processor with 3 independent bands and true stereo
pub struct MultibandDistortion {
    sample_rate: f32,

    // Two crossovers per channel: low/mid and mid/high
    xover_low_mid_l: LR2Crossover,
    xover_mid_high_l: LR2Crossover,
    xover_low_mid_r: LR2Crossover,
    xover_mid_high_r: LR2Crossover,

    // Crossover frequencies
    low_mid_freq: f32,
    mid_high_freq: f32,

    // Per-band drive (0.0 to 1.0, maps to 1x-100x gain)
    drive_low: f32,
    drive_mid: f32,
    drive_high: f32,

    // Per-band output gain (0.0 to 2.0, default 1.0)
    gain_low: f32,
    gain_mid: f32,
    gain_high: f32,

    // Global mix (wet/dry)
    mix: f32,

    // DC blocking filters per band per channel (x1, y1, coeff)
    dc_block_low_l: (f32, f32, f32),
    dc_block_mid_l: (f32, f32, f32),
    dc_block_high_l: (f32, f32, f32),
    dc_block_low_r: (f32, f32, f32),
    dc_block_mid_r: (f32, f32, f32),
    dc_block_high_r: (f32, f32, f32),
}

impl MultibandDistortion {
    /// Create a new multi-band distortion processor
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        // DC blocking filter coefficient (high-pass at ~10Hz)
        let cutoff = 10.0;
        let rc = 1.0 / (2.0 * PI * cutoff);
        let dt = 1.0 / sample_rate;
        let dc_coeff = rc / (rc + dt);

        Self {
            sample_rate,
            xover_low_mid_l: LR2Crossover::new(sample_rate, 200.0),
            xover_mid_high_l: LR2Crossover::new(sample_rate, 2000.0),
            xover_low_mid_r: LR2Crossover::new(sample_rate, 200.0),
            xover_mid_high_r: LR2Crossover::new(sample_rate, 2000.0),
            low_mid_freq: 200.0,
            mid_high_freq: 2000.0,
            drive_low: 0.0,
            drive_mid: 0.0,
            drive_high: 0.0,
            gain_low: 1.0,
            gain_mid: 1.0,
            gain_high: 1.0,
            mix: 0.5,
            dc_block_low_l: (0.0, 0.0, dc_coeff),
            dc_block_mid_l: (0.0, 0.0, dc_coeff),
            dc_block_high_l: (0.0, 0.0, dc_coeff),
            dc_block_low_r: (0.0, 0.0, dc_coeff),
            dc_block_mid_r: (0.0, 0.0, dc_coeff),
            dc_block_high_r: (0.0, 0.0, dc_coeff),
        }
    }

    /// Set low-mid crossover frequency (default 200Hz)
    pub fn set_low_mid_freq(&mut self, freq: f32) {
        let freq = freq.clamp(50.0, 500.0);
        if (self.low_mid_freq - freq).abs() > 1.0 {
            self.low_mid_freq = freq;
            self.xover_low_mid_l.set_frequency(self.sample_rate, freq);
            self.xover_low_mid_r.set_frequency(self.sample_rate, freq);
        }
    }

    /// Set mid-high crossover frequency (default 2000Hz)
    pub fn set_mid_high_freq(&mut self, freq: f32) {
        let freq = freq.clamp(1000.0, 8000.0);
        if (self.mid_high_freq - freq).abs() > 1.0 {
            self.mid_high_freq = freq;
            self.xover_mid_high_l.set_frequency(self.sample_rate, freq);
            self.xover_mid_high_r.set_frequency(self.sample_rate, freq);
        }
    }

    /// Set drive for bass band (0.0 to 1.0, maps to 1x-100x gain)
    pub fn set_drive_low(&mut self, drive: f32) {
        self.drive_low = drive.clamp(0.0, 1.0);
    }

    /// Set drive for mid band (0.0 to 1.0, maps to 1x-100x gain)
    pub fn set_drive_mid(&mut self, drive: f32) {
        self.drive_mid = drive.clamp(0.0, 1.0);
    }

    /// Set drive for high band (0.0 to 1.0, maps to 1x-100x gain)
    pub fn set_drive_high(&mut self, drive: f32) {
        self.drive_high = drive.clamp(0.0, 1.0);
    }

    /// Set output gain for bass band (0.0 to 2.0)
    pub fn set_gain_low(&mut self, gain: f32) {
        self.gain_low = gain.clamp(0.0, 2.0);
    }

    /// Set output gain for mid band (0.0 to 2.0)
    pub fn set_gain_mid(&mut self, gain: f32) {
        self.gain_mid = gain.clamp(0.0, 2.0);
    }

    /// Set output gain for high band (0.0 to 2.0)
    pub fn set_gain_high(&mut self, gain: f32) {
        self.gain_high = gain.clamp(0.0, 2.0);
    }

    /// Set wet/dry mix (0.0 = dry, 1.0 = full wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Apply tanh saturation with drive
    #[inline]
    fn saturate(input: f32, drive: f32) -> f32 {
        // Map drive (0.0 to 1.0) to gain (1.0 to 100.0)
        let gain = 1.0 + drive * 99.0;
        let x = input * gain;
        let saturated = x.tanh();
        // Compensate for gain increase
        saturated / (1.0 + drive * 0.5)
    }

    /// DC blocking filter
    #[inline]
    fn dc_block(state: &mut (f32, f32, f32), input: f32) -> f32 {
        let (x1, y1, coeff) = state;
        let output = *coeff * (*y1 + input - *x1);
        *x1 = input;
        *y1 = output;
        output
    }

    /// Process a single mono sample (uses left channel state)
    ///
    /// **Note**: For true stereo processing, use `process_stereo()` which maintains
    /// separate crossover and DC blocking state for each channel.
    pub fn process(&mut self, input: f32) -> f32 {
        // Split into low and (mid+high)
        let (low, mid_high) = self.xover_low_mid_l.process(input);

        // Split mid+high into mid and high
        let (mid, high) = self.xover_mid_high_l.process(mid_high);

        // Apply saturation to each band
        let low_sat = Self::saturate(low, self.drive_low);
        let mid_sat = Self::saturate(mid, self.drive_mid);
        let high_sat = Self::saturate(high, self.drive_high);

        // DC block each band
        let low_clean = Self::dc_block(&mut self.dc_block_low_l, low_sat);
        let mid_clean = Self::dc_block(&mut self.dc_block_mid_l, mid_sat);
        let high_clean = Self::dc_block(&mut self.dc_block_high_l, high_sat);

        // Apply per-band gain and sum
        let wet =
            low_clean * self.gain_low + mid_clean * self.gain_mid + high_clean * self.gain_high;

        // Mix wet and dry
        input * (1.0 - self.mix) + wet * self.mix
    }

    /// Process a stereo sample pair with independent L/R crossovers and DC blocking
    ///
    /// **True Stereo**: Each channel has independent crossover filters and DC blocking,
    /// preserving stereo imaging and frequency-dependent spatial information.
    pub fn process_stereo(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        // Process left channel
        let (low_l, mid_high_l) = self.xover_low_mid_l.process(input_l);
        let (mid_l, high_l) = self.xover_mid_high_l.process(mid_high_l);

        let low_sat_l = Self::saturate(low_l, self.drive_low);
        let mid_sat_l = Self::saturate(mid_l, self.drive_mid);
        let high_sat_l = Self::saturate(high_l, self.drive_high);

        let low_clean_l = Self::dc_block(&mut self.dc_block_low_l, low_sat_l);
        let mid_clean_l = Self::dc_block(&mut self.dc_block_mid_l, mid_sat_l);
        let high_clean_l = Self::dc_block(&mut self.dc_block_high_l, high_sat_l);

        let wet_l = low_clean_l * self.gain_low
            + mid_clean_l * self.gain_mid
            + high_clean_l * self.gain_high;
        let out_l = input_l * (1.0 - self.mix) + wet_l * self.mix;

        // Process right channel
        let (low_r, mid_high_r) = self.xover_low_mid_r.process(input_r);
        let (mid_r, high_r) = self.xover_mid_high_r.process(mid_high_r);

        let low_sat_r = Self::saturate(low_r, self.drive_low);
        let mid_sat_r = Self::saturate(mid_r, self.drive_mid);
        let high_sat_r = Self::saturate(high_r, self.drive_high);

        let low_clean_r = Self::dc_block(&mut self.dc_block_low_r, low_sat_r);
        let mid_clean_r = Self::dc_block(&mut self.dc_block_mid_r, mid_sat_r);
        let high_clean_r = Self::dc_block(&mut self.dc_block_high_r, high_sat_r);

        let wet_r = low_clean_r * self.gain_low
            + mid_clean_r * self.gain_mid
            + high_clean_r * self.gain_high;
        let out_r = input_r * (1.0 - self.mix) + wet_r * self.mix;

        (out_l, out_r)
    }

    /// Clear all filter state for both channels
    pub fn clear(&mut self) {
        self.xover_low_mid_l.clear();
        self.xover_mid_high_l.clear();
        self.xover_low_mid_r.clear();
        self.xover_mid_high_r.clear();
        self.dc_block_low_l = (0.0, 0.0, self.dc_block_low_l.2);
        self.dc_block_mid_l = (0.0, 0.0, self.dc_block_mid_l.2);
        self.dc_block_high_l = (0.0, 0.0, self.dc_block_high_l.2);
        self.dc_block_low_r = (0.0, 0.0, self.dc_block_low_r.2);
        self.dc_block_mid_r = (0.0, 0.0, self.dc_block_mid_r.2);
        self.dc_block_high_r = (0.0, 0.0, self.dc_block_high_r.2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_multiband_creation() {
        let mb = MultibandDistortion::new(44100.0);
        assert_eq!(mb.low_mid_freq, 200.0);
        assert_eq!(mb.mid_high_freq, 2000.0);
        assert_eq!(mb.drive_low, 0.0);
        assert_eq!(mb.drive_mid, 0.0);
        assert_eq!(mb.drive_high, 0.0);
        assert_eq!(mb.mix, 0.5);
    }

    #[test]
    fn test_multiband_dry_passthrough() {
        let mut mb = MultibandDistortion::new(44100.0);
        mb.set_mix(0.0); // Full dry

        // With mix at 0, output should equal input
        let input = 0.5;
        let output = mb.process(input);
        assert_relative_eq!(output, input, epsilon = 0.001);
    }

    #[test]
    fn test_multiband_parameter_clamping() {
        let mut mb = MultibandDistortion::new(44100.0);

        mb.set_drive_low(2.0);
        assert_eq!(mb.drive_low, 1.0);

        mb.set_drive_mid(-1.0);
        assert_eq!(mb.drive_mid, 0.0);

        mb.set_gain_low(5.0);
        assert_eq!(mb.gain_low, 2.0);

        mb.set_low_mid_freq(10.0);
        assert_eq!(mb.low_mid_freq, 50.0);

        mb.set_mid_high_freq(20000.0);
        assert_eq!(mb.mid_high_freq, 8000.0);
    }

    #[test]
    fn test_multiband_saturation() {
        let mut mb = MultibandDistortion::new(44100.0);
        mb.set_mix(1.0);
        mb.set_drive_low(1.0); // Full drive for strong saturation
        mb.set_drive_mid(1.0);
        mb.set_drive_high(1.0);

        // Process some samples to warm up filters
        for _ in 0..1000 {
            mb.process(0.5);
        }

        // With full saturation (drive=1.0 = 100x gain), tanh should compress heavily
        // Input of 0.9 * 100 = 90, tanh(90) ≈ 1.0, then compensated
        // The output should be finite and not explode
        let output = mb.process(0.9);
        assert!(output.is_finite());
        assert!(output.abs() < 5.0); // Should be bounded by saturation
    }

    #[test]
    fn test_multiband_stability() {
        let mut mb = MultibandDistortion::new(44100.0);
        mb.set_mix(1.0);
        mb.set_drive_low(1.0);
        mb.set_drive_mid(1.0);
        mb.set_drive_high(1.0);

        // Process many samples with extreme settings
        for _ in 0..10000 {
            let output = mb.process(0.9);
            assert!(output.is_finite());
            assert!(output.abs() < 10.0);
        }
    }

    #[test]
    fn test_multiband_stereo() {
        let mut mb = MultibandDistortion::new(44100.0);
        mb.set_mix(0.0);

        let (out_l, out_r) = mb.process_stereo(0.5, -0.3);
        assert_relative_eq!(out_l, 0.5, epsilon = 0.001);
        assert_relative_eq!(out_r, -0.3, epsilon = 0.001);
    }

    #[test]
    fn test_multiband_clear() {
        let mut mb = MultibandDistortion::new(44100.0);

        // Process some samples
        for _ in 0..100 {
            mb.process(0.5);
        }

        // Clear state
        mb.clear();

        // DC block state should be cleared (check both channels)
        assert_eq!(mb.dc_block_low_l.0, 0.0);
        assert_eq!(mb.dc_block_mid_l.0, 0.0);
        assert_eq!(mb.dc_block_high_l.0, 0.0);
        assert_eq!(mb.dc_block_low_r.0, 0.0);
        assert_eq!(mb.dc_block_mid_r.0, 0.0);
        assert_eq!(mb.dc_block_high_r.0, 0.0);
    }
}
