# DSynth-CLAP Development Roadmap

## Current Status: Foundation Complete ✅

The core trait-based architecture is designed and compiles. We have:

- ✅ Workspace structure (`dsynth` + `dsynth-clap`)
- ✅ Core traits (`ClapPlugin`, `ClapProcessor`, `PluginParams`)
- ✅ Plugin descriptor system
- ✅ Extension stubs (audio ports, note ports, params, state)
- ✅ Example showing the target API

## Phase 1: Complete Framework Implementation ✅

### Step 1.1: Plugin Instance Management ✅
**Goal**: Create the glue between traits and CLAP C API

**Tasks**:
- [x] Create `PluginInstance<P: ClapPlugin>` wrapper
- [x] Implement lifecycle callbacks (init, destroy, activate, deactivate)
- [x] Handle plugin_data pointer management safely
- [x] Bridge processor creation

**Files**: `dsynth-clap/src/instance.rs`

### Step 1.2: Complete Audio Ports Extension ✅
**Status**: Fully implemented  
**Completed**:
- [x] Read config from plugin descriptor
- [x] Support custom port configurations (Instrument/Effect/Custom)
- [x] Handle in-place processing pairs
- [x] Stereo port configuration

**Files**: `dsynth-clap/src/extensions/audio_ports.rs`

### Step 1.3: Complete Parameters Extension ✅
**Status**: Fully implemented  
**Completed**:
- [x] Iterate plugin parameter descriptors
- [x] Fill `clap_param_info` from `ParamDescriptor`
- [x] Implement get/set value with normalization
- [x] Handle flush events (automation)
- [x] Convert values to/from text

**Files**: `dsynth-clap/src/extensions/params.rs`

### Step 1.4: Complete State Extension ✅
**Status**: Fully implemented  
**Completed**:
- [x] Call plugin's `save_state()` / `load_state()`
- [x] Serialize/deserialize to CLAP streams (JSON)
- [x] Handle versioning for compatibility

**Files**: `dsynth-clap/src/extensions/state.rs`

### Step 1.5: Process Callback ✅
**Goal**: Bridge safe `ClapProcessor::process()` to unsafe CLAP callback

**Tasks**:
- [x] Extract audio buffers safely from `clap_process`
- [x] Create `AudioBuffers` wrapper with slices
- [x] Handle MIDI/parameter events
- [x] Call processor with safe API
- [x] Handle latency reporting

**Files**: `dsynth-clap/src/instance.rs` (process method), `dsynth-clap/src/processor.rs`

### Step 1.6: Entry Point Generator ⏳ IN PROGRESS
**Current**: Macro skeleton  
**Needs**:
- [ ] Generate factory implementation
- [ ] Create plugin descriptor from trait
- [ ] Instantiate plugin on create_plugin
- [ ] Handle multiple plugins (future)

**Files**: `dsynth-clap/src/entry.rs`

**Status**: Last piece before migration!

## Phase 2: Migration & Validation

### Step 2.1: Kick Plugin Migration
**Goal**: Prove the framework with simplest plugin

**Tasks**:
- [ ] Create `src/plugin/dsynth_clap/kick.rs`
- [ ] Implement `ClapPlugin` for kick
- [ ] Implement `PluginParams` for `KickParams`
- [ ] Adapt `KickEngine` to `ClapProcessor` trait
- [ ] Generate entry point
- [ ] Test in DAW (Bitwig/Reaper)

**Success Criteria**:
- ✅ Plugin loads in DAW
- ✅ Audio output works
- ✅ MIDI input triggers notes
- ✅ Parameters are automatable
- ✅ Presets save/load

**Estimate**: 1-2 days

### Step 2.2: Voice Plugin Migration
**Goal**: Validate audio effect configuration

**Tasks**:
- [ ] Implement for voice enhancer
- [ ] Test stereo input handling
- [ ] Validate latency reporting works

**Estimate**: 1 day

### Step 2.3: Main Synth Migration
**Goal**: Handle most complex case

**Tasks**:
- [ ] Migrate polyphonic synthesizer
- [ ] Test with complex parameter set
- [ ] Validate GUI integration

**Estimate**: 2 days

## Phase 3: Polish & Features

### Step 3.1: GUI Integration
**Current**: Not started  
**Needs**:
- [ ] GUI extension helpers
- [ ] VIZIA integration utilities
- [ ] Parameter ↔ GUI binding helpers

**Estimate**: 2-3 days

### Step 3.2: Testing & Documentation
- [ ] Add unit tests for framework
- [ ] Add integration tests
- [ ] Document all public APIs
- [ ] Create tutorial
- [ ] Add more examples

**Estimate**: 2-3 days

### Step 3.3: Advanced Features
- [ ] Multiple plugins per library
- [ ] Audio effect categories
- [ ] MIDI output support
- [ ] Sample-accurate automation
- [ ] Modulation system helpers

**Estimate**: As needed

## Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| **Foundation** | 1 day | ✅ Complete |✅ Complete (Steps 1.1-1.5) |
| **Entry Point Generator** | 1-2 hours | ⏳ In Progress (Step 1.6) |
| **Migration & Validation** | 4-5 days | ⬜ Next |
| **Polish & Features** | 4-6 days | ⬜ Future |

**Progress**: Framework core complete! Entry point generator is the final piece before kick plugin migration.
**Total**: ~2 weeks of focused development

## Decision Points

### Now: Start Phase 1?
- **Option A**: Complete framework implementation now (recommended)
- **Option B**: Prototype one migration first to validate design
- **Option C**: Pause and gather feedback on architecture

### After Phase 2.1: Continue or pivot?
If kick migration reveals issues, we can:
- Refine the traits
- Adjust the architecture
- Add missing abstractions

The framework is designed to be flexible and evolvable.

## Success Metrics

- ✅ Plugin instance management complete
- ✅ Audio buffer wrapper safe and ergonomic
- ✅ All 4 CLAP extensions implemented
- ⏳ Entry point generator (macro)
- ✅ Compiles in workspace
- ⏳ All plugins load in DAWs
- ⏳ Audio/MIDI processing works
- ⏳ Parameters automatable
- ⏳ State save/load works
- ⏳ **Code reduced by >80%**
- ⏳ **New plugins <200 lines**
- ⏳ **Bug fixes apply to all plugins**

## Notes

The current design is **sound**. The traits provide the right level of abstraction:
- Low enough to avoid performance overhead
- High enough to eliminate boilerplate
- Flexible enough for different plugin types

The hard part (architecture) is done. Now it's implementation work.
