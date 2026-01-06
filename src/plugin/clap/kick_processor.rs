/// CLAP Audio Processor for Kick Drum Synth
use super::super::kick_param_registry::{get_kick_registry, ParamId};
use crate::audio::kick_engine::{KickEngine, MidiEvent};
use crate::params_kick::KickParams;
use clap_sys::events::{clap_event_header, clap_event_note, clap_event_param_value};
use clap_sys::process::{clap_process, clap_process_status};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::OnceLock;

fn kick_log_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("DSYNTH_KICK_LOG").is_some())
}

fn log_kick_proc(msg: &str) {
    if !kick_log_enabled() {
        return;
    }

    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/dsynth_kick_clap.log")
    {
        let _ = writeln!(
            file,
            "[{}] {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            msg
        );
    }
}

/// Kick drum audio processor
pub struct KickClapProcessor {
    /// The kick synthesis engine
    engine: KickEngine,
    /// Current parameter values
    current_params: Arc<Mutex<KickParams>>,
    /// Current sample rate
    sample_rate: f32,
}

impl KickClapProcessor {
    /// Create a new kick CLAP processor
    pub fn new(sample_rate: f32, params: Arc<Mutex<KickParams>>) -> Self {
        let engine = KickEngine::new(sample_rate, Arc::clone(&params));

        Self {
            engine,
            current_params: params,
            sample_rate,
        }
    }

    /// Get current sample rate
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Set parameter value (CLAP plain value domain)
    pub fn set_param(&mut self, param_id: ParamId, value: f64) {
        if !value.is_finite() {
            if kick_log_enabled() {
                log_kick_proc(&format!(
                    "WARN non-finite param value param_id=0x{:08x} value={}",
                    param_id, value
                ));
            }
            return;
        }

        let registry = get_kick_registry();
        let mut params = self.current_params.lock();

        // CLAP events carry the parameter's plain value (e.g. Hz/ms), not normalized 0..1.
        // Internally, KickParamRegistry::apply_param expects normalized, so normalize here.
        let normalized = if let Some(desc) = registry.get_descriptor(param_id) {
            desc.normalize_value(value as f32) as f64
        } else {
            // Unknown param: best-effort clamp into 0..1
            value.clamp(0.0, 1.0)
        };
        registry.apply_param(&mut params, param_id, normalized);
    }

    /// Get parameter value (CLAP plain value domain)
    pub fn get_param(&self, param_id: ParamId) -> f64 {
        let params = self.current_params.lock();
        match param_id {
            crate::plugin::kick_param_registry::PARAM_KICK_OSC1_PITCH_START => {
                params.osc1_pitch_start as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_OSC1_PITCH_END => {
                params.osc1_pitch_end as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_OSC1_PITCH_DECAY => {
                params.osc1_pitch_decay as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_OSC1_LEVEL => params.osc1_level as f64,

            crate::plugin::kick_param_registry::PARAM_KICK_OSC2_PITCH_START => {
                params.osc2_pitch_start as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_OSC2_PITCH_END => {
                params.osc2_pitch_end as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_OSC2_PITCH_DECAY => {
                params.osc2_pitch_decay as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_OSC2_LEVEL => params.osc2_level as f64,

            crate::plugin::kick_param_registry::PARAM_KICK_AMP_ATTACK => params.amp_attack as f64,
            crate::plugin::kick_param_registry::PARAM_KICK_AMP_DECAY => params.amp_decay as f64,
            crate::plugin::kick_param_registry::PARAM_KICK_AMP_SUSTAIN => params.amp_sustain as f64,
            crate::plugin::kick_param_registry::PARAM_KICK_AMP_RELEASE => params.amp_release as f64,

            crate::plugin::kick_param_registry::PARAM_KICK_FILTER_CUTOFF => {
                params.filter_cutoff as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_FILTER_RESONANCE => {
                params.filter_resonance as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_FILTER_ENV_AMOUNT => {
                params.filter_env_amount as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_FILTER_ENV_DECAY => {
                params.filter_env_decay as f64
            }

            crate::plugin::kick_param_registry::PARAM_KICK_DISTORTION_AMOUNT => {
                params.distortion_amount as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_DISTORTION_TYPE => {
                match params.distortion_type {
                    crate::params_kick::DistortionType::Soft => 0.0,
                    crate::params_kick::DistortionType::Hard => 1.0,
                    crate::params_kick::DistortionType::Tube => 2.0,
                    crate::params_kick::DistortionType::Foldback => 3.0,
                }
            }

            crate::plugin::kick_param_registry::PARAM_KICK_MASTER_LEVEL => {
                params.master_level as f64
            }
            crate::plugin::kick_param_registry::PARAM_KICK_VELOCITY_SENSITIVITY => {
                params.velocity_sensitivity as f64
            }

            _ => 0.0,
        }
    }

    /// Get copy of current parameters
    pub fn get_params(&self) -> KickParams {
        self.current_params.lock().clone()
    }

    /// Set all parameters at once
    pub fn set_params(&mut self, params: KickParams) {
        *self.current_params.lock() = params;
    }

    /// Process audio buffer from CLAP host
    pub unsafe fn process(&mut self, process: *const clap_process) -> clap_process_status {
        if process.is_null() {
            log_kick_proc("ERROR process() got null process pointer");
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        let process = unsafe { &*process };
        let frames = process.frames_count as usize;

        // Process input events (MIDI, parameters)
        unsafe {
            self.process_input_events(process);
        }

        // Get output buffers (support both 32-bit and 64-bit host buffers)
        if process.audio_outputs.is_null() || process.audio_outputs_count == 0 {
            log_kick_proc("ERROR no audio outputs in process()");
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        let output = unsafe { &*process.audio_outputs };
        if output.channel_count != 2 {
            log_kick_proc(&format!(
                "ERROR unexpected channel_count={} in process()",
                output.channel_count
            ));
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        // Prefer 64-bit buffers if provided by the host, otherwise use 32-bit
        if !output.data64.is_null() {
            // Host provided f64 buffers
            let left = unsafe {
                std::slice::from_raw_parts_mut(*(output.data64 as *mut *mut f64).offset(0), frames)
            };
            let right = unsafe {
                std::slice::from_raw_parts_mut(*(output.data64 as *mut *mut f64).offset(1), frames)
            };

            // Temporary f32 buffer for processing
            let mut temp_left = vec![0.0f32; frames];
            let mut temp_right = vec![0.0f32; frames];
            self.engine
                .process_block_stereo(&mut temp_left, &mut temp_right);

            // Copy/convert f32 -> f64
            for i in 0..frames {
                left[i] = temp_left[i] as f64;
                right[i] = temp_right[i] as f64;
            }
        } else if !output.data32.is_null() {
            // Host provided f32 buffers
            let left = unsafe {
                std::slice::from_raw_parts_mut(*(output.data32 as *mut *mut f32).offset(0), frames)
            };
            let right = unsafe {
                std::slice::from_raw_parts_mut(*(output.data32 as *mut *mut f32).offset(1), frames)
            };

            self.engine.process_block_stereo(left, right);
        } else {
            // No valid buffer provided
            log_kick_proc("ERROR no valid output buffer (data32/data64 null)");
            return clap_sys::process::CLAP_PROCESS_ERROR;
        }

        clap_sys::process::CLAP_PROCESS_CONTINUE
    }

    /// Process input events from CLAP host
    unsafe fn process_input_events(&mut self, process: &clap_process) {
        if process.in_events.is_null() {
            return;
        }

        let in_events = unsafe { &*process.in_events };
        let event_count = unsafe { ((*in_events).size.unwrap())(in_events) };

        let note_queue = self.engine.get_note_queue();

        for i in 0..event_count {
            let event_header =
                unsafe { ((*in_events).get.unwrap())(in_events, i) as *const clap_event_header };
            if event_header.is_null() {
                continue;
            }

            let header = unsafe { &*event_header };

            match header.type_ {
                clap_sys::events::CLAP_EVENT_NOTE_ON => {
                    let note_event = event_header as *const clap_event_note;
                    let note = unsafe { &*note_event };

                    let velocity = (note.velocity as f32).clamp(0.0, 1.0);

                    let mut queue = note_queue.lock();
                    queue.push(MidiEvent::NoteOn {
                        note: note.key as u8,
                        velocity,
                    });
                }

                clap_sys::events::CLAP_EVENT_NOTE_OFF => {
                    let note_event = event_header as *const clap_event_note;
                    let note = unsafe { &*note_event };

                    let mut queue = note_queue.lock();
                    queue.push(MidiEvent::NoteOff {
                        note: note.key as u8,
                    });
                }

                clap_sys::events::CLAP_EVENT_PARAM_VALUE => {
                    let param_event = event_header as *const clap_event_param_value;
                    let param = unsafe { &*param_event };

                    if kick_log_enabled() {
                        log_kick_proc(&format!(
                            "in_events PARAM_VALUE param_id=0x{:08x} value={}",
                            param.param_id, param.value
                        ));
                    }

                    // CLAP delivers the parameter's plain value domain here (e.g. Hz/ms).
                    self.set_param(param.param_id, param.value);

                    // For the problematic parameter, log the post-apply denormalized value.
                    if param.param_id
                        == crate::plugin::kick_param_registry::PARAM_KICK_OSC1_PITCH_START
                    {
                        let p = self.current_params.lock();
                        let hz = p.osc1_pitch_start;
                        if kick_log_enabled() {
                            log_kick_proc(&format!("post-apply osc1_pitch_start hz={}", hz));
                        }
                    }
                }

                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let params = Arc::new(Mutex::new(KickParams::default()));
        let processor = KickClapProcessor::new(44100.0, params);
        assert_eq!(processor.sample_rate(), 44100.0);
    }

    #[test]
    fn test_param_get_set() {
        use crate::plugin::kick_param_registry::PARAM_KICK_OSC1_PITCH_START;

        let params = Arc::new(Mutex::new(KickParams::default()));
        let mut processor = KickClapProcessor::new(44100.0, params);

        // Set a parameter using plain value (Hz)
        let value_hz = 200.0;
        processor.set_param(PARAM_KICK_OSC1_PITCH_START, value_hz);

        // Read it back (plain)
        let readback = processor.get_param(PARAM_KICK_OSC1_PITCH_START);
        assert!((readback - value_hz).abs() < 0.1);
    }
}
