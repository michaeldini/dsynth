//! Simple sine wave synthesizer for testing dsynth-clap framework

use dsynth_clap::*;
use std::f32::consts::PI;
use std::fs::OpenOptions;
use std::io::Write;

fn log_to_file(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/simplesynth.log")
    {
        let _ = writeln!(file, "[SimpleSynth] {}", msg);
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct SimpleSynth;

impl ClapPlugin for SimpleSynth {
    type Processor = SimpleProcessor;
    type Params = SimpleParams;

    fn descriptor() -> PluginDescriptor {
        PluginDescriptor::instrument("Simple Synth", "com.dsynth.simple-synth")
            .version("0.1.0")
            .description("Minimal sine wave synthesizer")
            .vendor("DSynth")
    }

    fn clap_descriptor() -> &'static clap_sys::plugin::clap_plugin_descriptor {
        use std::ffi::CString;
        use clap_sys::plugin::clap_plugin_descriptor;
        
        static mut DESCRIPTOR: Option<clap_plugin_descriptor> = None;
        static mut STRINGS: Option<DescriptorStrings> = None;
        
        struct DescriptorStrings {
            id: CString,
            name: CString,
            vendor: CString,
            version: CString,
            description: CString,
            feature1: CString,
            feature2: CString,
            features: [*const i8; 3],
        }
        
        unsafe {
            if DESCRIPTOR.is_none() {
                let id = CString::new("com.dsynth.simple-synth").unwrap();
                let name = CString::new("Simple Synth").unwrap();
                let vendor = CString::new("DSynth").unwrap();
                let version = CString::new("0.1.0").unwrap();
                let description = CString::new("Minimal sine wave synthesizer").unwrap();
                let feature1 = CString::new("instrument").unwrap();
                let feature2 = CString::new("synthesizer").unwrap();
                
                let features = [
                    feature1.as_ptr(),
                    feature2.as_ptr(),
                    std::ptr::null(),
                ];
                
                let strings = DescriptorStrings {
                    id,
                    name,
                    vendor,
                    version,
                    description,
                    feature1,
                    feature2,
                    features,
                };
                
                DESCRIPTOR = Some(clap_plugin_descriptor {
                    clap_version: clap_sys::version::CLAP_VERSION,
                    id: strings.id.as_ptr(),
                    name: strings.name.as_ptr(),
                    vendor: strings.vendor.as_ptr(),
                    url: std::ptr::null(),
                    manual_url: std::ptr::null(),
                    support_url: std::ptr::null(),
                    version: strings.version.as_ptr(),
                    description: strings.description.as_ptr(),
                    features: strings.features.as_ptr(),
                });
                
                STRINGS = Some(strings);
            }
            
            DESCRIPTOR.as_ref().unwrap()
        }
    }

    fn new() -> Self {
        log_to_file("SimpleSynth::new() called");
        Self
    }

    fn create_processor(&mut self, sample_rate: f32) -> Self::Processor {
        log_to_file(&format!("create_processor() called with sample_rate={}", sample_rate));
        SimpleProcessor::new(sample_rate)
    }
}

// ============================================================================
// PROCESSOR
// ============================================================================

pub struct SimpleProcessor {
    sample_rate: f32,
    phase: f32,
    frequency: f32,
    amplitude: f32,
}

impl SimpleProcessor {
    fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            frequency: 440.0,
            amplitude: 0.0,
        }
    }
}

impl ClapProcessor for SimpleProcessor {
    fn process(&mut self, audio: &mut AudioBuffers, events: &Events) -> ProcessStatus {
        static mut PROCESS_COUNT: u32 = 0;
        unsafe {
            PROCESS_COUNT += 1;
            if PROCESS_COUNT == 1 || PROCESS_COUNT % 1000 == 0 {
                log_to_file(&format!("process() called (count: {})", PROCESS_COUNT));
            }
        }
        
        // Handle MIDI note events
        unsafe {
            for i in 0..events.input_event_count() {
                if let Some(event) = events.input_event(i) {
                    if event.space_id == clap_sys::events::CLAP_CORE_EVENT_SPACE_ID {
                        match event.type_ {
                            clap_sys::events::CLAP_EVENT_NOTE_ON => {
                                let note_event = &*(event as *const _ as *const clap_sys::events::clap_event_note);
                                let note = note_event.key;
                                self.frequency = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
                                self.amplitude = 0.3;
                                log_to_file(&format!("NOTE_ON: note={}, freq={:.2}", note, self.frequency));
                            }
                            clap_sys::events::CLAP_EVENT_NOTE_OFF => {
                                self.amplitude = 0.0;
                                log_to_file("NOTE_OFF");
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Generate audio
        let frames = audio.frames_count();
        
        unsafe {
            if let Some((left, right)) = audio.output_stereo_mut(0) {
                for i in 0..frames as usize {
                    let sample = if self.amplitude > 0.0 {
                        let s = (self.phase * 2.0 * PI).sin() * self.amplitude;
                        self.phase += self.frequency / self.sample_rate;
                        if self.phase >= 1.0 {
                            self.phase -= 1.0;
                        }
                        s
                    } else {
                        0.0
                    };
                    
                    left[i] = sample;
                    right[i] = sample;
                }
            }
        }
        
        ProcessStatus::Continue
    }

    fn activate(&mut self, sample_rate: f32) {
        log_to_file(&format!("activate() called with sample_rate={}", sample_rate));
        self.sample_rate = sample_rate;
        self.phase = 0.0;
        self.amplitude = 0.0;
    }

    fn reset(&mut self) {
        log_to_file("reset() called");
        self.phase = 0.0;
        self.amplitude = 0.0;
    }
}

// ============================================================================
// PARAMETERS
// ============================================================================

pub struct SimpleParams;

impl PluginParams for SimpleParams {
    fn param_count() -> u32 {
        0
    }
    
    fn param_descriptor(_index: u32) -> Option<ParamDescriptor> {
        None
    }
    
    fn param_descriptor_by_id(_id: ParamId) -> Option<ParamDescriptor> {
        None
    }

    fn get_param(_id: ParamId) -> Option<f32> {
        None
    }

    fn set_param(_id: ParamId, _value: f32) {}
    
    fn save_state() -> PluginState {
        PluginState::default()
    }
    
    fn load_state(_state: &PluginState) {}
}

// ============================================================================
// CLAP ENTRY POINT
// ============================================================================

dsynth_clap::generate_clap_entry!(SimpleSynth);
