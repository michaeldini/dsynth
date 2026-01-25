# Drive Calibration Analysis & Improved Saturation Techniques

## Problem Statement

**Current Issue**: Drive above 50% produces obvious distortion that degrades vocal clarity. The plugin should add harmonic richness while preserving intelligibility.

**User Requirement**: Transparent harmonic enhancement - vocals should sound warmer/richer without sounding "distorted"

## Root Cause Analysis

### 1. Aggressive Drive Mapping

**Current implementation**:
```rust
let drive_internal = drive.powf(2.5);  // drive^2.5
let gain = 1.0 + drive * 19.0;         // gain range: 1.0× to 20.0×
```

**At 50% drive**:
- Internal drive: `0.5^2.5 = 0.177`
- Stage 1 gain: `1.0 + 0.177*0.6*19.0 = ~3.0×` (too aggressive!)
- Total cascade: 3 stages compounding

**At 100% drive**:
- Internal drive: `1.0`
- Stage 1 gain: `1.0 + 0.6*19.0 = 12.4×` (extreme!)

**Problem**: Even with drive^2.5 tapering, the gain range (1-20×) is too wide for transparent processing.

### 2. Stage Distribution Too Heavy

**Current**: 60%/25%/15% = 100% total drive distributed across stages
- Stage 1: 60% of drive (does most of the work)
- Stage 2: 25% (adds more distortion)
- Stage 3: 15% (final clipping)

**Problem**: First stage hits too hard, second stage compounds it.

### 3. Hard Clipping in Waveshaping

**Warm (Tube)**:
```rust
if abs_x > 0.8 { return x.signum() * 0.9; }  // Hard clip at 0.8!
```

**Punchy (Console)**:
```rust
if abs_x > 1.5 { return x.signum() * 0.9; }  // Hard limit
```

**Problem**: Hard clipping = harsh harmonics = audible distortion

### 4. No Parallel Processing

Currently 100% wet signal. No option to blend clean with saturated.

**Problem**: Can't preserve transient clarity while adding harmonics.

## Proposed Solutions

### Solution 1: Gentler Drive Curve (IMMEDIATE FIX)

**Change drive exponent from 2.5 to 3.5-4.0**:
```rust
// OLD: let drive_internal = drive.powf(2.5);
let drive_internal = drive.powf(3.5);  // Much gentler response
```

**Effect**:
- 25% drive: `0.25^3.5 = 0.024` (subtle)
- 50% drive: `0.50^3.5 = 0.088` (moderate, not aggressive)
- 75% drive: `0.75^3.5 = 0.316` (noticeable)
- 100% drive: `1.0^3.5 = 1.0` (full saturation)

### Solution 2: Reduce Gain Range (CRITICAL)

**Change from 1-20× to 1-8×**:
```rust
// OLD: let gain = 1.0 + drive * 19.0;
let gain = 1.0 + drive * 7.0;  // Max 8× instead of 20×
```

**Effect**: Even at max drive, input is only amplified 8× before saturation, preventing harsh clipping.

### Solution 3: Softer Stage Distribution

**Change from 60/25/15 to 40/20/10 (70% total)**:
```rust
// OLD:
let stage1_drive = adaptive_drive * 0.60;
let stage2_drive = adaptive_drive * 0.25;
let stage3_drive = adaptive_drive * 0.15;

// NEW:
let stage1_drive = adaptive_drive * 0.40;
let stage2_drive = adaptive_drive * 0.20;
let stage3_drive = adaptive_drive * 0.10;
```

**Effect**: Reduces cumulative distortion by 30% while maintaining multi-stage character.

### Solution 4: Softer Waveshaping Curves

#### A. Warm (Tube) - Use Polynomial Instead of Hard Clip

```rust
fn warm_saturation(&self, x: f32) -> f32 {
    // Asymmetric soft saturation (preserves even harmonics)
    // Uses Chebyshev-inspired polynomial
    let abs_x = x.abs();
    let sign = x.signum();
    
    if abs_x < 0.1 {
        x  // Linear region for very low levels
    } else {
        // Soft knee polynomial: x - x^3/3 + x^5/5 (Taylor series of arctanh)
        let x2 = abs_x * abs_x;
        let x3 = x2 * abs_x;
        let soft = abs_x - 0.33 * x3;
        
        // Asymmetric bias for even harmonics (tube characteristic)
        let biased = soft + 0.1 * x2;  // Quadratic term adds 2nd harmonic
        
        sign * biased.min(1.0)
    }
}
```

#### B. Smooth (Tape) - Reduce Tanh Scaling

```rust
fn smooth_saturation(&self, x: f32) -> f32 {
    // OLD: let scaled = x * 1.2; scaled.tanh() * 0.9
    
    // NEW: Gentler tanh with softer knee
    let scaled = x * 0.8;  // Reduced from 1.2 to 0.8
    scaled.tanh() * 0.95   // Less output attenuation
}
```

#### C. Punchy (Console) - Softer Knee

```rust
fn punchy_saturation(&self, x: f32) -> f32 {
    // Soft clip with gentler transition
    let abs_x = x.abs();
    
    if abs_x <= 0.7 {  // Increased linear region from 0.5 to 0.7
        x
    } else {
        // Smoother polynomial transition instead of hard knee
        let sign = x.signum();
        let excess = abs_x - 0.7;
        let compressed = 0.7 + excess * (1.0 / (1.0 + excess));  // Hyperbolic compression
        sign * compressed.min(1.0)
    }
}
```

### Solution 5: Add Dry/Wet Mix (NEW PARAMETER)

Add parallel processing to preserve clarity:

```rust
pub struct AdaptiveSaturator {
    // ... existing fields ...
    mix: f32,  // 0.0 = dry, 1.0 = wet
}

pub fn process(
    &mut self,
    left: f32,
    right: f32,
    drive: f32,
    mix: f32,  // NEW parameter
    character: SaturationCharacter,
    analysis: &SignalAnalysis,
) -> (f32, f32) {
    // Store dry signal
    let dry_left = left;
    let dry_right = right;
    
    // ... process saturation as before ...
    
    // Blend dry and wet
    let out_left = dry_left * (1.0 - mix) + left_out * mix;
    let out_right = dry_right * (1.0 - mix) + right_out * mix;
    
    (out_left, out_right)
}
```

**Benefits**:
- 100% mix = full saturation (current behavior)
- 50% mix = parallel saturation (preserves transients + adds harmonics)
- 30% mix = subtle enhancement (optimal for vocals)

### Solution 6: Frequency-Dependent Saturation (ADVANCED)

Split signal into 3 bands, apply different saturation:

```rust
struct MultibandSaturator {
    low_crossover: f32,   // 300 Hz
    high_crossover: f32,  // 3000 Hz
    low_drive_mult: f32,  // 1.2× (enhance bass)
    mid_drive_mult: f32,  // 0.6× (protect vocal clarity)
    high_drive_mult: f32, // 1.0× (normal highs)
}
```

**Reasoning**:
- **Low (0-300Hz)**: Safe to saturate heavily (adds warmth, sub-harmonics)
- **Mid (300-3kHz)**: Preserve clarity (vocal intelligibility range)
- **High (3kHz+)**: Moderate saturation (adds air, presence)

**Trade-off**: Adds latency (filter phase shift), increases CPU cost

### Solution 7: Harmonic Exciter Instead of Saturation

Add subtle high-frequency content without distortion:

```rust
fn harmonic_exciter(&self, input: f32) -> f32 {
    // Generate harmonics without clipping
    let fundamental = input;
    let second_harmonic = (input * 2.0).sin() * 0.1;   // Even harmonic
    let third_harmonic = (input * 3.0).sin() * 0.05;   // Odd harmonic
    
    fundamental + second_harmonic + third_harmonic
}
```

**Benefits**: Adds "excitement" without audible distortion

### Solution 8: Dynamic Drive Based on Input Level

Reduce drive for loud signals, increase for quiet:

```rust
let level_factor = 1.0 / (1.0 + analysis.rms_level);  // Inverse relationship
let adaptive_drive = drive_internal * level_factor;
```

**Effect**: Prevents loud vocals from clipping while enhancing quiet passages

## Recommended Implementation Order

### Phase 1: Quick Fixes (15 minutes)
1. ✅ Change drive exponent: `2.5 → 3.5`
2. ✅ Reduce gain range: `1-20× → 1-8×`
3. ✅ Soften stage distribution: `60/25/15 → 40/20/10`
4. ✅ Test and validate at 50% drive

### Phase 2: Waveshaping Improvements (30 minutes)
5. ✅ Implement softer Warm saturation (polynomial)
6. ✅ Reduce Smooth tanh scaling (`1.2 → 0.8`)
7. ✅ Soften Punchy knee transition
8. ✅ Add tests for new curves

### Phase 3: Mix Parameter (Optional, 20 minutes)
9. ⏸️ Add `mix` parameter to `VoiceParams`
10. ⏸️ Implement parallel processing in `AdaptiveSaturator`
11. ⏸️ Update GUI with mix knob
12. ⏸️ Update tests and benchmarks

### Phase 4: Advanced Techniques (Future)
- Multiband saturation (complex, adds latency)
- Harmonic exciter (alternative approach)
- Dynamic drive compensation (CPU cost)

## Expected Results

**After Phase 1 (Quick Fixes)**:
- 25% drive: Barely noticeable warmth (perfect for subtle enhancement)
- 50% drive: Clear harmonic richness without distortion
- 75% drive: Obvious saturation but still musical
- 100% drive: Heavy saturation (creative effect)

**After Phase 2 (Waveshaping)**:
- Smoother saturation curves = less harsh harmonics
- Better preservation of transient clarity
- More "analog" feel (less digital clipping artifacts)

**After Phase 3 (Mix Parameter)**:
- 30% mix: Optimal vocal transparency (70% dry + 30% saturated)
- 50% mix: Balanced blend
- 100% mix: Full saturation (current behavior)

## Testing Strategy

### A/B Comparison Tests
1. Record dry vocal phrase
2. Process with current settings (50% drive)
3. Process with new settings (50% drive)
4. Compare:
   - Intelligibility (can you understand words?)
   - Warmth (does it sound richer?)
   - Harshness (does it sound distorted?)

### Null Test (Phase Cancellation)
1. Process at 0% drive
2. Compare to dry signal
3. Should be nearly identical (validates bypass)

### Saturation Progression Test
Process at 0%, 25%, 50%, 75%, 100% drive and verify:
- Gradual increase in harmonics
- No sudden jumps or artifacts
- Musical at all settings

## Benchmark Impact Prediction

**Phase 1 Changes**: Negligible impact
- Drive curve change: Same operations, different exponent
- Gain range change: Same multiplication
- Stage distribution: Same processing, different coefficients

**Phase 2 Changes**: Possible 5-10% CPU increase
- Polynomial waveshaping: More operations than hard clip
- Hyperbolic functions: Similar to tanh (already optimized)

**Phase 3 Changes**: ~5% CPU increase
- Dry/wet mixing: 2 extra multiplications + 2 additions per sample

**Expected**: Still well under 100 ns/sample, maintaining excellent real-time performance

## Sound Quality Priority

Following DSynth philosophy: **Sound quality > performance**

These changes prioritize:
1. ✅ Vocal clarity preservation
2. ✅ Musical harmonic enhancement
3. ✅ Transparent processing at moderate settings
4. ✅ Analog authenticity

Even if CPU increases by 20%, we're still at <100 ns/sample, which is excellent for real-time use.

## References

**Soft saturation techniques**:
- Chebyshev polynomials (specific harmonic control)
- Arctanh Taylor series (smooth limiting)
- Hyperbolic compression (gradual saturation)

**Analog modeling**:
- Tube: Even harmonics (2nd, 4th) via asymmetric clipping
- Tape: Smooth compression via tanh
- Console: Transient punch via soft-clip with knee

**Parallel processing**:
- "New York compression" technique adapted for saturation
- Preserves transients while adding density
