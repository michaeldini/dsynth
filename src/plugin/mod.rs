use nih_plug::prelude::*;
use std::sync::Arc;
use triple_buffer::Input;

use crate::audio::create_parameter_buffer;
use crate::audio::engine::SynthEngine;
use crate::params::SynthParams;

#[cfg(feature = "vst")]
use crate::gui::plugin_gui;

mod convert;
mod params;

pub use params::DSynthParams;

/// DSynth VST plugin wrapper
pub struct DSynthPlugin {
    params: Arc<DSynthParams>,
    params_producer: Input<SynthParams>,
    engine: SynthEngine,
    sample_rate: f32,
}

impl Default for DSynthPlugin {
    fn default() -> Self {
        let (producer, consumer) = create_parameter_buffer();
        Self {
            params: Arc::new(DSynthParams::default()),
            params_producer: producer,
            engine: SynthEngine::new(44100.0, consumer),
            sample_rate: 44100.0,
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

    #[cfg(feature = "vst")]
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        plugin_gui::create(self.params.clone(), plugin_gui::default_state())
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
        let (producer, consumer) = create_parameter_buffer();
        self.params_producer = producer;
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
        // Convert and send parameters to engine
        let synth_params = self.convert_params();
        self.params_producer.write(synth_params);

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

        let (left_channel, rest) = output.split_at_mut(1);
        let left = &mut left_channel[0];
        let right = &mut rest[0];

        // Process audio block
        for (left, right) in left.iter_mut().zip(right.iter_mut()) {
            let sample = self.engine.process();

            *left = sample;
            *right = sample;
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
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(DSynthPlugin);
nih_export_vst3!(DSynthPlugin);
