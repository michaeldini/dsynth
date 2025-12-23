# DSynth AI Coding Instructions

## Project Overview
DSynth is a **high-performance polyphonic synthesizer** built in Rust with three compilation targets:
- **Standalone**: Complete app with GUI (Iced) + audio I/O (cpal) + MIDI input (midir)
- **CLAP Plugin**: Native CLAP plugin with custom wrapper and VIZIA GUI (baseview backend)
- **Library**: Reusable synthesizer modules

**Core principle**: The `SynthEngine` is 100% shared between all targets. Only the wrapper layer differs.

**GUI Architecture**: Moving towards a **unified GUI** - one shared codebase that works identically for both plugin and standalone, reducing maintenance burden and ensuring UI consistency. CLAP plugin now uses VIZIA framework (git version) with baseview backend for native window embedding.

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

### CLAP Parameter System
- **Custom parameter descriptors** in [param_descriptor.rs](src/plugin/param_descriptor.rs):
  - Each parameter has unique ID (namespace pattern: upper 8 bits = module, lower 24 bits = index)
  - Supports Float, Bool, Enum, Int types with ranges, units, and automation flags
  - Logarithmic/exponential skewing for frequency/time parameters
- **Centralized registry** in [param_registry.rs](src/plugin/param_registry.rs):
  - Global `ParamRegistry` for parameter lookup by ID
  - Handles normalization (internal range ↔ CLAP 0.0-1.0 range)
- **Parameter flow**: DAW automation → `params_flush()` → `param_apply::apply_param()` → `SynthParams` → triple-buffer → audio thread
- **Enum parameters**: Must return indices (0, 1, 2, ...) in `param_get`, CLAP normalizes to 0-1
- **Important**: Always clamp parameter values to prevent extreme values (see pitch ±24 semitones)

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

### VIZIA GUI Architecture (CLAP Plugin)
- **Framework**: [VIZIA](https://github.com/vizia/vizia) (git version) with baseview backend
- **Current state**: 
  - CLAP plugin uses VIZIA for native window embedding
  - Standalone still uses [iced](https://github.com/iced-rs/iced) (separate implementation)
  - VIZIA provides reactive UI with entity-component system
- **Implementation**:
  - Located in [gui/vizia_gui/](src/gui/vizia_gui/): VIZIA-specific plugin GUI
  - `GuiState` with `Arc<RwLock<SynthParams>>` for shared parameter access
  - `GuiMessage` enum for events (ParamChanged, PresetLoad, etc.)
  - Layout-first approach: VStack/HStack with Labels, custom widgets coming
  - `WindowHandleWrapper` for raw-window-handle 0.5 compatibility
- **Future unification**:
  - Extract shared components to [gui/shared/](src/gui/shared/) when both GUIs stabilize
  - Trait-based parameter binding for consistent behavior
- **Benefits**: Native window embedding, reactive updates, clean separation from iced standalone

### Plugin vs Standalone Separation
- **Standalone**: [main.rs](src/main.rs) owns GUI + audio thread + MIDI thread
- **CLAP Plugin**: [plugin/clap/](src/plugin/clap/) implements native CLAP interface
  - [plugin/clap/plugin.rs](src/plugin/clap/plugin.rs): Main CLAP plugin instance and lifecycle
  - [plugin/clap/processor.rs](src/plugin/clap/processor.rs): Audio processing and MIDI handling
  - [plugin/clap/params.rs](src/plugin/clap/params.rs): CLAP parameter extension (automation, query, flush)
  - [plugin/clap/state.rs](src/plugin/clap/state.rs): Save/load state for presets
  - [plugin/param_descriptor.rs](src/plugin/param_descriptor.rs): Custom parameter system (IDs, ranges, units)
  - [plugin/param_registry.rs](src/plugin/param_registry.rs): Centralized parameter lookup and normalization
  - [plugin/param_update.rs](src/plugin/param_update.rs): Lock-free parameter updates via triple-buffer
- **GUI**: Separate implementations (unified architecture planned)
  - [gui/vizia_gui/](src/gui/vizia_gui/): VIZIA GUI for CLAP plugin (baseview backend)
    - [plugin_window.rs](src/gui/vizia_gui/plugin_window.rs): Window integration and layout
    - [state.rs](src/gui/vizia_gui/state.rs): GuiState with Arc<RwLock<SynthParams>>
    - [messages.rs](src/gui/vizia_gui/messages.rs): GuiMessage enum for events
    - [widgets/](src/gui/vizia_gui/widgets/): Custom parameter controls
  - [gui/standalone_gui/](src/gui/standalone_gui/): Iced GUI for standalone app
  - **Goal**: Extract shared components to [gui/shared/](src/gui/shared/) for code reuse
- **Core**: [audio/engine.rs](src/audio/engine.rs) is format-agnostic, just processes samples

## Development Workflows

### Building
```bash
# Standalone (default features)
cargo build --release

# CLAP Plugin
cargo build --release --lib --features clap
./bundle_clap.sh  # macOS - creates CLAP bundle in target/bundled/
cp -r target/bundled/DSynth.clap ~/Library/Audio/Plug-Ins/CLAP/

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
1. Add CLAP Plugin:
   - Add parameter descriptor to [plugin/param_registry.rs](src/plugin/param_registry.rs) with unique ID (< 0xFFFFFFFF)
   - Add parameter mapping in [plugin/param_update.rs](src/plugin/param_update.rs):
     - `param_apply::apply_param()` for host → engine updates
     - `param_get::get_param()` for engine → host queries
2. For VIZIA Plugin GUI:
   - Add control in [gui/vizia_gui/plugin_window.rs](src/gui/vizia_gui/plugin_window.rs) layout sections
   - Use `param_knob()` widget function with parameter ID, label, and initial value
   - Widget interactions emit `GuiMessage::ParamChanged(param_id, normalized_value)`
   - Events flow → `param_update_buffer` → audio thread via triple-buffer
3. For Iced Standalone GUI:
   - Add control in [gui/standalone_gui/sections.rs](src/gui/standalone_gui/sections.rs)
   - Widget changes → `Message::ParamChanged` → `param_update_buffer` → audio thread

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

❌ **DON'T** return raw enum indices from `param_get` without considering CLAP normalization
✅ **DO** return enum index (0, 1, 2, ...) and let `normalize_param_value()` convert to 0-1

❌ **DON'T** forget to clamp parameter values in `param_apply`
✅ **DO** add `.clamp(min, max)` for parameters with restricted ranges (pitch, detune, etc.)

## Key Files Reference
- [audio/engine.rs](src/audio/engine.rs): Core synthesis engine, voice management, parameter throttling
- [audio/voice.rs](src/audio/voice.rs): Single voice (3 oscillators + 3 filters + envelope)
- [dsp/oscillator.rs](src/dsp/oscillator.rs): Oversampled waveform generation with anti-aliasing
- [dsp/filter.rs](src/dsp/filter.rs): Biquad filters with stability guarantees
- [params.rs](src/params.rs): Shared parameter definitions for all targets

### CLAP Plugin System
- [plugin/clap/plugin.rs](src/plugin/clap/plugin.rs): CLAP plugin lifecycle and extension registration
- [plugin/clap/processor.rs](src/plugin/clap/processor.rs): Real-time audio + MIDI processing
- [plugin/clap/params.rs](src/plugin/clap/params.rs): CLAP parameter extension (count, info, get/set, flush)
- [plugin/clap/state.rs](src/plugin/clap/state.rs): Preset save/load via CLAP state extension
- [plugin/param_descriptor.rs](src/plugin/param_descriptor.rs): Parameter metadata (type, range, unit, automation)
- [plugin/param_registry.rs](src/plugin/param_registry.rs): Global parameter registry and normalization
- [plugin/param_update.r
  - [bundle_clap.sh](bundle_clap.sh) (macOS CLAP)
  - [bundle_standalone.sh](bundle_standalone.sh) (macOS standalone)
  - Platform-specific variants for Linux/Windows
- See [BUILD_AND_DISTRIBUTE.md](BUILD_AND_DISTRIBUTE.md) for cross-compilation and GitHub Actions setup
- CLAP plugins install to: `~/Library/Audio/Plug-Ins/CLAP/` (macOS), `%COMMONPROGRAMFILES%\CLAP\` (Windows), `~/.clap/` (Linux)

### GUI System
- [gui/vizia_gui/](src/gui/vizia_gui/): VIZIA GUI for CLAP plugin
  - [plugin_window.rs](src/gui/vizia_gui/plugin_window.rs): Window integration, layout, ADSR envelope section
  - [state.rs](src/gui/vizia_gui/state.rs): GuiState with Arc<RwLock<SynthParams>>
  - [messages.rs](src/gui/vizia_gui/messages.rs): GuiMessage enum (ParamChanged, PresetLoad, etc.)
  - [widgets/param_knob.rs](src/gui/vizia_gui/widgets/param_knob.rs): Parameter control widget
- [gui/standalone_gui/app.rs](src/gui/standalone_gui/app.rs): Iced GUI for standalone application
- [gui/shared/](src/gui/shared/): **Future home** of unified GUI components (sections, widgets, messages)

### Entry Points
- [main.rs](src/main.rs): Standalone app entry point (GUI + audio + MIDI threads)
- [lib.rs](src/lib.rs): Library exports and CLAP plugin entry point envelope)
- [dsp/oscillator.rs](src/dsp/oscillator.rs): Oversampled waveform generation with anti-aliasing
- [dsp/filter.rs](src/dsp/filter.rs): Biquad filters with stability guarantees
- [params.rs](src/params.rs): Shared parameter definitions for all targets
- [plugin/params.rs](src/plugin/params.rs): VST parameter mapping (`nih_plug` → core params)
- [main.rs](src/main.rs): Standalone app entry point (GUI + audio + MIDI threads)

## Distribution
- Use bundling scripts: [bundle.sh](bundle.sh) (macOS), [bundle.bat](bundle.bat) (Windows), [bundle-linux.sh](bundle-linux.sh) (Linux)
- See [BUILD_AND_DISTRIBUTE.md](BUILD_AND_DISTRIBUTE.md) for cross-compilation and GitHub Actions setup
- Plugins install to standard locations: `~/Library/Audio/Plug-Ins/VST3/` (macOS), `%COMMONPROGRAMFILES%\VST3\` (Windows)
