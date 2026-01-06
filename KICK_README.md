# DSynth Kick - Specialized Kick Drum Synthesizer

A focused, monophonic kick drum synthesizer built from the DSynth codebase. Optimized specifically for creating punchy, professional kick drum sounds.

## Features

- **Dual Oscillator Design**: Body oscillator + click/transient oscillator
- **Pitch Envelopes**: Independent exponential pitch decay for each oscillator
- **Simple Amplitude Envelope**: Attack, Decay, Sustain (fixed at 0), Release
- **Lowpass Filter**: With envelope modulation
- **Built-in Distortion**: Soft, Hard, Tube, and Foldback saturation
- **Velocity Sensitivity**: Control how much MIDI velocity affects output
- **Monophonic**: One kick at a time for CPU efficiency
- **Low CPU Usage**: Significantly lighter than the full polyphonic synth

## Building

### Standalone Application
```bash
# Build standalone kick synth
cargo build --release --bin dsynth-kick --features kick-synth --no-default-features

# Run directly
cargo run --release --bin dsynth-kick --features kick-synth --no-default-features

# Create macOS app bundle
./bundle_kick_standalone.sh
```

### Running Tests
```bash
# Run kick synth tests
cargo test --features kick-synth --no-default-features --lib kick
```

## Presets

The kick synth comes with three built-in presets accessible via the GUI:

### 808
Classic TR-808 style kick drum with:
- Deep pitch sweep (180Hz → 50Hz)
- Moderate decay (400ms)
- Subtle distortion
- Low click presence

### Techno
Hard, punchy techno kick with:
- Faster pitch sweep (200Hz → 60Hz)
- Short decay (200ms)
- More prominent click
- Harder distortion for punch

### Sub
Deep sub-bass kick with:
- Low pitch sweep (120Hz → 40Hz)
- Long decay (600ms)
- Minimal click
- Very subtle distortion

## Parameters

### Body Oscillator (Osc 1)
- **Start Pitch**: Initial frequency (typically 100-400Hz)
- **End Pitch**: Final resting frequency (typically 40-80Hz)
- **Pitch Decay**: Time for pitch envelope (10-500ms)
- **Level**: Body oscillator volume (0.0-1.0)

### Click Oscillator (Osc 2)
- **Start Pitch**: Initial frequency (typically 1000-8000Hz)
- **End Pitch**: Final frequency (typically 100-500Hz)
- **Pitch Decay**: Time for click decay (5-100ms)
- **Level**: Click oscillator volume (0.0-1.0)

### Amplitude Envelope
- **Attack**: Initial attack time (0.1-10ms)
- **Decay**: Main decay time (50-2000ms)
- **Sustain**: Always 0 (kicks are percussive)
- **Release**: Tail release time (10-500ms)

### Filter
- **Cutoff**: Lowpass cutoff frequency (50-20000Hz)
- **Resonance**: Filter resonance (0.0-1.0)
- **Env Amount**: Filter envelope modulation (-1.0 to 1.0)
- **Env Decay**: Filter envelope decay time (10-500ms)

### Distortion
- **Amount**: Saturation amount (0.0-1.0)
- **Type**: Soft, Hard, Tube, or Foldback

### Master
- **Level**: Master output level (0.0-1.0)
- **Velocity Sensitivity**: MIDI velocity response (0.0-1.0)

## Architecture

The kick synth shares core DSP modules with the main synth (oscillators, filters, envelopes) but uses a simplified synthesis architecture:

```
MIDI Input
    ↓
KickEngine (monophonic)
    ↓
KickVoice (single voice)
    ├─ Osc 1 (body) → Pitch Envelope
    ├─ Osc 2 (click) → Pitch Envelope
    ├─ Mixer
    ├─ Amplitude Envelope
    ├─ Filter → Filter Envelope
    └─ Distortion
    ↓
Audio Output
```

### Key Differences from Main Synth
- **Monophonic** vs Polyphonic (16 voices)
- **2 oscillators** vs 3 oscillators per voice
- **Pitch envelopes** instead of static pitch
- **No LFOs** - optimized for percussive sounds
- **No effects chain** - built-in distortion only
- **Simpler parameter set** - focused on kick drums

## Use Cases

- **Live Performance**: Trigger kicks from MIDI pads or keyboards
- **DAW Integration**: Future CLAP plugin for use in production
- **Sound Design**: Create custom kick drum sounds
- **Learning**: Study synthesized percussion synthesis techniques

## CPU Performance

The kick synth is extremely efficient:
- **Single voice**: ~1-2% CPU at 44.1kHz (vs ~11% for 16-voice polyphonic synth)
- **No voice allocation overhead**: Direct MIDI → voice triggering
- **Optimized for percussive sounds**: No sustain, simple envelopes

## Future Enhancements

Planned features for future versions:
- CLAP plugin version for DAW integration
- More distortion algorithms
- Sub-oscillator for extra low end
- Noise generator for transient enhancement
- Preset management and saving
- MIDI CC parameter automation
- Enhanced GUI with real-time waveform display

## Technical Details

- **Sample Rate**: 44.1kHz default (configurable)
- **Bit Depth**: 32-bit floating point internally
- **Latency**: Minimal (~1-3ms depending on audio driver)
- **Pitch Envelope**: Exponential decay using formula: `f(t) = f_end + (f_start - f_end) * e^(-t/τ)`
- **Filter**: Biquad lowpass with Audio EQ Cookbook coefficients
- **Distortion**: Waveshaping algorithms with automatic makeup gain

## License

Same as main DSynth project.

## Credits

Built on the DSynth audio engine by [your name].
Inspired by classic drum machines: TR-808, TR-909, and modern synthesized kicks from Kick 2, Sonic Academy, etc.
