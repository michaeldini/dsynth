# DSynth VST Plugin Guide

## Overview
DSynth can be built as both a standalone application and a VST3/CLAP plugin. The plugin uses the same audio engine as the standalone version, with a thin wrapper layer.

## Architecture

### Standalone vs Plugin
```
┌─────────────────────────────────────────┐
│          Standalone                     │
├─────────────────────────────────────────┤
│  main.rs (GUI + MIDI + Audio via cpal) │
│              ↓                          │
│        SynthEngine (Core DSP)           │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│            VST Plugin                   │
├─────────────────────────────────────────┤
│    plugin.rs (nih_plug wrapper)        │
│              ↓                          │
│        SynthEngine (Core DSP)           │
└─────────────────────────────────────────┘
```

The core `SynthEngine` is **100% shared** between both versions. Only the wrapper layer differs.

## Features

### VST Plugin Features
- ✅ **Full MIDI support** - Notes, velocity, aftertouch
- ✅ **Parameter automation** - All synth parameters automatable in DAW
- ✅ **Stereo output** - Clean stereo signal path
- ✅ **Low latency** - Efficient block processing
- ✅ **VST3 and CLAP** - Both formats supported
- ✅ **Cross-platform** - macOS, Windows, Linux

### Parameter Mapping
The plugin exposes the following parameters to your DAW:
- Master Gain
- Monophonic Mode (on/off)
- Per-oscillator: Waveform, Pitch, Detune, Gain, Pan
- Per-filter: Type, Cutoff, Resonance, Amount
- More parameters can be easily added to `plugin.rs`

## Building

### 1. Build the Plugin
```bash
# macOS: Use the bundle script
./bundle.sh

# Windows/Linux: Build the library
cargo build --release --lib --features vst
# Then manually create the plugin bundle

# Build standalone application
cargo build --release --features standalone
```

### 2. Install the Plugin
After building, the plugins will be in `target/bundled/`:
- `DSynth.vst3` - VST3 plugin
- `DSynth.clap` - CLAP plugin

**Installation locations:**
- **macOS**: 
  - VST3: `~/Library/Audio/Plug-Ins/VST3/`
  - CLAP: `~/Library/Audio/Plug-Ins/CLAP/`
- **Windows**: 
  - VST3: `C:\Program Files\Common Files\VST3\`
  - CLAP: `C:\Program Files\Common Files\CLAP\`
- **Linux**: 
  - VST3: `~/.vst3/`
  - CLAP: `~/.clap/`

```bash
# Example: Install on macOS
cp -r target/bundled/DSynth.vst3 ~/Library/Audio/Plug-Ins/VST3/
cp -r target/bundled/DSynth.clap ~/Library/Audio/Plug-Ins/CLAP/
```

## Using in Your DAW

### 1. Scan for Plugins
After installation, rescan plugins in your DAW:
- **Ableton Live**: Preferences → Plug-Ins → Rescan
- **FL Studio**: Options → Manage Plugins → Find Plugins
- **Reaper**: Actions → Show Action List → "Rescan VST paths"
- **Logic Pro**: Opens automatically after copying to plugin folder

### 2. Load DSynth
- Create a new instrument track
- Select "DSynth" from your synth/instrument list
- The plugin will load with default parameters

### 3. Play Notes
- Connect a MIDI track to the DSynth instrument
- Play notes via MIDI keyboard or piano roll
- All MIDI note on/off messages are handled correctly

### 4. Automate Parameters
All DSynth parameters are automatable:
- Select a parameter (e.g., Filter 1 Cutoff)
- Enable automation in your DAW
- Draw automation curves or record live parameter changes

## Technical Details

### MIDI Handling
The plugin receives MIDI from the DAW through nih-plug's `NoteEvent` system:
- **Note On** → `engine.note_on(note, velocity)`
- **Note Off** → `engine.note_off(note)`
- No `midir` dependency needed - DAW handles MIDI routing

### Audio Processing
- Processes audio in blocks (buffer size set by DAW)
- Calls `engine.process()` for each sample
- Outputs stereo (left/right channels)
- Lock-free parameter updates via triple-buffer (same as standalone)

### Parameter Updates
- DAW parameter changes flow through nih-plug
- Currently: Parameters update per-block (future: sample-accurate automation)
- Triple-buffer ensures audio thread never blocks

### Voice Management
- Same 16-voice polyphony as standalone
- Same voice stealing algorithm (quietest-voice)
- Monophonic mode available as plugin parameter

## Development

### Adding New Parameters
To expose additional synth parameters:

1. Add to `DSynthParams` in [plugin.rs](src/plugin.rs):
```rust
#[id = "osc1_unison"]
pub osc1_unison: IntParam,
```

2. Map to `SynthParams` in `process()` method
3. Rebuild the plugin

### Debugging
```bash
# Build debug version (more logs)
cargo xtask bundle dsynth

# Run tests (covers engine logic)
cargo test --features vst
```

### Testing in DAW
1. Build and install the plugin
2. Load in your DAW
3. Check MIDI input (play notes)
4. Verify audio output
5. Test parameter automation

## Differences from Standalone

| Feature | Standalone | VST Plugin |
|---------|-----------|-----------|
| MIDI Input | `midir` library | DAW provides |
| Audio Output | `cpal` library | DAW provides |
| GUI | Custom Iced GUI | DAW's generic controls* |
| Parameter Control | GUI sliders | DAW automation |
| Preset System | JSON files | DAW presets** |

\* Custom plugin GUI can be added using `nih_plug_iced`  
\*\* DAW preset system wraps plugin parameters

## Roadmap

### Current (v0.1.1)
- [x] Basic VST3/CLAP wrapper
- [x] MIDI note handling
- [x] Core parameter automation
- [x] Stereo output

### Next Steps
- [ ] Custom plugin GUI using `nih_plug_iced`
- [ ] Sample-accurate automation
- [ ] All synth parameters exposed
- [ ] Preset saving within plugin
- [ ] MIDI CC mapping

## Troubleshooting

### Plugin doesn't appear in DAW
- Verify plugin is in correct folder
- Rescan plugins in DAW
- Check DAW's plugin blacklist/blocklist
- Try CLAP format if VST3 fails (or vice versa)

### No sound output
- Check MIDI is routed to the instrument track
- Verify oscillator gains are not zero
- Check master gain parameter
- Ensure notes are being triggered (check MIDI monitor)

### Clicking/popping in audio
- Increase DAW buffer size
- Check CPU usage (may need optimization)
- Verify sample rate matches DAW

### Parameters don't automate
- Enable automation mode in DAW
- Check parameter is selected for automation
- Some DAWs require "touch" mode for live recording

## Support

For issues or questions:
- Check [README.md](README.md) for general DSynth documentation
- Review `src/plugin.rs` for implementation details
- Test in standalone mode first (helps isolate issues)
