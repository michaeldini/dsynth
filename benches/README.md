# DSynth Benchmark Suite

Performance benchmarks organized by plugin and component level.

## Quick Reference

```bash
# Run ALL benchmarks
cargo bench

# Run specific benchmark file
cargo bench --bench <name>

# Run specific test within a file
cargo bench -- <test_name>
```

---

## Core DSP Primitives

**File:** `dsp_primitives.rs`  
**Purpose:** Individual DSP building blocks (oscillators, filters, envelopes)  
**Use Case:** Fast unit tests during development

```bash
# Run all DSP primitive benchmarks
cargo bench --bench dsp_primitives

# Run specific component
cargo bench --bench dsp_primitives -- oscillator_waveforms
cargo bench --bench dsp_primitives -- filter_types
cargo bench --bench dsp_primitives -- envelope_processing
```

---

## Main Polyphonic Synthesizer

**File:** `main_synth_perf.rs`  
**Purpose:** Full synthesizer performance and stress testing  
**Target:** <11% CPU for 16 voices @ 44.1kHz

```bash
# Run all main synth benchmarks
cargo bench --bench main_synth_perf

# Run specific benchmarks
cargo bench --bench main_synth_perf -- voice_processing
cargo bench --bench main_synth_perf -- engine_8_voices
cargo bench --bench main_synth_perf -- engine_16_voices
cargo bench --bench main_synth_perf -- parameter_automation
cargo bench --bench main_synth_perf -- stress_test
cargo bench --bench main_synth_perf -- block_scaling
```

---

## Kick Drum Synthesizer

**File:** `kick_synth_perf.rs`  
**Purpose:** Monophonic kick drum performance  
**Target:** <5% CPU @ 44.1kHz  
**Requires:** `--features kick-clap`

```bash
# Run all kick synth benchmarks
cargo bench --features kick-clap --bench kick_synth_perf

# Run specific benchmarks
cargo bench --features kick-clap --bench kick_synth_perf -- kick_clean
cargo bench --features kick-clap --bench kick_synth_perf -- kick_stress_test
cargo bench --features kick-clap --bench kick_synth_perf -- kick_key_tracking
cargo bench --features kick-clap --bench kick_synth_perf -- kick_distortion_types
cargo bench --features kick-clap --bench kick_synth_perf -- kick_block_processing
```

---

## Voice Enhancement Plugin

### Performance Benchmarks

**File:** `voice_plugin_perf.rs`  
**Purpose:** Voice enhancer signal chain performance  
**Target:** <15% CPU @ 44.1kHz  
**Requires:** `--features voice-clap`

```bash
# Run all voice plugin performance benchmarks
cargo bench --features voice-clap --bench voice_plugin_perf

# Run specific benchmarks
cargo bench --features voice-clap --bench voice_plugin_perf -- saturator_per_character
cargo bench --features voice-clap --bench voice_plugin_perf -- three_stage_cascade
cargo bench --features voice-clap --bench voice_plugin_perf -- signal_analysis
cargo bench --features voice-clap --bench voice_plugin_perf -- voice_engine
cargo bench --features voice-clap --bench voice_plugin_perf -- drive_levels
```

### Latency Validation

**File:** `voice_plugin_latency.rs`  
**Purpose:** Prove zero-latency operation with impulse response tests  
**Requires:** `--features voice-clap`

```bash
# Run all voice plugin latency benchmarks
cargo bench --features voice-clap --bench voice_plugin_latency

# Run specific latency tests
cargo bench --features voice-clap --bench voice_plugin_latency -- latency_query
cargo bench --features voice-clap --bench voice_plugin_latency -- impulse_response
cargo bench --features voice-clap --bench voice_plugin_latency -- realtime_throughput
cargo bench --features voice-clap --bench voice_plugin_latency -- per_sample_latency
cargo bench --features voice-clap --bench voice_plugin_latency -- latency_drive_sweep
cargo bench --features voice-clap --bench voice_plugin_latency -- worst_case_latency
```

---

## Advanced Usage

### Baseline Comparison

Save a baseline for performance regression detection:

```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main

# Compare specific benchmark
cargo bench --bench main_synth_perf -- --baseline main
```

### Quick Runs

For faster iteration during development:

```bash
# Quick run (fewer samples, less accurate)
cargo bench -- --quick

# Specific test with quick mode
cargo bench --bench dsp_primitives -- oscillator --quick
```

### HTML Reports

Criterion generates detailed HTML reports at:
```
target/criterion/report/index.html
```

Open in browser:
```bash
open target/criterion/report/index.html  # macOS
```

---

## Benchmark Organization

```
benches/
├── dsp_primitives.rs         # Core DSP components (no plugin logic)
├── main_synth_perf.rs         # Main synthesizer performance
├── kick_synth_perf.rs         # Kick drum synthesizer
├── voice_plugin_perf.rs       # Voice enhancer performance
└── voice_plugin_latency.rs    # Voice enhancer latency proof
```

### Adding New Benchmarks

**For new plugins:** Follow the pattern `{plugin}_perf.rs` + `{plugin}_latency.rs` (optional)

**For new components:** Add to `dsp_primitives.rs` if they're reusable DSP units

---

## Performance Targets

| Plugin | Target | Polyphony |
|--------|--------|-----------|
| Main Synth | <11% CPU | 16 voices |
| Kick Synth | <5% CPU | Monophonic |
| Voice Plugin | <15% CPU | Real-time processing |

All targets measured at 44.1kHz sample rate on Apple Silicon.
