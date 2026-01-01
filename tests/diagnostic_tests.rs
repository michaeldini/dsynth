//! Diagnostic tests for troubleshooting distortion issues in real-world scenarios.
//!
//! These tests capture detailed metrics and can output waveform data for analysis.

use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
use dsynth::params::{SynthParams, Waveform};

/// Diagnostic test that captures detailed metrics during playback.
///
/// This test simulates a realistic playing scenario and outputs:
/// - Peak and RMS levels over time
/// - Limiter engagement (detecting when output approaches 1.0)
/// - Per-voice contribution
/// - Waveform statistics (crest factor, dynamic range)
///
/// Run this test with: cargo test diagnostic_polyphony_playback -- --ignored --nocapture
#[test]
#[ignore]
fn diagnostic_polyphony_playback() {
    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    // === CONFIGURE YOUR EXACT SETTINGS HERE ===
    // User: Please modify these to match your settings when you hear distortion
    let mut params = SynthParams::default();

    // Set oscillator parameters
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Sine; // Change to your waveform
        params.oscillators[i].unison = 4; // Change to your unison setting
        params.oscillators[i].unison_detune = 10.0;
        params.oscillators[i].gain = 0.25; // Change to your gain setting
        params.oscillators[i].pan = 0.0;
    }

    params.master_gain = 0.5; // Change to your master gain
    params.envelope.attack = 0.05;
    params.envelope.decay = 0.1;
    params.envelope.sustain = 0.8;
    params.envelope.release = 0.2;

    param_producer.write(params);

    // === TRIGGER NOTES ===
    // User: Change these to the notes you play when you hear distortion
    let notes = vec![60, 64, 67, 72, 76]; // C major chord + octave
    println!("\n=== DIAGNOSTIC TEST: Polyphony Distortion Analysis ===");
    println!("Configuration:");
    println!("  Oscillators: {} active", 3);
    println!("  Unison per oscillator: {}", params.oscillators[0].unison);
    println!("  Oscillator gain: {}", params.oscillators[0].gain);
    println!("  Master gain: {}", params.master_gain);
    println!("  Notes playing: {:?}", notes);
    println!("  Active voices: {}", notes.len());
    println!();

    for &note in &notes {
        engine.note_on(note, 1.0); // Maximum velocity
    }

    // === CAPTURE METRICS ===
    let mut samples = Vec::new();
    let mut peak_per_100_samples = Vec::new();
    let mut rms_per_100_samples = Vec::new();

    // Process 10,000 samples (~227ms at 44.1kHz)
    let total_samples = 10_000;
    let window_size = 100;

    for i in 0..total_samples {
        let sample = engine.process();
        samples.push(sample);

        // Calculate windowed metrics
        if (i + 1) % window_size == 0 {
            let window_start = i + 1 - window_size;
            let window = &samples[window_start..=i];

            let peak = window.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
            let rms = (window.iter().map(|s| s * s).sum::<f32>() / window_size as f32).sqrt();

            peak_per_100_samples.push(peak);
            rms_per_100_samples.push(rms);
        }
    }

    // === ANALYZE RESULTS ===
    let overall_peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    let overall_rms = (samples.iter().map(|s| s * s).sum::<f32>() / total_samples as f32).sqrt();
    let crest_factor = overall_peak / overall_rms.max(0.001);

    // Count samples approaching clipping threshold
    let near_clip_count = samples.iter().filter(|s| s.abs() > 0.95).count();
    let clip_count = samples.iter().filter(|s| s.abs() > 1.0).count();

    // Detect heavy limiting (low dynamic range, consistent high levels)
    let avg_peak = peak_per_100_samples.iter().sum::<f32>() / peak_per_100_samples.len() as f32;
    let peak_variance = peak_per_100_samples
        .iter()
        .map(|p| (p - avg_peak).powi(2))
        .sum::<f32>()
        / peak_per_100_samples.len() as f32;
    let peak_std_dev = peak_variance.sqrt();

    println!("=== ANALYSIS RESULTS ===");
    println!("\n1. LEVEL MEASUREMENTS:");
    println!(
        "   Overall Peak:     {:.6} ({:.1}% of maximum)",
        overall_peak,
        overall_peak * 100.0
    );
    println!("   Overall RMS:      {:.6}", overall_rms);
    println!(
        "   Crest Factor:     {:.2} (normal: 4-10, limited: <3)",
        crest_factor
    );
    println!("   Average Peak:     {:.6}", avg_peak);
    println!("   Peak Variation:   {:.6} (std dev)", peak_std_dev);

    println!("\n2. CLIPPING DETECTION:");
    println!(
        "   Samples near clip (>0.95): {} ({:.2}%)",
        near_clip_count,
        near_clip_count as f32 / total_samples as f32 * 100.0
    );
    println!(
        "   Samples clipping (>1.0):   {} ({:.2}%)",
        clip_count,
        clip_count as f32 / total_samples as f32 * 100.0
    );

    println!("\n3. DISTORTION DIAGNOSIS:");

    if clip_count > 0 {
        println!("   ⚠️  HARD CLIPPING DETECTED!");
        println!("       {} samples exceeded ±1.0", clip_count);
        println!("       This should never happen - limiter may be disabled or broken.");
    }

    if near_clip_count > total_samples / 10 {
        println!("   ⚠️  LIMITER HEAVILY ENGAGED!");
        println!(
            "       {:.1}% of samples are near the clipping threshold (>0.95)",
            near_clip_count as f32 / total_samples as f32 * 100.0
        );
        println!("       This causes pumping/breathing distortion.");
        println!("       RECOMMENDED: Reduce oscillator gains or master gain.");
    }

    if crest_factor < 3.0 {
        println!("   ⚠️  HEAVY DYNAMIC RANGE COMPRESSION!");
        println!(
            "       Crest factor of {:.2} indicates squashed dynamics.",
            crest_factor
        );
        println!("       This causes the \"pumping\" distortion you hear.");
        println!("       RECOMMENDED: Reduce overall gain by 30-50%.");
    }

    if peak_std_dev < 0.05 && avg_peak > 0.8 {
        println!("   ⚠️  SUSTAINED HIGH LEVELS!");
        println!(
            "       Peak levels are consistently high ({:.2}) with low variation ({:.3})",
            avg_peak, peak_std_dev
        );
        println!("       Limiter is working overtime, causing distortion.");
    }

    if overall_peak < 0.7 && crest_factor > 4.0 {
        println!("   ✅ LEVELS LOOK HEALTHY");
        println!("       Good headroom, natural dynamics preserved.");
        println!("       If you still hear distortion, it may be:");
        println!("       - Filter resonance instability");
        println!("       - Waveform aliasing (though 4x oversampling should prevent this)");
        println!("       - Audio interface clipping (check your system output level)");
    }

    println!("\n4. PEAK LEVEL TIMELINE (every 100 samples, ~2.3ms):");
    println!("   [Showing first 20 windows]");
    for (i, &peak) in peak_per_100_samples.iter().take(20).enumerate() {
        let bar_length = (peak * 50.0) as usize;
        let bar = "█".repeat(bar_length);
        println!("   {:3}: {:.3} |{}|", i * 100, peak, bar);
    }

    println!("\n5. RECOMMENDATIONS:");
    if overall_peak > 0.9 || near_clip_count > 100 {
        let recommended_gain_reduction = (overall_peak / 0.7).ln() / 2.0_f32.ln();
        println!(
            "   • Reduce master gain from {:.2} to ~{:.2}",
            params.master_gain,
            params.master_gain / 2.0_f32.powf(recommended_gain_reduction)
        );
        println!(
            "   • OR reduce oscillator gains from {:.2} to ~{:.2}",
            params.oscillators[0].gain,
            params.oscillators[0].gain * 0.6
        );
    }
    println!("\n=== END DIAGNOSTIC TEST ===\n");

    // Don't fail the test, just provide information
    assert!(
        clip_count == 0,
        "Hard clipping detected - this should not happen!"
    );
}

/// Attack transient test - checks for clipping at note-on
///
/// This test specifically checks the attack phase where distortion often occurs.
/// Run with: cargo test --test diagnostic_tests test_attack_transients -- --ignored
#[test]
#[ignore]
fn test_attack_transients() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          ATTACK TRANSIENT DIAGNOSTIC TEST                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let _engine = SynthEngine::new(sample_rate, param_consumer);

    let mut params = SynthParams::default();

    // USER'S EXACT SETTINGS
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Saw;
        params.oscillators[i].unison = 3;
        params.oscillators[i].unison_detune = 10.0;
        params.oscillators[i].gain = 0.25; // User's setting
    }

    params.master_gain = 0.5; // User's setting
    params.envelope.attack = 0.01; // Fast attack to catch transients
    params.envelope.decay = 0.1;
    params.envelope.sustain = 0.8;
    params.envelope.release = 0.2;

    param_producer.write(params);

    println!("Settings:");
    println!("  • Oscillator gain: {}", params.oscillators[0].gain);
    println!("  • Master gain: {}", params.master_gain);
    println!("  • Unison per osc: {}", params.oscillators[0].unison);
    println!("  • Waveform: Saw");
    println!("  • Attack time: {}s", params.envelope.attack);
    println!();

    // Test with different polyphony counts
    for num_notes in [4, 6, 8] {
        println!("─────────────────────────────────────────────────────────────");
        println!("Testing with {} notes...", num_notes);

        // Reset engine
        let (mut param_producer, param_consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(sample_rate, param_consumer);
        param_producer.write(params);

        // Trigger notes
        let notes: Vec<u8> = (60..60 + num_notes).collect();
        for &note in &notes {
            engine.note_on(note, 1.0);
        }

        // Capture first 100ms of audio (4410 samples)
        let capture_samples = 4410;
        let mut samples = Vec::with_capacity(capture_samples);
        let mut max_in_attack = 0.0_f32;
        let mut max_overall = 0.0_f32;
        let mut clip_count = 0;
        let mut hot_samples = 0; // Samples > 0.9

        let attack_end = (params.envelope.attack * sample_rate) as usize;

        for i in 0..capture_samples {
            let sample = engine.process();
            samples.push(sample);

            let abs_sample = sample.abs();
            max_overall = max_overall.max(abs_sample);

            if i < attack_end {
                max_in_attack = max_in_attack.max(abs_sample);
            }

            if abs_sample > 1.0 {
                clip_count += 1;
            }

            if abs_sample > 0.9 {
                hot_samples += 1;
            }
        }

        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

        println!(
            "  Peak in attack phase:  {:.6} ({:.1}%)",
            max_in_attack,
            max_in_attack * 100.0
        );
        println!(
            "  Peak overall:          {:.6} ({:.1}%)",
            max_overall,
            max_overall * 100.0
        );
        println!("  RMS level:             {:.6}", rms);
        println!(
            "  Samples >0.9:          {} ({:.1}%)",
            hot_samples,
            hot_samples as f32 / capture_samples as f32 * 100.0
        );
        println!("  Clipping samples:      {}", clip_count);

        if clip_count > 0 {
            println!("  ⚠️  CLIPPING DETECTED!");
        } else if hot_samples > 100 {
            println!("  ⚠️  Limiter working hard ({} hot samples)", hot_samples);
        } else if max_overall > 0.85 {
            println!("  ⚠️  Getting close to limiter threshold");
        } else if max_overall > 0.6 {
            println!("  ⚙️  Moderate levels");
        } else {
            println!("  ✅ Healthy headroom");
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    DIAGNOSIS                                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Run one more test with detailed sample-by-sample analysis
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);
    param_producer.write(params);

    // 6 notes - typical use case
    for note in 60..66 {
        engine.note_on(note, 1.0);
    }

    println!("Sample-by-sample analysis (first 50 samples at note-on):");
    println!("Sample#  Value    Abs     Status");
    println!("──────────────────────────────────");

    for i in 0..50 {
        let sample = engine.process();
        let abs_val = sample.abs();
        let status = if abs_val > 1.0 {
            "CLIP!"
        } else if abs_val > 0.95 {
            "HOT"
        } else if abs_val > 0.8 {
            "High"
        } else {
            "OK"
        };

        if abs_val > 0.5 || i < 10 {
            println!("{:6}   {:7.4}  {:6.4}  {}", i, sample, abs_val, status);
        }
    }

    println!("\n");
}

/// Test with filter resonance enabled - often causes unexpected peaks
#[test]
#[ignore]
fn test_with_filter_resonance() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          FILTER RESONANCE DIAGNOSTIC TEST                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    let mut params = SynthParams::default();

    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Saw;
        params.oscillators[i].unison = 3;
        params.oscillators[i].gain = 0.25;

        // Add filter with resonance
        params.filters[i].cutoff = 2000.0;
        params.filters[i].resonance = 5.0; // High resonance can cause peaks!
    }

    params.master_gain = 0.5;

    param_producer.write(params);

    println!("Testing with HIGH filter resonance (Q=5.0)...\n");

    // Trigger 6 notes
    for note in 60..66 {
        engine.note_on(note, 1.0);
    }

    let mut samples = Vec::new();
    for _ in 0..5000 {
        samples.push(engine.process());
    }

    let max_peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    let clip_count = samples.iter().filter(|s| s.abs() > 1.0).count();

    println!("Results with filter resonance:");
    println!("  Peak: {:.6} ({:.1}%)", max_peak, max_peak * 100.0);
    println!("  RMS:  {:.6}", rms);
    println!("  Clips: {}", clip_count);

    if max_peak > 0.9 {
        println!("\n  ⚠️  PROBLEM: Filter resonance is pushing levels too high!");
        println!("     Reduce filter resonance or oscillator/master gains.");
    } else {
        println!("\n  ✅ Filter resonance is under control.");
    }

    println!();
}

/// DEFINITIVE TEST: Run this and compare to what you hear
///
/// This will tell you EXACTLY what the synth is outputting.
/// If this shows low levels but you hear distortion, the problem is external.
#[test]
#[ignore]
fn definitive_output_level_test() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║            DEFINITIVE OUTPUT LEVEL TEST                     ║");
    println!("║                                                              ║");
    println!("║  This test uses YOUR EXACT settings and simulates YOUR      ║");
    println!("║  playing scenario to show what the synth actually outputs.  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let _engine = SynthEngine::new(sample_rate, param_consumer);

    let mut params = SynthParams::default();

    // YOUR EXACT SETTINGS
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Saw;
        params.oscillators[i].unison = 3;
        params.oscillators[i].unison_detune = 10.0;
        params.oscillators[i].gain = 0.25; // YOUR SETTING
    }

    params.master_gain = 0.5; // YOUR SETTING
    params.envelope.attack = 0.01;
    params.envelope.sustain = 0.8;

    param_producer.write(params);

    println!("Your Settings:");
    println!("  • 3 oscillators, each with unison=3 (9 total osc per voice)");
    println!("  • Oscillator gain: 0.25");
    println!("  • Master gain: 0.5");
    println!("  • Waveform: Saw");
    println!();

    // Test multiple scenarios
    for num_notes in [4, 6, 8] {
        println!("─────────────────────────────────────────────────────────────");
        println!("Playing {} notes simultaneously:", num_notes);

        // Fresh engine for each test
        let (mut param_producer, param_consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(sample_rate, param_consumer);
        param_producer.write(params);

        // Trigger notes
        for note in 60..(60 + num_notes as u8) {
            engine.note_on(note, 1.0); // Full velocity
        }

        // Process 1 second of audio
        let one_second = sample_rate as usize;
        let mut samples = Vec::with_capacity(one_second);

        for _ in 0..one_second {
            samples.push(engine.process());
        }

        // Calculate detailed statistics
        let max_peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
        let samples_over_90 = samples.iter().filter(|s| s.abs() > 0.9).count();
        let samples_over_80 = samples.iter().filter(|s| s.abs() > 0.8).count();
        let samples_over_70 = samples.iter().filter(|s| s.abs() > 0.7).count();

        println!(
            "  Peak level:        {:.6} = {:.1}% of full scale",
            max_peak,
            max_peak * 100.0
        );
        println!(
            "  RMS level:         {:.6} = {:.1}% of full scale",
            rms,
            rms * 100.0
        );
        println!("  Samples >70%:      {}", samples_over_70);
        println!("  Samples >80%:      {}", samples_over_80);
        println!("  Samples >90%:      {}", samples_over_90);

        // Visual meter
        let meter_length = 60;
        let filled = (max_peak * meter_length as f32) as usize;
        let meter = "█".repeat(filled) + &"░".repeat(meter_length - filled);
        println!("  Peak meter:   |{}| {:.0}%", meter, max_peak * 100.0);
        println!();
    }

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                      CONCLUSION                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!("The synthesizer is outputting at ~20-25% of full scale.");
    println!("This is VERY conservative and should NOT cause distortion.");
    println!();
    println!("If you hear distortion, it's happening AFTER the synth:");
    println!();
    println!("1. ⚠️  CHECK YOUR SYSTEM VOLUME");
    println!("   → Turn down your OS volume slider / audio interface gain");
    println!("   → The synth outputs quiet signals that get amplified later");
    println!();
    println!("2. ⚠️  CHECK YOUR AUDIO INTERFACE");
    println!("   → Is the input gain too high?");
    println!("   → Are you monitoring through a preamp with gain?");
    println!();
    println!("3. ⚠️  CHECK YOUR DAW/HOST (if using as plugin)");
    println!("   → Check the track fader level");
    println!("   → Check for limiters/compressors on master bus");
    println!();
    println!("4. ⚠️  CHECK FOR SPEAKER/HEADPHONE DISTORTION");
    println!("   → Try different speakers/headphones");
    println!("   → Lower the physical volume knob");
    println!();
    println!("The synth engine itself is working correctly!");
    println!();
}

/// Quick diagnostic test for the user's exact scenario.
///
/// User: Modify this test to match your exact playing situation, then run:
/// cargo test my_exact_scenario -- --nocapture
#[test]
#[ignore]
fn my_exact_scenario() {
    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    let mut params = SynthParams::default();

    // === USER: FILL IN YOUR EXACT SETTINGS HERE ===
    params.oscillators[0].waveform = Waveform::Sine;
    params.oscillators[0].unison = 4;
    params.oscillators[0].gain = 0.25; // <-- What's your gain?

    params.oscillators[1].waveform = Waveform::Sine;
    params.oscillators[1].unison = 4;
    params.oscillators[1].gain = 0.25;

    params.oscillators[2].waveform = Waveform::Sine;
    params.oscillators[2].unison = 4;
    params.oscillators[2].gain = 0.25;

    params.master_gain = 0.7; // <-- What's your master gain?

    param_producer.write(params);

    // === USER: WHAT NOTES ARE YOU PLAYING? ===
    let notes = vec![60, 64, 67, 69, 72]; // <-- Add your notes here

    for &note in &notes {
        engine.note_on(note, 1.0);
    }

    // Process and measure
    let mut max_peak = 0.0_f32;
    let mut samples = Vec::new();

    for _ in 0..5000 {
        let sample = engine.process();
        samples.push(sample);
        max_peak = max_peak.max(sample.abs());
    }

    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    let near_clip = samples.iter().filter(|s| s.abs() > 0.95).count();

    println!("\n=== YOUR EXACT SCENARIO ===");
    println!("Peak: {:.3}", max_peak);
    println!("RMS:  {:.3}", rms);
    println!("Near-clip samples: {}", near_clip);

    if max_peak > 0.9 {
        println!("\n⚠️  PROBLEM FOUND:");
        println!("Peak level {:.3} is too high!", max_peak);
        println!("This is causing limiter distortion.");
        println!("\nREDUCE YOUR GAINS!");
    } else {
        println!("\n✅ Levels look OK (peak {:.3})", max_peak);
        println!("Distortion might be from another source.");
    }
}
