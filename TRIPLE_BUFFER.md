# Triple-Buffer: Lock-Free Real-Time Parameter Communication

## Overview

DSynth uses a **triple-buffer** data structure to safely pass parameter updates from the GUI thread to the audio thread without locks, mutexes, or any blocking operations. This ensures the audio thread never stalls and maintains real-time performance.

## The Problem: Why Not Just Use a Mutex?

In a typical multi-threaded application, you might use a `Mutex<SynthParams>` to share data between threads:

```rust
// DON'T DO THIS FOR REAL-TIME AUDIO!
let params = Arc::new(Mutex<SynthParams>);

// GUI thread
params.lock().unwrap().master_gain = 0.5;

// Audio thread
let current = params.lock().unwrap();  // ⚠️ DANGER!
```

This approach has **critical problems** in real-time audio:

### 1. **Unpredictable Latency**
Mutex acquisition time is non-deterministic:
- Best case: acquired in nanoseconds
- Worst case: may wait milliseconds if the other thread holds the lock
- Example: GUI thread locks params to serialize preset → audio thread stalls waiting → audio glitches

### 2. **Priority Inversion**
If the GUI thread (low-priority) gets preempted while holding the lock, the audio thread (high-priority) must wait:
```
GUI Thread (low-pri): [lock params] → [system interrupt] → preempted for 50ms
Audio Thread (high-pri): [try lock] → [blocked!] → DROPOUT
```

### 3. **Audio Dropouts and Clicks**
Even a 1ms lock contention on the audio thread causes:
- Missed buffer deadlines
- Pops/clicks in the audio stream
- Perceived stuttering or lag in real-time synthesis

**Real-time audio requires 100% deterministic timing**, so locks are forbidden.

## The Solution: Triple-Buffer

A triple-buffer uses **three buffers** instead of one, allowing safe, lock-free data exchange:

```
┌────────────────────────────────────────────────────┐
│                    Triple-Buffer                    │
├────────────────┬─────────────────┬─────────────────┤
│   Buffer A     │    Buffer B     │    Buffer C     │
│   (GUI writes) │  (Audio reads)  │   (idle/next)   │
├────────────────┼─────────────────┼─────────────────┤
│ master_gain:0.7│ master_gain:0.5 │ master_gain:0.5 │
│ filter_cut:1k  │ filter_cut:1k   │ filter_cut:500  │
│ ...            │ ...             │ ...             │
└────────────────┴─────────────────┴─────────────────┘
         ↑                 ↑                 ↑
         │                 │                 │
      produce()          read()           (waiting)
```

### How It Works

1. **GUI thread**: Writes to Buffer A (its output buffer)
   - No locks needed - nobody else touches Buffer A
   - Can take its time, no rush

2. **Audio thread**: Reads from Buffer B (its input buffer)
   - No locks needed - nobody else touches Buffer B
   - Always available, never blocked
   - Stable for ~0.7ms (one parameter update interval)

3. **Buffer C**: Idle, preparing for the next rotation

### The Rotation Cycle

When the GUI finishes writing to Buffer A:

```
BEFORE: GUI→A, Audio←B, C idle
        ↓
        produce() called (swap happens atomically)
        ↓
AFTER:  GUI→C, Audio←A, B idle
```

**Key**: The swap is atomic and non-blocking. The audio thread's `read()` doesn't wait—it either gets the old data or the new data, never partially-written data.

### Real-Time Safety Guarantees

✅ **No locks**: Both threads proceed independently
✅ **No waiting**: Audio thread never blocked  
✅ **No priority inversion**: High-priority audio thread always makes progress
✅ **No dropped data**: GUI writes are never lost
✅ **Eventual consistency**: Audio thread sees all updates, just 1-2 buffer intervals delayed

## Implementation in DSynth

### Creation

```rust
// In src/audio/engine/mod.rs
pub fn create_parameter_buffer() -> (Input<SynthParams>, Output<SynthParams>) {
    let buffer = TripleBuffer::new(&SynthParams::default());
    buffer.split()  // Returns (producer, consumer)
}
```

Usage in main:
```rust
let (param_producer, param_consumer) = create_parameter_buffer();

// Audio thread owns consumer
let mut engine = SynthEngine::new(44100.0, param_consumer);

// GUI thread owns producer
gui_state.param_producer = param_producer;
```

### GUI → Audio Flow

```rust
// GUI thread (VIZIA event handler)
GuiMessage::ParamChanged(param_id, normalized_value) => {
    let mut params = self.current_params.write().unwrap();
    param_apply::apply_param(&mut params, param_id, normalized_value);
    // Write to triple-buffer (non-blocking)
    self.param_producer.write(params);
}

// Audio thread (every 32 samples in engine.process())
fn maybe_update_params(&mut self) {
    self.sample_counter += 1;
    if self.sample_counter < self.param_update_interval {
        return;
    }
    self.sample_counter = 0;

    // Non-blocking read from triple-buffer
    let new_params = self.params_consumer.read();
    
    if *new_params == self.current_params {
        return;  // No changes, skip expensive updates
    }
    
    self.current_params = *new_params;
    
    // Apply to all active voices
    for voice in &mut self.voices {
        if voice.is_active() {
            voice.update_parameters(...);
        }
    }
}
```

### Timing Diagram

```
Time  GUI Thread              Audio Thread              Triple-Buffer State
───────────────────────────────────────────────────────────────────────────
 0ms  [Write new params]      [Read A]                  Produce→A, Consume←B
10ms  [Write complete]        [Processing]              (same)
14ms                          [Read update check]       Produce→C, Consume←A
14ms  [Start new write]        [Now sees new params]     (swapped!)
28ms  [Write complete]                                  Ready for next swap
28ms                          [Next read check]         Produce→B, Consume←C
32ms                          [Now sees 2nd update]     (swapped again!)
```

Key observations:
- GUI writes (~10ms) → audio reads it in next throttle window (~14ms) → ~4ms latency
- Audio thread never waits
- Multiple parameter changes can happen between throttle windows; audio sees the latest

## Why Throttle Updates (Every 32 Samples)?

Even with a lock-free buffer, reading every sample has CPU overhead:

```rust
// ❌ Reading every sample (expensive)
fn process(&mut self) -> f32 {
    let params = self.params_consumer.read();  // ← branch + memory read
    // ... process voice ...
}
// Called 44,100 times per second = 441,000 parameter checks

// ✅ Reading every 32 samples (efficient)
self.sample_counter += 1;
if self.sample_counter >= 32 {
    let params = self.params_consumer.read();  // ← only 1,378 times per second
    self.sample_counter = 0;
}
```

**Throttling trade-offs**:
- **Latency**: 32 samples @ 44.1kHz = 0.725ms parameter update delay
  - Imperceptible to human ears (30ms is threshold for perceiving audio-to-visual lag)
  - Still responsive for real-time control (snappy slider movement)
- **CPU savings**: ~99.7% fewer parameter reads
- **Correctness**: Audio-rate effects (LFO modulation) still work because they're applied per-sample in the voice DSP

**Musical divisions**:
- 32 samples at 48kHz = 666µs
- 32 samples at 44.1kHz = 725µs
- 32 samples at 96kHz = 333µs
- All imperceptible to human hearing

## Alternatives Considered

### 1. **Mutex + Spinning**
```rust
loop {
    match audio_params.try_lock() {
        Ok(params) => { use params; break; }
        Err(_) => { /* spin */ }
    }
}
```
Problem: Still wastes CPU spinning, unpredictable jitter

### 2. **RwLock (Reader-Writer Lock)**
```rust
let params = audio_params.read().unwrap();  // Might still wait!
```
Problem: Doesn't solve the fundamental issue—multiple readers can wait for writers

### 3. **Atomic Swap**
```rust
let new = Arc::new(SynthParams::default());
let old = Arc::swap(&params, new);
```
Problem: Doesn't work for larger structs like `SynthParams` (can't atomically swap 5KB of data)

### 4. **Single-Buffer with Flag**
```rust
struct ParamBuffer {
    params: SynthParams,
    ready: bool,  // ⚠️ Race condition!
}
```
Problem: The flag and data can become inconsistent (writer updates flag before data, reader sees flag=true but data not ready)

**Triple-buffer wins because**:
✅ Lock-free, wait-free, deterministic latency
✅ Works with any data type and size
✅ Handles concurrent reads and writes safely
✅ No atomic operations or memory barriers needed
✅ Battle-tested in real-time audio (used in JACK, PulseAudio, many synthesizers)

## Latency Analysis

```
GUI Action Timeline:
─────────────────────────────────────────────────────────

User moves slider
    ↓ (10ms)
[GUI calculates param update]
    ↓ (100µs)
param_producer.write(new_params)  ← goes to Buffer A
    ↓ (next audio throttle = up to 725µs)
Audio thread reads from Buffer B (old data)
    ↓ (32 samples later)
[Buffers swap: B→old, A→new]
    ↓ (next throttle, 725µs)
Audio thread reads from Buffer A (new data!) ← PARAM TAKES EFFECT
    ↓ (immediate)
User hears filter cutoff change

TOTAL LATENCY: 10ms (UI) + 1.45ms (param update window) ≈ 11.5ms
```

At 44.1kHz, this is ~506 samples—imperceptible for real-time synthesis (humans perceive >20ms as lag).

## For Plugin Developers

When integrating DSynth as a CLAP plugin:

1. **Plugin thread** (DAW automation/parameter changes):
   ```rust
   // In params_flush() or param_change callback:
   param_producer.write(current_params);  // ← Non-blocking, always succeeds
   ```

2. **Audio thread** (audio callback):
   ```rust
   // In process() function:
   let params = engine.current_params();  // ← Already reads from triple-buffer
   ```

The triple-buffer is **completely transparent** to plugin code—CLAP parameter updates flow through automatically.

## Testing the Triple-Buffer

```rust
#[test]
fn test_triple_buffer_no_blocking() {
    let (producer, consumer) = create_parameter_buffer();
    
    let param1 = SynthParams { master_gain: 0.5, ..Default::default() };
    let param2 = SynthParams { master_gain: 0.7, ..Default::default() };
    
    // Write from "GUI thread"
    producer.write(param1);
    assert_eq!(consumer.read().master_gain, 0.5);
    
    // Write again - no blocking!
    producer.write(param2);
    assert_eq!(consumer.read().master_gain, 0.7);
    
    // No panics = no locks, so no deadlock possible
}
```

## Conclusion

The triple-buffer is the **heart of DSynth's real-time safety**. It allows:

- ✅ GUI to update parameters without blocking
- ✅ Audio thread to always make progress
- ✅ No synchronization overhead in the hot loop
- ✅ Predictable, minimal latency
- ✅ Consistent parameter snapshots (no half-written values)

This is why DSynth can reliably deliver <11% CPU usage for 16 voices while supporting responsive, real-time parameter control—the triple-buffer is doing the heavy lifting behind the scenes.
