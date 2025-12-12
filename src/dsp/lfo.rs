use crate::params::LFOWaveform;
use std::f32::consts::PI;

/// Low Frequency Oscillator for modulation
pub struct LFO {
    sample_rate: f32,
    phase: f32,
    waveform: LFOWaveform,
    rate: f32, // Hz
}

impl LFO {
    /// Create a new LFO
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            waveform: LFOWaveform::Sine,
            rate: 2.0,
        }
    }

    /// Set the LFO rate in Hz
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.clamp(0.01, 20.0);
    }

    /// Set the LFO waveform
    pub fn set_waveform(&mut self, waveform: LFOWaveform) {
        self.waveform = waveform;
    }

    /// Generate next LFO sample
    /// Returns a value between -1.0 and 1.0
    pub fn process(&mut self) -> f32 {
        let output = match self.waveform {
            LFOWaveform::Sine => {
                (self.phase * 2.0 * PI).sin()
            }
            LFOWaveform::Triangle => {
                // Triangle: rises 0→1 in first half, falls 1→0 in second half
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    -4.0 * self.phase + 3.0
                }
            }
            LFOWaveform::Square => {
                if self.phase < 0.5 { 1.0 } else { -1.0 }
            }
            LFOWaveform::Saw => {
                // Saw: rises -1→1 linearly
                2.0 * self.phase - 1.0
            }
        };

        // Advance phase
        let phase_increment = self.rate / self.sample_rate;
        self.phase += phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output
    }

    /// Reset phase to 0
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfo_creation() {
        let lfo = LFO::new(44100.0);
        assert_eq!(lfo.sample_rate, 44100.0);
        assert_eq!(lfo.phase, 0.0);
        assert_eq!(lfo.rate, 2.0);
    }

    #[test]
    fn test_lfo_sine_output_range() {
        let mut lfo = LFO::new(44100.0);
        lfo.set_waveform(LFOWaveform::Sine);
        lfo.set_rate(2.0);

        // Check multiple samples
        for _ in 0..1000 {
            let sample = lfo.process();
            assert!(sample >= -1.0 && sample <= 1.0, "LFO output out of range: {}", sample);
        }
    }

    #[test]
    fn test_lfo_triangle() {
        let mut lfo = LFO::new(44100.0);
        lfo.set_waveform(LFOWaveform::Triangle);
        lfo.set_rate(1.0);

        // Check range
        for _ in 0..100 {
            let sample = lfo.process();
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_lfo_square() {
        let mut lfo = LFO::new(44100.0);
        lfo.set_waveform(LFOWaveform::Square);
        lfo.set_rate(1.0);

        let sample = lfo.process();
        assert!(sample == 1.0 || sample == -1.0);
    }

    #[test]
    fn test_lfo_saw() {
        let mut lfo = LFO::new(44100.0);
        lfo.set_waveform(LFOWaveform::Saw);
        lfo.set_rate(1.0);

        for _ in 0..100 {
            let sample = lfo.process();
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_lfo_rate_clamping() {
        let mut lfo = LFO::new(44100.0);
        
        lfo.set_rate(0.001); // Too low
        assert_eq!(lfo.rate, 0.01);

        lfo.set_rate(100.0); // Too high
        assert_eq!(lfo.rate, 20.0);
    }

    #[test]
    fn test_lfo_reset() {
        let mut lfo = LFO::new(44100.0);
        
        // Advance phase
        for _ in 0..100 {
            lfo.process();
        }
        
        assert!(lfo.phase > 0.0);
        
        lfo.reset();
        assert_eq!(lfo.phase, 0.0);
    }

    #[test]
    fn test_lfo_phase_wraps() {
        let mut lfo = LFO::new(44100.0);
        lfo.set_rate(20.0); // Fast rate to wrap quickly

        // Process many samples
        for _ in 0..10000 {
            lfo.process();
            assert!(lfo.phase >= 0.0 && lfo.phase < 1.0, "Phase not wrapping correctly");
        }
    }
}
