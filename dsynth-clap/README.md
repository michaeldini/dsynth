# DSynth CLAP Framework

A reusable Rust library for building CLAP audio plugins with minimal boilerplate.

## Design Goals

1. **Trait-based abstractions** - Clean interfaces over CLAP C API
2. **Type safety** - Leverage Rust's type system for correctness
3. **Minimal boilerplate** - Generate repetitive CLAP glue code
4. **Flexibility** - Support different plugin types (instruments, effects, etc.)
5. **Zero-cost** - Abstractions compile away to efficient code

## Architecture

```
┌─────────────────────────────────────┐
│      Your Plugin Code               │
│  (Implements ClapPlugin trait)      │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      dsynth-clap Library            │
│  • Trait definitions                │
│  • CLAP extension handlers          │
│  • Entry point generation           │
│  • Parameter system                 │
│  • State management                 │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│          clap-sys                   │
│  (Raw CLAP C bindings)              │
└─────────────────────────────────────┘
```

## Core Traits

### `ClapPlugin`
The main plugin trait. Implement this to define your plugin:

```rust
pub trait ClapPlugin: Send + Sync + 'static {
    type Processor: ClapProcessor;
    type Params: PluginParams;
    
    fn descriptor(&self) -> PluginDescriptor;
    fn create_processor(&mut self, sample_rate: f32) -> Self::Processor;
    fn params(&self) -> &Self::Params;
    fn params_mut(&mut self) -> &mut Self::Params;
}
```

### `ClapProcessor`
The audio processing trait (called from real-time thread):

```rust
pub trait ClapProcessor: Send + 'static {
    fn process(&mut self, audio: &mut AudioBuffers, events: &Events) -> ProcessStatus;
    fn activate(&mut self, sample_rate: f32);
    fn reset(&mut self);
    fn latency(&self) -> u32 { 0 }
}
```

### `PluginParams`
Parameter management trait:

```rust
pub trait PluginParams: Send + Sync + 'static {
    fn descriptors(&self) -> Vec<ParamDescriptor>;
    fn get_param(&self, id: ParamId) -> Option<f32>;
    fn set_param(&mut self, id: ParamId, value: f32);
}
```

## Usage Example

See [examples/kick_minimal.rs](examples/kick_minimal.rs) for a complete minimal kick drum plugin (~150 lines vs 1000+ in raw CLAP).

## Comparison

### Before (raw CLAP):
- 1000+ lines per plugin
- Manual CLAP struct initialization
- Unsafe C FFI everywhere
- Duplicate extension implementations
- Complex lifetime management

### After (dsynth-clap):
- ~150 lines per plugin
- Safe Rust traits
- Generated CLAP glue code
- Shared extension implementations
- Clear separation of concerns

## Status

**Phase 1 - Core Framework** (In Progress)
- [x] Trait definitions
- [x] Plugin descriptor
- [x] Basic extension stubs
- [ ] Complete extension implementations
- [ ] Entry point generation
- [ ] Parameter automation
- [ ] State save/load

**Phase 2 - Migration**
- [ ] Port kick plugin
- [ ] Port voice plugin
- [ ] Port main synth

**Phase 3 - Polish**
- [ ] GUI integration helpers
- [ ] Documentation
- [ ] Examples
- [ ] Testing utilities

## License

MIT OR Apache-2.0
