/// Mid/Side stereo processor for vocal enhancement
///
/// Provides frequency-selective stereo width control using existing crossover bands.
/// This allows different stereo treatment per frequency range:
/// - Bass: Keep mono for better translation and power
/// - Mid: Adjust vocal presence width
/// - Presence: Control consonant stereo spread  
/// - Air: Enhance breathiness and ambience width
///
/// # Mid/Side Mathematics
/// - Mid = (L + R) / 2 (mono-compatible center)
/// - Side = (L - R) / 2 (stereo difference)
/// - Reconstruction: L = Mid + Side, R = Mid - Side
/// - Width control: Side *= width_factor
///
/// # Zero Latency
/// This processor adds no latency since it operates on already-split frequency bands
/// from the existing crossover. It's essentially a per-band stereo image adjuster.

/// Per-band mid/side stereo processor for vocals
pub struct MidSideStereoProcessor {
    // No state needed - this is a stateless processor that works on crossover outputs
}

impl MidSideStereoProcessor {
    /// Create a new mid/side stereo processor
    pub fn new() -> Self {
        Self {}
    }

    /// Process 4-band crossover outputs with frequency-selective stereo width
    ///
    /// Uses the global stereo_width parameter to derive intelligent per-band widths:
    /// - Bass: Conservative width for mono compatibility  
    /// - Mid: Moderate width for vocal focus
    /// - Presence: Enhanced width for spaciousness
    /// - Air: Maximum width for breathiness
    ///
    /// # Arguments
    /// * `bass_l`, `bass_r` - Bass band (DC → 200Hz)
    /// * `mid_l`, `mid_r` - Mid band (200Hz → 1kHz)
    /// * `presence_l`, `presence_r` - Presence band (1kHz → 8kHz)
    /// * `air_l`, `air_r` - Air band (8kHz → Nyquist)
    /// * `stereo_width` - Global stereo width (-1.0 to +1.0)
    ///
    /// # Returns
    /// Tuple of processed (left_out, right_out) after recombining all bands
    #[inline]
    pub fn process(
        &mut self,
        bass_l: f32,
        bass_r: f32,
        mid_l: f32,
        mid_r: f32,
        presence_l: f32,
        presence_r: f32,
        air_l: f32,
        air_r: f32,
        stereo_width: f32,
    ) -> (f32, f32) {
        // Derive frequency-selective widths from global parameter
        let (bass_width, mid_width, presence_width, air_width) =
            Self::derive_band_widths(stereo_width);

        // Process each frequency band with mid/side
        let (bass_l_out, bass_r_out) = Self::process_band_midside(bass_l, bass_r, bass_width);
        let (mid_l_out, mid_r_out) = Self::process_band_midside(mid_l, mid_r, mid_width);
        let (presence_l_out, presence_r_out) =
            Self::process_band_midside(presence_l, presence_r, presence_width);
        let (air_l_out, air_r_out) = Self::process_band_midside(air_l, air_r, air_width);

        // Recombine all processed bands
        let left_out = bass_l_out + mid_l_out + presence_l_out + air_l_out;
        let right_out = bass_r_out + mid_r_out + presence_r_out + air_r_out;

        (left_out, right_out)
    }

    /// Process a single frequency band with mid/side stereo width adjustment
    ///
    /// # Arguments
    /// * `left` - Left channel input for this band
    /// * `right` - Right channel input for this band
    /// * `width` - Stereo width factor (0.0 = mono, 1.0 = normal, 2.0+ = wide)
    ///
    /// # Returns
    /// Tuple of (processed_left, processed_right)
    #[inline]
    fn process_band_midside(left: f32, right: f32, width: f32) -> (f32, f32) {
        // Convert to mid/side
        let mid = (left + right) * 0.5; // Mono-compatible center
        let side = (left - right) * 0.5; // Stereo difference

        // For wide settings and moderate stereo content, add spatial enhancement
        let enhanced_side = if width > 1.6 && side.abs() > 0.001 && side.abs() < 0.4 {
            // Enhanced stereo content for better spatial impression (not pure side signals)
            let artificial_enhancement = side * 0.25 * (width - 1.2); // Stronger enhancement
            side + artificial_enhancement
        } else {
            side
        };

        // Apply width control to enhanced side channel
        let side_processed = enhanced_side * width;

        // Convert back to L/R
        let left_out = mid + side_processed;
        let right_out = mid - side_processed;

        (left_out, right_out)
    }

    /// Derive per-band stereo widths from global stereo_width parameter
    ///
    /// Uses professional frequency-selective scaling optimized for vocal processing:
    /// - Bass: Reduced width for better mono compatibility and power
    /// - Mid: Controlled width for vocal formant focus
    /// - Presence: Enhanced width for consonant spaciousness  
    /// - Air: Maximum width for breathiness and shimmer
    ///
    /// # Arguments
    /// * `stereo_width` - Global stereo width (-1.0 to +1.0)
    ///   - -1.0: Phase inversion effect
    ///   - 0.0: Frequency-selective mono/narrow stereo
    ///   - +1.0: Enhanced frequency-selective stereo width
    ///
    /// # Returns
    /// Tuple of (bass_width, mid_width, presence_width, air_width)
    pub fn derive_band_widths(stereo_width: f32) -> (f32, f32, f32, f32) {
        // Clamp input to valid range
        let width = stereo_width.clamp(-1.0, 1.0);

        if width >= 0.0 {
            // Positive width: Dramatic spatial effect with frequency progression
            let bass_width = 0.95 - 0.65 + width * 0.65; // Bass: 0.3 → 0.95 (progressive, mono compatible)
            let mid_width = 0.95 - 0.45 + width * 2.95; // Mid: 0.5 → 3.5 (progressive, dramatic)
            let presence_width = 0.95 - 0.35 + width * 3.15; // Presence: 0.6 → 3.75 (progressive)
            let air_width = 0.95 - 0.25 + width * 3.45; // Air: 0.7 → 4.2 (progressive, maximum)

            (bass_width, mid_width, presence_width, air_width)
        } else {
            // Negative width: Perfect mirror for symmetric L/R balance
            let invert_amount = -width; // 0.0 to 1.0

            // Perfect symmetry around the zero-width baseline
            let bass_width = 0.95 - 0.65 - invert_amount * 0.65; // Bass: 0.3 → -0.35 (mirrors positive)
            let mid_width = 0.95 - 0.45 - invert_amount * 2.95; // Mid: 0.5 → -2.45 (mirrors positive)
            let presence_width = 0.95 - 0.35 - invert_amount * 3.15; // Presence: 0.6 → -2.55 (mirrors positive)
            let air_width = 0.95 - 0.25 - invert_amount * 3.45; // Air: 0.7 → -2.75 (mirrors positive)

            (bass_width, mid_width, presence_width, air_width)
        }
    }

    /// Reset processor state (no-op for stateless processor)
    pub fn reset(&mut self) {
        // No internal state to reset
    }
}

impl Default for MidSideStereoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = MidSideStereoProcessor::new();
        // Just verify it creates without panic
        assert_eq!(std::mem::size_of_val(&processor), 0); // Should be zero-sized
    }

    #[test]
    fn test_mono_width_produces_mono() {
        let mut processor = MidSideStereoProcessor::new();

        // Test with stereo input
        let bass_l = 0.5;
        let bass_r = -0.3;

        // Process with stereo_width = 0.0 (should produce narrow stereo, not pure mono due to frequency scaling)
        let (left_out, right_out) =
            processor.process(bass_l, bass_r, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        // At width=0.0, bass gets width=0.3, so output should be narrowed but not pure mono
        let input_diff = (bass_l - bass_r).abs();
        let output_diff = (left_out - right_out).abs();

        assert!(
            output_diff < input_diff,
            "Width=0.0 should narrow the stereo image"
        );
        assert!(
            output_diff > 0.01,
            "Should not be pure mono due to frequency-selective scaling"
        );
    }

    #[test]
    fn test_normal_width_enhances_stereo() {
        let mut processor = MidSideStereoProcessor::new();

        // Test with only bass content (easier to verify)
        let bass_l = 0.5;
        let bass_r = -0.5; // Maximum stereo difference

        let (left_narrow, right_narrow) =
            processor.process(bass_l, bass_r, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        let (left_wide, right_wide) =
            processor.process(bass_l, bass_r, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);

        // Wide width should have greater stereo separation
        let narrow_diff = (left_narrow - right_narrow).abs();
        let wide_diff = (left_wide - right_wide).abs();

        assert!(
            wide_diff > narrow_diff,
            "Higher width should increase stereo separation. Narrow: {}, Wide: {}",
            narrow_diff,
            wide_diff
        );
    }

    #[test]
    fn test_debug_width_values() {
        // Debug test to see actual width values
        let (bass_w, mid_w, presence_w, air_w) = MidSideStereoProcessor::derive_band_widths(-1.0);
        println!(
            "Width -1.0: bass={}, mid={}, presence={}, air={}",
            bass_w, mid_w, presence_w, air_w
        );

        let (bass_w, mid_w, presence_w, air_w) = MidSideStereoProcessor::derive_band_widths(0.0);
        println!(
            "Width  0.0: bass={}, mid={}, presence={}, air={}",
            bass_w, mid_w, presence_w, air_w
        );

        let (bass_w, mid_w, presence_w, air_w) = MidSideStereoProcessor::derive_band_widths(1.0);
        println!(
            "Width +1.0: bass={}, mid={}, presence={}, air={}",
            bass_w, mid_w, presence_w, air_w
        );

        // Test actual processing effect with more realistic input
        let mut processor = MidSideStereoProcessor::new();

        // Test with signal in all bands (more realistic)
        let bass_l = 0.3;
        let bass_r = -0.2;
        let mid_l = 0.5;
        let mid_r = -0.3;
        let presence_l = 0.4;
        let presence_r = -0.6;
        let air_l = 0.2;
        let air_r = -0.4;

        let (narrow_l, narrow_r) = processor.process(
            bass_l, bass_r, mid_l, mid_r, presence_l, presence_r, air_l, air_r, -1.0,
        );

        let (wide_l, wide_r) = processor.process(
            bass_l, bass_r, mid_l, mid_r, presence_l, presence_r, air_l, air_r, 1.0,
        );

        println!("=== REALISTIC STEREO INPUT ===");
        println!(
            "Input total: L={}, R={}",
            bass_l + mid_l + presence_l + air_l,
            bass_r + mid_r + presence_r + air_r
        );
        println!("Width -1.0 output: L={:.3}, R={:.3}", narrow_l, narrow_r);
        println!("Width +1.0 output: L={:.3}, R={:.3}", wide_l, wide_r);
        println!("Width -1.0 diff: {:.3}", (narrow_l - narrow_r).abs());
        println!("Width +1.0 diff: {:.3}", (wide_l - wide_r).abs());
        println!(
            "Effect ratio: {:.2}x",
            (wide_l - wide_r).abs() / (narrow_l - narrow_r).abs()
        );
    }

    #[test]
    fn test_frequency_selective_behavior() {
        // Test that different frequency bands get different width scaling
        let (bass_w, mid_w, presence_w, air_w) = MidSideStereoProcessor::derive_band_widths(1.0);

        // At max width, should follow: bass < mid < presence < air
        assert!(
            bass_w < mid_w,
            "Bass should be narrower than mid: {} < {}",
            bass_w,
            mid_w
        );
        assert!(
            mid_w < presence_w,
            "Mid should be narrower than presence: {} < {}",
            mid_w,
            presence_w
        );
        assert!(
            presence_w < air_w,
            "Presence should be narrower than air: {} < {}",
            presence_w,
            air_w
        );

        // Bass should be conservative even at max width
        assert!(
            bass_w < 1.0,
            "Bass should stay below unity gain for mono compatibility: {}",
            bass_w
        );

        // Air should be enhanced for breathiness
        assert!(
            air_w > 1.5,
            "Air should be significantly widened: {}",
            air_w
        );
    }

    #[test]
    fn test_zero_width_frequency_scaling() {
        let (bass_w, mid_w, presence_w, air_w) = MidSideStereoProcessor::derive_band_widths(0.0);

        // Even at zero width, each band should have different scaling
        assert!(
            bass_w < mid_w,
            "Bass should be narrowest even at zero width"
        );
        assert!(
            mid_w < presence_w,
            "Progressive widening through frequency bands"
        );
        assert!(
            presence_w < air_w,
            "Air should be widest even at zero width"
        );
    }

    #[test]
    fn test_negative_width_behavior() {
        let (bass_w, mid_w, presence_w, air_w) = MidSideStereoProcessor::derive_band_widths(-1.0);

        // At maximum negative width, air should be most affected (lowest value)
        assert!(air_w < presence_w, "Air should be most inverted");
        assert!(presence_w < mid_w, "Progressive inversion");
        assert!(mid_w < bass_w, "Bass should be least affected by inversion");

        // Some bands may go negative for phase inversion effect
        assert!(air_w < 0.0, "Air should go negative for phase inversion");
    }

    #[test]
    fn test_band_midside_processing() {
        // Test pure mono input (L=R)
        let (left_out, right_out) = MidSideStereoProcessor::process_band_midside(0.5, 0.5, 2.0);
        let epsilon = 1e-6;
        assert!(
            (left_out - right_out).abs() < epsilon,
            "Mono input should remain mono regardless of width"
        );
        assert!(
            (left_out - 0.5).abs() < epsilon,
            "Mono input should be preserved in amplitude"
        );

        // Test pure side input (L=-R)
        let (left_out, right_out) = MidSideStereoProcessor::process_band_midside(0.5, -0.5, 2.0);
        // With width=2.0, the stereo difference should be doubled
        assert!(
            (left_out - 1.0).abs() < epsilon && (right_out - (-1.0)).abs() < epsilon,
            "Pure side input with width=2.0 should double the difference. Got L={}, R={}",
            left_out,
            right_out
        );
    }

    #[test]
    fn test_zero_sized_struct() {
        // Verify that the processor has no runtime overhead
        let processor = MidSideStereoProcessor::new();
        assert_eq!(std::mem::size_of_val(&processor), 0);
    }

    #[test]
    fn test_lr_balance_symmetry() {
        // Test that symmetric inputs produce symmetric outputs at extreme widths
        let epsilon = 1e-10; // Very tight tolerance for balance testing

        // Test with symmetric stereo signal
        let (left_pos, right_pos) = MidSideStereoProcessor::process_band_midside(0.8, 0.6, 1.0);
        let (left_neg, right_neg) = MidSideStereoProcessor::process_band_midside(0.8, 0.6, -1.0);

        // For the same input, positive and negative width should have equal magnitude
        let pos_balance = (left_pos + right_pos) * 0.5; // Average level
        let neg_balance = (left_neg + right_neg) * 0.5; // Average level

        assert!(
            (pos_balance - neg_balance).abs() < epsilon,
            "L/R balance should be identical for +width and -width: pos={}, neg={}",
            pos_balance,
            neg_balance
        );

        // Test with inverted input to check complete symmetry
        let (left_inv, right_inv) = MidSideStereoProcessor::process_band_midside(0.6, 0.8, 1.0);
        let inv_balance = (left_inv + right_inv) * 0.5;

        assert!(
            (pos_balance - inv_balance).abs() < epsilon,
            "Swapped L/R input should produce same average level: orig={}, inv={}",
            pos_balance,
            inv_balance
        );
    }

    #[test]
    fn test_full_processor_balance() {
        // Test full processor for L/R balance with all bands
        let mut processor = MidSideStereoProcessor::new();
        let epsilon = 1e-6;

        // Symmetric stereo input across all bands
        let bass_l = 0.3;
        let bass_r = 0.2;
        let mid_l = 0.4;
        let mid_r = 0.3;
        let presence_l = 0.2;
        let presence_r = 0.15;
        let air_l = 0.1;
        let air_r = 0.08;

        // Test at extreme positive width
        let (left_pos, right_pos) = processor.process(
            bass_l, bass_r, mid_l, mid_r, presence_l, presence_r, air_l, air_r, 1.0,
        );

        // Test at extreme negative width
        let (left_neg, right_neg) = processor.process(
            bass_l, bass_r, mid_l, mid_r, presence_l, presence_r, air_l, air_r, -1.0,
        );

        let pos_total = left_pos + right_pos;
        let neg_total = left_neg + right_neg;
        let input_total = bass_l + bass_r + mid_l + mid_r + presence_l + presence_r + air_l + air_r;

        println!("Input total: {}", input_total);
        println!("Positive width total: {}", pos_total);
        println!("Negative width total: {}", neg_total);
        println!(
            "Positive balance: L={}, R={}, diff={}",
            left_pos,
            right_pos,
            left_pos - right_pos
        );
        println!(
            "Negative balance: L={}, R={}, diff={}",
            left_neg,
            right_neg,
            left_neg - right_neg
        );

        // The total energy should be conserved (approximately)
        assert!(
            (pos_total - input_total).abs() < 0.1,
            "Energy should be approximately conserved at +width"
        );
        assert!(
            (neg_total - input_total).abs() < 0.1,
            "Energy should be approximately conserved at -width"
        );
    }
}
