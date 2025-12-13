# DSynth Plugin - Adding Missing Parameters

## Current State
The plugin only has:
- Master gain, monophonic
- 3 oscillators with: waveform, pitch, detune, gain, pan (5 params each)
- 1 filter with: type, cutoff, resonance, amount (4 params)

## Missing Parameters

### Per Oscillator (need to add for OSC 1, 2, 3):
- unison (IntParam, 1-7)
- unison_detune (FloatParam, 0-50 cents)
- phase (FloatParam, 0-1)
- shape (FloatParam, -1 to 1)

### Filters (need to add Filter 2 and 3):
- Each filter needs: type, cutoff, resonance, drive, key_tracking

### Filter Envelopes (need to add for all 3 filters):
- attack, decay, sustain, release, amount

### LFOs (need to add all 3):
- waveform, rate, depth, filter_amount

### Velocity Sensitivity:
- amp_sensitivity, filter_sensitivity, filter_env_sensitivity

## Total Additional Parameters Needed:
- Oscillators: 4 params × 3 = 12
- Filters 2 & 3: 5 params × 2 = 10
- Filter 1 add: drive, key_tracking = 2
- Filter Envelopes: 5 params × 3 = 15
- LFOs: 4 params × 3 = 12
- Velocity: 3 params
- **Total: 54 additional parameters**

## Implementation Strategy:
1. Add all parameter declarations to DSynthParams struct
2. Add all parameter initializations in Default impl
3. Create a method to convert DSynthParams -> SynthParams
4. Call this conversion in process() to update engine parameters
