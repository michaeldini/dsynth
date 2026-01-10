# DSynth AI Coding Instructions

## Project Overview
DSynth is a **high-performance polyphonic synthesizer** built in Rust with three compilation targets:
- **Standalone**: Complete app with VIZIA GUI (winit backend) + audio I/O (cpal) + MIDI input (midir)
- **CLAP Plugin**: Native CLAP plugin with custom wrapper and VIZIA GUI (baseview backend)
- **Library**: Reusable synthesizer modules

**Core principle**: The `SynthEngine` is 100% shared between all targets. Only the wrapper layer differs.

**GUI Architecture**: **Unified VIZIA GUI** - one shared codebase that works identically for both plugin and standalone, using different backends (baseview for CLAP, winit for standalone). This ensures UI consistency and reduces maintenance burden.

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

### VIZIA GUI Architecture
- **Framework**: [VIZIA](https://github.com/vizia/vizia) (git version) with dual backends
- **Unified implementation**: 
  - CLAP plugin uses baseview backend for native window embedding
  - Standalone uses winit backend for desktop application window
  - Both share the same VIZIA UI code for consistency
- **Implementation**:
  - Located in [gui/vizia_gui/](src/gui/vizia_gui/): Unified VIZIA GUI
  - `GuiState` with `Arc<RwLock<SynthParams>>` for shared parameter access
  - `GuiMessage` enum for events (ParamChanged, PresetLoad, etc.)
  - Layout-first approach: VStack/HStack with Labels and custom widgets
  - `shared_ui.rs` contains the shared UI layout used by both targets
- **Benefits**: Native window embedding, reactive updates, consistent UI across targets

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
- **GUI**: Unified VIZIA implementation
  - [gui/vizia_gui/](src/gui/vizia_gui/): Shared VIZIA GUI for both targets
    - [plugin_window.rs](src/gui/vizia_gui/plugin_window.rs): CLAP plugin window (baseview)
    - [standalone_window.rs](src/gui/vizia_gui/standalone_window.rs): Standalone window (winit)
    - [shared_ui.rs](src/gui/vizia_gui/shared_ui.rs): Shared UI layout used by both
    - [state.rs](src/gui/vizia_gui/state.rs): GuiState with Arc<RwLock<SynthParams>>
    - [messages.rs](src/gui/vizia_gui/messages.rs): GuiMessage enum for events
    - [widgets/](src/gui/vizia_gui/widgets/): Custom parameter controls
- **Core**: [audio/engine.rs](src/audio/engine.rs) is format-agnostic, just processes samples

## DSynth Kick: Simplified Kick Drum Synthesizer

The project includes a **separate kick drum synthesizer** (`DSynthKick`) optimized for electronic music kick drum synthesis. It shares architectural patterns with the main synth but is simplified and specialized.

### Key Differences from Main Synth

**Simplified Voice Model:**
- **Single monophonic voice** (no polyphony/voice stealing) - kicks are typically one-shot
- **2 oscillators** instead of 3 (Body/Tone + Click/Transient)
- **Exponential pitch envelopes** for classic 808-style pitch sweeps (start_pitch → end_pitch over decay time)
- **Single filter** with envelope modulation
- **Distortion module** with 4 types (Soft, Hard, Tube, Foldback)

**Specialized Parameters:**
- Oscillator parameters use **absolute Hz values** (not MIDI note conversion by default)
- Pitch envelopes are separate from amplitude envelope (independent decay times)
- Filter envelope is simplified (fast attack, adjustable decay, no sustain)
- Velocity sensitivity controls amplitude scaling

**Key Tracking System:**
- Added in v0.3.0 to enable chromatic kick playback
- `key_tracking` parameter (0.0-1.0): scales pitch envelope by MIDI note frequency
- Formula: `key_tracking_mult = (note_freq / ref_freq).powf(key_tracking)`
- Reference note: C4 (60) = 261.63 Hz
- At `key_tracking=0.0`: All notes trigger same pitch (default 808 behavior)
- At `key_tracking=1.0`: Full chromatic tracking (C5 = 2× C4 pitch)
- Preserves pitch envelope sweep ratio (start/end ratio constant across notes)

### Kick Synth Architecture Files

**Core Audio Engine:**
- [audio/kick_engine.rs](src/audio/kick_engine.rs): Monophonic engine, MIDI event handling, direct trigger method
- [audio/kick_voice.rs](src/audio/kick_voice.rs): Single voice with pitch envelopes, key tracking math, distortion
  - `trigger(note: u8, velocity: f32, params: &KickParams)`: Note-on with key tracking
  - `midi_note_to_freq()`: Equal temperament conversion (A4=440Hz)
  - `calculate_pitch_envelope()`: Exponential decay formula

**Parameters:**
- [params_kick.rs](src/params_kick.rs): `KickParams` struct with 20 parameters
  - Osc 1: `pitch_start`, `pitch_end`, `pitch_decay`, `level`
  - Osc 2: `pitch_start`, `pitch_end`, `pitch_decay`, `level`
  - Envelope: `amp_attack`, `amp_decay`, `amp_sustain`, `amp_release`
  - Filter: `cutoff`, `resonance`, `env_amount`, `env_decay`
  - Distortion: `amount`, `type` (enum)
  - Master: `level`, `velocity_sensitivity`, `key_tracking`
  - Includes preset methods: `preset_808()`, `preset_techno()`, `preset_sub()`

**CLAP Plugin:**
- [plugin/kick_param_registry.rs](src/plugin/kick_param_registry.rs): Parameter descriptors and registry
  - Namespace: `0x0200_xxxx` (separate from main synth `0x0100_xxxx`)
  - `apply_param()` and `get_param()` for parameter synchronization
  - Logarithmic scaling for pitch/time parameters
- [plugin/clap/kick_plugin.rs](src/plugin/clap/kick_plugin.rs): CLAP plugin entry point and lifecycle
- [plugin/clap/kick_processor.rs](src/plugin/clap/kick_processor.rs): Audio processing callback

**GUI:**
- [gui/kick_plugin_window.rs](src/gui/kick_plugin_window.rs): VIZIA GUI for kick plugin
  - Uses same pattern as main synth: `param_knob()` helper, section builders
  - Sections: Body Osc, Click Osc, Envelope, Filter, Distortion, Master
  - Background image embedded as `KICK_BG_JPG` constant

**Entry Points:**
- [lib.rs](src/lib.rs): Exports kick plugin via `clap_entry` macro when `kick-clap` feature enabled

### Building Kick Synth

```bash
# Kick CLAP Plugin
cargo build --release --lib --features kick-clap
./bundle_kick_clap.sh  # macOS - creates DSynthKick.clap
cp -r target/bundled/DSynthKick.clap ~/Library/Audio/Plug-Ins/CLAP/
```

### Adding Parameters to Kick Synth

Follow this pattern (example: adding `key_tracking`):

1. **Add to KickParams** ([params_kick.rs](src/params_kick.rs)):
   ```rust
   pub struct KickParams {
       // ... existing fields ...
       pub key_tracking: f32, // 0.0-1.0
   }
   ```
   - Add to `Default` impl with sensible default
   - Add to all preset methods to maintain compatibility

2. **Register in KickParamRegistry** ([kick_param_registry.rs](src/plugin/kick_param_registry.rs)):
   ```rust
   pub const PARAM_KICK_KEY_TRACKING: ParamId = 0x0200_0052;
   
   add_param!(
       PARAM_KICK_KEY_TRACKING,
       ParamDescriptor::float(
           PARAM_KICK_KEY_TRACKING,
           "Key Tracking",
           "Master",
           0.0, 1.0, 0.0,  // min, max, default
           Some("%")
       )
   );
   ```
   - Add case to `apply_param()` match statement
   - Add case to `get_param()` match statement

3. **Implement in Voice** ([kick_voice.rs](src/audio/kick_voice.rs)):
   - Access parameter in `process()` method
   - Apply DSP logic (e.g., key tracking multiplier calculation)

4. **Add GUI Control** ([kick_plugin_window.rs](src/gui/kick_plugin_window.rs)):
   ```rust
   let master_row = [
       item(PARAM_KICK_KEY_TRACKING, "KeyTrack"),
       // ... other params ...
   ];
   ```

5. **Write Tests** ([kick_voice.rs](src/audio/kick_voice.rs) `#[cfg(test)]`):
   - Test parameter behavior in `tests` module
   - Use `approx::assert_relative_eq!` for DSP accuracy

### Kick Synth Testing

Tests are in [audio/kick_voice.rs](src/audio/kick_voice.rs) `#[cfg(test)]` module:
```bash
cargo test --lib --features kick-clap
```

Current test coverage:
- `test_kick_voice_creation`: Voice initialization
- `test_trigger_activates_voice`: Note-on activation
- `test_pitch_envelope_decay`: Exponential pitch envelope accuracy
- `test_process_generates_audio`: Audio generation verification
- `test_voice_eventually_stops`: Envelope deactivation
- `test_distortion_types`: All distortion modes produce valid audio
- `test_velocity_sensitivity`: Velocity → amplitude scaling
- `test_key_tracking`: Chromatic pitch scaling (C5 = 2× C4)
- `test_key_tracking_disabled`: Default behavior (no tracking)
- `test_midi_note_to_freq`: MIDI conversion accuracy (A4=440Hz)

### Common Patterns for Kick Synth

**Pitch Envelope Usage:**
```rust
// Classic 808 kick: high → low pitch sweep
osc1_pitch_start: 150.0,  // Starting tone
osc1_pitch_end: 55.0,     // Fundamental pitch
osc1_pitch_decay: 100.0,  // Sweep time (ms)
```

**Key Tracking Integration:**
```rust
// Scale pitch envelope by note frequency
let key_tracking_mult = if params.key_tracking > 0.0 {
    let note_freq = Self::midi_note_to_freq(self.note);
    let ref_freq = Self::midi_note_to_freq(60); // C4
    (note_freq / ref_freq).powf(params.key_tracking)
} else {
    1.0  // No tracking
};
let tracked_start = params.osc1_pitch_start * key_tracking_mult;
```

**MIDI Note Handling:**
- Engine captures note number from `NoteOn` events
- Passes to `voice.trigger(note, velocity, params)`
- Voice stores `note: u8` field for key tracking calculation
- Non-MIDI `trigger()` uses C4 (60) as default

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
2. For VIZIA GUI (both plugin and standalone):
   - Add control in [gui/vizia_gui/shared_ui.rs](src/gui/vizia_gui/shared_ui.rs) layout sections
   - Use `param_knob()` widget function with parameter ID, label, and initial value
   - Widget interactions emit `GuiMessage::ParamChanged(param_id, normalized_value)`
   - Events flow → `param_update_buffer` → audio thread via triple-buffer

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
- Depracated for now - use DAW preset management via CLAP state extension
- Presets are JSON files serialized from `SynthParams`
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
- [plugin/param_update.rs](src/plugin/param_update.rs): Lock-free parameter updates via triple-buffer

### Bundling Scripts
- **CLAP Plugin**:
  - [bundle.sh](bundle.sh) (macOS CLAP)
  - [bundle_standalone.sh](bundle_standalone.sh) (macOS standalone)
  - Platform-specific variants for Linux/Windows
- See [BUILD_AND_DISTRIBUTE.md](BUILD_AND_DISTRIBUTE.md) for cross-compilation and GitHub Actions setup
- CLAP plugins install to: `~/Library/Audio/Plug-Ins/CLAP/` (macOS), `%COMMONPROGRAMFILES%\CLAP\` (Windows), `~/.clap/` (Linux)

### GUI System
- [gui/](src/gui/): Unified VIZIA GUI for both targets
  - [plugin_window.rs](src/gui/plugin_window.rs): CLAP plugin window integration (baseview)
  - [standalone_window.rs](src/gui/standalone_window.rs): Standalone window integration (winit)
  - [state.rs](src/gui/state.rs): GuiState with Arc<RwLock<SynthParams>>
  - [messages.rs](src/gui/messages.rs): GuiMessage enum (ParamChanged, PresetLoad, etc.)
  - [theme.rs](src/gui/theme.rs): VIZIA theme and styling
  - [shared_ui/](src/gui/shared_ui/): Modularized UI components for all sections
    - [oscillators.rs](src/gui/shared_ui/oscillators.rs): Oscillator control panel
    - [filters.rs](src/gui/shared_ui/filters.rs): Filter control panel
    - [dynamics.rs](src/gui/shared_ui/dynamics.rs): ADSR envelope and dynamics controls
    - [lfos.rs](src/gui/shared_ui/lfos.rs): LFO control panel
    - [master.rs](src/gui/shared_ui/master.rs): Master output and settings
    - [effects/](src/gui/shared_ui/effects/): Effect sections for reverb, delay, etc.
    - [helpers.rs](src/gui/shared_ui/helpers.rs): Utility functions for UI construction
    - [traits.rs](src/gui/shared_ui/traits.rs): Common traits for UI components
  - [widgets/](src/gui/widgets/): Reusable parameter control widgets
    - [knob.rs](src/gui/widgets/knob.rs): Rotary knob widget for parameters
    - [param_checkbox.rs](src/gui/widgets/param_checkbox.rs): Boolean toggle widget
    - [param_cycle_button.rs](src/gui/widgets/param_cycle_button.rs): Cycle/enum selector widget
    - [vslider.rs](src/gui/widgets/vslider.rs): Vertical slider widget
    - [envelope_editor.rs](src/gui/widgets/envelope_editor.rs): Visual envelope editor

### Entry Points
- [main.rs](src/main.rs): Standalone app entry point (GUI + audio + MIDI threads)
- [lib.rs](src/lib.rs): Library exports and CLAP plugin entry point

## Distribution
- Use bundling scripts:
  - [bundle.sh](bundle.sh) (macOS CLAP plugin)
  - [bundle_standalone.sh](bundle_standalone.sh) (macOS standalone app)
  - Platform-specific variants: [bundle.bat](bundle.bat) (Windows), [bundle-linux.sh](bundle-linux.sh) (Linux)
- See [BUILD_AND_DISTRIBUTE.md](BUILD_AND_DISTRIBUTE.md) for cross-compilation and GitHub Actions setup
- CLAP plugins install to standard locations:
  - macOS: `~/Library/Audio/Plug-Ins/CLAP/`
  - Windows: `%COMMONPROGRAMFILES%\CLAP\`
  - Linux: `~/.clap/`
