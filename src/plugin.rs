use nih_plug::prelude::*;
use std::sync::Arc;

use crate::audio::engine::SynthEngine;
use crate::audio::create_parameter_buffer;
use crate::params::{Waveform, FilterType};

/// DSynth VST plugin wrapper
struct DSynthPlugin {
    params: Arc<DSynthParams>,
    engine: SynthEngine,
    sample_rate: f32,
}

/// Plugin parameters that map to our SynthParams
#[derive(Params)]
struct DSynthParams {
    /// Master output gain
    #[id = "master_gain"]
    pub master_gain: FloatParam,

    /// Monophonic mode
    #[id = "monophonic"]
    pub monophonic: BoolParam,

    // Oscillator 1 parameters
    #[id = "osc1_waveform"]
    pub osc1_waveform: EnumParam<Waveform>,
    
    #[id = "osc1_pitch"]
    pub osc1_pitch: FloatParam,
    
    #[id = "osc1_detune"]
    pub osc1_detune: FloatParam,
    
    #[id = "osc1_gain"]
    pub osc1_gain: FloatParam,
    
    #[id = "osc1_pan"]
    pub osc1_pan: FloatParam,

    // Oscillator 2 parameters
    #[id = "osc2_waveform"]
    pub osc2_waveform: EnumParam<Waveform>,
    
    #[id = "osc2_pitch"]
    pub osc2_pitch: FloatParam,
    
    #[id = "osc2_detune"]
    pub osc2_detune: FloatParam,
    
    #[id = "osc2_gain"]
    pub osc2_gain: FloatParam,
    
    #[id = "osc2_pan"]
    pub osc2_pan: FloatParam,

    // Oscillator 3 parameters
    #[id = "osc3_waveform"]
    pub osc3_waveform: EnumParam<Waveform>,
    
    #[id = "osc3_pitch"]
    pub osc3_pitch: FloatParam,
    
    #[id = "osc3_detune"]
    pub osc3_detune: FloatParam,
    
    #[id = "osc3_gain"]
    pub osc3_gain: FloatParam,
    
    #[id = "osc3_pan"]
    pub osc3_pan: FloatParam,

    // Filter 1 parameters
    #[id = "filter1_type"]
    pub filter1_type: EnumParam<FilterType>,
    
    #[id = "filter1_cutoff"]
    pub filter1_cutoff: FloatParam,
    
    #[id = "filter1_resonance"]
    pub filter1_resonance: FloatParam,
    
    #[id = "filter1_amount"]
    pub filter1_amount: FloatParam,
}

impl Default for DSynthPlugin {
    fn default() -> Self {
        let (_producer, consumer) = create_parameter_buffer();
        Self {
            params: Arc::new(DSynthParams::default()),
            engine: SynthEngine::new(44100.0, consumer),
            sample_rate: 44100.0,
        }
    }
}

impl Default for DSynthParams {
    fn default() -> Self {
        Self {
            master_gain: FloatParam::new(
                "Master Gain",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            
            monophonic: BoolParam::new("Monophonic", false),

            // Oscillator 1
            osc1_waveform: EnumParam::new("Osc 1 Wave", Waveform::Sine),
            osc1_pitch: FloatParam::new(
                "Osc 1 Pitch",
                0.0,
                FloatRange::Linear { min: -24.0, max: 24.0 },
            ).with_unit(" semi"),
            osc1_detune: FloatParam::new(
                "Osc 1 Detune",
                0.0,
                FloatRange::Linear { min: -50.0, max: 50.0 },
            ).with_unit(" cents"),
            osc1_gain: FloatParam::new(
                "Osc 1 Gain",
                0.33,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc1_pan: FloatParam::new(
                "Osc 1 Pan",
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ),

            // Oscillator 2
            osc2_waveform: EnumParam::new("Osc 2 Wave", Waveform::Saw),
            osc2_pitch: FloatParam::new(
                "Osc 2 Pitch",
                0.0,
                FloatRange::Linear { min: -24.0, max: 24.0 },
            ).with_unit(" semi"),
            osc2_detune: FloatParam::new(
                "Osc 2 Detune",
                0.0,
                FloatRange::Linear { min: -50.0, max: 50.0 },
            ).with_unit(" cents"),
            osc2_gain: FloatParam::new(
                "Osc 2 Gain",
                0.33,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc2_pan: FloatParam::new(
                "Osc 2 Pan",
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ),

            // Oscillator 3
            osc3_waveform: EnumParam::new("Osc 3 Wave", Waveform::Square),
            osc3_pitch: FloatParam::new(
                "Osc 3 Pitch",
                0.0,
                FloatRange::Linear { min: -24.0, max: 24.0 },
            ).with_unit(" semi"),
            osc3_detune: FloatParam::new(
                "Osc 3 Detune",
                0.0,
                FloatRange::Linear { min: -50.0, max: 50.0 },
            ).with_unit(" cents"),
            osc3_gain: FloatParam::new(
                "Osc 3 Gain",
                0.33,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            osc3_pan: FloatParam::new(
                "Osc 3 Pan",
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ),

            // Filter 1
            filter1_type: EnumParam::new("Filter 1 Type", FilterType::Lowpass),
            filter1_cutoff: FloatParam::new(
                "Filter 1 Cutoff",
                8000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            ).with_unit(" Hz"),
            filter1_resonance: FloatParam::new(
                "Filter 1 Resonance",
                0.7,
                FloatRange::Linear { min: 0.1, max: 10.0 },
            ),
            filter1_amount: FloatParam::new(
                "Filter 1 Amount",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

impl Plugin for DSynthPlugin {
    const NAME: &'static str = "DSynth";
    const VENDOR: &'static str = "DSynth";
    const URL: &'static str = "https://github.com/yourusername/dsynth";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Update sample rate
        self.sample_rate = buffer_config.sample_rate;
        
        // Recreate engine with new sample rate
        let (_producer, consumer) = create_parameter_buffer();
        self.engine = SynthEngine::new(self.sample_rate, consumer);

        true
    }

    fn reset(&mut self) {
        // Reset all voices when transport stops or settings change
        self.engine.all_notes_off();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Process MIDI events
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn { note, velocity, .. } => {
                    self.engine.note_on(note, velocity);
                }
                NoteEvent::NoteOff { note, .. } => {
                    self.engine.note_off(note);
                }
                NoteEvent::PolyPressure { .. } => {}
                NoteEvent::PolyVolume { .. } => {}
                NoteEvent::PolyPan { .. } => {}
                NoteEvent::PolyTuning { .. } => {}
                NoteEvent::PolyVibrato { .. } => {}
                NoteEvent::PolyExpression { .. } => {}
                NoteEvent::PolyBrightness { .. } => {}
                NoteEvent::MidiChannelPressure { .. } => {}
                NoteEvent::MidiPitchBend { .. } => {}
                NoteEvent::MidiCC { .. } => {}
                NoteEvent::MidiProgramChange { .. } => {}
                _ => {}
            }
        }

        // Get output channels
        let output = buffer.as_slice();
        let channels = output.len();
        
        if channels < 2 {
            return ProcessStatus::Normal;
        }

        let samples = output[0].len();
        
        // Process audio block
        for sample_idx in 0..samples {
            let sample = self.engine.process();
            
            // Write to stereo output
            output[0][sample_idx] = sample;
            output[1][sample_idx] = sample;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for DSynthPlugin {
    const CLAP_ID: &'static str = "com.dsynth.dsynth";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Digital Synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for DSynthPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"DSynthPluginVST3";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
    ];
}

nih_export_clap!(DSynthPlugin);
nih_export_vst3!(DSynthPlugin);
