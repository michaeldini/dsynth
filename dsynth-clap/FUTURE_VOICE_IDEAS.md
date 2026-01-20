I'll explore creative ways to combine pitch tracking with compressor and exciter for musical vocal enhancement. 

Read [](file:///Users/michaeldini/Dev/dsynth/src/dsp/effects/compressor.rs#1-1), lines 1 to 150

Read [](file:///Users/michaeldini/Dev/dsynth/src/dsp/effects/exciter.rs#1-1), lines 1 to 150

Now let me think about creative applications. Here are some interesting ideas:

**Creative Pitch + Compressor/Exciter Combinations:**

## 1. **Pitch-Responsive Dynamic EQ** 
**Concept**: Compressor threshold and ratio dynamically follow the detected pitch.
- **High notes** = tighter compression (prevent sibilance/harshness)
- **Low notes** = loose compression (preserve bass weight)

```rust
// Pseudocode
let pitch_normalized = (detected_pitch - 80.0) / (400.0 - 80.0);
let dynamic_threshold = -30.0 + pitch_normalized * 15.0;  // -30dB low, -15dB high
let dynamic_ratio = 2.0 + pitch_normalized * 3.0;         // 2:1 low, 5:1 high
compressor.set_threshold(dynamic_threshold);
compressor.set_ratio(dynamic_ratio);
```

**Why it's musical**: Bass notes need breathing room, high notes need controlling.

---

## 2. **Pitch-Tracked Exciter Frequency**
**Concept**: Exciter high-pass frequency follows the detected pitch.
- Creates a **formant** that always sits above the vocal fundamental
- Example: If you sing at 150Hz, exciter boosts 250Hz+

```rust
// Keep exciter always a major 3rd above detected pitch
let exciter_freq = detected_pitch * (2.0_f32.powf(4.0 / 12.0)); // Up 4 semitones
exciter.set_frequency(exciter_freq.clamp(2000.0, 12000.0));
```

**Why it's musical**: Creates dynamic "clarity" that follows your voice, like a presence peak.

---

## 3. **Pitch-Triggered Exciter Boost** 
**Concept**: Exciter amount spikes when you hit specific notes (like compressor sidechain).
- **High confidence pitch + high energy** = boost exciter
- Creates **emphasis on clear, sung notes** vs. breathy/noisy sections

```rust
let clarity_boost = pitch_confidence * (1.0 + energy_level);
exciter.set_mix(base_mix + clarity_boost * 0.3);
```

**Why it's musical**: Automatically emphasizes "good" vocal takes, reduces harshness on breathy parts.

---

## 4. **Harmonic Compression** 
**Concept**: Compress only specific frequency bands based on detected pitch harmonics.
- **Fundamental** (detected pitch) → gentle compression
- **2× harmonic** (2× detected pitch) → aggressive compression  
- **3× harmonic** → even more compression

This prevents the vocal from getting "muddy" when stacking harmonizers.

```rust
// Compress each harmonic separately
let fundamental_level = detect_level_at(detected_pitch);
let harmonic2_level = detect_level_at(detected_pitch * 2.0);
let harmonic3_level = detect_level_at(detected_pitch * 3.0);

// Apply increasing compression to higher harmonics
compress_at_freq(harmonic2_level, ratio=3.0);
compress_at_freq(harmonic3_level, ratio=5.0);
```

**Why it's musical**: Keeps the vocal stack sounding natural without the "wall of voices" effect.

---

## 5. **Confidence-Based Exciter/Compression Balance**
**Concept**: Switch between "expressive" and "corrective" modes based on pitch confidence.
- **High confidence** (>0.7) = less compression, more exciter (trust the voice)
- **Low confidence** (<0.4) = more compression, less exciter (fix the wobble)

```rust
if pitch_confidence > 0.7 {
    // Expressive mode - let them breathe
    compressor.set_ratio(1.5);
    exciter.set_mix(0.6);
} else {
    // Corrective mode - tame the mess
    compressor.set_ratio(4.0);
    exciter.set_mix(0.2);
}
```

**Why it's musical**: Automatically adapts to breathy vs. clear singing.

---

## 6. **Vibrato Correction via Compression**
**Concept**: Detect pitch wobble (vibrato) and use it to trigger fast compression attacks.
- When pitch is **stable** → release compression (breathe)
- When pitch is **wobbling** (vibrato) → tighten compression (control it)

```rust
let pitch_stability = 1.0 - (pitch_delta.abs() / 20.0).min(1.0);
let dynamic_attack = if pitch_stability > 0.8 {
    50.0  // Slow attack when stable
} else {
    5.0   // Fast attack when unstable
};
compressor.set_attack(dynamic_attack);
```

**Why it's musical**: Vibrato stays musical, but micro-wobbles get smoothed out.

---

## **My Top Pick: #2 - Pitch-Tracked Exciter Frequency**

This is the **most musical with minimal code**:
- ✅ Already have pitch detection
- ✅ Already have exciter with frequency control
- ✅ Just need 1 line: `exciter.set_frequency(detected_pitch * 1.5)`
- ✅ Creates dynamic "presence" that follows your melody
- ✅ Works with ANY vocal (clear, breathy, belted)

**Musical Result**: Your voice gets an automatic presence peak that sits above whatever note you're singing - like a professional vocal engineer riding the exciter in real-time.

**Parameters to add** (optional):
- `exciter_follow_enable: bool` - Toggle pitch tracking on/off
- `exciter_follow_ratio: f32` - Multiply factor (1.5 = major 3rd, 2.0 = octave)

---

Would you like me to implement any of these? #2 is super quick (~30 minutes), or I can combine #1 and #5 for a more comprehensive "intelligent compression" system?