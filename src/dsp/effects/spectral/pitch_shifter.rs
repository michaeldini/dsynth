/// Real-Time Pitch Shifter
///
/// Uses a simple but effective delay-based pitch shifting algorithm.
/// This works by modulating read position in a delay buffer to create pitch shifts.
///
/// For subtle pitch correction (auto-tune), this approach is sufficient and avoids
/// the complexity and artifacts of granular/PSOLA methods. The key insight is that
/// small pitch corrections (<1 semitone) can be achieved with minimal formant shifting.
///
/// Algorithm:
/// 1. Write input to circular delay buffer
/// 2. Read from buffer at variable rate to create pitch shift
/// 3. Use linear interpolation for fractional sample positions
/// 4. Smooth read rate changes to avoid clicks

const BUFFER_SIZE_MS: f32 = 50.0; // 50ms circular buffer

#[allow(dead_code)]
pub struct PitchShifter {
    sample_rate: f32,

    // Circular delay buffer
    buffer: Vec<f32>,
    buffer_size: usize,
    write_pos: f32,
    read_pos: f32,

    // Target read rate (how fast to advance through buffer)
    target_read_rate: f32,
    current_read_rate: f32,

    // Smoothing for read rate changes (prevents clicks)
    rate_smoothing: f32,
}

impl PitchShifter {
    pub fn new(sample_rate: f32) -> Self {
        let buffer_size = ((sample_rate * BUFFER_SIZE_MS / 1000.0) as usize).next_power_of_two();
        let initial_delay = buffer_size as f32 / 2.0; // Start in middle of buffer

        Self {
            sample_rate,
            buffer: vec![0.0; buffer_size],
            buffer_size,
            write_pos: 0.0,
            read_pos: initial_delay,
            target_read_rate: 1.0,
            current_read_rate: 1.0,
            rate_smoothing: 0.999, // Very smooth rate changes
        }
    }

    /// Process a single sample with pitch shifting
    ///
    /// # Arguments
    /// * `input` - Input audio sample
    /// * `detected_pitch_hz` - Current detected pitch in Hz (used for validation)
    /// * `target_pitch_hz` - Target pitch in Hz
    /// * `mix` - Dry/wet mix (0.0 = dry, 1.0 = wet)
    ///
    /// # Returns
    /// Pitch-shifted sample
    pub fn process(
        &mut self,
        input: f32,
        detected_pitch_hz: f32,
        target_pitch_hz: f32,
        mix: f32,
    ) -> f32 {
        // If no valid pitch detected, pass through
        if detected_pitch_hz < 50.0 || detected_pitch_hz > 800.0 {
            return input;
        }

        // Calculate pitch shift ratio
        // ratio > 1.0 = shift up (read faster)
        // ratio < 1.0 = shift down (read slower)
        let pitch_ratio = target_pitch_hz / detected_pitch_hz;

        // If ratio is close to 1.0, minimal shift needed - optimize by passing through
        if (pitch_ratio - 1.0).abs() < 0.001 {
            return input;
        }

        // Clamp to reasonable range (±1 octave)
        let pitch_ratio = pitch_ratio.clamp(0.5, 2.0);

        // Write input to buffer
        let write_idx = self.write_pos as usize % self.buffer_size;
        self.buffer[write_idx] = input;
        self.write_pos = (self.write_pos + 1.0) % self.buffer_size as f32;

        // Update target read rate and smooth towards it
        self.target_read_rate = pitch_ratio;
        self.current_read_rate = self.rate_smoothing * self.current_read_rate
            + (1.0 - self.rate_smoothing) * self.target_read_rate;

        // Read from buffer with linear interpolation
        let output = self.read_interpolated();

        // Advance read position at current rate
        self.read_pos += self.current_read_rate;

        // Keep read position within buffer bounds
        while self.read_pos >= self.buffer_size as f32 {
            self.read_pos -= self.buffer_size as f32;
        }
        while self.read_pos < 0.0 {
            self.read_pos += self.buffer_size as f32;
        }

        // Maintain safe distance from write position (prevent buffer underrun/overrun)
        let distance =
            (self.write_pos - self.read_pos + self.buffer_size as f32) % self.buffer_size as f32;

        // If we're getting too close to write position, nudge read position back
        let min_distance = self.buffer_size as f32 * 0.1; // 10% minimum
        let max_distance = self.buffer_size as f32 * 0.9; // 90% maximum

        if distance < min_distance {
            self.read_pos -= 1.0;
        } else if distance > max_distance {
            self.read_pos += 1.0;
        }

        // Mix with dry signal
        input * (1.0 - mix) + output * mix
    }

    /// Read from buffer with linear interpolation for smooth pitch shifting
    fn read_interpolated(&self) -> f32 {
        let idx0 = self.read_pos.floor() as usize % self.buffer_size;
        let idx1 = (idx0 + 1) % self.buffer_size;
        let frac = self.read_pos.fract();

        // Linear interpolation
        self.buffer[idx0] * (1.0 - frac) + self.buffer[idx1] * frac
    }

    /// Reset the pitch shifter state
    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        let initial_delay = self.buffer_size as f32 / 2.0;
        self.write_pos = 0.0;
        self.read_pos = initial_delay;
        self.target_read_rate = 1.0;
        self.current_read_rate = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_shifter_creation() {
        let shifter = PitchShifter::new(44100.0);
        assert_eq!(shifter.sample_rate, 44100.0);
        assert!(shifter.buffer_size > 0);
    }

    #[test]
    fn test_passthrough_no_shift() {
        let mut shifter = PitchShifter::new(44100.0);

        // Same input and target pitch = passthrough
        let input = 0.5;
        let output = shifter.process(input, 440.0, 440.0, 1.0);

        // Should be close to input (may have slight deviation due to buffering)
        assert!((output - input).abs() < 0.1);
    }

    #[test]
    fn test_pitch_shift_generates_valid_audio() {
        let mut shifter = PitchShifter::new(44100.0);

        // Process several samples with pitch shift
        for i in 0..1000 {
            let input = (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 0.5;
            let output = shifter.process(input, 440.0, 550.0, 1.0); // Shift up 4 semitones

            assert!(output.is_finite());
            assert!(output.abs() <= 1.1); // Allow slight overshoot
        }
    }

    #[test]
    fn test_reset_clears_state() {
        let mut shifter = PitchShifter::new(44100.0);

        // Process some samples
        for _ in 0..100 {
            shifter.process(0.5, 440.0, 550.0, 1.0);
        }

        // Reset
        shifter.reset();

        // State should be cleared
        assert_eq!(shifter.write_pos, 0.0);
        assert!(shifter.buffer.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_small_pitch_corrections() {
        let mut shifter = PitchShifter::new(44100.0);

        // Test subtle corrections (typical auto-tune range)
        for i in 0..2000 {
            let t = i as f32 / 44100.0;
            let input = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;

            // Shift by 10 cents (1/10 of a semitone)
            let target = 440.0 * 2.0_f32.powf(10.0 / 1200.0);
            let output = shifter.process(input, 440.0, target, 1.0);

            assert!(output.is_finite());
            assert!(output.abs() <= 1.1); // Allow slight overshoot
        }
    }
}
