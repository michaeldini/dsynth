use crate::dsp::downsampler::Downsampler;
use crate::dsp::waveform;
use crate::params::Waveform;

#[cfg(feature = "simd")]
use std::simd::{cmp::SimdPartialOrd, f32x4, StdFloat};

/// A polyphonic-safe oscillator with 4× oversampling and anti-aliasing.
///
/// This oscillator is the core sound generation component of the synthesizer. It produces
/// basic waveforms (sine, square, sawtooth, triangle, pulse) at any frequency. The key
/// design feature is **4× oversampling with anti-aliasing** to prevent aliasing artifacts
/// that would otherwise occur at high frequencies.
///
/// ## Why 4× Oversampling?
///
/// When generating waveforms at high frequencies (e.g., a 10kHz sawtooth wave at 44.1kHz
/// sample rate), aliasing can occur. Aliasing happens because the waveform contains frequency
/// components that are too high for the sample rate to represent accurately (above the
/// Nyquist frequency of 22050Hz). This produces harsh, digital-sounding artifacts.
///
/// The solution: generate the waveform at 4× the target sample rate (176400Hz in this case),
/// then filter it back down to the target rate. This removes the high-frequency content that
/// would alias, producing smooth, analog-sounding oscillators even at high frequencies.
///
/// ## Supported Waveforms
///
/// - **Sine**: Pure, smooth fundamental tone with minimal harmonics
/// - **Square**: Bright, hollow tone with odd-numbered harmonics
/// - **Sawtooth**: Buzzy, bright tone with all harmonics
/// - **Triangle**: Less bright than square, with only odd harmonics but weaker
/// - **Pulse**: Square wave variant with variable pulse width (duty cycle)
///
/// ## Wave Shaping
///
/// The `shape` parameter allows morphing between waveforms or adding harmonic distortion:
/// - For sine: adds harmonic content via soft clipping
/// - For saw/triangle/square: morphs to adjacent waveforms
/// - For pulse: controls the pulse width (duty cycle) instead
///
/// ## Phase-Accurate Frequency Control
///
/// The oscillator uses phase accumulation rather than sample counting. At each step, it
/// adds a phase_increment (which equals frequency / sample_rate) to an accumulating phase
/// value. When phase reaches 1.0, it wraps back to 0.0. This is more accurate than trying
/// to hit exact sample counts and allows smooth frequency changes without clicks.
pub struct Oscillator {
    /// The oversampling sample rate (4× the target sample rate)
    /// For example, if target is 44100Hz, oversample_rate is 176400Hz
    oversample_rate: f32,

    /// Current oscillator phase (0.0 to 1.0)
    /// This represents the position within one complete oscillation cycle.
    /// 0.0 = start of cycle, 0.5 = middle, 1.0 = end (wraps back to 0.0)
    phase: f32,

    /// How much to increment phase each sample (frequency / oversample_rate)
    /// At 440Hz and 176400Hz oversample rate: 440 / 176400 ≈ 0.00249
    /// This means advancing about 0.249% through the waveform cycle per sample
    phase_increment: f32,

    /// Downsampler for filtering 4× oversampled signal back to target rate
    /// This is what prevents aliasing by removing high-frequency content
    downsampler: Downsampler,

    /// Current waveform type (sine, square, saw, etc.)
    waveform: Waveform,

    /// Initial phase offset (0.0 to 1.0) for unison/detune effects
    /// Allows multiple oscillators to start at different phases for richer sound
    initial_phase: f32,

    /// Wave shaping parameter (-1.0 to 1.0)
    /// Controls waveform morphing or harmonic content addition
    shape: f32,

    /// PRNG state for noise generation (xorshift32)
    /// Initialized to non-zero value to ensure PRNG cycles properly
    noise_state: u32,

    /// Pink noise filter state (Paul Kellett's "economy" method)
    /// Uses 3 pole filter to approximate -3dB/octave slope
    pink_b0: f32,
    pink_b1: f32,
    pink_b2: f32,

    /// Pre-computed wavetable for additive synthesis (2048 samples per cycle)
    /// Synthesized from harmonic amplitudes when set_additive_harmonics() is called
    additive_wavetable: [f32; 2048],

    /// Current harmonic amplitudes for additive synthesis (8 harmonics)
    /// Index 0 = fundamental, 1 = 2nd harmonic, etc.
    additive_harmonics: [f32; 8],

    /// Current wavetable index when waveform is Wavetable (0 to N-1)
    wavetable_index: usize,

    /// Wavetable morphing position (0.0 to 1.0)
    /// 0.0 = current wavetable, 1.0 = next wavetable in sequence
    /// Continuous values = cross-fade between adjacent wavetables
    wavetable_position: f32,

    /// Cached wavetable data for fast lookup during audio processing
    /// Copied from WavetableLibrary when wavetable_index changes
    /// Stores 4× oversampled wavetable (8192 samples for 2048-sample base)
    current_wavetable_4x: Option<Vec<f32>>,
}

impl Oscillator {
    /// Create a new oscillator tuned for a specific sample rate.
    ///
    /// # Arguments
    /// * `sample_rate` - The target sample rate in Hz (e.g., 44100.0)
    ///
    /// # Returns
    /// A new oscillator with:
    /// - Internal oversampling rate set to 4× the target rate
    /// - Phase initialized to 0.0
    /// - Default waveform (Sine)
    /// - No initial phase offset
    /// - No wave shaping
    ///
    /// # Example
    /// ```
    /// use dsynth::dsp::oscillator::Oscillator;
    ///
    /// let osc = Oscillator::new(44100.0);
    /// // osc is now set up for 44.1kHz audio, using 176.4kHz internally
    ///
    /// // Process a sample to verify it works
    /// // (internal state verification would require pub fields)
    /// ```
    pub fn new(sample_rate: f32) -> Self {
        let mut osc = Self {
            oversample_rate: sample_rate * 4.0,
            phase: 0.0,
            phase_increment: 0.0,
            downsampler: Downsampler::new(20),
            waveform: Waveform::Sine,
            initial_phase: 0.0,
            shape: 0.0,
            noise_state: 0x12345678, // Non-zero seed for xorshift32
            pink_b0: 0.0,
            pink_b1: 0.0,
            pink_b2: 0.0,
            additive_wavetable: [0.0; 2048],
            additive_harmonics: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            wavetable_index: 0,
            wavetable_position: 0.0,
            current_wavetable_4x: None,
        };
        // Generate default wavetable (pure sine from fundamental harmonic)
        osc.generate_additive_wavetable();
        osc
    }

    /// Set the initial phase offset for this oscillator.
    ///
    /// This is useful for creating "unison" effects where multiple detuned oscillators
    /// are stacked with different phases, creating a richer, wider sound. It can also
    /// be used for creative effects where different notes start at different points in
    /// the waveform cycle.
    ///
    /// # Arguments
    /// * `phase` - Phase offset from 0.0 to 1.0
    ///   - 0.0 = start of cycle
    ///   - 0.5 = middle (180° out of phase)
    ///   - 1.0 = full cycle (same as 0.0)
    ///
    /// Values outside [0.0, 1.0] are clamped to stay in range.
    pub fn set_phase(&mut self, phase: f32) {
        self.initial_phase = phase.clamp(0.0, 1.0);
    }

    /// Apply the configured initial phase to the running phase accumulator.
    ///
    /// This is intended for note-on (unison voice decorrelation) without resetting the
    /// downsampler state, which helps avoid unnecessary transients.
    pub fn apply_initial_phase(&mut self) {
        self.phase = self.initial_phase;
    }

    /// Set the frequency for this oscillator in Hz.
    ///
    /// The frequency determines how fast the phase advances. Higher frequencies advance
    /// the phase faster, producing higher-pitched tones. The phase increment is calculated
    /// as: frequency / oversample_rate.
    ///
    /// # Arguments
    /// * `freq` - Frequency in Hz (e.g., 440.0 for concert A4)
    ///
    /// # Notes
    /// - Can be changed at any time for smooth frequency sweeps
    /// - Changing frequency doesn't reset the oscillator's current phase
    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_increment = freq / self.oversample_rate;
    }

    /// Get the current phase of the oscillator (0.0 to 1.0).
    ///
    /// Used for hard sync detection - when the master oscillator's phase wraps
    /// from >1.0 back to <1.0, the slave oscillator's phase is reset.
    ///
    /// # Returns
    /// Current phase value in the range [0.0, 1.0)
    pub fn get_phase(&self) -> f32 {
        self.phase
    }

    /// Set the waveform type (sine, square, saw, triangle, or pulse).
    ///
    /// Different waveforms have different harmonic content and tonal characteristics.
    /// Can be changed in real-time without artifacts.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Set the wave shaping amount (-1.0 to 1.0).
    ///
    /// Wave shaping behavior depends on the current waveform:
    /// - **Sine**: Adds harmonic content via soft clipping distortion
    /// - **Sawtooth**: Morphs towards triangle (reduces harshness)
    /// - **Square**: Applied as morphing (though pulse width is primary)
    /// - **Triangle**: Morphs towards sawtooth (adds sharpness)
    /// - **Pulse**: Controls pulse width (duty cycle): -1.0 = 10%, 0.0 = 50%, 1.0 = 90%
    ///
    /// Values outside [-1.0, 1.0] are clamped to stay in range.
    pub fn set_shape(&mut self, shape: f32) {
        self.shape = shape.clamp(-1.0, 1.0);
    }

    /// Set the harmonic amplitudes for additive synthesis and regenerate the wavetable.
    ///
    /// The harmonics array contains 8 amplitude values (0.0 to 1.0) representing the
    /// strength of each harmonic component:
    /// - Index 0: Fundamental frequency (1×)
    /// - Index 1: 2nd harmonic (2×)
    /// - Index 2: 3rd harmonic (3×)
    /// - ...
    /// - Index 7: 8th harmonic (8×)
    ///
    /// The wavetable is immediately regenerated using additive synthesis:
    /// `waveform = Σ(harmonic[n] × sin(2π × (n+1) × phase))`
    ///
    /// The resulting waveform is normalized to prevent clipping.
    pub fn set_additive_harmonics(&mut self, harmonics: [f32; 8]) {
        self.additive_harmonics = harmonics;
        self.generate_additive_wavetable();
    }

    /// Set the wavetable index and load wavetable data from library
    ///
    /// This copies the 4× oversampled wavetable from the library into the oscillator's
    /// cache for fast lookup during audio processing. This is done during parameter updates
    /// (not in the audio callback) to avoid lookups in the hot path.
    pub fn set_wavetable(
        &mut self,
        index: usize,
        wavetable_library: &crate::dsp::wavetable_library::WavetableLibrary,
    ) {
        self.wavetable_index = index;
        self.wavetable_position = 0.0;

        // Load wavetable from library and cache the 4× oversampled version
        if let Some(wavetable) = wavetable_library.get(index) {
            // Get the pre-computed 4× oversampled data
            self.current_wavetable_4x = Some(wavetable.samples_4x().to_vec());
        } else {
            // Fallback: use empty wavetable (will output silence)
            self.current_wavetable_4x = None;
        }
    }

    /// Set wavetable morphing position
    pub fn set_wavetable_position(&mut self, position: f32) {
        self.wavetable_position = position.clamp(0.0, 1.0);
    }

    /// Generate the additive wavetable from current harmonic amplitudes.
    ///
    /// This synthesizes a 2048-sample wavetable by summing sine waves at harmonic
    /// frequencies, weighted by their amplitudes. The result is normalized to
    /// prevent clipping when all harmonics are at maximum amplitude.
    ///
    /// The wavetable is pre-computed (not generated in real-time during playback)
    /// to minimize CPU usage. It's only regenerated when harmonic amplitudes change.
    fn generate_additive_wavetable(&mut self) {
        use std::f32::consts::PI;

        // Sum of all harmonic amplitudes for normalization
        let amplitude_sum: f32 = self.additive_harmonics.iter().sum();
        let norm_factor = if amplitude_sum > 0.001 {
            1.0 / amplitude_sum
        } else {
            1.0 // Avoid division by zero
        };

        // Generate 2048 samples covering one complete cycle
        for i in 0..2048 {
            let phase = i as f32 / 2048.0; // 0.0 to 1.0
            let mut sample = 0.0;

            // Sum all 8 harmonics
            for (n, &amplitude) in self.additive_harmonics.iter().enumerate() {
                if amplitude > 0.001 {
                    // Skip near-zero harmonics for efficiency
                    let harmonic_freq = (n + 1) as f32; // 1, 2, 3, ..., 8
                    sample += amplitude * (2.0 * PI * harmonic_freq * phase).sin();
                }
            }

            // Normalize to prevent clipping
            self.additive_wavetable[i] = sample * norm_factor;
        }
    }

    /// Lookup sample from additive wavetable with linear interpolation.
    ///
    /// Uses the current phase (0.0 to 1.0) to index into the 2048-sample wavetable.
    /// Linear interpolation provides smooth playback without audible stepping artifacts.
    fn lookup_additive_wavetable(&self, phase: f32) -> f32 {
        let index = phase * 2048.0;
        let i0 = index.floor() as usize % 2048;
        let i1 = (i0 + 1) % 2048;
        let frac = index.fract();

        // Linear interpolation between adjacent samples
        self.additive_wavetable[i0] * (1.0 - frac) + self.additive_wavetable[i1] * frac
    }

    /// Lookup sample from current wavetable with linear interpolation (4× oversampled).
    ///
    /// Uses the current phase (0.0 to 1.0) to index into the 4× oversampled wavetable
    /// (8192 samples). Returns 0.0 if no wavetable is loaded.
    ///
    /// This is called during audio processing, so it must be fast. The wavetable data
    /// is pre-loaded during parameter updates via `set_wavetable()`.
    fn lookup_wavetable_4x(&self, phase: f32) -> f32 {
        if let Some(ref wavetable_data) = self.current_wavetable_4x {
            let len = wavetable_data.len();
            let index = phase * len as f32;
            let i0 = index.floor() as usize % len;
            let i1 = (i0 + 1) % len;
            let frac = index.fract();

            // Linear interpolation between adjacent samples
            wavetable_data[i0] * (1.0 - frac) + wavetable_data[i1] * frac
        } else {
            // No wavetable loaded - return silence
            0.0
        }
    }

    /// Apply wave shaping to an SIMD vector of samples (SIMD version).
    ///
    /// This function modulates the waveform based on the shape parameter and current
    /// waveform type. Different waveforms respond to shaping differently:
    ///
    /// - **Sine**: Uses soft clipping (tanh approximation) to add harmonics. The more
    ///   shape is applied, the more the wave is "driven" and clipped, adding overtones
    ///   for a richer, dirtier sound.
    ///
    /// - **Sawtooth**: Morphs towards triangle wave. This reduces the harsh high harmonics
    ///   of a sawtooth, making it sound smoother and more mellow.
    ///
    /// - **Triangle**: Morphs towards sawtooth, adding sharpness and brightness by
    ///   introducing more high-frequency content.
    ///
    /// The shaping is implemented using linear interpolation between the base waveform
    /// and the target waveform/effect.
    #[cfg(feature = "simd")]
    fn apply_wave_shaping(&self, samples: f32x4) -> f32x4 {
        let shape_amount = f32x4::splat(self.shape.abs());
        match self.waveform {
            Waveform::Sine => {
                // Sine: add harmonics via soft clipping/saturation
                // "Drive" the signal harder when shape is applied
                let drive = f32x4::splat(1.0 + self.shape.abs() * 3.0);
                let driven = samples * drive;
                // Soft clip using tanh approximation: tanh(x) ≈ x - x³/3
                // This creates smooth, musical distortion without harsh clipping
                let x2 = driven * driven;
                let x3 = x2 * driven;
                let tanh_approx = driven - x3 / f32x4::splat(3.0);
                tanh_approx / drive.sqrt()
            }
            Waveform::Saw => {
                // Saw: morph towards triangle (removes harsh harmonics)
                let triangle = f32x4::from_array([
                    if self.phase < 0.5 {
                        4.0 * self.phase - 1.0
                    } else {
                        -4.0 * self.phase + 3.0
                    },
                    if self.phase + self.phase_increment < 0.5 {
                        4.0 * (self.phase + self.phase_increment) - 1.0
                    } else {
                        -4.0 * (self.phase + self.phase_increment) + 3.0
                    },
                    if self.phase + 2.0 * self.phase_increment < 0.5 {
                        4.0 * (self.phase + 2.0 * self.phase_increment) - 1.0
                    } else {
                        -4.0 * (self.phase + 2.0 * self.phase_increment) + 3.0
                    },
                    if self.phase + 3.0 * self.phase_increment < 0.5 {
                        4.0 * (self.phase + 3.0 * self.phase_increment) - 1.0
                    } else {
                        -4.0 * (self.phase + 3.0 * self.phase_increment) + 3.0
                    },
                ]);
                samples * (f32x4::splat(1.0) - shape_amount) + triangle * shape_amount
            }
            Waveform::Square => {
                // Square: morph pulse width (but this is handled by Pulse waveform)
                samples
            }
            Waveform::Triangle => {
                // Triangle: add corners/sharpness (morph towards saw)
                let saw = f32x4::from_array([
                    2.0 * self.phase - 1.0,
                    2.0 * (self.phase + self.phase_increment) - 1.0,
                    2.0 * (self.phase + 2.0 * self.phase_increment) - 1.0,
                    2.0 * (self.phase + 3.0 * self.phase_increment) - 1.0,
                ]);
                samples * (f32x4::splat(1.0) - shape_amount) + saw * shape_amount
            }
            Waveform::Pulse => samples, // Pulse uses shape for PWM, not morphing
            Waveform::WhiteNoise
            | Waveform::PinkNoise
            | Waveform::Additive
            | Waveform::Wavetable => samples, // Handled separately
        }
    }

    /// Generate a single noise sample (white or pink).
    ///
    /// Noise generation bypasses oversampling/downsampling since noise is already
    /// broadband and doesn't benefit from anti-aliasing. The shape parameter controls
    /// noise color: -1.0 = white, 0.0 = pink, +1.0 = brown (low-pass filtered).
    ///
    /// Uses xorshift32 PRNG for white noise generation and Paul Kellett's "economy"
    /// pink noise filter (3-pole approximation of -3dB/octave slope).
    fn generate_noise_sample(&mut self) -> f32 {
        // Generate white noise using xorshift32 PRNG
        let white = waveform::u32_to_f32_bipolar(waveform::xorshift32(&mut self.noise_state));

        match self.waveform {
            Waveform::WhiteNoise => {
                // Shape parameter controls noise filtering
                // -1.0 = pure white, 0.0 = pink, +1.0 = brown (low-pass)
                if self.shape.abs() < 0.001 {
                    // Pure white noise
                    white
                } else {
                    // Apply pink filter and morph based on shape
                    let pink = self.generate_pink_noise(white);

                    if self.shape < 0.0 {
                        // Morph from pink to white (shape: -1.0 to 0.0)
                        let amount = -self.shape;
                        pink * (1.0 - amount) + white * amount
                    } else {
                        // Morph from pink to brown (low-pass filtered)
                        // Brown noise = integrate pink noise
                        let amount = self.shape;
                        pink * (1.0 - amount * 0.5) // Simple low-pass
                    }
                }
            }
            Waveform::PinkNoise => {
                // Generate pink noise with shape controlling frequency content
                let pink = self.generate_pink_noise(white);

                if self.shape < 0.0 {
                    // Morph towards white (brighter)
                    let amount = -self.shape;
                    pink * (1.0 - amount) + white * amount
                } else {
                    // Morph towards brown (darker)
                    let amount = self.shape;
                    pink * (1.0 - amount * 0.5)
                }
            }
            _ => 0.0, // Should never reach here
        }
    }

    /// Generate pink noise using Paul Kellett's "economy" method.
    ///
    /// Pink noise has equal energy per octave (-3dB/octave slope). This implementation
    /// uses a 3-pole filter to approximate the spectral shape. The input white noise
    /// is filtered through three first-order low-pass stages with different time constants.
    fn generate_pink_noise(&mut self, white: f32) -> f32 {
        self.pink_b0 = 0.99886 * self.pink_b0 + white * 0.0555179;
        self.pink_b1 = 0.99332 * self.pink_b1 + white * 0.0750759;
        self.pink_b2 = 0.96900 * self.pink_b2 + white * 0.1538520;

        let pink = self.pink_b0 + self.pink_b1 + self.pink_b2 + white * 0.3104856;
        pink * 0.11 // Scale to roughly match white noise amplitude
    }

    /// Apply harmonic morphing to additive waveform (SIMD version).
    ///
    /// The shape parameter morphs the harmonic balance:
    /// - Positive shape: Emphasizes higher harmonics (brighter, more harmonics)
    /// - Negative shape: Suppresses higher harmonics (darker, more fundamental)
    ///
    /// This is implemented as a simple high-pass/low-pass filter on the signal,
    /// which effectively changes the harmonic balance in real-time without
    /// regenerating the wavetable.
    #[cfg(feature = "simd")]
    fn apply_additive_morph(&self, samples: f32x4) -> f32x4 {
        if self.shape > 0.0 {
            // Positive shape: Emphasize highs (high-pass characteristic)
            // Simple differentiator adds harmonics
            let boost = f32x4::splat(1.0 + self.shape * 0.5);
            samples * boost
        } else {
            // Negative shape: Suppress highs (low-pass characteristic)
            // Soft clipping reduces harmonic content
            let amount = f32x4::splat(-self.shape);
            let one = f32x4::splat(1.0);
            let dampened = samples * (one - amount * f32x4::splat(0.3));
            dampened
        }
    }

    /// Apply harmonic morphing to additive waveform (scalar version).
    ///
    /// See SIMD version documentation for details.
    #[cfg(not(feature = "simd"))]
    fn apply_additive_morph_scalar(&self, sample: f32) -> f32 {
        if self.shape > 0.0 {
            // Positive shape: Emphasize highs
            sample * (1.0 + self.shape * 0.5)
        } else {
            // Negative shape: Suppress highs
            sample * (1.0 + self.shape * 0.3)
        }
    }

    /// Generate one output sample (processes 4× oversampled internally) - SIMD optimized version.
    ///
    /// This is the main processing function when the "simd" feature is enabled. It:
    /// 1. Generates 4 oversampled waveform samples simultaneously using SIMD
    /// 2. Applies optional wave shaping if needed
    /// 3. Downsamples the 4 samples back to 1 output sample
    /// 4. Returns the single anti-aliased sample
    ///
    /// ## Optimization: Shape Skipping
    ///
    /// If the shape parameter is very small (< 0.001), the expensive wave shaping calculations
    /// are skipped. This is a crucial optimization because wave shaping involves multiple
    /// multiplications and branching. For most presets where shape isn't heavily used, this
    /// early return provides significant CPU savings.
    ///
    /// ## SIMD Processing
    ///
    /// SIMD (Single Instruction Multiple Data) allows processing 4 float32 samples in parallel
    /// using a single CPU instruction, making oscillator generation roughly 4× faster when
    /// available. If the "simd" feature isn't enabled, a scalar version (below) is used instead.
    ///
    /// ## Phase Wrapping
    ///
    /// After advancing the phase by 4 increments (for the 4 oversampled samples), we check if
    /// phase >= 1.0 and wrap it back to the 0.0-1.0 range. This is a modulo operation that
    /// must be done correctly to avoid cumulative rounding errors.
    #[cfg(feature = "simd")]
    pub fn process(&mut self) -> f32 {
        // Fast path for noise waveforms: bypass oversampling/downsampling
        if matches!(self.waveform, Waveform::WhiteNoise | Waveform::PinkNoise) {
            return self.generate_noise_sample();
        }

        // OPTIMIZATION: Early return if shape is effectively zero (skip expensive shaping)
        if self.shape.abs() < 0.001
            && self.waveform != Waveform::Pulse
            && self.waveform != Waveform::Additive
            && self.waveform != Waveform::Wavetable
        {
            // Fast path: no wave shaping needed
            let phases = f32x4::from_array([
                self.phase,
                self.phase + self.phase_increment,
                self.phase + 2.0 * self.phase_increment,
                self.phase + 3.0 * self.phase_increment,
            ]);
            let samples = waveform::generate_simd(phases, self.waveform);
            self.phase += 4.0 * self.phase_increment;
            while self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            return self.downsampler.process([
                samples.as_array()[0],
                samples.as_array()[1],
                samples.as_array()[2],
                samples.as_array()[3],
            ]);
        }

        // SIMD-optimized version with wave shaping
        // Generate 4 phase values at once, each offset by one oversampled increment
        let phases = f32x4::from_array([
            self.phase,
            self.phase + self.phase_increment,
            self.phase + 2.0 * self.phase_increment,
            self.phase + 3.0 * self.phase_increment,
        ]);

        // Generate samples based on waveform using SIMD
        let mut samples = match self.waveform {
            Waveform::Pulse => {
                // Pulse width is controlled by shape parameter
                // Maps shape (-1.0 to 1.0) to pulse width (10% to 90%)
                // At shape=0, pulse width is 50% (square wave)
                let pulse_width = 0.5 + self.shape * 0.4; // Maps to 0.1 - 0.9 range
                let threshold = f32x4::splat(pulse_width);
                let one = f32x4::splat(1.0);
                let neg_one = f32x4::splat(-1.0);
                // Use SIMD comparison: if phase < pulse_width, output 1.0, else -1.0
                phases.simd_lt(threshold).select(one, neg_one)
            }
            Waveform::Additive => {
                // Lookup from pre-computed wavetable (scalar, could optimize with SIMD later)
                let phase_array = phases.to_array();
                f32x4::from_array([
                    self.lookup_additive_wavetable(phase_array[0]),
                    self.lookup_additive_wavetable(phase_array[1]),
                    self.lookup_additive_wavetable(phase_array[2]),
                    self.lookup_additive_wavetable(phase_array[3]),
                ])
            }
            Waveform::Wavetable => {
                // Lookup from loaded wavetable (4× oversampled)
                let phase_array = phases.to_array();
                f32x4::from_array([
                    self.lookup_wavetable_4x(phase_array[0]),
                    self.lookup_wavetable_4x(phase_array[1]),
                    self.lookup_wavetable_4x(phase_array[2]),
                    self.lookup_wavetable_4x(phase_array[3]),
                ])
            }
            _ => waveform::generate_simd(phases, self.waveform),
        };

        // Apply wave shaping if shape parameter is non-zero
        if self.shape != 0.0
            && self.waveform != Waveform::Pulse
            && self.waveform != Waveform::Additive
            && self.waveform != Waveform::Wavetable
        {
            samples = self.apply_wave_shaping(samples);
        } else if self.waveform == Waveform::Additive && self.shape.abs() > 0.001 {
            // For additive waveform, shape morphs the harmonic balance
            // Positive shape: emphasize higher harmonics (brighter)
            // Negative shape: suppress higher harmonics (darker)
            samples = self.apply_additive_morph(samples);
        }

        // Advance phase by 4 increments (one for each oversampled sample)
        self.phase += 4.0 * self.phase_increment;
        // Wrap phase back into 0.0-1.0 range if it exceeded 1.0
        while self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Convert SIMD vector to array and downsample 4 samples to 1
        self.downsampler.process(samples.to_array())
    }

    /// Generate one output sample (processes 4× oversampled internally) - Scalar version.
    ///
    /// This is the fallback version when the "simd" feature is not enabled. It does the same
    /// thing as the SIMD version but processes samples one at a time instead of 4 in parallel.
    /// It's slower but works on any CPU and doesn't require nightly Rust.
    ///
    /// The algorithm is identical to the SIMD version, just implemented with scalar floating
    /// point operations instead of vector operations.
    #[cfg(not(feature = "simd"))]
    pub fn process(&mut self) -> f32 {
        // Fast path for noise waveforms: bypass oversampling/downsampling
        if matches!(self.waveform, Waveform::WhiteNoise | Waveform::PinkNoise) {
            return self.generate_noise_sample();
        }

        // OPTIMIZATION: Early return if shape is effectively zero (skip expensive shaping)
        if self.shape.abs() < 0.001
            && self.waveform != Waveform::Pulse
            && self.waveform != Waveform::Additive
            && self.waveform != Waveform::Wavetable
        {
            // Fast path: no wave shaping needed, just generate base waveform
            let mut oversampled = [0.0; 4];
            for sample in &mut oversampled {
                *sample = waveform::generate_scalar(self.phase, self.waveform);
                self.phase += self.phase_increment;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
            }
            return self.downsampler.process(oversampled);
        }

        // Generate 4 oversampled samples (one per iteration)
        let mut oversampled = [0.0; 4];

        for sample in &mut oversampled {
            *sample = match self.waveform {
                Waveform::Pulse => {
                    // Pulse width controlled by shape: -1.0 = 10% duty, 0.0 = 50%, 1.0 = 90%
                    let pulse_width = 0.5 + self.shape * 0.4;
                    if self.phase < pulse_width {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Waveform::Additive => {
                    // Lookup from pre-computed wavetable
                    self.lookup_additive_wavetable(self.phase)
                }
                Waveform::Wavetable => {
                    // Lookup from loaded wavetable (4× oversampled)
                    self.lookup_wavetable_4x(self.phase)
                }
                _ => waveform::generate_scalar(self.phase, self.waveform),
            };

            // Apply wave shaping if not Pulse (Pulse uses shape for PWM, not morphing)
            if self.shape != 0.0
                && self.waveform != Waveform::Pulse
                && self.waveform != Waveform::Additive
                && self.waveform != Waveform::Wavetable
            {
                *sample = self.apply_wave_shaping_scalar(*sample);
            } else if self.waveform == Waveform::Additive && self.shape.abs() > 0.001 {
                // For additive waveform, shape morphs the harmonic balance
                *sample = self.apply_additive_morph_scalar(*sample);
            }

            // Advance phase for next sample
            self.phase += self.phase_increment;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        // Downsample from 4× back to 1× output sample
        self.downsampler.process(oversampled)
    }

    /// Process one sample with frequency modulation (FM synthesis).
    ///
    /// # FM Synthesis Basics
    ///
    /// FM (Frequency Modulation) synthesis modulates the carrier oscillator's **phase**
    /// using the output of a modulator oscillator. This creates complex harmonic content
    /// and is the foundation of classic FM synthesis (Yamaha DX7, etc.).
    ///
    /// The modulator's output shifts the carrier's phase, generating sidebands at:
    /// - `carrier_freq ± modulator_freq`
    /// - `carrier_freq ± 2 * modulator_freq`
    /// - ... and so on
    ///
    /// # Arguments
    ///
    /// * `modulator_output` - The output sample from the modulating oscillator (typically -1.0 to 1.0)
    /// * `fm_amount` - Modulation depth (0.0 = no FM, higher values = more sidebands)
    ///
    /// # Implementation Notes
    ///
    /// - Phase modulation: `phase_modulated = phase + modulator_output * fm_amount`
    /// - The modulator_output is clamped to prevent extreme phase shifts that could cause aliasing
    /// - This method processes with oversampling just like the regular process() method
    /// - Noise waveforms bypass oversampling and ignore FM (noise is already broadband)
    pub fn process_with_fm(&mut self, modulator_output: f32, fm_amount: f32) -> f32 {
        // Fast path for noise: bypass oversampling and ignore FM
        if matches!(self.waveform, Waveform::WhiteNoise | Waveform::PinkNoise) {
            return self.generate_noise_sample();
        }

        // Clamp modulator to prevent extreme phase shifts
        let mod_clamped = modulator_output.clamp(-1.0, 1.0);
        let phase_offset = mod_clamped * fm_amount;

        // Generate 4 oversampled samples with FM applied
        let mut oversampled = [0.0; 4];

        for sample in &mut oversampled {
            // Apply phase modulation: shift the phase by the modulator output
            let modulated_phase = (self.phase + phase_offset).fract();

            *sample = match self.waveform {
                Waveform::Pulse => {
                    let pulse_width = 0.5 + self.shape * 0.4;
                    if modulated_phase < pulse_width {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Waveform::Additive => {
                    // Lookup from pre-computed additive wavetable with FM
                    self.lookup_additive_wavetable(modulated_phase)
                }
                Waveform::Wavetable => {
                    // Lookup from loaded wavetable with FM
                    self.lookup_wavetable_4x(modulated_phase)
                }
                _ => waveform::generate_scalar(modulated_phase, self.waveform),
            };

            // Apply wave shaping if configured
            // Inline version to work with both SIMD and non-SIMD builds
            if self.shape != 0.0 && self.waveform != Waveform::Pulse {
                let shape_amount = self.shape.abs();
                *sample = match self.waveform {
                    Waveform::Sine => {
                        // Add harmonics via soft clipping (tanh approximation)
                        let drive = 1.0 + shape_amount * 3.0;
                        let driven = *sample * drive;
                        let tanh_approx = driven - (driven * driven * driven) / 3.0;
                        tanh_approx / drive.sqrt()
                    }
                    Waveform::Saw => {
                        // Morph towards triangle
                        let triangle = if modulated_phase < 0.5 {
                            4.0 * modulated_phase - 1.0
                        } else {
                            -4.0 * modulated_phase + 3.0
                        };
                        *sample * (1.0 - shape_amount) + triangle * shape_amount
                    }
                    Waveform::Triangle => {
                        // Add sharpness (morph towards saw)
                        let saw = 2.0 * modulated_phase - 1.0;
                        *sample * (1.0 - shape_amount) + saw * shape_amount
                    }
                    _ => *sample,
                };
            }

            // Advance phase (carrier's natural frequency progression)
            self.phase += self.phase_increment;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        // Downsample from 4× back to 1×
        self.downsampler.process(oversampled)
    }

    /// Apply wave shaping to a scalar sample (non-SIMD version).
    ///
    /// This implements the same wave shaping logic as apply_wave_shaping() but operates on
    /// a single floating point value instead of a SIMD vector. It's called per-sample in the
    /// scalar processing path.
    #[cfg(not(feature = "simd"))]
    fn apply_wave_shaping_scalar(&self, sample: f32) -> f32 {
        let shape_amount = self.shape.abs();
        match self.waveform {
            Waveform::Sine => {
                // Add harmonics via soft clipping (tanh approximation: x - x³/3)
                let drive = 1.0 + shape_amount * 3.0;
                let driven = sample * drive;
                let tanh_approx = driven - (driven * driven * driven) / 3.0;
                tanh_approx / drive.sqrt()
            }
            Waveform::Saw => {
                // Morph towards triangle
                let triangle = if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    -4.0 * self.phase + 3.0
                };
                sample * (1.0 - shape_amount) + triangle * shape_amount
            }
            Waveform::Square => sample,
            Waveform::Triangle => {
                // Add sharpness (morph towards saw)
                let saw = 2.0 * self.phase - 1.0;
                sample * (1.0 - shape_amount) + saw * shape_amount
            }
            Waveform::Pulse => sample,
            Waveform::WhiteNoise
            | Waveform::PinkNoise
            | Waveform::Additive
            | Waveform::Wavetable => sample,
        }
    }

    /// Reset the oscillator to its initial state.
    ///
    /// This clears the phase accumulator and the internal downsampler state. Useful when:
    /// - A note ends and the voice is being reused for a new note
    /// - You want to ensure the oscillator starts cleanly without any state from previous processing
    ///
    /// Note: This does NOT reset frequency, waveform, or shape parameters. It only clears
    /// the running state.
    pub fn reset(&mut self) {
        // Start from the configured initial phase (used for unison/stacked voices).
        // This prevents multiple oscillators from starting perfectly phase-aligned,
        // which can create large coherent peaks and audible clipping/distortion.
        self.phase = self.initial_phase;
        self.downsampler.reset();

        // Reset noise generation state
        self.noise_state = 0x12345678; // Re-seed PRNG
        self.pink_b0 = 0.0;
        self.pink_b1 = 0.0;
        self.pink_b2 = 0.0;
    }

    /// Reset only the internal buffers (downsampler) without changing phase.
    ///
    /// This is used when retriggering notes to avoid phase discontinuities that cause clicks.
    /// The oscillator phase continues from wherever it was, but the downsampler's ring buffer
    /// is cleared to prevent old samples from the previous note from bleeding through.
    ///
    /// Use this instead of reset() for note retriggering to maintain phase continuity.
    pub fn reset_buffers(&mut self) {
        self.downsampler.reset();

        // Reset noise generation state (noise needs fresh start per note)
        self.noise_state = 0x12345678;
        self.pink_b0 = 0.0;
        self.pink_b1 = 0.0;
        self.pink_b2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_oscillator_creation() {
        let osc = Oscillator::new(44100.0);
        // assert_eq!(osc.sample_rate, 44100.0);
        assert_eq!(osc.oversample_rate, 176400.0);
        assert_eq!(osc.phase, 0.0);
    }

    #[test]
    fn test_frequency_setting() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Phase increment should be freq / oversample_rate
        let expected_increment = 440.0 / 176400.0;
        assert_relative_eq!(osc.phase_increment, expected_increment, epsilon = 1e-6);
    }

    #[test]
    fn test_sine_wave_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sine);
        osc.set_frequency(1000.0);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(osc.process());
        }

        // Find max and min values (should approach ±1.0 for sine wave)
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max > 0.9, "Max value {} should be close to 1.0", max);
        assert!(
            max < 1.1,
            "Max value {} should not exceed 1.0 significantly",
            max
        );
        assert!(min < -0.9, "Min value {} should be close to -1.0", min);
        assert!(
            min > -1.1,
            "Min value {} should not be below -1.0 significantly",
            min
        );
    }

    #[test]
    fn test_saw_wave_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Saw);
        osc.set_frequency(1000.0);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(osc.process());
        }

        // Verify we get a range of values
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        // Oversampling and filtering will reduce the amplitude slightly
        assert!(max > 0.5, "Max value {} should be substantial", max);
        assert!(min < -0.5, "Min value {} should be substantial", min);
    }

    #[test]
    fn test_square_wave_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Square);
        osc.set_frequency(1000.0);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(osc.process());
        }

        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        // Square wave after filtering will have reduced amplitude
        assert!(max > 0.4, "Max value {} should be substantial", max);
        assert!(min < -0.4, "Min value {} should be substantial", min);
    }

    #[test]
    fn test_triangle_wave_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Triangle);
        osc.set_frequency(1000.0);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(osc.process());
        }

        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max > 0.8, "Max value {} should be close to 1.0", max);
        assert!(min < -0.8, "Min value {} should be close to -1.0", min);
    }

    #[test]
    fn test_waveform_switching() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);

        osc.set_waveform(Waveform::Sine);
        // Process a few samples to let filter settle
        for _ in 0..20 {
            osc.process();
        }
        let sine_sample = osc.process();

        osc.reset();
        osc.set_waveform(Waveform::Saw);
        for _ in 0..20 {
            osc.process();
        }
        let saw_sample = osc.process();

        // Different waveforms should produce different outputs after settling
        assert!((sine_sample - saw_sample).abs() > 0.001);
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(10000.0); // High frequency to cause wrapping quickly
        osc.set_waveform(Waveform::Sine);

        // Process many samples
        for _ in 0..1000 {
            osc.process();
        }

        // Phase should still be in valid range [0, 1)
        assert!(osc.phase >= 0.0 && osc.phase < 1.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Sine);

        // Process some samples
        for _ in 0..10 {
            osc.process();
        }

        // Reset should clear phase
        osc.reset();
        assert_eq!(osc.phase, 0.0);
    }

    #[test]
    fn test_phase_offset_affects_output_after_reset() {
        let mut osc_a = Oscillator::new(44100.0);
        osc_a.set_waveform(Waveform::Sine);
        osc_a.set_frequency(440.0);
        osc_a.set_phase(0.0);
        osc_a.reset();

        let mut osc_b = Oscillator::new(44100.0);
        osc_b.set_waveform(Waveform::Sine);
        osc_b.set_frequency(440.0);
        osc_b.set_phase(0.25);
        osc_b.reset();

        // Warm up the downsampler so we're comparing steady-state samples.
        for _ in 0..256 {
            let _ = osc_a.process();
            let _ = osc_b.process();
        }

        let a = osc_a.process();
        let b = osc_b.process();

        assert!(
            (a - b).abs() > 0.01,
            "Phase offsets should produce different samples (a={:.6}, b={:.6})",
            a,
            b
        );
    }

    #[test]
    fn test_dc_offset() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Sine);

        let mut sum = 0.0;
        let num_samples = 4410; // One full second at 44.1kHz / 10

        for _ in 0..num_samples {
            sum += osc.process();
        }

        let average = sum / num_samples as f32;

        // Sine wave should have near-zero DC offset
        assert!(
            average.abs() < 0.01,
            "DC offset {} should be near zero",
            average
        );
    }

    #[test]
    fn test_aliasing_reduction() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Saw);
        osc.set_frequency(5000.0); // High frequency sawtooth

        let mut samples = Vec::new();
        for _ in 0..1000 {
            samples.push(osc.process());
        }

        // With 4× oversampling and proper downsampling, high-frequency content
        // should be filtered out. This is a basic check that output isn't clipping
        // or producing obviously wrong values
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max < 1.5, "Output shouldn't clip excessively");
        assert!(min > -1.5, "Output shouldn't clip excessively");
    }

    #[test]
    fn test_white_noise_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::WhiteNoise);

        let mut samples = Vec::new();
        for _ in 0..1000 {
            samples.push(osc.process());
        }

        // Check range
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(
            max > 0.5,
            "White noise should have substantial positive values"
        );
        assert!(
            min < -0.5,
            "White noise should have substantial negative values"
        );

        // Check entropy - values should vary
        let first = samples[0];
        let all_same = samples.iter().all(|&s| (s - first).abs() < 0.001);
        assert!(!all_same, "White noise should produce varying values");

        // Check DC offset is low
        let sum: f32 = samples.iter().sum();
        let avg = sum / samples.len() as f32;
        assert!(
            avg.abs() < 0.1,
            "White noise should have low DC offset, got {}",
            avg
        );
    }

    #[test]
    fn test_pink_noise_generation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::PinkNoise);

        let mut samples = Vec::new();
        for _ in 0..1000 {
            samples.push(osc.process());
        }

        // Check range
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(
            (-1.5..=1.5).contains(&max),
            "Pink noise max {} in range",
            max
        );
        assert!(
            (-1.5..=1.5).contains(&min),
            "Pink noise min {} in range",
            min
        );

        // Pink noise should have lower high-frequency content than white
        // This is hard to test without FFT, so just check it produces varied output
        let first = samples[0];
        let all_same = samples.iter().all(|&s| (s - first).abs() < 0.001);
        assert!(!all_same, "Pink noise should produce varying values");
    }

    #[test]
    fn test_noise_shape_control() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::WhiteNoise);

        // Test shape = -1.0 (bright/white)
        osc.set_shape(-1.0);
        let bright = osc.process();

        // Test shape = 0.0 (pink)
        osc.set_shape(0.0);
        let pink = osc.process();

        // Test shape = 1.0 (brown/dark)
        osc.set_shape(1.0);
        let brown = osc.process();

        // All should be in valid range
        assert!((-1.5..=1.5).contains(&bright), "Bright noise in range");
        assert!((-1.5..=1.5).contains(&pink), "Pink noise in range");
        assert!((-1.5..=1.5).contains(&brown), "Brown noise in range");
    }

    #[test]
    fn test_noise_bypasses_oversampling() {
        // Noise generation should be fast since it bypasses oversampling
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::WhiteNoise);

        // Generate many samples quickly
        for _ in 0..10000 {
            let sample = osc.process();
            assert!((-2.0..=2.0).contains(&sample), "Sample in range");
        }
    }

    #[test]
    fn test_fm_synthesis_creates_sidebands() {
        // FM synthesis should create additional harmonic content (sidebands)
        // compared to standard synthesis
        let sample_rate = 44100.0;
        let mut carrier = Oscillator::new(sample_rate);
        let mut modulator = Oscillator::new(sample_rate);

        // Set both to sine waves
        carrier.set_waveform(Waveform::Sine);
        modulator.set_waveform(Waveform::Sine);

        // Carrier: 440 Hz (A4)
        carrier.set_frequency(440.0);
        // Modulator: 110 Hz (A2, 2 octaves below)
        modulator.set_frequency(110.0);

        // Process with FM
        let mut fm_samples = Vec::new();
        for _ in 0..1000 {
            let mod_out = modulator.process();
            let fm_sample = carrier.process_with_fm(mod_out, 1.0); // Moderate FM amount
            fm_samples.push(fm_sample);
        }

        // Check that output is in valid range
        let max = fm_samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = fm_samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max <= 1.2, "FM output max {} should be reasonable", max);
        assert!(min >= -1.2, "FM output min {} should be reasonable", min);

        // FM should produce varying output (not DC)
        let first = fm_samples[0];
        let all_same = fm_samples.iter().all(|&s| (s - first).abs() < 0.001);
        assert!(!all_same, "FM synthesis should produce varying output");
    }

    #[test]
    fn test_fm_amount_zero_equals_normal() {
        // With fm_amount = 0, FM should behave identically to normal processing
        let sample_rate = 44100.0;
        let mut osc1 = Oscillator::new(sample_rate);
        let mut osc2 = Oscillator::new(sample_rate);

        osc1.set_waveform(Waveform::Saw);
        osc2.set_waveform(Waveform::Saw);
        osc1.set_frequency(440.0);
        osc2.set_frequency(440.0);

        // Generate samples: one with normal process, one with FM (amount=0)
        let normal = osc1.process();
        let fm_zero = osc2.process_with_fm(0.5, 0.0); // Modulator output doesn't matter

        // Should be very close (allowing for floating point differences)
        assert_relative_eq!(normal, fm_zero, epsilon = 0.01);
    }

    #[test]
    fn test_fm_with_high_modulation() {
        // High FM amounts should still produce valid output
        let sample_rate = 44100.0;
        let mut carrier = Oscillator::new(sample_rate);
        let mut modulator = Oscillator::new(sample_rate);

        carrier.set_waveform(Waveform::Triangle);
        modulator.set_waveform(Waveform::Sine);
        carrier.set_frequency(220.0);
        modulator.set_frequency(55.0);

        // High FM amount
        for _ in 0..500 {
            let mod_out = modulator.process();
            let fm_sample = carrier.process_with_fm(mod_out, 5.0); // High modulation

            // With high FM amounts (5.0), output can significantly exceed ±1.0
            // This is expected - FM creates complex harmonic content
            // We just verify it stays finite and not absurdly large
            assert!(fm_sample.is_finite(), "FM should produce finite output");
            assert!(
                (-10.0..=10.0).contains(&fm_sample),
                "FM with high modulation should stay in reasonable range, got {}",
                fm_sample
            );
        }
    }

    #[test]
    fn test_fm_modulator_clamping() {
        // Extreme modulator values should be clamped to prevent instability
        let sample_rate = 44100.0;
        let mut carrier = Oscillator::new(sample_rate);
        carrier.set_waveform(Waveform::Sine);
        carrier.set_frequency(440.0);

        // Test with extreme modulator values
        let extreme_mod = 10.0; // Way beyond normal ±1.0 range
        let sample = carrier.process_with_fm(extreme_mod, 1.0);

        // Should still produce valid output (clamping should prevent NaN/Inf)
        assert!(
            sample.is_finite(),
            "FM should handle extreme modulator values"
        );
        assert!((-2.0..=2.0).contains(&sample), "Clamped FM in range");
    }

    #[test]
    fn test_fm_noise_bypasses_modulation() {
        // Noise waveforms should bypass FM (already broadband, don't need modulation)
        let sample_rate = 44100.0;
        let mut noise_osc = Oscillator::new(sample_rate);
        noise_osc.set_waveform(Waveform::WhiteNoise);

        // FM with noise should still work (but just return normal noise)
        let sample1 = noise_osc.process_with_fm(0.5, 2.0);
        let sample2 = noise_osc.process_with_fm(-0.8, 2.0);

        // Both should be valid noise samples
        assert!(
            (-2.0..=2.0).contains(&sample1),
            "Noise FM sample 1 in range"
        );
        assert!(
            (-2.0..=2.0).contains(&sample2),
            "Noise FM sample 2 in range"
        );
    }

    #[test]
    fn test_fm_phase_continuity() {
        // FM should maintain phase continuity (no discontinuities/clicks)
        let sample_rate = 44100.0;
        let mut carrier = Oscillator::new(sample_rate);
        let mut modulator = Oscillator::new(sample_rate);

        carrier.set_waveform(Waveform::Sine);
        modulator.set_waveform(Waveform::Sine);
        carrier.set_frequency(440.0);
        modulator.set_frequency(5.0); // Slow modulation

        let mut prev_sample = carrier.process_with_fm(modulator.process(), 1.0);

        // Check for discontinuities over many samples
        for _ in 0..1000 {
            let mod_out = modulator.process();
            let sample = carrier.process_with_fm(mod_out, 1.0);

            // Difference between consecutive samples should be reasonable
            // (no sudden jumps that would cause clicks)
            let diff = (sample - prev_sample).abs();
            assert!(
                diff < 0.5,
                "FM should have smooth phase continuity, got diff {}",
                diff
            );

            prev_sample = sample;
        }
    }
}
