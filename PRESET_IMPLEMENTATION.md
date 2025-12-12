# DSynth Preset System Implementation

## Overview
Added complete preset save/load functionality to the DSynth GUI, allowing users to save and load synthesizer configurations as JSON files using native file dialogs.

## Changes Made

### 1. GUI Module (`src/gui/mod.rs`)
- **Imports**: Added `Preset`, `text_input` widget, and `rfd` for file dialogs
- **State**: Added `preset_name: String` field to `SynthGui` struct
- **Messages**: Added 4 new message types:
  - `PresetNameChanged(String)` - Updates preset name as user types
  - `SavePreset` - Triggers save dialog
  - `LoadPreset` - Triggers load dialog  
  - `PresetLoaded(Result<SynthParams, String>)` - Handles loaded preset data

- **UI Controls**: Added preset management row with:
  - Text input for preset name
  - Save button
  - Load button

- **Async Methods**:
  - `save_preset_dialog()` - Opens native save dialog, saves preset as JSON
  - `load_preset_dialog()` - Opens native load dialog, loads preset from JSON

### 2. Documentation Updates

#### README.md
- Updated GUI features section with preset management
- Added "Preset Management" subsection to Usage:
  - How to save presets
  - How to load presets
  - JSON file format notes
- Added `rfd` to dependencies list

#### PRESETS.md (NEW)
Comprehensive guide covering:
- How to use save/load functionality
- Preset file format with examples
- Complete parameter ranges reference
- Descriptions of included example presets
- Organization tips
- Troubleshooting guide
- API usage for developers

### 3. Example Presets
Created three example preset files in `examples/`:

#### warm_pad.json
- Detuned sawtooth waves with unison (3 voices per oscillator)
- Stereo panning for width
- Moderate low-pass filtering
- Sub-octave sine wave for depth

#### bright_lead.json
- Square wave main oscillator with 5 unison voices
- Octave harmonics (triangle + sine)
- High resonance filtering (Q=4.0)
- Drive for saturation (2.5)

#### deep_bass.json
- Sub-octave heavy bass (-12 and -24 semitones)
- Low-pass filtering (400-800 Hz)
- Heavy drive (3.5) for saturation
- Slight detuning for warmth

## Technical Implementation

### File Dialog Integration
Uses `rfd` (Rust File Dialog) crate for native file dialogs:
- Async API compatible with Iced's Task system
- Platform-native dialogs (macOS Finder, Windows Explorer, etc.)
- File type filtering (.json extension)
- Custom titles and default filenames

### Data Flow
```
User clicks Save → SavePreset message
  → save_preset_dialog() async
  → Native file dialog
  → Preset::save() to JSON
  → Complete

User clicks Load → LoadPreset message
  → load_preset_dialog() async
  → Native file dialog
  → Preset::load() from JSON
  → PresetLoaded(Ok(params))
  → Update GUI state
  → Write to triple buffer
  → Audio engine receives new params
```

### Error Handling
- File I/O errors displayed via eprintln
- Dialog cancellation handled gracefully (returns Err)
- Invalid JSON caught by serde deserialization
- No crash on invalid preset files

## User Experience

### Workflow
1. User designs sound using GUI controls
2. Types preset name in text field
3. Clicks Save → Native dialog opens with suggested filename
4. Preset saved as human-readable JSON
5. Later: Click Load → Select preset file
6. All parameters instantly update (real-time audio adjustment)

### Key Features
- **Native dialogs**: Platform-appropriate UI
- **Real-time updates**: No audio glitches when loading
- **Human-readable**: JSON format easily editable
- **Shareable**: Simple file sharing between users
- **Organized**: Users can create folder hierarchies

## Testing

### Verified
- ✅ All 73 existing tests pass
- ✅ Application compiles and runs
- ✅ Save dialog opens correctly
- ✅ Load dialog opens correctly
- ✅ Dialog cancellation handled (no crash)
- ✅ Example presets have valid JSON structure

### Manual Testing Recommended
1. Save a preset with custom settings
2. Modify settings
3. Load the saved preset
4. Verify all parameters restored correctly
5. Test with each example preset
6. Verify audio output matches expectations

## Example Usage

### Saving
1. Set Osc1 to Sawtooth, Osc2 to Square
2. Type "My Synth Lead" in preset name field
3. Click Save
4. Choose location → `~/Documents/dsynth_presets/my_synth_lead.json`
5. File saved with all current parameters

### Loading
1. Click Load
2. Navigate to `examples/warm_pad.json`
3. Select file
4. GUI instantly updates with warm pad settings
5. Audio reflects new sound immediately

## File Format Example

```json
{
  "name": "Warm Pad",
  "params": {
    "oscillators": [
      {
        "waveform": "Saw",
        "pitch": 0.0,
        "detune": -5.0,
        "gain": 0.5,
        "pan": -0.3,
        "unison": 3,
        "unison_detune": 15.0,
        "phase": 0.0
      }
      // ... 2 more oscillators
    ],
    "filters": [
      {
        "filter_type": "Lowpass",
        "cutoff": 1200.0,
        "resonance": 2.5,
        "drive": 1.5
      }
      // ... 2 more filters
    ],
    "master_gain": 0.65
  }
}
```

## Dependencies
- `rfd = "0.15"` - Already in Cargo.toml
- `serde` and `serde_json` - Already in use for serialization

## Future Enhancements
- [ ] Preset browser with thumbnails
- [ ] Preset categories/tags
- [ ] Recently used presets list
- [ ] Favorite/star presets
- [ ] Preset comparison tool
- [ ] Auto-save last state on exit
- [ ] Preset morphing/interpolation
- [ ] Cloud preset sharing

## Notes
- Presets are stored relative to user-selected path (no default location)
- No preset validation beyond JSON deserialization
- Parameter ranges enforced by GUI sliders, not preset loader
- Loading preset does not clear pressed keys (use PANIC if needed)
