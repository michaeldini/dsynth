# DSynth Kick - User Guide

## Overview

**DSynth Kick** is a high-performance kick drum synthesizer CLAP plugin designed for electronic music production. It features **dual oscillators with pitch envelopes**, comprehensive **dynamics processing**, and a transparent **effects chain** optimized for creating punchy, professional kicks from deep 808s to hard techno punches.

**Core Philosophy:**
- **Transparency First**: All effects default to OFF so you know exactly what's shaping your sound
- **Sound Quality**: 4× oversampling and anti-aliasing for pristine audio
- **Performance**: Optimized real-time processing with <5% CPU usage
- **Workflow**: Intuitive layout with visual feedback and DAW automation support

---

## Installation

### macOS
1. Download `DSynthKick.clap` from releases
2. Copy to: `~/Library/Audio/Plug-Ins/CLAP/`
3. Restart your DAW or rescan plugins
4. Load from: **Instruments → CLAP → DSynth Kick**

### Windows
1. Download `DSynthKick.clap`
2. Copy to: `%COMMONPROGRAMFILES%\CLAP\`
3. Rescan plugins in your DAW

### Linux
1. Download `DSynthKick.clap`
2. Copy to: `~/.clap/`
3. Rescan plugins in your DAW

---

## Quick Start

1. **Load the plugin** in your DAW as an instrument track
2. **Play C4 (MIDI note 60)** to hear the default kick sound
3. **Enable effects** you want to use by clicking their ✓ checkbox
4. **Adjust Body Oscillator** (Osc 1) for fundamental tone
5. **Adjust Click Oscillator** (Osc 2) for transient definition
6. **Shape envelope** to control attack and decay timing
7. **Add compression/distortion** for punch and character

**Tip:** Start with a preset (808, Techno, Sub) and tweak from there!

---

## Signal Flow

```
MIDI Note → [Osc 1 Body] ─┐
                           ├→ Mix → Filter → Envelope → Distortion* → 
         → [Osc 2 Click] ─┘                              ↓
                                               Multiband Compressor* →
                                                          ↓
                                                    Exciter* →
                                                          ↓
                                                Transient Shaper* →
                                                          ↓
                                                     Clipper* → Output

* = Optional (disabled by default)
```

All effects are **OFF by default** unless you enable them with the ✓ checkbox.

---

## Interface Sections

### 1. Body Oscillator (Osc 1)
The fundamental tone of your kick. Generates a sine wave with **exponential pitch envelope**.

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Start** | 40-800 Hz | Starting pitch of pitch envelope (high = brighter kick) |
| **End** | 20-400 Hz | Ending pitch (your kick's fundamental frequency) |
| **Decay** | 10-500 ms | How fast the pitch sweeps down (short = fast sweep) |
| **Level** | 0-100% | Body oscillator volume in the mix |

**Typical Settings:**
- **808-style**: Start 150Hz → End 55Hz, Decay 100ms
- **Techno**: Start 200Hz → End 60Hz, Decay 80ms
- **Sub-bass**: Start 120Hz → End 40Hz, Decay 200ms

---

### 2. Click Oscillator (Osc 2)
The transient/attack portion of your kick. Adds definition and presence.

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Start** | 100-8000 Hz | Starting pitch (high = sharp click) |
| **End** | 40-500 Hz | Ending pitch (lower = softer tail) |
| **Decay** | 5-100 ms | Click decay time (very short for sharp attack) |
| **Level** | 0-100% | Click oscillator volume in the mix |

**Typical Settings:**
- **808 click**: Start 3000Hz → End 200Hz, Decay 20ms, Level 30%
- **Hard techno**: Start 4000Hz → End 250Hz, Decay 12ms, Level 40%
- **Minimal click**: Start 1500Hz → End 100Hz, Decay 25ms, Level 15%

**Tip:** Lower Click Level for softer kicks, raise it for aggressive attack.

---

### 3. Envelope (Amplitude)
Controls the overall volume shape of the kick drum.

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Attack** | 0.1-10 ms | How fast the kick reaches full volume (0.5ms = punchy) |
| **Decay** | 50-2000 ms | Main body of the kick (200-600ms typical) |
| **Sustain** | 0-100% | Usually 0% for kicks (no held sustain) |
| **Release** | 10-500 ms | Tail after note-off (30-100ms typical) |

**Typical Settings:**
- **Punchy**: Attack 0.5ms, Decay 300ms, Sustain 0%, Release 50ms
- **Long tail**: Attack 1ms, Decay 600ms, Sustain 0%, Release 100ms
- **Snappy**: Attack 0.2ms, Decay 200ms, Sustain 0%, Release 30ms

---

### 4. Filter
Lowpass filter with envelope modulation for tone shaping.

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Cutoff** | 50-20000 Hz | Lowpass filter frequency (8000Hz = neutral) |
| **Resonance** | 0-100% | Filter resonance (boost at cutoff, 20% typical) |
| **Env Amount** | -100 to +100% | Filter envelope depth (positive = opens filter) |
| **Env Decay** | 10-500 ms | Filter envelope decay time |

**Use Cases:**
- **Dark kicks**: Low cutoff (5000Hz), low resonance
- **Bright transient**: High cutoff (10000Hz), positive env amount (50%)
- **Resonant body**: Medium cutoff (6000Hz), high resonance (30-50%)

**Tip:** Negative Env Amount can create reverse sweep effects.

---

### 5. Distortion (Optional - OFF by default)
Adds harmonic saturation and grit. **Click ✓ to enable.**

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Enable ✓** | On/Off | Master switch for distortion effect |
| **Amount** | 0-100% | Distortion intensity (10-30% = subtle warmth) |
| **Type** | 4 modes | Distortion character (see below) |

**Distortion Types:**
- **Soft**: Gentle saturation, tube-like warmth (default)
- **Hard**: Aggressive clipping, digital character
- **Tube**: Vintage tube warmth, even harmonics
- **Foldback**: Extreme harmonic distortion, metallic

**Typical Settings:**
- **Subtle warmth**: Soft, 10-15%
- **Aggressive**: Hard, 35-50%
- **Vintage**: Tube, 20-30%

---

### 6. Multiband Compressor (Optional - OFF by default)
3-band dynamics processor for frequency-specific control. **Click Enable ✓ to activate.**

**Important:** The multiband compressor now features **automatic makeup gain** that compensates for compression-induced volume loss. When you enable it, the output level should remain similar to the uncompressed signal while adding punch and control.

#### Global Controls
| Parameter | Range | Description |
|-----------|-------|-------------|
| **Enable ✓** | On/Off | Master switch for multiband compression |
| **Low Xover** | 50-500 Hz | Sub/Body crossover frequency (150Hz default) |
| **High Xover** | 400-2000 Hz | Body/Click crossover frequency (800Hz default) |
| **Mix** | 0-100% | Wet/dry blend (100% = full compression) |

#### Sub Band (40-150 Hz)
Controls the deep sub-bass frequencies.

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Bypass** | ✓/✗ | Bypass this band |
| **Threshold** | -60 to 0 dB | Compression kicks in above this level (-20dB default) |
| **Ratio** | 1:1 to 20:1 | Compression strength (4:1 = moderate) |
| **Attack** | 0.1-1000 ms | How fast compressor reacts (5ms = fast) |
| **Release** | 1-5000 ms | Return to normal speed (100ms typical) |
| **Gain** | 0-200% | Post-compression makeup gain (100% = unity) |

#### Body Band (150-800 Hz)
The fundamental tone of most kicks.

Same parameters as Sub Band. **Defaults:** Threshold -15dB, Ratio 3:1, Attack 10ms, Release 150ms

#### Click Band (800 Hz+)
The high-frequency transient and presence.

Same parameters as Sub Band. **Defaults:** Threshold -10dB, Ratio 2:1, Attack 0.5ms, Release 50ms

**Typical Multiband Settings:**
- **808 Glue**: Sub Ratio 4:1, Body Ratio 3:1, Click Ratio 2:1 (default preset)
- **Techno Punch**: Sub Ratio 5:1 Gain 110%, Click Ratio 2.5:1 Gain 120%
- **Sub Focus**: Sub Ratio 6:1 Gain 130%, Body/Click bypassed

**Note on Automatic Makeup Gain:**
Each compressor band automatically calculates and applies makeup gain based on its threshold and ratio:
- **Formula**: `makeup_gain_dB = -threshold_dB × (1 - 1/ratio) × 0.9`
- **Sub band** (-20dB, 4:1): ~13.5dB makeup gain
- **Body band** (-15dB, 3:1): ~9.0dB makeup gain  
- **Click band** (-10dB, 2:1): ~4.5dB makeup gain

This means when you enable the multiband compressor, it should **maintain loudness** while adding punch. The Band Gain knobs now act as additional post-compression level adjustments on top of the automatic makeup gain.

**If volume is still too quiet:** Increase the Band Gain knobs (Sub/Body/Click) to 110-130% for additional boost.

---

### 7. Exciter (Optional - OFF by default)
High-frequency harmonic enhancement for presence. **Click Enable ✓ to activate.**

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Enable ✓** | On/Off | Master switch for exciter |
| **Frequency** | 2000-12000 Hz | High-pass cutoff (affects only highs, 4000Hz default) |
| **Drive** | 0-100% | Harmonic generation intensity (30% = subtle) |
| **Mix** | 0-100% | Wet/dry blend (30% typical) |

**Use Cases:**
- **Air and presence**: Frequency 4000Hz, Drive 30%, Mix 30%
- **Aggressive click**: Frequency 5000Hz, Drive 50%, Mix 40%
- **Subtle polish**: Frequency 3000Hz, Drive 20%, Mix 20%

---

### 8. Transient Shaper (Optional - OFF by default)
Directly manipulates attack and sustain. **Click Enable ✓ to activate.**

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Enable ✓** | On/Off | Master switch for transient shaper |
| **Attack Boost** | 0-100% | Emphasize initial transient (30% = punchier) |
| **Sustain Cut** | 0-100% | Reduce body/tail (20% = tighter kick) |

**Typical Settings:**
- **Punchier**: Attack Boost 50%, Sustain Cut 40%
- **Balanced**: Attack Boost 30%, Sustain Cut 20% (default preset)
- **Subtle**: Attack Boost 20%, Sustain Cut 10%

**Tip:** Use with compression for maximum punch control.

---

### 9. Clipper (Optional - OFF by default)
Brick-wall limiting for maximum loudness. **Click Enable ✓ to activate.**

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Enable ✓** | On/Off | Master switch for clipper |
| **Threshold** | 70-100% | Clipping level (95% = gentle, 85% = aggressive) |

**Use Cases:**
- **Loud techno kicks**: Enable, Threshold 85%
- **Gentle safety limit**: Enable, Threshold 95%
- **Transparent**: Disable (default)

**Warning:** Low thresholds (<90%) introduce audible distortion. Use sparingly!

---

### 10. Master Section

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Level** | 0-100% | Master output volume (80% default) |
| **Vel Sens** | 0-100% | Velocity → amplitude scaling (50% = moderate) |
| **Key Track** | 0-100% | Chromatic pitch tracking (see below) |

#### Key Tracking Explained
Controls how MIDI note pitch affects the kick's fundamental frequency.

- **0%**: All notes trigger the same pitch (default 808 behavior)
- **50%**: Partial chromatic tracking (C5 = 1.4× C4 pitch)
- **100%**: Full chromatic tracking (C5 = 2× C4 pitch, like a keyboard)

**Reference Note:** C4 (MIDI 60) = 261.63 Hz

**Use Cases:**
- **Classic one-shot drum**: Key Track 0% (all notes sound identical)
- **Bassline kicks**: Key Track 50-80% (subtle pitch variation)
- **Melodic kicks**: Key Track 100% (full keyboard response)

**Tip:** Key tracking scales the entire pitch envelope, preserving the sweep ratio.

---

## Presets

DSynth Kick includes **3 factory presets** optimized for common kick styles:

### 808 Kick
- **Character**: Classic analog warmth, subtle distortion
- **Effects Enabled**: Distortion (Soft 10%), Multiband Comp, Exciter (30%), Transient Shaper (30%/20%)
- **Best For**: House, hip-hop, retro electronic
- **Settings**: Body 150→55Hz (100ms), Click 3000→200Hz (20ms)

### Techno Kick
- **Character**: Hard, aggressive, maximum punch
- **Effects Enabled**: Distortion (Hard 35%), Multiband Comp (aggressive ratios), Exciter (50%), Transient Shaper (50%/40%), Clipper (85%)
- **Best For**: Techno, hardstyle, industrial
- **Settings**: Body 200→60Hz (80ms), Click 4000→250Hz (12ms)

### Sub Kick
- **Character**: Deep, subby, minimal click
- **Effects Enabled**: Distortion (Tube 5%), Multiband Comp (sub-focused), Exciter (20%), Transient Shaper (20%/10%)
- **Best For**: Dubstep, trap, deep house
- **Settings**: Body 120→40Hz (200ms), Click 1500→100Hz (25ms), longer decay

**Tip:** Load a preset and adjust to taste - presets are starting points!

---

## Workflow Tips

### Creating Your First Kick

1. **Start Clean**: Load the default patch (all effects OFF)
2. **Set Body Pitch**: 
   - Low kicks (sub): 120Hz → 40Hz
   - Mid kicks (808): 150Hz → 55Hz
   - High kicks (techno): 200Hz → 70Hz
3. **Add Click**: Start with Level 30%, adjust to taste
4. **Shape Envelope**: Attack 0.5ms, Decay 300ms, Sustain 0%, Release 50ms
5. **Filter**: Start at 8000Hz cutoff, adjust down for darker sound
6. **Add Effects**: Enable one at a time to hear what each does
7. **Compress**: Enable multiband, tweak Sub/Body bands
8. **Polish**: Add exciter for air, transient shaper for punch

### Troubleshooting

| Problem | Solution |
|---------|----------|
| **Kick too quiet** | Raise Master Level, check Velocity Sensitivity (should be 50-100%) |
| **No low end** | Lower Body Osc End Pitch (40-60Hz), disable high-pass filtering in DAW |
| **Muddy/boomy** | Lower Filter Cutoff, reduce multiband Sub Gain, shorter Decay |
| **Weak attack** | Raise Click Level, shorter Attack time (0.2-0.5ms), enable Transient Shaper |
| **Too clicky** | Lower Click Level, longer Click Decay, reduce Exciter Mix |
| **Distorted output** | Lower Master Level, disable Clipper, reduce Distortion Amount |
| **Effects not working** | Check that effect Enable ✓ checkbox is ON |

### Mixing Tips

- **Leave headroom**: Master Level 80% is a safe starting point
- **Layer carefully**: Use EQ to separate from bass/other kicks
- **Sidechain**: Use your DAW's sidechain compression for kick-bass ducking
- **Automation**: Automate Filter Cutoff, Distortion Amount for variation
- **MIDI velocity**: Use Velocity Sensitivity 70-100% for dynamic performances

---

## Technical Specifications

- **Sample Rate**: 44.1-192 kHz supported
- **Bit Depth**: 32-bit float processing
- **Polyphony**: Monophonic (last-note priority)
- **Latency**: ~1ms (<64 samples at 44.1kHz)
- **CPU Usage**: <5% (Apple Silicon M1, 44.1kHz)
- **Oversampling**: 4× internal (176.4kHz for 44.1kHz output)
- **Anti-Aliasing**: 20-tap Kaiser FIR filter (β=8.5)
- **Format**: CLAP plugin (requires CLAP-compatible DAW)

**Supported DAWs:**
- Bitwig Studio 3.0+
- Reaper 6.29+
- FL Studio 21.2+
- Cubase 12+
- Studio One 6+

---

## Keyboard Shortcuts

*(DAW-dependent, check your DAW's CLAP plugin keyboard mapping)*

- **Click checkbox**: Toggle effect enable/disable
- **Click + Drag knob**: Adjust parameter value
- **Shift + Drag knob**: Fine adjustment (10× precision)
- **Double-click knob**: Reset to default value
- **Ctrl/Cmd + Click knob**: Enter numeric value (if DAW supports)

---

## DAW Integration

### Automation
All parameters support **DAW automation**, including effect enable checkboxes. Checkboxes respond to:
- On-screen mouse clicks
- Host automation (0.0 = OFF, 1.0 = ON)
- MIDI CC mapping (via DAW's MIDI learn)

### Presets
- **Save in DAW**: Use your DAW's preset system to save custom settings
- **Factory Reset**: Reload plugin or select "Initialize" in DAW to get default sound

### MIDI Setup
1. Create an instrument track
2. Load DSynth Kick
3. Route MIDI from controller/sequencer to track
4. Play notes (C4 recommended as reference pitch)
5. Use MIDI velocity for dynamics (if Velocity Sensitivity > 0%)

---

## FAQ

**Q: Why are all effects off by default?**  
A: Transparency! You know exactly what's shaping your sound. Enable only what you need.

**Q: What's the difference between Body and Click oscillators?**  
A: Body = fundamental tone (low frequency sweep). Click = transient/attack (high frequency burst).

**Q: Do I need multiband compression?**  
A: No! Start simple. Add multiband if you need frequency-specific control.

**Q: Why doesn't key tracking work on pitch envelopes?**  
A: It does! Key tracking scales the entire pitch envelope. At 100%, C5 plays 2× the pitch of C4, preserving the sweep ratio.

**Q: Can I use this for bass sounds?**  
A: Yes! Set longer Decay (1000ms+), enable Key Tracking (80-100%), and play melodically.

**Q: How do I get a classic 808 sound?**  
A: Load the "808" preset. Or manually: Body 150→55Hz, Click 3000→200Hz, Decay 300ms, Soft distortion 10%.

**Q: What sample rate should I use?**  
A: 44.1kHz or 48kHz is fine. The plugin uses 4× oversampling internally for pristine quality.

**Q: Can I layer multiple kicks?**  
A: Yes! Load multiple instances of DSynth Kick on separate tracks. Pan/EQ to taste.

---

## Support & Resources

- **GitHub**: [github.com/your-repo/dsynth](https://github.com) *(replace with actual repo)*
- **Issues**: Report bugs via GitHub Issues
- **Manual**: This document (`KICK_USER_GUIDE.md`)
- **Build Instructions**: See `BUILD_AND_DISTRIBUTE.md`

**Version**: 0.3.0  
**License**: See LICENSE file  
**Author**: DSynth Development Team

---

## Changelog

### v0.3.0 (Current)
- ✓ Checkbox UI for all boolean parameters (Enable/Bypass)
- ✓ All effects default to OFF for transparency
- ✓ Added enable switches: Distortion, Exciter, Transient Shaper
- ✓ GUI scrolling support (baseview backend)

### v0.2.0
- Added key tracking (0-100% chromatic pitch scaling)
- Added transient shaper effect
- Added clipper for brick-wall limiting
- GUI improvements

### v0.1.0
- Initial release
- Dual oscillators with pitch envelopes
- Filter with envelope modulation
- Distortion (4 types)
- Multiband compression (3 bands)
- Exciter effect

---

**Happy kick crafting! 🥁**
