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

#[derive(Default)]
struct OscStates {
    waveform: nih_widgets::param_slider::State,
    pitch: nih_widgets::param_slider::State,
    detune: nih_widgets::param_slider::State,
    gain: nih_widgets::param_slider::State,
    pan: nih_widgets::param_slider::State,
    unison: nih_widgets::param_slider::State,
    unison_detune: nih_widgets::param_slider::State,
    shape: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct FilterStates {
    filter_type: nih_widgets::param_slider::State,
    cutoff: nih_widgets::param_slider::State,
    resonance: nih_widgets::param_slider::State,
    drive: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct Filter1States {
    base: FilterStates,
    key_tracking: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct EnvStates {
    attack: nih_widgets::param_slider::State,
    decay: nih_widgets::param_slider::State,
    sustain: nih_widgets::param_slider::State,
    release: nih_widgets::param_slider::State,
    amount: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct LfoStates {
    waveform: nih_widgets::param_slider::State,
    rate: nih_widgets::param_slider::State,
    depth: nih_widgets::param_slider::State,
    filter_amount: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct VelocityStates {
    amp: nih_widgets::param_slider::State,
    filter: nih_widgets::param_slider::State,
    filter_env: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct ParamStates {
    master_gain: nih_widgets::param_slider::State,
    monophonic: nih_widgets::param_slider::State,

    osc1: OscStates,
    osc2: OscStates,
    osc3: OscStates,

    filter1: Filter1States,
    filter2: FilterStates,
    filter3: FilterStates,

    fenv1: EnvStates,
    fenv2: EnvStates,
    fenv3: EnvStates,

    lfo1: LfoStates,
    lfo2: LfoStates,
    lfo3: LfoStates,

    velocity: VelocityStates,
}

struct PluginGui {
    params: Arc<crate::plugin::DSynthParams>,
    context: Arc<dyn GuiContext>,
    scrollable_state: scrollable::State,

    param_states: ParamStates,
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

            param_states: Default::default(),
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

        let params = self.params.as_ref();

        let title = Text::new("DSynth").size(26);

        // Master controls
        let master = Column::new()
            .push(Text::new("Master").size(18))
            .push(param_row(
                "Gain",
                &params.master_gain,
                &mut self.param_states.master_gain,
            ))
            .push(param_row(
                "Monophonic",
                &params.monophonic,
                &mut self.param_states.monophonic,
            ))
            .spacing(5)
            .padding(10);

        // Oscillators
        let osc1 = Self::osc1_section("Osc 1", params, &mut self.param_states.osc1);
        let osc2 = Self::osc2_section("Osc 2", params, &mut self.param_states.osc2);
        let osc3 = Self::osc3_section("Osc 3", params, &mut self.param_states.osc3);

        let oscillators = Row::new().push(osc1).push(osc2).push(osc3).spacing(10);

        // Filters
        let filt1 = Self::filter1_section("Filter 1", params, &mut self.param_states.filter1);
        let filt2 = Self::filter2_section("Filter 2", params, &mut self.param_states.filter2);
        let filt3 = Self::filter3_section("Filter 3", params, &mut self.param_states.filter3);

        let filters = Row::new().push(filt1).push(filt2).push(filt3).spacing(10);

        // Filter Envelopes
        let fenv1 = Self::fenv1_section("F-Env 1", params, &mut self.param_states.fenv1);
        let fenv2 = Self::fenv2_section("F-Env 2", params, &mut self.param_states.fenv2);
        let fenv3 = Self::fenv3_section("F-Env 3", params, &mut self.param_states.fenv3);

        let filter_envs = Row::new().push(fenv1).push(fenv2).push(fenv3).spacing(10);

        // LFOs
        let lfo1 = Self::lfo1_section("LFO 1", params, &mut self.param_states.lfo1);
        let lfo2 = Self::lfo2_section("LFO 2", params, &mut self.param_states.lfo2);
        let lfo3 = Self::lfo3_section("LFO 3", params, &mut self.param_states.lfo3);

        let lfos = Row::new().push(lfo1).push(lfo2).push(lfo3).spacing(10);

        // Velocity
        let velocity = Column::new()
            .push(Text::new("Velocity").size(16))
            .push(param_row(
                "Amp",
                &params.velocity_amp,
                &mut self.param_states.velocity.amp,
            ))
            .push(param_row(
                "Filter",
                &params.velocity_filter,
                &mut self.param_states.velocity.filter,
            ))
            .push(param_row(
                "F-Env",
                &params.velocity_filter_env,
                &mut self.param_states.velocity.filter_env,
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
    fn osc1_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut OscStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.osc1_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Pitch", &params.osc1_pitch, &mut states.pitch))
            .push(param_row("Detune", &params.osc1_detune, &mut states.detune))
            .push(param_row("Gain", &params.osc1_gain, &mut states.gain))
            .push(param_row("Pan", &params.osc1_pan, &mut states.pan))
            .push(param_row("Unison", &params.osc1_unison, &mut states.unison))
            .push(param_row(
                "UniDet",
                &params.osc1_unison_detune,
                &mut states.unison_detune,
            ))
            .push(param_row("Shape", &params.osc1_shape, &mut states.shape))
            .spacing(3)
            .padding(8)
    }

    fn osc2_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut OscStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.osc2_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Pitch", &params.osc2_pitch, &mut states.pitch))
            .push(param_row("Detune", &params.osc2_detune, &mut states.detune))
            .push(param_row("Gain", &params.osc2_gain, &mut states.gain))
            .push(param_row("Pan", &params.osc2_pan, &mut states.pan))
            .push(param_row("Unison", &params.osc2_unison, &mut states.unison))
            .push(param_row(
                "UniDet",
                &params.osc2_unison_detune,
                &mut states.unison_detune,
            ))
            .push(param_row("Shape", &params.osc2_shape, &mut states.shape))
            .spacing(3)
            .padding(8)
    }

    fn osc3_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut OscStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.osc3_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Pitch", &params.osc3_pitch, &mut states.pitch))
            .push(param_row("Detune", &params.osc3_detune, &mut states.detune))
            .push(param_row("Gain", &params.osc3_gain, &mut states.gain))
            .push(param_row("Pan", &params.osc3_pan, &mut states.pan))
            .push(param_row("Unison", &params.osc3_unison, &mut states.unison))
            .push(param_row(
                "UniDet",
                &params.osc3_unison_detune,
                &mut states.unison_detune,
            ))
            .push(param_row("Shape", &params.osc3_shape, &mut states.shape))
            .spacing(3)
            .padding(8)
    }

    fn filter1_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut Filter1States,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &params.filter1_type,
                &mut states.base.filter_type,
            ))
            .push(param_row(
                "Cutoff",
                &params.filter1_cutoff,
                &mut states.base.cutoff,
            ))
            .push(param_row(
                "Res",
                &params.filter1_resonance,
                &mut states.base.resonance,
            ))
            .push(param_row(
                "Drive",
                &params.filter1_drive,
                &mut states.base.drive,
            ))
            .push(param_row(
                "KeyTrk",
                &params.filter1_amount,
                &mut states.key_tracking,
            ))
            .spacing(3)
            .padding(8)
    }

    fn filter2_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut FilterStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &params.filter2_type,
                &mut states.filter_type,
            ))
            .push(param_row(
                "Cutoff",
                &params.filter2_cutoff,
                &mut states.cutoff,
            ))
            .push(param_row(
                "Res",
                &params.filter2_resonance,
                &mut states.resonance,
            ))
            .push(param_row("Drive", &params.filter2_drive, &mut states.drive))
            .spacing(3)
            .padding(8)
    }

    fn filter3_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut FilterStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &params.filter3_type,
                &mut states.filter_type,
            ))
            .push(param_row(
                "Cutoff",
                &params.filter3_cutoff,
                &mut states.cutoff,
            ))
            .push(param_row(
                "Res",
                &params.filter3_resonance,
                &mut states.resonance,
            ))
            .push(param_row("Drive", &params.filter3_drive, &mut states.drive))
            .spacing(3)
            .padding(8)
    }

    fn fenv1_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut EnvStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Attack",
                &params.fenv1_attack,
                &mut states.attack,
            ))
            .push(param_row("Decay", &params.fenv1_decay, &mut states.decay))
            .push(param_row(
                "Sustain",
                &params.fenv1_sustain,
                &mut states.sustain,
            ))
            .push(param_row(
                "Release",
                &params.fenv1_release,
                &mut states.release,
            ))
            .push(param_row(
                "Amount",
                &params.fenv1_amount,
                &mut states.amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn fenv2_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut EnvStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Attack",
                &params.fenv2_attack,
                &mut states.attack,
            ))
            .push(param_row("Decay", &params.fenv2_decay, &mut states.decay))
            .push(param_row(
                "Sustain",
                &params.fenv2_sustain,
                &mut states.sustain,
            ))
            .push(param_row(
                "Release",
                &params.fenv2_release,
                &mut states.release,
            ))
            .push(param_row(
                "Amount",
                &params.fenv2_amount,
                &mut states.amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn fenv3_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut EnvStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Attack",
                &params.fenv3_attack,
                &mut states.attack,
            ))
            .push(param_row("Decay", &params.fenv3_decay, &mut states.decay))
            .push(param_row(
                "Sustain",
                &params.fenv3_sustain,
                &mut states.sustain,
            ))
            .push(param_row(
                "Release",
                &params.fenv3_release,
                &mut states.release,
            ))
            .push(param_row(
                "Amount",
                &params.fenv3_amount,
                &mut states.amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn lfo1_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut LfoStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.lfo1_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Rate", &params.lfo1_rate, &mut states.rate))
            .push(param_row("Depth", &params.lfo1_depth, &mut states.depth))
            .push(param_row(
                "F-Amt",
                &params.lfo1_filter_amount,
                &mut states.filter_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn lfo2_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut LfoStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.lfo2_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Rate", &params.lfo2_rate, &mut states.rate))
            .push(param_row("Depth", &params.lfo2_depth, &mut states.depth))
            .push(param_row(
                "F-Amt",
                &params.lfo2_filter_amount,
                &mut states.filter_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    fn lfo3_section<'a>(
        title: &str,
        params: &'a crate::plugin::DSynthParams,
        states: &'a mut LfoStates,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.lfo3_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Rate", &params.lfo3_rate, &mut states.rate))
            .push(param_row("Depth", &params.lfo3_depth, &mut states.depth))
            .push(param_row(
                "F-Amt",
                &params.lfo3_filter_amount,
                &mut states.filter_amount,
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
