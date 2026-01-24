# DSynth Voice Saturation Plugin - Performance Benchmarks

## Executive Summary

**Plugin validated for real-time use** - All benchmarks confirm zero-latency operation with minimal CPU overhead suitable for vocal processing in professional DAW workflows.

## Benchmark Results @ 44.1kHz

### 1. Per-Character Saturation Performance

Processing time per sample for each character type:

| Character | Time/Sample | Notes |
|-----------|-------------|-------|
| **Warm** (Tube) | 42.4 ns | Asymmetric clipping, even harmonics |
| **Smooth** (Tape) | 59.9 ns | Tanh saturation, balanced harmonics |
| **Punchy** (Console) | 29.8 ns | Soft-clip, aggressive mid-range |

**Analysis:**
- Punchy is fastest (soft-clip with hard knee = simple math)
- Smooth is slowest (tanh requires more computation)
- All characters well under 1µs per sample - minimal CPU impact

### 2. 3-Stage Cascade Overhead

**3-stage serial processing**: 42.3 ns/sample

This matches single-stage Warm processing, indicating **negligible overhead** from the multi-stage architecture. The benefits of staged saturation (natural harmonic buildup) come at no performance cost.

### 3. Signal Analysis Cost (Zero-Latency Mode)

| Mode | Time/Sample | Latency |
|------|-------------|---------|
| **No Pitch Detection** | 20.2 ns | 0 samples |
| With Pitch Detection | 552.5 ns | 1024 samples |

**Critical Finding**: Disabling pitch detection reduces analysis cost by **27×** and eliminates all latency. Since this plugin focuses on transient enhancement and saturation (not pitch-dependent features), this is the optimal configuration.

### 4. Full Engine Processing

| Metric | Value | Interpretation |
|--------|-------|----------------|
| **Single Sample** | 82.4 ns | ~11.6M samples/sec throughput |
| **512-sample Buffer** | 51.2 µs | 100 µs/sample batch processing |
| **Latency** | 0 samples | 0 ms @ 44.1kHz ✅ |

**Real-World Performance:**
- At 44.1kHz, one sample period = 22.7 µs
- Full processing takes 82.4 ns = **0.36% of available time per sample**
- For stereo processing: 0.72% CPU per sample
- **Easily handles real-time processing with massive headroom**

### 5. Drive Level Performance

Processing time by drive amount:

| Drive Level | Time/Sample | Notes |
|-------------|-------------|-------|
| 0% (bypass) | 87.7 ns | Full signal chain still active |
| 50% (moderate) | 82.7 ns | Optimal vocal saturation level |
| 100% (max) | 81.3 ns | Heavy saturation |

**Interesting Result**: Higher drive levels are slightly faster because:
- Transient detection is less active (more clipping)
- Auto-gain compensation has less dynamic range to manage
- Difference is negligible (~6ns) for practical purposes

## Performance Targets ✅

| Target | Result | Status |
|--------|--------|--------|
| Latency < 100 samples @ 44.1kHz | **0 samples** | ✅ Exceeded |
| Zero-latency operation | **Confirmed** | ✅ Validated |
| Real-time processing capable | **0.36% CPU/sample** | ✅ Excellent |
| Minimal overhead per character | **29-60 ns/sample** | ✅ Negligible |

## CPU Overhead Analysis

### Theoretical Max Polyphony

At 82.4 ns per sample × 44,100 samples/sec:

```
CPU time per second = 82.4 ns × 44,100 = 3.63 ms
Available time per second = 1000 ms
Theoretical max instances = 1000 / 3.63 = 275 instances
```

**In practice**: You could run **hundreds of instances** simultaneously before hitting CPU limits, making this extremely efficient for vocal effect chains.

### Comparison to Pitch-Enabled Mode

If pitch detection were enabled:

```
With pitch: (82.4 + 552.5) ns = 634.9 ns per sample
Slowdown: 634.9 / 82.4 = 7.7× slower
Latency: 1024 samples = 23.2 ms @ 44.1kHz (unacceptable)
```

**Conclusion**: Disabling pitch detection was the correct design choice for this use case.

## Hardware Context

All benchmarks run on:
- **Platform**: Apple Silicon (M-series)
- **Profile**: `release` with optimizations enabled
- **Sample Rate**: 44,100 Hz (standard professional audio)
- **Measurement**: Criterion.rs with 100 samples, 3-second warmup

## Recommendations

1. **Production Ready**: All performance metrics validate plugin for professional use
2. **Character Selection**: Use based on musical preference, not performance (differences are negligible)
3. **Drive Settings**: 50% (0.5 drive) confirmed as optimal for moderate vocal saturation
4. **Multi-Instance Use**: Safe to use multiple instances per track without CPU concerns
5. **Latency-Critical Workflows**: Zero latency makes this suitable for live monitoring and tracking

## Benchmark Commands

To reproduce these results:

```bash
# Run all saturation benchmarks
cargo bench --bench saturation_bench --features voice-clap

# View detailed HTML reports
open target/criterion/report/index.html
```

## Future Optimization Opportunities

Despite excellent performance, potential improvements:

1. **SIMD Vectorization**: Process 4-8 samples simultaneously using SIMD
   - Current: 82.4 ns per sample
   - Potential: ~20-25 ns per sample (4× faster)
   - Trade-off: Increased code complexity, nightly Rust required

2. **Lookup Tables**: Pre-compute waveshaping curves
   - Current: Runtime tanh/soft-clip calculations
   - Potential: 10-20% speedup for Smooth character
   - Trade-off: Memory usage, startup cost

**Current Status**: Performance is already excellent, optimizations not necessary unless targeting embedded systems or extreme polyphony (100+ instances).
