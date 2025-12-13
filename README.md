# DSynth - Production-Grade Digital Synthesizer

A high-performance, cross-platform digital synthesizer built in Rust using test-driven development.

## Features

### Audio Engine
- **16-voice polyphony** with intelligent voice stealing (quietest-voice algorithm)
- **3 oscillators per voice** with individual filters
- **Waveforms**: Sine, Sawtooth, Square, Triangle
- **4× oversampling** with Kaiser-windowed FIR downsampler for anti-aliasing
- **Biquad filters** (Lowpass, Highpass, Bandpass) with stability guarantees
- **ADSR envelope** generator per voice
- **Sample-rate agnostic** design (parametric by sample rate)

### Performance
- **SIMD optimizations** using `std::simd` for oscillator processing
- **Lock-free parameter updates** via triple-buffer
- **<11% CPU usage** for 16 voices at 44.1kHz (measured on Apple Silicon)
- Benchmarks:
  - Single oscillator: ~23-37ns per sample
  - Single voice (3 osc + 3 filter): ~119ns per sample
  - 16-voice engine: ~2.4µs per sample

### Input
- **MIDI support** via `midir`
- **Computer keyboard input** for playing notes
  - AWSEDFTGYHUJKOLP maps to chromatic piano keys (C4-D#5)
  - ZXCVBNM maps to lower octave (C3-B3)
- Real-time parameter control through GUI

### GUI
- **Cross-platform** GUI using Iced 0.13
- **Real-time controls** for all synthesis parameters:
  - 3 oscillator panels (waveform, pitch, detune, gain, pan, unison, phase)
  - 3 filter sections (type, cutoff, resonance, drive)
  - Master gain control
  - Panic button (all notes off)
- **Preset management**: Save and load presets as JSON files
- Lock-free parameter updates to audio thread

### Testing
- **76 total tests** (73 unit + 3 integration)
- Test-driven development throughout
- Floating-point accuracy tests using `approx` crate
- Full audio pipeline integration tests
- Performance benchmarks using Criterion

## Architecture

### DSP Pipeline
```
MIDI Input → SynthEngine → Voice (16×) → Audio Output
                ↓               ↓
            Triple Buffer   Oscillator (3×) → Filter (3×) → Envelope → Mix
                            (4× oversampled)  (Biquad)      (ADSR)
```

### Threading Model
- **Main thread**: Iced GUI runtime
- **Audio thread**: cpal audio callback (lock-free reads from triple-buffer)
- **MIDI thread**: MIDI event processing and engine control

### Lock-Free Communication
- **GUI → Audio**: Triple-buffer for parameter updates
- **MIDI → Audio**: Arc<Mutex<>> for note events (minimal contention)

## Technical Details

### Anti-Aliasing
- 4× oversampling at oscillator level
- 20-tap Kaiser window FIR filter (β=8.5) for downsampling
- Windowed-sinc interpolation for high stopband attenuation

### Filter Design
- Audio EQ Cookbook formulas for coefficient calculation
- Direct Form I implementation
- Coefficient clamping for numerical stability
- Parameter smoothing to prevent discontinuities

### Voice Stealing
- RMS tracking per voice (updated every 128 samples)
- Quietest-voice selection algorithm
- Seamless voice reallocation

### SIMD Optimizations
- Vectorized oscillator processing using `std::simd::f32x4`
- 4 oversampled values generated in parallel
- Manual loop unrolling in voice mixing

## Building

### Requirements
- Rust nightly (for `portable_simd` feature)
- macOS, Linux, or Windows
- Audio output device

### Build
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Build without SIMD (uses scalar fallback)
cargo build --release --no-default-features
```

### Run
```bash
# Run the synthesizer
cargo run --release

# Run tests
cargo test --release

# Run benchmarks
cargo bench
```

## Usage

1. Launch the application: `cargo run --release`
2. The GUI will appear with 3 oscillator/filter sections
3. Connect a MIDI controller (optional) or use computer keyboard
4. Adjust oscillator waveforms, pitch, detune, and gain
5. Configure filters (type, cutoff, resonance) for each oscillator
6. Adjust master gain
7. Press "PANIC" to stop all notes

### Preset Management
- **Save Preset**: Enter a name in the preset field and click "Save" to export current settings as JSON
- **Load Preset**: Click "Load" to open a file dialog and load a previously saved preset
- Presets are stored as human-readable JSON files for easy editing and sharing

### Computer Keyboard Mapping
- **AWSEDFTGYHUJKOLP**: Chromatic piano keys (C4 to D#5)
- **ZXCVBNM**: Lower octave (C3 to B3)

## Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs              # Library root
├── params.rs           # Parameter definitions
├── preset.rs           # Preset save/load
├── dsp/
│   ├── downsampler.rs  # Kaiser-windowed FIR downsampler
│   ├── oscillator.rs   # 4× oversampled oscillator with SIMD
│   ├── filter.rs       # Biquad filter
│   └── envelope.rs     # ADSR envelope generator
├── audio/
│   ├── voice.rs        # Voice (3 osc + 3 filter + envelope)
│   ├── engine.rs       # Polyphonic synthesis engine
│   └── output.rs       # cpal audio output
├── midi/
│   └── handler.rs      # MIDI input handling
└── gui/
    └── mod.rs          # Iced GUI implementation

tests/
└── integration_tests.rs # End-to-end integration tests

benches/
└── dsp_bench.rs        # Performance benchmarks
```

## Dependencies

### Core
- `iced` 0.13 - Cross-platform GUI framework
- `cpal` 0.15 - Cross-platform audio I/O
- `midir` 0.10 - Cross-platform MIDI input
- `triple_buffer` 6.0 - Lock-free triple buffering
- `crossbeam-channel` 0.5 - MPSC channels for MIDI events
- `serde` 1.0 - Serialization for presets
- `rfd` 0.15 - Native file dialogs

### Development
- `approx` 0.5 - Floating-point assertions
- `criterion` 0.5 - Benchmarking framework
- `tempfile` 3.0 - Temporary files for tests

## Performance Tuning

### Buffer Size
Adjust audio buffer size in `src/audio/output.rs` for latency vs. stability trade-off.

### SIMD
Enable/disable SIMD via features:
- `--features simd` (default) - Use SIMD optimizations
- `--no-default-features` - Scalar fallback

### Oversampling
Modify `OVERSAMPLE_FACTOR` in `oscillator.rs` (currently 4×). Higher = better quality, more CPU.

## Testing

```bash
# Run all tests
cargo test --release

# Run with output
cargo test --release -- --nocapture

# Run specific test suite
cargo test --lib --release          # Unit tests
cargo test --test integration_tests  # Integration tests

# Check coverage
cargo test --lib
```

## Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench -- oscillator
cargo bench -- voice
cargo bench -- engine

# Save baseline
cargo bench -- --save-baseline main

# Compare to baseline
cargo bench -- --baseline main
```

## Future Enhancements

- [ ] Additional waveforms (pulse, noise)
- [ ] LFO modulation
- [ ] Effects (reverb, delay, chorus)
- [ ] Preset browser with categories
- [ ] Automation recording
- [ ] VST3 plugin wrapper
- [ ] Additional filter types (notch, allpass)
- [ ] Polyphonic aftertouch support
- [ ] Preset search and tagging
- vst plugin wrapper
- midi cc mapping

## License

MIT

## Acknowledgments

- Audio EQ Cookbook by Robert Bristow-Johnson for filter formulas
- Kaiser window design from Julius O. Smith III's DSP resources
- Rust audio community for excellent crates (cpal, midir)
