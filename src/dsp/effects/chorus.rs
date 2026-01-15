/// Multi-voice chorus effect
///
/// Chorus creates a "thick" sound by mixing the dry signal with multiple slightly
/// detuned and delayed copies. This simulates multiple instruments playing the same
/// note with slight timing and pitch variations (like a choir or string ensemble).
///
/// # Architecture
/// - 4 delay lines with LFO-modulated read positions
/// - Each voice has slightly different LFO rate and phase for natural variation
/// - Typical delay time: 10-30ms with ±5ms LFO modulation
/// - Creates pitch variation (vibrato) and timing variation (chorusing)
///
/// # Parameters
/// - **rate**: LFO speed in Hz (0.1 to 5.0)
/// - **depth**: Modulation depth - how much pitch variation (0.0 to 1.0)
/// - **mix**: Wet/dry balance (0.0 = dry, 1.0 = full wet)
///
/// # Real-Time Safety
/// All delay buffers are pre-allocated (50ms per voice).
/// No allocations during `process()`.
use std::f32::consts::PI;

const NUM_VOICES: usize = 4;
const MAX_DELAY_MS: f32 = 50.0; // Maximum delay time
const BASE_DELAY_MS: f32 = 20.0; // Center delay time

/// Single chorus voice with LFO-modulated delay
struct ChorusVoice {
    buffer: Vec<f32>,
    write_index: usize,
    lfo_phase: f32,
    lfo_rate: f32,
    #[allow(dead_code)]
    phase_offset: f32,
    sample_rate: f32,
}

impl ChorusVoice {
    fn new(sample_rate: f32, phase_offset: f32, rate_offset: f32) -> Self {
        let max_samples = (sample_rate * MAX_DELAY_MS / 1000.0) as usize;

        Self {
            buffer: vec![0.0; max_samples],
            write_index: 0,
            lfo_phase: phase_offset,
            lfo_rate: 0.5 + rate_offset,
            phase_offset,
            sample_rate,
        }
    }

    fn set_rate(&mut self, rate: f32, rate_offset: f32) {
        self.lfo_rate = rate + rate_offset;
    }

    fn process(&mut self, input: f32, depth: f32) -> f32 {
        // Write input to circular buffer
        self.buffer[self.write_index] = input;

        // Calculate delay time with LFO modulation
        // LFO oscillates delay between (BASE - depth*10ms) and (BASE + depth*10ms)
        let lfo = (self.lfo_phase * 2.0 * PI).sin();
        let delay_ms = BASE_DELAY_MS + lfo * depth * 10.0;
        let delay_samples = (delay_ms * self.sample_rate / 1000.0) as usize;

        // Calculate read position with linear interpolation
        let read_pos_float = self.write_index as f32 - delay_samples as f32;
        let read_pos_int = read_pos_float.floor() as isize;
        let frac = read_pos_float - read_pos_float.floor();

        // Wrap read positions
        let buffer_len = self.buffer.len() as isize;
        let read_index1 = if read_pos_int < 0 {
            (buffer_len + read_pos_int) as usize
        } else {
            (read_pos_int % buffer_len) as usize
        };

        let read_index2 = (read_index1 + 1) % self.buffer.len();

        // Linear interpolation between samples
        let sample1 = self.buffer[read_index1];
        let sample2 = self.buffer[read_index2];
        let output = sample1 + (sample2 - sample1) * frac;

        // Advance write position
        self.write_index = (self.write_index + 1) % self.buffer.len();

        // Advance LFO phase
        let phase_increment = self.lfo_rate / self.sample_rate;
        self.lfo_phase += phase_increment;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        output
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_index = 0;
    }
}

/// Stereo chorus processor
pub struct Chorus {
    voices_l: [ChorusVoice; NUM_VOICES],
    voices_r: [ChorusVoice; NUM_VOICES],
    #[allow(dead_code)]
    sample_rate: f32,

    // Parameters
    rate: f32,
    depth: f32,
    mix: f32,
}

impl Chorus {
    /// Create a new chorus effect
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0)
    ///
    /// # Voice Configuration
    /// Each voice has a slightly different phase offset and rate variation
    /// to create natural, non-periodic modulation.
    pub fn new(sample_rate: f32) -> Self {
        // Phase offsets for 4 voices (spread evenly across LFO cycle)
        let phase_offsets = [0.0, 0.25, 0.5, 0.75];

        // Rate variations for each voice (creates beating effect)
        let rate_variations = [0.0, 0.1, -0.05, 0.15];

        let voices_l: [ChorusVoice; NUM_VOICES] = std::array::from_fn(|i| {
            ChorusVoice::new(sample_rate, phase_offsets[i], rate_variations[i])
        });

        let voices_r: [ChorusVoice; NUM_VOICES] = std::array::from_fn(|i| {
            // Right channel has slightly different phases for stereo width
            ChorusVoice::new(sample_rate, phase_offsets[i] + 0.125, rate_variations[i])
        });

        Self {
            voices_l,
            voices_r,
            sample_rate,
            rate: 0.5,
            depth: 0.5,
            mix: 0.5,
        }
    }

    /// Set LFO rate in Hz (0.1 to 5.0)
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.clamp(0.1, 5.0);

        // Update all voices with their rate offsets
        let rate_variations = [0.0, 0.1, -0.05, 0.15];
        for i in 0..NUM_VOICES {
            self.voices_l[i].set_rate(self.rate, rate_variations[i]);
            self.voices_r[i].set_rate(self.rate, rate_variations[i]);
        }
    }

    /// Set modulation depth (0.0 to 1.0)
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    /// Set wet/dry mix (0.0 = dry, 1.0 = full wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Reset LFO phase to 0.0 for all voices
    ///
    /// This is called when tempo sync mode changes to ensure predictable timing.
    pub fn reset_phase(&mut self) {
        let phase_offsets = [0.0, 0.25, 0.5, 0.75];
        for i in 0..NUM_VOICES {
            self.voices_l[i].lfo_phase = phase_offsets[i];
            self.voices_r[i].lfo_phase = phase_offsets[i] + 0.125;
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
    pub fn process(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        // Process through all chorus voices and sum
        let mut chorus_l = 0.0;
        let mut chorus_r = 0.0;

        for i in 0..NUM_VOICES {
            chorus_l += self.voices_l[i].process(input_l, self.depth);
            chorus_r += self.voices_r[i].process(input_r, self.depth);
        }

        // Normalize by number of voices
        chorus_l /= NUM_VOICES as f32;
        chorus_r /= NUM_VOICES as f32;

        // Mix wet and dry
        let output_l = input_l * (1.0 - self.mix) + chorus_l * self.mix;
        let output_r = input_r * (1.0 - self.mix) + chorus_r * self.mix;

        (output_l, output_r)
    }

    /// Clear all delay buffers
    pub fn clear(&mut self) {
        for i in 0..NUM_VOICES {
            self.voices_l[i].clear();
            self.voices_r[i].clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_chorus_creation() {
        let chorus = Chorus::new(44100.0);
        assert_eq!(chorus.sample_rate, 44100.0);
        assert_eq!(chorus.rate, 0.5);
        assert_eq!(chorus.depth, 0.5);
        assert_eq!(chorus.mix, 0.5);
    }

    #[test]
    fn test_chorus_parameters() {
        let mut chorus = Chorus::new(44100.0);

        chorus.set_rate(2.0);
        assert_eq!(chorus.rate, 2.0);

        chorus.set_depth(0.8);
        assert_eq!(chorus.depth, 0.8);

        chorus.set_mix(0.7);
        assert_eq!(chorus.mix, 0.7);
    }

    #[test]
    fn test_chorus_parameter_clamping() {
        let mut chorus = Chorus::new(44100.0);

        chorus.set_rate(10.0); // Over max
        assert_eq!(chorus.rate, 5.0);

        chorus.set_rate(0.01); // Under min
        assert_eq!(chorus.rate, 0.1);

        chorus.set_depth(1.5);
        assert_eq!(chorus.depth, 1.0);

        chorus.set_mix(-0.5);
        assert_eq!(chorus.mix, 0.0);
    }

    #[test]
    fn test_chorus_dry_passthrough() {
        let mut chorus = Chorus::new(44100.0);
        chorus.set_mix(0.0); // Fully dry

        let (out_l, out_r) = chorus.process(0.5, -0.5);

        // With mix=0.0, output should approximately equal input
        assert_relative_eq!(out_l, 0.5, epsilon = 0.01);
        assert_relative_eq!(out_r, -0.5, epsilon = 0.01);
    }

    #[test]
    fn test_chorus_produces_modulation() {
        let mut chorus = Chorus::new(44100.0);
        chorus.set_rate(1.0);
        chorus.set_depth(1.0);
        chorus.set_mix(1.0); // Full wet

        // Send continuous tone
        let mut outputs = Vec::new();
        for _ in 0..1000 {
            let (out_l, _) = chorus.process(0.5, 0.5);
            outputs.push(out_l);
        }

        // Output should vary (modulation is happening)
        let min = outputs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = outputs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let variation = max - min;

        assert!(
            variation > 0.01,
            "Chorus should produce amplitude variation"
        );
    }

    #[test]
    fn test_chorus_clear() {
        let mut chorus = Chorus::new(44100.0);

        // Process some signal
        for _ in 0..100 {
            chorus.process(0.5, 0.5);
        }

        // Clear
        chorus.clear();

        // Process silence - should be silent
        chorus.set_mix(1.0); // Full wet to test chorus buffer
        for _ in 0..100 {
            let (out_l, out_r) = chorus.process(0.0, 0.0);
            assert_relative_eq!(out_l, 0.0, epsilon = 0.0001);
            assert_relative_eq!(out_r, 0.0, epsilon = 0.0001);
        }
    }

    #[test]
    fn test_chorus_stability() {
        let mut chorus = Chorus::new(44100.0);
        chorus.set_depth(1.0);
        chorus.set_rate(5.0);

        // Process for a long time
        for _ in 0..44100 {
            let (out_l, out_r) = chorus.process(0.5, 0.5);

            // Should not blow up
            assert!(out_l.abs() < 10.0, "Chorus became unstable (left)");
            assert!(out_r.abs() < 10.0, "Chorus became unstable (right)");
            assert!(out_l.is_finite(), "Chorus produced NaN/inf (left)");
            assert!(out_r.is_finite(), "Chorus produced NaN/inf (right)");
        }
    }

    #[test]
    fn test_chorus_stereo_width() {
        let mut chorus = Chorus::new(44100.0);
        chorus.set_mix(1.0);
        chorus.set_depth(1.0);

        // Process mono input
        let mut l_samples = Vec::new();
        let mut r_samples = Vec::new();

        for _ in 0..500 {
            let (out_l, out_r) = chorus.process(0.5, 0.5);
            l_samples.push(out_l);
            r_samples.push(out_r);
        }

        // L and R should be decorrelated (not identical)
        // Note: chorus modulation is subtle, so there will be many similar samples
        let mut different_count = 0;
        for i in 0..500 {
            if (l_samples[i] - r_samples[i]).abs() > 0.01 {
                different_count += 1;
            }
        }

        // At least some samples should be noticeably different (stereo width)
        assert!(
            different_count > 10,
            "Chorus should create stereo width, found {} different",
            different_count
        );
    }
}
