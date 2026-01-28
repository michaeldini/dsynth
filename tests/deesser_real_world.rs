/// Test de-esser with realistic vocal sibilance scenarios
/// This tests what happens with actual vocal characteristics
use dsynth::dsp::effects::dynamics::IntelligentDeEsser;
use dsynth::dsp::signal_analyzer::{SignalAnalysis, SignalAnalyzer};
use std::f32::consts::PI;

/// Simulate a real vocal with periodic sibilance bursts
#[test]
fn test_realistic_vocal_sibilance_reduction() {
    let sample_rate = 44100.0;
    let mut deesser = IntelligentDeEsser::new(sample_rate);
    let mut analyzer = SignalAnalyzer::new_no_pitch(sample_rate);

    println!("\n=== Realistic Vocal Sibilance Test ===\n");

    // Simulate vocal: 200Hz fundamental + harmonics + periodic sibilance bursts
    let fundamental_freq = 200.0;
    let sibilance_freq = 7000.0;

    let mut total_input_high_energy = 0.0f32;
    let mut total_output_high_energy = 0.0f32;
    let mut sibilance_input_high_energy = 0.0f32;
    let mut sibilance_output_high_energy = 0.0f32;
    let mut sibilance_detected_count = 0;
    let mut compression_triggered_count = 0;

    for i in 0..44100 {
        // Time in seconds
        let t = i as f32 / sample_rate;

        // Vowel sound (200Hz + harmonics)
        let vowel = (2.0 * PI * fundamental_freq * t).sin() * 0.3
            + (2.0 * PI * fundamental_freq * 2.0 * t).sin() * 0.15
            + (2.0 * PI * fundamental_freq * 3.0 * t).sin() * 0.08;

        // Sibilance bursts every 0.5 seconds (simulate "ssss" sounds)
        let sibilance_gate = if (t % 0.5) < 0.1 {
            // 100ms sibilance burst
            let burst_progress = (t % 0.5) / 0.1;
            let envelope = (burst_progress * PI).sin(); // Attack/decay envelope
            envelope * 0.4 // Moderate sibilance level
        } else {
            0.0
        };
        let sibilance = (2.0 * PI * sibilance_freq * t).sin() * sibilance_gate;

        // Combined signal
        let input = vowel + sibilance;

        // Analyze signal
        let analysis = analyzer.analyze(input, input);

        // Process through de-esser (threshold=0.6, amount=1.0 for full effect)
        let (output, _) = deesser.process(input, input, 0.6, 1.0, &analysis);

        // Measure high-frequency content (above 6kHz)
        let high_freq_input = (2.0 * PI * 7000.0 * t).sin() * input;
        let high_freq_output = (2.0 * PI * 7000.0 * t).sin() * output;

        total_input_high_energy += high_freq_input * high_freq_input;
        total_output_high_energy += high_freq_output * high_freq_output;

        // Track detection
        if analysis.has_sibilance {
            sibilance_detected_count += 1;

            // Measure reduction only during sibilance
            sibilance_input_high_energy += high_freq_input * high_freq_input;
            sibilance_output_high_energy += high_freq_output * high_freq_output;

            // Check if compression would trigger
            let effective_threshold = 1.0 - 0.6; // 0.4
            if analysis.sibilance_strength >= effective_threshold {
                compression_triggered_count += 1;
            }
        }

        // Print diagnostic info during sibilance bursts
        if sibilance_gate > 0.1 && i % 1000 == 0 {
            println!(
                "t={:.3}s: has_sibilance={}, strength={:.3}, input={:.3}, output={:.3}",
                t, analysis.has_sibilance, analysis.sibilance_strength, input, output
            );
        }
    }

    let input_rms = (total_input_high_energy / 44100.0).sqrt();
    let output_rms = (total_output_high_energy / 44100.0).sqrt();
    let reduction_db = 20.0 * (output_rms / input_rms).log10();

    let sibilance_input_rms =
        (sibilance_input_high_energy / sibilance_detected_count.max(1) as f32).sqrt();
    let sibilance_output_rms =
        (sibilance_output_high_energy / sibilance_detected_count.max(1) as f32).sqrt();
    let sibilance_reduction_db = 20.0 * (sibilance_output_rms / sibilance_input_rms).log10();

    println!("\n--- Results ---");
    println!(
        "Sibilance detected: {}/{} samples ({:.1}%)",
        sibilance_detected_count,
        44100,
        (sibilance_detected_count as f32 / 44100.0) * 100.0
    );
    println!(
        "Compression triggered: {}/{} samples ({:.1}%)",
        compression_triggered_count,
        44100,
        (compression_triggered_count as f32 / 44100.0) * 100.0
    );
    println!("\n--- Overall Signal ---");
    println!("Input high-freq RMS: {:.4}", input_rms);
    println!("Output high-freq RMS: {:.4}", output_rms);
    println!("Reduction: {:.2} dB", reduction_db);
    println!("\n--- During Sibilance Only ---");
    println!("Input high-freq RMS: {:.4}", sibilance_input_rms);
    println!("Output high-freq RMS: {:.4}", sibilance_output_rms);
    println!("Reduction: {:.2} dB", sibilance_reduction_db);
    println!("\n=== End Test ===\n");

    // We should see at least 1dB of reduction during sibilance bursts
    assert!(
        sibilance_reduction_db < -1.0,
        "Expected at least 1dB reduction during sibilance, got {:.2}dB",
        sibilance_reduction_db
    );
}

/// Test if de-esser parameters are being applied correctly
#[test]
fn test_parameter_application() {
    let sample_rate = 44100.0;
    let mut deesser = IntelligentDeEsser::new(sample_rate);

    // Create analysis with strong sibilance
    let mut analysis = SignalAnalysis::default();
    analysis.has_sibilance = true;
    analysis.sibilance_strength = 0.8;

    println!("\n=== Parameter Application Test ===\n");

    // Test 1: amount=0 should bypass
    let input = 0.5;
    let (output_bypass, _) = deesser.process(input, input, 0.5, 0.0, &analysis);
    println!(
        "amount=0.0: input={:.4}, output={:.4}",
        input, output_bypass
    );
    assert!(
        (output_bypass - input).abs() < 0.001,
        "amount=0 should be bit-perfect bypass"
    );

    // Test 2: amount=1.0 should apply full processing
    let (output_full, _) = deesser.process(input, input, 0.5, 1.0, &analysis);
    println!("amount=1.0: input={:.4}, output={:.4}", input, output_full);

    // Test 3: threshold variations
    for threshold in [0.0, 0.3, 0.6, 0.9] {
        let (output, _) = deesser.process(input, input, threshold, 1.0, &analysis);
        let effective_threshold = 1.0 - threshold;
        let will_trigger = analysis.sibilance_strength >= effective_threshold;
        println!(
            "threshold={:.1}: effective_threshold={:.1}, will_trigger={}, output={:.4}",
            threshold, effective_threshold, will_trigger, output
        );
    }

    println!("\n=== End Test ===\n");
}
