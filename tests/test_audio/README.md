# Test Audio Files for Phase Testing

## vocal_test.wav

Place a test vocal recording here named `vocal_test.wav` for real-world phase testing.

**Required Format:**
- **Sample Rate**: 44.1kHz (44100 Hz)
- **Channels**: Stereo (2 channels)
- **Bit Depth**: 16-bit, 24-bit, or 32-bit PCM
- **Duration**: 2-5 seconds of singing/speaking (longer files OK, but will take more time)

**Content Suggestions:**
- Clean vocal recording (minimal background noise)
- Sung phrase with sustained notes (tests harmonic content)
- Include some sibilance (S/T sounds) to test high-frequency phase
- Stereo recording preferred (or mono duplicated to stereo)

**How to Create:**
1. Record or find a vocal sample
2. Convert to 44.1kHz stereo 16-bit WAV using Audacity/DAW
3. Export as `vocal_test.wav`
4. Place in this directory: `tests/test_audio/vocal_test.wav`

**Running the Test:**
```bash
# Run only the real vocal phase test
cargo test --lib --features voice-clap test_phase_with_real_vocal -- --ignored

# Run all phase tests including real vocal
cargo test --lib --features voice-clap adaptive_saturator::tests -- --ignored --include-ignored
```

**What the Test Validates:**
- Stereo correlation preserved through processing (no phase collapse)
- No severe power loss when summing L+R (no comb filtering)
- Real-world harmonic content doesn't trigger phase cancellation
- Saturator handles transients, sibilance, and sustained notes without phase issues
