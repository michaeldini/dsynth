# Preset Management Guide

DSynth includes a preset system that allows you to save and load synthesizer configurations as JSON files.

## Using Presets

### Saving a Preset

1. Configure your sound using the oscillator and filter controls
2. Enter a name for your preset in the "Preset" text field at the top of the GUI
3. Click the **Save** button
4. Choose a location and filename in the file dialog
5. Your preset will be saved as a `.json` file

### Loading a Preset

1. Click the **Load** button
2. Navigate to and select a `.json` preset file
3. The synthesizer parameters will immediately update to match the preset
4. The audio engine will apply the new settings in real-time

## Preset File Format

Presets are stored as human-readable JSON files. You can edit them manually or share them with others.

### Example Preset Structure

```json
{
  "name": "My Sound",
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
      },
      // ... two more oscillators
    ],
    "filters": [
      {
        "filter_type": "Lowpass",
        "cutoff": 1200.0,
        "resonance": 2.5,
        "drive": 1.5
      },
      // ... two more filters
    ],
    "master_gain": 0.65
  }
}
```

## Parameter Ranges

### Oscillator Parameters
- `waveform`: "Sine", "Saw", "Square", or "Triangle"
- `pitch`: -24.0 to 24.0 (semitones)
- `detune`: -50.0 to 50.0 (cents)
- `gain`: 0.0 to 1.0
- `pan`: -1.0 (left) to 1.0 (right)
- `unison`: 1 to 7 (number of voices)
- `unison_detune`: 0.0 to 50.0 (cents)
- `phase`: 0.0 to 1.0

### Filter Parameters
- `filter_type`: "Lowpass", "Highpass", or "Bandpass"
- `cutoff`: 20.0 to 20000.0 (Hz)
- `resonance`: 0.5 to 10.0
- `drive`: 1.0 to 10.0

### Master Parameters
- `master_gain`: 0.0 to 1.0

## Included Example Presets

The `examples/` directory contains several example presets to get you started:

### warm_pad.json
- Lush, detuned sawtooth waves with moderate filtering
- Uses 3 unison voices per oscillator for thickness
- Good for ambient pads and sustained chords

### bright_lead.json
- Bright, cutting lead sound with unison and harmonics
- Square wave with 5 unison voices + octave harmonics
- High resonance for presence

### deep_bass.json
- Powerful bass sound with sub-octave sine
- Heavy drive and low-pass filtering
- Detuned oscillators for width

## Tips

1. **Organize Your Presets**: Create folders by category (Pads, Leads, Bass, etc.)
2. **Backup Important Presets**: Keep copies of your favorite sounds
3. **Share Presets**: Exchange preset files with other DSynth users
4. **Edit Manually**: You can fine-tune presets by editing the JSON directly
5. **Version Control**: Use git to track preset evolution over time

## Troubleshooting

### Preset Won't Load
- Check that the JSON syntax is valid
- Ensure all parameter values are within valid ranges
- Verify the file has all required fields (oscillators, filters, master_gain)

### Sound Doesn't Match Expected
- Different audio interfaces may produce slightly different results
- Ensure your sample rate matches what was used when creating the preset
- Check that no notes are currently playing (use PANIC button)

## Creating Preset Banks

You can create collections of presets organized by type:

```
my_presets/
├── bass/
│   ├── deep_bass.json
│   ├── funky_bass.json
│   └── sub_bass.json
├── leads/
│   ├── bright_lead.json
│   └── soft_lead.json
└── pads/
    └── warm_pad.json
```

## API Usage (for developers)

Presets can be loaded programmatically:

```rust
use dsynth::preset::Preset;

// Load a preset
let preset = Preset::load("path/to/preset.json")?;
let params = preset.params;

// Create and save a preset
let preset = Preset::new("My Sound".to_string(), my_params);
preset.save("path/to/preset.json")?;
```
