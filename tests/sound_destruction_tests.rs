//! Tests specifically for "sound destruction" issues - phase cancellation, dropouts, etc.

use dsynth::audio::engine::{create_parameter_buffer, SynthEngine};
use dsynth::params::{SynthParams, Waveform};

/// Test for phase cancellation with sine waves + high unison
///
/// When multiple sine waves at similar frequencies are combined with phase offsets,
/// they can cancel each other out, causing the sound to "disappear" or get "destroyed".
#[test]
fn test_sine_wave_phase_cancellation() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         SINE WAVE PHASE CANCELLATION TEST                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let sample_rate = 44100.0;

    // Test different unison counts
    for unison_count in [1, 3, 5, 7] {
        println!("─────────────────────────────────────────────────────────────");
        println!("Testing with SINE wave, unison={}", unison_count);

        let (mut param_producer, param_consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(sample_rate, param_consumer);

        let mut params = SynthParams::default();

        // PURE SINE WAVES - most susceptible to phase cancellation
        for i in 0..3 {
            params.oscillators[i].waveform = Waveform::Sine;
            params.oscillators[i].unison = unison_count;
            params.oscillators[i].unison_detune = 10.0; // Small detune
            params.oscillators[i].gain = 0.5; // Higher gain to make issues obvious
        }

        params.master_gain = 0.7;
        params.envelope.attack = 0.001; // Nearly instant
        params.envelope.sustain = 1.0;

        param_producer.write(params);

        // Play 4 notes (realistic scenario)
        for note in [60, 64, 67, 72] {
            engine.note_on(note, 1.0);
        }

        // Let attack finish (100 samples)
        for _ in 0..100 {
            engine.process_mono();
        }

        // Now measure steady-state output
        let mut samples = Vec::new();
        for _ in 0..5000 {
            samples.push(engine.process_mono());
        }

        let max_peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

        // Check for dropouts (samples that are suspiciously quiet)
        let very_quiet_samples = samples.iter().filter(|s| s.abs() < 0.001).count();
        let dropout_percentage = very_quiet_samples as f32 / samples.len() as f32 * 100.0;

        // Check for "nulls" - periods where amplitude drops significantly
        let mut null_count = 0;
        let window_size = 100;
        for i in 0..(samples.len() - window_size) {
            let window_rms = (samples[i..i + window_size]
                .iter()
                .map(|s| s * s)
                .sum::<f32>()
                / window_size as f32)
                .sqrt();
            if window_rms < rms * 0.1 {
                // Drop to 10% of average
                null_count += 1;
            }
        }

        println!("  Peak amplitude:     {:.6}", max_peak);
        println!("  RMS amplitude:      {:.6}", rms);
        println!(
            "  Very quiet samples: {} ({:.1}%)",
            very_quiet_samples, dropout_percentage
        );
        println!("  Detected nulls:     {} windows", null_count);

        if rms < 0.01 {
            println!("  ⚠️  RMS IS EXTREMELY LOW - Sound is nearly inaudible!");
            println!("      Likely cause: Aggressive normalization or phase cancellation");
        } else if dropout_percentage > 20.0 {
            println!("  ⚠️  LOTS OF DROPOUTS - Sound is getting destroyed");
        } else if null_count > 100 {
            println!("  ⚠️  MANY NULLS DETECTED - Phase cancellation is severe");
        } else if rms < 0.05 {
            println!(
                "  ⚠️  RMS is quite low ({:.3}) - Sound might be too quiet",
                rms
            );
        } else {
            println!("  ✅ Sound output looks healthy");
        }

        println!();
    }
}

/// Test the normalization math directly
///
/// Check if our sqrt() compensation is too aggressive
#[test]
fn test_normalization_aggressiveness() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         NORMALIZATION AGGRESSIVENESS TEST                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("Current normalization formula:");
    println!("  osc_normalization = 1.0 / (active_osc_count * sqrt(avg_unison))");
    println!();
    println!("Expected output levels:");
    println!();
    println!("Config                        Normalization  Effective Gain");
    println!("─────────────────────────────────────────────────────────────");

    for active_osc in [1, 2, 3] {
        for unison in [1, 3, 5, 7] {
            let avg_unison = unison as f32;
            let unison_comp = avg_unison.sqrt();
            let normalization = 1.0 / (active_osc as f32 * unison_comp);

            // With gain=0.5 per oscillator
            let effective_gain = 0.5 * normalization * active_osc as f32;

            println!(
                "{} osc × unison={}            {:.4}         {:.4}",
                active_osc, unison, normalization, effective_gain
            );
        }
        println!();
    }

    println!("Analysis:");
    println!("  • With 3 osc × unison=7: normalization = 0.1260");
    println!("  • This means each oscillator is reduced to 12.6% of its original level");
    println!("  • With gain=0.5, effective output = 0.189 per voice");
    println!("  • This might be TOO aggressive and make sound seem 'destroyed'");
    println!();
}

/// Test for sound consistency - should not have dropouts or nulls
#[test]
fn test_sound_consistency_over_time() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         SOUND CONSISTENCY TEST                               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let sample_rate = 44100.0;
    let (mut param_producer, param_consumer) = create_parameter_buffer();
    let mut engine = SynthEngine::new(sample_rate, param_consumer);

    let mut params = SynthParams::default();

    // User's scenario: sine waves, high unison, multiple keys
    for i in 0..3 {
        params.oscillators[i].waveform = Waveform::Sine;
        params.oscillators[i].unison = 7; // HIGH unison
        params.oscillators[i].unison_detune = 10.0;
        params.oscillators[i].gain = 0.5; // Higher to hear issues
    }

    params.master_gain = 0.7;
    params.envelope.attack = 0.001;
    params.envelope.sustain = 1.0;

    param_producer.write(params);

    // Play 6 notes
    for note in [60, 62, 64, 67, 69, 72] {
        engine.note_on(note, 1.0);
    }

    // Skip attack
    for _ in 0..200 {
        engine.process_mono();
    }

    // Capture 10 seconds of audio and analyze for consistency
    let ten_seconds = sample_rate as usize * 10;
    let mut samples = Vec::with_capacity(ten_seconds);

    println!("Capturing 10 seconds of audio with 6 notes playing...");
    println!("(sine waves, unison=7, 3 oscillators)\n");

    for _ in 0..ten_seconds {
        samples.push(engine.process_mono());
    }

    // Analyze in 1-second windows
    let window_size = sample_rate as usize;
    println!("RMS per second:");
    println!("Sec   RMS      Status");
    println!("───────────────────────");

    let mut all_rms = Vec::new();
    for i in 0..10 {
        let start = i * window_size;
        let end = start + window_size;
        let window = &samples[start..end];

        let rms = (window.iter().map(|s| s * s).sum::<f32>() / window_size as f32).sqrt();
        all_rms.push(rms);

        let status = if rms < 0.01 {
            "DESTROYED"
        } else if rms < 0.05 {
            "Very weak"
        } else if rms < 0.1 {
            "Weak"
        } else {
            "OK"
        };

        println!("{:2}    {:.5}  {}", i + 1, rms, status);
    }

    let avg_rms = all_rms.iter().sum::<f32>() / all_rms.len() as f32;
    let variance =
        all_rms.iter().map(|r| (r - avg_rms).powi(2)).sum::<f32>() / all_rms.len() as f32;
    let std_dev = variance.sqrt();

    println!();
    println!("Statistics:");
    println!("  Average RMS:  {:.5}", avg_rms);
    println!("  Std Dev:      {:.5}", std_dev);
    println!(
        "  Consistency:  {:.1}%",
        (1.0 - std_dev / avg_rms.max(0.001)) * 100.0
    );
    println!();

    if avg_rms < 0.05 {
        println!("⚠️  PROBLEM FOUND: Sound is extremely weak!");
        println!("    RMS of {:.3} is barely audible.", avg_rms);
        println!("    The normalization is TOO aggressive for high unison counts.");
        println!();
        println!("    SOLUTION: Reduce the sqrt() compensation or use different scaling.");
    } else if std_dev > avg_rms * 0.3 {
        println!("⚠️  PROBLEM: Sound is inconsistent - lots of variation!");
        println!("    This could sound like the audio is 'dropping out' or 'pulsing'.");
    } else {
        println!("✅ Sound is consistent and at reasonable levels.");
    }
}

/// Direct comparison: unison=1 vs unison=7 to see the difference
#[test]
fn test_unison_output_comparison() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         UNISON OUTPUT LEVEL COMPARISON                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let sample_rate = 44100.0;

    println!("Playing same notes (60, 64, 67, 72) with different unison:\n");

    for unison in [1, 3, 5, 7] {
        let (mut param_producer, param_consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(sample_rate, param_consumer);

        let mut params = SynthParams::default();

        for i in 0..3 {
            params.oscillators[i].waveform = Waveform::Sine;
            params.oscillators[i].unison = unison;
            params.oscillators[i].gain = 0.5;
        }

        params.master_gain = 0.7;
        params.envelope.attack = 0.001;
        params.envelope.sustain = 1.0;

        param_producer.write(params);

        for note in [60, 64, 67, 72] {
            engine.note_on(note, 1.0);
        }

        // Skip attack
        for _ in 0..200 {
            engine.process_mono();
        }

        // Measure sustained output
        let mut samples = Vec::new();
        for _ in 0..5000 {
            samples.push(engine.process_mono());
        }

        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
        let peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);

        let meter_len = 50_usize;
        let filled = (rms * 5.0 * meter_len as f32).min(meter_len as f32) as usize;
        let meter = "█".repeat(filled) + &"░".repeat(meter_len.saturating_sub(filled));

        println!(
            "Unison={}: Peak={:.4} RMS={:.4} |{}|",
            unison, peak, rms, meter
        );
    }

    println!();
    println!("What you should see:");
    println!("  • RMS should stay relatively similar across unison counts");
    println!("  • If RMS drops drastically with high unison, normalization is too aggressive");
    println!("  • If unison=7 sounds 'destroyed', it's probably too quiet");
    println!();
}
