# DSynth CLAP Migration Plan

**Objective**: Replace `nih_plug` with raw CLAP + `iced_baseview` + `iced_audio` for maximum GUI flexibility and cross-platform plugin support.

**Timeline**: 8-12 weeks (prototype in 3-4 weeks)  
**Platforms**: macOS, Linux, Windows (CLAP)  
**Status**: Planning Phase

---

## Executive Summary

### Why This Migration?
- ✅ **GUI Freedom**: Move beyond sliders to custom widgets (knobs, XY pads, waveforms, visualizers)
- ✅ **Cross-Platform**: CLAP works consistently across macOS/Linux/Windows
- ✅ **Future-Proof**: CLAP is the emerging standard; nih_plug is stable but less actively developed
- ✅ **Simpler Wrapper**: CLAP is C-based; VST3 is C++ complexity
- ✅ **Unified GUI**: ⭐ **NEW** - One GUI codebase works for both plugin AND standalone (saves maintenance burden)

### What Stays Unchanged (95% of Code)
- `SynthEngine` — zero changes
- `SynthParams` — zero changes  
- Audio processing pipeline — identical
- Preset serialization logic — mostly identical
- Tests and benchmarks — run unmodified

### What Changes (5% of Code)
- Parameter definition system (custom descriptors instead of `nih_plug` macros)
- Plugin lifecycle (CLAP interface instead of `Plugin` trait)
- GUI framework (unified `iced` + `iced_audio` for both plugin and standalone)
- Feature flags in `Cargo.toml`

### Bonus: Unified GUI (Phase 3.5) ⭐ **NEW ADDITION**
After migrating to CLAP, we can achieve something unique: **a single GUI codebase that works identically for both plugin and standalone**. This is possible because:
1. Plugin will use `iced_baseview` (embeds `iced`)
2. Standalone already uses raw `iced`
3. Abstracting parameter binding creates a unified layer

**Benefits**:
- ✅ Write GUI features once, use everywhere
- ✅ Identical UI/UX across both targets
- ✅ 50% reduction in GUI maintenance code
- ✅ Every new widget/section benefits both targets automatically
- ✅ Single visual design system

---

## Architecture

### Before (nih_plug)
```
DAW (Host)
    ↓
[nih_plug wrapper]
    ├─→ nih_plug::Plugin trait
    ├─→ nih_plug_iced (GUI framework)
    └─→ SynthEngine (audio processing)
```

### After (CLAP)
```
DAW (Host) ←→ CLAP Interface
                    ↓
        [DSynthClap Wrapper]
            ├─→ Plugin Lifecycle (create, activate, process)
            ├─→ Parameter System (descriptors + automation)
            ├─→ MIDI Handler (note events)
            └─→ Preset Manager (state serialization)
                    ↓
        [iced_baseview GUI]
            ├─→ iced_audio widgets (Knob, XY Pad, Waveform)
            ├─→ Custom layouts
            └─→ Parameter binding → CLAP automation
                    ↓
            SynthEngine (untouched)
```

### Thread Model
```
┌─────────────────────────────────────────┐
│ CLAP Host                               │
└──────────┬──────────────────────────────┘
           │
    ┌──────┴──────────┐
    │                 │
Audio Thread      GUI Thread
(process_sample)  (iced_baseview)
    │                 │
    ├─ SynthEngine    ├─ Parameter reads
    │                 │
    └────TripleBuffer─┤ (parameter updates)
                      │
              Event Queue
              (MIDI/preset changes)
```

---

## Phase Breakdown

### Phase 1: Parameter System (2-3 weeks)

#### Objective
Replace `nih_plug`'s parameter macros with a custom descriptor system that CLAP can consume.

#### Deliverables
- Custom `ParamDescriptor` struct supporting ranges, units, automation
- Thread-safe parameter update mechanism (using existing triple-buffer pattern)
- CLAP parameter ID → internal parameter mapping
- Persistence layer (serialize/deserialize state for presets)

#### Tasks

##### 1.1: Create Parameter Descriptor System
**File**: `src/plugin/param_descriptor.rs` (~300 lines)

```rust
pub struct ParamDescriptor {
    id: u32,
    name: String,
    module: String,                    // "oscillator_1", "filter_2", etc.
    value_type: ParamType,             // Float, Bool, Enum
    default: f32,
    min: f32,
    max: f32,
    step: Option<f32>,                 // for enum/discrete params
    unit: Option<String>,              // "Hz", "dB", "%", etc.
    logarithmic: bool,                 // true for freq/time params
    automation_state: AutomationState,  // none, read, readwrite
}

pub enum ParamType {
    Float { skew: Skew },
    Bool,
    Enum { variants: Vec<String> },
}

pub enum Skew {
    Linear,
    Logarithmic,
    Exponential { exponent: f32 },
}
```

**Acceptance Criteria**:
- [ ] All 45+ current parameters from `SynthParams` are mapped
- [ ] Parameter IDs are consistent (< 0xFFFFFFFF)
- [ ] Logarithmic frequency parameters properly skewed
- [ ] Automation flags correctly set (read-only vs. automatable)

##### 1.2: Create Parameter Registry
**File**: `src/plugin/param_registry.rs` (~200 lines)

A centralized registry so CLAP can query parameter info:
- Get descriptor by ID
- Get parameter value by ID
- Set parameter value by ID
- Iterate all parameters
- Convert CLAP normalized value (0.0-1.0) ↔ internal value

**Acceptance Criteria**:
- [ ] `registry.param_by_id(clap_id)` returns correct descriptor
- [ ] Normalized values (0-1) map correctly to internal ranges
- [ ] Logarithmic mapping is accurate (test against expected curves)
- [ ] Registry is Send + Sync for thread-safety

##### 1.3: Update Parameter Update Mechanism
**File**: `src/plugin/param_update.rs` (~150 lines)

Adapt the existing triple-buffer pattern to support CLAP's parameter automation:

```rust
pub struct ParamUpdateBuffer {
    // Triple buffer for lock-free updates
    param_values: Arc<TripleBuffer<HashMap<u32, f32>>>,
    // Queue for parameter automation updates from DSP→Host
    automation_queue: Arc<Mutex<VecDeque<(u32, f32)>>>,
}

impl ParamUpdateBuffer {
    pub fn update_param(&self, param_id: u32, normalized_value: f32) { ... }
    pub fn read_params(&self) -> HashMap<u32, f32> { ... }
    pub fn queue_automation(&self, param_id: u32, value: f32) { ... }
    pub fn poll_automation(&self) -> Vec<(u32, f32)> { ... }
}
```

**Acceptance Criteria**:
- [ ] Parameter updates from DAW reach audio thread lock-free
- [ ] Audio thread can report parameter changes (for modulation visualization)
- [ ] No mutex contention in audio processing
- [ ] Triple-buffer update frequency matches current (~0.7ms at 44.1kHz)

##### 1.4: Implement State Serialization
**File**: `src/plugin/state.rs` (~200 lines)

```rust
pub struct PluginState {
    version: u32,
    synth_params: SynthParams,
    preset_name: String,
    // future: UI state, unison mode, etc.
}

impl PluginState {
    pub fn to_bytes(&self) -> Vec<u8> { ... }  // for CLAP save_state
    pub fn from_bytes(data: &[u8]) -> Result<Self> { ... }
    pub fn to_json(&self) -> String { ... }    // for preset files
}
```

**Acceptance Criteria**:
- [ ] State serialization is backward compatible (old versions still load)
- [ ] All 45+ parameters serialize/deserialize correctly
- [ ] JSON preset format matches existing examples (additive_bright.json, etc.)
- [ ] Deserialize fails gracefully with helpful error messages

---

### Phase 2: CLAP Wrapper (2-3 weeks)

#### Objective
Implement the CLAP C interface, handling plugin lifecycle, audio processing, and MIDI.

#### Deliverables
- CLAP plugin descriptor and entry point
- Lifecycle management (create, destroy, activate, deactivate)
- Audio process callback with sample timing
- Parameter synchronization (DAW ↔ plugin)
- MIDI event handling
- Preset save/load hooks

#### Tasks

##### 2.1: Create CLAP Wrapper Skeleton
**File**: `src/plugin/clap_wrapper.rs` (~100 lines)

```rust
// CLAP plugin descriptor
const CLAP_PLUGIN_DESCRIPTOR: clap_plugin_descriptor_t = clap_plugin_descriptor_t {
    clap_version: CLAP_VERSION,
    id: "com.michaeldini.dsynth\0",
    name: "DSynth\0",
    version: "0.4.0\0",
    vendor: "Michael Dini\0",
    description: "High-performance polyphonic synthesizer\0",
    url: "https://github.com/michaeldini/dsynth\0",
    support_url: "https://github.com/michaeldini/dsynth/issues\0",
    manual_url: "https://github.com/michaeldini/dsynth\0",
    support_email: "michael@example.com\0",
    features: [
        CLAP_PLUGIN_FEATURE_INSTRUMENT,
        CLAP_PLUGIN_FEATURE_SYNTHESIZER,
        CLAP_PLUGIN_FEATURE_STEREO,
        null(),
    ],
};

pub extern "C" fn clap_plugin_factory(factory_id: *const c_char) -> *const c_void {
    // CLAP entry point
}
```

**Acceptance Criteria**:
- [ ] Compiles to cdylib without errors
- [ ] CLAP validator (clap-info) recognizes the plugin
- [ ] Plugin descriptor matches spec (no null pointers, valid feature flags)

##### 2.2: Implement Plugin Lifecycle
**File**: `src/plugin/clap_wrapper.rs` (~150 lines)

Implement CLAP plugin traits:
- `clap_plugin::create()` — allocate plugin state, set handlers
- `clap_plugin::destroy()` — cleanup resources
- `clap_plugin::activate()` — initialize audio thread state, sample rate
- `clap_plugin::deactivate()` — stop audio processing
- `clap_plugin::process()` — audio + event processing

```rust
struct DSynthClapPlugin {
    synth_engine: Arc<Mutex<SynthEngine>>,
    sample_rate: f32,
    param_registry: ParamRegistry,
    param_update_buffer: ParamUpdateBuffer,
    midi_handler: MidiHandler,
    host: *const clap_host_t,
}
```

**Acceptance Criteria**:
- [ ] Plugin loads in Reaper without crashing
- [ ] Sample rate correctly set on activation
- [ ] Audio output is audible (test with a simple sine wave)
- [ ] Deactivate properly stops synthesis

##### 2.3: Implement Audio Processing
**File**: `src/plugin/clap_wrapper.rs` (add to existing ~200 lines)

```rust
unsafe extern "C" fn process(
    plugin: *const clap_plugin_t,
    process: *const clap_process_t,
) -> clap_process_status_t {
    let plugin = &*(plugin as *const DSynthClapPlugin);
    let process = &*process;
    
    // 1. Read parameter updates from DAW
    plugin.param_update_buffer.read_params();
    
    // 2. Process input events (MIDI, parameter automations)
    process_events(&plugin, process.in_events);
    
    // 3. Process audio
    let mut engine = plugin.synth_engine.lock().unwrap();
    for sample_idx in 0..process.frames_count {
        let (left, right) = engine.process_sample();
        process.out_audio[0][sample_idx] = left;
        process.out_audio[1][sample_idx] = right;
    }
    
    // 4. Queue automation updates back to host
    process_out_events(&plugin, process.out_events);
    
    CLAP_PROCESS_OK
}
```

**Acceptance Criteria**:
- [ ] Audio processing runs without xruns/buffer underruns
- [ ] MIDI notes trigger synthesis correctly
- [ ] Parameter changes are heard in real-time
- [ ] CPU usage < 11% for 16 voices (benchmark)

##### 2.4: Implement MIDI Event Handler
**File**: `src/plugin/midi_handler.rs` (~100 lines)

Convert CLAP MIDI events to note_on/note_off calls:

```rust
pub struct ClapMidiHandler;

impl ClapMidiHandler {
    pub fn process_events(
        events: *const clap_event_list_t,
        synth_engine: &mut SynthEngine,
    ) {
        // Iterate CLAP events
        // Extract MIDI note_on, note_off
        // Call synth_engine.note_on() / note_off()
    }
}
```

**Acceptance Criteria**:
- [ ] note_on with velocity works
- [ ] note_off with velocity works
- [ ] Portamento works (monophonic mode)
- [ ] Polyphony voice stealing activates correctly

##### 2.5: Implement Parameter Extensions
**File**: `src/plugin/clap_wrapper.rs` (add ~150 lines)

Implement CLAP parameter extension for automation:

```rust
// Parameter info query (DAW asks: "what are your parameters?")
unsafe extern "C" fn params_info(
    plugin: *const clap_plugin_t,
    param_index: u32,
    info: *mut clap_param_info_t,
) -> bool {
    // Return descriptor for param_index
}

// Parameter value query (DAW asks: "what is parameter X currently?")
unsafe extern "C" fn params_get_value(
    plugin: *const clap_plugin_t,
    param_id: clap_id,
    out_value: *mut f64,
) -> bool {
    // Retrieve normalized value (0.0-1.0)
}

// Parameter value set (DAW sends: "set parameter X to Y")
unsafe extern "C" fn params_set_value(
    plugin: *const clap_plugin_t,
    param_id: clap_id,
    value: f64,
) -> bool {
    // Queue parameter update (thread-safe)
}
```

**Acceptance Criteria**:
- [ ] DAW can automate all parameters
- [ ] Parameter names display correctly in DAW
- [ ] Parameter ranges/min/max honored in DAW UI
- [ ] Logarithmic parameters curve correctly in DAW

##### 2.6: Implement State Extension (Save/Load)
**File**: `src/plugin/clap_wrapper.rs` (add ~100 lines)

```rust
unsafe extern "C" fn state_save(
    plugin: *const clap_plugin_t,
    stream: *const clap_ostream_t,
) -> bool {
    let plugin = &*(plugin as *const DSynthClapPlugin);
    let state = PluginState::from_engine(&plugin.synth_engine);
    let bytes = state.to_bytes();
    (*stream).write(stream, bytes.as_ptr() as _, bytes.len());
    true
}

unsafe extern "C" fn state_load(
    plugin: *const clap_plugin_t,
    stream: *const clap_istream_t,
) -> bool {
    let plugin = &*(plugin as *const DSynthClapPlugin);
    let mut bytes = vec![0u8; 4096];
    let read_count = (*stream).read(stream, bytes.as_mut_ptr() as _, bytes.len());
    bytes.truncate(read_count);
    let state = PluginState::from_bytes(&bytes)?;
    plugin.synth_engine.lock().unwrap().load_params(&state.synth_params);
    true
}
```

**Acceptance Criteria**:
- [ ] Save preset → load preset works identically
- [ ] Undo/redo in DAW works for preset changes
- [ ] Backward compatible with old presets (v0.3.0)

---

### Phase 3: GUI Migration (3-4 weeks)

#### Objective
Replace `nih_plug_iced` GUI with raw `iced` + `iced_baseview` + `iced_audio` widgets for full customization.

#### Deliverables
- `iced_baseview` window hosting inside CLAP plugin
- Custom widget library using `iced_audio` (knobs, XY pads, waveform display)
- Parameter binding system (GUI widgets ↔ CLAP parameters)
- Custom layouts (sections for different synth modules)

#### Tasks

##### 3.1: Set Up iced_baseview Foundation
**File**: `src/gui/plugin_gui/baseview_host.rs` (~200 lines)

```rust
use iced_baseview::{Settings, WindowSettings};

pub struct DSynthEditor {
    synth_params: Arc<RwLock<SynthParams>>,
    param_update_buffer: Arc<ParamUpdateBuffer>,
}

impl Editor for DSynthEditor {
    fn view(&self) -> Element<Message> {
        // Main GUI layout
    }
    
    fn update(&mut self, message: Message) {
        // Handle slider changes, knob turns, button clicks
    }
}

pub fn create_editor(
    parent_window: ParentWindowHandle,
    synth_params: Arc<RwLock<SynthParams>>,
) -> WindowHandle {
    let settings = WindowSettings {
        window: iced::window::Settings {
            size: iced::Size { width: 1200.0, height: 800.0 },
        },
        scale: 1.0,
    };
    
    iced_baseview::open::<DSynthEditor>(
        settings,
        synth_params,
        iced_baseview::IcedBaseviewConfig::default(),
    )
}
```

**Acceptance Criteria**:
- [ ] iced_baseview window opens in DAW
- [ ] Window can be resized
- [ ] Window handles focus/blur correctly
- [ ] No audio glitches when opening/closing GUI

##### 3.2: Create Knob Widget Using iced_audio
**File**: `src/gui/plugin_gui/widgets/knob.rs` (~150 lines)

Wrapper around `iced_audio::Knob` with DSynth styling:

```rust
use iced_audio::{Knob, knob};

pub struct DSynthKnob {
    parameter_id: u32,
    label: String,
    value: f32,
    default: f32,
}

impl DSynthKnob {
    pub fn new(parameter_id: u32, label: impl Into<String>) -> Self { ... }
    pub fn value(mut self, value: f32) -> Self { ... }
    pub fn on_change(self, callback: Box<dyn Fn(f32) -> Message>) -> Element { ... }
}
```

**Acceptance Criteria**:
- [ ] Knob visual renders correctly (colors, size, label)
- [ ] Drag up/down changes value smoothly
- [ ] Right-click opens text input for direct value entry
- [ ] Double-click resets to default
- [ ] Tooltip shows current value + unit

##### 3.3: Create XY Pad Widget
**File**: `src/gui/plugin_gui/widgets/xy_pad.rs` (~150 lines)

For complex parameter control (e.g., filter cutoff ↔ resonance crossfade):

```rust
pub struct XyPad {
    x_param_id: u32,
    y_param_id: u32,
    x_label: String,
    y_label: String,
    x_value: f32,
    y_value: f32,
}

impl XyPad {
    pub fn on_drag(&mut self, x: f32, y: f32) {
        // Update x_value and y_value
        // Emit message to update both parameters
    }
}
```

**Acceptance Criteria**:
- [ ] Visual renders correctly (grid lines, cursor, labels)
- [ ] X/Y range mapping is accurate (0-1 normalized)
- [ ] Drag updates both parameters simultaneously
- [ ] Left-click sets position, right-click to reset

##### 3.4: Create Waveform Display Widget
**File**: `src/gui/plugin_gui/widgets/waveform_display.rs` (~200 lines)

Visualize oscillator output / filter frequency response:

```rust
pub struct WaveformDisplay {
    waveform_data: Vec<f32>,
    sample_rate: f32,
    frequency: f32,
}

impl WaveformDisplay {
    pub fn update_waveform(&mut self, new_data: Vec<f32>) { ... }
    pub fn render(&self, frame: &mut Frame) { ... }
}
```

**Acceptance Criteria**:
- [ ] Oscillator waveform renders at 60 FPS
- [ ] Updates reflect parameter changes (waveform type, shape)
- [ ] Frequency response of filters displays correctly

##### 3.5: Build Main GUI Layout
**File**: `src/gui/plugin_gui/sections.rs` (~400 lines)

Organize GUI into logical sections (replace current `nih_plug_iced` layout):

```rust
pub fn main_layout(params: &SynthParams) -> Element<Message> {
    Column::new()
        .push(oscillator_section(&params.osc1))
        .push(filter_section(&params.filter1))
        .push(envelope_section(&params.env_amp))
        .push(effects_section(&params.distortion, &params.reverb))
        .push(control_bar())
        .into()
}

fn oscillator_section(osc: &OscillatorParams) -> Element<Message> {
    Row::new()
        .push(DSynthKnob::new(OCE_TYPE_ID, "Type").value(osc.waveform))
        .push(DSynthKnob::new(OSC1_FREQ_ID, "Tune").value(osc.freq_semitones))
        .push(DSynthKnob::new(OSC1_DETUNE_ID, "Detune").value(osc.detune))
        .push(WaveformDisplay::new(&osc))
        .into()
}
```

**Acceptance Criteria**:
- [ ] All 45+ parameters have corresponding GUI controls
- [ ] Layout is visually organized (similar grouping to current GUI)
- [ ] Sections are collapsible (optional, nice-to-have)
- [ ] Preset manager visible (load/save buttons)

##### 3.6: Parameter Binding System
**File**: `src/gui/plugin_gui/param_binding.rs` (~150 lines)

Connect GUI widget changes ↔ parameter updates ↔ CLAP host automation:

```rust
pub enum Message {
    ParamChanged(u32, f32),  // param_id, normalized_value
    PresetLoad(String),
    PresetSave(String),
}

pub fn handle_message(
    message: Message,
    param_update_buffer: &ParamUpdateBuffer,
    clap_host: &Clap_Host,
) {
    match message {
        Message::ParamChanged(param_id, value) => {
            // 1. Update internal state
            param_update_buffer.update_param(param_id, value);
            
            // 2. Notify host of automation
            if let Some(host) = clap_host.host {
                unsafe {
                    (*host).request_process(host);  // Trigger processing
                }
            }
        }
    }
}
```

**Acceptance Criteria**:
- [ ] GUI knob changes heard immediately in audio
- [ ] DAW sees parameter automation from GUI (shows in automation lane)
- [ ] Multiple parameters can change without artifacts
- [ ] Parameter smoothing prevents clicks

##### 3.7: Preset Manager in GUI
**File**: `src/gui/plugin_gui/preset_manager.rs` (~100 lines)

Load/save presets directly from GUI:

```rust
pub fn preset_manager_buttons() -> Element<Message> {
    Row::new()
        .push(Button::new("← Prev").on_press(Message::PresetPrev))
        .push(Text::new("Bright Lead"))
        .push(Button::new("Next →").on_press(Message::PresetNext))
        .push(Button::new("Save").on_press(Message::PresetSave))
        .push(Button::new("Random").on_press(Message::PresetRandom))
}
```

**Acceptance Criteria**:
- [ ] Browse presets with prev/next buttons
- [ ] Load preset updates all GUI elements
- [ ] Save preset stores current parameters
- [ ] Randomize button generates new sound

---

### Phase 3.5: Unified GUI (Bonus Phase - 2-3 weeks) ⭐ **NEW**

#### Objective
Merge plugin and standalone GUIs into a **single shared GUI codebase** that works for both targets.

#### Why This Is Feasible After CLAP Migration
1. **Both use raw `iced`**: Plugin will use `iced_baseview` (which embeds `iced`), standalone already uses `iced`
2. **Parameter binding abstraction**: Create a trait that abstracts the parameter update mechanism
   - Plugin: Updates flow through CLAP parameter API
   - Standalone: Updates flow directly to `SynthEngine` via Arc<Mutex>
3. **Layout reuse**: Current standalone GUI in `src/gui/standalone_gui/sections.rs` can be adapted
4. **Cost**: ~1 week to refactor, saves maintenance burden forever

#### Deliverables
- Shared GUI component library (`src/gui/shared/`)
- Trait-based parameter binding system (`src/gui/param_binding.rs`)
- Plugin-specific wrapper (`src/gui/plugin_gui/host.rs`)
- Standalone-specific wrapper (`src/gui/standalone_gui/host.rs`)
- Single message enum for both targets

#### Architecture
```
┌──────────────────────────────────────────────────┐
│ Shared GUI Components (src/gui/shared/)         │
│ - Sections (oscillators, filters, effects)      │
│ - Widgets (knobs, sliders, XY pads)             │
│ - Layouts (grid, column arrangement)            │
│ - Message enum (param changes, preset loads)    │
└──────────┬───────────────────────────────────────┘
           │
    ┌──────┴──────────┐
    │                 │
Plugin Host       Standalone Host
(CLAP context)    (direct SynthEngine)
    │                 │
    └────ParamBinding─┘
    
    Trait: ParamBinding
    - update_param(id, value)
    - queue_automation(id, value)
```

#### Tasks

##### 3.5.1: Create Shared GUI Component Library
**File**: `src/gui/shared/mod.rs` (~50 lines, imports only)

Move common components from standalone into shared:

```rust
// src/gui/shared/mod.rs
pub mod sections;
pub mod widgets;
pub mod messages;
pub mod layout;

pub use messages::Message;
pub use sections::{oscillator_section, filter_section, effects_section, envelope_section};
```

**Acceptance Criteria**:
- [ ] All visual components in `src/gui/shared/sections.rs`
- [ ] No target-specific code (no `nih_plug`, no standalone thread references)
- [ ] Sections render identically in plugin and standalone
- [ ] Widget state managed generically

##### 3.5.2: Create Parameter Binding Trait
**File**: `src/gui/param_binding.rs` (~150 lines)

Abstract interface for parameter updates, implemented differently per target:

```rust
pub trait ParamBinding: Send + Sync {
    fn update_param(&self, param_id: u32, normalized_value: f32);
    fn read_param(&self, param_id: u32) -> f32;
    fn queue_automation(&self, param_id: u32, value: f32);
    fn get_param_info(&self, param_id: u32) -> ParamInfo;
}

pub struct ParamInfo {
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub unit: Option<String>,
}

// Plugin implementation: uses CLAP parameter API
pub struct ClapParamBinding {
    registry: Arc<ParamRegistry>,
    update_buffer: Arc<ParamUpdateBuffer>,
    clap_host: *const clap_host_t,
}

impl ParamBinding for ClapParamBinding {
    fn update_param(&self, param_id: u32, value: f32) {
        self.update_buffer.update_param(param_id, value);
        // Notify CLAP host
    }
}

// Standalone implementation: direct engine access
pub struct StandaloneParamBinding {
    synth_engine: Arc<Mutex<SynthEngine>>,
    param_producer: Input<SynthParams>,
}

impl ParamBinding for StandaloneParamBinding {
    fn update_param(&self, param_id: u32, value: f32) {
        // Update SynthParams directly
        self.param_producer.write(/* updated params */);
    }
}
```

**Acceptance Criteria**:
- [ ] `ParamBinding` trait is Send + Sync
- [ ] Plugin implementation works through CLAP
- [ ] Standalone implementation works with arc/triple-buffer
- [ ] Both implementations tested independently

##### 3.5.3: Extract Standalone GUI Sections to Shared
**File**: `src/gui/shared/sections.rs` (~400 lines)

Move from `src/gui/standalone_gui/sections.rs` → `src/gui/shared/sections.rs`:
- `oscillator_controls()`
- `filter_controls()`
- `lfo_controls()`
- `effects_controls()`
- `envelope_section()`

Change function signatures to use shared `Message` enum and `ParamBinding` trait:

```rust
// Before (standalone only):
pub fn oscillator_controls<'a>(
    params: &'a SynthParams,
    index: usize,
    label: &'a str,
) -> Element<'a, StandaloneMessage> { ... }

// After (works for both):
pub fn oscillator_controls<'a>(
    params: &'a SynthParams,
    index: usize,
    label: &'a str,
    param_binding: &'a Arc<dyn ParamBinding>,
) -> Element<'a, SharedMessage> { ... }
```

**Acceptance Criteria**:
- [ ] All sections moved to shared without modification
- [ ] Functions accept `ParamBinding` trait object
- [ ] Visual output identical to current standalone
- [ ] Plugin and standalone render same layout

##### 3.5.4: Create Plugin GUI Host
**File**: `src/gui/plugin_gui/host.rs` (~150 lines)

Thin wrapper that integrates shared GUI with CLAP:

```rust
pub struct ClapGuiHost {
    param_binding: Arc<dyn ParamBinding>,
    synth_params: Arc<RwLock<SynthParams>>,
}

impl ClapGuiHost {
    pub fn new(
        registry: Arc<ParamRegistry>,
        update_buffer: Arc<ParamUpdateBuffer>,
        clap_host: *const clap_host_t,
    ) -> Self {
        let param_binding = Arc::new(ClapParamBinding::new(registry, update_buffer, clap_host));
        Self {
            param_binding,
            synth_params: Arc::new(RwLock::new(SynthParams::default())),
        }
    }
    
    pub fn view(&self) -> Element<SharedMessage> {
        shared::main_layout(&self.synth_params, &self.param_binding)
    }
}
```

**Acceptance Criteria**:
- [ ] Plugin GUI renders shared layout
- [ ] Parameter changes propagate through `ParamBinding` to CLAP
- [ ] No code duplication from `sections.rs`

##### 3.5.5: Update Standalone GUI to Use Shared Components
**File**: `src/gui/standalone_gui/host.rs` (NEW, ~150 lines)

Wrap standalone app to use shared GUI:

```rust
pub struct StandaloneGuiHost {
    param_binding: Arc<dyn ParamBinding>,
    synth_params: Arc<RwLock<SynthParams>>,
    param_producer: Input<SynthParams>,
}

impl StandaloneGuiHost {
    pub fn view(&self) -> Element<SharedMessage> {
        shared::main_layout(&self.synth_params, &self.param_binding)
    }
}
```

**File**: Update `src/gui/standalone_gui/app.rs` to remove inline sections, call shared functions

**Acceptance Criteria**:
- [ ] Standalone GUI looks identical to before
- [ ] All parameters still functional
- [ ] No performance regression

##### 3.5.6: Migrate Preset Manager to Shared
**File**: `src/gui/shared/preset_manager.rs` (~120 lines)

Move preset UI to shared (works for both plugin and standalone):

```rust
pub fn preset_manager_ui(
    preset_name: &str,
    param_binding: &Arc<dyn ParamBinding>,
) -> Element<SharedMessage> {
    // Load/save/random buttons
    // Preset browser
}
```

**Acceptance Criteria**:
- [ ] Plugin can browse/load/save presets
- [ ] Standalone preset manager unchanged
- [ ] Save/load use same `PluginState` serialization

#### Benefits of Unified GUI

| Aspect | Before | After |
|--------|--------|-------|
| **Code Duplication** | ~500 lines in both plugin + standalone | Shared in one place |
| **Maintenance** | Fix bugs twice | Fix once, works everywhere |
| **UI Consistency** | Plugin has basic sliders, standalone has sections | Identical UI in both |
| **Feature Additions** | Implement twice (e.g., XY pad, waveform display) | Implement once, use everywhere |
| **Testing** | Test GUI twice | Test once, covers both |

---

### Phase 4: Testing & Refinement (1-2 weeks)

#### Objective
Validate functionality across multiple DAWs and ensure production quality.

#### Tasks

##### 4.1: Cross-DAW Testing Matrix

| DAW | macOS | Linux | Windows | Status |
|-----|-------|-------|---------|--------|
| Reaper | [ ] | [ ] | [ ] | |
| Bitwig | [ ] | [ ] | [ ] | |
| Ableton | [ ] | [ ] | [ ] | |
| Studio One | [ ] | [ ] | [ ] | |
| Logic (AU) | [ ] | N/A | N/A | |

**Test Cases**:
- [ ] Plugin loads without errors
- [ ] Audio output is audible
- [ ] All parameters automatable
- [ ] MIDI input works
- [ ] Preset save/load works
- [ ] GUI opens/closes without crashes
- [ ] CPU usage < 11% for 16 voices

##### 4.2: Audio Quality Tests

**File**: `tests/clap_audio_tests.rs` (~200 lines)

Ensure audio output matches nih_plug baseline:
- [ ] Oscillator output matches between nih_plug and CLAP versions
- [ ] Filter frequency response is identical
- [ ] Voice stealing/polyphony behaves correctly
- [ ] No audio artifacts (clicks, pops) on parameter changes

##### 4.3: GUI Stress Tests

**File**: `tests/gui_stress_tests.rs` (~150 lines)

- [ ] Rapid parameter changes (automation) don't glitch
- [ ] Memory doesn't leak with repeated GUI open/close
- [ ] Large automation curves render smoothly
- [ ] No hangs when processing large audio buffers

##### 4.4: Performance Benchmarking

Run with profiler to confirm:
```bash
cargo bench --bench optimization_bench
```

- [ ] CPU usage < 11% (target maintained)
- [ ] GUI updates don't cause audio xruns
- [ ] No memory growth over time

##### 4.5: Documentation Updates

- [ ] Update `README.md` with CLAP build instructions
- [ ] Update plugin installation instructions
- [ ] Document GUI keyboard shortcuts (if any)
- [ ] Update development guide with CLAP architecture

---

## Dependency Changes

### Current (Cargo.toml)
```toml
[features]
default = ["simd", "standalone"]
simd = []
standalone = ["iced", "cpal", "midir", "rfd"]
vst = ["nih_plug", "nih_plug_iced"]

[dependencies]
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", optional = true }
nih_plug_iced = { git = "https://github.com/robbert-vdh/nih-plug.git", optional = true }
```

### New (Cargo.toml)
```toml
[features]
default = ["simd", "standalone"]
simd = []
standalone = ["iced", "cpal", "midir", "rfd"]
clap = ["iced_baseview", "iced_audio"]

[dependencies]
# Removed:
# nih_plug
# nih_plug_iced

# Added for CLAP:
clap_sys = "0.6"
clap-validator = { version = "0.3", optional = true }  # for testing
iced_baseview = { version = "0.1", optional = true }
iced_audio = { version = "0.8", optional = true }

# Keep existing:
iced = { version = "0.13", optional = true }
triple_buffer = "6.0"
crossbeam-channel = "0.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
# ... etc
```

### Build Profile
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

---

## New File Structure

```
src/
├── lib.rs                    (add CLAP export)
├── plugin.rs                 (router, minimal changes)
├── params.rs                 (unchanged)
├── preset.rs                 (unchanged)
├── audio/
│   ├── engine.rs            (unchanged)
│   ├── voice.rs             (unchanged)
│   └── ...
├── plugin/
│   ├── mod.rs               (remove nih_plug, add CLAP)
│   ├── clap_wrapper.rs      (NEW - CLAP C interface)
│   ├── param_descriptor.rs  (NEW - parameter definitions)
│   ├── param_registry.rs    (NEW - parameter lookup)
│   ├── param_update.rs      (NEW - parameter sync)
│   ├── state.rs             (NEW - save/load state)
│   ├── midi_handler.rs      (NEW - MIDI event processing)
│   ├── convert.rs           (unchanged)
│   └── params.rs            (REMOVED - use new param_descriptor.rs)
├── gui/
│   ├── mod.rs               (router)
│   ├── plugin_gui.rs        (router)
│   ├── param_binding.rs     (NEW - abstraction layer for param updates)
│   ├── shared/              (⭐ NEW - unified GUI components)
│   │   ├── mod.rs           (exports)
│   │   ├── messages.rs      (shared Message enum)
│   │   ├── sections.rs      (oscillators, filters, effects, envelope)
│   │   ├── preset_manager.rs (preset UI)
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── knob.rs      (iced_audio knob wrapper)
│   │   │   ├── xy_pad.rs    (XY pad)
│   │   │   └── waveform.rs  (waveform display)
│   │   └── layout.rs        (grid, spacing helpers)
│   ├── standalone_gui/
│   │   ├── mod.rs
│   │   ├── app.rs           (UPDATE - use shared GUI)
│   │   ├── host.rs          (NEW - StandaloneGuiHost wrapper)
│   │   ├── keyboard.rs      (unchanged)
│   │   ├── messages.rs      (REMOVE - use shared/messages)
│   │   ├── sections.rs      (REMOVE - moved to shared)
│   │   └── ...              (other standalone-specific files)
│   └── plugin_gui/
│       ├── mod.rs           (NEW - iced_baseview setup)
│       ├── baseview_host.rs (NEW - window hosting)
│       ├── host.rs          (NEW - ClapGuiHost wrapper)
│       ├── param_binding.rs (NEW - CLAP param binding impl)
│       └── widgets/         (REMOVED - use shared/widgets)
├── midi/
│   ├── handler.rs           (KEEP - reuse for CLAP)
│   └── mod.rs
├── dsp/
│   └── ...                  (unchanged)
└── main.rs                  (unchanged)

tests/
├── clap_audio_tests.rs      (NEW)
├── gui_stress_tests.rs      (NEW)
├── gui_integration_tests.rs (NEW - test shared GUI on both targets)
└── ...                      (existing tests unchanged)
```

---

## Risk & Mitigation

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| GUI less responsive than nih_plug | Medium | Medium | Profile early, optimize hot paths |
| CLAP host compatibility issues | Medium | Medium | Test on 4+ DAWs during Phase 2 |
| Audio regression vs. nih_plug | Low | High | Keep both versions during Phase 2, A/B test |
| Parameter serialization breaks presets | Low | High | Test backward compatibility in Phase 1.4 |
| MIDI timing issues | Low | Medium | Unit tests for MIDI handler in Phase 2.4 |

### Rollback Plan

If CLAP integration becomes problematic:
1. Keep `nih_plug` feature in Cargo.toml (don't delete it)
2. Tag git commit before Phase 2 start
3. Can revert to nih_plug version while fixing CLAP
4. Maintain both targets briefly if needed

---

## Success Criteria

### Must Have (MVP)
- [x] CLAP plugin loads in Reaper/Bitwig
- [x] All 45+ parameters are automatable
- [x] MIDI input produces audio output
- [x] Preset save/load works
- [x] GUI renders with knobs + sliders
- [x] CPU usage < 11% for 16 voices
- [x] No audio artifacts
- [x] Plugin and standalone have identical GUI appearance ⭐

### Should Have (Production Quality)
- [ ] XY pad widget for creative control
- [ ] Waveform display in oscillator section
- [ ] Preset browser with favorites
- [ ] Parameter undo/redo
- [ ] Works on macOS, Linux, Windows
- [ ] Comprehensive documentation
- [ ] Shared GUI code reuse measurable (target: <50 lines of GUI code duplication)

### Nice to Have (Polish)
- [ ] Custom theme/dark mode (shared across both targets)
- [ ] Keyboard shortcuts in GUI (unified between plugin + standalone)
- [ ] A/B comparison mode
- [ ] Spectrum analyzer display (shared widget)
- [ ] Mid-side stereo visualizer (shared widget)
- [ ] GUI responsive at 4K resolution

---

## Timeline

### Detailed Schedule (10-14 weeks with Unified GUI)

**Week 1-2: Phase 1.1 → 1.4 (Parameter System)**
- Monday: Create ParamDescriptor system
- Wednesday: Create ParamRegistry
- Friday: Integration testing

**Week 3-4: Phase 2.1 → 2.6 (CLAP Wrapper)**
- Monday: CLAP wrapper skeleton
- Tuesday: Plugin lifecycle
- Wednesday: Audio processing
- Thursday: MIDI + parameter extensions
- Friday: State save/load
- Weekend: Testing on Reaper

**Week 5-7: Phase 3.1 → 3.7 (Plugin GUI)**
- Week 5: iced_baseview setup + knob widget
- Week 6: XY pad + waveform display + layout
- Week 7: Parameter binding + preset manager

**Week 8-9: Phase 3.5 (Unified GUI) ⭐ NEW**
- Monday-Tuesday: Create `ParamBinding` trait + shared component library
- Wednesday-Thursday: Migrate standalone sections to shared
- Friday: Update both plugin + standalone to use shared GUI
- **Outcome**: Single GUI codebase, works for both targets

**Week 10: Phase 4 (Testing)**
- Monday-Tuesday: Cross-DAW testing matrix
- Wednesday: Audio quality tests
- Thursday: GUI stress tests
- Friday: Performance benchmarking

**Week 11-12: Refinement & Documentation**
- Fix issues from testing
- Update README/docs
- Create example projects
- Document unified GUI architecture

---

## Dependencies by Phase

| Phase | Required Crates | Version |
|-------|---|---|
| 1 | `serde`, `serde_json` | 1.0 |
| 2 | `clap_sys` | 0.6 |
| 3 | `iced_baseview`, `iced_audio` | 0.1, 0.8 |
| 4 | `criterion` (benchmarking) | 0.5 |

---

## Measurement & Checkpoints

### Phase 1 Checkpoint (End of Week 2)
- [ ] All parameters have descriptors
- [ ] State serialization round-trips correctly
- [ ] Unit tests pass for parameter registry

### Phase 2 Checkpoint (End of Week 4)
- [ ] Plugin loads in Reaper
- [ ] Simple sine wave plays at correct pitch
- [ ] Parameter automation from DAW works
- [ ] Presets save/load correctly

### Phase 3 Checkpoint (End of Week 7)
- [ ] GUI window opens without crashes
- [ ] All 45+ parameters have GUI controls
- [ ] Knob turning produces audible parameter changes
- [ ] Preset manager loads/saves presets

### Phase 4 Checkpoint (End of Week 8)
- [ ] Passes cross-DAW testing matrix
- [ ] CPU usage benchmarks meet target
- [ ] Audio quality tests pass vs. nih_plug

---

## References & Resources

### CLAP Specification
- Official Spec: https://github.com/free-audio/clap
- CLAP Best Practices: https://github.com/free-audio/clap/blob/main/include/clap/plugin.h

### iced Ecosystem
- iced: https://github.com/iced-rs/iced
- iced_baseview: https://github.com/BillyDM/iced_baseview
- iced_audio: https://github.com/BillyDM/iced_audio

### Example CLAP Plugins
- Surge Synthesizer: https://github.com/surge-synthesizer/surge (C++ reference)
- clap-host (test host): https://github.com/free-audio/clap-host

---

## Document History

| Date | Author | Version | Notes |
|------|--------|---------|-------|
| 2025-12-21 | Michael Dini | 1.0 | Initial comprehensive plan |

---

## Questions & Decisions to Make

Before starting Phase 1, decide:

1. **Preset Format**: Keep JSON or switch to binary?
   - [ ] Keep JSON (human-readable, easier debugging)
   - [ ] Switch to binary (smaller, faster load)

2. **GUI Window Size**: Fixed or resizable?
   - [ ] Fixed 1200×800
   - [ ] Resizable with minimum bounds

3. **Preset Browser**: Filesystem or embedded list?
   - [ ] Load from `~/.config/dsynth/presets/`
   - [ ] Embed presets in binary

4. **Target DAWs Priority**: Which to test first?
   - [ ] Reaper (easiest to debug)
   - [ ] Bitwig (CLAP pioneer)
   - [ ] Ableton (largest user base)

---

**Next Steps**: Review this plan with team, finalize decisions above, then begin Phase 1 (Week 1).

---

## Summary: Why Unified GUI Changes Everything

### The Problem Today
- **Plugin GUI**: Basic sliders via `nih_plug_iced`
- **Standalone GUI**: Rich controls via `iced` (oscillator tabs, waveform display, etc.)
- **Result**: Code duplication, maintenance burden, UI inconsistency

### The Solution After CLAP Migration
- **Shared GUI Library** (`src/gui/shared/`): All visual components in one place
- **Trait-based Parameter Binding**: Abstract how parameters are updated
  - Plugin: Uses CLAP parameter API
  - Standalone: Direct `SynthEngine` access
- **Result**: Single codebase, identical UI, half the maintenance work

### Real Example: Adding a New Widget

**Today (with current nih_plug + standalone architecture)**:
```
1. Create knob widget in iced_audio (done)
2. Integrate in standalone GUI (src/gui/standalone_gui/sections.rs)
3. Integrate separately in plugin GUI (src/gui/plugin_gui/mod.rs)
4. Keep synchronized as bugs are found
→ 2 implementations, 2x maintenance cost
```

**After Unified GUI (Phase 3.5)**:
```
1. Create knob widget in iced_audio (done)
2. Add to shared library (src/gui/shared/widgets/)
3. Use in src/gui/shared/sections.rs
4. Both plugin + standalone automatically get it
→ 1 implementation, 1x maintenance cost, instant feature parity
```

### Impact Over Time

| Year 1 | Code Duplication Saved | Maintenance Burden |
|--------|------------------------|--------------------|
| Phase 1-3 (CLAP GUI) | ~200 lines (widget implementations) | Medium (two GUIs) |
| Phase 3.5 (Unified) | ~500 lines (sections + layout) | Low (one GUI) |
| Year 2+ | Every new feature shared automatically | **Exponential savings** |

### Why This Wasn't Possible Before
- **nih_plug_iced is VST3-specific** — plugin was locked into that framework
- **After CLAP migration, both use raw `iced`** — unification becomes natural
- **This is a happy accident** of choosing CLAP + `iced_baseview`

---

## How to Use This Plan

1. **Print it out** or keep it in your editor
2. **Phase 1-2**: Follow the task descriptions precisely, use acceptance criteria as validation
3. **Phase 3**: Build the plugin GUI, reference this plan for widget structure
4. **Phase 3.5**: ⭐ The bonus—once plugin GUI works, refactor into shared components (2-3 weeks, saves months later)
5. **Phase 4**: Test against acceptance criteria in testing section

**Key insight**: Don't skip Phase 3.5. It's "only" 2-3 weeks of work but pays dividends forever.

---
