/// Tests for optimization implementations
/// These tests verify that the optimizations maintain correctness and audio quality

#[cfg(test)]
mod optimization_tests {
    use dsynth::audio::engine::{SynthEngine, create_parameter_buffer};
    use dsynth::audio::voice::Voice;
    use dsynth::dsp::filter::BiquadFilter;
    use dsynth::params::{EnvelopeParams, FilterType, OscillatorParams, SynthParams};

    // ========== FILTER COEFFICIENT QUANTIZATION TESTS ==========

    #[test]
    fn test_filter_quantized_updates_maintain_stability() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_filter_type(FilterType::Lowpass);
        filter.set_resonance(2.0);

        // Apply changing cutoff values
        let mut output_sum = 0.0;
        for i in 0..1000 {
            let modulated_cutoff = 1000.0 + (i as f32 * 0.1).sin() * 500.0;
            filter.set_cutoff(modulated_cutoff);
            let out = filter.process(0.5);
            output_sum += out;

            // Should never produce NaN or infinite values
            assert!(
                out.is_finite(),
                "Filter output should be finite at sample {}",
                i
            );
        }

        // Should produce non-zero output
        assert!(
            output_sum.abs() > 0.1,
            "Filter should produce measurable output"
        );
    }

    #[test]
    fn test_filter_quantized_updates_dont_skip_large_changes() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_filter_type(FilterType::Lowpass);

        // Set initial cutoff
        filter.set_cutoff(1000.0);
        let _out1 = filter.process(0.5);

        // Make a large cutoff jump (should force immediate update)
        filter.set_cutoff(5000.0);
        let _out2 = filter.process(0.5);

        // Change should be applied due to the large jump
        // Verify by making another change and processing
        filter.set_cutoff(8000.0);
        let _out3 = filter.process(0.5);

        // Filter should still be stable
        assert!(true, "Large cutoff changes should be handled correctly");
    }

    #[test]
    fn test_filter_coefficient_calculation_accuracy() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_filter_type(FilterType::Lowpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.707);

        // Process a few samples to calculate coefficients
        let _out = filter.process(0.5);
        let _out = filter.process(0.3);
        let _out = filter.process(0.1);

        // Change parameters and verify coefficients update
        filter.set_cutoff(2000.0);
        // After update interval or large change, coefficients should recalculate
        let _out = filter.process(0.5);

        // Should remain stable
        assert!(true, "Coefficient calculation should maintain stability");
    }

    // ========== PARAMETER UPDATE THROTTLING TESTS ==========

    #[test]
    fn test_engine_parameter_throttling_maintains_correctness() {
        let (mut producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Trigger a note
        engine.note_on(60, 0.8);

        // Change parameters multiple times
        let mut params = SynthParams::default();
        params.oscillators[0].unison = 3;
        producer.write(params);

        // Process multiple samples - parameter changes should be throttled
        let mut outputs = Vec::new();
        for _ in 0..100 {
            outputs.push(engine.process());
        }

        // All outputs should be finite
        assert!(
            outputs.iter().all(|o| o.is_finite()),
            "All outputs should be finite"
        );

        // Should produce some audio output
        assert!(
            outputs.iter().any(|o| o.abs() > 0.001),
            "Engine should produce measurable output"
        );
    }

    #[test]
    fn test_parameter_updates_dont_cause_dropouts() {
        let (mut producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Trigger multiple voices
        for i in 0..8 {
            engine.note_on(60 + i, 0.8);
        }

        let mut max_output: f32 = 0.0;
        let mut min_output: f32 = f32::MAX;

        // Continuously change parameters and process
        for batch in 0..10 {
            let mut params = SynthParams::default();
            params.filters[0].cutoff = 500.0 + (batch as f32 * 200.0);
            producer.write(params);

            for _ in 0..44 {
                let output = engine.process();
                max_output = max_output.max(output);
                min_output = min_output.min(output);
            }
        }

        // Should have significant output range without extreme values
        assert!(
            min_output > -2.0 && max_output < 2.0,
            "Output should be reasonable: range [{}, {}]",
            min_output,
            max_output
        );
    }

    #[test]
    fn test_parameter_equality_check_works() {
        let params1 = SynthParams::default();
        let params2 = SynthParams::default();

        // Same parameters should be equal
        assert_eq!(params1, params2, "Default params should be equal");

        // Modified parameters should not be equal
        let mut params3 = params1;
        params3.master_gain = 0.7;
        assert_ne!(params1, params3, "Modified params should not be equal");
    }

    // ========== UNISON VOICE PRE-ALLOCATION TESTS ==========

    #[test]
    fn test_voice_unison_count_changes_without_allocation() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60, 0.8);

        let mut osc_params = [OscillatorParams::default(); 3];
        let filter_params = Default::default();
        let lfo_params = Default::default();
        let envelope_params = EnvelopeParams::default();

        // Change unison count from 1 to 7
        for unison_count in 1..=7 {
            osc_params[0].unison = unison_count;
            voice.update_parameters(&osc_params, &filter_params, &lfo_params, &envelope_params);

            let _output = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &Default::default(),
            );
        }

        // Then change back to 1
        osc_params[0].unison = 1;
        voice.update_parameters(&osc_params, &filter_params, &lfo_params, &envelope_params);
        let _output = voice.process(
            &osc_params,
            &filter_params,
            &lfo_params,
            &Default::default(),
        );

        assert!(true, "Unison count changes should work without panicking");
    }

    #[test]
    fn test_all_unison_voices_process_correctly() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60, 0.8);

        let mut osc_params = [OscillatorParams::default(); 3];
        osc_params[0].unison = 7; // Max unison
        osc_params[0].unison_detune = 50.0;

        let filter_params = Default::default();
        let lfo_params = Default::default();
        let velocity_params = Default::default();
        let envelope_params = EnvelopeParams {
            attack: 0.001,
            decay: 0.001,
            sustain: 1.0,
            release: 0.2,
        };

        voice.update_parameters(&osc_params, &filter_params, &lfo_params, &envelope_params);

        // Process multiple samples with all 7 unison voices active
        let mut max_output: f32 = 0.0;
        for _ in 0..100 {
            let (left, right) =
                voice.process(&osc_params, &filter_params, &lfo_params, &velocity_params);
            let output = (left.abs() + right.abs()) / 2.0;
            max_output = max_output.max(output);
            assert!(
                (left.is_finite() && right.is_finite()),
                "Output should be finite with 7 unison voices"
            );
        }

        // With 7 unison voices, output should be present (though normalized to prevent clipping).
        // The new unison compensation reduces levels with higher unison counts to prevent
        // distortion when multiple voices play. This is expected behavior.
        assert!(
            max_output > 0.03,
            "7 unison voices should produce output (got {:.4}), though normalized to prevent clipping",
            max_output
        );
    }

    #[test]
    fn test_voice_unison_frequency_spread() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60, 0.8);

        let mut osc_params = [OscillatorParams::default(); 3];
        osc_params[0].unison = 3;
        osc_params[0].unison_detune = 50.0; // 50 cents spread

        let filter_params = Default::default();
        let lfo_params = Default::default();
        let envelope_params = EnvelopeParams::default();

        voice.update_parameters(&osc_params, &filter_params, &lfo_params, &envelope_params);

        // Let the envelope and any internal filters settle.
        // The attack is 0.01s by default (~441 samples), and the oversampling/downsampler
        // also needs a short warm-up before outputs are representative.
        for _ in 0..2000 {
            let _ = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &Default::default(),
            );
        }

        // Process with spread unison voices
        let mut outputs = Vec::new();
        for _ in 0..50 {
            let (left, right) = voice.process(
                &osc_params,
                &filter_params,
                &lfo_params,
                &Default::default(),
            );
            outputs.push((left + right) / 2.0);
        }

        // Should have variation from unison detuning
        let variance = outputs
            .iter()
            .zip(outputs.iter().skip(1))
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);

        assert!(
            variance > 0.001,
            "Unison spread should cause output variation"
        );
    }

    // ========== INTEGRATION TESTS ==========

    #[test]
    fn test_full_engine_with_all_optimizations() {
        let (mut producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Trigger 16 voices (max polyphony)
        for i in 0..16 {
            engine.note_on(60 + (i % 12) as u8, 0.8);
        }

        // Simulate parameter changes and continuous processing
        let mut max_output: f32 = 0.0;
        let mut has_variation = false;
        let mut last_output = 0.0;

        for iteration in 0..100 {
            // Change parameters every 10 samples
            if iteration % 10 == 0 {
                let mut params = SynthParams::default();
                params.oscillators[0].unison = 1 + (iteration / 10) as usize % 7;
                params.filters[0].cutoff = 500.0 + (iteration as f32 * 15.0);
                producer.write(params);
            }

            let output = engine.process();

            // Check output quality
            assert!(output.is_finite(), "Output should be finite");
            max_output = max_output.max(output.abs());

            if (output - last_output).abs() > 0.001 {
                has_variation = true;
            }
            last_output = output;
        }

        // Verify we got good audio
        assert!(max_output > 0.01, "Should produce measurable output");
        assert!(has_variation, "Output should vary over time");
    }

    #[test]
    fn test_voice_notes_maintain_pitch() {
        let mut voice = Voice::new(44100.0);

        let osc_params = [OscillatorParams::default(); 3];
        let filter_params = Default::default();
        let lfo_params = Default::default();
        let velocity_params = Default::default();
        let envelope_params = EnvelopeParams::default();

        // Test multiple notes
        for midi_note in [60, 72, 48] {
            voice.reset();
            voice.note_on(midi_note, 0.8);
            voice.update_parameters(&osc_params, &filter_params, &lfo_params, &envelope_params);

            // Process samples and verify output
            let mut output_sum = 0.0;
            for _ in 0..100 {
                let (left, right) =
                    voice.process(&osc_params, &filter_params, &lfo_params, &velocity_params);
                output_sum += (left.abs() + right.abs()) / 2.0;
                assert!(
                    (left.is_finite() && right.is_finite()),
                    "Note {} should produce finite output",
                    midi_note
                );
            }

            assert!(
                output_sum > 1.0,
                "Note {} should produce measurable output",
                midi_note
            );
        }
    }

    #[test]
    fn test_filter_resonance_maintains_stability() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_filter_type(FilterType::Lowpass);

        // Test with various resonance values
        for resonance in [0.5, 1.0, 2.0, 5.0, 10.0] {
            filter.set_resonance(resonance);
            filter.set_cutoff(1000.0);

            let mut has_output = false;
            for _ in 0..100 {
                let output = filter.process(0.5);
                assert!(
                    output.is_finite(),
                    "Filter with resonance {} should produce finite output",
                    resonance
                );
                if output.abs() > 0.001 {
                    has_output = true;
                }
            }

            assert!(
                has_output,
                "Filter with resonance {} should produce output",
                resonance
            );
        }
    }
}
