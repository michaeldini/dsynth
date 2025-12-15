# DSynth AI Coding Instructions

## Project Overview
DSynth is a **high-performance polyphonic synthesizer** built in Rust with three compilation targets:
- **Standalone**: Complete app with GUI (Iced) + audio I/O (cpal) + MIDI input (midir)
- **VST Plugin**: VST3/CLAP plugins via `nih_plug` wrapper around the same core engine
- **Library**: Reusable synthesizer modules

**Core principle**: The `SynthEngine` is 100% shared between all targets. Only the wrapper layer differs.

## Development Philosophy

### Test-Driven Development
- **Write tests first** before implementing features or fixes
- **76 tests total** validate DSP accuracy, voice management, and audio pipeline
- Use `approx::assert_relative_eq!` for floating-point comparisons
- Run `cargo test` frequently during development

### Sound Quality Over Performance
1. **Sound quality is the #1 priority** - never compromise audio fidelity for speed
2. **Performance is secondary** but still important - target <11% CPU for 16 voices
3. When optimizing, benchmark first (`cargo bench`) to identify actual bottlenecks
4. Use 4× oversampling and proper anti-aliasing even though it costs CPU cycles
5. If a feature would degrade sound quality, discuss alternatives before implementing

### Extensibility
- The codebase is designed for future feature additions
- When adding functionality, maintain the existing architectural patterns
- Keep the core engine format-agnostic (no standalone/plugin-specific code in `audio/` or `dsp/`)

## Critical Architecture Patterns

### Lock-Free Real-Time Communication
- **GUI → Audio Thread**: Triple-buffer (`triple_buffer` crate) for parameter updates
  - Updated every 32 samples (~0.7ms at 44.1kHz) to balance CPU efficiency and responsiveness
  - See [engine.rs](src/audio/engine.rs) `sample_counter` and `param_update_interval`
- **MIDI → Audio Thread**: `Arc<Mutex<>>` for note events (minimal contention acceptable here)
- **Never allocate** in audio thread code - pre-allocate all buffers in constructors

### Voice Management & Polyphony
- **16 pre-allocated voices** in `SynthEngine::voices` array
- **Voice stealing**: When all voices are busy, steals the **quietest voice** (RMS-tracked every 128 samples)
- **Monophonic mode**: Implements **last-note priority** via `note_stack` - when you release a key, automatically plays the previous held note without retriggering

### 4× Oversampling Anti-Aliasing
- All oscillators generate at 4× sample rate internally (176.4kHz for 44.1kHz output)
- **20-tap Kaiser-windowed FIR filter** (β=8.5) downsamples back to target rate
- This prevents aliasing artifacts at high frequencies - see [oscillator.rs](src/dsp/oscillator.rs) and [downsampler.rs](src/dsp/downsampler.rs)

### SIMD Optimizations
- Enabled via `simd` feature flag → requires **Rust nightly** (see [rust-toolchain.toml](rust-toolchain.toml))
- Uses `std::simd::f32x4` for vectorized oscillator processing
- Conditional compilation: `#[cfg(feature = "simd")]` blocks with scalar fallbacks

### Plugin vs Standalone Separation
- **Standalone**: [main.rs](src/main.rs) owns GUI + audio thread + MIDI thread
- **Plugin**: [plugin.rs](src/plugin.rs) is a thin re-export; real implementation in [plugin/](src/plugin/)
  - [plugin/params.rs](src/plugin/params.rs): Maps `nih_plug` parameters → `SynthParams`
  - [plugin/convert.rs](src/plugin/convert.rs): Conversion utilities between plugin and core types
- **Core**: [audio/engine.rs](src/audio/engine.rs) is format-agnostic, just processes samples

## Development Workflows

### Building
```bash
# Standalone (default features)
cargo build --release

# VST Plugin
cargo build --release --lib --features vst
./bundle.sh  # macOS - creates VST3/CLAP bundles in target/bundled/

# Without SIMD (stable Rust)
cargo build --no-default-features --features standalone
```

### Testing (Test-Driven Development)
- **Write tests BEFORE implementation** - TDD is the standard workflow
- **76 tests total**: 73 unit tests embedded in modules, 3 integration tests in [tests/](tests/)
- Run `cargo test` after every significant change
- **Floating-point tests**: Use `approx::assert_relative_eq!` with `epsilon` for DSP accuracy
  - Example pattern in [oscillator.rs](src/dsp/oscillator.rs): `assert_relative_eq!(sample, expected, epsilon = 0.01)`
- **Integration tests**: Test full audio pipeline in [tests/integration_tests.rs](tests/integration_tests.rs)
- When fixing bu (When Appropriate)
```bash
cargo bench  # Uses Criterion, generates HTML reports in target/criterion/report/
```
- **Benchmark before optimizing** - measure to identify real bottlenecks, don't guess
- Benchmarks in [benches/](benches/): `dsp_bench.rs` (individual DSP components) and `optimization_bench.rs` (full engine)
- Target: <11% CPU for 16 voices at 44.1kHz on Apple Silicon
- Add benchmarks for new DSP algorithms or when performance-critical changes are made
- Remember: **sound quality > performance** - don't optimize away audio fidelity
- Benchmarks in [benches/](benches/): `dsp_bench.rs` (individual DSP components) and `optimization_bench.rs` (full engine)
- Target: <11% CPU for 16 voices at 44.1kHz on Apple Silicon

## Code Conventions

### Parameter Update Pattern
When adding new parameters:
1. Add field to `SynthParams` in [params.rs](src/params.rs) with `#[serde(default)]` for preset compatibility
2. For VST: Add `nih_plug` parameter to `DSynthParams` in [plugin/params.rs](src/plugin/params.rs) with unique `#[id = "..."]`
3. Add conversion in [plugin/convert.rs](src/plugin/convert.rs) `DSynthParams::to_synth_params()`
4. For GUI: Add control in [gui/plugin_gui/sections.rs](src/gui/plugin_gui/sections.rs)

### DSP Module Structure
Each DSP component ([dsp/](src/dsp/)) follows this pattern:
- **Phase accumulation** (not sample counting) for frequency control
- **Inline comments** explaining the "why" behind DSP math (see extensive comments in [oscillator.rs](src/dsp/oscillator.rs))
- **Unit tests** at bottom of file in `#[cfg(test)] mod tests { ... }`
- **Sample-rate parametric**: Pass `sample_rate` to constructor, calculate coefficients/increments there

### Filter Stability
- Uses **Audio EQ Cookbook** formulas for biquad coefficients ([filter.rs](src/dsp/filter.rs))
- **Coefficient clamping** prevents numerical instability at extreme settings
- **Parameter smoothing** prevents audio discontinuities when changing cutoff/resonance

### Preset System
- Presets are JSON files serialized from `SynthParams` (see [examples/](examples/))
- Load/save via [preset.rs](src/preset.rs)
- Include `#[serde(default)]` on new fields to maintain backward compatibility with old presets

## Common Pitfalls

❌ **DON'T** allocate (Vec::new(), Box::new(), etc.) in `SynthEngine::process_sample()` or `Voice::process()`
✅ **DO** pre-allocate in constructors

❌ **DON'T** use `Mutex` or blocking locks in audio thread paths
✅ **DO** use triple-buffer for parameters, or accept brief lock contention for infrequent events (like MIDI)

❌ **DON'T** check parameters every sample
✅ **DO** throttle parameter reads (see `param_update_interval` pattern)

❌ **DON'T** assume 44.1kHz sample rate in DSP code
✅ **DO** calculate frequency/time constants from `sample_rate` parameter

## Key Files Reference

- [audio/engine.rs](src/audio/engine.rs): Core synthesis engine, voice management, parameter throttling
- [audio/voice.rs](src/audio/voice.rs): Single voice (3 oscillators + 3 filters + envelope)
- [dsp/oscillator.rs](src/dsp/oscillator.rs): Oversampled waveform generation with anti-aliasing
- [dsp/filter.rs](src/dsp/filter.rs): Biquad filters with stability guarantees
- [params.rs](src/params.rs): Shared parameter definitions for all targets
- [plugin/params.rs](src/plugin/params.rs): VST parameter mapping (`nih_plug` → core params)
- [main.rs](src/main.rs): Standalone app entry point (GUI + audio + MIDI threads)

## Distribution
- Use bundling scripts: [bundle.sh](bundle.sh) (macOS), [bundle.bat](bundle.bat) (Windows), [bundle-linux.sh](bundle-linux.sh) (Linux)
- See [BUILD_AND_DISTRIBUTE.md](BUILD_AND_DISTRIBUTE.md) for cross-compilation and GitHub Actions setup
- Plugins install to standard locations: `~/Library/Audio/Plug-Ins/VST3/` (macOS), `%COMMONPROGRAMFILES%\VST3\` (Windows)
