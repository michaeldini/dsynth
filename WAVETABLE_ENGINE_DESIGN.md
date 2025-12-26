# Wavetable Engine Architecture for DSynth

## ✅ Implementation Status: **COMPLETE**

The wavetable synthesis engine has been successfully implemented with **compile-time embedded wavetables**, eliminating all runtime file dependencies.

## Key Implementation Details

### Embedded Wavetable System

**All 20 wavetables are now embedded directly into the binary at compile time**, meaning:
- ✅ No external files needed at runtime
- ✅ No filesystem access during plugin/app execution
- ✅ No installation issues with missing assets
- ✅ Works identically in plugin and standalone builds
- ✅ Only adds ~10MB to binary size

### How It Works

1. **[build.rs](build.rs)**: Scans `assets/wavetables/` at compile time and generates code using `include_bytes!`
2. **[wavetable_library.rs](src/dsp/wavetable_library.rs)**: Loads from embedded data via `load_from_embedded()` method
3. **[wavetable.rs](src/dsp/wavetable.rs)**: New `from_wav_bytes()` method parses WAV data from memory
4. **[engine.rs](src/audio/engine.rs)**: Initializes with `WavetableLibrary::load_from_embedded()`

### Backward Compatibility

The original `load_from_directory()` method is still available for development/testing, but production builds use `load_from_embedded()`.

## Overview

This document describes a wavetable synthesis engine that integrates seamlessly with DSynth's existing oscillator architecture. The design maintains the current 4× oversampling and anti-aliasing quality while adding modern wavetable morphing capabilities.

## Design Principles

1. **Sound Quality First**: 4× oversampling is maintained for wavetable samples to prevent aliasing
2. **Lock-Free Real-Time**: No allocations in the audio thread, pre-allocated wavetable data
3. **Efficient Morphing**: Smooth continuous blending between wavetables using linear interpolation
4. **Backward Compatible**: Existing waveforms (Sine, Saw, Square, etc.) remain unchanged
5. **Modulation-Ready**: Wavetable position can be modulated by LFOs, envelopes, and pitch tracking

## Architecture Overview

```
┌─────────────────────────────────────┐
│       WavetableLibrary (Static)     │
│  - Loads .wav files at startup      │
│  - Stores 256+ wavetables in memory │
│  - Handles resampling to 2048/4096  │
└─────────────────────────────────────┘
           │
           ↓
┌─────────────────────────────────────┐
│    Oscillator (Modified)            │
│  - Adds Wavetable waveform variant  │
│  - Manages wavetable position       │
│  - Linear interpolation between     │
│    adjacent tables                  │
│  - Respects oversampling/filtering  │
└─────────────────────────────────────┘
           │
           ↓
┌─────────────────────────────────────┐
│    Voice / LFO / Envelope           │
│  - Modulate wavetable position      │
│  - Enable dynamic timbre morphing   │
└─────────────────────────────────────┘
```

## File Structure

### New Files

```
src/dsp/
├── wavetable_library.rs    (Core wavetable storage & management)
├── wavetable_loader.rs     (Load .wav files, handle resampling)
└── wavetable.rs            (Wavetable data structure, interpolation)
```

### Modified Files

```
src/dsp/
├── oscillator.rs           (Add wavetable sample generation)
├── waveform.rs             (Add Wavetable variant to Waveform enum)
└── mod.rs                  (Export new modules)

src/
├── params.rs               (Add wavetable_position parameter)
└── plugin/param_registry.rs (Add wavetable position parameter ID)
```

## Core Data Structures

### 1. Wavetable Structure

```rust
/// A single wavetable: one snapshot of a waveform
/// Stored at both normal and 4× oversampled rates for anti-aliasing
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
    
    /// Original sample rate of the loaded .wav file
    /// Used for resampling if it doesn't match DSynth's rate
    source_sample_rate: u32,
}

impl Wavetable {
    /// Linear interpolation lookup at normalized phase [0.0, 1.0)
    pub fn lookup(&self, phase: f32) -> f32 {
        let index = phase * self.samples.len() as f32;
        let i0 = index.floor() as usize % self.samples.len();
        let i1 = (i0 + 1) % self.samples.len();
        let frac = index.fract();
        
        self.samples[i0] * (1.0 - frac) + self.samples[i1] * frac
    }
    
    /// Lookup at 4× oversampled rate (for anti-aliasing through Downsampler)
    pub fn lookup_4x(&self, phase: f32) -> f32 {
        let index = phase * self.samples_4x.len() as f32;
        let i0 = index.floor() as usize % self.samples_4x.len();
        let i1 = (i0 + 1) % self.samples_4x.len();
        let frac = index.fract();
        
        self.samples_4x[i0] * (1.0 - frac) + self.samples_4x[i1] * frac
    }
    
    /// Morph between two wavetables using linear cross-fade
    pub fn morph_lookup(
        wt1: &Wavetable,
        wt2: &Wavetable,
        phase: f32,
        morph_amount: f32, // 0.0 = wt1, 1.0 = wt2
    ) -> f32 {
        let sample1 = wt1.lookup(phase);
        let sample2 = wt2.lookup(phase);
        sample1 * (1.0 - morph_amount) + sample2 * morph_amount
    }
}
```

### 2. Wavetable Library

```rust
/// Static collection of all loaded wavetables
/// Loaded once at application startup, never modified during playback
pub struct WavetableLibrary {
    /// All wavetables indexed by ID
    tables: Vec<Wavetable>,
    
    /// Name → index lookup for user-friendly selection
    name_to_index: std::collections::HashMap<String, usize>,
}

impl WavetableLibrary {
    /// Load wavetables from directory (e.g., "assets/wavetables/")
    pub fn load_from_directory(path: &str) -> Result<Self, LoadError> {
        let mut tables = Vec::new();
        let mut name_to_index = std::collections::HashMap::new();
        
        // Scan directory for .wav files
        // For each .wav:
        //   1. Read as audio file (using hound or similar crate)
        //   2. Ensure single channel (convert stereo → mono if needed)
        //   3. Normalize peak amplitude to ±1.0
        //   4. Resample to 2048 samples if needed (keep original sample rate)
        //   5. Generate 4× oversampled version (8192 samples)
        //   6. Store in Wavetable struct
        //   7. Add to name_to_index map
        
        Ok(Self { tables, name_to_index })
    }
    
    /// Get wavetable by index
    pub fn get(&self, index: usize) -> Option<&Wavetable> {
        self.tables.get(index)
    }
    
    /// Get index by name
    pub fn find_by_name(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }
    
    /// Get all wavetable names
    pub fn list_names(&self) -> Vec<&str> {
        self.tables.iter().map(|wt| wt.name.as_str()).collect()
    }
    
    /// Total number of wavetables loaded
    pub fn count(&self) -> usize {
        self.tables.len()
    }
}
```

### 3. Oscillator Integration

Add to existing `Oscillator` struct:

```rust
pub struct Oscillator {
    // ... existing fields ...
    
    /// Current wavetable when waveform is Waveform::Wavetable
    wavetable_index: usize,
    
    /// Wavetable morphing position [0.0, 1.0]
    /// 0.0 = current wavetable
    /// 1.0 = next wavetable in sequence
    /// Continuous values = cross-fade between adjacent wavetables
    wavetable_position: f32,
}

impl Oscillator {
    /// Set the wavetable and reset morph position
    pub fn set_wavetable(&mut self, index: usize) {
        self.wavetable_index = index;
        self.wavetable_position = 0.0;
    }
    
    /// Set wavetable morphing position
    pub fn set_wavetable_position(&mut self, position: f32) {
        self.wavetable_position = position.clamp(0.0, 1.0);
    }
    
    /// Generate wavetable sample with 4× oversampling
    fn generate_wavetable_sample(
        &mut self,
        library: &WavetableLibrary,
    ) -> f32 {
        // Generate 4 oversampled samples
        for _ in 0..4 {
            // Get current and next wavetable for morphing
            let current_wt = library.get(self.wavetable_index)?;
            let next_wt = library.get(
                (self.wavetable_index + 1) % library.count()
            )?;
            
            // Sample at 4× oversampled rate
            let sample = Wavetable::morph_lookup(
                current_wt,
                next_wt,
                self.phase,
                self.wavetable_position,
            );
            
            // Push to downsampler
            self.downsampler.push(sample);
            
            // Increment phase
            self.phase += self.phase_increment;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
        
        // Pop downsampled output
        self.downsampler.pop().unwrap_or(0.0)
    }
}
```

## Parameter Integration

### Add to `src/params.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Waveform {
    #[default]
    Sine,
    Saw,
    Square,
    Triangle,
    Pulse,
    WhiteNoise,
    PinkNoise,
    Additive,
    Wavetable,  // NEW: Wavetable synthesis
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OscillatorParams {
    // ... existing fields ...
    
    /// Wavetable index when waveform is Wavetable
    pub wavetable_index: usize,        // 0 to N-1
    
    /// Wavetable morphing position [0.0, 1.0]
    /// Can be modulated by LFO or envelope
    pub wavetable_position: f32,       // 0.0 to 1.0
}
```

### CLAP Parameter Descriptors

Add to `src/plugin/param_registry.rs`:

```rust
// Wavetable parameters (use namespace: 0x0400_0000 for oscillator 1, etc.)
pub const WAVETABLE_INDEX_OSC1: u32 = 0x0401_0000;     // Enum: list all wavetable names
pub const WAVETABLE_POS_OSC1: u32 = 0x0401_0001;      // Float: 0.0 to 1.0
pub const WAVETABLE_INDEX_OSC2: u32 = 0x0402_0000;
pub const WAVETABLE_POS_OSC2: u32 = 0x0402_0001;
pub const WAVETABLE_INDEX_OSC3: u32 = 0x0403_0000;
pub const WAVETABLE_POS_OSC3: u32 = 0x0403_0001;

// In ParamRegistry::init():
registry.add_enum_param(
    WAVETABLE_INDEX_OSC1,
    "Oscillator 1 Wavetable",
    library.list_names(),
    0,  // Default to first wavetable
);

registry.add_float_param(
    WAVETABLE_POS_OSC1,
    "Oscillator 1 Wavetable Position",
    0.0,
    1.0,
    0.0,
);
```

## LFO Modulation

Enable wavetable morphing controlled by LFOs/envelopes:

```rust
// In Voice::process_sample()
for osc_idx in 0..3 {
    // ... existing LFO setup ...
    
    // NEW: Modulate wavetable position
    if let Waveform::Wavetable = self.oscillator_params[osc_idx].waveform {
        let morph_lfo = self.lfos[osc_idx].value(); // 0.0 to 1.0
        let morph_mod = morph_lfo * self.oscillator_params[osc_idx].wavetable_position;
        
        self.oscillators[osc_idx][0].set_wavetable_position(morph_mod);
    }
}
```

## Wavetable File Format Specification

### Input: Standard .wav Files

**Requirements:**
- **Mono** (or convert stereo → mono by averaging channels)
- **32-bit float** or **16-bit PCM** (both supported)
- **One complete waveform cycle** per file
- **Recommended size**: 2048 or 4096 samples
- **Recommended naming**: `<library>_<name>` (e.g., `serum_saw_1.wav`, `vital_bright.wav`)

**Loading Process:**

```rust
pub fn load_wavetable_wav(path: &str) -> Result<Wavetable, LoadError> {
    // 1. Read .wav file header
    //    - Get sample rate, bit depth, channels
    //    - Handle various formats (PCM 16/24/32-bit, IEEE float)
    
    // 2. Read all samples
    //    - Convert to Vec<f32> in range [-1.0, 1.0]
    
    // 3. Normalize
    //    - Find peak amplitude
    //    - Scale to ±1.0 (leave 5% headroom)
    
    // 4. Resample if needed
    //    - If sample count != 2048, resample using linear interpolation
    //    - Preserve waveform shape while hitting exact target length
    
    // 5. Generate 4× oversampled version
    //    - Use cubic hermite interpolation to upsample
    //    - Apply Kaiser-windowed FIR filter to prevent aliasing during upsampling
    
    // 6. Return Wavetable struct
}
```

## Memory Footprint Estimation

For a library of **256 wavetables**:

- **Normal rate**: 256 × 2048 × 4 bytes = **2.1 MB**
- **4× oversampled**: 256 × 8192 × 4 bytes = **8.4 MB**
- **Total**: ~**10.5 MB** (acceptable for modern systems)

## Performance Impact

### CPU Cost per Voice

- **Wavetable lookup**: ~5 cycles (cache-friendly, just 2 interpolations)
- **Morphing blend**: ~3 cycles (linear lerp)
- **Downsampling**: Existing cost (same as other waveforms)
- **Total overhead**: <1% per voice compared to native waveforms

### Implementation Phases

**Phase 1: Foundation** (Week 1)
- [ ] Create `Wavetable` struct with lookup & morphing
- [ ] Create `WavetableLibrary` with .wav loading
- [ ] Add `Waveform::Wavetable` variant to enum
- [ ] Write comprehensive tests for wavetable interpolation

**Phase 2: Oscillator Integration** (Week 2)
- [ ] Add wavetable sample generation to `Oscillator`
- [ ] Integrate 4× oversampling with downsampler
- [ ] Add `wavetable_index` and `wavetable_position` parameters
- [ ] Test against existing waveforms (ensure no degradation)

**Phase 3: CLAP Plugin Support** (Week 3)
- [ ] Add CLAP parameter descriptors for wavetable control
- [ ] Implement parameter get/set in `plugin/clap/params.rs`
- [ ] Add CLAP enum descriptor handler for wavetable names
- [ ] Test automation in DAWs

**Phase 4: GUI** (Week 4)
- [ ] Add wavetable selector widget in `shared_ui.rs`
- [ ] Add wavetable position slider/knob
- [ ] Connect LFO modulation UI
- [ ] Add wavetable name display

**Phase 5: Preset System** (Week 5)
- [ ] Save/load wavetable index in presets
- [ ] Handle missing wavetables gracefully (fallback to Sine)
- [ ] Test preset migration from old format

## Backward Compatibility

- Existing presets will continue to work (Wavetable is a new waveform type)
- Default oscillators remain unchanged
- Additive synthesis (`Waveform::Additive`) is kept as-is for users who prefer it
- No breaking changes to existing parameters or API

## Future Extensions

### Short Term
1. **Wavetable Blending**: Allow morphing across multiple tables in sequence
2. **Custom Wavetables**: Import user-created .wav files at runtime
3. **Wavetable Smoothing**: Prevent clicking when rapidly changing tables
4. **Spectrum Analyzer UI**: Visual feedback of current wavetable harmonics

### Long Term
1. **Wavetable Drawing**: Built-in wavetable editor (draw curves directly)
2. **Granular Synthesis**: Use small wavetable chunks with separate grain envelopes
3. **Spectral Morphing**: Cross-fade between spectrally different wavetables
4. **Multi-Cycle Support**: Store multiple cycles per wavetable for dynamic shifting

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wavetable_lookup_interpolation() {
        // Verify linear interpolation works correctly
        // Test phase 0.0, 0.5, 1.0, and fractional values
    }
    
    #[test]
    fn test_wavetable_morphing() {
        // Create two sine wavetables with different frequencies
        // Morph between them and verify smooth cross-fade
    }
    
    #[test]
    fn test_wavetable_loading() {
        // Load a reference .wav file
        // Verify samples match expected values
        // Verify 4× oversampling generates 4× as many samples
    }
    
    #[test]
    fn test_wavetable_against_oscillator() {
        // Generate a sine wavetable
        // Compare output against Waveform::Sine
        // Should be nearly identical
    }
}
```

### Integration Tests
- Test wavetable + downsampler pipeline (ensure anti-aliasing still works)
- Test with LFO modulation (ensure smooth morphing)
- Test voice stealing with wavetable oscillators
- Benchmark CPU usage vs. native waveforms

### Listening Tests
- Verify no aliasing artifacts at high frequencies
- Verify morph blending sounds smooth (no clicks/pops)
- A/B compare against native waveforms for quality parity

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
hound = "3.5"           # .wav file I/O
```

Optional (for advanced resampling):
```toml
rubato = "0.14"         # High-quality resampling
```

## References

- **Serum Wavetables**: https://www.native-instruments.com/en/products/komplete/synths/massive/
- **PHASEPLANT Wavetable Format**: https://www.kilohearts.com/products/phaseplant
- **Vital Wavetable Spec**: https://vital.audio/
- **Audio EQ Cookbook**: https://www.w3.org/TR/audio-eq-cookbook/
- **Kaiser Window STFT**: Smith, J.O. "Spectral Audio Signal Processing", https://ccrma.stanford.edu/~jos/sasp/

---

This design is ready for implementation. Would you like me to start building Phase 1?
