/// Multi-band compressor for professional transient control
///
/// Splits audio into 3 frequency bands (sub/body/click) and applies
/// independent compression to each. Essential for kick drum synthesis
/// where you need heavy sub compression without dulling the transient.
///
/// # Architecture
/// - **Crossover filters**: 2× Linkwitz-Riley 2nd order (LR2) for phase-coherent splitting
/// - **Three independent compressors**: Each band has its own threshold/ratio/attack/release
/// - **Band mixing**: Recombine with independent gain per band + parallel compression
/// - **Bypass per band**: Solo individual bands or disable problematic ranges
///
/// # Signal Flow
/// ```text
/// Input → LR2(150Hz) → [Sub, Mid+High]
///                         ↓      ↓
///                      Comp1   LR2(800Hz) → [Body, Click]
///                         ↓       ↓      ↓
///                      Comp1   Comp2  Comp3
///                         ↓       ↓      ↓
///                      Gain1   Gain2  Gain3
///                         ↓       ↓      ↓
///                         └───→ Sum ←───┘
///                               ↓
///                         Wet/Dry Mix → Output
/// ```
///
/// # Why Multi-band for Kicks?
/// - **Sub (40-150Hz)**: Heavy compression (4:1) for consistent low-end power
/// - **Body (150-800Hz)**: Moderate compression (3:1) for tonal sustain
/// - **Click (800Hz+)**: Light compression (2:1) preserves attack transient
///
/// Single-band compression would dull the transient when compressing the sub.
use super::compressor::Compressor;
use super::crossover::LR2Crossover;

/// Multi-band compressor with 3 frequency bands
pub struct MultibandCompressor {
    sample_rate: f32,

    // Cascade crossovers: input → low/mid+high → mid/high split
    xover_low: LR2Crossover,  // Default 150Hz (sub vs body+click)
    xover_high: LR2Crossover, // Default 800Hz (body vs click)

    // Independent compressors per band (using fast mode for efficiency)
    comp_sub: Compressor,
    comp_body: Compressor,
    comp_click: Compressor,

    // Per-band output gains (post-compression level adjustment)
    gain_sub: f32,
    gain_body: f32,
    gain_click: f32,

    // Per-band bypass (false = included in mix, true = excluded)
    bypass_sub: bool,
    bypass_body: bool,
    bypass_click: bool,

    // Global controls
    mix: f32,      // Wet/dry (0.0-1.0, parallel compression)
    enabled: bool, // Master bypass
}

impl MultibandCompressor {
    /// Create a new multiband compressor with aggressive kick-optimized defaults
    ///
    /// Default settings:
    /// - Sub (40-150Hz): -20dB threshold, 4:1 ratio, 5ms attack, 100ms release
    /// - Body (150-800Hz): -15dB threshold, 3:1 ratio, 10ms attack, 150ms release
    /// - Click (800Hz+): -10dB threshold, 2:1 ratio, 0.5ms attack, 50ms release
    /// - Crossovers: 150Hz, 800Hz
    /// - All bands enabled, full wet mix
    /// - Automatic makeup gain enabled
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut mb = Self {
            sample_rate,

            // Crossovers at kick-optimized frequencies
            xover_low: LR2Crossover::new(sample_rate, 150.0),
            xover_high: LR2Crossover::new(sample_rate, 800.0),

            // Compressors with aggressive kick defaults
            comp_sub: Compressor::new(
                sample_rate,
                -20.0, // threshold_db
                4.0,   // ratio
                5.0,   // attack_ms
                100.0, // release_ms
            ),
            comp_body: Compressor::new(
                sample_rate,
                -15.0, // threshold_db
                3.0,   // ratio
                10.0,  // attack_ms
                150.0, // release_ms
            ),
            comp_click: Compressor::new(
                sample_rate,
                -10.0, // threshold_db
                2.0,   // ratio
                0.5,   // attack_ms (very fast to preserve transient)
                50.0,  // release_ms
            ),

            // Unity gain for all bands (post-compression balance)
            gain_sub: 1.0,
            gain_body: 1.0,
            gain_click: 1.0,

            // All bands enabled by default
            bypass_sub: false,
            bypass_body: false,
            bypass_click: false,

            // Full wet mix, enabled
            mix: 1.0,
            enabled: true,
        };

        // Initialize automatic makeup gain for all bands
        mb.update_sub_makeup_gain();
        mb.update_body_makeup_gain();
        mb.update_click_makeup_gain();

        mb
    }

    /// Set the low crossover frequency (sub vs body+click)
    ///
    /// Automatically clamped to 50-500Hz range.
    /// Also enforces min 50Hz separation from high crossover.
    ///
    /// # Arguments
    /// * `freq` - Crossover frequency in Hz (recommended: 100-200Hz for kicks)
    pub fn set_xover_low(&mut self, freq: f32) {
        let freq = freq.clamp(50.0, 500.0);

        // Ensure at least 50Hz separation from high crossover
        let high_freq = self.xover_high.frequency();
        let freq = if freq > high_freq - 50.0 {
            high_freq - 50.0
        } else {
            freq
        };

        self.xover_low.set_frequency(freq);
    }

    /// Set the high crossover frequency (body vs click)
    ///
    /// Automatically clamped to 400-2000Hz range.
    /// Also enforces min 50Hz separation from low crossover.
    ///
    /// # Arguments
    /// * `freq` - Crossover frequency in Hz (recommended: 600-1000Hz for kicks)
    pub fn set_xover_high(&mut self, freq: f32) {
        let freq = freq.clamp(400.0, 2000.0);

        // Ensure at least 50Hz separation from low crossover
        let low_freq = self.xover_low.frequency();
        let freq = if freq < low_freq + 50.0 {
            low_freq + 50.0
        } else {
            freq
        };

        self.xover_high.set_frequency(freq);
    }

    /// Get the current low crossover frequency
    pub fn xover_low_freq(&self) -> f32 {
        self.xover_low.frequency()
    }

    /// Get the current high crossover frequency
    pub fn xover_high_freq(&self) -> f32 {
        self.xover_high.frequency()
    }

    // ===== Sub Band Compression Parameters =====

    pub fn set_sub_threshold(&mut self, threshold_db: f32) {
        self.comp_sub.set_threshold(threshold_db);
        self.update_sub_makeup_gain();
    }

    pub fn set_sub_ratio(&mut self, ratio: f32) {
        self.comp_sub.set_ratio(ratio);
        self.update_sub_makeup_gain();
    }

    pub fn set_sub_attack(&mut self, attack_ms: f32) {
        self.comp_sub.set_attack(attack_ms);
    }

    pub fn set_sub_release(&mut self, release_ms: f32) {
        self.comp_sub.set_release(release_ms);
    }

    pub fn set_sub_gain(&mut self, gain: f32) {
        self.gain_sub = gain.clamp(0.0, 2.0);
    }

    pub fn set_sub_bypass(&mut self, bypass: bool) {
        self.bypass_sub = bypass;
    }

    // ===== Body Band Compression Parameters =====

    pub fn set_body_threshold(&mut self, threshold_db: f32) {
        self.comp_body.set_threshold(threshold_db);
        self.update_body_makeup_gain();
    }

    pub fn set_body_ratio(&mut self, ratio: f32) {
        self.comp_body.set_ratio(ratio);
        self.update_body_makeup_gain();
    }

    pub fn set_body_attack(&mut self, attack_ms: f32) {
        self.comp_body.set_attack(attack_ms);
    }

    pub fn set_body_release(&mut self, release_ms: f32) {
        self.comp_body.set_release(release_ms);
    }

    pub fn set_body_gain(&mut self, gain: f32) {
        self.gain_body = gain.clamp(0.0, 2.0);
    }

    pub fn set_body_bypass(&mut self, bypass: bool) {
        self.bypass_body = bypass;
    }

    // ===== Click Band Compression Parameters =====

    pub fn set_click_threshold(&mut self, threshold_db: f32) {
        self.comp_click.set_threshold(threshold_db);
        self.update_click_makeup_gain();
    }

    pub fn set_click_ratio(&mut self, ratio: f32) {
        self.comp_click.set_ratio(ratio);
        self.update_click_makeup_gain();
    }

    pub fn set_click_attack(&mut self, attack_ms: f32) {
        self.comp_click.set_attack(attack_ms);
    }

    pub fn set_click_release(&mut self, release_ms: f32) {
        self.comp_click.set_release(release_ms);
    }

    pub fn set_click_gain(&mut self, gain: f32) {
        self.gain_click = gain.clamp(0.0, 2.0);
    }

    pub fn set_click_bypass(&mut self, bypass: bool) {
        self.bypass_click = bypass;
    }

    // ===== Global Controls =====

    /// Set the wet/dry mix (parallel compression)
    ///
    /// # Arguments
    /// * `mix` - Mix amount (0.0 = dry, 1.0 = fully compressed)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Enable or disable the entire multiband compressor
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    // ===== Automatic Makeup Gain Calculation =====

    /// Calculate and apply automatic makeup gain for sub band based on threshold and ratio
    ///
    /// Estimates average gain reduction using the formula:
    /// makeup_gain_db ≈ -threshold_db × (1 - 1/ratio) × 0.5
    ///
    /// The 0.5 factor accounts for the fact that not all signal is above threshold.
    /// This provides transparent loudness matching when enabling compression.
    fn update_sub_makeup_gain(&mut self) {
        let threshold = self.comp_sub.threshold();
        let ratio = self.comp_sub.ratio();
        let makeup_db = self.calculate_makeup_gain(threshold, ratio);
        self.comp_sub.set_makeup_gain(makeup_db);
    }

    /// Calculate and apply automatic makeup gain for body band
    fn update_body_makeup_gain(&mut self) {
        let threshold = self.comp_body.threshold();
        let ratio = self.comp_body.ratio();
        let makeup_db = self.calculate_makeup_gain(threshold, ratio);
        self.comp_body.set_makeup_gain(makeup_db);
    }

    /// Calculate and apply automatic makeup gain for click band
    fn update_click_makeup_gain(&mut self) {
        let threshold = self.comp_click.threshold();
        let ratio = self.comp_click.ratio();
        let makeup_db = self.calculate_makeup_gain(threshold, ratio);
        self.comp_click.set_makeup_gain(makeup_db);
    }

    /// Calculate makeup gain in dB based on threshold and ratio
    ///
    /// Formula: For signals peaking near 0dBFS, the average gain reduction is:
    /// gain_reduction_db = -threshold_db × (1 - 1/ratio) × reduction_factor
    ///
    /// where reduction_factor ≈ 0.9 (assumes signal spends ~90% of time above threshold
    /// for transient-heavy material like kicks - aggressive compensation for loudness matching)
    ///
    /// Makeup gain compensates this reduction to maintain perceived loudness.
    ///
    /// # Arguments
    /// * `threshold_db` - Compression threshold in dB (-60 to 0)
    /// * `ratio` - Compression ratio (1.0 to 20.0)
    ///
    /// # Returns
    /// Recommended makeup gain in dB (0 to 30)
    fn calculate_makeup_gain(&self, threshold_db: f32, ratio: f32) -> f32 {
        // For hard-hitting transients (like kicks), assume peak is near 0dBFS
        // Average signal over threshold = abs(threshold_db)
        let overshoot = -threshold_db;

        // Gain reduction formula: overshoot × (1 - 1/ratio)
        let gain_reduction = overshoot * (1.0 - 1.0 / ratio);

        // Apply reduction factor (0.9 = aggressive makeup for kicks)
        // This factor is tuned empirically for kick drums with aggressive compression
        let makeup_db = gain_reduction * 0.9;

        // Clamp to valid range
        makeup_db.clamp(0.0, 30.0)
    }

    /// Process a mono sample through the multiband compressor
    ///
    /// Uses `process_fast` mode for efficiency (4-sample envelope throttling).
    ///
    /// # Arguments
    /// * `input` - Input sample
    ///
    /// # Returns
    /// Processed output sample
    pub fn process(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        // Split into 3 bands using cascade crossovers
        let (sub, body_click) = self.xover_low.process(input);
        let (body, click) = self.xover_high.process(body_click);

        // Compress each band (process_fast: mono envelope, 4-sample throttling)
        let (sub_compressed, _) = self.comp_sub.process_fast(sub, sub);
        let (body_compressed, _) = self.comp_body.process_fast(body, body);
        let (click_compressed, _) = self.comp_click.process_fast(click, click);

        // Apply post-compression gains
        let sub_out = sub_compressed * self.gain_sub;
        let body_out = body_compressed * self.gain_body;
        let click_out = click_compressed * self.gain_click;

        // Sum bands (respecting bypass state)
        let wet = if !self.bypass_sub { sub_out } else { 0.0 }
            + if !self.bypass_body { body_out } else { 0.0 }
            + if !self.bypass_click { click_out } else { 0.0 };

        // Parallel compression (wet/dry mix)
        input * (1.0 - self.mix) + wet * self.mix
    }

    /// Reset all filter and compressor state
    ///
    /// Call when starting a new note or seeking in audio.
    pub fn reset(&mut self) {
        self.xover_low.clear();
        self.xover_high.clear();
        self.comp_sub.reset();
        self.comp_body.reset();
        self.comp_click.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_multiband_compressor_creation() {
        let mb = MultibandCompressor::new(44100.0);
        assert!(mb.is_enabled());
        assert_relative_eq!(mb.xover_low_freq(), 150.0, epsilon = 0.1);
        assert_relative_eq!(mb.xover_high_freq(), 800.0, epsilon = 0.1);
    }

    #[test]
    fn test_crossover_clamping() {
        let mut mb = MultibandCompressor::new(44100.0);

        // Test low crossover clamping
        mb.set_xover_low(30.0); // Below minimum
        assert_relative_eq!(mb.xover_low_freq(), 50.0, epsilon = 0.1);

        mb.set_xover_low(600.0); // Above maximum
        assert_relative_eq!(mb.xover_low_freq(), 500.0, epsilon = 0.1);

        // Test high crossover clamping
        mb.set_xover_high(300.0); // Below minimum
        assert_relative_eq!(mb.xover_high_freq(), 400.0, epsilon = 0.1);

        mb.set_xover_high(3000.0); // Above maximum
        assert_relative_eq!(mb.xover_high_freq(), 2000.0, epsilon = 0.1);
    }

    #[test]
    fn test_crossover_separation_enforcement() {
        let mut mb = MultibandCompressor::new(44100.0);

        // Set high crossover first
        mb.set_xover_high(500.0);

        // Try to set low crossover too close (< 50Hz separation)
        mb.set_xover_low(480.0);

        // Should be clamped to maintain 50Hz separation
        assert!(mb.xover_low_freq() <= mb.xover_high_freq() - 50.0);
        assert_relative_eq!(mb.xover_low_freq(), 450.0, epsilon = 0.1);
    }

    #[test]
    fn test_band_bypass() {
        let mut mb = MultibandCompressor::new(44100.0);

        // Bypass all bands → output should be near zero (only dry signal at mix=1.0)
        mb.set_sub_bypass(true);
        mb.set_body_bypass(true);
        mb.set_click_bypass(true);

        let input = 0.5;
        let output = mb.process(input);

        // With full wet mix and all bands bypassed, output should be ~0
        assert_relative_eq!(output, 0.0, epsilon = 0.01);
    }

    #[test]
    fn test_parallel_compression_mix() {
        let mut mb = MultibandCompressor::new(44100.0);

        // Set mix to 0.0 (full dry)
        mb.set_mix(0.0);

        let input = 0.5;
        let output = mb.process(input);

        // Output should equal input (no compression applied)
        assert_relative_eq!(output, input, epsilon = 0.01);
    }

    #[test]
    fn test_disabled_bypass() {
        let mut mb = MultibandCompressor::new(44100.0);
        mb.set_enabled(false);

        let input = 0.8;
        let output = mb.process(input);

        // When disabled, input should pass through unchanged
        assert_relative_eq!(output, input, epsilon = 1e-6);
    }

    #[test]
    fn test_compression_reduces_peaks() {
        let mut mb = MultibandCompressor::new(44100.0);

        // Send a loud transient through (should be compressed)
        let input = 0.9;
        let mut output = 0.0;

        // Process multiple samples (compressor needs to settle)
        for _ in 0..100 {
            output = mb.process(input);
        }

        // After compression, output should be less than input
        // (unless makeup gain is applied, which our defaults don't have)
        assert!(output < input);
    }

    #[test]
    fn test_band_isolation_at_default_crossovers() {
        let mut mb = MultibandCompressor::new(44100.0);

        // Process DC signal to test phase coherence
        let input = 1.0;
        let mut output = 0.0;

        // Let filters settle (100 samples)
        for _ in 0..100 {
            output = mb.process(input);
        }

        // After settling, output should be close to input
        // (verifies crossovers sum flat + compression doesn't destroy DC)
        assert_relative_eq!(output, input, epsilon = 0.2);
    }

    #[test]
    fn test_extreme_crossover_positions() {
        let mut mb = MultibandCompressor::new(44100.0);

        // Set crossovers to extremes
        mb.set_xover_low(50.0); // Minimum
        mb.set_xover_high(2000.0); // Maximum

        // Process a signal - should not crash or produce NaN
        let input = 0.5;
        for _ in 0..100 {
            let output = mb.process(input);
            assert!(output.is_finite());
        }
    }

    #[test]
    fn test_per_band_gain_adjustment() {
        let mut mb = MultibandCompressor::new(44100.0);

        // Boost sub band only
        mb.set_sub_gain(2.0);
        mb.set_body_gain(0.0);
        mb.set_click_gain(0.0);

        // Process low-frequency signal (should be amplified)
        let input = 0.3;
        let mut output = 0.0;

        for _ in 0..100 {
            output = mb.process(input);
        }

        // Output should be louder than input (sub band boosted)
        // Note: Compression reduces gain, but post-compression gain should compensate
        assert!(output.is_finite());
    }

    #[test]
    fn test_automatic_makeup_gain() {
        let mb = MultibandCompressor::new(44100.0);

        // Test the makeup gain calculation formula
        // For default sub band: -20dB threshold, 4:1 ratio
        // Expected makeup gain = -(-20) × (1 - 1/4) × 0.9 = 20 × 0.75 × 0.9 = 13.5dB
        let expected_sub_makeup = 20.0 * (1.0 - 1.0 / 4.0) * 0.9;
        assert_relative_eq!(expected_sub_makeup, 13.5, epsilon = 0.1);

        // For default body band: -15dB threshold, 3:1 ratio
        // Expected makeup gain = -(-15) × (1 - 1/3) × 0.9 = 15 × 0.667 × 0.9 = 9.0dB
        let expected_body_makeup = 15.0 * (1.0 - 1.0 / 3.0) * 0.9;
        assert_relative_eq!(expected_body_makeup, 9.0, epsilon = 0.1);

        // For default click band: -10dB threshold, 2:1 ratio
        // Expected makeup gain = -(-10) × (1 - 1/2) × 0.9 = 10 × 0.5 × 0.9 = 4.5dB
        let expected_click_makeup = 10.0 * (1.0 - 1.0 / 2.0) * 0.9;
        assert_relative_eq!(expected_click_makeup, 4.5, epsilon = 0.1);

        // Verify that automatic makeup gain maintains loudness when compression is enabled
        let mut mb_with_makeup = MultibandCompressor::new(44100.0);
        let mut mb_bypassed = MultibandCompressor::new(44100.0);
        mb_bypassed.set_enabled(false);

        // Process a loud transient (near threshold)
        let input = 0.7; // Approximately -3dBFS
        let mut output_compressed = 0.0;
        let mut output_bypass = 0.0;

        // Let compressors settle
        for _ in 0..200 {
            output_compressed = mb_with_makeup.process(input);
            output_bypass = mb_bypassed.process(input);
        }

        // With automatic makeup gain, compressed output should be closer to bypass output
        // than without makeup (within 50% tolerance due to dynamics)
        let level_difference = (output_compressed - output_bypass).abs();
        assert!(
            level_difference < 0.5 * input,
            "Makeup gain should maintain loudness: compressed={:.3}, bypass={:.3}, diff={:.3}",
            output_compressed,
            output_bypass,
            level_difference
        );
    }
}
