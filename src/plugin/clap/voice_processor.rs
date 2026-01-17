/// CLAP Voice Enhancer Plugin - Audio Processor
///
/// This module implements the audio processing thread for the voice enhancement plugin.
/// Key differences from kick/main synth:
/// - Takes stereo audio INPUT (not MIDI events)
/// - Processes through voice enhancement chain
/// - Reports processing latency to host
use clap_sys::events::{
    clap_event_header, clap_event_param_value, clap_input_events, clap_output_events,
    CLAP_EVENT_PARAM_VALUE,
};
use clap_sys::process::{
    clap_process, clap_process_status, CLAP_PROCESS_CONTINUE, CLAP_PROCESS_SLEEP,
};
use std::sync::{Arc, RwLock};

use crate::audio::voice_engine::VoiceEngine;
use crate::params_voice::VoiceParams;
use crate::plugin::voice_param_registry;

/// Audio processor for voice enhancement plugin
pub struct VoiceProcessor {
    /// Voice enhancement engine
    engine: VoiceEngine,

    /// Shared parameter state (read by GUI, written by CLAP automation)
    params: Arc<RwLock<VoiceParams>>,

    /// Current sample rate
    pub sample_rate: f32,
}

impl VoiceProcessor {
    /// Create a new voice processor
    pub fn new(sample_rate: f32, params: Arc<RwLock<VoiceParams>>) -> Self {
        let mut engine = VoiceEngine::new(sample_rate);

        // Initialize engine with current parameters
        if let Ok(params_guard) = params.read() {
            engine.update_params(params_guard.clone());
        }

        Self {
            engine,
            params,
            sample_rate,
        }
    }

    /// Get processing latency in samples
    pub fn get_latency(&self) -> u32 {
        self.engine.get_latency()
    }

    /// Process audio block
    ///
    /// # Safety
    /// This function processes CLAP audio buffers. The caller must ensure:
    /// - Input/output pointers are valid
    /// - Buffer sizes match frame_count
    /// - Process is called from the audio thread only
    pub unsafe fn process(&mut self, process: *const clap_process) -> clap_process_status {
        let process = &*process;

        // Handle events (parameter changes from automation)
        if !process.in_events.is_null() {
            self.process_events(&*process.in_events);
        }

        // Check for audio input
        if process.audio_inputs_count == 0 || process.audio_inputs.is_null() {
            // No input - reset engine state to prevent sub oscillator drone
            self.engine.reset();
            return CLAP_PROCESS_SLEEP;
        }

        // Check for audio output
        if process.audio_outputs_count == 0 || process.audio_outputs.is_null() {
            return CLAP_PROCESS_SLEEP;
        }

        let audio_input = &*process.audio_inputs;
        let audio_output = &*process.audio_outputs;

        // We expect stereo input and stereo output
        if audio_input.channel_count < 2 || audio_output.channel_count < 2 {
            return CLAP_PROCESS_SLEEP;
        }

        let frame_count = process.frames_count as usize;

        // Get input buffers (stereo)
        let input_left = std::slice::from_raw_parts(*audio_input.data32.offset(0), frame_count);
        let input_right = std::slice::from_raw_parts(*audio_input.data32.offset(1), frame_count);

        // Get output buffers (stereo)
        let output_left =
            std::slice::from_raw_parts_mut(*audio_output.data32.offset(0) as *mut f32, frame_count);
        let output_right =
            std::slice::from_raw_parts_mut(*audio_output.data32.offset(1) as *mut f32, frame_count);

        // Process audio through voice enhancement chain
        self.engine.process_buffer(
            input_left,
            input_right,
            output_left,
            output_right,
            frame_count,
        );

        CLAP_PROCESS_CONTINUE
    }

    /// Process CLAP events (parameter automation)
    unsafe fn process_events(&mut self, events: &clap_input_events) {
        let event_count = (events.size.unwrap())(events);

        for i in 0..event_count {
            let event_header = (events.get.unwrap())(events, i);
            if event_header.is_null() {
                continue;
            }

            let header = &*event_header;

            // Handle parameter value changes
            if header.type_ == CLAP_EVENT_PARAM_VALUE {
                let param_event = event_header as *const clap_event_param_value;
                let param_event = &*param_event;

                self.apply_param_change(param_event.param_id, param_event.value);
            }
        }
    }

    /// Apply parameter change from host automation
    fn apply_param_change(&mut self, param_id: u32, normalized_value: f64) {
        // Get parameter descriptor for denormalization
        if let Some(descriptor) = voice_param_registry::get_param_descriptor(param_id) {
            let denorm_value = match &descriptor.param_type {
                crate::plugin::param_descriptor::ParamType::Float { min, max, .. } => {
                    min + (normalized_value as f32) * (max - min)
                }
                crate::plugin::param_descriptor::ParamType::Bool => normalized_value as f32,
                crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                    ((normalized_value as f32) * (variants.len() - 1) as f32).round()
                }
                crate::plugin::param_descriptor::ParamType::Int { min, max } => {
                    (*min as f32) + (normalized_value as f32) * ((max - min) as f32)
                }
            };

            // Update shared parameter state
            if let Ok(mut params_guard) = self.params.write() {
                voice_param_registry::apply_param(&mut params_guard, param_id, denorm_value);

                // Update engine immediately
                self.engine.update_params(params_guard.clone());
            }
        }
    }

    /// Reset processor state (called when transport stops or starts)
    pub fn reset(&mut self) {
        self.engine.reset();
    }

    /// Activate processing (called when plugin is turned on)
    pub fn activate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.engine = VoiceEngine::new(sample_rate);

        // Reload current parameters
        if let Ok(params_guard) = self.params.read() {
            self.engine.update_params(params_guard.clone());
        }
    }

    /// Deactivate processing (called when plugin is turned off)
    pub fn deactivate(&mut self) {
        self.engine.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let params = Arc::new(RwLock::new(VoiceParams::default()));
        let processor = VoiceProcessor::new(44100.0, params);
        assert_eq!(processor.sample_rate, 44100.0);
    }

    #[test]
    fn test_get_latency() {
        let params = Arc::new(RwLock::new(VoiceParams::default()));
        let processor = VoiceProcessor::new(44100.0, params);
        let latency = processor.get_latency();

        // Should be pitch buffer (1024) + limiter lookahead (~220)
        assert!(latency > 1000);
        assert!(latency < 1300);
    }

    #[test]
    fn test_activate_deactivate() {
        let params = Arc::new(RwLock::new(VoiceParams::default()));
        let mut processor = VoiceProcessor::new(44100.0, params);

        processor.activate(48000.0);
        assert_eq!(processor.sample_rate, 48000.0);

        processor.deactivate();
        // Should not crash
    }

    #[test]
    fn test_reset() {
        let params = Arc::new(RwLock::new(VoiceParams::default()));
        let mut processor = VoiceProcessor::new(44100.0, params);

        processor.reset();
        // Should not crash and reset engine state
    }

    #[test]
    fn test_apply_param_change() {
        let params = Arc::new(RwLock::new(VoiceParams::default()));
        let mut processor = VoiceProcessor::new(44100.0, params.clone());

        // Apply a parameter change (input gain: 0dB -> 6dB)
        let param_id = voice_param_registry::PARAM_VOICE_INPUT_GAIN;
        let normalized = 0.75; // Normalized value for ~6dB

        processor.apply_param_change(param_id, normalized);

        // Verify parameter was updated
        let params_guard = params.read().unwrap();
        assert!(params_guard.input_gain > 5.0 && params_guard.input_gain < 7.0);
    }

    #[test]
    fn test_param_sync_to_engine() {
        let params = Arc::new(RwLock::new(VoiceParams::default()));
        let mut processor = VoiceProcessor::new(44100.0, params.clone());

        // Modify parameters directly
        {
            let mut params_guard = params.write().unwrap();
            params_guard.gate_threshold = -40.0;
            params_guard.sub_level = 0.8;
        }

        // Reactivate to sync params
        processor.activate(44100.0);

        // Engine should have received updated params (can't verify directly, but shouldn't crash)
        processor.reset();
    }
}
