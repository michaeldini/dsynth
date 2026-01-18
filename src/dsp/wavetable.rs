//! Wavetable data structure for wavetable synthesis
//!
//! A wavetable is a single-cycle waveform stored as an array of samples.
//! This implementation stores both normal-rate and 4× oversampled versions
//! for anti-aliasing during playback.

/// A single wavetable: one snapshot of a waveform
/// Stored at both normal and 4× oversampled rates for anti-aliasing
#[derive(Clone, Debug)]
pub struct Wavetable {
    /// Name of the wavetable (e.g., "Serum Saw 1", "Vital Buzzy")
    name: String,

    /// The actual waveform samples at normal sample rate
    /// Length: typically 2048 samples (covers one cycle)
    samples: Vec<f32>,

    /// Oversampled version at 4× rate (for anti-aliasing)
    /// Length: typically 8192 samples
    /// Will be downsampled during playback using existing Downsampler
    samples_4x: Vec<f32>,
}

impl Wavetable {
    /// Create a new wavetable from samples
    ///
    /// # Arguments
    /// * `name` - Human-readable name for this wavetable
    /// * `samples` - Single-cycle waveform (typically 2048 samples)
    ///
    /// The 4× oversampled version is automatically generated using cubic interpolation
    pub fn new(name: String, samples: Vec<f32>) -> Self {
        let samples_4x = Self::generate_oversampled(&samples);
        Self {
            name,
            samples,
            samples_4x,
        }
    }

    /// Get the wavetable name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the number of samples at normal rate
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Return true if the wavetable has no samples
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get access to the 4× oversampled buffer
    ///
    /// Returns a slice containing the pre-computed 4× oversampled version
    /// of this wavetable. Used by Oscillator to cache wavetable data.
    pub fn samples_4x(&self) -> &[f32] {
        &self.samples_4x
    }

    /// Linear interpolation lookup at normalized phase [0.0, 1.0)
    ///
    /// # Arguments
    /// * `phase` - Normalized phase position (0.0 to 1.0)
    ///
    /// # Returns
    /// Interpolated sample value in range [-1.0, 1.0]
    #[inline]
    pub fn lookup(&self, phase: f32) -> f32 {
        let index = phase * self.samples.len() as f32;
        let i0 = index.floor() as usize % self.samples.len();
        let i1 = (i0 + 1) % self.samples.len();
        let frac = index.fract();

        self.samples[i0] * (1.0 - frac) + self.samples[i1] * frac
    }

    /// Lookup at 4× oversampled rate (for anti-aliasing through Downsampler)
    ///
    /// # Arguments
    /// * `phase` - Normalized phase position (0.0 to 1.0)
    ///
    /// # Returns
    /// Interpolated sample value from 4× oversampled buffer
    #[inline]
    pub fn lookup_4x(&self, phase: f32) -> f32 {
        let index = phase * self.samples_4x.len() as f32;
        let i0 = index.floor() as usize % self.samples_4x.len();
        let i1 = (i0 + 1) % self.samples_4x.len();
        let frac = index.fract();

        self.samples_4x[i0] * (1.0 - frac) + self.samples_4x[i1] * frac
    }

    /// Morph between two wavetables using linear cross-fade
    ///
    /// # Arguments
    /// * `wt1` - First wavetable
    /// * `wt2` - Second wavetable
    /// * `phase` - Phase position (0.0 to 1.0)
    /// * `morph_amount` - Morph factor (0.0 = wt1, 1.0 = wt2)
    ///
    /// # Returns
    /// Morphed sample value
    #[inline]
    pub fn morph_lookup(wt1: &Wavetable, wt2: &Wavetable, phase: f32, morph_amount: f32) -> f32 {
        let sample1 = wt1.lookup(phase);
        let sample2 = wt2.lookup(phase);
        sample1 * (1.0 - morph_amount) + sample2 * morph_amount
    }

    /// Morph between two wavetables at 4× oversampled rate
    ///
    /// # Arguments
    /// * `wt1` - First wavetable
    /// * `wt2` - Second wavetable
    /// * `phase` - Phase position (0.0 to 1.0)
    /// * `morph_amount` - Morph factor (0.0 = wt1, 1.0 = wt2)
    ///
    /// # Returns
    /// Morphed sample value from 4× oversampled buffers
    #[inline]
    pub fn morph_lookup_4x(wt1: &Wavetable, wt2: &Wavetable, phase: f32, morph_amount: f32) -> f32 {
        let sample1 = wt1.lookup_4x(phase);
        let sample2 = wt2.lookup_4x(phase);
        sample1 * (1.0 - morph_amount) + sample2 * morph_amount
    }

    /// Generate 4× oversampled version using cubic interpolation
    ///
    /// Uses Catmull-Rom spline interpolation for smooth upsampling
    fn generate_oversampled(samples: &[f32]) -> Vec<f32> {
        let len = samples.len();
        let oversample_len = len * 4;
        let mut oversampled = Vec::with_capacity(oversample_len);

        for i in 0..oversample_len {
            let pos = (i as f32) / 4.0;
            let idx = pos.floor() as usize;
            let frac = pos.fract();

            // Get 4 points for cubic interpolation (wrap around)
            let p0 = samples[(idx + len - 1) % len];
            let p1 = samples[idx % len];
            let p2 = samples[(idx + 1) % len];
            let p3 = samples[(idx + 2) % len];

            // Catmull-Rom spline coefficients
            let a = -0.5 * p0 + 1.5 * p1 - 1.5 * p2 + 0.5 * p3;
            let b = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
            let c = -0.5 * p0 + 0.5 * p2;
            let d = p1;

            // Evaluate cubic polynomial
            let sample = a * frac * frac * frac + b * frac * frac + c * frac + d;
            oversampled.push(sample);
        }

        oversampled
    }

    /// Create a sine wave wavetable (for testing/fallback)
    pub fn sine(name: String, num_samples: usize) -> Self {
        use std::f32::consts::PI;
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| (2.0 * PI * i as f32 / num_samples as f32).sin())
            .collect();
        Self::new(name, samples)
    }

    /// Create a sawtooth wave wavetable (for testing/fallback)
    pub fn sawtooth(name: String, num_samples: usize) -> Self {
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| 2.0 * (i as f32 / num_samples as f32) - 1.0)
            .collect();
        Self::new(name, samples)
    }

    /// Load a wavetable from raw .wav file bytes (for embedded wavetables)
    ///
    /// # Arguments
    /// * `wav_bytes` - Raw .wav file content as bytes
    /// * `name` - Name to assign to this wavetable
    ///
    /// # Returns
    /// Result containing the loaded wavetable or an error message
    pub fn from_wav_bytes(wav_bytes: &[u8], name: String) -> Result<Self, String> {
        use std::io::Cursor;

        // Use hound to read the WAV data from memory
        let cursor = Cursor::new(wav_bytes);
        let mut reader =
            hound::WavReader::new(cursor).map_err(|e| format!("Failed to parse WAV: {}", e))?;

        let spec = reader.spec();

        // Read all samples from the WAV file
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => reader
                .samples::<f32>()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to read samples: {}", e))?,
            hound::SampleFormat::Int => {
                // Convert integer samples to float (-1.0 to 1.0)
                let max_value = (1 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / max_value))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Failed to read samples: {}", e))?
            }
        };

        if samples.is_empty() {
            return Err("WAV file contains no samples".to_string());
        }

        // Convert stereo to mono by averaging channels
        let mono_samples: Vec<f32> = if spec.channels == 1 {
            samples
        } else {
            samples
                .chunks(spec.channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
                .collect()
        };

        // Create the wavetable (4× oversampling is generated automatically in new())
        Ok(Self::new(name, mono_samples))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_wavetable_lookup_interpolation() {
        // Create a simple linear ramp wavetable
        let samples: Vec<f32> = (0..8).map(|i| i as f32 / 7.0).collect();
        let wt = Wavetable::new("test".to_string(), samples);

        // Test exact sample positions
        assert_relative_eq!(wt.lookup(0.0), 0.0, epsilon = 0.001);
        assert_relative_eq!(wt.lookup(0.125), 1.0 / 7.0, epsilon = 0.001);

        // Test interpolated positions (should be linear)
        assert_relative_eq!(wt.lookup(0.0625), 0.5 / 7.0, epsilon = 0.001);
    }

    #[test]
    fn test_wavetable_morphing() {
        // Create two wavetables: one all zeros, one all ones
        let wt1 = Wavetable::new("zeros".to_string(), vec![0.0; 8]);
        let wt2 = Wavetable::new("ones".to_string(), vec![1.0; 8]);

        // Test morphing at various positions
        assert_relative_eq!(
            Wavetable::morph_lookup(&wt1, &wt2, 0.0, 0.0),
            0.0,
            epsilon = 0.001
        );
        assert_relative_eq!(
            Wavetable::morph_lookup(&wt1, &wt2, 0.0, 0.5),
            0.5,
            epsilon = 0.001
        );
        assert_relative_eq!(
            Wavetable::morph_lookup(&wt1, &wt2, 0.0, 1.0),
            1.0,
            epsilon = 0.001
        );
    }

    #[test]
    fn test_wavetable_sine() {
        let wt = Wavetable::sine("sine".to_string(), 2048);

        // Test at known phase positions
        assert_relative_eq!(wt.lookup(0.0), 0.0, epsilon = 0.01);
        assert_relative_eq!(wt.lookup(0.25), 1.0, epsilon = 0.01); // Peak at 90°
        assert_relative_eq!(wt.lookup(0.5), 0.0, epsilon = 0.01);
        assert_relative_eq!(wt.lookup(0.75), -1.0, epsilon = 0.01); // Trough at 270°
    }

    #[test]
    fn test_wavetable_4x_oversampling() {
        let wt = Wavetable::sine("sine".to_string(), 2048);

        // Verify 4× oversampled buffer has 4× the samples
        assert_eq!(wt.samples_4x.len(), 2048 * 4);

        // Verify oversampled lookup gives similar results
        let normal = wt.lookup(0.25);
        let oversampled = wt.lookup_4x(0.25);
        assert_relative_eq!(normal, oversampled, epsilon = 0.05);
    }
}
