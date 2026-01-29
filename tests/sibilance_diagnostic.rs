//! Diagnostics for sibilance detection behavior.
//!
//! These tests are intentionally `#[ignore]` because they are meant for manual
//! inspection and print-based debugging.

use dsynth::dsp::analysis::SibilanceDetector;
use std::f32::consts::PI;

#[test]
#[ignore]
fn diagnose_sibilance_strength_values() {
    let sample_rate = 44100.0;
    let mut detector = SibilanceDetector::new(sample_rate);
    detector.set_threshold(0.0);

    // Pure 6kHz tone.
    detector.reset();
    for _ in 0..500 {
        let _ = detector.process(0.0);
    }
    let mut max_strength = 0.0f32;
    for i in 0..2000 {
        let t = i as f32 / sample_rate;
        let signal = (2.0 * PI * 6000.0 * t).sin() * 0.5;
        let (_, strength) = detector.process(signal);
        max_strength = max_strength.max(strength);
    }

    // No assert: diagnostic only.
    let _ = max_strength;
}
