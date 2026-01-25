# Adaptive Saturator Enhancement Proposals

## Current Performance Baseline

**Benchmark Results:**
- Warm saturation: 44.6 ns/sample
- Smooth saturation: 56.5 ns/sample
- Punchy saturation: 43.1 ns/sample
- Full engine: 82.4 ns/sample
- Signal analysis (no pitch): 20.2 ns/sample

**Available Headroom:**
- One sample period @ 44.1kHz: 22,700 ns
- Current usage: 82.4 ns (0.36% of time)
- **Available for enhancements: ~22,600 ns (99.6% headroom!)**

We can add **MASSIVE** complexity while maintaining zero latency and real-time performance.

## Available Signal Analysis Data

From `SignalAnalysis` struct:
- ✅ `rms_level` - Average signal level
- ✅ `peak_level` - Peak amplitude
- ✅ `is_transient` - Transient detection flag
- ✅ `transient_strength` (0.0-1.0) - **Already using for drive boost!**
- ✅ `zcr_hz` - Zero crossing rate (indicates frequency content)
- ✅ `signal_type` - Tonal/Noisy/Transient/Silence classification
- ✅ `is_voiced` / `is_unvoiced` - Vocal detection
- ✅ `has_sibilance` - High-frequency "s" sound detection
- ✅ `sibilance_strength` (0.0-1.0) - How strong the sibilance is

## Enhancement Proposals

### Priority 1: Sibilance-Aware Saturation (HIGH VALUE)

**Problem**: Sibilant sounds (s, sh, ch) at 4-10kHz become harsh when saturated
**Solution**: Reduce saturation intensity during sibilance

**Implementation:**
```rust
// In process() method, after transient_mult calculation:
let sibilance_reduction = if analysis.has_sibilance {
    1.0 - (analysis.sibilance_strength * 0.7)  // Reduce up to 70%
} else {
    1.0
};
let adaptive_drive = (drive_internal * transient_mult * sibilance_reduction).min(1.0);
```

**Benefits:**
- ✅ Prevents harsh "s" sounds
- ✅ Preserves vocal intelligibility
- ✅ More transparent on sibilant-heavy vocals
- ✅ **Cost: ~2 multiplications = <1 ns/sample**

**Recommended:** ⭐⭐⭐⭐⭐ (Essential for vocal transparency)

---

### Priority 2: Dynamic Stage Count (EXPERIMENTAL)

**Concept**: Use more stages for smooth content, fewer for transients

**Implementation:**
```rust
// Detect content type and adjust stages
let num_stages = if analysis.is_transient {
    2  // Fast path: preserve transient attack
} else if analysis.signal_type == SignalType::Tonal {
    5  // Smooth content: more harmonic complexity
} else {
    3  // Default
};

// Process variable number of stages
for stage in 0..num_stages {
    let stage_drive = adaptive_drive / num_stages as f32;
    left_out = self.saturate_stage(left_out, stage_drive, character, stage);
}
```

**Benefits:**
- ✅ Preserves transient attack (less processing during peaks)
- ✅ Adds harmonic complexity to sustained notes
- ✅ Adaptive to content

**Drawbacks:**
- ❌ Non-deterministic (stage count varies)
- ❌ Requires pre-allocated stage buffers (currently fixed at 3)
- ❌ More complex

**Cost:** ~50-100 ns/sample (still well within budget)

**Recommended:** ⭐⭐⭐ (Interesting but complex, test carefully)

---

### Priority 3: Voiced/Unvoiced Processing (VOCAL OPTIMIZATION)

**Concept**: Different saturation strategies for voiced (sustained vowels) vs unvoiced (consonants) content

**Implementation:**
```rust
// Adjust drive based on voiced/unvoiced
let voice_factor = if analysis.is_voiced {
    1.0  // Full saturation on vowels (adds warmth)
} else if analysis.is_unvoiced {
    0.6  // Reduce saturation on consonants (preserve clarity)
} else {
    0.8  // Mixed content
};
let adaptive_drive = (drive_internal * transient_mult * voice_factor).min(1.0);
```

**Benefits:**
- ✅ Optimized for vocal characteristics
- ✅ Preserves consonant clarity
- ✅ Adds warmth to sustained vowels

**Cost:** <1 ns/sample (just a multiplication)

**Recommended:** ⭐⭐⭐⭐ (Great for vocal-focused plugin)

---

### Priority 4: Level-Dependent Saturation Curves (DYNAMIC RANGE)

**Concept**: Gentler saturation for quiet signals, more aggressive for loud signals

**Implementation:**
```rust
// In saturate_stage(), adjust gain based on input level
fn saturate_stage(&mut self, input: f32, drive: f32, character: SaturationCharacter, stage_idx: usize) -> f32 {
    let level = input.abs();
    
    // Scale gain based on level (gentle on quiet signals)
    let level_factor = if level < 0.2 {
        0.5  // Very gentle on quiet signals
    } else if level < 0.5 {
        0.8  // Moderate on mid-level
    } else {
        1.0  // Full saturation on loud signals
    };
    
    let gain = 1.0 + drive * 7.0 * level_factor;
    // ... rest of processing
}
```

**Benefits:**
- ✅ Preserves quiet detail (breath, room tone)
- ✅ More aggressive on peaks (where saturation is musical)
- ✅ Better dynamic range preservation

**Cost:** ~2 comparisons + 1 multiplication per stage = ~3 ns/sample

**Recommended:** ⭐⭐⭐⭐ (Excellent for preserving dynamics)

---

### Priority 5: Pre-Emphasis / De-Emphasis (FREQUENCY SHAPING)

**Concept**: Boost highs before saturation, cut them after (analog tape technique)

**Implementation:**
```rust
pub struct AdaptiveSaturator {
    // ... existing fields ...
    pre_emphasis_state: [f32; 2],  // Left/right previous samples
    de_emphasis_state: [f32; 2],
}

// Simple 1-pole high-shelf filter
fn pre_emphasis(&mut self, input: f32, channel: usize) -> f32 {
    let coeff = 0.3;  // Boost amount
    let output = input + coeff * (input - self.pre_emphasis_state[channel]);
    self.pre_emphasis_state[channel] = input;
    output
}

fn de_emphasis(&mut self, input: f32, channel: usize) -> f32 {
    let coeff = 0.3;
    let output = input - coeff * (input - self.de_emphasis_state[channel]);
    self.de_emphasis_state[channel] = input;
    output
}
```

**Benefits:**
- ✅ Adds brightness without harshness
- ✅ Classic analog tape sound
- ✅ Prevents muddy saturation

**Cost:** ~10 ns/sample (very efficient filters)

**Recommended:** ⭐⭐⭐ (Nice touch, but adds complexity)

---

### Priority 6: ZCR-Based Frequency Adaptation (CONTENT-AWARE)

**Concept**: Adjust saturation based on frequency content (ZCR = zero crossing rate)

**Implementation:**
```rust
// Low ZCR = bass-heavy, High ZCR = treble-heavy
let freq_factor = if analysis.zcr_hz < 200.0 {
    1.2  // More saturation on bass (adds warmth)
} else if analysis.zcr_hz > 1000.0 {
    0.7  // Less saturation on highs (prevents harshness)
} else {
    1.0  // Neutral on mids
};
let adaptive_drive = (drive_internal * transient_mult * freq_factor).min(1.0);
```

**Benefits:**
- ✅ Frequency-adaptive without filters (zero latency!)
- ✅ Adds warmth to bass without muddying highs
- ✅ Uses existing signal analysis

**Cost:** ~2 comparisons = <1 ns/sample

**Recommended:** ⭐⭐⭐⭐ (Clever use of existing data)

---

### Priority 7: Parallel Multiband Saturation (ADVANCED)

**Concept**: Split into 3 bands (bass/mid/high), saturate differently, recombine

**Implementation:**
```rust
// 3-band crossover filters
struct MultibandSaturator {
    low_crossover: BiquadFilter,   // 300 Hz
    high_crossover: BiquadFilter,  // 3000 Hz
    // ... saturators per band
}
```

**Benefits:**
- ✅ Surgical control (saturate bass, preserve vocal clarity)
- ✅ Professional mixing tool

**Drawbacks:**
- ❌ **Adds latency** (filter phase shift: ~50-100 samples)
- ❌ Complex implementation
- ❌ Higher CPU cost (~200-300 ns/sample)

**Recommended:** ❌ (Violates zero-latency requirement)

---

### Priority 8: Harmonic Exciter Integration (ADDITIVE)

**Concept**: Add harmonics without clipping (additive synthesis)

**Implementation:**
```rust
fn add_harmonics(&self, input: f32, amount: f32) -> f32 {
    let fundamental = input;
    let second = (input * 2.0).sin() * amount * 0.1;   // 2nd harmonic
    let third = (input * 3.0).sin() * amount * 0.05;   // 3rd harmonic
    fundamental + second + third
}
```

**Benefits:**
- ✅ Adds "air" and presence without distortion
- ✅ Complementary to saturation

**Drawbacks:**
- ❌ Not true analog saturation (synthetic)
- ❌ May sound artificial

**Cost:** ~15 ns/sample (2 sin() calls)

**Recommended:** ⭐⭐ (Nice effect but different from analog emulation)

---

## Recommended Implementation Plan

### Phase 1: Low-Hanging Fruit (Immediate)
1. ✅ **Sibilance-aware reduction** - Essential for vocal clarity
2. ✅ **Voiced/unvoiced adaptation** - Natural for vocal plugin
3. ✅ **ZCR-based frequency adaptation** - Free performance using existing data

**Total cost: <5 ns/sample**
**Benefit: Massive improvement in transparency**

### Phase 2: Advanced (Optional)
4. ⏸️ Level-dependent curves - Better dynamics
5. ⏸️ Pre-emphasis/de-emphasis - Analog tape character
6. ⏸️ Dynamic stage count - Experimental, test carefully

**Total cost: ~50-100 ns/sample (still <0.5% CPU)**

### Phase 3: Future (Beyond Zero-Latency)
- Multiband saturation (requires latency budget)
- Convolution-based saturation (high CPU)

## Performance Budget Analysis

**Current:**
- 82.4 ns/sample
- 0.36% of sample period

**After Phase 1 additions:**
- ~87 ns/sample (add 5 ns for sibilance/voice/ZCR)
- Still <0.4% of sample period
- **Zero latency maintained** ✅

**After Phase 2 additions:**
- ~150 ns/sample (add 68 ns for level curves, pre/de-emphasis, dynamic stages)
- Still <0.7% of sample period
- **Zero latency maintained** ✅

**Maximum safe usage:**
- Target: <500 ns/sample (<2.3% CPU)
- Headroom for 200+ simultaneous instances

## Benchmarking Strategy

After implementing Phase 1, run:
```bash
cargo bench --bench saturation_bench --features voice-clap
```

Compare:
1. Per-character speed (should be <50 ns/sample)
2. Full engine (should be <100 ns/sample)
3. Latency (must remain 0 samples)

If performance degrades >20%, profile with:
```bash
cargo bench --bench saturation_bench --features voice-clap -- --profile-time=10
```

## User Testing Priorities

Test improvements on:
1. **Sibilant vocals** ("s", "sh", "ch" sounds) - Should be smoother
2. **Sustained vowels** - Should be warm, not muddy
3. **Breathy vocals** - Should preserve dynamics
4. **Aggressive rap vocals** - Should handle transients well

## Conclusion

**Recommended immediate action**: Implement Phase 1 (sibilance + voice + ZCR adaptations)

**Cost**: <5 ns/sample
**Benefit**: Dramatically improved vocal transparency
**Risk**: Minimal (all using existing signal analysis data)
**Latency**: Zero (maintained)

This gives you the best bang for your buck - huge quality improvement with negligible performance cost.
