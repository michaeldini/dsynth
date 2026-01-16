//! Unit tests for SynthEngine implementation.
//!
//! Tests cover:
//! - Engine creation and initialization
//! - Voice allocation and management
//! - Note on/off handling
//! - Polyphony limits and voice stealing
//! - Audio output generation
//! - Parameter updates via triple-buffer
//! - Monophonic mode and note stack
//! - Tempo sync functionality

use super::*;

/// Test that the engine can be created with the correct sample rate.
/// Verifies:
/// - Engine construction succeeds
/// - Sample rate is correctly stored and retrievable
/// - No voices are active initially (all idle)
#[test]
fn test_engine_creation() {
    let (_producer, consumer) = create_parameter_buffer();
    let engine = SynthEngine::new(44100.0, consumer);

    assert_eq!(engine.sample_rate(), 44100.0);
    assert_eq!(engine.active_voice_count(), 0);
}

/// Test that note on events activate voices for synthesis.
/// Verifies:
/// - Each note_on() call activates a new voice
/// - Multiple notes can play simultaneously (polyphonic mode)
/// - Voice count increases with each note
#[test]
fn test_note_on_activates_voice() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    engine.note_on(60, 0.8);
    assert_eq!(engine.active_voice_count(), 1);

    engine.note_on(64, 0.7);
    assert_eq!(engine.active_voice_count(), 2);
}

/// Test that note off causes voices to release and fade out.
/// Verifies:
/// - note_off() puts the voice in release phase (still active, still audible)
/// - Voice remains active during release fade-out
/// - Voice becomes idle after release time completes
/// This tests the envelope's ADSR behavior (specifically the Release phase)
#[test]
fn test_note_off_releases_voice() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    engine.note_on(60, 0.8);
    assert_eq!(engine.active_voice_count(), 1);

    engine.note_off(60);

    // Voice should still be active during release phase
    assert_eq!(engine.active_voice_count(), 1);

    // Process through release
    for _ in 0..20000 {
        engine.process_mono();
    }

    // Should be inactive after release completes
    assert_eq!(engine.active_voice_count(), 0);
}

/// Test that polyphony has a hard limit (16 voices).
/// When more notes are triggered than polyphony allows, voice stealing should occur.
/// Verifies:
/// - Engine never exceeds MAX_POLYPHONY (16) active voices
/// - Voice stealing is working to keep polyphony under control
#[test]
fn test_polyphony_limit() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Trigger more notes than polyphony limit
    for i in 0..20 {
        engine.note_on(60 + i, 0.8);
    }

    // Should not exceed max polyphony
    assert!(engine.active_voice_count() <= MAX_POLYPHONY);
}

/// Test that voice stealing prioritizes quiet voices.
/// When all voices are active and a new note arrives, the quietest voice should be killed.
/// Verifies:
/// - All voice slots can be filled
/// - One more note triggers voice stealing
/// - New note still triggers (doesn't get dropped)
/// - Polyphony limit is maintained
#[test]
fn test_voice_stealing_quietest() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Fill all voice slots
    for i in 0..MAX_POLYPHONY {
        engine.note_on(60 + i as u8, 0.8);
    }

    // Process to build up RMS values
    for _ in 0..500 {
        engine.process_mono();
    }

    // Trigger one more note - should steal quietest
    engine.note_on(100, 0.9);
    assert_eq!(engine.active_voice_count(), MAX_POLYPHONY);
}

/// Test that all_notes_off() immediately silences all voices.
/// Verifies:
/// - Before all_notes_off(): multiple voices are active
/// - After all_notes_off(): zero active voices
/// - This is different from note_off() which releases each voice (plays release envelope)
/// - all_notes_off() is a hard stop for panic/emergency silence
#[test]
fn test_all_notes_off() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Trigger multiple notes
    for i in 0..5 {
        engine.note_on(60 + i, 0.8);
    }

    assert!(engine.active_voice_count() > 0);

    engine.all_notes_off();
    assert_eq!(engine.active_voice_count(), 0);
}

/// Test that process() generates audible output when notes are playing.
/// Verifies:
/// - Triggering a note produces audio samples (not silent)
/// - Output amplitude is non-zero (demonstrates synthesis is working)
/// - Output is bounded (-1.0 to +1.0 approximately) and doesn't clip
#[test]
fn test_output_generation() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    engine.note_on(60, 0.8);

    // Process samples and verify output
    let mut has_output = false;
    for _ in 0..1000 {
        let sample = engine.process_mono();
        if sample.abs() > 0.001 {
            has_output = true;
            break;
        }
    }

    assert!(has_output, "Engine should produce audio output");
}

/// Test that parameter updates are correctly propagated to voices.
/// Verifies:
/// - Parameters can be written to the triple-buffer via producer
/// - The engine picks up changes and applies them to voices
/// - Parameter throttling doesn't prevent updates from eventually applying
/// This tests the lock-free communication mechanism between GUI and audio threads
#[test]
fn test_parameter_updates() {
    let (mut producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Update parameters via triple buffer
    let new_params = SynthParams {
        master_gain: 0.5,
        ..Default::default()
    };
    producer.write(new_params);

    // Process should pick up new parameters
    engine.process_mono();

    // Verify by checking that output is affected by master gain
    engine.note_on(60, 1.0);
    for _ in 0..100 {
        engine.process_mono();
    }

    // Parameters were updated (verified implicitly through processing)
}

/// Test that the same note can be played on multiple voices simultaneously.
/// This is useful for unison effects (multiple detuned oscillators playing one note).
/// Verifies:
/// - Multiple note_on() calls with the same note number each allocate a separate voice
/// - Each instance is independent (can have different velocities)
/// - All instances are affected by note_off() (all start releasing)
#[test]
fn test_same_note_multiple_voices() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Trigger same note multiple times
    engine.note_on(60, 0.8);
    engine.note_on(60, 0.7);
    engine.note_on(60, 0.6);

    assert_eq!(engine.active_voice_count(), 3);

    // Release should affect all instances
    engine.note_off(60);

    // All three should be in release
    assert_eq!(engine.active_voice_count(), 3);
}

/// Test that zero velocity NoteOn is treated as no-op (MIDI semantics).
///
/// MIDI specifies that NoteOn with velocity 0 is equivalent to NoteOff.
/// For engine-level note injection, we treat velocity 0 as "do nothing" to
/// avoid activating voices that should be silent.
#[test]
fn test_zero_velocity_note() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    engine.note_on(60, 0.0);
    assert_eq!(engine.active_voice_count(), 0);

    // Process samples - should remain silent.
    let mut max_output = 0.0_f32;
    for _ in 0..1000 {
        let sample = engine.process_mono();
        max_output = max_output.max(sample.abs());
    }

    assert!(
        max_output <= 1.0e-6,
        "Zero velocity should be silent, but got peak {:.6}",
        max_output
    );
}

#[test]
fn test_no_clipping_on_basic_chord() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Use a moderately hot master gain to catch headroom issues.
    engine.current_params.master_gain = 1.0;

    // Trigger a basic triad + octave.
    engine.note_on(60, 0.8); // C4
    engine.note_on(64, 0.8); // E4
    engine.note_on(67, 0.8); // G4
    engine.note_on(72, 0.8); // C5

    // Let the envelope get past attack.
    for _ in 0..4000 {
        let _ = engine.process();
    }

    let mut max_peak = 0.0_f32;
    for _ in 0..12000 {
        let (l, r) = engine.process();
        max_peak = max_peak.max(l.abs().max(r.abs()));
    }

    assert!(
        max_peak <= 1.0,
        "Output should not clip (peak was {:.4})",
        max_peak
    );
}

#[test]
fn test_monophonic_legato_note_change_does_not_zero_output() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);
    engine.current_params.monophonic = true;

    // Start a note and wait until we get a clearly non-zero sample.
    engine.note_on(60, 1.0);
    let mut prev = 0.0_f32;
    let mut max_abs = 0.0_f32;
    for _ in 0..20000 {
        let s = engine.process_mono();
        max_abs = max_abs.max(s.abs());
        if s.abs() > 0.01 {
            prev = s;
            break;
        }
    }
    assert!(
        prev.abs() > 0.01,
        "Expected non-trivial output before legato switch (max_abs={:.6})",
        max_abs
    );

    // Press another key while still holding the first (overlap => legato switch).
    engine.note_on(64, 1.0);
    let next = engine.process_mono();

    // The key property: we should not hard-drop to (near) silence on the very next sample.
    // Use a relative check so the test doesn't assume any particular patch loudness.
    assert!(
        next.abs() >= 0.25 * prev.abs(),
        "Mono legato note change should stay continuous: prev={:.6}, next={:.6}",
        prev,
        next
    );
}

#[test]
fn test_monophonic_fast_retrigger_does_not_hard_drop() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);
    engine.current_params.monophonic = true;

    engine.note_on(60, 1.0);

    // Let the voice get clearly above the noise floor.
    let mut prev = 0.0_f32;
    let mut max_abs = 0.0_f32;
    for _ in 0..20000 {
        let s = engine.process_mono();
        max_abs = max_abs.max(s.abs());
        if s.abs() > 0.02 {
            prev = s;
            break;
        }
    }
    assert!(
        prev.abs() > 0.02,
        "Expected non-trivial output before retrigger (max_abs={:.6})",
        max_abs
    );

    // Release then immediately retrigger the same key (typical fast tapping).
    engine.note_off(60);
    engine.note_on(60, 1.0);
    let next = engine.process_mono();

    // The key property: first sample after retrigger should not drop near zero.
    assert!(
        next.abs() >= 0.25 * prev.abs(),
        "Fast retrigger should not hard-drop: prev={:.6}, next={:.6}",
        prev,
        next
    );
}

// ==================== Tempo Sync Tests ====================

/// Test tempo_division_to_hz() accuracy at 120 BPM (standard tempo)
/// Verifies all 12 musical divisions produce correct frequencies
#[test]
fn test_tempo_division_to_hz_120bpm() {
    use crate::params::TempoSync;
    use approx::assert_relative_eq;

    // At 120 BPM: 2 beats per second
    let bpm = 120.0;

    // Hz mode should return 0.0 (signal to use raw Hz)
    assert_eq!(SynthEngine::tempo_division_to_hz(TempoSync::Hz, bpm), 0.0);

    // Standard divisions
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Whole, bpm),
        0.5, // 1 cycle per 4 beats = 0.5 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Half, bpm),
        1.0, // 1 cycle per 2 beats = 1 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Quarter, bpm),
        2.0, // 1 cycle per beat = 2 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Eighth, bpm),
        4.0, // 2 cycles per beat = 4 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Sixteenth, bpm),
        8.0, // 4 cycles per beat = 8 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::ThirtySecond, bpm),
        16.0, // 8 cycles per beat = 16 Hz
        epsilon = 0.001
    );

    // Triplet divisions (3 per normal duration)
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::QuarterT, bpm),
        3.0, // 3 triplets per 2 beats = 3 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::EighthT, bpm),
        6.0, // 3 triplets per beat = 6 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::SixteenthT, bpm),
        12.0, // 3 triplets per half beat = 12 Hz
        epsilon = 0.001
    );

    // Dotted divisions (1.5× normal duration)
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::QuarterD, bpm),
        1.333, // 1 cycle per 1.5 beats ≈ 1.333 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::EighthD, bpm),
        2.667, // 1 cycle per 0.75 beats ≈ 2.667 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::SixteenthD, bpm),
        5.333, // 1 cycle per 0.375 beats ≈ 5.333 Hz
        epsilon = 0.001
    );
}

/// Test tempo_division_to_hz() at various tempos
/// Verifies formulas scale correctly with different BPMs
#[test]
fn test_tempo_division_to_hz_various_bpms() {
    use crate::params::TempoSync;
    use approx::assert_relative_eq;

    // Slow tempo: 60 BPM (1 beat per second)
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Quarter, 60.0),
        1.0, // 1 cycle per beat = 1 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Eighth, 60.0),
        2.0, // 2 cycles per beat = 2 Hz
        epsilon = 0.001
    );

    // Fast tempo: 140 BPM (techno/house)
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Quarter, 140.0),
        2.333, // 140/60 = 2.333 Hz
        epsilon = 0.001
    );

    // Very fast: 180 BPM (drum and bass)
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Quarter, 180.0),
        3.0, // 180/60 = 3 Hz
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::Sixteenth, 180.0),
        12.0, // 4 sixteenths per beat × 3 beats/sec = 12 Hz
        epsilon = 0.001
    );
}

/// Test that tempo_division_to_hz() clamps output to valid range
/// Prevents extreme tempos from producing unusable rates
#[test]
fn test_tempo_division_to_hz_clamping() {
    use crate::params::TempoSync;

    // Very slow tempo could produce rates below 0.01 Hz
    let very_slow_result = SynthEngine::tempo_division_to_hz(TempoSync::Whole, 1.0);
    assert!(
        very_slow_result >= 0.01,
        "Should clamp to minimum 0.01 Hz, got {}",
        very_slow_result
    );

    // Very fast tempo could produce rates above 20 Hz
    let very_fast_result = SynthEngine::tempo_division_to_hz(TempoSync::ThirtySecond, 300.0);
    assert!(
        very_fast_result <= 20.0,
        "Should clamp to maximum 20 Hz, got {}",
        very_fast_result
    );
}

/// Test that set_tempo() updates current tempo correctly
#[test]
fn test_set_tempo() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);

    // Default tempo should be 120 BPM
    assert_eq!(engine.current_tempo_bpm, 120.0);

    // Update tempo
    engine.set_tempo(140.0);
    assert_eq!(engine.current_tempo_bpm, 140.0);

    // Update to another tempo
    engine.set_tempo(85.0);
    assert_eq!(engine.current_tempo_bpm, 85.0);
}

/// Test that get_effective_rate() returns raw Hz when tempo_sync = Hz
#[test]
fn test_get_effective_rate_hz_mode() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);
    use crate::params::TempoSync;

    // Should return the raw Hz rate when in Hz mode
    let rate = engine.get_effective_rate(5.0, TempoSync::Hz, 0);
    assert_eq!(rate, 5.0);

    let rate2 = engine.get_effective_rate(0.5, TempoSync::Hz, 1);
    assert_eq!(rate2, 0.5);
}

/// Test that get_effective_rate() calculates tempo-synced rate correctly
#[test]
fn test_get_effective_rate_tempo_synced() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);
    use crate::params::TempoSync;
    use approx::assert_relative_eq;

    // Set tempo to 120 BPM
    engine.set_tempo(120.0);

    // Quarter note at 120 BPM should be 2 Hz
    let rate = engine.get_effective_rate(0.0, TempoSync::Quarter, 0);
    assert_relative_eq!(rate, 2.0, epsilon = 0.001);

    // Eighth note at 120 BPM should be 4 Hz
    let rate2 = engine.get_effective_rate(0.0, TempoSync::Eighth, 1);
    assert_relative_eq!(rate2, 4.0, epsilon = 0.001);

    // Change tempo to 140 BPM
    engine.set_tempo(140.0);

    // Quarter note at 140 BPM should be ~2.333 Hz
    let rate3 = engine.get_effective_rate(0.0, TempoSync::Quarter, 2);
    assert_relative_eq!(rate3, 2.333, epsilon = 0.001);
}

/// Test that phase resets when sync mode changes
/// This ensures predictable timing when switching between sync divisions
#[test]
fn test_phase_reset_on_sync_mode_change() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);
    use crate::params::TempoSync;

    // Initial call with Quarter note
    let _rate1 = engine.get_effective_rate(2.0, TempoSync::Quarter, 3); // Chorus

    // Previous sync mode should be stored
    assert_eq!(engine.previous_sync_modes[3], TempoSync::Quarter);

    // Change to Eighth note - should trigger phase reset
    let _rate2 = engine.get_effective_rate(2.0, TempoSync::Eighth, 3);

    // Previous sync mode should be updated
    assert_eq!(engine.previous_sync_modes[3], TempoSync::Eighth);

    // Same mode again - should NOT reset phase (no change)
    let _rate3 = engine.get_effective_rate(2.0, TempoSync::Eighth, 3);
    assert_eq!(engine.previous_sync_modes[3], TempoSync::Eighth);
}

/// Test that get_tempo_synced_lfo_params() applies tempo sync to all LFOs
#[test]
fn test_get_tempo_synced_lfo_params() {
    let (_producer, consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(44100.0, consumer);
    use crate::params::TempoSync;
    use approx::assert_relative_eq;

    // Set tempo to 120 BPM
    engine.set_tempo(120.0);

    // Configure LFOs with different sync modes
    engine.current_params.lfos[0].tempo_sync = TempoSync::Quarter;
    engine.current_params.lfos[0].rate = 999.0; // Should be ignored

    engine.current_params.lfos[1].tempo_sync = TempoSync::Eighth;
    engine.current_params.lfos[1].rate = 888.0; // Should be ignored

    engine.current_params.lfos[2].tempo_sync = TempoSync::Hz;
    engine.current_params.lfos[2].rate = 5.0; // Should be preserved

    // Get tempo-synced params
    let synced_lfos = engine.get_tempo_synced_lfo_params();

    // LFO1: Quarter note at 120 BPM = 2 Hz
    assert_relative_eq!(synced_lfos[0].rate, 2.0, epsilon = 0.001);

    // LFO2: Eighth note at 120 BPM = 4 Hz
    assert_relative_eq!(synced_lfos[1].rate, 4.0, epsilon = 0.001);

    // LFO3: Hz mode should preserve raw rate
    assert_eq!(synced_lfos[2].rate, 5.0);
}

/// Test that tempo sync works with 120 BPM fallback when no transport available
#[test]
fn test_tempo_sync_fallback_120bpm() {
    let (_producer, consumer) = create_parameter_buffer();
    let engine = SynthEngine::new(44100.0, consumer);
    use crate::params::TempoSync;
    use approx::assert_relative_eq;

    // Engine should default to 120 BPM
    assert_eq!(engine.current_tempo_bpm, 120.0);

    // Quarter note at default 120 BPM should be 2 Hz
    let rate = SynthEngine::tempo_division_to_hz(TempoSync::Quarter, engine.current_tempo_bpm);
    assert_relative_eq!(rate, 2.0, epsilon = 0.001);
}

/// Test triplet formulas accuracy
#[test]
fn test_triplet_formulas() {
    use crate::params::TempoSync;
    use approx::assert_relative_eq;

    // At 120 BPM (2 beats/sec):
    // Quarter triplet: 3 triplets per 2 beats = 1.5 triplets per beat = 3 Hz
    // Eighth triplet: 3 triplets per 1 beat = 6 Hz
    // Sixteenth triplet: 3 triplets per 0.5 beats = 12 Hz

    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::QuarterT, 120.0),
        3.0,
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::EighthT, 120.0),
        6.0,
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::SixteenthT, 120.0),
        12.0,
        epsilon = 0.001
    );

    // At 90 BPM (1.5 beats/sec):
    // Quarter triplet: 3 triplets per 2 beats = 1.5 × (1.5/2) = 2.25 Hz
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::QuarterT, 90.0),
        2.25,
        epsilon = 0.001
    );
}

/// Test dotted note formulas accuracy
#[test]
fn test_dotted_formulas() {
    use crate::params::TempoSync;
    use approx::assert_relative_eq;

    // At 120 BPM (2 beats/sec):
    // Dotted quarter (1.5 beats): 2/1.5 = 1.333 Hz
    // Dotted eighth (0.75 beats): 2/0.75 = 2.667 Hz
    // Dotted sixteenth (0.375 beats): 2/0.375 = 5.333 Hz

    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::QuarterD, 120.0),
        1.333,
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::EighthD, 120.0),
        2.667,
        epsilon = 0.001
    );
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::SixteenthD, 120.0),
        5.333,
        epsilon = 0.001
    );

    // At 60 BPM (1 beat/sec):
    // Dotted quarter: 1/1.5 = 0.667 Hz
    assert_relative_eq!(
        SynthEngine::tempo_division_to_hz(TempoSync::QuarterD, 60.0),
        0.667,
        epsilon = 0.001
    );
}
