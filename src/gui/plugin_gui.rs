use nih_plug::prelude::*;
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;

use crate::params::{FilterType, Waveform};

pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(1200, 800)
}

pub(crate) fn create(
    params: Arc<crate::plugin::DSynthParams>,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<PluginGui>(editor_state, params)
}

struct PluginGui {
    params: Arc<crate::plugin::DSynthParams>,
    context: Arc<dyn GuiContext>,

    // Scrollable state
    scrollable_state: scrollable::State,

    // Slider states
    master_gain_state: nih_widgets::param_slider::State,
    osc1_pitch_state: nih_widgets::param_slider::State,
    osc1_detune_state: nih_widgets::param_slider::State,
    osc1_gain_state: nih_widgets::param_slider::State,
    osc1_pan_state: nih_widgets::param_slider::State,
    osc2_pitch_state: nih_widgets::param_slider::State,
    osc2_detune_state: nih_widgets::param_slider::State,
    osc2_gain_state: nih_widgets::param_slider::State,
    osc2_pan_state: nih_widgets::param_slider::State,
    osc3_pitch_state: nih_widgets::param_slider::State,
    osc3_detune_state: nih_widgets::param_slider::State,
    osc3_gain_state: nih_widgets::param_slider::State,
    osc3_pan_state: nih_widgets::param_slider::State,
    filter1_cutoff_state: nih_widgets::param_slider::State,
    filter1_resonance_state: nih_widgets::param_slider::State,
    filter1_amount_state: nih_widgets::param_slider::State,
}

#[derive(Debug, Clone)]
enum Message {
    ParamUpdate(nih_widgets::ParamMessage),
}

impl IcedEditor for PluginGui {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = Arc<crate::plugin::DSynthParams>;

    fn new(
        params: Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        let editor = PluginGui {
            params,
            context,

            scrollable_state: Default::default(),

            master_gain_state: Default::default(),
            osc1_pitch_state: Default::default(),
            osc1_detune_state: Default::default(),
            osc1_gain_state: Default::default(),
            osc1_pan_state: Default::default(),
            osc2_pitch_state: Default::default(),
            osc2_detune_state: Default::default(),
            osc2_gain_state: Default::default(),
            osc2_pan_state: Default::default(),
            osc3_pitch_state: Default::default(),
            osc3_detune_state: Default::default(),
            osc3_gain_state: Default::default(),
            osc3_pan_state: Default::default(),
            filter1_cutoff_state: Default::default(),
            filter1_resonance_state: Default::default(),
            filter1_amount_state: Default::default(),
        };
        (editor, Command::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::ParamUpdate(param_message) => {
                self.handle_param_message(param_message);
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        use widget::*;

        let title = Text::new("DSynth").size(28);

        let master_section = Column::new()
            .push(Text::new("Master").size(20))
            .push(
                Row::new()
                    .push(Text::new("Gain:").width(Length::Units(100)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.master_gain_state,
                            &self.params.master_gain,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .spacing(10)
            .padding(10);

        // Oscillator 1
        let osc1 = Column::new()
            .push(Text::new("Oscillator 1").size(18))
            .push(
                Row::new()
                    .push(Text::new("Pitch:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc1_pitch_state,
                            &self.params.osc1_pitch,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Detune:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc1_detune_state,
                            &self.params.osc1_detune,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Gain:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc1_gain_state,
                            &self.params.osc1_gain,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Pan:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc1_pan_state,
                            &self.params.osc1_pan,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .spacing(10)
            .padding(10);

        // Oscillator 2
        let osc2 = Column::new()
            .push(Text::new("Oscillator 2").size(18))
            .push(
                Row::new()
                    .push(Text::new("Pitch:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc2_pitch_state,
                            &self.params.osc2_pitch,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Detune:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc2_detune_state,
                            &self.params.osc2_detune,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Gain:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc2_gain_state,
                            &self.params.osc2_gain,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Pan:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc2_pan_state,
                            &self.params.osc2_pan,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .spacing(10)
            .padding(10);

        // Oscillator 3
        let osc3 = Column::new()
            .push(Text::new("Oscillator 3").size(18))
            .push(
                Row::new()
                    .push(Text::new("Pitch:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc3_pitch_state,
                            &self.params.osc3_pitch,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Detune:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc3_detune_state,
                            &self.params.osc3_detune,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Gain:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc3_gain_state,
                            &self.params.osc3_gain,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Pan:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.osc3_pan_state,
                            &self.params.osc3_pan,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .spacing(10)
            .padding(10);

        let filter1_section = Column::new()
            .push(Text::new("Filter 1").size(18))
            .push(
                Row::new()
                    .push(Text::new("Cutoff:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.filter1_cutoff_state,
                            &self.params.filter1_cutoff,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Resonance:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.filter1_resonance_state,
                            &self.params.filter1_resonance,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .push(
                Row::new()
                    .push(Text::new("Amount:").width(Length::Units(80)))
                    .push(
                        nih_widgets::ParamSlider::new(
                            &mut self.filter1_amount_state,
                            &self.params.filter1_amount,
                        )
                        .map(Message::ParamUpdate),
                    )
                    .spacing(10)
                    .padding(5),
            )
            .spacing(10)
            .padding(10);

        let oscillators_row = Row::new()
            .push(osc1)
            .push(osc2)
            .push(osc3)
            .spacing(20)
            .padding(10);

        let content = Column::new()
            .push(title)
            .push(master_section)
            .push(oscillators_row)
            .push(filter1_section)
            .spacing(20)
            .padding(20);

        Scrollable::new(&mut self.scrollable_state)
            .push(content)
            .into()
    }
}
