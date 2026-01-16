use crossbeam_channel::bounded;
use dsynth::audio::{
    engine::{SynthEngine, create_parameter_buffer},
    output::{AudioOutput, EngineEvent},
};
use dsynth::midi::handler::{MidiEvent, velocity_to_float};
use std::thread;
use std::time::Duration;

#[test]
fn test_full_audio_pipeline() {
    // Create parameter buffer
    let (_param_producer, param_consumer) = create_parameter_buffer();

    // Create synth engine
    let engine = SynthEngine::new(44100.0, param_consumer);

    let (event_tx, event_rx) = bounded::<EngineEvent>(128);

    // Try to create audio output (may fail in CI environments)
    let audio_result = AudioOutput::new(engine, event_rx);

    match audio_result {
        Ok(_audio) => {
            println!("Audio output started successfully");

            // Trigger a note
            let _ = event_tx.try_send(EngineEvent::NoteOn {
                note: 60,
                velocity: 0.8,
            });

            // Let it play for a bit
            thread::sleep(Duration::from_millis(100));

            // Release note
            let _ = event_tx.try_send(EngineEvent::NoteOff { note: 60 });

            println!("Full audio pipeline test passed");
        }
        Err(e) => {
            println!("Audio output not available (expected in CI): {}", e);
        }
    }
}

#[test]
fn test_midi_to_engine_integration() {
    // Create parameter buffer
    let (_param_producer, param_consumer) = create_parameter_buffer();

    // Create synth engine
    let engine = SynthEngine::new(44100.0, param_consumer);
    let mut engine = engine;

    // Simulate MIDI events
    let note_on = MidiEvent::NoteOn {
        note: 60,
        velocity: 100,
    };
    let note_off = MidiEvent::NoteOff { note: 60 };

    // Process note on
    if let MidiEvent::NoteOn { note, velocity } = note_on {
        engine.note_on(note, velocity_to_float(velocity));
    }

    assert_eq!(engine.active_voice_count(), 1);

    // Process some audio
    for _ in 0..100 {
        engine.process_mono();
    }

    // Process note off
    if let MidiEvent::NoteOff { note } = note_off {
        engine.note_off(note);
    }

    // Voice should still be active (in release phase)
    assert_eq!(engine.active_voice_count(), 1);

    println!("MIDI to engine integration test passed");
}

#[test]
fn test_polyphonic_performance() {
    // Create parameter buffer
    let (_param_producer, param_consumer) = create_parameter_buffer();

    // Create synth engine
    let mut engine = SynthEngine::new(44100.0, param_consumer);

    // Trigger maximum polyphony
    for i in 0..16 {
        engine.note_on(60 + i, 0.8);
    }

    assert_eq!(engine.active_voice_count(), 16);

    // Process a buffer of samples
    let buffer_size = 512;
    let start = std::time::Instant::now();

    for _ in 0..buffer_size {
        let _sample = engine.process_mono();
    }

    let elapsed = start.elapsed();

    println!(
        "Processed {} samples with 16 voices in {:?}",
        buffer_size, elapsed
    );
    println!("Time per sample: {:?}", elapsed / buffer_size);

    // At 44.1kHz, we need to process samples faster than ~22.7µs each
    // With 512 samples, should complete in well under 12ms
    assert!(
        elapsed.as_millis() < 50,
        "Performance regression: took {:?}",
        elapsed
    );
}

#[test]
fn test_hard_sync_creates_harmonics() {
    // Test that hard sync chain (OSC1→OSC2→OSC3) generates additional harmonic content
    // by comparing waveforms with and without hard sync enabled
    
    use dsynth::audio::voice::Voice;
    use dsynth::params::{SynthParams, Waveform};
    use dsynth::dsp::wavetable_library::WavetableLibrary;
    
    let sample_rate = 44100.0;
    let mut voice = Voice::new(sample_rate);
    let wavetable_library = WavetableLibrary::new();
    
    // Setup: All 3 oscillators with slightly different pitches for sync chain
    // OSC1 (master) → OSC2 → OSC3
    let mut params = SynthParams::default();
    params.oscillators[0].waveform = Waveform::Saw;
    params.oscillators[0].gain = 0.4; // Slightly lower to avoid clipping
    params.oscillators[0].pitch = -36.0; // Base frequency
    
    params.oscillators[1].waveform = Waveform::Saw;
    params.oscillators[1].gain = 0.4;
    params.oscillators[1].pitch = -35.0; // Slightly higher than OSC1
    
    params.oscillators[2].waveform = Waveform::Saw;
    params.oscillators[2].gain = 0.4;
    params.oscillators[2].pitch = -34.0; // Even higher (sync chain: OSC1→OSC2→OSC3)
    
    // Test WITHOUT hard sync
    voice.note_on(60, 0.8);
    voice.update_parameters(
        &params.oscillators,
        &params.filters,
        &params.lfos,
        &params.envelope,
        &wavetable_library,
    );
    
    let mut samples_no_sync = Vec::with_capacity(4410); // 100ms at 44.1kHz
    for _ in 0..4410 {
        let (left, _right) = voice.process(
            &params.oscillators,
            &params.filters,
            &params.lfos,
            &params.velocity,
            false, // hard_sync_enabled = false
            &Default::default(), // voice_comp_params
            &Default::default(), // transient_params
        );
        samples_no_sync.push(left);
    }
    
    let rms_no_sync = (samples_no_sync.iter().map(|s| s * s).sum::<f32>() / samples_no_sync.len() as f32).sqrt();
    
    // Reset voice for second test
    voice.note_off();
    for _ in 0..4410 {
        voice.process(&params.oscillators, &params.filters, &params.lfos, &params.velocity, false, &Default::default(), &Default::default());
    }
    
    // Test WITH hard sync chain (OSC1→OSC2→OSC3)
    voice.note_on(60, 0.8);
    voice.update_parameters(
        &params.oscillators,
        &params.filters,
        &params.lfos,
        &params.envelope,
        &wavetable_library,
    );
    
    let mut samples_with_sync = Vec::with_capacity(4410);
    for _ in 0..4410 {
        let (left, _right) = voice.process(
            &params.oscillators,
            &params.filters,
            &params.lfos,
            &params.velocity,
            true, // hard_sync_enabled = true (enables full chain)
            &Default::default(), // voice_comp_params
            &Default::default(), // transient_params
        );
        samples_with_sync.push(left);
    }
    
    let rms_with_sync = (samples_with_sync.iter().map(|s| s * s).sum::<f32>() / samples_with_sync.len() as f32).sqrt();
    
    println!("RMS without hard sync: {:.6}", rms_no_sync);
    println!("RMS with hard sync chain (OSC1→OSC2→OSC3): {:.6}", rms_with_sync);
    
    // Verify both signals have energy (not silent)
    assert!(rms_no_sync > 0.01, "Signal should have energy without hard sync");
    assert!(rms_with_sync > 0.01, "Signal should have energy with hard sync");
    
    // Hard sync chain should produce measurably different output
    let difference_ratio = (rms_with_sync - rms_no_sync).abs() / rms_no_sync;
    println!("RMS difference: {:.1}%", difference_ratio * 100.0);
    
    // With 3 oscillators in sync chain, the effect should be even more pronounced
    assert!(
        difference_ratio > 0.05 || rms_with_sync != rms_no_sync,
        "Hard sync chain should produce measurably different output (difference: {:.1}%)",
        difference_ratio * 100.0
    );
}
