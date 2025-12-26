# Wavetable Directory

Place your custom single-cycle wavetable .wav files in this directory.

## Requirements

- **Format**: WAV files (.wav extension)
- **Length**: Typically 2048 samples per cycle (but any length is supported)
- **Channels**: Mono or stereo (stereo will be converted to mono)
- **Sample Format**: 16-bit, 24-bit, or 32-bit integer, or 32-bit float
- **Content**: Single-cycle waveforms (one complete period of the wave)

## How to Use

1. Copy your .wav wavetable files to this directory
2. Restart DSynth
3. In the GUI, select "Wavetable" as the oscillator waveform
4. Use the "Wavetable" knob to select which wavetable to use (0-63)
5. Use the "Position" knob to morph between adjacent wavetables

## Where to Get Wavetables

Free wavetable sources:
- **Adventure Kid Waveforms (AKWF)**: https://www.adventurekid.se/akrt/waveforms/
  - 4300+ single-cycle waveforms, open source
- **WaveEdit**: Create your own using free wavetable editors
- **Serum/Vital**: Many free wavetable packs available online

## Built-in Wavetables

If no .wav files are found, DSynth uses these built-in wavetables:
- 0: Sine
- 1: Sawtooth
- 2: Square
- 3: Triangle
- 4: Pulse 25%
- 5: Pulse 75%

## Technical Details

Loaded wavetables are:
- Pre-processed with 4× oversampling for anti-aliasing
- Cached in memory (no disk I/O during playback)
- Accessible via CLAP automation (wavetable index parameter)
