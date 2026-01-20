/// Vocal Choir Effect
///
/// Creates a lush ensemble of multiple pitch-shifted voices, simulating a choir.
/// Unlike doubler (2 voices) or chorus (modulated), choir uses 4-8 static voices
/// with carefully tuned detuning and delay spread for maximum richness.
///
/// Features:
/// - 4-8 independent voices with unique delays and detuning
/// - Stereo panning spread across the stereo field
/// - Adjustable detune amount (±10-30 cents typical)
/// - Delay spread (10-40ms) for ensemble depth
/// - Dry/wet mix control
///
/// Algorithm:
/// 1. Create multiple delay lines with different times
/// 2. Apply pitch shifting via fractional delay modulation
/// 3. Pan voices across stereo field
/// 4. Sum all voices with appropriate levels
use std::collections::VecDeque;

const MAX_DELAY_MS: f32 = 50.0;
const MAX_VOICES: usize = 8;

/// Single voice in the choir ensemble
struct ChoirVoice {
    buffer: VecDeque<f32>,
    delay_ms: f32,
    detune_cents: f32,
    pan: f32, // -1.0 (left) to +1.0 (right)
}

impl ChoirVoice {
    fn new(max_delay_samples: usize, delay_ms: f32, detune_cents: f32, pan: f32) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_delay_samples + 1),
            delay_ms,
            detune_cents,
            pan,
        }
    }

    /// Process sample and return (left, right) output
    fn process(&mut self, input: f32, sample_rate: f32) -> (f32, f32) {
        // Push new sample
        self.buffer.push_back(input);

        // Maintain buffer size
        let max_len = self.buffer.capacity();
        while self.buffer.len() > max_len {
            self.buffer.pop_front();
        }

        // Calculate delay with pitch shifting
        let base_delay_samples = (self.delay_ms / 1000.0) * sample_rate;
        let pitch_ratio = 2.0_f32.powf(self.detune_cents / 1200.0);
        let delay_samples = base_delay_samples * pitch_ratio;

        // Read delayed sample with interpolation
        let sample = self.read_delayed(delay_samples);

        // Apply panning
        let pan_normalized = (self.pan + 1.0) * 0.5; // Convert -1..1 to 0..1
        let left_gain = (1.0 - pan_normalized).sqrt();
        let right_gain = pan_normalized.sqrt();

        (sample * left_gain, sample * right_gain)
    }

    fn read_delayed(&self, delay_samples: f32) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }

        let delay_samples = delay_samples.clamp(0.0, self.buffer.len() as f32 - 1.0);
        let read_pos = self.buffer.len() as f32 - delay_samples - 1.0;

        if read_pos < 0.0 {
            return 0.0;
        }

        // Linear interpolation
        let pos_int = read_pos.floor() as usize;
        let pos_frac = read_pos - read_pos.floor();

        let sample1 = self.buffer.get(pos_int).copied().unwrap_or(0.0);
        let sample2 = self.buffer.get(pos_int + 1).copied().unwrap_or(sample1);

        sample1 * (1.0 - pos_frac) + sample2 * pos_frac
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}

pub struct VocalChoir {
    sample_rate: f32,
    voices: Vec<ChoirVoice>,
    
    // Parameters
    num_voices: usize,        // 2-8 voices
    detune_amount: f32,       // 0-30 cents (total spread)
    delay_spread: f32,        // 10-40ms (delay range)
    stereo_spread: f32,       // 0.0-1.0 (panning width)
    mix: f32,                 // 0.0-1.0 (dry/wet)
    
    max_delay_samples: usize,
}

impl VocalChoir {
    pub fn new(sample_rate: f32) -> Self {
        let max_delay_samples = ((MAX_DELAY_MS / 1000.0) * sample_rate).ceil() as usize;
        
        let mut choir = Self {
            sample_rate,
            voices: Vec::new(),
            num_voices: 4,
            detune_amount: 15.0,
            delay_spread: 25.0,
            stereo_spread: 0.8,
            mix: 0.5,
            max_delay_samples,
        };
        
        choir.rebuild_voices();
        choir
    }
    
    /// Set number of voices (2-8)
    pub fn set_num_voices(&mut self, num: usize) {
        let new_num = num.clamp(2, MAX_VOICES);
        if new_num != self.num_voices {
            self.num_voices = new_num;
            self.rebuild_voices();
        }
    }
    
    /// Set detune amount in cents (0-30 typical)
    /// This is the total spread, so voices will be ±(amount/2)
    pub fn set_detune_amount(&mut self, cents: f32) {
        self.detune_amount = cents.clamp(0.0, 50.0);
        self.rebuild_voices();
    }
    
    /// Set delay spread in milliseconds (10-40 typical)
    pub fn set_delay_spread(&mut self, ms: f32) {
        self.delay_spread = ms.clamp(5.0, MAX_DELAY_MS);
        self.rebuild_voices();
    }
    
    /// Set stereo spread (0.0 = mono, 1.0 = full stereo)
    pub fn set_stereo_spread(&mut self, spread: f32) {
        self.stereo_spread = spread.clamp(0.0, 1.0);
        self.rebuild_voices();
    }
    
    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
    
    /// Rebuild voice array with new parameters
    fn rebuild_voices(&mut self) {
        self.voices.clear();
        
        for i in 0..self.num_voices {
            // Distribute detuning evenly across voices
            // Formula: detune = (i - center) * (amount / (num_voices - 1))
            let center = (self.num_voices - 1) as f32 / 2.0;
            let detune_factor = if self.num_voices > 1 {
                (i as f32 - center) / ((self.num_voices - 1) as f32 / 2.0)
            } else {
                0.0
            };
            let detune_cents = detune_factor * (self.detune_amount / 2.0);
            
            // Distribute delays evenly
            let delay_factor = i as f32 / (self.num_voices - 1).max(1) as f32;
            let delay_ms = 10.0 + delay_factor * self.delay_spread;
            
            // Distribute panning across stereo field
            let pan = if self.num_voices > 1 {
                -self.stereo_spread + (i as f32 / (self.num_voices - 1) as f32) * 2.0 * self.stereo_spread
            } else {
                0.0 // Center for single voice
            };
            
            let voice = ChoirVoice::new(
                self.max_delay_samples,
                delay_ms,
                detune_cents,
                pan,
            );
            self.voices.push(voice);
        }
    }
    
    /// Process stereo sample through choir
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        // Mix input to mono for processing
        let mono_in = (left_in + right_in) * 0.5;
        
        // Process through all voices and sum
        let mut left_sum = 0.0;
        let mut right_sum = 0.0;
        
        for voice in &mut self.voices {
            let (l, r) = voice.process(mono_in, self.sample_rate);
            left_sum += l;
            right_sum += r;
        }
        
        // Normalize by number of voices to prevent clipping
        let voice_count = self.voices.len() as f32;
        left_sum /= voice_count;
        right_sum /= voice_count;
        
        // Mix with dry signal
        let left_out = left_in * (1.0 - self.mix) + left_sum * self.mix;
        let right_out = right_in * (1.0 - self.mix) + right_sum * self.mix;
        
        (left_out, right_out)
    }
    
    /// Reset all voice buffers
    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_choir_creation() {
        let choir = VocalChoir::new(44100.0);
        assert_eq!(choir.num_voices, 4);
        assert_eq!(choir.voices.len(), 4);
    }
    
    #[test]
    fn test_num_voices_clamping() {
        let mut choir = VocalChoir::new(44100.0);
        
        // Below minimum
        choir.set_num_voices(1);
        assert_eq!(choir.num_voices, 2);
        assert_eq!(choir.voices.len(), 2);
        
        // Above maximum
        choir.set_num_voices(10);
        assert_eq!(choir.num_voices, MAX_VOICES);
        assert_eq!(choir.voices.len(), MAX_VOICES);
        
        // Normal range
        choir.set_num_voices(6);
        assert_eq!(choir.num_voices, 6);
        assert_eq!(choir.voices.len(), 6);
    }
    
    #[test]
    fn test_voice_distribution() {
        let mut choir = VocalChoir::new(44100.0);
        choir.set_num_voices(4);
        choir.set_detune_amount(20.0); // ±10 cents
        choir.set_stereo_spread(1.0);  // Full stereo
        
        assert_eq!(choir.voices.len(), 4);
        
        // Check that detuning is distributed (center voices near 0, outer voices ±10)
        let center_voice = &choir.voices[1]; // Second voice
        assert!(center_voice.detune_cents.abs() < 7.0, "Center voice should be near 0 cents");
        
        let outer_voice = &choir.voices[3]; // Last voice
        assert!(outer_voice.detune_cents.abs() > 5.0, "Outer voice should be detuned");
        
        // Check stereo panning
        assert!(choir.voices[0].pan < -0.3, "First voice should be panned left");
        assert!(choir.voices[3].pan > 0.3, "Last voice should be panned right");
    }
    
    #[test]
    fn test_stereo_output() {
        let mut choir = VocalChoir::new(44100.0);
        choir.set_num_voices(4);
        choir.set_mix(1.0); // 100% wet
        
        // Test 1: Zero stereo spread should give equal outputs
        choir.set_stereo_spread(0.0);
        
        // Fill buffers with varying signal (sine wave to make phase differences visible)
        for i in 0..5000 {
            let t = i as f32 / 44100.0;
            let sine = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
            choir.process(sine, sine);
        }
        
        let (left1, right1) = choir.process(0.5, 0.5);
        
        // With zero spread, panning should be center → equal outputs
        assert_relative_eq!(left1, right1, epsilon = 0.001);
        
        // Test 2: Non-zero stereo spread should give different outputs  
        choir.set_stereo_spread(1.0); // 100% spread
        
        // Process more samples to let detuning take effect
        for i in 0..5000 {
            let t = i as f32 / 44100.0;
            let sine = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
            choir.process(sine, sine);
        }
        
        let (left2, right2) = choir.process(0.5, 0.5);
        
        // With spread, outputs should differ (not equal within tight tolerance)
        let diff = (left2 - right2).abs();
        assert!(diff > 0.01, "Stereo spread should create L/R difference, got diff={}", diff);
        
        // Outputs should be reasonable (not clipping)
        assert!(left2.abs() < 2.0, "Left output should be in reasonable range");
        assert!(right2.abs() < 2.0, "Right output should be in reasonable range");
    }
    
    #[test]
    fn test_dry_wet_mix() {
        let mut choir = VocalChoir::new(44100.0);
        
        let input_left = 0.5;
        let input_right = -0.5;
        
        // 0% mix = dry only
        choir.set_mix(0.0);
        let (left, right) = choir.process(input_left, input_right);
        assert_relative_eq!(left, input_left, epsilon = 0.001);
        assert_relative_eq!(right, input_right, epsilon = 0.001);
    }
    
    #[test]
    fn test_mono_to_stereo_expansion() {
        let mut choir = VocalChoir::new(44100.0);
        choir.set_num_voices(6);
        choir.set_stereo_spread(1.0); // Full spread
        choir.set_mix(1.0); // 100% wet
        
        // Fill buffers with mono input
        for _ in 0..500 {
            choir.process(1.0, 1.0);
        }
        
        let (left, right) = choir.process(1.0, 1.0);
        
        // Mono input should become stereo output due to panning
        assert_ne!(left, right, "Mono input should expand to stereo");
    }
    
    #[test]
    fn test_detune_distribution() {
        let mut choir = VocalChoir::new(44100.0);
        choir.set_num_voices(5);
        choir.set_detune_amount(20.0); // ±10 cents spread
        
        // Check that center voice is near 0 cents
        let center_idx = 2; // Middle of 5 voices
        assert!(
            choir.voices[center_idx].detune_cents.abs() < 2.0,
            "Center voice should be close to 0 cents, got {}",
            choir.voices[center_idx].detune_cents
        );
        
        // Check that outer voices are detuned
        assert!(
            choir.voices[0].detune_cents < -5.0,
            "First voice should be detuned down"
        );
        assert!(
            choir.voices[4].detune_cents > 5.0,
            "Last voice should be detuned up"
        );
    }
}
