# DSynth Voice User Manual

**Version 1.0** | Real-Time Vocal Enhancement Plugin

---

## Overview

DSynth Voice is a professional vocal saturation plugin designed for real-time processing of vocal tracks. It uses a sophisticated **4-band multiband saturation architecture** with **mid-side processing** to add warmth, character, and harmonic richness while maintaining clarity and phase coherence.

### What Makes It Different

Unlike traditional saturators that process the entire frequency spectrum uniformly, DSynth Voice splits your signal into four distinct frequency bands, allowing you to apply different amounts of harmonic enhancement to each range. This surgical approach prevents low-frequency mud while adding controlled presence and air where you need it.

### Key Features

- **4-band multiband saturation**: Bass, Mids, Presence, and Air
- **Mid-side processing**: Independent control over center and stereo content
- **Phase-coherent design**: No comb filtering or "underwater" artifacts
- **Analog-style waveshaping**: Tube-inspired tanh saturation with dynamic drive
- **Parallel processing**: Global mix control for blend with dry signal
- **Zero latency processing**: Real-time performance suitable for tracking
- **Intelligent de-esser**: Dynamic EQ-based sibilance reduction (6.5 kHz center)
- **Transient shaper**: Attack enhancement for clarity and punch control
- **Adaptive compression/limiting**: Transient-aware dynamics with -0.5dB ceiling

---

## Signal Flow

Understanding the signal path helps you make better mixing decisions:

```
Input Audio
    ↓
Input Gain (staging)
    ↓
4-Band Crossover Split (80Hz, 200Hz, 1kHz, 8kHz)
    ↓
Mid-Side Encoding (L/R → M/S)
    ↓
Per-Band Saturation (Bass, Mid, Presence, Air)
    ↓
Stereo Width Control (M/S manipulation)
    ↓
Mid-Side Decoding (M/S → L/R)
    ↓
Global Mix (parallel blend)
    ↓
Output Gain (makeup/final level)
    ↓
Output Audio
```

### Frequency Band Splits

- **Bass**: DC to 200 Hz (fundamental, body, warmth)
- **Mids**: 200 Hz to 1 kHz (vocal presence, clarity)
- **Presence**: 1 kHz to 8 kHz (intelligibility, edge)
- **Air**: 8 kHz and above (brightness, shimmer)

---

## Parameters

### Input Section

#### Input Gain (-12 dB to +12 dB)
**What it does**: Adjusts the level of the incoming signal before processing.

**Why use it**: Proper gain staging is critical for saturation. Too low and you won't engage the saturation algorithms; too loud and you'll get excessive distortion.

**Expected effect**: 
- **-6 to 0 dB**: Gentle saturation (subtle warmth)
- **0 to +3 dB**: Moderate saturation (vocal character)
- **+3 to +6 dB**: Heavy saturation (aggressive texture)
- **+6 to +12 dB**: Extreme saturation (lo-fi/effect)

**Pro tip**: Start at 0 dB and adjust based on your input level. If your vocal peaks around -12 dBFS, try +3 dB input gain to hit the sweet spot.

---

### Bass Band (DC - 200 Hz)

This band affects the low-end body and warmth without muddying the vocal.

#### Bass Drive (0% - 100%)
**What it does**: Controls the intensity of saturation applied to the bass frequencies.

**Why use it**: Adds weight and warmth to thin vocals, or adds sub-harmonic content to make vocals sit better on small speakers.

**Expected effect**:
- **0-20%**: Subtle thickening, barely noticeable
- **30-50%**: Noticeable warmth, fuller low-end
- **60-80%**: Strong bass character, vintage console vibe
- **90-100%**: Heavy distortion, creative effect

**When to use**:
- Male vocals that sound thin
- Podcasts/voiceovers needing authority
- Modern pop vocals for sub-bass presence

**Caution**: Too much bass drive can cause muddiness on full mixes. Solo the vocal to judge.

#### Bass Mix (0% - 100%)
**What it does**: Blends the saturated bass signal with the clean bass signal (parallel processing at the band level).

**Why use it**: Allows you to dial in saturation intensity without losing the clean fundamental.

**Expected effect**:
- **0%**: Clean bass (no saturation)
- **30-50%**: Balanced mix (warmth + clarity)
- **70-100%**: Full saturation (maximum effect)

**Pro tip**: High drive + low mix = subtle color. Low drive + high mix = transparent enhancement.

---

### Mids Band (200 Hz - 1 kHz)

The vocal presence zone - where most of the "voice" lives.

#### Mid Drive (0% - 100%)
**What it does**: Saturation intensity for the midrange frequencies.

**Why use it**: This is where you add "vocal color" and character. The midrange carries the fundamental pitch and vowel formants.

**Expected effect**:
- **0-30%**: Clean and modern
- **40-60%**: Tape-style warmth (most common range)
- **70-90%**: Vintage console character
- **100%**: Heavy distortion (special effects)

**When to use**:
- Digital recordings that sound too clean/sterile
- Vocals that need to "sit" in a dense mix
- Emulating vintage recording equipment

**Sweet spot**: 40-50% drive with 40-50% mix for natural warmth.

#### Mid Mix (0% - 100%)
**What it does**: Parallel blend of saturated midrange with clean midrange.

**Why use it**: Critical for maintaining vocal clarity while adding character.

**Expected effect**:
- **20-40%**: Subtle enhancement (transparent)
- **50-70%**: Clear saturation (present but musical)
- **80-100%**: Full effect (bold, forward)

**Pro tip**: Lower mix percentages work well with higher drive settings for controlled saturation.

---

### Presence Band (1 kHz - 8 kHz)

The intelligibility and articulation zone.

#### Presence Drive (0% - 100%)
**What it does**: Saturation intensity for the upper-mid frequencies.

**Why use it**: Adds edge, bite, and helps vocals cut through dense mixes. This is the "radio voice" frequency range.

**Expected effect**:
- **0-30%**: Natural, soft (ballads, intimate vocals)
- **40-60%**: Clear, articulate (pop, rock)
- **70-90%**: Aggressive, forward (rap, hard rock)
- **100%**: Harsh distortion (creative use only)

**When to use**:
- Vocals buried in the mix
- Mumbled or unclear performances
- Broadcasting/podcasting (intelligibility)

**Caution**: Too much presence can cause ear fatigue and harshness. Use sparingly.

#### Presence Mix (0% - 100%)
**What it does**: Blends saturated presence with clean presence.

**Why use it**: Allows you to add articulation without making vocals sound processed.

**Expected effect**:
- **20-40%**: Subtle definition boost
- **50-70%**: Clear enhancement
- **80-100%**: Full saturation (may be harsh)

**Pro tip**: Keep presence mix lower than other bands (30-40%) for natural results.

---

### Air Band (8 kHz+)

The "air" and "shimmer" frequency range.

#### Air Drive (0% - 100%)
**What it does**: Controls harmonic exciter intensity in the high frequencies.

**Why use it**: Adds brightness, airiness, and "expensive" sheen without boosting raw high frequencies (which can sound harsh).

**Expected effect**:
- **0%**: No air enhancement (purist mode)
- **5-15%**: Subtle shine (most natural)
- **20-40%**: Clear brightness (modern production)
- **50-100%**: Aggressive sparkle (creative effect)

**When to use**:
- Dull or dark-sounding vocals
- Modern pop/EDM production
- Making vocals sound more "expensive"

**How it works**: The air exciter adds **harmonically-related overtones** above 8 kHz, rather than just boosting existing content. This creates perceived brightness without harshness.

#### Air Mix (0% - 100%)
**What it does**: Blends the excited high frequencies with the original highs.

**Why use it**: Controls how much sparkle you add without overwhelming the natural vocal tone.

**Expected effect**:
- **10-20%**: Subtle sheen (transparent)
- **30-50%**: Noticeable air (clear but musical)
- **60-100%**: Maximum brightness (bold)

**Pro tip**: Start with 10% drive and 15% mix, then adjust to taste. A little goes a long way.

---

### Dynamics Processing

The following three processors work together to provide professional vocal dynamics control with zero latency.

---

### De-Esser (Intelligent Sibilance Reduction)

The intelligent de-esser uses a **dynamic EQ approach** centered at 6.5 kHz to reduce harsh sibilance (s, sh, ch sounds) without affecting the rest of the vocal spectrum.

#### How It Works

Unlike traditional split-band de-essers that process high frequencies broadly, this uses:
- **Band-pass detector** at 6.5 kHz to isolate sibilance energy
- **Dynamic high-shelf cut** that only activates when sibilance is detected
- **Stereo-linked detection** for consistent stereo image
- **Zero latency** envelope followers (no lookahead)

#### De-Esser Threshold (0% - 100%)

**What it does**: Controls how sensitive the de-esser is to sibilance. Higher values = less sensitive (triggers less often).

**Why use it**: Different voices and microphones produce different amounts of sibilance. This parameter adapts the de-esser to your source material.

**Expected effect**:
- **0-20%**: Very sensitive (catches all sibilance, may over-process)
- **30-50%**: Balanced (typical sibilance reduction)
- **60-80%**: Conservative (only harsh sibilance)
- **90-100%**: Minimal processing (barely triggers)

**How to set it**:
1. Set Amount to 100% (full effect)
2. Play back vocal with sibilant content ("s", "sh", "ch" sounds)
3. Adjust Threshold until sibilance is controlled without dulling consonants
4. Reduce Amount to taste (typically 40-70%)

**Technical note**: The threshold range is mapped non-linearly (0% = -50dB, 100% = -10dB) with a power curve for musical control in the mid-range.

#### De-Esser Amount (0% - 100%)

**What it does**: Controls the intensity of sibilance reduction (like a wet/dry mix for the de-essing effect).

**Why use it**: Allows you to dial in the perfect balance between clarity and sibilance control.

**Expected effect**:
- **0%**: No de-essing (bypass)
- **20-40%**: Subtle control (natural)
- **50-70%**: Moderate reduction (most common range)
- **80-100%**: Heavy reduction (aggressive)

**When to use**:
- **High values (70-100%)**: Bright microphones, harsh sibilance, broadcast
- **Medium values (40-60%)**: Typical vocal production
- **Low values (20-30%)**: Subtle control, already well-recorded vocals

**Pro tip**: Start at 50% amount and 40% threshold, then adjust. Use your DAW's spectrum analyzer to see the 5-8 kHz range during sibilant sounds.

**Caution**: Excessive de-essing can make vocals sound lispy or dull. Always compare with bypass.

---

### Transient Shaper (Attack Enhancer)

The transient shaper enhances or suppresses transient content (consonants, plosives, percussive elements) while leaving sustained notes untouched. It uses **pre-computed transient detection** for zero-latency, sample-accurate processing.

#### How It Works

- **Analysis-driven**: Uses real-time transient detection (fixed 0.15 sensitivity)
- **Zero latency**: Envelope-based gain modulation with immediate response
- **Transient-only**: Only modulates during detected transients (non-transients pass through)
- **Fast envelope**: 1ms attack for precise transient tracking
- **Gain smoothing**: 5ms smoothing to prevent clicks

#### Attack (-100% to +100%)

**What it does**: Controls transient boost (positive) or suppression (negative).

**Why use it**: Gives you surgical control over vocal clarity and punch without affecting the body of the vocal.

**Expected effect**:
- **-100% to -50%**: Heavy transient suppression (smooth, warm, vintage)
- **-40% to -20%**: Moderate softening (gentle, intimate)
- **-10% to +10%**: Neutral zone (minimal effect)
- **+20% to +40%**: Moderate enhancement (clear, present)
- **+50% to +70%**: Strong enhancement (punchy, aggressive)
- **+80% to +100%**: Maximum punch (hyper-articulate, modern)

**When to use positive values** (transient boost):
- Mumbled or unclear vocals
- Dense mixes where vocals get buried
- Modern pop/hip-hop production
- Emphasizing consonants for intelligibility
- Broadcast/podcasting clarity

**When to use negative values** (transient suppression):
- Harsh or sibilant vocals
- Vintage/warm character
- Intimate ballads
- Smoothing plosives (p, t, k sounds)
- Reducing mic pops

**How it works technically**:
1. Fast envelope (1ms) tracks signal peaks
2. When transient detected (strength ≥ 0.15), applies gain based on Attack parameter
3. Gain formula: `1.0 + attack_param × (envelope - 0.5)`
4. Non-transients pass through at unity gain (no processing)

**Pro tip**: 
- For clarity: +40% attack + 50% de-esser amount
- For warmth: -30% attack + low de-esser amount
- For radio voice: +60% attack + 60% de-esser amount

**Important**: At 0% attack, the processor bypasses completely (bit-perfect passthrough). Small adjustments (±10-20%) are often sufficient.

---

### Adaptive Compression/Limiter

A professional compressor-limiter that combines intelligent compression with hard ceiling limiting. Uses **envelope followers** (no lookahead) for zero-latency operation while being gentle on transients.

#### How It Works

- **Stereo-linked RMS detection**: Uses max of L/R for consistent stereo image
- **Adaptive attack**: 1ms for peaks, 10ms for transients (preserves punch)
- **Fixed 10:1 ratio**: Limiting characteristic (transparent → brick wall)
- **Fixed -0.5dB ceiling**: Safe headroom to prevent intersample peaks
- **Soft knee**: 3dB knee for smooth compression curve

**Signal flow:**
```
Input → RMS Detection (stereo-linked)
      → Compression Stage (adaptive ratio based on transients)
      → Hard Ceiling Limiter (-0.5dB)
      → Output
```

#### Comp Threshold (-20 dB to 0 dB)

**What it does**: Sets the level above which compression begins. This is the "knee point" where the compressor starts working.

**Why use it**: Controls how much of your vocal is being compressed. Lower thresholds compress more of the signal; higher thresholds only compress peaks.

**Expected effect**:
- **-20 to -15 dB**: Heavy compression (most of signal compressed)
- **-12 to -8 dB**: Moderate compression (standard vocal setting)
- **-6 to -3 dB**: Light compression (only peaks compressed)
- **-1 to 0 dB**: Safety limiting (almost no compression)

**How to set it**:
1. Play back vocal at typical level
2. Watch your DAW's meters to see peak levels
3. Set threshold 3-6 dB below average peak level
4. Adjust to taste (lower = more compression)

**When to use**:
- **-10 dB**: Typical starting point for vocals
- **-15 dB**: Controlling dynamic range on expressive vocals
- **-6 dB**: Gentle glue on already well-controlled vocals
- **-3 dB**: Transparent peak limiting only

**Pro tip**: The compressor has a **soft knee** (3dB), so compression starts gently before the threshold and gradually increases. This makes the threshold setting more forgiving.

#### Comp Release (50 ms to 500 ms)

**What it does**: Controls how quickly the compressor recovers after the signal drops below the threshold.

**Why use it**: Determines the "breathing" character of the compression. Faster = tighter, more obvious; slower = smoother, more transparent.

**Expected effect**:
- **50-100 ms**: Fast release (tight, pumpy, obvious compression)
- **150-250 ms**: Moderate release (most common range, natural)
- **300-400 ms**: Slow release (smooth, transparent, glue)
- **450-500 ms**: Very slow (gentle, minimal pumping)

**When to use**:
- **Fast (50-100ms)**: Aggressive vocal control, modern pop, rap
- **Medium (150-250ms)**: General purpose, most vocals
- **Slow (300-500ms)**: Ballads, smooth jazz, transparent control

**How it interacts with material**:
- **Fast release + low threshold**: Obvious "pumping" effect
- **Slow release + low threshold**: Smooth leveling, bus glue
- **Medium release + medium threshold**: Balanced, natural control

**Technical note**: The compressor uses **envelope followers** with separate attack/release coefficients. Release time determines how fast the gain reduction "lets go" after the signal level drops.

**Pro tip**: Start at 200ms (default), then adjust:
- If compression sounds obvious/pumpy → increase release time
- If compression sounds sluggish/laggy → decrease release time
- Match release to song tempo for rhythmic compression

#### Transient-Aware Attack

**How it works** (automatic, not user-adjustable):
- **Non-transient peaks**: 1ms attack (fast response to level spikes)
- **Transients**: 10ms attack (preserves punch on consonants/plosives)

The compressor automatically detects transients using the signal analyzer and adjusts its attack time accordingly. This preserves vocal articulation while still controlling sustained notes and peaks.

**Why this matters**:
- Traditional compressors with fixed attack times either:
  - **Fast attack**: Controls peaks but dulls transients
  - **Slow attack**: Preserves transients but lets peaks through
- Adaptive attack gives you **both**: controlled peaks AND punchy transients

#### Hard Ceiling Limiter

**What it does** (automatic, not user-adjustable):
- Brick wall at **-0.5dB** to prevent intersample peaks
- Always active as final safety stage
- Operates after compression stage

**Why it's fixed at -0.5dB**:
- Prevents digital clipping
- Leaves headroom for intersample peaks (sample reconstruction can exceed 0dBFS)
- Ensures safe output for all downstream processing

**How compression and limiting interact**:
1. Compression stage reduces dynamic range (threshold + 10:1 ratio)
2. Hard limiter catches any peaks above -0.5dB
3. Combined effect: controlled dynamics + absolute ceiling

---

### Dynamics Section Workflow

The three dynamics processors work together as a professional vocal chain:

#### Order of Operations (in voice_engine.rs)
```
Input → Noise Gate → Pitch Detection
      → Parametric EQ
      → Compressor (adaptive compression/limiter)
      → De-Esser (sibilance reduction)
      → Sub Oscillator
      → Exciter
      → Transient Shaper (attack enhancement)
      → Limiter (final safety)
      → Output
```

#### Recommended Starting Settings

**For Modern Pop Vocals:**
- Comp Threshold: -10 dB
- Comp Release: 200 ms
- De-Esser Threshold: 40%
- De-Esser Amount: 60%
- Attack: +30%

**For Warm/Vintage Character:**
- Comp Threshold: -12 dB
- Comp Release: 300 ms
- De-Esser Threshold: 60%
- De-Esser Amount: 40%
- Attack: -20%

**For Broadcast/Podcast:**
- Comp Threshold: -8 dB
- Comp Release: 150 ms
- De-Esser Threshold: 30%
- De-Esser Amount: 70%
- Attack: +50%

**For Transparent Enhancement:**
- Comp Threshold: -6 dB
- Comp Release: 250 ms
- De-Esser Threshold: 50%
- De-Esser Amount: 40%
- Attack: +10%

#### Tips for Dialing In

1. **Start with compression**:
   - Set threshold to taste (-8 to -12 dB typical)
   - Leave release at 200ms initially
   - This provides the foundation for dynamics control

2. **Add de-essing**:
   - Set amount to 50%, threshold to 40%
   - Play sibilant content ("s", "sh" sounds)
   - Adjust threshold until sibilance is controlled but not dull
   - Fine-tune amount for intensity

3. **Fine-tune attack**:
   - Start at 0% (neutral)
   - Boost (+20 to +40%) for clarity in dense mixes
   - Cut (-20 to -40%) for warmth or to tame harshness
   - Small adjustments make big differences

4. **Listen in context**:
   - Always A/B with bypass
   - Check mono compatibility (fold to mono in DAW)
   - Verify on multiple playback systems
   - Rest your ears periodically (ear fatigue affects judgment)

5. **Avoid over-processing**:
   - If it sounds "squashed" → raise compression threshold or increase release
   - If it sounds "dull" → reduce de-esser amount
   - If consonants sound unnatural → reduce attack boost
   - When in doubt, use less

---

### Stereo Section

#### Stereo Width (-100% to +100%)
**What it does**: Manipulates the stereo image by adjusting the balance between mid (center) and side (stereo) content.

**Why use it**: Control how "wide" or "narrow" the processed vocal sounds.

**Expected effect**:
- **-100% to -50%**: Narrower than original (more mono)
- **-49% to 0%**: Slightly narrower (mono-compatible)
- **0%**: Original stereo image (no change)
- **+1% to +50%**: Slightly wider (subtle enhancement)
- **+51% to +100%**: Maximum width (bold stereo)

**When to use**:
- **Positive values**: Background vocals, doubles, stereo effects
- **Zero**: Lead vocals (keep centered)
- **Negative values**: Ensuring mono compatibility for broadcast

**Caution**: Extreme positive values can cause phase issues and poor mono compatibility. Always check in mono.

---

### Master Section

#### Global Mix (0% - 100%)
**What it does**: Master parallel processing control - blends the entire saturated signal with the original dry signal.

**Why use it**: The most powerful control for subtlety. You can dial in heavy saturation settings, then back off the global mix for "saturation under the hood."

**Expected effect**:
- **0%**: Completely dry (bypass equivalent)
- **25-40%**: Subtle enhancement (gentle glue)
- **50%**: Classic parallel saturation (pro technique)
- **75%**: Strong saturation (bold character)
- **100%**: Full wet signal (all saturation, no dry)

**Professional technique**: Set all your saturation parameters to taste, then use Global Mix to control intensity. This is how professional mixers use parallel compression and saturation.

**When to use**:
- **50% mix**: Standard parallel saturation technique
- **30% mix**: Subtle "glue" without obvious processing
- **100% mix**: When you want the full effect (creative/aggressive)

#### Output Gain (-12 dB to +12 dB)
**What it does**: Final makeup gain to match the output level to your input level (or adjust to taste).

**Why use it**: Saturation often changes perceived loudness. This compensates for level changes and allows A/B comparison at matched volumes.

**Expected effect**:
- **-6 to 0 dB**: Reduce output (if saturation made it too loud)
- **0 dB**: Unity gain (no level change)
- **+1 to +6 dB**: Boost output (if saturation reduced level)

**Pro tip**: Use your DAW's gain meter to match the bypassed and processed levels. This ensures you're judging the effect, not just "louder = better."

---

## Workflow Tips

### Starting From Scratch

1. **Set Global Mix to 100%** initially so you can hear what you're doing
2. **Leave Input Gain at 0 dB** to start
3. **Dial in each band** one at a time:
   - Start with Mids (40% drive, 40% mix)
   - Add Bass if needed (60% drive, 50% mix)
   - Add Presence carefully (35% drive, 35% mix)
   - Add Air subtly (10% drive, 15% mix)
4. **Adjust Stereo Width** only if processing doubles/backgrounds (keep lead at 0%)
5. **Pull back Global Mix** to taste (try 50% for parallel saturation)
6. **Match levels** with Output Gain

### Common Presets (Conceptual)

#### "Clean Vocal" (Transparent Enhancement)
- Bass: 30% drive, 40% mix
- Mids: 40% drive, 30% mix
- Presence: 25% drive, 25% mix
- Air: 8% drive, 12% mix
- Global Mix: 100%

#### "Radio Voice" (Broadcast/Podcast)
- Bass: 70% drive, 60% mix
- Mids: 55% drive, 45% mix
- Presence: 50% drive, 40% mix
- Air: 12% drive, 18% mix
- Global Mix: 100%

#### "Vintage Warmth" (Tape/Console Vibe)
- Bass: 60% drive, 50% mix
- Mids: 50% drive, 60% mix
- Presence: 35% drive, 40% mix
- Air: 5% drive, 10% mix
- Global Mix: 75%

#### "Modern Pop" (Bright & Present)
- Bass: 40% drive, 40% mix
- Mids: 45% drive, 40% mix
- Presence: 45% drive, 45% mix
- Air: 20% drive, 25% mix
- Global Mix: 100%

---

## Technical Notes

### Harmonic Content

The saturation algorithms use analog-style waveshaping:
- **Hyperbolic tangent (tanh)**: Smooth, tube-inspired soft clipping
- **Dynamic drive**: Louder passages get more saturation for analog compression feel
- **Per-band processing**: Independent saturation characteristics for Bass, Mids, Presence, and Air
- **Auto-gain compensation**: Maintains perceived loudness across drive settings

This creates natural, musical harmonic distortion that responds dynamically to your input signal.

### Phase Coherence

DSynth Voice uses a **single unified crossover** for both mid and side channels, ensuring phase-aligned frequency splitting. This prevents the "phasy" or "underwater" sound common in poorly-designed multiband processors.

All processing maintains **linear-phase relationships** across bands, making it safe to use on stereo material and in parallel with other tracks.

### Latency

The plugin operates with **zero latency** - no internal buffering delays. This makes it ideal for real-time tracking and live performance.

### CPU Usage

Optimized for real-time performance. Typical CPU usage: **<3%** on modern systems (Apple Silicon, 2020+).

---

## Troubleshooting

### "The vocals sound harsh"
- Reduce Presence Drive and/or Mix
- Lower Input Gain by 2-3 dB
- Try Global Mix at 50% instead of 100%

### "The vocals sound muddy"
- Reduce Bass Drive and/or Mix
- Check your source material - may need EQ before saturation
- Try narrower Stereo Width (-20%)

### "The effect is too subtle"
- Increase Input Gain (+3 to +6 dB)
- Ensure Global Mix is at 100% (not 50%)
- Increase individual Drive parameters

### "The vocals sound 'underwater' or phasy"
- This should not occur - the plugin is designed to be phase-coherent
- Check if you're using other multiband plugins in series (potential phase accumulation)
- Ensure Stereo Width is at 0% for mono sources

### "The vocals are too loud/quiet after processing"
- Adjust Output Gain to match your input level
- Use your DAW's metering to A/B bypass at matched levels

---

## Comparison to Traditional Tools

| Traditional Approach | DSynth Voice Advantage |
|---------------------|------------------------|
| Full-spectrum saturation | Surgical per-band control |
| Static saturation curve | Dynamic, level-responsive drive |
| Phase issues in multiband | Single unified crossover |
| Mono or stereo only | Mid-side processing |
| All-or-nothing processing | Parallel processing (Global Mix) |

---

## Credits & Technical Details

**DSynth Voice** is part of the DSynth audio synthesis suite.

- **Format**: CLAP Plugin
- **Processing**: 64-bit floating-point
- **Sample Rates**: 44.1 kHz - 192 kHz
- **Latency**: ~28 samples (reported for DAW compensation)
- **Channels**: Stereo in/out
- **Platform**: macOS (Apple Silicon + Intel), Windows planned

---

## Mid-Side Processing Explained
Mid-side processing separates the stereo signal into two components:
- **Mid (M)**: The center channel (sum of left and right)
- **Side (S)**: The stereo information (difference between left and right)
This allows independent processing of the center and stereo elements, giving you more control over how the vocal sits in the mix.
---

**For technical support or feature requests, visit:**  
[https://github.com/yourusername/dsynth](https://github.com/yourusername/dsynth)

---

*DSynth Voice v1.0 - Professional Vocal Saturation Plugin*
