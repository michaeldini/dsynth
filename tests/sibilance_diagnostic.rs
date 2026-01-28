/// Diagnostic test to understand sibilance detection strength values
///
/// This helps us understand what sibilance_strength values we get in practice
/// and whether the de-esser is configured appropriately.
use dsynth::dsp::analysis::SibilanceDetector;
use std::f32::consts::PI;

#[test]
fn diagnose_sibilance_strength_values() {
    let mut detector = SibilanceDetector::new(44100.0);
    detector.set_threshold(0.0); // Accept all detections
    let sample_rate = 44100.0;

    println!("\n=== Sibilance Detection Diagnostics ===\n");

    // Test 1: Pure 6kHz tone (peak sibilance frequency)
    println!("Test 1: Pure 6kHz tone at 0.5 amplitude");
    detector.reset();
    for _ in 0..500 {
        // Warm up
        detector.process(0.0);
    }
    let mut max_strength = 0.0f32;
    for i in 0..2000 {
        let phase = i as f32 / sample_rate;
        let signal = (2.0 * PI * 6000.0 * phase).sin() * 0.5;
        let (_, strength) = detector.process(signal);
        max_strength = max_strength.max(strength);
    }
    println!("  → Max sibilance_strength: {:.4}", max_strength);

    // Test 2: Broadband vocal with sibilance (fundamental + harmonics + high freq)
    println!("\nTest 2: Broadband vocal simulation (200Hz + harmonics + 7kHz sibilance)");
    detector.reset();
    for _ in 0..500 {
        detector.process(0.0);
    }
    max_strength = 0.0;
    for i in 0..2000 {
        let phase = i as f32 / sample_rate;
        let fundamental = (2.0 * PI * 200.0 * phase).sin() * 0.4;
        let harmonic2 = (2.0 * PI * 400.0 * phase).sin() * 0.2;
        let harmonic3 = (2.0 * PI * 600.0 * phase).sin() * 0.1;
        let sibilance = (2.0 * PI * 7000.0 * phase).sin() * 0.4; // Strong sibilance
        let signal = fundamental + harmonic2 + harmonic3 + sibilance;
        let (_, strength) = detector.process(signal);
        max_strength = max_strength.max(strength);
    }
    println!("  → Max sibilance_strength: {:.4}", max_strength);

    // Test 3: Broadband vocal with WEAK sibilance
    println!("\nTest 3: Broadband vocal with WEAK sibilance (7kHz at 0.15 amplitude)");
    detector.reset();
    for _ in 0..500 {
        detector.process(0.0);
    }
    max_strength = 0.0;
    for i in 0..2000 {
        let phase = i as f32 / sample_rate;
        let fundamental = (2.0 * PI * 200.0 * phase).sin() * 0.4;
        let harmonic2 = (2.0 * PI * 400.0 * phase).sin() * 0.2;
        let harmonic3 = (2.0 * PI * 600.0 * phase).sin() * 0.1;
        let sibilance = (2.0 * PI * 7000.0 * phase).sin() * 0.15; // Weak sibilance
        let signal = fundamental + harmonic2 + harmonic3 + sibilance;
        let (_, strength) = detector.process(signal);
        max_strength = max_strength.max(strength);
    }
    println!("  → Max sibilance_strength: {:.4}", max_strength);

    // Test 4: Very loud sibilance ("SSSSS" sound)
    println!("\nTest 4: Very loud pure sibilance (7kHz at 0.8 amplitude)");
    detector.reset();
    for _ in 0..500 {
        detector.process(0.0);
    }
    max_strength = 0.0;
    for i in 0..2000 {
        let phase = i as f32 / sample_rate;
        let signal = (2.0 * PI * 7000.0 * phase).sin() * 0.8;
        let (_, strength) = detector.process(signal);
        max_strength = max_strength.max(strength);
    }
    println!("  → Max sibilance_strength: {:.4}", max_strength);

    // Test 5: High-frequency noise burst (simulates "ts" or "ch")
    println!("\nTest 5: High-frequency noise burst (4-10kHz mixed)");
    detector.reset();
    for _ in 0..500 {
        detector.process(0.0);
    }
    max_strength = 0.0;
    for i in 0..2000 {
        let phase = i as f32 / sample_rate;
        let f1 = (2.0 * PI * 5000.0 * phase).sin() * 0.2;
        let f2 = (2.0 * PI * 7000.0 * phase).sin() * 0.3;
        let f3 = (2.0 * PI * 9000.0 * phase).sin() * 0.2;
        let signal = f1 + f2 + f3;
        let (_, strength) = detector.process(signal);
        max_strength = max_strength.max(strength);
    }
    println!("  → Max sibilance_strength: {:.4}", max_strength);

    println!("\n=== End Diagnostics ===\n");
}

#[test]
fn diagnose_deesser_threshold_mapping() {
    println!("\n=== De-Esser Threshold Mapping ===\n");

    // Show what effective_threshold values we get for different user-facing thresholds
    for user_threshold in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let effective_threshold = 1.0 - user_threshold;
        println!(
            "User threshold: {:.2} → effective_threshold: {:.2}",
            user_threshold, effective_threshold
        );

        // Show what sibilance_strength values would trigger compression
        println!(
            "  Triggers when sibilance_strength >= {:.2}",
            effective_threshold
        );

        // Show dynamic threshold calculation (LATEST FORMULA)
        for sib_strength in [0.2, 0.5, 0.8, 1.0] {
            let dynamic_threshold_db = -12.0 + (sib_strength * 6.0);
            println!(
                "    If sib_strength={:.2} → compressor threshold={:.1} dB",
                sib_strength, dynamic_threshold_db
            );
        }
        println!();
    }

    println!("=== End Threshold Mapping ===\n");
}
