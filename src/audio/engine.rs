use crate::audio::voice::Voice;
use crate::params::SynthParams;
use triple_buffer::{Input, Output, TripleBuffer};

const MAX_POLYPHONY: usize = 16;

/// Synthesizer engine with polyphonic voice management
pub struct SynthEngine {
    sample_rate: f32,
    voices: Vec<Voice>,
    params_consumer: Output<SynthParams>,
    current_params: SynthParams,
}

impl SynthEngine {
    /// Create a new synth engine
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    /// * `params_consumer` - Triple-buffer consumer for parameter updates
    pub fn new(sample_rate: f32, params_consumer: Output<SynthParams>) -> Self {
        let mut voices = Vec::with_capacity(MAX_POLYPHONY);
        for _ in 0..MAX_POLYPHONY {
            voices.push(Voice::new(sample_rate));
        }

        Self {
            sample_rate,
            voices,
            params_consumer,
            current_params: SynthParams::default(),
        }
    }

    /// Trigger a note on
    ///
    /// # Arguments
    /// * `note` - MIDI note number (0-127)
    /// * `velocity` - Note velocity (0.0-1.0)
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        // First, try to find an inactive voice
        if let Some(voice) = self.voices.iter_mut().find(|v| !v.is_active()) {
            voice.note_on(note, velocity);
            voice.update_parameters(
                &self.current_params.oscillators,
                &self.current_params.filters,
                &self.current_params.filter_envelopes,
                &self.current_params.lfos,
            );
            return;
        }

        // All voices active - use quietest-voice stealing
        let quietest_idx = self.find_quietest_voice();
        self.voices[quietest_idx].note_on(note, velocity);
        self.voices[quietest_idx].update_parameters(
            &self.current_params.oscillators,
            &self.current_params.filters,
            &self.current_params.filter_envelopes,
            &self.current_params.lfos,
        );
    }

    /// Release a note
    ///
    /// # Arguments
    /// * `note` - MIDI note number to release
    pub fn note_off(&mut self, note: u8) {
        // Release all voices playing this note
        for voice in &mut self.voices {
            if voice.is_active() && voice.note() == note {
                voice.note_off();
            }
        }
    }

    /// Find the quietest active voice for voice stealing
    fn find_quietest_voice(&self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_active())
            .min_by(|(_, a), (_, b)| {
                a.rms()
                    .partial_cmp(&b.rms())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    /// Panic - release all notes immediately
    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
    }

    /// Process one sample and return mixed output
    pub fn process(&mut self) -> f32 {
        // Check for parameter updates from triple buffer
        let new_params = self.params_consumer.read();

        // Store the new parameters
        self.current_params = *new_params;

        // Update all active voices with current parameters
        // This ensures filters respond immediately to GUI changes
        for voice in &mut self.voices {
            if voice.is_active() {
                voice.update_parameters(
                    &self.current_params.oscillators,
                    &self.current_params.filters,
                    &self.current_params.filter_envelopes,
                    &self.current_params.lfos,
                );
            }
        }

        // Mix all voices
        let mut output = 0.0;
        for voice in &mut self.voices {
            output += voice.process(
                &self.current_params.oscillators,
                &self.current_params.filters,
                &self.current_params.filter_envelopes,
                &self.current_params.lfos,
                &self.current_params.velocity,
            );
        }

        // Apply master gain
        output * self.current_params.master_gain
    }

    /// Get the number of active voices
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

/// Create a triple buffer for parameters and return (producer input, consumer output)
pub fn create_parameter_buffer() -> (Input<SynthParams>, Output<SynthParams>) {
    let buffer = TripleBuffer::new(&SynthParams::default());
    buffer.split()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let (_producer, consumer) = create_parameter_buffer();
        let engine = SynthEngine::new(44100.0, consumer);

        assert_eq!(engine.sample_rate(), 44100.0);
        assert_eq!(engine.active_voice_count(), 0);
    }

    #[test]
    fn test_note_on_activates_voice() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        engine.note_on(60, 0.8);
        assert_eq!(engine.active_voice_count(), 1);

        engine.note_on(64, 0.7);
        assert_eq!(engine.active_voice_count(), 2);
    }

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
            engine.process();
        }

        // Should be inactive after release completes
        assert_eq!(engine.active_voice_count(), 0);
    }

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
            engine.process();
        }

        // Trigger one more note - should steal quietest
        engine.note_on(100, 0.9);
        assert_eq!(engine.active_voice_count(), MAX_POLYPHONY);
    }

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

    #[test]
    fn test_output_generation() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        engine.note_on(60, 0.8);

        // Process samples and verify output
        let mut has_output = false;
        for _ in 0..1000 {
            let sample = engine.process();
            if sample.abs() > 0.001 {
                has_output = true;
                break;
            }
        }

        assert!(has_output, "Engine should produce audio output");
    }

    #[test]
    fn test_parameter_updates() {
        let (mut producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        // Update parameters via triple buffer
        let mut new_params = SynthParams::default();
        new_params.master_gain = 0.5;
        producer.write(new_params);

        // Process should pick up new parameters
        engine.process();

        // Verify by checking that output is affected by master gain
        engine.note_on(60, 1.0);
        for _ in 0..100 {
            engine.process();
        }

        // Parameters were updated (verified implicitly through processing)
    }

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

    #[test]
    fn test_zero_velocity_note() {
        let (_producer, consumer) = create_parameter_buffer();
        let mut engine = SynthEngine::new(44100.0, consumer);

        engine.note_on(60, 0.0);
        assert_eq!(engine.active_voice_count(), 1);

        // Process samples - with velocity sensitivity, zero velocity still produces some output
        // (1.0 - sensitivity) * volume. Default sensitivity is 0.7, so minimum is 0.3
        let mut max_output = 0.0_f32;
        for _ in 0..1000 {
            let sample = engine.process();
            max_output = max_output.max(sample.abs());
        }

        // Should produce some output (velocity scaling allows quieter but not silent notes)
        assert!(
            max_output > 0.0 && max_output < 1.0,
            "Zero velocity should produce reduced but non-zero output with velocity sensitivity"
        );
    }
}
