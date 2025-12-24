# Quick Start Guide

## Running DSynth

```bash
# Start the synthesizer
cargo run --release
```

The application will:
1. Initialize the audio output (you'll see "✓ Audio output started at 44100 Hz")
2. Attempt to connect to MIDI (optional - will show warning if no MIDI device)
3. Launch the GUI

## GUI Controls

### Oscillator Sections (3 identical panels)

Each oscillator has:
- **Waveform** dropdown: Choose Sine, Saw, Square, or Triangle
- **Pitch** slider: Transpose in semitones (-24 to +24)
- **Detune** slider: Fine-tune in cents (-50 to +50)
- **Gain** slider: Volume level (0.0 to 1.0)
- **Unison** slider: Number of stacked voices (1-7)
- **Unison Detune** slider: Spread amount in cents (0-100) for massive width

### Filter Sections (one per oscillator)

Each filter has:
- **Filter Type** dropdown: Lowpass, Highpass, or Bandpass
- **Cutoff** slider: Cutoff frequency (20 Hz to 20,000 Hz)
- **Resonance** slider: Q factor (0.5 to 50.0) - higher values for screaming acid sounds

### Master Controls

- **Master Gain**: Overall output level
- **PANIC** button: Stop all playing notes immediately

## MIDI Input

If you have a MIDI controller connected:
1. It will be automatically detected on startup
2. Play notes to hear the synthesizer
3. Velocity sensitivity is supported
4. All 128 MIDI notes are supported

## Computer Keyboard

The synthesizer now supports playing notes via computer keyboard!

**Keyboard Layout:**
- **AWSEDFTGYHUJKOLP** - Piano keys from C4 to D#5 (chromatic scale)
  - A = C4, W = C#4, S = D4, E = D#4, D = E4, F = F4, etc.
- **ZXCVBNM** - Lower octave (C3 to B3)
  - Z = C3, X = D3, C = E3, V = F3, B = G3, N = A3, M = B3

**Usage:**
- Press and hold keys to play notes
- Release keys to stop notes
- Multiple keys can be pressed simultaneously for chords
- Fixed velocity (0.8) for keyboard notes
- Use PANIC button to stop all notes if needed

## Tips

### Getting Started Sounds

**Pad Sound:**
- Oscillator 1: Sine wave, 0 pitch, 0 detune, gain 0.8
- Oscillator 2: Saw wave, +12 pitch, +5 detune, gain 0.5
- Oscillator 3: Off (gain 0)
- Filter 1: Lowpass, 800 Hz, resonance 2.0
- Filter 2: Lowpass, 1200 Hz, resonance 1.5

**Lead Sound:**
- Oscillator 1: Saw wave, 0 pitch, 0 detune, gain 0.7
- Oscillator 2: Saw wave, 0 pitch, +10 detune, gain 0.7
- Oscillator 3: Square wave, +12 pitch, 0 detune, gain 0.4
- Filters: Lowpass, 2000 Hz, resonance 4.0

**Bass Sound:**
- Oscillator 1: Saw wave, -12 pitch, 0 detune, gain 1.0
- Oscillator 2: Square wave, -12 pitch, 0 detune, gain 0.8
- Oscillator 3: Off
- Filters: Lowpass, 400 Hz, resonance 1.0

### Performance Tips

- The synthesizer supports 16 simultaneous voices
- Voice stealing uses "quietest voice" algorithm - play loud notes to guarantee they get a voice
- CPU usage is typically <11% for full polyphony
- If you hear glitches, check system audio buffer settings

### Troubleshooting

**No sound:**
- Check system audio output is working
- Verify master gain is not at 0
- Verify at least one oscillator has gain > 0
- Check that notes are being triggered (MIDI or keyboard)

**Clicks/pops:**
- Filter resonance too high can cause instability - reduce it
- Cutoff frequency too high can alias - reduce it or increase sample rate

**High CPU:**
- Reduce polyphony (recompile with fewer voices)
- Disable SIMD if it's causing issues: `cargo run --release --no-default-features`

## Development

### Running Tests
```bash
cargo test --release
```

### Running Benchmarks
```bash
cargo bench
```

### Building Without SIMD
```bash
cargo build --release --no-default-features
```

This uses scalar fallback code instead of SIMD optimizations.
