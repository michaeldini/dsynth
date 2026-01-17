# Migration Guide: From Raw CLAP to dsynth-clap

## Overview

This guide shows how to migrate existing DSynth CLAP plugins to use the new `dsynth-clap` framework.

## Benefits

| Aspect | Before (Raw CLAP) | After (dsynth-clap) |
|--------|-------------------|---------------------|
| **Lines of code** | ~1000+ per plugin | ~150-200 per plugin |
| **Boilerplate** | Manual struct init, FFI everywhere | One trait impl + entry macro |
| **Safety** | Unsafe blocks throughout | Unsafe only in framework |
| **Reusability** | Copy-paste between plugins | Shared library code |
| **Maintainability** | Bug fixes need N changes | Bug fixes in one place |

## Migration Steps

### 1. Plugin Struct (Was: Complex state management)

**Before** (kick_plugin.rs):
```rust
pub struct KickClapPlugin {
    pub plugin: clap_plugin,
    _host: *const clap_host,
    pub processor: Option<KickClapProcessor>,
    params_ext: clap_sys::ext::params::clap_plugin_params,
    state_ext: clap_sys::ext::state::clap_plugin_state,
    audio_ports_ext: clap_plugin_audio_ports,
    note_ports_ext: clap_plugin_note_ports,
    gui_ext: clap_plugin_gui,
    gui_window: Option<Box<dyn std::any::Any>>,
    kick_params: Arc<Mutex<KickParams>>,
    gui_size: (u32, u32),
    gui_parent: Option<RawWindowHandle>,
}
```

**After** (dsynth-clap):
```rust
pub struct KickPlugin {
    params: KickParams,  // Just your params!
}

impl ClapPlugin for KickPlugin {
    type Processor = KickProcessor;
    type Params = KickParams;
    
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor::instrument("DSynth Kick", "com.dsynth.kick")
            .version("0.3.0")
            .with_features(&["synthesizer", "drum"])
    }
    
    fn create_processor(&mut self, sample_rate: f32) -> Self::Processor {
        KickProcessor::new(sample_rate, &self.params)
    }
    
    fn params(&self) -> &Self::Params { &self.params }
    fn params_mut(&mut self) -> &mut Self::Params { &mut self.params }
}
```

### 2. Parameters (Was: Manual registry with IDs)

**Before**:
```rust
pub const PARAM_KICK_OSC1_PITCH_START: ParamId = 0x0200_0001;
pub const PARAM_KICK_OSC1_PITCH_END: ParamId = 0x0200_0002;
// ... 20 more lines ...

pub fn get_kick_registry() -> &'static KickParamRegistry { /* ... */ }

impl KickParamRegistry {
    pub fn apply_param(&self, params: &mut KickParams, id: ParamId, value: f64) {
        match id {
            PARAM_KICK_OSC1_PITCH_START => params.osc1_pitch_start = /* denormalize */,
            // ... 20 more cases ...
        }
    }
}
```

**After**:
```rust
impl PluginParams for KickParams {
    fn descriptors(&self) -> Vec<ParamDescriptor> {
        vec![
            ParamDescriptor::float(0, "Start Pitch", "Osc1", 20.0, 200.0, 150.0).unit("Hz"),
            ParamDescriptor::float(1, "End Pitch", "Osc1", 20.0, 200.0, 55.0).unit("Hz"),
            // ...
        ]
    }
    
    fn get_param(&self, id: ParamId) -> Option<f32> {
        match id {
            0 => Some(self.osc1_pitch_start),
            1 => Some(self.osc1_pitch_end),
            _ => None,
        }
    }
    
    fn set_param(&mut self, id: ParamId, value: f32) {
        match id {
            0 => self.osc1_pitch_start = value,
            1 => self.osc1_pitch_end = value,
            _ => {}
        }
    }
}
```

### 3. Audio Processing (Was: Unsafe CLAP process callback)

**Before**:
```rust
pub unsafe fn process(&mut self, process: *const clap_process) -> clap_process_status {
    let process = &*process;
    
    if process.audio_outputs.is_null() { return CLAP_PROCESS_ERROR; }
    
    let audio_output = &*process.audio_outputs;
    let frame_count = process.frames_count as usize;
    
    let output_left = std::slice::from_raw_parts_mut(
        *audio_output.data32.offset(0) as *mut f32,
        frame_count
    );
    let output_right = std::slice::from_raw_parts_mut(
        *audio_output.data32.offset(1) as *mut f32,
        frame_count
    );
    
    // Process events...
    // Generate audio...
    
    CLAP_PROCESS_CONTINUE
}
```

**After**:
```rust
impl ClapProcessor for KickProcessor {
    fn process(&mut self, audio: &mut AudioBuffers, events: &Events) -> ProcessStatus {
        // Safe audio buffer access!
        for i in 0..audio.frame_count {
            let sample = self.generate_sample();
            
            for output in audio.outputs.iter_mut() {
                output[i] = sample;
            }
        }
        
        ProcessStatus::Continue
    }
    
    fn activate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}
```

### 4. Entry Point (Was: 200+ lines of factory boilerplate)

**Before**:
```rust
#[no_mangle]
pub static clap_entry: clap_sys::entry::clap_plugin_entry = /* ... */;

unsafe extern "C" fn entry_init(_plugin_path: *const c_char) -> bool { /* ... */ }
unsafe extern "C" fn entry_deinit() { /* ... */ }
unsafe extern "C" fn entry_get_factory(factory_id: *const c_char) -> *const c_void { /* ... */ }

static KICK_FACTORY: clap_plugin_factory = /* ... */;
unsafe extern "C" fn factory_get_plugin_count(_factory: *const clap_plugin_factory) -> u32 { /* ... */ }
unsafe extern "C" fn factory_get_plugin_descriptor(/* ... */) -> *const clap_plugin_descriptor { /* ... */ }
unsafe extern "C" fn factory_create_plugin(/* ... */) -> *const clap_plugin { /* ... */ }
```

**After**:
```rust
// ONE LINE:
generate_clap_entry!(KickPlugin);
```

## Code Reduction

| Plugin | Before | After | Reduction |
|--------|--------|-------|-----------|
| Main Synth | ~1200 lines | ~200 lines | **83%** |
| Kick Drum | ~1100 lines | ~150 lines | **86%** |
| Voice Enhancer | ~650 lines | ~120 lines | **82%** |

## Next Steps

1. **Start with kick plugin** (simplest)
2. **Then voice plugin** (audio effect)
3. **Finally main synth** (most complex)

Each migration validates the framework and reveals missing features.

## Path A Strategy (Recommended)

**Goal:** migrate DSynth, DSynth Kick, and DSynth Voice to `dsynth-clap`.

This keeps all CLAP ABI/FFI unsafety centralized and ensures any future host-compat fixes happen in one place.

## What to do with SimpleSynth

Keep SimpleSynth as a **host-interop canary** living under `dsynth-clap/examples/simple_synth/`.

- It should remain minimal and stable.
- It should be the first thing you build/load when touching CLAP ABI, extensions, or entry symbols.
- Packaging policy: keep producing the **flat `.clap` Mach-O file** (not a bundle) if that’s what your Reaper setup reliably loads.

## Migration Checklist

This is the “Definition of Done” checklist for each plugin migration.

### 0) Pre-flight (one-time)

- Confirm `dsynth-clap` loads in Reaper via SimpleSynth.
- Confirm entry symbol is exported as **data**: `#[no_mangle] pub static clap_entry: clap_plugin_entry = ...`.
- Confirm all extension vtables returned from `get_extension()` are **stable statics** (no stack temporaries).

### 1) Minimal migrated plugin loads and plays

- Implement a thin plugin adapter crate/module using `dsynth-clap` (Kick first).
- Audio ports:
    - Instruments: 0 inputs / 2 outputs (stereo), `port_type` points to a static C string.
    - Effects: 2 inputs / 2 outputs (stereo).
- Note ports:
    - Instruments expose 1 note input.
- Processing:
    - No allocations in audio thread.
    - Event parsing: note on/off + param events.

### 2) Parameters (host automation works)

- Provide a full parameter descriptor list (names, modules, ranges, units).
- Ensure CLAP normalized domain is correct:
    - CLAP-side is 0..1.
    - Convert to/from engine “plain” values consistently.
- Ensure enum parameters are stable and round-trip (index-based).

### 3) State save/load (DAW preset/project reload)

- `state_save()` writes all parameters (and any extra state you choose) deterministically.
- `state_load()` updates shared params only; never assumes a processor exists during load.
- Validate:
    - Save a DAW project, reload, no crash.
    - Parameter values restored.

### 4) GUI (optional per milestone)

- Minimum: plugin can run headless.
- If adding GUI early:
    - Use the existing VIZIA code where possible.
    - GUI→audio thread updates must stay lock-free (triple-buffer pattern).

### 5) Packaging + install

- Produce exactly one artifact per plugin:
    - `DSynth.clap`, `DSynthKick.clap`, `DSynthVoice.clap` (flat Mach-O files).
- Verify in Reaper:
    - Scan finds it.
    - Instantiate works.
    - Audio/MIDI behave.

## Suggested Order

1. **Kick** (simplest instrument)
2. **Voice** (effect; validates audio-input handling + latency reporting if needed)
3. **Main DSynth** (most complex)

## Framework Status (Jan 2026)

`dsynth-clap` is now **host-functional** (SimpleSynth loads in Reaper).

Key fixes already implemented in the framework:

- Correct `clap_entry` export (static data symbol).
- Extension lifetime safety: returned vtables are stable statics.
- `audio_ports` correctness: `port_type` is a pointer to a static C string (not a writable buffer).

Remaining work for full Path A completion is primarily runtime validation in hosts (REAPER) for each migrated plugin.
