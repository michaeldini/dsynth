//! Diagnostic/real-world sanity checks for the voice de-esser.
//!
//! These tests are intentionally `#[ignore]` because they are long-running and
//! intended for manual profiling/listening diagnostics.

use dsynth::dsp::effects::dynamics::DeEsser;
use dsynth::dsp::signal_analyzer::SignalAnalyzer;
use std::f32::consts::PI;

#[test]
#[ignore]
fn diagnose_realistic_vocal_sibilance_reduction() {
    let sample_rate = 44100.0;
    let mut deesser = DeEsser::new(sample_rate);
    let mut analyzer = SignalAnalyzer::new_no_pitch(sample_rate);

    // Simulate vocal: 200Hz fundamental + harmonics + periodic sibilance bursts.
    let fundamental_freq = 200.0;
    let sibilance_freq = 7000.0;

    let mut total_input_energy = 0.0f32;
    let mut total_output_energy = 0.0f32;

    for i in 0..44100 {
        let t = i as f32 / sample_rate;

        let vowel = (2.0 * PI * fundamental_freq * t).sin() * 0.3
            + (2.0 * PI * fundamental_freq * 2.0 * t).sin() * 0.15
            + (2.0 * PI * fundamental_freq * 3.0 * t).sin() * 0.08;

        // 100ms burst every 0.5s.
        let gate = if (t % 0.5) < 0.1 {
            let burst_progress = (t % 0.5) / 0.1;
            (burst_progress * PI).sin() * 0.4
        } else {
            0.0
        };
        let sibilance = (2.0 * PI * sibilance_freq * t).sin() * gate;

        let input = vowel + sibilance;
        let analysis = analyzer.analyze(input, input);

        // Example settings.
        let ((output, _), _) = deesser.process(input, input, 0.6, 1.0, &analysis);

        total_input_energy += input * input;
        total_output_energy += output * output;
    }

    // No assert: this is a diagnostic. We just ensure it runs.
    let _input_rms = (total_input_energy / 44100.0).sqrt();
    let _output_rms = (total_output_energy / 44100.0).sqrt();
}
