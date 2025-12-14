use nih_plug::prelude::*;
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;

pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(1200, 900)
}

pub(crate) fn create(
    params: Arc<crate::plugin::DSynthParams>,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<PluginGui>(editor_state, params)
}

macro_rules! param_states {
    ($($name:ident),* $(,)?) => {
        $(pub $name: nih_widgets::param_slider::State,)*
    };
}

struct PluginGui {
    params: Arc<crate::plugin::DSynthParams>,
    context: Arc<dyn GuiContext>,
    scrollable_state: scrollable::State,

    // Parameter states
    param_states! {
        master_gain, monophonic,

        osc1_waveform, osc1_pitch, osc1_detune, osc1_gain, osc1_pan,
        osc1_unison, osc1_unison_detune, osc1_shape,

        osc2_waveform, osc2_pitch, osc2_detune, osc2_gain, osc2_pan,
        osc2_unison, osc2_unison_detune, osc2_shape,

        osc3_waveform, osc3_pitch, osc3_detune, osc3_gain, osc3_pan,
        osc3_unison, osc3_unison_detune, osc3_shape,

        filter1_type, filter1_cutoff, filter1_resonance, filter1_drive, filter1_amount,
        filter2_type, filter2_cutoff, filter2_resonance, filter2_drive,
        filter3_type, filter3_cutoff, filter3_resonance, filter3_drive,

        fenv1_attack, fenv1_decay, fenv1_sustain, fenv1_release, fenv1_amount,
        fenv2_attack, fenv2_decay, fenv2_sustain, fenv2_release, fenv2_amount,
        fenv3_attack, fenv3_decay, fenv3_sustain, fenv3_release, fenv3_amount,

        lfo1_waveform, lfo1_rate, lfo1_depth, lfo1_filter_amount,
        lfo2_waveform, lfo2_rate, lfo2_depth, lfo2_filter_amount,
        lfo3_waveform, lfo3_rate, lfo3_depth, lfo3_filter_amount,

        velocity_amp, velocity_filter, velocity_filter_env,
    }
}

#[derive(Debug, Clone)]
enum Message {
    ParamUpdate(nih_widgets::ParamMessage),
}

macro_rules! init_states {
    ($($name:ident),* $(,)?) => {
        $($name: Default::default(),)*
    };
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

            init_states! {
                master_gain, monophonic,

                osc1_waveform, osc1_pitch, osc1_detune, osc1_gain, osc1_pan,
                osc1_unison, osc1_unison_detune, osc1_shape,

                osc2_waveform, osc2_pitch, osc2_detune, osc2_gain, osc2_pan,
                osc2_unison, osc2_unison_detune, osc2_shape,

                osc3_waveform, osc3_pitch, osc3_detune, osc3_gain, osc3_pan,
                osc3_unison, osc3_unison_detune, osc3_shape,

                filter1_type, filter1_cutoff, filter1_resonance, filter1_drive, filter1_amount,
                filter2_type, filter2_cutoff, filter2_resonance, filter2_drive,
                filter3_type, filter3_cutoff, filter3_resonance, filter3_drive,

                fenv1_attack, fenv1_decay, fenv1_sustain, fenv1_release, fenv1_amount,
                fenv2_attack, fenv2_decay, fenv2_sustain, fenv2_release, fenv2_amount,
                fenv3_attack, fenv3_decay, fenv3_sustain, fenv3_release, fenv3_amount,

                lfo1_waveform, lfo1_rate, lfo1_depth, lfo1_filter_amount,
                lfo2_waveform, lfo2_rate, lfo2_depth, lfo2_filter_amount,
                lfo3_waveform, lfo3_rate, lfo3_depth, lfo3_filter_amount,

                velocity_amp, velocity_filter, velocity_filter_env,
            }
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

        let title = Text::new("DSynth").size(26);

        // Master controls
        let master = Column::new()
            .push(Text::new("Master").size(18))
            .push(param_row("Gain", &self.params.master_gain, &mut self.master_gain))
            .push(param_row(
                "Monophonic",
                &self.params.monophonic,
                &mut self.monophonic,
            ))
            .spacing(5)
            .padding(10);

        // Oscillators
        let osc1 = self.osc_section("Osc 1");
        let osc2 = self.osc_section2("Osc 2");
        let osc3 = self.osc_section3("Osc 3");

        let oscillators = Row::new().push(osc1).push(osc2).push(osc3).spacing(10);

        // Filters
        let filt1 = self.filter_section1("Filter 1");
        let filt2 = self.filter_section2("Filter 2");
        let filt3 = self.filter_section3("Filter 3");

        let filters = Row::new().push(filt1).push(filt2).push(filt3).spacing(10);

        // Filter Envelopes
        let fenv1 = self.fenv_section1("F-Env 1");
        let fenv2 = self.fenv_section2("F-Env 2");
        let fenv3 = self.fenv_section3("F-Env 3");

        let filter_envs = Row::new().push(fenv1).push(fenv2).push(fenv3).spacing(10);

        // LFOs
        let lfo1 = self.lfo_section1("LFO 1");
        let lfo2 = self.lfo_section2("LFO 2");
        let lfo3 = self.lfo_section3("LFO 3");

        let lfos = Row::new().push(lfo1).push(lfo2).push(lfo3).spacing(10);

        // Velocity
        let velocity = Column::new()
            .push(Text::new("Velocity").size(16))
            .push(param_row(
                "Amp",
                &self.params.velocity_amp,
                &mut self.velocity_amp,
            ))
            .push(param_row(
                "Filter",
                &self.params.velocity_filter,
                &mut self.velocity_filter,
            ))
            .push(param_row(
                "F-Env",
                &self.params.velocity_filter_env,
                &mut self.velocity_filter_env,
            ))
            .spacing(5)
            .padding(10);

        let content = Column::new()
            .push(title)
            .push(master)
            .push(oscillators)
            .push(filters)
            .push(filter_envs)
            .push(lfos)
            .push(velocity)
            .spacing(10)
            .padding(15);

        Scrollable::new(&mut self.scrollable_state)
            .push(content)
            .into()
    }
}

impl PluginGui {
    fn osc_section<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &self.params.osc1_waveform,
                &mut self.osc1_waveform,
            ))
            .push(param_row(
                "Pitch",
                &self.params.osc1_pitch,
                &mut self.osc1_pitch,
            ))
            .push(param_row(
                "Detune",
                &self.params.osc1_detune,
                &mut self.osc1_detune,
            ))
            .push(param_row(
                "Gain",
                &self.params.osc1_gain,
                &mut self.osc1_gain,
            ))
            .push(param_row("Pan", &self.params.osc1_pan, &mut self.osc1_pan))
            .push(param_row(
                "Unison",
                &self.params.osc1_unison,
                &mut self.osc1_unison,
            ))
            .push(param_row(
                "UniDet",
                &self.params.osc1_unison_detune,
                &mut self.osc1_unison_detune,
            ))
            .push(param_row(
                "Shape",
                &self.params.osc1_shape,
                &mut self.osc1_shape,
            ))
            .spacing(3)
            .padding(8)
    }

    fn osc_section2<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &self.params.osc2_waveform,
                &mut self.osc2_waveform,
            ))
            .push(param_row(
                "Pitch",
                &self.params.osc2_pitch,
                &mut self.osc2_pitch,
            ))
            .push(param_row(
                "Detune",
                &self.params.osc2_detune,
                &mut self.osc2_detune,
            ))
            .push(param_row(
                "Gain",
                &self.params.osc2_gain,
                &mut self.osc2_gain,
            ))
            .push(param_row("Pan", &self.params.osc2_pan, &mut self.osc2_pan))
            .push(param_row(
                "Unison",
                &self.params.osc2_unison,
                &mut self.osc2_unison,
            ))
            .push(param_row(
                "UniDet",
                &self.params.osc2_unison_detune,
                &mut self.osc2_unison_detune,
            ))
            .push(param_row(
                "Shape",
                &self.params.osc2_shape,
                &mut self.osc2_shape,
            ))
            .spacing(3)
            .padding(8)
    }

    fn osc_section3<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &self.params.osc3_waveform,
                &mut self.osc3_waveform,
            ))
            .push(param_row(
                "Pitch",
                &self.params.osc3_pitch,
                &mut self.osc3_pitch,
            ))
            .push(param_row(
                "Detune",
                &self.params.osc3_detune,
                &mut self.osc3_detune,
            ))
            .push(param_row(
                "Gain",
                &self.params.osc3_gain,
                &mut self.osc3_gain,
            ))
            .push(param_row("Pan", &self.params.osc3_pan, &mut self.osc3_pan))
            .push(param_row(
                "Unison",
                &self.params.osc3_unison,
                &mut self.osc3_unison,
            ))
            .push(param_row(
                "UniDet",
                &self.params.osc3_unison_detune,
                &mut self.osc3_unison_detune,
            ))
            .push(param_row(
                "Shape",
                &self.params.osc3_shape,
                &mut self.osc3_shape,
            ))
            .spacing(3)
            .padding(8)
    }

    fn filter_section1<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &self.params.filter1_type,
                &mut self.filter1_type,
            ))
            .push(param_row(
                "Cutoff",
                &self.params.filter1_cutoff,
                &mut self.filter1_cutoff,
            ))
            .push(param_row(
                "Res",
                &self.params.filter1_resonance,
                &mut self.filter1_resonance,
            ))
            .push(param_row(
                "Drive",
                &self.params.filter1_drive,
                &mut self.filter1_drive,
            ))
            .push(param_row(
                "KeyTrk",
                &self.params.filter1_amount,
                &mut self.filter1_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn filter_section2<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &self.params.filter2_type,
                &mut self.filter2_type,
            ))
            .push(param_row(
                "Cutoff",
                &self.params.filter2_cutoff,
                &mut self.filter2_cutoff,
            ))
            .push(param_row(
                "Res",
                &self.params.filter2_resonance,
                &mut self.filter2_resonance,
            ))
            .push(param_row(
                "Drive",
                &self.params.filter2_drive,
                &mut self.filter2_drive,
            ))
            .spacing(3)
            .padding(8)
    }

    fn filter_section3<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &self.params.filter3_type,
                &mut self.filter3_type,
            ))
            .push(param_row(
                "Cutoff",
                &self.params.filter3_cutoff,
                &mut self.filter3_cutoff,
            ))
            .push(param_row(
                "Res",
                &self.params.filter3_resonance,
                &mut self.filter3_resonance,
            ))
            .push(param_row(
                "Drive",
                &self.params.filter3_drive,
                &mut self.filter3_drive,
            ))
            .spacing(3)
            .padding(8)
    }

    fn fenv_section1<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Attack",
                &self.params.fenv1_attack,
                &mut self.fenv1_attack,
            ))
            .push(param_row(
                "Decay",
                &self.params.fenv1_decay,
                &mut self.fenv1_decay,
            ))
            .push(param_row(
                "Sustain",
                &self.params.fenv1_sustain,
                &mut self.fenv1_sustain,
            ))
            .push(param_row(
                "Release",
                &self.params.fenv1_release,
                &mut self.fenv1_release,
            ))
            .push(param_row(
                "Amount",
                &self.params.fenv1_amount,
                &mut self.fenv1_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn fenv_section2<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Attack",
                &self.params.fenv2_attack,
                &mut self.fenv2_attack,
            ))
            .push(param_row(
                "Decay",
                &self.params.fenv2_decay,
                &mut self.fenv2_decay,
            ))
            .push(param_row(
                "Sustain",
                &self.params.fenv2_sustain,
                &mut self.fenv2_sustain,
            ))
            .push(param_row(
                "Release",
                &self.params.fenv2_release,
                &mut self.fenv2_release,
            ))
            .push(param_row(
                "Amount",
                &self.params.fenv2_amount,
                &mut self.fenv2_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn fenv_section3<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Attack",
                &self.params.fenv3_attack,
                &mut self.fenv3_attack,
            ))
            .push(param_row(
                "Decay",
                &self.params.fenv3_decay,
                &mut self.fenv3_decay,
            ))
            .push(param_row(
                "Sustain",
                &self.params.fenv3_sustain,
                &mut self.fenv3_sustain,
            ))
            .push(param_row(
                "Release",
                &self.params.fenv3_release,
                &mut self.fenv3_release,
            ))
            .push(param_row(
                "Amount",
                &self.params.fenv3_amount,
                &mut self.fenv3_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn lfo_section1<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &self.params.lfo1_waveform,
                &mut self.lfo1_waveform,
            ))
            .push(param_row(
                "Rate",
                &self.params.lfo1_rate,
                &mut self.lfo1_rate,
            ))
            .push(param_row(
                "Depth",
                &self.params.lfo1_depth,
                &mut self.lfo1_depth,
            ))
            .push(param_row(
                "F-Amt",
                &self.params.lfo1_filter_amount,
                &mut self.lfo1_filter_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn lfo_section2<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &self.params.lfo2_waveform,
                &mut self.lfo2_waveform,
            ))
            .push(param_row(
                "Rate",
                &self.params.lfo2_rate,
                &mut self.lfo2_rate,
            ))
            .push(param_row(
                "Depth",
                &self.params.lfo2_depth,
                &mut self.lfo2_depth,
            ))
            .push(param_row(
                "F-Amt",
                &self.params.lfo2_filter_amount,
                &mut self.lfo2_filter_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn lfo_section3<'a>(&'a mut self, title: &str) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &self.params.lfo3_waveform,
                &mut self.lfo3_waveform,
            ))
            .push(param_row(
                "Rate",
                &self.params.lfo3_rate,
                &mut self.lfo3_rate,
            ))
            .push(param_row(
                "Depth",
                &self.params.lfo3_depth,
                &mut self.lfo3_depth,
            ))
            .push(param_row(
                "F-Amt",
                &self.params.lfo3_filter_amount,
                &mut self.lfo3_filter_amount,
            ))
            .spacing(3)
            .padding(8)
    }
}

fn param_row<'a, P: Param>(
    label: &str,
    param: &'a P,
    state: &'a mut nih_widgets::param_slider::State,
) -> widget::Row<'a, Message> {
    use widget::*;
    Row::new()
        .push(Text::new(label).width(Length::Units(60)))
        .push(nih_widgets::ParamSlider::new(state, param).map(Message::ParamUpdate))
        .spacing(8)
        .padding(2)
}
