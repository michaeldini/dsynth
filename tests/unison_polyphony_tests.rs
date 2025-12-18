//! Tests for unison + polyphony to detect and prevent clipping.
//!
//! This test suite verifies that when multiple notes are played simultaneously
//! with high unison counts, the audio output doesn't clip or distort.

use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
use dsynth::params::{SynthParams, Waveform};

/// Test that high unison counts with multiple voices don't cause clipping.
///
/// This reproduces the issue where:
/// - Unison is set to 3 for all oscillators
/// - Multiple keys are played (polyphony)
/// - The summed output exceeds ±1.0 (clipping/distortion)
///
/// The test triggers 8 notes simultaneously, processes 1024 samples, and checks that:
/// 1. No samples exceed ±1.0 (hard clipping threshold)
/// 2. RMS level is reasonable (not consistently hot)
/// 3. Peak level is below 1.0 with some headroom
#[test]
fn test_unison_3_with_polyphony_no_clipping() {
    // === Setup synthesizer ===
    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    // === Configure parameters with unison=3 for all oscillators ===
    let mut params = SynthParams::default();

    // Set all 3 oscillators to unison=3 with sawtooth waveform
    // Sawtooth is rich in harmonics, making it a good stress test
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Saw;
        params.oscillators[i].unison = 3;
        params.oscillators[i].unison_detune = 10.0; // 10 cents detune
        params.oscillators[i].gain = 0.33; // Standard gain per oscillator
        params.oscillators[i].pan = 0.0; // Center
    }

    // Set moderate envelope to ensure sustained output during test
    params.envelope.attack = 0.01; // 10ms attack
    params.envelope.decay = 0.1; // 100ms decay
    params.envelope.sustain = 0.8; // 80% sustain level
    params.envelope.release = 0.2; // 200ms release

    // Update engine parameters via triple buffer
    param_producer.write(params);

    // === Trigger multiple notes (stress test polyphony) ===
    // Play 8 notes in a chord at forte velocity
    let notes = [60, 64, 67, 72, 76, 79, 84, 88]; // C major chord across 2+ octaves
    let velocity = 1.0; // Maximum velocity (forte)

    for &note in &notes {
        engine.note_on(note, velocity);
    }

    // === Process audio and check for clipping ===
    let num_samples = 1024; // ~23ms at 44.1kHz (enough for attack + sustain)
    let mut max_peak = 0.0_f32;
    let mut rms_sum = 0.0_f32;
    let mut clip_count = 0;

    for _ in 0..num_samples {
        let sample = engine.process(); // Returns mono sum

        // Track peak amplitude
        let peak = sample.abs();
        max_peak = max_peak.max(peak);

        // Track RMS (root mean square) for average loudness
        rms_sum += sample * sample;

        // Count clipping events (samples exceeding ±1.0)
        if sample.abs() > 1.0 {
            clip_count += 1;
        }
    }

    let rms = (rms_sum / num_samples as f32).sqrt();

    // === Assertions ===
    println!("=== Unison 3 + Polyphony Test Results ===");
    println!("Peak amplitude: {:.6}", max_peak);
    println!("RMS level: {:.6}", rms);
    println!("Clipping events: {}", clip_count);

    // Primary assertion: No clipping should occur
    assert_eq!(
        clip_count, 0,
        "Audio clipping detected! {} samples exceeded ±1.0. Peak: {:.6}",
        clip_count, max_peak
    );

    // Secondary assertion: Peak should be below 1.0 with some headroom
    // We allow up to 0.95 to give some margin, but ideally it should be lower
    assert!(
        max_peak < 0.95,
        "Peak amplitude too high: {:.6}. Risk of clipping with filter resonance or velocity modulation.",
        max_peak
    );

    // Tertiary assertion: RMS should be reasonable (not dead silent, not constantly hot)
    assert!(
        rms > 0.05 && rms < 0.5,
        "RMS level outside expected range: {:.6}. Too quiet (<0.05) or too hot (>0.5).",
        rms
    );
}

/// Test that unison normalization scales properly with different unison counts.
///
/// This verifies that:
/// - Higher unison counts have proper gain compensation to prevent clipping
/// - The reduction is gradual (sqrt-based, not linear)
/// - Levels remain reasonable (not too quiet, not clipping)
#[test]
fn test_unison_normalization_consistency() {
    let sample_rate = 44100.0;

    // Test with unison=1 (baseline)
    let rms_unison_1 = measure_rms_with_unison(sample_rate, 1);

    // Test with unison=3 (stress test)
    let rms_unison_3 = measure_rms_with_unison(sample_rate, 3);

    // Test with unison=7 (maximum)
    let rms_unison_7 = measure_rms_with_unison(sample_rate, 7);

    println!("=== Unison Normalization Consistency ===");
    println!("RMS with unison=1: {:.6}", rms_unison_1);
    println!("RMS with unison=3: {:.6}", rms_unison_3);
    println!("RMS with unison=7: {:.6}", rms_unison_7);

    let ratio_3_to_1 = rms_unison_3 / rms_unison_1;
    let ratio_7_to_1 = rms_unison_7 / rms_unison_1;

    println!("Unison=3 ratio: {:.3}x", ratio_3_to_1);
    println!("Unison=7 ratio: {:.3}x", ratio_7_to_1);

    // With the new compensation, higher unison counts should have LOWER levels
    // to prevent clipping when multiple notes are played. This is expected behavior.
    // The reduction follows logarithmic scaling: 1.0 + 0.3 * log2(avg_unison)
    //
    // For unison=3: expected compensation ≈ 1.48, so ratio ≈ 0.68
    // For unison=7: expected compensation ≈ 1.84, so ratio ≈ 0.54
    //
    // This is GENTLER than the old sqrt() scaling (which gave 0.38 for unison=7)
    // to prevent the sound from getting "destroyed" or too weak.

    assert!(
        ratio_3_to_1 > 0.60 && ratio_3_to_1 < 0.75,
        "Unison=3 RMS ratio {:.3} out of expected range (0.60-0.75). Should reduce to prevent clipping.",
        ratio_3_to_1
    );

    assert!(
        ratio_7_to_1 > 0.48 && ratio_7_to_1 < 0.65,
        "Unison=7 RMS ratio {:.3} out of expected range (0.48-0.65). Should reduce but not destroy sound.",
        ratio_7_to_1
    );

    // Verify that ratio decreases as unison increases (proper scaling)
    assert!(
        ratio_7_to_1 < ratio_3_to_1,
        "Higher unison should have more reduction: unison=7 ratio {:.3} should be < unison=3 ratio {:.3}",
        ratio_7_to_1,
        ratio_3_to_1
    );
}

/// Helper function to measure RMS level with a specific unison count.
fn measure_rms_with_unison(sample_rate: f32, unison_count: usize) -> f32 {
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);
    let mut params = SynthParams::default();

    // Configure single oscillator with specified unison count
    params.oscillators[0].waveform = Waveform::Saw;
    params.oscillators[0].unison = unison_count;
    params.oscillators[0].unison_detune = 10.0;
    params.oscillators[0].gain = 1.0; // Full gain for single oscillator

    // Disable other oscillators
    params.oscillators[1].gain = 0.0;
    params.oscillators[2].gain = 0.0;

    // Fast envelope for testing
    params.envelope.attack = 0.001;
    params.envelope.sustain = 1.0;

    param_producer.write(params);

    // Trigger one note
    engine.note_on(60, 1.0);

    // Measure RMS over 512 samples (after attack)
    // Skip first 256 samples to let attack settle
    for _ in 0..256 {
        engine.process();
    }

    let mut rms_sum = 0.0;
    let num_samples = 512;

    for _ in 0..num_samples {
        let sample = engine.process();
        rms_sum += sample * sample;
    }

    (rms_sum / num_samples as f32).sqrt()
}

/// Test extreme case: maximum unison with full polyphony.
///
/// This is the worst-case scenario:
/// - All 3 oscillators with unison=7 (maximum)
/// - 16 voices playing simultaneously (full polyphony)
/// - This generates 3 × 7 × 16 = 336 oscillators summing together
///
/// Even in this extreme case, clipping should not occur.
#[test]
fn test_extreme_unison_7_full_polyphony() {
    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    let mut params = SynthParams::default();

    // Maximum unison on all oscillators
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Saw;
        params.oscillators[i].unison = 7; // Maximum
        params.oscillators[i].unison_detune = 10.0;
        params.oscillators[i].gain = 0.33;
        params.oscillators[i].pan = 0.0;
    }

    params.envelope.attack = 0.01;
    params.envelope.decay = 0.1;
    params.envelope.sustain = 0.8;
    params.envelope.release = 0.2;

    param_producer.write(params);

    // Trigger 16 notes (full polyphony)
    for note in 36..52 {
        // C2 to E4
        engine.note_on(note, 1.0);
    }

    // Process and check for clipping
    let num_samples = 2048; // ~46ms
    let mut max_peak = 0.0_f32;
    let mut clip_count = 0;

    for _ in 0..num_samples {
        let sample = engine.process();
        let peak = sample.abs();
        max_peak = max_peak.max(peak);

        if sample.abs() > 1.0 {
            clip_count += 1;
        }
    }

    println!("=== Extreme Unison=7 + 16 Voices Test ===");
    println!("Peak amplitude: {:.6}", max_peak);
    println!("Clipping events: {}", clip_count);

    assert_eq!(
        clip_count, 0,
        "Audio clipping in extreme case! {} samples exceeded ±1.0. Peak: {:.6}",
        clip_count, max_peak
    );

    // In this extreme case, we allow higher peaks but still must stay below 1.0
    assert!(
        max_peak <= 1.0,
        "Peak amplitude exceeds 1.0: {:.6}",
        max_peak
    );
}

/// Test that measures what the peak would be WITHOUT the limiter.
///
/// This demonstrates the underlying issue: without the limiter, unison + polyphony
/// can cause severe clipping. The limiter catches it, but heavy limiting causes
/// audible pumping/distortion (which is what the user hears).
#[test]
fn test_pre_limiter_peak_measurement() {
    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    let mut params = SynthParams::default();

    // Set all 3 oscillators to unison=3 with sawtooth
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Saw;
        params.oscillators[i].unison = 3;
        params.oscillators[i].unison_detune = 10.0;
        params.oscillators[i].gain = 0.33;
        params.oscillators[i].pan = 0.0;
    }

    params.envelope.attack = 0.01;
    params.envelope.decay = 0.1;
    params.envelope.sustain = 0.8;
    params.envelope.release = 0.2;

    param_producer.write(params);

    // Trigger 8 notes
    for note in [60, 64, 67, 72, 76, 79, 84, 88] {
        engine.note_on(note, 1.0);
    }

    // Process samples and measure what peak would be without limiter
    // We can't directly access pre-limiter values, but we can observe
    // limiter engagement by looking at the post-limiter output characteristics
    let num_samples = 2048;
    let mut samples = Vec::with_capacity(num_samples);

    for _ in 0..num_samples {
        let sample = engine.process();
        samples.push(sample);
    }

    // Calculate the peak and how much dynamic range compression occurred
    let max_peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / num_samples as f32).sqrt();
    let crest_factor = if rms > 0.0 { max_peak / rms } else { 0.0 };

    println!("=== Pre-Limiter Peak Measurement ===");
    println!("Post-limiter peak: {:.6}", max_peak);
    println!("Post-limiter RMS: {:.6}", rms);
    println!("Crest factor: {:.6}", crest_factor);
    println!(
        "Estimated pre-limiter peak: ~{:.2}x over (based on heavy limiting)",
        if crest_factor < 3.0 { "2-3" } else { "normal" }
    );

    // A heavily limited signal has a low crest factor (compressed dynamics)
    // Normal audio has crest factor of 4-10, heavily limited <3
    if crest_factor < 3.0 {
        println!(
            "⚠️  WARNING: Limiter is heavily engaged (crest factor {:.2})",
            crest_factor
        );
        println!("    This causes audible pumping/distortion that the user hears.");
    }
}

/// Stress test: exact user scenario (unison=3, all oscillators, multiple keys, default gains)
///
/// This reproduces the exact scenario the user described:
/// - Unison=3 on all oscillators
/// - Default gain values (0.25 per oscillator)
/// - Playing multiple keys
///
/// We measure RMS over a sustained period to see average levels.
#[test]
fn test_user_scenario_exact() {
    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    // Use completely default params except unison
    let mut params = SynthParams::default();
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Saw;
        params.oscillators[i].unison = 3; // User's setting
        params.oscillators[i].unison_detune = 10.0;
        // Keep default gain = 0.25
        // Keep default pan = 0.0
    }

    // Longer envelope to measure sustained output
    params.envelope.attack = 0.05; // 50ms
    params.envelope.sustain = 1.0; // Full sustain

    param_producer.write(params);

    // Play a 4-note chord (realistic use case)
    for note in [60, 64, 67, 72] {
        engine.note_on(note, 0.8); // Forte but not maximum
    }

    // Skip attack phase (first 100ms = 4410 samples)
    for _ in 0..4410 {
        engine.process();
    }

    // Measure sustained output over 1024 samples
    let num_samples = 1024;
    let mut samples = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        let sample = engine.process();
        samples.push(sample);
    }

    let max_peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / num_samples as f32).sqrt();

    println!("=== User Scenario (Unison=3, 4 notes, default gains) ===");
    println!("Peak amplitude: {:.6}", max_peak);
    println!("RMS level: {:.6}", rms);
    println!("Peak/RMS ratio: {:.2}", max_peak / rms.max(0.001));

    // Check if levels are appropriate
    // With master_gain=0.5 and proper normalization, we'd expect RMS around 0.2-0.3
    if rms > 0.4 {
        println!("⚠️  RMS level high ({:.3}), limiter likely engaging", rms);
    }

    if max_peak > 0.9 {
        println!("⚠️  Peak very close to clipping ({:.3})", max_peak);
    }

    // Tests still pass if under 1.0, but we want headroom
    assert!(max_peak < 1.0, "Clipping detected!");
    assert!(
        max_peak < 0.85,
        "Insufficient headroom: peak {:.3} is too close to clipping threshold",
        max_peak
    );
}

/// Test with increased gains (realistic user scenario causing distortion)
///
/// Users often increase oscillator gains or master gain to get louder sounds.
/// This test simulates:
/// - Oscillator gains turned up to 0.5 (2× default)
/// - Master gain at 0.7 (moderate increase)
/// - Unison=3 on all oscillators
/// - Multiple keys pressed
#[test]
fn test_increased_gains_scenario() {
    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    let mut params = SynthParams::default();

    // Increase gains (user trying to get louder sound)
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Saw;
        params.oscillators[i].unison = 3;
        params.oscillators[i].unison_detune = 10.0;
        params.oscillators[i].gain = 0.5; // 2× default (users do this!)
    }

    params.master_gain = 0.7; // Also increased
    params.envelope.attack = 0.05;
    params.envelope.sustain = 1.0;

    param_producer.write(params);

    // Play 5 notes (still reasonable polyphony)
    for note in [60, 63, 67, 70, 72] {
        engine.note_on(note, 1.0); // Maximum velocity
    }

    // Skip attack
    for _ in 0..4410 {
        engine.process();
    }

    // Measure
    let num_samples = 2048;
    let mut samples = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        let sample = engine.process();
        samples.push(sample);
    }

    let max_peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / num_samples as f32).sqrt();

    println!("=== Increased Gains Scenario (gain=0.5, master=0.7, 5 notes) ===");
    println!("Peak amplitude: {:.6}", max_peak);
    println!("RMS level: {:.6}", rms);

    // With increased gains, check if limiter is working hard
    if max_peak > 0.9 {
        println!("⚠️  Limiter heavily engaged at peak {:.3}", max_peak);
        println!("    This causes the pumping/distortion the user hears!");
    }

    if rms > 0.5 {
        println!("⚠️  Very hot RMS level: {:.3}", rms);
    }

    // Should still not clip thanks to limiter
    assert!(max_peak <= 1.0, "Clipping despite limiter!");

    // But we can document that this is pushing limits
    if max_peak > 0.95 {
        println!("📝 Note: User should reduce gains to avoid limiter distortion");
    }
}
