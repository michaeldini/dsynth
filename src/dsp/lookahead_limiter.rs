/// Look-ahead limiter for transparent peak limiting.
///
/// This limiter analyzes incoming audio ahead of time (look-ahead buffer) to detect
/// peaks before they occur, allowing for smoother gain reduction without artifacts.
/// This is superior to simple reactive limiting because it can "see" peaks coming
/// and apply gain reduction gradually, preventing the pumping/breathing artifacts
/// common in fast attack limiters.
///
/// ## How It Works
///
/// 1. **Delay Buffer**: Incoming samples are stored in a circular buffer
/// 2. **Peak Detection**: Efficiently track the maximum peak using a sliding window algorithm
/// 3. **Gain Calculation**: Calculate required gain reduction to keep peak under threshold
/// 4. **Smooth Application**: Apply smoothed gain reduction to the delayed output
/// 5. **Output**: Emit the delayed (but limited) sample
///
/// The delay (typically 5ms) is imperceptible but gives us time to react to peaks smoothly.
///
/// ## Optimization
///
/// Uses an efficient sliding window maximum algorithm with a monotonic deque to track
/// peaks in O(1) amortized time instead of O(N) linear scans, dramatically reducing CPU usage.
use std::collections::VecDeque;

/// Entry in the peak tracking deque (value and position)
#[derive(Copy, Clone)]
struct PeakEntry {
    peak: f32,
    position: usize,
}

pub struct LookAheadLimiter {
    /// Sample rate in Hz
    sample_rate: f32,

    /// Look-ahead time in samples (e.g., 220 samples = 5ms at 44.1kHz)
    lookahead_samples: usize,

    /// Threshold for limiting (0.0-1.0), typically 0.98 or 0.99
    threshold: f32,

    /// Delay buffer for left channel (holds lookahead_samples worth of audio)
    delay_buffer_left: VecDeque<f32>,

    /// Delay buffer for right channel
    delay_buffer_right: VecDeque<f32>,

    /// Current gain reduction factor (1.0 = no reduction, 0.5 = -6dB reduction)
    current_gain: f32,

    /// Attack coefficient for smoothing gain reduction (how fast gain drops)
    attack_coeff: f32,

    /// Release coefficient for smoothing gain recovery (how fast gain returns)
    release_coeff: f32,

    /// Monotonic deque for efficient sliding window maximum tracking
    /// Stores peaks in descending order with their positions
    peak_queue: VecDeque<PeakEntry>,

    /// Current write position in the circular buffer (for peak tracking)
    write_pos: usize,
}

impl LookAheadLimiter {
    /// Create a new look-ahead limiter.
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0)
    /// * `lookahead_ms` - Look-ahead time in milliseconds (typically 5-10ms)
    /// * `threshold` - Peak threshold (0.0-1.0), typically 0.98 or 0.99
    /// * `attack_ms` - Attack time for gain smoothing (typically 0.1-1.0ms)
    /// * `release_ms` - Release time for gain smoothing (typically 50-100ms)
    pub fn new(
        sample_rate: f32,
        lookahead_ms: f32,
        threshold: f32,
        attack_ms: f32,
        release_ms: f32,
    ) -> Self {
        let lookahead_samples = (lookahead_ms * sample_rate / 1000.0).round() as usize;

        // Calculate smoothing coefficients (one-pole lowpass)
        let attack_coeff = (-1.0 / (attack_ms * sample_rate / 1000.0)).exp();
        let release_coeff = (-1.0 / (release_ms * sample_rate / 1000.0)).exp();

        // Pre-allocate delay buffers filled with zeros
        let mut delay_buffer_left = VecDeque::with_capacity(lookahead_samples);
        let mut delay_buffer_right = VecDeque::with_capacity(lookahead_samples);
        for _ in 0..lookahead_samples {
            delay_buffer_left.push_back(0.0);
            delay_buffer_right.push_back(0.0);
        }

        Self {
            sample_rate,
            lookahead_samples,
            threshold,
            delay_buffer_left,
            delay_buffer_right,
            current_gain: 1.0,
            attack_coeff,
            release_coeff,
            peak_queue: VecDeque::with_capacity(lookahead_samples),
            write_pos: 0,
        }
    }

    /// Process a stereo sample pair through the look-ahead limiter.
    ///
    /// This is the main processing function. It:
    /// 1. Adds the incoming sample to the delay buffer
    /// 2. Efficiently tracks the peak using a monotonic deque (O(1) amortized)
    /// 3. Calculates required gain reduction
    /// 4. Applies smoothed gain to the oldest (delayed) sample
    /// 5. Returns the limited output
    ///
    /// # Arguments
    /// * `left` - Input sample for left channel
    /// * `right` - Input sample for right channel
    ///
    /// # Returns
    /// Tuple of (limited_left, limited_right)
    #[inline]
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Add new samples to the end of the delay buffers
        self.delay_buffer_left.push_back(left);
        self.delay_buffer_right.push_back(right);

        // Calculate peak of incoming sample (stereo max)
        let incoming_peak = left.abs().max(right.abs());

        // Remove peaks that have exited the look-ahead window
        let oldest_valid_pos = self.write_pos.saturating_sub(self.lookahead_samples);
        while let Some(entry) = self.peak_queue.front() {
            if entry.position < oldest_valid_pos {
                self.peak_queue.pop_front();
            } else {
                break;
            }
        }

        // Maintain monotonic decreasing order: remove smaller peaks from back
        // If new peak is larger than existing ones, they can never be the maximum
        while let Some(entry) = self.peak_queue.back() {
            if incoming_peak >= entry.peak {
                self.peak_queue.pop_back();
            } else {
                break;
            }
        }

        // Add new peak to the back
        self.peak_queue.push_back(PeakEntry {
            peak: incoming_peak,
            position: self.write_pos,
        });

        // The front of the queue is always the maximum peak in the window
        let max_peak = self.peak_queue.front().map(|e| e.peak).unwrap_or(0.0);

        // Increment write position
        self.write_pos = self.write_pos.wrapping_add(1);

        // Calculate target gain to keep peak under threshold
        let target_gain = if max_peak > self.threshold {
            self.threshold / max_peak
        } else {
            1.0
        };

        // Smooth the gain reduction to avoid artifacts
        let coeff = if target_gain < self.current_gain {
            self.attack_coeff // Fast attack when we need to limit
        } else {
            self.release_coeff // Slow release when peak subsides
        };
        self.current_gain = coeff * self.current_gain + (1.0 - coeff) * target_gain;

        // Remove the oldest samples (front of buffer) - these are now limited
        let delayed_left = self.delay_buffer_left.pop_front().unwrap_or(0.0);
        let delayed_right = self.delay_buffer_right.pop_front().unwrap_or(0.0);

        // Apply gain reduction
        let limited_left = delayed_left * self.current_gain;
        let limited_right = delayed_right * self.current_gain;

        // Final safety clamp (should rarely engage with look-ahead)
        (
            limited_left.clamp(-1.0, 1.0),
            limited_right.clamp(-1.0, 1.0),
        )
    }

    /// Reset the limiter state (clears delay buffers and resets gain).
    /// Useful when stopping playback or switching presets.
    pub fn reset(&mut self) {
        self.delay_buffer_left.clear();
        self.delay_buffer_right.clear();
        for _ in 0..self.lookahead_samples {
            self.delay_buffer_left.push_back(0.0);
            self.delay_buffer_right.push_back(0.0);
        }
        self.current_gain = 1.0;
        self.peak_queue.clear();
        self.write_pos = 0;
    }

    /// Get the current gain reduction amount (for metering/visualization)
    /// Returns 1.0 for no reduction, lower values indicate active limiting
    pub fn get_gain_reduction(&self) -> f32 {
        self.current_gain
    }

    /// Get the current latency in samples introduced by the look-ahead buffer
    pub fn get_latency_samples(&self) -> usize {
        self.lookahead_samples
    }

    /// Get the current latency in milliseconds
    pub fn get_latency_ms(&self) -> f32 {
        self.lookahead_samples as f32 * 1000.0 / self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_limiter_creation() {
        let limiter = LookAheadLimiter::new(44100.0, 5.0, 0.98, 0.5, 50.0);
        assert_eq!(limiter.lookahead_samples, 220); // 5ms at 44.1kHz = 220.5 ≈ 220
        assert_eq!(limiter.get_latency_samples(), 220);
        assert_relative_eq!(limiter.get_latency_ms(), 5.0, epsilon = 0.1);
        assert_relative_eq!(limiter.get_gain_reduction(), 1.0, epsilon = 0.001);
    }

    #[test]
    fn test_limiter_passes_quiet_signal() {
        let mut limiter = LookAheadLimiter::new(44100.0, 5.0, 0.98, 0.5, 50.0);

        // Process 1000 samples at 0.5 amplitude (well below threshold)
        for _ in 0..1000 {
            let (out_l, out_r) = limiter.process(0.5, 0.5);
            // After delay buffer fills, output should match input
            if out_l != 0.0 {
                assert_relative_eq!(out_l, 0.5, epsilon = 0.01);
                assert_relative_eq!(out_r, 0.5, epsilon = 0.01);
            }
        }

        // No gain reduction should have occurred
        assert_relative_eq!(limiter.get_gain_reduction(), 1.0, epsilon = 0.01);
    }

    #[test]
    fn test_limiter_catches_peak() {
        let mut limiter = LookAheadLimiter::new(44100.0, 5.0, 0.98, 0.5, 50.0);

        // Fill delay buffer with quiet signal
        for _ in 0..300 {
            limiter.process(0.1, 0.1);
        }

        // Send a peak that exceeds threshold
        limiter.process(1.5, 1.5);

        // Process more samples and check that gain reduction occurred
        let mut max_output = 0.0f32;
        for _ in 0..300 {
            let (out_l, out_r) = limiter.process(0.1, 0.1);
            max_output = max_output.max(out_l.abs().max(out_r.abs()));
        }

        // Gain reduction should have engaged
        assert!(limiter.get_gain_reduction() < 0.99);

        // Peak should be caught and limited (might see the 1.5 peak delayed but reduced)
        // Due to look-ahead, the peak shouldn't exceed threshold by much
        assert!(max_output <= 1.0, "Peak should be limited to safe range");
    }

    #[test]
    fn test_limiter_reset() {
        let mut limiter = LookAheadLimiter::new(44100.0, 5.0, 0.98, 0.5, 50.0);

        // Process some samples to fill buffer
        for _ in 0..300 {
            limiter.process(0.8, 0.8);
        }

        // Reset
        limiter.reset();

        // Verify state is clean
        assert_relative_eq!(limiter.get_gain_reduction(), 1.0, epsilon = 0.001);

        // Process a sample - output should be 0 (buffer was cleared)
        let (out_l, out_r) = limiter.process(0.5, 0.5);
        assert_relative_eq!(out_l, 0.0, epsilon = 0.001);
        assert_relative_eq!(out_r, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_limiter_latency() {
        let mut limiter = LookAheadLimiter::new(44100.0, 5.0, 0.98, 0.5, 50.0);
        let latency = limiter.get_latency_samples();

        // Process latency + 1 samples of a step function
        for i in 0..(latency + 50) {
            let input = if i < latency { 0.0 } else { 0.7 };
            let (out_l, _) = limiter.process(input, input);

            // Output should lag input by exactly latency samples
            if i == latency {
                // First non-zero output should appear after latency samples
                assert_relative_eq!(out_l, 0.0, epsilon = 0.001);
            } else if i == latency + 1 {
                // Next sample should have the step
                assert!(out_l > 0.5);
            }
        }
    }
}
