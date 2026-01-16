use super::super::{param_descriptor::ParamId, param_update::param_apply};
use crate::audio::engine::SynthEngine;
use crate::params::SynthParams;
use crate::plugin::gui_param_change::GuiParamChange;
use clap_sys::events::{
    clap_event_header, clap_event_note, clap_event_param_value, clap_event_transport,
};
/// CLAP Audio Processor
///
/// Handles audio processing callbacks from the CLAP host, integrating with SynthEngine.
use clap_sys::process::{clap_process, clap_process_status};
use parking_lot::RwLock;
use std::sync::Arc;
use triple_buffer::Input;
use triple_buffer::Output;

/// Audio processor state
pub struct ClapProcessor {
    /// The synthesis engine
    engine: SynthEngine,
    /// Triple-buffer producer for parameter updates
    pub param_producer: Input<SynthParams>,
    /// Current parameter values (synchronized copy for queries)
    pub current_params: SynthParams,
    /// Current sample rate
    sample_rate: f32,

    /// Consumer for GUI-initiated param changes (GUI -> audio thread)
    gui_param_consumer: Option<Output<GuiParamChange>>,

    /// Shared synth params for full sync (e.g., randomization)
    synth_params: Option<Arc<RwLock<SynthParams>>>,

    last_gui_change: GuiParamChange,
}

impl ClapProcessor {
    /// Create a new CLAP processor
    pub fn new(sample_rate: f32, gui_param_consumer: Output<GuiParamChange>) -> Self {
        let (mut producer, consumer) = crate::audio::create_parameter_buffer();
        let engine = SynthEngine::new(sample_rate, consumer);

        // Initialize with default parameters
        let default_params = SynthParams::default();
        producer.write(default_params);

        Self {
            engine,
            param_producer: producer,
            current_params: default_params,
            sample_rate,
            gui_param_consumer: Some(gui_param_consumer),
            synth_params: None,
            last_gui_change: GuiParamChange::default(),
        }
    }

    /// Set the shared synth_params reference (for full sync operations like randomization)
    pub fn set_synth_params(&mut self, synth_params: Arc<RwLock<SynthParams>>) {
        self.synth_params = Some(synth_params);
    }

    #[inline]
    fn maybe_apply_gui_param_change(&mut self) {
        let Some(consumer) = &mut self.gui_param_consumer else {
            return;
        };

        let change = *consumer.read();
        if change == self.last_gui_change {
            return;
        }

        self.last_gui_change = change;
        if change.param_id == 0 {
            return;
        }

        // Special signal: 0xFFFFFFFF means "full sync" (e.g., after randomization)
        // Copy the entire synth_params to current_params
        if change.param_id == 0xFFFFFFFF {
            if let Some(ref synth_params) = self.synth_params {
                let params = synth_params.read();
                self.current_params = *params;
                self.param_producer.write(self.current_params);
            }
            return;
        }

        // Apply the GUI change on the audio thread (single producer).
        param_apply::apply_param(&mut self.current_params, change.param_id, change.normalized);
        self.param_producer.write(self.current_params);
    }

    /// Process audio buffer from CLAP host
    pub unsafe fn process(&mut self, process: *const clap_process) -> clap_process_status {
        if process.is_null() {
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        let process = unsafe { &*process };
        let frames = process.frames_count as usize;

        // Apply any GUI param changes (GUI -> audio thread)
        self.maybe_apply_gui_param_change();

        // Process input events (MIDI, parameters)
        unsafe {
            self.process_input_events(process);
        }

        // Get output buffers
        if process.audio_outputs.is_null() || process.audio_outputs_count == 0 {
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        let output = unsafe { &*process.audio_outputs };
        if output.channel_count != 2 {
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        let left_out = unsafe {
            std::slice::from_raw_parts_mut(*(output.data32 as *mut *mut f32).offset(0), frames)
        };
        let right_out = unsafe {
            std::slice::from_raw_parts_mut(*(output.data32 as *mut *mut f32).offset(1), frames)
        };

        // Process audio through the engine
        for i in 0..frames {
            let (left, right) = self.engine.process();
            left_out[i] = left;
            right_out[i] = right;
        }

        clap_sys::process::CLAP_PROCESS_CONTINUE
    }

    /// Process input events (notes, parameter changes)
    unsafe fn process_input_events(&mut self, process: &clap_process) {
        if process.in_events.is_null() {
            return;
        }

        let in_events = unsafe { &*process.in_events };
        let size_fn = in_events.size.expect("in_events.size is null");
        let get_fn = in_events.get.expect("in_events.get is null");

        let event_count = unsafe { size_fn(in_events) };

        for i in 0..event_count {
            let event = unsafe { get_fn(in_events, i) };
            if event.is_null() {
                continue;
            }

            let header = unsafe { &*(event as *const clap_event_header) };

            match header.type_ {
                clap_sys::events::CLAP_EVENT_NOTE_ON => unsafe {
                    self.process_note_on_event(event as *const clap_event_note);
                },
                clap_sys::events::CLAP_EVENT_NOTE_OFF => unsafe {
                    self.process_note_off_event(event as *const clap_event_note);
                },
                clap_sys::events::CLAP_EVENT_PARAM_VALUE => unsafe {
                    self.process_param_value_event(event as *const clap_event_param_value);
                },
                clap_sys::events::CLAP_EVENT_TRANSPORT => unsafe {
                    self.process_transport_event(event as *const clap_event_transport);
                },
                _ => {}
            }
        }
    }

    /// Process note on event
    unsafe fn process_note_on_event(&mut self, event: *const clap_event_note) {
        let note = unsafe { &*event };
        self.engine.note_on(note.key as u8, note.velocity as f32);
    }

    /// Process note off event
    unsafe fn process_note_off_event(&mut self, event: *const clap_event_note) {
        let note = unsafe { &*event };
        self.engine.note_off(note.key as u8);
    }

    /// Process parameter value change event
    unsafe fn process_param_value_event(&mut self, event: *const clap_event_param_value) {
        let param_event = unsafe { &*event };
        let param_id = param_event.param_id as ParamId;
        let normalized = param_event.value as f32;

        // Update our stored current params
        param_apply::apply_param(&mut self.current_params, param_id, normalized);

        // Write to triple buffer for engine
        self.param_producer.write(self.current_params);
    }

    /// Process transport event (tempo changes)
    unsafe fn process_transport_event(&mut self, event: *const clap_event_transport) {
        let transport = unsafe { &*event };

        // Check if tempo is valid (CLAP_TRANSPORT_HAS_TEMPO flag)
        const CLAP_TRANSPORT_HAS_TEMPO: u32 = 1 << 0;
        if transport.flags & CLAP_TRANSPORT_HAS_TEMPO != 0 {
            let bpm = transport.tempo;
            self.engine.set_tempo(bpm);
        }
    }

    /// Activate the processor (called when plugin is turned on)
    pub fn activate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Engine already created with correct sample rate
    }

    /// Deactivate the processor (called when plugin is turned off)
    pub fn deactivate(&mut self) {
        // Reset engine state if needed
    }

    /// Get a reference to the engine for state access
    pub fn engine(&self) -> &SynthEngine {
        &self.engine
    }

    /// Get a mutable reference to the engine
    pub fn engine_mut(&mut self) -> &mut SynthEngine {
        &mut self.engine
    }

    /// Get the parameter producer for external param updates
    pub fn param_producer(&mut self) -> &mut Input<SynthParams> {
        &mut self.param_producer
    }

    pub fn take_gui_param_consumer(&mut self) -> Option<Output<GuiParamChange>> {
        self.gui_param_consumer.take()
    }
}
