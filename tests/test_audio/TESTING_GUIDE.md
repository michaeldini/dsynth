# Running the Real Vocal Phase Test

## Quick Start

1. **Place your vocal WAV file:**
   ```bash
   # Required format: 44.1kHz, stereo, 16-bit PCM, 2-5 seconds
   # Place at: tests/test_audio/vocal_test.wav
   ```

2. **Run the test:**
   ```bash
   cargo test --lib --features voice-clap test_phase_with_real_vocal -- --ignored --nocapture
   ```

3. **Expected output (if passing):**
   ```
   ✅ Real vocal test PASSED:
      Input correlation: 0.8234
      Output correlation: 0.7891
      Correlation diff: 0.0343
      Input RMS: 0.123456
      Output RMS: 0.098765
      RMS ratio: 0.8012
   ```

## What the Test Validates

- **Stereo Correlation**: Measures phase relationship between L/R channels
  - Input: Original stereo correlation
  - Output: Processed stereo correlation
  - **Pass criteria**: Difference < 0.3 (correlation preserved within tolerance)

- **Power Preservation**: Ensures no severe phase cancellation
  - Sums L+R and measures RMS power
  - **Pass criteria**: Output RMS > Input RMS × 0.2 (no catastrophic cancellation)

## Interpreting Results

**PASSING**: Correlation diff < 0.3, RMS ratio > 0.2
- Plugin maintains phase coherency with real vocal content
- Safe for production use

**FAILING**: Correlation diff > 0.3 or RMS ratio < 0.2
- Phase cancellation detected
- Investigate which frequency band causes issue
- Check for filter state corruption

## Troubleshooting

If test fails to find file:
```bash
# Check file location
ls -la tests/test_audio/vocal_test.wav

# Verify WAV format
file tests/test_audio/vocal_test.wav
# Should show: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, stereo 44100 Hz
```

If test fails due to wrong format:
```bash
# Convert using ffmpeg
ffmpeg -i your_vocal.wav -ar 44100 -ac 2 -sample_fmt s16 tests/test_audio/vocal_test.wav
```

## Example Recording Tips

**Good test content:**
- Sung vowels (aaah, eeee) - tests harmonic content
- Whispered speech - tests breathy transients
- Sibilant sounds (s, sh, ch) - tests high-frequency phase
- Mix of loud and quiet passages - tests dynamic saturation

**Avoid:**
- Heavy background noise (test isolates plugin phase, not recording quality)
- Extreme effects/reverb already applied
- Severely clipped/distorted recordings
