/// CLAP GUI Host using iced_baseview
///
/// This module implements the GUI for the CLAP plugin using iced_baseview,
/// which embeds an iced window inside the DAW's native window.
///
/// Design:
/// - Uses iced_baseview to create an embedded window
/// - Connects to parameter system via Arc<RwLock<SynthParams>>
/// - Updates flow: GUI widget change → Message → update_param → triple buffer → audio thread
/// - Parameter reads: Audio thread → triple buffer → GUI reads current values
use crate::params::SynthParams;
use crate::plugin::param_update::ParamUpdateBuffer;
use std::sync::{Arc, RwLock};

#[cfg(feature = "clap")]
use iced_baseview::baseview::{Size, WindowOpenOptions, WindowScalePolicy};

/// Messages sent from GUI widgets
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// Parameter changed by user (param_id, normalized_value 0-1)
    ParamChanged(u32, f32),
    /// Preset load requested
    PresetLoad,
    /// Preset save requested
    PresetSave,
    /// Randomize parameters
    Randomize,
}

/// The main GUI state for the CLAP plugin
pub struct DSynthEditor {
    /// Current synthesizer parameters (read-only for GUI display)
    synth_params: Arc<RwLock<SynthParams>>,
    /// Parameter update buffer for sending changes to audio thread
    param_update_buffer: Arc<ParamUpdateBuffer>,
    /// Current preset name
    preset_name: String,
    /// Scale factor for HiDPI displays
    scale_factor: f32,
}

impl DSynthEditor {
    /// Create a new editor instance
    pub fn new(
        synth_params: Arc<RwLock<SynthParams>>,
        param_update_buffer: Arc<ParamUpdateBuffer>,
    ) -> Self {
        Self {
            synth_params,
            param_update_buffer,
            preset_name: "Init".to_string(),
            scale_factor: 1.0,
        }
    }

    /// Set the scale factor for HiDPI displays
    pub fn set_scale(&mut self, scale: f32) {
        self.scale_factor = scale;
    }
}

#[cfg(feature = "clap")]
impl iced_baseview::Application for DSynthEditor {
    type Message = Message;
    type Theme = iced_baseview::Theme;
    type Executor = iced_baseview::executor::Default;
    type Flags = (Arc<RwLock<SynthParams>>, Arc<ParamUpdateBuffer>);

    fn new(flags: Self::Flags) -> (Self, iced_baseview::Task<Self::Message>) {
        let (synth_params, param_update_buffer) = flags;
        (
            Self {
                synth_params,
                param_update_buffer,
                preset_name: "Init".to_string(),
                scale_factor: 1.0,
            },
            iced_baseview::Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> iced_baseview::Task<Self::Message> {
        match message {
            Message::ParamChanged(param_id, normalized_value) => {
                // Update the internal parameter state immediately so GUI reflects change
                if let Ok(mut params) = self.synth_params.write() {
                    use crate::plugin::param_update::param_apply;
                    param_apply::apply_param(&mut params, param_id, normalized_value);
                }

                // Queue parameter update for audio thread
                self.param_update_buffer
                    .queue_automation(param_id, normalized_value, 0);
            }
            Message::PresetLoad => {
                // TODO: Implement preset loading
            }
            Message::PresetSave => {
                // TODO: Implement preset saving
            }
            Message::Randomize => {
                // TODO: Implement randomization
            }
        }
        iced_baseview::Task::none()
    }

    fn view(
        &self,
    ) -> iced_baseview::Element<'_, Self::Message, Self::Theme, iced_baseview::Renderer> {
        use crate::params::Waveform;
        use crate::plugin::param_descriptor::*;
        use iced_baseview::widget::{Button, Column, Container, Row, Text};

        use super::knob::{
            detune_knob, frequency_knob, linear_knob, percent_knob, pitch_knob, time_knob,
        };

        // Read current parameters for display
        let params = self.synth_params.read().unwrap();

        // Header section
        let header = Row::new()
            .spacing(20)
            .padding(10)
            .push(Text::new("DSynth").size(32))
            .push(Text::new(format!("Preset: {}", self.preset_name)).size(16));

        // Oscillator 1 section
        let osc1 = Column::new()
            .spacing(10)
            .padding(10)
            .push(Text::new("Oscillator 1").size(20))
            .push(Row::new().spacing(10).push(Text::new(format!(
                "Wave: {:?}",
                params.oscillators[0].waveform
            ))))
            .push(
                Row::new()
                    .spacing(8)
                    .push(percent_knob(
                        "Gain",
                        params.oscillators[0].gain,
                        PARAM_OSC1_GAIN,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(pitch_knob(
                        "Pitch",
                        params.oscillators[0].pitch,
                        PARAM_OSC1_PITCH,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(detune_knob(
                        "Detune",
                        params.oscillators[0].detune,
                        PARAM_OSC1_DETUNE,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(percent_knob(
                        "Pan",
                        params.oscillators[0].pan,
                        PARAM_OSC1_PAN,
                        |id, v| Message::ParamChanged(id, v),
                    )),
            );

        // Oscillator 2 section
        let osc2 = Column::new()
            .spacing(10)
            .padding(10)
            .push(Text::new("Oscillator 2").size(20))
            .push(Row::new().spacing(10).push(Text::new(format!(
                "Wave: {:?}",
                params.oscillators[1].waveform
            ))))
            .push(
                Row::new()
                    .spacing(8)
                    .push(percent_knob(
                        "Gain",
                        params.oscillators[1].gain,
                        PARAM_OSC2_GAIN,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(pitch_knob(
                        "Pitch",
                        params.oscillators[1].pitch,
                        PARAM_OSC2_PITCH,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(detune_knob(
                        "Detune",
                        params.oscillators[1].detune,
                        PARAM_OSC2_DETUNE,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(percent_knob(
                        "Pan",
                        params.oscillators[1].pan,
                        PARAM_OSC2_PAN,
                        |id, v| Message::ParamChanged(id, v),
                    )),
            );

        // Oscillator 3 section
        let osc3 = Column::new()
            .spacing(10)
            .padding(10)
            .push(Text::new("Oscillator 3").size(20))
            .push(Row::new().spacing(10).push(Text::new(format!(
                "Wave: {:?}",
                params.oscillators[2].waveform
            ))))
            .push(
                Row::new()
                    .spacing(8)
                    .push(percent_knob(
                        "Gain",
                        params.oscillators[2].gain,
                        PARAM_OSC3_GAIN,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(pitch_knob(
                        "Pitch",
                        params.oscillators[2].pitch,
                        PARAM_OSC3_PITCH,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(detune_knob(
                        "Detune",
                        params.oscillators[2].detune,
                        PARAM_OSC3_DETUNE,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(percent_knob(
                        "Pan",
                        params.oscillators[2].pan,
                        PARAM_OSC3_PAN,
                        |id, v| Message::ParamChanged(id, v),
                    )),
            );

        // Filter 1 section
        let filter1 = Column::new()
            .spacing(10)
            .padding(10)
            .push(Text::new("Filter 1").size(20))
            .push(Row::new().spacing(10).push(Text::new(format!(
                "Type: {:?}",
                params.filters[0].filter_type
            ))))
            .push(
                Row::new()
                    .spacing(8)
                    .push(frequency_knob(
                        "Cutoff",
                        params.filters[0].cutoff,
                        PARAM_FILTER1_CUTOFF,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(linear_knob(
                        "Resonance",
                        params.filters[0].resonance,
                        PARAM_FILTER1_RESONANCE,
                        None,
                        2,
                        |id, v| Message::ParamChanged(id, v),
                    )),
            );

        // Filter 2 section
        let filter2 = Column::new()
            .spacing(10)
            .padding(10)
            .push(Text::new("Filter 2").size(20))
            .push(Row::new().spacing(10).push(Text::new(format!(
                "Type: {:?}",
                params.filters[1].filter_type
            ))))
            .push(
                Row::new()
                    .spacing(8)
                    .push(frequency_knob(
                        "Cutoff",
                        params.filters[1].cutoff,
                        PARAM_FILTER2_CUTOFF,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(linear_knob(
                        "Resonance",
                        params.filters[1].resonance,
                        PARAM_FILTER2_RESONANCE,
                        None,
                        2,
                        |id, v| Message::ParamChanged(id, v),
                    )),
            );

        // Filter 3 section
        let filter3 = Column::new()
            .spacing(10)
            .padding(10)
            .push(Text::new("Filter 3").size(20))
            .push(Row::new().spacing(10).push(Text::new(format!(
                "Type: {:?}",
                params.filters[2].filter_type
            ))))
            .push(
                Row::new()
                    .spacing(8)
                    .push(frequency_knob(
                        "Cutoff",
                        params.filters[2].cutoff,
                        PARAM_FILTER3_CUTOFF,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(linear_knob(
                        "Resonance",
                        params.filters[2].resonance,
                        PARAM_FILTER3_RESONANCE,
                        None,
                        2,
                        |id, v| Message::ParamChanged(id, v),
                    )),
            );

        // Envelope section
        let envelope = Column::new()
            .spacing(10)
            .padding(10)
            .push(Text::new("Envelope").size(20))
            .push(
                Row::new()
                    .spacing(8)
                    .push(time_knob(
                        "Attack",
                        params.envelope.attack,
                        PARAM_ENVELOPE_ATTACK,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(time_knob(
                        "Decay",
                        params.envelope.decay,
                        PARAM_ENVELOPE_DECAY,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(linear_knob(
                        "Sustain",
                        params.envelope.sustain,
                        PARAM_ENVELOPE_SUSTAIN,
                        None,
                        2,
                        |id, v| Message::ParamChanged(id, v),
                    ))
                    .push(time_knob(
                        "Release",
                        params.envelope.release,
                        PARAM_ENVELOPE_RELEASE,
                        |id, v| Message::ParamChanged(id, v),
                    )),
            );

        // Preset controls
        let preset_controls = Row::new()
            .spacing(10)
            .padding(10)
            .push(Button::new(Text::new("Load")).on_press(Message::PresetLoad))
            .push(Button::new(Text::new("Save")).on_press(Message::PresetSave))
            .push(Button::new(Text::new("Random")).on_press(Message::Randomize));

        // Main layout
        let content = Column::new()
            .spacing(10)
            .padding(10)
            .push(header)
            .push(Row::new().spacing(15).push(osc1).push(osc2).push(osc3))
            .push(
                Row::new()
                    .spacing(15)
                    .push(filter1)
                    .push(filter2)
                    .push(filter3),
            )
            .push(envelope)
            .push(preset_controls);

        Container::new(content)
            .width(iced_baseview::Length::Fill)
            .height(iced_baseview::Length::Fill)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        iced_baseview::Theme::Dark
    }
}

/// Open the GUI window for the CLAP plugin
#[cfg(feature = "clap")]
pub fn open_editor(
    parent: raw_window_handle::RawWindowHandle,
    synth_params: Arc<RwLock<SynthParams>>,
    param_update_buffer: Arc<ParamUpdateBuffer>,
) -> Result<Box<dyn std::any::Any>, String> {
    // Wrapper struct that implements HasRawWindowHandle
    struct ParentHandle(raw_window_handle::RawWindowHandle);

    unsafe impl raw_window_handle::HasRawWindowHandle for ParentHandle {
        fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
            self.0
        }
    }

    let parent_handle = ParentHandle(parent);

    let options = WindowOpenOptions {
        title: "DSynth".to_string(),
        size: Size::new(1200.0, 800.0),
        scale: WindowScalePolicy::SystemScaleFactor,
    };

    let flags = (synth_params, param_update_buffer);

    let settings = iced_baseview::Settings {
        window: options,
        graphics_settings: Default::default(),
        iced_baseview: Default::default(),
        ..Default::default()
    };

    let handle = iced_baseview::open_parented::<DSynthEditor, _>(&parent_handle, flags, settings);

    Ok(Box::new(handle))
}

#[cfg(not(feature = "clap"))]
pub fn open_editor(
    _parent: (),
    _synth_params: Arc<RwLock<SynthParams>>,
    _param_update_buffer: Arc<ParamUpdateBuffer>,
) -> Result<(), String> {
    Err("CLAP feature not enabled".to_string())
}
