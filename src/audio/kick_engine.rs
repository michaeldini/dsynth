/// Kick Drum Synthesis Engine
/// Monophonic engine optimized for kick drum synthesis
use crate::audio::kick_voice::KickVoice;
use crate::params_kick::KickParams;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct KickEngine {
    voice: KickVoice,
    params: Arc<Mutex<KickParams>>,
    sample_rate: f32,

    // MIDI note queue (parking_lot::Mutex for consistency)
    note_queue: Arc<Mutex<Vec<MidiEvent>>>,
}

#[derive(Clone, Copy, Debug)]
pub enum MidiEvent {
    NoteOn { note: u8, velocity: f32 },
    NoteOff { note: u8 },
}

impl KickEngine {
    pub fn new(sample_rate: f32, params: Arc<Mutex<KickParams>>) -> Self {
        Self {
            voice: KickVoice::new(sample_rate),
            params,
            sample_rate,
            note_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a clone of the note queue for MIDI input thread
    pub fn get_note_queue(&self) -> Arc<Mutex<Vec<MidiEvent>>> {
        Arc::clone(&self.note_queue)
    }

    /// Process MIDI events from the queue
    fn process_midi_events(&mut self, params: &KickParams) {
        let mut queue = self.note_queue.lock();
        for event in queue.drain(..) {
            match event {
                MidiEvent::NoteOn { velocity, .. } => {
                    // Trigger kick on any note
                    self.voice.trigger(velocity, params);
                }
                MidiEvent::NoteOff { .. } => {
                    // Kicks typically ignore note-off, but we'll implement it anyway
                    self.voice.release();
                }
            }
        }
    }

    /// Process a single sample
    pub fn process_sample(&mut self) -> f32 {
        // Get current parameters (lock briefly)
        let params = self.params.lock().clone();

        // Process MIDI events
        self.process_midi_events(&params);

        // Generate audio from voice
        let sample = self.voice.process(&params);

        // Apply master level
        sample * params.master_level
    }

    /// Process a block of samples (more efficient for buffer-based APIs)
    pub fn process_block(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.process_sample();
        }
    }

    /// Process stereo block (duplicate mono to both channels)
    pub fn process_block_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(
            left.len(),
            right.len(),
            "Left and right buffers must be same length"
        );

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let sample = self.process_sample();
            *l = sample;
            *r = sample;
        }
    }

    /// Trigger kick directly (for testing or non-MIDI use)
    pub fn trigger(&mut self, velocity: f32) {
        let params = self.params.lock();
        self.voice.trigger(velocity, &params);
    }

    /// Get current sample rate
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Check if voice is currently active
    pub fn is_active(&self) -> bool {
        self.voice.is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let params = Arc::new(Mutex::new(KickParams::default()));
        let engine = KickEngine::new(44100.0, params);

        assert_eq!(engine.sample_rate(), 44100.0);
        assert!(!engine.is_active());
    }

    #[test]
    fn test_trigger_via_midi() {
        let params = Arc::new(Mutex::new(KickParams::default()));
        let mut engine = KickEngine::new(44100.0, Arc::clone(&params));

        // Add MIDI note-on event
        let queue = engine.get_note_queue();
        {
            let mut q = queue.lock();
            q.push(MidiEvent::NoteOn {
                note: 36,
                velocity: 1.0,
            }); // MIDI note 36 = C1 (kick)
        }

        // Process a sample (should trigger the kick)
        let sample = engine.process_sample();

        // Should be active and producing sound
        assert!(engine.is_active());
        assert!(sample.abs() > 0.0);
    }

    #[test]
    fn test_direct_trigger() {
        let params = Arc::new(Mutex::new(KickParams::default()));
        let mut engine = KickEngine::new(44100.0, params);

        // Trigger directly
        engine.trigger(1.0);

        // Should be active
        assert!(engine.is_active());

        // Process and check audio
        let sample = engine.process_sample();
        assert!(sample.abs() > 0.0);
    }

    #[test]
    fn test_process_block() {
        let params = Arc::new(Mutex::new(KickParams::default()));
        let mut engine = KickEngine::new(44100.0, params);

        engine.trigger(1.0);

        let mut buffer = vec![0.0; 64];
        engine.process_block(&mut buffer);

        // Should have generated audio
        let has_audio = buffer.iter().any(|&s| s.abs() > 0.0);
        assert!(has_audio);
    }

    #[test]
    fn test_process_block_stereo() {
        let params = Arc::new(Mutex::new(KickParams::default()));
        let mut engine = KickEngine::new(44100.0, params);

        engine.trigger(1.0);

        let mut left = vec![0.0; 64];
        let mut right = vec![0.0; 64];
        engine.process_block_stereo(&mut left, &mut right);

        // Both channels should have audio
        let left_has_audio = left.iter().any(|&s| s.abs() > 0.0);
        let right_has_audio = right.iter().any(|&s| s.abs() > 0.0);
        assert!(left_has_audio);
        assert!(right_has_audio);

        // Channels should be identical (mono kick)
        for (l, r) in left.iter().zip(right.iter()) {
            assert_eq!(l, r);
        }
    }

    #[test]
    fn test_multiple_triggers() {
        let params = Arc::new(Mutex::new(KickParams::default()));
        let mut engine = KickEngine::new(44100.0, params);

        // Trigger multiple times (monophonic, so should retrigger)
        engine.trigger(1.0);
        engine.process_sample();

        engine.trigger(0.8);
        engine.process_sample();

        engine.trigger(0.6);
        let sample = engine.process_sample();

        // Should still be producing sound
        assert!(engine.is_active());
        assert!(sample.abs() > 0.0);
    }

    #[test]
    fn test_master_level() {
        let params = Arc::new(Mutex::new(KickParams::default()));
        let mut engine1 = KickEngine::new(44100.0, Arc::clone(&params));
        let mut engine2 = KickEngine::new(44100.0, Arc::clone(&params));

        // Set different master levels
        {
            let mut p = params.lock();
            p.master_level = 1.0;
        }
        engine1.trigger(1.0);
        let sample1 = engine1.process_sample();

        {
            let mut p = params.lock();
            p.master_level = 0.5;
        }
        engine2.trigger(1.0);
        let sample2 = engine2.process_sample();

        // Lower master level should produce quieter output
        assert!(sample1.abs() > sample2.abs());
    }

    #[test]
    fn test_preset_808() {
        let params = Arc::new(Mutex::new(KickParams::preset_808()));
        let mut engine = KickEngine::new(44100.0, params);

        engine.trigger(1.0);
        let sample = engine.process_sample();

        assert!(engine.is_active());
        assert!(sample.abs() > 0.0);
    }
}
