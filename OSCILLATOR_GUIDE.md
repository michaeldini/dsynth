# DSynth Oscillator User Guide

## Overview

DSynth features **three independent oscillators** per voice, each with identical parameters and capabilities. The oscillators are the core sound generation components of the synthesizer, producing the raw audio signal that is then shaped by filters, envelopes, and effects.

Each oscillator uses **4× oversampling with anti-aliasing** to produce smooth, analog-quality waveforms even at high frequencies, preventing the harsh digital artifacts (aliasing) that plague many digital synthesizers.

---

## Master Section

The Master section contains global controls that affect the entire synthesizer, including playback mode, velocity sensitivity, and the master output level.

### Master Gain
**Range:** 0.0 to 2.0 (0% to 200%)  
**Default:** 1.0 (100%)

**What it does:** Controls the final output level of the synthesizer before the signal leaves to your DAW or audio interface. This is the last gain stage in the signal chain.

**How it changes the sound:**
- **0.0** - Complete silence
- **0.5** - Half volume, -6dB quieter
- **1.0** - Unity gain (100%), no change to volume
- **1.5** - 50% louder, use for hot output
- **2.0** - Double volume (+6dB), use cautiously to avoid clipping

**Usage tips:**
- Keep at 1.0 for most patches and adjust mixer/DAW faders instead
- Reduce to 0.5-0.7 if your sound is clipping the output
- Increase to 1.2-1.5 for quiet patches that need more presence
- This is NOT the same as velocity sensitivity—it affects all notes equally
- Does not affect the synth's internal dynamics, only the final output

---

### Monophonic (Mono)
**Type:** Checkbox (on/off)  
**Default:** Off (polyphonic mode)

**What it does:** Switches between polyphonic (multiple notes at once) and monophonic (single note) playback modes. When enabled, only one note can play at a time.

**How it changes the sound:**
- **Off (Polyphonic)** - Multiple notes can play simultaneously (up to 16 voices). Chords are possible.
- **On (Monophonic)** - Only one note plays at a time. When you press a new key, the previous note stops. Implements **last-note priority**: when you release a key while holding others, the synthesizer automatically plays the most recently pressed held note without retriggering the envelope.

**Usage tips:**
- Enable for classic monophonic synth bass and lead sounds
- Monophonic mode with legato (last-note priority) is great for smooth, connected melodies
- When you slide between notes in mono mode, the pitch changes but the envelope doesn't retrigger (legato playing)
- Polyphonic mode is better for pads, chords, and complex harmonies
- Mono mode saves CPU since only one voice is active at a time

---

### Hard Sync
**Type:** Checkbox (on/off)  
**Default:** Off

**What it does:** Enables oscillator hard sync, where Oscillator 1 acts as the "master" and forces Oscillators 2 and 3 (the "slaves") to restart their waveforms whenever Osc 1's waveform completes a cycle. This creates aggressive harmonic overtones and timbral changes when oscillators are detuned.

**How it changes the sound:**
- **Off** - Oscillators run independently, no synchronization
- **On** - Osc 2 and Osc 3 are "slaved" to Osc 1. Every time Osc 1's phase wraps from 1.0 back to 0.0, the slave oscillators' phases are reset to 0.0.

**Sonic characteristics when enabled:**
- When all oscillators are tuned to the same pitch: minimal effect
- When Osc 2/3 are detuned from Osc 1: creates harsh, metallic overtones and formant-like resonances
- The effect is more pronounced with larger pitch differences
- Classic sound heard in vintage analog leads and aggressive bass tones

**Usage tips:**
- Enable Hard Sync, then detune Osc 2 by +7 to +12 semitones for classic sync lead sounds
- Automate the pitch of Osc 2/3 while Hard Sync is on for dramatic sweeping timbral changes
- Works best with bright waveforms (saw, square) for maximum harmonic content
- Combine with filter sweeps for evolving, aggressive textures
- Use subtly (small pitch differences) for slight harmonic enhancement, or extreme (large pitch differences) for aggressive digital tones

---

### Velocity → Amp (Vel→Amp)
**Range:** 0.0 to 1.0 (0% to 100% sensitivity)  
**Default:** 0.5 (50%)

**What it does:** Controls how much the MIDI velocity (how hard you hit a key) affects the note's amplitude/volume. Higher values make the synth more velocity-sensitive.

**How it changes the sound:**
- **0.0** - No velocity sensitivity; all notes play at the same volume regardless of how hard you hit the key
- **0.5** - Moderate sensitivity (default); soft notes are quieter, hard notes are louder, but not extreme
- **1.0** - Full velocity sensitivity; soft notes (velocity ~32) can be very quiet, hard notes (velocity ~127) are very loud

**Formula:** `output_amplitude = 1.0 + sensitivity × (velocity - 0.5)`
- At MIDI velocity 64 (50%): no change regardless of sensitivity setting
- At MIDI velocity 127 (100%): louder by `0.5 × sensitivity`
- At MIDI velocity 0 (0%): quieter by `0.5 × sensitivity`

**Usage tips:**
- Set to 0.0 for electronic, consistent sounds where dynamics don't matter (techno bass, arpeggios)
- Use 0.3-0.6 for balanced expressiveness without extreme volume differences
- Set to 0.8-1.0 for highly expressive playing where you want maximum dynamic range (piano-like response)
- Combine with Velocity → Filter for full velocity-expressive patches
- Lower sensitivity if quiet notes feel too weak or disappear in the mix

---

### Velocity → Filter (Vel→Flt)
**Range:** 0.0 to 1.0 (0% to 100% sensitivity)  
**Default:** 0.5 (50%)

**What it does:** Controls how much the MIDI velocity affects the filter cutoff frequency. Higher velocity = brighter sound (higher cutoff), lower velocity = darker sound (lower cutoff).

**How it changes the sound:**
- **0.0** - No velocity sensitivity; filter cutoff stays constant regardless of velocity
- **0.5** - Moderate sensitivity; harder notes open the filter more, making them brighter
- **1.0** - Full sensitivity; soft notes can be very dark/muffled, hard notes very bright/open

**Formula:** `cutoff_offset = sensitivity × (velocity - 0.5)`
- The offset is added to the filter's base cutoff frequency
- At MIDI velocity 64: no change to cutoff
- At MIDI velocity 127: filter opens up (brighter)
- At MIDI velocity 0: filter closes down (darker)

**Usage tips:**
- Set to 0.0 for static timbral character regardless of playing dynamics
- Use 0.4-0.7 for natural, expressive filter response that mimics acoustic instruments
- High values (0.8-1.0) create dramatic timbral changes between soft and hard playing
- Combine with resonance for more pronounced filter sweeps on harder hits
- Lower sensitivity if soft notes sound too muffled or lose definition
- Essential for expressive leads, basses, and pads that respond to playing dynamics

---

### Randomize Button (🎲 Randomize)
**What it does:** Generates a completely random patch by randomizing all synthesis parameters. This is a creative sound design tool for discovering new sounds and happy accidents.

**How it changes the sound:**
- Randomizes waveforms, pitches, detuning, filters, LFOs, effects, and more
- Each click generates a completely different sound
- Some randomized patches will be musical, others experimental or chaotic

**Usage tips:**
- Click repeatedly until you find an interesting starting point
- Use as creative inspiration when stuck in a sound design rut
- Save any interesting random patches as presets before randomizing again
- You'll often get extreme settings—tweak them back to taste after randomizing
- Great for discovering parameter combinations you wouldn't have tried manually
- Randomization respects parameter ranges but not musical conventions—expect surprises!

---

## Envelope Section

The Envelope section controls the amplitude envelope (volume shape) of each note over time using the classic ADSR (Attack, Decay, Sustain, Release) model. This is one of the most important aspects of sound design, as it determines how a sound evolves from the moment you press a key until the sound fades away.

The envelope is visualized with an interactive editor that shows the four stages graphically.

### Attack
**Range:** 0.001 to 5.0 seconds (1ms to 5s)  
**Default:** 0.001 seconds (1ms)

**What it does:** Controls how long it takes for the sound to reach full volume after you press a key. This is the "rise time" from silence to peak amplitude.

**How it changes the sound:**
- **0.001s (1ms)** - Instant, percussive attack; sound reaches full volume immediately (drums, plucks, stabs)
- **0.01-0.05s** - Very fast attack with slight softness (piano, electric piano, mallet percussion)
- **0.1-0.3s** - Noticeable fade-in, slower attack (soft pads, strings)
- **0.5-1.0s** - Slow swell, gradual fade-in (cinematic pads, ambient textures)
- **2.0-5.0s** - Very slow attack, reverse-like effect (experimental pads, evolving textures)

**Usage tips:**
- Keep at 1-5ms for percussive, punchy sounds (bass, leads, plucks)
- Use 50-200ms for soft, gentle attacks (pads, strings, choirs)
- Automate attack time for evolving sounds (fast attack in chorus, slow attack in verses)
- Long attack times hide transients—useful for smooth, non-percussive textures
- Combine with transient shaper for more control over the attack character

---

### Decay
**Range:** 0.001 to 5.0 seconds (1ms to 5s)  
**Default:** 0.1 seconds (100ms)

**What it does:** Controls how long it takes for the sound to fall from peak volume (after the attack phase) down to the sustain level. This is the "initial fade" after the attack.

**How it changes the sound:**
- **0.001-0.01s** - Instant drop to sustain level, no decay phase (sustained organs, pads)
- **0.05-0.2s** - Quick decay, typical for plucks and short percussion (electric piano, mallets)
- **0.3-0.8s** - Moderate decay, sound has a noticeable "tail" after the attack (synth brass, bells)
- **1.0-5.0s** - Slow decay, long tail before reaching sustain (evolving pads, long bells)

**Usage tips:**
- Short decay (10-50ms) for punchy, immediate sounds
- Medium decay (100-300ms) for natural instrument-like response
- Long decay (1-2s) for slowly evolving textures and ambient sounds
- If sustain is at 0.0, decay becomes the total note length (percussive envelope)
- Combine short decay with moderate sustain for classic pluck sounds
- Longer decay with low sustain creates "struck" sounds (piano, bells, glockenspiel)

---

### Sustain
**Range:** 0.0 to 1.0 (0% to 100%)  
**Default:** 0.9 (90%)

**What it does:** Controls the volume level that the sound holds at while a key is pressed, after the attack and decay phases complete. Unlike the other three parameters (which are times), sustain is a **level** (amplitude).

**How it changes the sound:**
- **0.0** - No sustain; sound fades to silence after decay (percussion, plucks, struck sounds)
- **0.3-0.5** - Low sustain; sound fades significantly but doesn't disappear (piano, guitars)
- **0.7-0.9** - High sustain; sound stays strong while key is held (organs, synth leads, sustained pads)
- **1.0** - Full sustain; sound stays at peak volume indefinitely (classic organ, no decay)

**Usage tips:**
- Set to 0.0 for percussive, "struck" sounds that don't hold (drums, plucks, short stabs)
- Use 0.3-0.6 for natural decay behavior (piano, mallets, most acoustic instruments)
- Set to 0.8-1.0 for sustained sounds (organs, pads, leads, basses)
- High sustain (0.9) is the default for typical subtractive synthesis patches
- Combine low sustain (0.0-0.3) with long decay for bell-like, fading tones
- If sustain = 1.0 and attack = 0.001s, you get a classic organ-style instant-on, sustained envelope

---

### Release
**Range:** 0.001 to 5.0 seconds (1ms to 5s)  
**Default:** 0.2 seconds (200ms)

**What it does:** Controls how long it takes for the sound to fade to silence after you release a key. This is the "tail" or "fade-out" time after you let go.

**How it changes the sound:**
- **0.001-0.01s (1-10ms)** - Instant cutoff when key is released; no tail (stabs, gated sounds, chiptune)
- **0.05-0.2s** - Quick fade-out, slight natural tail (plucks, short percussion)
- **0.3-0.8s** - Moderate release, noticeable fade-out (most synth sounds, typical default)
- **1.0-2.0s** - Long release, sound lingers after key release (pads, ambient, reverb-like)
- **3.0-5.0s** - Very long release, sound continues for seconds (cinematic pads, drones)

**Usage tips:**
- Very short release (1-20ms) for chopped, staccato sounds
- Short release (50-200ms) for natural, realistic instrument behavior
- Medium release (200-500ms) is typical for most synth sounds (default 200ms)
- Long release (1-3s) for smooth, overlapping pad sounds that blend together
- Extremely long release (3-5s) for ambient textures and sound design
- Release time is crucial for playing style: short for staccato, long for legato
- Long release can cause voices to "stack up" and create polyphonic buildup

---

### Envelope Editor (Visual)
**What it does:** The envelope editor provides a visual representation of the ADSR envelope shape. You can see how the attack, decay, sustain, and release stages combine to create the overall amplitude contour.

**How to use it:**
- The graph shows time (horizontal axis) vs. amplitude (vertical axis)
- **Attack** - The rising slope from 0 to peak
- **Decay** - The falling slope from peak to sustain level
- **Sustain** - The horizontal line where the envelope holds while a key is pressed
- **Release** - The falling slope from sustain to silence after key release

**Usage tips:**
- Use the visual editor to understand the envelope shape at a glance
- Adjust the knobs below and watch the graph update in real-time
- The visual representation helps predict how percussive or sustained a sound will be
- A steep attack with high sustain = punchy lead; a slow attack with high sustain = pad swell

---

## Voice Compressor Section

The Voice Compressor applies per-voice dynamic range compression **before** the signal enters the master effects chain. This is optimized for transient control and can make sounds punchier or more controlled. Unlike a typical master compressor, this processes each voice individually, preventing one loud voice from ducking others.

### On (Enabled)
**Type:** Checkbox (on/off)  
**Default:** Off

**What it does:** Enables or disables the voice-level compressor. When off, the compressor has no effect on the signal.

**Usage tips:**
- Enable for tighter, more controlled dynamics
- Leave off if you want natural, uncompressed dynamics or plan to compress in your DAW
- Great for taming overly dynamic patches or adding punch

---

### Threshold (Thresh)
**Range:** -60.0 to 0.0 dB  
**Default:** -12.0 dB

**What it does:** Sets the level above which compression begins. Signals louder than the threshold are compressed according to the ratio; signals below the threshold pass through unaffected.

**How it changes the sound:**
- **-60.0 dB** - Extremely low threshold; almost all signal is compressed (heavy compression)
- **-30.0 dB** - Low threshold; moderate signals are compressed
- **-12.0 dB** - Higher threshold (default); only louder peaks are compressed
- **-6.0 dB** - High threshold; only the loudest transients are compressed
- **0.0 dB** - Maximum threshold; almost no signal is compressed (compressor barely active)

**Usage tips:**
- Set lower (-30 to -20 dB) for overall dynamic control and leveling
- Set higher (-12 to -6 dB) for transient control and peak limiting only
- If you want aggressive, pumping compression, use low threshold + high ratio
- If you want transparent dynamics, use moderate threshold + low ratio
- Watch for over-compression: if threshold is too low with high ratio, the sound can become lifeless

---

### Ratio
**Range:** 1.0 to 20.0 (1:1 to 20:1)  
**Default:** 3.0 (3:1)

**What it does:** Controls how much the signal is compressed once it exceeds the threshold. Ratio is the relationship between input level change and output level change.

**How it changes the sound:**
- **1.0 (1:1)** - No compression; signal passes through unchanged
- **2.0-4.0 (2:1 to 4:1)** - Gentle to moderate compression (transparent, natural)
- **5.0-8.0 (5:1 to 8:1)** - Strong compression (obvious effect, controlled dynamics)
- **10.0-20.0 (10:1 to 20:1)** - Extreme compression/limiting (heavily squashed, pumping)

**Formula:** For every X dB the input exceeds the threshold, the output increases by 1 dB.
- At 2:1 ratio: 4 dB input over threshold → 2 dB output over threshold
- At 10:1 ratio: 10 dB input over threshold → 1 dB output over threshold

**Usage tips:**
- Use 2-4:1 for subtle, transparent control (most musical applications)
- Use 5-8:1 for noticeable compression (punchy drums, tight bass)
- Use 10-20:1 for limiting-style compression (heavy control, EDM, aggressive sounds)
- Higher ratio = less dynamic range, more "squashed" sound
- Combine with makeup gain to compensate for volume loss from compression

---

### Attack
**Range:** 0.1 to 50.0 milliseconds  
**Default:** 1.0 ms

**What it does:** Controls how quickly the compressor responds to signals exceeding the threshold. Fast attack means compression starts immediately; slow attack lets transients through before compressing.

**How it changes the sound:**
- **0.1-2.0 ms** - Very fast attack; catches even quick transients, makes sound duller/more controlled
- **3.0-10.0 ms** - Moderate attack; lets some transient punch through before compressing
- **15.0-30.0 ms** - Slow attack; preserves most transient character, compresses the body of the sound
- **40.0-50.0 ms** - Very slow attack; transients pass through completely uncompressed

**Usage tips:**
- Fast attack (0.5-3ms) for maximum control and limiting (bass, sustained sounds)
- Moderate attack (5-15ms) for balanced punch and control (most applications)
- Slow attack (20-50ms) to preserve transient punch while controlling sustain (drums, percussion)
- If the sound feels too dull or lifeless with compression, increase attack time
- For punchy bass, use fast attack to tame low-end boom
- For snappy leads, use slower attack to keep the initial bite

---

### Release
**Range:** 10.0 to 200.0 milliseconds  
**Default:** 50.0 ms

**What it does:** Controls how quickly the compressor stops compressing after the signal falls below the threshold. Fast release means compression releases quickly; slow release means it lingers longer.

**How it changes the sound:**
- **10.0-30.0 ms** - Fast release; compressor reacts quickly, can sound "pumping" or "breathing"
- **40.0-80.0 ms** - Moderate release; balanced, natural-sounding compression
- **100.0-200.0 ms** - Slow release; smooth, gentle compression that doesn't fluctuate rapidly

**Usage tips:**
- Fast release (10-30ms) for rhythmic, pumping compression (sidechain-style effect)
- Moderate release (40-80ms) for most musical applications (default 50ms works well)
- Slow release (100-200ms) for smooth, transparent compression without artifacts
- If you hear "pumping" or "breathing" (volume fluctuations), increase release time
- If compression feels sluggish or doesn't recover fast enough, decrease release time
- Match release time to the tempo/rhythm of your music for musical compression

---

### Knee
**Range:** 0.0 to 20.0 dB  
**Default:** 3.0 dB

**What it does:** Controls the transition smoothness around the threshold point. A hard knee (0 dB) means compression starts abruptly at the threshold; a soft knee means compression gradually increases as the signal approaches the threshold.

**How it changes the sound:**
- **0.0 dB** - Hard knee; compression starts abruptly at threshold (precise, digital)
- **3.0-6.0 dB** - Soft knee (default); smooth transition into compression (natural, analog-like)
- **10.0-20.0 dB** - Very soft knee; compression starts gently well below threshold (very smooth, transparent)

**Usage tips:**
- Hard knee (0 dB) for precise, obvious compression (limiting, gating)
- Soft knee (3-8 dB) for natural, musical compression (most applications)
- Very soft knee (10-20 dB) for transparent, unnoticeable compression
- Soft knees sound more "analog" and less harsh
- Use hard knee when you want compression to be an obvious effect
- Use soft knee when you want compression to be invisible and natural

---

### Makeup Gain (Makeup)
**Range:** 0.0 to 30.0 dB  
**Default:** 0.0 dB (no makeup gain)

**What it does:** Adds gain after compression to compensate for the volume reduction caused by compression. Since compression reduces peaks, the overall level often drops—makeup gain brings it back up.

**How it changes the sound:**
- **0.0 dB** - No makeup gain; compressed signal may be quieter than the input
- **6.0-12.0 dB** - Moderate makeup gain; compensates for typical compression
- **18.0-30.0 dB** - High makeup gain; compensates for heavy compression or brings up quiet signals

**Usage tips:**
- Adjust by ear to match the perceived loudness of the compressed signal to the uncompressed signal
- Rule of thumb: for every 3:1 ratio, you might need ~3-6 dB of makeup gain
- If the compressed sound feels too quiet, add makeup gain
- Be careful not to add too much—can cause clipping if you over-compensate
- Use your DAW's meters to match compressed/uncompressed levels for fair comparison
- Makeup gain doesn't affect the compression itself, only the output level

---

## Transient Shaper Section

The Transient Shaper manipulates the attack and sustain portions of the sound independently using envelope-following gain modulation. This is different from compression—instead of reducing loud signals, it boosts or cuts specific parts of the envelope to shape the sound's transient character.

### On (Enabled)
**Type:** Checkbox (on/off)  
**Default:** Off

**What it does:** Enables or disables the transient shaper. When off, the transient shaper has no effect.

**Usage tips:**
- Enable to enhance or reduce attack transients (punch)
- Great for making sounds more percussive or more smooth
- Pairs well with compression for detailed dynamic control

---

### Attack
**Range:** 0.0 to 1.0 (0% to 100% boost)  
**Default:** 0.0 (no boost)

**What it does:** Boosts the attack portion of the sound's envelope, making the initial transient louder and more pronounced. This adds "punch," "snap," or "bite" to the sound.

**How it changes the sound:**
- **0.0** - No attack boost; sound plays normally
- **0.3-0.5** - Moderate attack boost; adds noticeable punch and presence
- **0.6-0.8** - Strong attack boost; very snappy, aggressive transient
- **1.0** - Maximum attack boost (+100% gain); extremely punchy, can sound clicky

**Usage tips:**
- Use 0.2-0.4 for subtle punch enhancement (leads, bass, keys)
- Use 0.5-0.8 for aggressive, percussive character (drums, plucks, stabs)
- Use 1.0 for extreme transient emphasis (sound design, experimental)
- Great for making soft attack envelopes sound punchier without changing the envelope itself
- Combine with envelope attack time: fast attack + transient boost = ultra-sharp transient
- Can make sounds cut through a dense mix better
- Be careful with high values—can introduce clicks or distortion

---

### Sustain
**Range:** 0.0 to 1.0 (0% to 100% reduction)  
**Default:** 0.0 (no reduction)

**What it does:** Reduces the sustain portion of the sound's envelope, making the body of the sound quieter relative to the attack. This creates more separation between the transient and the sustain, emphasizing the attack even more.

**How it changes the sound:**
- **0.0** - No sustain reduction; sound plays normally
- **0.3-0.5** - Moderate sustain reduction; body is quieter, attack stands out more
- **0.6-0.8** - Strong sustain reduction; attack-heavy sound, short body
- **1.0** - Maximum sustain reduction (-100%); almost no body, only transient remains

**Usage tips:**
- Use 0.2-0.4 for subtle sustain shaping (making leads or bass tighter)
- Use 0.5-0.7 for short, plucky sounds (reducing sustain makes sounds more percussive)
- Use 0.8-1.0 for extreme transient-only effects (clicks, short stabs)
- Great for turning sustained sounds into plucks or shorter sounds
- Combine with high attack boost for ultra-percussive, transient-focused sounds
- Useful for creating rhythmic, gated effects without a separate gate effect
- Can help clean up muddy mixes by reducing overlapping sustains

**Combining Attack and Sustain:**
- **Attack 0.5 + Sustain 0.5** - Classic transient shaper effect: punchy attack, short body
- **Attack 0.8 + Sustain 0.8** - Extreme percussive effect: all attack, minimal sustain
- **Attack 0.3 + Sustain 0.0** - Adds punch without changing sustain length
- **Attack 0.0 + Sustain 0.7** - Reduces body without boosting attack (makes sounds shorter/tighter)

---

## Oscillator Parameters

### Waveform Selector
**What it does:** Selects the basic waveform shape that the oscillator generates.

**Available waveforms:**
- **Sine** - Pure fundamental tone with no harmonics. Smooth and clean, ideal for sub-bass, pads, or as a modulation source. Sounds like a tuning fork or flute.
  
- **Saw** - Bright, buzzy waveform containing all harmonics (1st, 2nd, 3rd, 4th...). Classic analog synthesizer sound used for leads, basses, and brass. Sounds aggressive and cutting.
  
- **Square** - Hollow, clarinet-like tone with only odd harmonics (1st, 3rd, 5th, 7th...). Great for woodwind-style sounds, retro video game tones, and reedy textures.
  
- **Triangle** - Softer than square with only odd harmonics but at reduced strength. Warmer and mellower, good for subtle pads, bells, or flute-like sounds.
  
- **Pulse** - Variable-width square wave. The pulse width (controlled by the Shape parameter) dramatically changes the harmonic content. Classic for bass and lead sounds.
  
- **White Noise** - Random signal containing all frequencies at equal energy. Used for percussion, wind effects, or adding air/breath to sounds.
  
- **Pink Noise** - Filtered noise with -3dB/octave rolloff, sounding more natural than white noise. Great for synthesizing natural sounds like ocean waves, wind, or snare drums.
  
- **Additive** - Custom waveform built from up to 8 harmonics (see Harmonics section below). Allows you to sculpt unique timbres by controlling individual harmonic amplitudes.
  
- **Wavetable** - Morphable wavetables loaded from the wavetable library. Allows smooth transitions between complex timbres (see Wavetable section below).

**How it changes the sound:** The waveform is the fundamental building block of your sound. It determines the harmonic content (which frequencies are present) and the basic tonal character. Start here when designing a new patch.

---

### Pitch
**Range:** ±24 semitones (±2 octaves)  
**Default:** 0 semitones (concert pitch)

**What it does:** Transposes the oscillator up or down by semitones (half-steps on a piano). This allows you to create interval relationships between oscillators or detune them by octaves.

**How it changes the sound:**
- **0 semitones** - Oscillator plays at the note you press on the keyboard
- **+12 semitones** - Oscillator plays one octave higher (doubling the frequency)
- **-12 semitones** - Oscillator plays one octave lower (halving the frequency)
- **+7 semitones** - Creates a perfect fifth interval above the root note (power chord effect)
- **+19 semitones** - Creates a perfect fifth one octave up (common for bright leads)

**Usage tips:**
- Set Osc 1 to 0, Osc 2 to +12, and Osc 3 to -12 for a classic three-octave spread bass/lead sound
- Use +7 or +5 semitones for instant chord sounds
- Detune by octaves (+12/-12) for massive, wide sounds without beating

---

### Detune
**Range:** ±50 cents  
**Default:** 0 cents

**What it does:** Fine-tunes the oscillator by cents (1 semitone = 100 cents). Creates subtle frequency shifts that add movement and width to the sound.

**How it changes the sound:**
- **0 cents** - Perfectly in tune, stable pitch
- **±5-15 cents** - Subtle beating/chorus effect, adds warmth and analog character
- **±20-35 cents** - More pronounced beating, creates slow oscillations in volume (amplitude modulation from interference)
- **±40-50 cents** - Significant detuning, creates intentional dissonance or "out of tune" effect

**Usage tips:**
- Set Osc 1 to -10 cents and Osc 2 to +10 cents for instant stereo width
- Use 0-7 cents for subtle analog drift simulation
- Combine with unison for massive supersaw-style leads
- Small detune values (2-5 cents) add life to otherwise static sounds without obvious pitch wobble

---

### Gain
**Range:** 0.0 to 1.0 (0% to 100%)  
**Default:** Osc 1 = 1.0, Osc 2 & 3 = 0.0 (off)

**What it does:** Controls the volume/amplitude of the oscillator in the mix. Acts as a simple mixer level for each oscillator.

**How it changes the sound:**
- **0.0** - Oscillator is completely silent (off)
- **0.5** - Oscillator at half volume
- **1.0** - Oscillator at full volume

**Usage tips:**
- Start with Osc 1 at full gain, then blend in Osc 2 and 3 to taste
- Use lower gain values (0.3-0.5) when stacking multiple oscillators to prevent clipping
- Automate gain for swells and volume envelope effects beyond the standard ADSR
- If using multiple oscillators with unison, reduce gain on each to prevent overload

---

### Pan
**Range:** -1.0 (left) to 1.0 (right)  
**Default:** 0.0 (center)

**What it does:** Positions the oscillator in the stereo field. Each oscillator can be panned independently, creating wide stereo images.

**How it changes the sound:**
- **-1.0** - Oscillator completely in left speaker
- **0.0** - Oscillator centered (equal in both speakers)
- **+1.0** - Oscillator completely in right speaker
- **±0.5** - Moderately panned left or right, still present in both speakers

**Usage tips:**
- Pan Osc 1 slightly left (-0.3), Osc 2 center (0.0), and Osc 3 slightly right (+0.3) for natural width
- Hard pan (-1.0 and +1.0) for extreme stereo separation
- Keep bass/sub oscillators centered (0.0) for mono compatibility
- Combine with detune for animated stereo movement as detuned oscillators beat against each other

---

### Saturation (Sat)
**Range:** 0.0 to 1.0 (0% to 100%)  
**Default:** 0.0 (no saturation)

**What it does:** Applies analog-style soft clipping distortion to the oscillator output before it enters the filter. Adds harmonic richness and warmth.

**How it changes the sound:**
- **0.0** - Clean, uncolored oscillator output
- **0.3** - Subtle warmth and thickness, adds even-order harmonics
- **0.6** - Noticeable saturation, fuller sound with more body
- **1.0** - Heavy saturation approaching distortion, aggressive harmonic content

**Usage tips:**
- Use 0.2-0.4 for subtle analog warmth on bass and lead sounds
- Push to 0.6+ for aggressive, driven tones or to add presence to thin waveforms
- Saturation makes oscillators "sit" better in a mix by adding harmonic complexity
- Particularly effective on sine and triangle waves to add character
- Stacks with filter drive for even more saturation (apply both sparingly to avoid mud)

---

### FM Amount (FM Amt)
**Range:** 0.0 to 10.0  
**Default:** 0.0 (no FM)

**What it does:** Controls the depth of frequency modulation applied to this oscillator. The modulation source is selected using the **FM Source** button. FM creates complex, metallic, and bell-like timbres by using one oscillator to modulate the frequency of another.

**How it changes the sound:**
- **0.0** - No frequency modulation, oscillator plays normally
- **0.5-2.0** - Subtle harmonic enrichment, adds movement and character
- **3.0-6.0** - Noticeable FM effects: metallic, bell-like, or hollow tones
- **7.0-10.0** - Extreme FM: clangorous bells, aggressive digital timbres, chaotic overtones

**Usage tips:**
- Start with FM Amount at 1.0-2.0 and slowly increase to find sweet spots
- Classic electric piano: Use sine wave with FM Amount around 3-5
- Bell tones: Use sine or triangle modulating sine with FM Amount 6-8
- Subtle movement: FM Amount 0.5-1.5 adds gentle harmonic shift
- Combine with envelope modulation of FM Amount for evolving textures
- Higher pitched modulation sources create brighter, more complex FM tones

---

### FM Source Button
**What it does:** Selects which oscillator will frequency-modulate this oscillator. Click to cycle through options: **Off → Osc 1 → Osc 2 → Osc 3**.

**How it changes the sound:**
- **Off** - No FM applied regardless of FM Amount setting
- **Osc 1/2/3** - The selected oscillator's output modulates this oscillator's frequency

**Usage tips:**
- Common routing: Osc 2 modulates Osc 1 (set on Osc 1's FM Source to "Osc 2")
- For complex timbres, create FM chains: Osc 3 → Osc 2 → Osc 1
- Modulating oscillator's waveform affects FM character: sine = smooth, saw = harsh
- The modulator's pitch also matters: higher pitched modulators create brighter FM
- Avoid feedback loops (Osc 1 modulating Osc 2 while Osc 2 modulates Osc 1)

---

### Unison
**Range:** 1 to 7 voices  
**Default:** 1 (no unison)

**What it does:** Creates multiple copies (voices) of the oscillator, each slightly detuned and phase-offset from each other. This produces the classic "supersaw" or "unison" effect heard in trance leads and fat bass sounds.

**How it changes the sound:**
- **1** - Single voice, no unison effect (standard operation)
- **2-3** - Subtle thickening and width, good for pads and chords
- **4-5** - Clear unison character, chorusing effect becomes obvious
- **6-7** - Massive, wide "wall of sound" effect, maximum thickness

**Usage tips:**
- For supersaw leads, use 5-7 voices with unison detune at 20-40 cents
- For subtle analog drift, use 2-3 voices with low unison detune (5-10 cents)
- More voices = more CPU usage, but also bigger sound
- Unison affects all oscillators independently—can use different unison counts per osc
- Enable UNorm (unison normalize) to prevent clipping when using many voices

---

### Unison Detune (UDet)
**Range:** 0 to 100 cents  
**Default:** 20 cents

**What it does:** Controls how far apart the unison voices are detuned from each other. Higher values create more dramatic beating and movement.

**How it changes the sound:**
- **0 cents** - All unison voices in perfect tune (sounds like one voice, no effect)
- **5-15 cents** - Subtle chorus effect, gentle wavering
- **20-40 cents** - Classic unison sound, noticeable beating and width
- **50-70 cents** - Heavy detuning, slow volume oscillations, very wide
- **80-100 cents** - Extreme detuning approaching semitone differences, can sound out-of-tune

**Usage tips:**
- Default of 20 cents is a good starting point for most sounds
- For trance supersaws, push to 30-50 cents for maximum width
- Lower values (10-15 cents) work better for subtle thickening without obvious chorusing
- Combine with stereo panning for unison voices to spread across the stereo field
- Higher unison detune with more voices can cause phase cancellation—adjust to taste

---

### Shape
**Range:** -1.0 to 1.0  
**Default:** 0.0

**What it does:** Morphs or modulates the waveform character. The exact effect depends on the selected waveform (see Waveform parameter above).

**How it changes the sound by waveform:**

**Sine Wave:**
- **-1.0 to 0.0** - No effect (sine remains pure)
- **0.0 to 1.0** - Adds soft clipping distortion, introducing harmonics (brightening effect)
- Use positive values to add character to otherwise bland sine waves

**Pulse Wave:**
- **-1.0** - Very narrow pulse width (~10% duty cycle), thin and nasal
- **0.0** - Square wave (50% duty cycle), hollow and clarinet-like
- **+1.0** - Very wide pulse width (~90% duty cycle), thin and nasal (mirror of -1.0)
- Sweep through range for classic PWM (pulse width modulation) analog effects

**Sawtooth:**
- **-1.0** - Full sawtooth (bright, buzzy)
- **0.0** - Neutral sawtooth
- **+1.0** - Morphs toward triangle (mellower, less bright)

**Triangle:**
- **-1.0** - Morphs toward sawtooth (brighter, more aggressive)
- **0.0** - Pure triangle
- **+1.0** - Remains triangle (less effect in positive direction)

**Other waveforms:**
- Square, noise waveforms have minimal shape effect

**Usage tips:**
- Pulse Shape automation creates classic PWM sounds (moving between -0.5 and +0.5)
- Use Shape on sine waves to add harmonics without switching to a brighter waveform
- Experiment with LFO modulation of Shape for animated timbral movement

---

### Unison Normalize (UNorm)
**Type:** Checkbox (on/off)  
**Default:** On (enabled)

**What it does:** When enabled, automatically reduces the gain of each unison voice to prevent clipping when multiple voices are summed. When disabled, unison voices are summed at full volume for a thicker (but potentially clipping) sound.

**How it changes the sound:**
- **Enabled (checked)** - Safe, prevents clipping, maintains clean output even with 7 unison voices. Volume stays consistent regardless of unison count.
- **Disabled (unchecked)** - Louder, thicker sound with more "weight," but can distort if too many voices are active. Volume increases with unison count.

**Usage tips:**
- Keep enabled (default) for predictable, clean results
- Disable for "broken" or intentionally overdriven unison effects
- If the sound feels thin with unison enabled, reduce unison count or increase oscillator gain
- When disabled, you may need to lower the oscillator gain to compensate for the louder unison sum

---

### Solo
**Type:** Checkbox (on/off)  
**Default:** Off

**What it does:** When enabled on any oscillator, **only oscillators with Solo enabled will be heard**. All non-solo'd oscillators are muted. Useful for auditioning individual oscillators or focusing on specific layers.

**How it changes the sound:**
- **Off** - Oscillator plays normally in the mix with other oscillators
- **On** - Mutes all non-solo'd oscillators; only solo'd oscillators are audible

**Usage tips:**
- Solo Osc 1 to hear just the fundamental tone
- Solo Osc 2 and 3 together to hear just the layered oscillators without Osc 1
- Great for debugging sounds: solo each oscillator to identify which one has unwanted character
- Remember to turn Solo off when done—easy to forget and wonder why your sound is thin!

---

## Additive Synthesis (Harmonics Section)

**Visible when:** Waveform is set to "Additive"

The harmonics section allows you to build custom waveforms by controlling the amplitude of individual harmonics (overtones). This is additive synthesis: combining sine waves at integer multiples of the fundamental frequency.

### Harmonic Sliders (H1-H8)
**Range:** 0.0 to 1.0 (0% to 100% amplitude)  
**Defaults:** H1 = 1.0 (fundamental), H2-H8 = 0.0 (silent)

**What they do:**
- **H1** - Fundamental frequency (1×), the root pitch of the note
- **H2** - 2nd harmonic (2×), one octave above the fundamental
- **H3** - 3rd harmonic (3×), an octave and a fifth above
- **H4** - 4th harmonic (4×), two octaves above
- **H5** - 5th harmonic (5×), two octaves and a major third above
- **H6** - 6th harmonic (6×), two octaves and a fifth above
- **H7** - 7th harmonic (7×), two octaves and a minor seventh above
- **H8** - 8th harmonic (8×), three octaves above

**How they change the sound:**
- **H1 only** - Pure sine wave, smooth and mellow
- **H1 + H2** - Richer, fuller tone (common in natural instruments)
- **Odd harmonics (H1, H3, H5, H7)** - Hollow, clarinet-like (approximates square wave)
- **Even harmonics (H2, H4, H6, H8)** - Brighter, more present
- **All harmonics decreasing in amplitude** - Approximates sawtooth wave
- **High harmonics stronger (H5-H8)** - Bright, metallic, aggressive

**Usage tips:**
- Start with H1 at full, then add harmonics one at a time to hear their effect
- For warm pads, use H1 strong with gentle amounts of H2, H3, H4
- For bright leads, boost higher harmonics (H5-H8)
- For hollow/square-like sounds, use only odd harmonics
- The wavetable is automatically regenerated when you change harmonic amplitudes
- Automate harmonic amplitudes for evolving timbres (wavetable scanning effect)

---

## Wavetable Section

**Visible when:** Waveform is set to "Wavetable"

Wavetables are collections of single-cycle waveforms that can be smoothly morphed between, allowing for complex and evolving timbres beyond basic waveforms.

### Wavetable Selector
**Range:** 0 to N-1 (depends on loaded wavetables)  
**Default:** 0 (first wavetable in library)

**What it does:** Selects which wavetable to use from the wavetable library. Each wavetable contains multiple frames (sub-waveforms) that can be morphed between using the Position parameter.

**How it changes the sound:**
- Each wavetable has unique harmonic content and character
- Wavetables can range from subtle variations of classic waveforms to complex, evolving textures
- The exact sonic character depends on the specific wavetable loaded

**Usage tips:**
- Browse through wavetables to find interesting starting points
- Wavetables work best with Position modulation for evolving sounds
- Some wavetables are designed to be subtle (analog-style), others dramatic (digital/FM-style)
- Combine multiple oscillators using different wavetables for layered complexity

---

### Position
**Range:** 0.0 to 1.0  
**Default:** 0.0

**What it does:** Scans through the frames (sub-waveforms) within the current wavetable. At 0.0, you hear the first frame; at 1.0, you hear the last frame. Values in between smoothly crossfade between adjacent frames.

**How it changes the sound:**
- **0.0** - First frame of the wavetable
- **0.5** - Middle frame of the wavetable
- **1.0** - Last frame of the wavetable
- **Sweeping 0.0 → 1.0** - Morphs through all frames, creating evolving timbres

**Usage tips:**
- Manually sweep Position to preview the wavetable's evolution
- Modulate Position with an LFO for animated, constantly-shifting textures
- Use envelope modulation for sounds that evolve from attack to sustain
- Position can be automated for film-score style morphing pads
- Try small Position changes (0.1-0.3 range) for subtle movement
- Full Position sweeps (0.0 → 1.0) create dramatic timbral shifts

---

## Signal Flow Summary

The oscillator signal flows through the following stages:

1. **Waveform Generation** - Base waveform (sine, saw, etc.) or wavetable/additive synthesis
2. **Wave Shaping** - Shape parameter morphs/distorts the waveform
3. **Frequency Modulation** - FM Amount and FM Source modulate the frequency (if enabled)
4. **Unison Processing** - Multiple detuned voices stacked (if Unison > 1)
5. **Saturation** - Soft clipping distortion adds harmonics (if Sat > 0)
6. **Pan** - Oscillator positioned in stereo field
7. **Gain** - Final volume control before entering filters

---

## Sound Design Tips

### Fat Bass
- **Osc 1:** Saw at 0 semitones, Gain 1.0, Unison 3, UDet 15 cents, Sat 0.3
- **Osc 2:** Sine at -12 semitones (one octave down), Gain 0.8, centered
- **Osc 3:** Off (Gain 0.0)
- Use lowpass filter with moderate resonance

### Trance Supersaw Lead
- **Osc 1:** Saw at 0 semitones, Gain 0.7, Pan -0.3, Unison 7, UDet 35 cents
- **Osc 2:** Saw at +12 semitones, Gain 0.6, Pan +0.3, Unison 5, UDet 30 cents
- **Osc 3:** Saw at -12 semitones, Gain 0.5, centered, Unison 3, UDet 20 cents
- Enable UNorm on all, reduce gain if clipping

### Warm Pad
- **Osc 1:** Additive (H1=1.0, H2=0.5, H3=0.3, H4=0.2), Unison 2, UDet 8 cents
- **Osc 2:** Triangle at +7 semitones, Detune +5 cents, Gain 0.4, Unison 2
- **Osc 3:** Sine at 0 semitones, Gain 0.3, slow LFO on Gain for swells
- Use gentle saturation (0.1-0.2) for warmth

### FM Bell/Pluck
- **Osc 1:** Sine at 0 semitones, Gain 1.0, FM Source = Osc 2, FM Amount 4-6
- **Osc 2:** Sine at +12 semitones (modulator), Gain 0.0 (inaudible, just modulates)
- **Osc 3:** Triangle at +19 semitones, Gain 0.3 (adds sparkle), fast attack envelope
- Use bandpass filter with moderate resonance

### Detuned Analog Lead
- **Osc 1:** Saw at 0 semitones, Detune -8 cents, Gain 0.8, Pan -0.2, Sat 0.4
- **Osc 2:** Saw at 0 semitones, Detune +8 cents, Gain 0.8, Pan +0.2, Sat 0.4
- **Osc 3:** Pulse (Shape 0.2 with LFO), Gain 0.5, centered
- Classic vintage sound with stereo width

---

## Performance Considerations

- Each unison voice increases CPU usage linearly (7 voices = 7× the processing)
- Using wavetables or additive synthesis is slightly more CPU-intensive than basic waveforms
- Saturation adds minimal CPU overhead
- FM synthesis is efficient—use it freely
- If CPU is a concern, reduce unison counts on oscillators that don't need massive width

---

## Next Steps

After understanding the oscillators, explore:
- **Filters** - Shape the harmonic content and tone of the oscillators
- **Envelopes** - Control how the sound evolves over time (attack, decay, sustain, release)
- **LFOs** - Add movement and modulation to parameters
- **Effects** - Add reverb, delay, chorus, and more for polished, professional sounds

The oscillators are just the starting point—the real magic happens when you combine them with modulation, filtering, and effects!
