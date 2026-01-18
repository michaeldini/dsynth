//! Freeverb-style algorithmic reverb
//!
//! This is a classic "Schroeder reverb" design using parallel comb filters
//! followed by series allpass filters. It's computationally efficient and
//! produces a smooth, natural-sounding reverb suitable for most synthesizer applications.
//!
//! # Architecture
//! - 8 parallel comb filters (tuned to different prime-number delays)
//! - 4 series allpass filters (for echo density)
//! - Stereo output with decorrelated left/right channels
//! - Low-pass damping in feedback path (simulates air absorption)
//!
//! # Parameters
//! - **room_size**: Controls feedback amount (0.0 = small room, 1.0 = huge hall)
//! - **damping**: High-frequency absorption (0.0 = bright, 1.0 = dark/muffled)
//! - **wet**: Reverb signal level (0.0 = dry, 1.0 = full wet)
//! - **dry**: Direct signal level (0.0 = none, 1.0 = full dry)
//! - **width**: Stereo width (0.0 = mono, 1.0 = full stereo)
//!
//! # Real-Time Safety
//! All delay buffers are pre-allocated in `new()` with maximum possible size.
//! No allocations happen during `process()`, making it safe for audio threads.

/// Comb filter delays (in samples at 44.1kHz)
/// These are tuned to prime numbers to avoid modal resonances
const COMB_TUNINGS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];

/// Allpass filter delays (in samples at 44.1kHz)
const ALLPASS_TUNINGS: [usize; 4] = [556, 441, 341, 225];

/// Scale factor for room size parameter (affects feedback gain)
const ROOM_SIZE_SCALE: f32 = 0.28;
const ROOM_SIZE_OFFSET: f32 = 0.7;

/// Damping scale factors (lowpass filter coefficients)
const DAMPING_SCALE: f32 = 0.4;

/// Fixed gain to prevent buildup in comb filters
const FIXED_GAIN: f32 = 0.015;

/// Stereo spread - slightly different tunings for L/R
const STEREO_SPREAD: usize = 23;

/// Single comb filter with lowpass-damped feedback
struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    damping: f32,
    filter_state: f32, // One-pole lowpass state
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
            feedback: 0.5,
            damping: 0.5,
            filter_state: 0.0,
        }
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    fn set_damping(&mut self, damping: f32) {
        self.damping = damping;
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];

        // One-pole lowpass filter in feedback path
        self.filter_state = output * (1.0 - self.damping) + self.filter_state * self.damping;

        // Write input + damped feedback
        self.buffer[self.index] = input + self.filter_state * self.feedback;

        // Advance circular buffer index
        self.index = (self.index + 1) % self.buffer.len();

        output
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.filter_state = 0.0;
        self.index = 0;
    }
}

/// Allpass filter for echo density
struct AllpassFilter {
    buffer: Vec<f32>,
    index: usize,
}

impl AllpassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let output = -input + delayed;

        self.buffer[self.index] = input + delayed * 0.5;
        self.index = (self.index + 1) % self.buffer.len();

        output
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
    }
}

/// Stereo reverb processor
pub struct Reverb {
    #[allow(dead_code)]
    sample_rate: f32,

    // Left channel processing
    comb_l: [CombFilter; 8],
    allpass_l: [AllpassFilter; 4],

    // Right channel processing (slightly detuned for stereo)
    comb_r: [CombFilter; 8],
    allpass_r: [AllpassFilter; 4],

    // Parameters
    room_size: f32,
    damping: f32,
    wet: f32,
    dry: f32,
    width: f32,
}

impl Reverb {
    /// Create a new reverb processor
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0)
    ///
    /// # Pre-allocation
    /// All delay buffers are allocated here based on sample rate.
    /// For 44.1kHz, this allocates ~60KB per channel (120KB total).
    pub fn new(sample_rate: f32) -> Self {
        let scale_factor = sample_rate / 44100.0;

        // Create left channel comb filters
        let comb_l: [CombFilter; 8] = std::array::from_fn(|i| {
            let size = (COMB_TUNINGS[i] as f32 * scale_factor) as usize;
            CombFilter::new(size)
        });

        // Create right channel comb filters (with stereo spread)
        let comb_r: [CombFilter; 8] = std::array::from_fn(|i| {
            let size = ((COMB_TUNINGS[i] + STEREO_SPREAD) as f32 * scale_factor) as usize;
            CombFilter::new(size)
        });

        // Create left channel allpass filters
        let allpass_l: [AllpassFilter; 4] = std::array::from_fn(|i| {
            let size = (ALLPASS_TUNINGS[i] as f32 * scale_factor) as usize;
            AllpassFilter::new(size)
        });

        // Create right channel allpass filters (with stereo spread)
        let allpass_r: [AllpassFilter; 4] = std::array::from_fn(|i| {
            let size = ((ALLPASS_TUNINGS[i] + STEREO_SPREAD) as f32 * scale_factor) as usize;
            AllpassFilter::new(size)
        });

        let mut reverb = Self {
            sample_rate,
            comb_l,
            comb_r,
            allpass_l,
            allpass_r,
            room_size: 0.5,
            damping: 0.5,
            wet: 0.33,
            dry: 0.67,
            width: 1.0,
        };

        reverb.update();
        reverb
    }

    /// Set room size (0.0 to 1.0)
    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size.clamp(0.0, 1.0);
        self.update();
    }

    /// Set damping amount (0.0 = bright, 1.0 = dark)
    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
        self.update();
    }

    /// Set wet level (0.0 to 1.0)
    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet.clamp(0.0, 1.0);
    }

    /// Set dry level (0.0 to 1.0)
    pub fn set_dry(&mut self, dry: f32) {
        self.dry = dry.clamp(0.0, 1.0);
    }

    /// Set stereo width (0.0 = mono, 1.0 = full stereo)
    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(0.0, 1.0);
    }

    /// Update internal filter coefficients
    fn update(&mut self) {
        let feedback = ROOM_SIZE_OFFSET + self.room_size * ROOM_SIZE_SCALE;
        let damp = self.damping * DAMPING_SCALE;

        for i in 0..8 {
            self.comb_l[i].set_feedback(feedback);
            self.comb_l[i].set_damping(damp);
            self.comb_r[i].set_feedback(feedback);
            self.comb_r[i].set_damping(damp);
        }
    }

    /// Process a stereo sample pair
    ///
    /// # Arguments
    /// * `input_l` - Left channel input
    /// * `input_r` - Right channel input
    ///
    /// # Returns
    /// Tuple of (left_output, right_output)
    ///
    /// # Real-Time Safety
    /// This method performs no allocations and has bounded execution time.
    pub fn process(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        // Mix input to mono for feeding to reverb
        let input = (input_l + input_r) * FIXED_GAIN;

        // Process through parallel comb filters
        let mut out_l = 0.0;
        let mut out_r = 0.0;

        for i in 0..8 {
            out_l += self.comb_l[i].process(input);
            out_r += self.comb_r[i].process(input);
        }

        // Process through series allpass filters
        for i in 0..4 {
            out_l = self.allpass_l[i].process(out_l);
            out_r = self.allpass_r[i].process(out_r);
        }

        // Apply stereo width
        let wet1 = self.wet * (self.width / 2.0 + 0.5);
        let wet2 = self.wet * ((1.0 - self.width) / 2.0);

        // Mix wet and dry signals
        let output_l = out_l * wet1 + out_r * wet2 + input_l * self.dry;
        let output_r = out_r * wet1 + out_l * wet2 + input_r * self.dry;

        (output_l, output_r)
    }

    /// Clear all delay buffers
    pub fn clear(&mut self) {
        for i in 0..8 {
            self.comb_l[i].clear();
            self.comb_r[i].clear();
        }
        for i in 0..4 {
            self.allpass_l[i].clear();
            self.allpass_r[i].clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_reverb_creation() {
        let reverb = Reverb::new(44100.0);
        assert_eq!(reverb.sample_rate, 44100.0);
        assert_eq!(reverb.wet, 0.33);
        assert_eq!(reverb.dry, 0.67);
    }

    #[test]
    fn test_reverb_parameters() {
        let mut reverb = Reverb::new(44100.0);

        reverb.set_room_size(0.8);
        assert_eq!(reverb.room_size, 0.8);

        reverb.set_damping(0.6);
        assert_eq!(reverb.damping, 0.6);

        reverb.set_wet(0.5);
        assert_eq!(reverb.wet, 0.5);

        reverb.set_dry(0.5);
        assert_eq!(reverb.dry, 0.5);

        reverb.set_width(0.7);
        assert_eq!(reverb.width, 0.7);
    }

    #[test]
    fn test_reverb_parameter_clamping() {
        let mut reverb = Reverb::new(44100.0);

        reverb.set_room_size(1.5);
        assert_eq!(reverb.room_size, 1.0);

        reverb.set_damping(-0.5);
        assert_eq!(reverb.damping, 0.0);

        reverb.set_wet(2.0);
        assert_eq!(reverb.wet, 1.0);

        reverb.set_width(-1.0);
        assert_eq!(reverb.width, 0.0);
    }

    #[test]
    fn test_reverb_dry_signal_passthrough() {
        let mut reverb = Reverb::new(44100.0);
        reverb.set_wet(0.0);
        reverb.set_dry(1.0);

        let (out_l, out_r) = reverb.process(0.5, -0.5);

        // With dry=1.0 and wet=0.0, output should be close to input
        assert_relative_eq!(out_l, 0.5, epsilon = 0.01);
        assert_relative_eq!(out_r, -0.5, epsilon = 0.01);
    }

    #[test]
    fn test_reverb_produces_decay() {
        let mut reverb = Reverb::new(44100.0);
        reverb.set_room_size(0.8);
        reverb.set_wet(1.0);
        reverb.set_dry(0.0);

        // Send impulse
        let (_first_l, _first_r) = reverb.process(1.0, 1.0);

        // Continue processing silence - should get reverb tail
        let mut has_decay = false;

        for _ in 0..5000 {
            let (out_l, out_r) = reverb.process(0.0, 0.0);

            // Check if we're getting output (reverb tail)
            if out_l.abs() > 0.00001 || out_r.abs() > 0.00001 {
                has_decay = true;
                break;
            }
        }

        assert!(has_decay, "Reverb should produce a decay tail");
    }

    #[test]
    fn test_reverb_clear() {
        let mut reverb = Reverb::new(44100.0);

        // Send impulse
        reverb.process(1.0, 1.0);

        // Clear
        reverb.clear();

        // Process silence - should be silent (no reverb tail)
        for _ in 0..100 {
            let (out_l, out_r) = reverb.process(0.0, 0.0);
            assert_relative_eq!(out_l, 0.0, epsilon = 0.0001);
            assert_relative_eq!(out_r, 0.0, epsilon = 0.0001);
        }
    }

    #[test]
    fn test_reverb_stability() {
        let mut reverb = Reverb::new(44100.0);
        reverb.set_room_size(1.0); // Maximum feedback

        // Process for a long time with continuous input
        for _ in 0..44100 {
            let (out_l, out_r) = reverb.process(0.1, 0.1);

            // Should not blow up
            assert!(out_l.abs() < 10.0, "Reverb became unstable (left)");
            assert!(out_r.abs() < 10.0, "Reverb became unstable (right)");
            assert!(out_l.is_finite(), "Reverb produced NaN/inf (left)");
            assert!(out_r.is_finite(), "Reverb produced NaN/inf (right)");
        }
    }

    #[test]
    fn test_reverb_stereo_decorrelation() {
        let mut reverb = Reverb::new(44100.0);
        reverb.set_wet(1.0); // Full wet to hear reverb tail
        reverb.set_dry(0.0); // No dry signal
        reverb.set_width(1.0);
        reverb.set_room_size(0.9); // Large room for longer tail

        // Send strong asymmetric impulse (different L/R) to produce clear stereo effect
        for _ in 0..10 {
            reverb.process(1.0, 0.5);
        }

        // After some samples, L and R should be decorrelated
        let mut l_samples = Vec::new();
        let mut r_samples = Vec::new();

        // Process to collect reverb tail
        for _ in 0..2000 {
            let (out_l, out_r) = reverb.process(0.0, 0.0);
            l_samples.push(out_l);
            r_samples.push(out_r);
        }

        // L and R shouldn't be identical (stereo decorrelation)
        // With STEREO_SPREAD offset, channels will differ eventually
        let mut different_count = 0;
        let mut has_output_l = false;
        let mut has_output_r = false;

        for i in 0..2000 {
            let abs_l = l_samples[i].abs();
            let abs_r = r_samples[i].abs();
            if abs_l > 0.00001 {
                has_output_l = true;
            }
            if abs_r > 0.00001 {
                has_output_r = true;
            }
            if (l_samples[i] - r_samples[i]).abs() > 0.00001 {
                different_count += 1;
            }
        }

        // Reverb should produce stereo output (both channels active)
        assert!(
            has_output_l && has_output_r,
            "Reverb should produce output on both channels"
        );
        // With stereo spread, samples will differ
        assert!(
            different_count > 0,
            "Reverb should produce some stereo decorrelation, found {} different out of 2000",
            different_count
        );
    }
}
