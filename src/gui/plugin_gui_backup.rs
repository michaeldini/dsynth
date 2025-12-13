use nih_plug::prelude::*;
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;
use std::collections::HashMap;

pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(1400, 900)
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
    scrollable_state: scrollable::State,
    param_states: HashMap<&'static str, nih_widgets::param_slider::State>,
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

        let title = Text::new("DSynth - Digital Synthesizer").size(28);

        // Master section
        let master_section = Column::new()
            .push(Text::new("Master").size(20))
            .push(param_row("Gain", &self.params.master_gain))
            .push(param_row_toggle("Monophonic", &self.params.monophonic))
            .spacing(5)
            .padding(10);

        // Oscillators in a row
        let oscillators = Row::new()
            .push(self.oscillator_section(1, &self.params.osc1_waveform, &self.params.osc1_pitch, 
                &self.params.osc1_detune, &self.params.osc1_gain, &self.params.osc1_pan,
                &self.params.osc1_unison, &self.params.osc1_unison_detune, &self.params.osc1_shape))
            .push(self.oscillator_section(2, &self.params.osc2_waveform, &self.params.osc2_pitch,
                &self.params.osc2_detune, &self.params.osc2_gain, &self.params.osc2_pan,
                &self.params.osc2_unison, &self.params.osc2_unison_detune, &self.params.osc2_shape))
            .push(self.oscillator_section(3, &self.params.osc3_waveform, &self.params.osc3_pitch,
                &self.params.osc3_detune, &self.params.osc3_gain, &self.params.osc3_pan,
                &self.params.osc3_unison, &self.params.osc3_unison_detune, &self.params.osc3_shape))
            .spacing(15)
            .padding(10);

        // Filters in a row
        let filters = Row::new()
            .push(self.filter_section(1, &self.params.filter1_type, &self.params.filter1_cutoff,
                &self.params.filter1_resonance, &self.params.filter1_drive, &self.params.filter1_amount))
            .push(self.filter_section(2, &self.params.filter2_type, &self.params.filter2_cutoff,
                &self.params.filter2_resonance, &self.params.filter2_drive, &self.params.filter1_amount))
            .push(self.filter_section(3, &self.params.filter3_type, &self.params.filter3_cutoff,
                &self.params.filter3_resonance, &self.params.filter3_drive, &self.params.filter1_amount))
            .spacing(15)
            .padding(10);

        // Filter envelopes in a row
        let filter_envs = Row::new()
            .push(self.envelope_section("Filter Env 1", &self.params.fenv1_attack, &self.params.fenv1_decay,
                &self.params.fenv1_sustain, &self.params.fenv1_release, &self.params.fenv1_amount))
            .push(self.envelope_section("Filter Env 2", &self.params.fenv2_attack, &self.params.fenv2_decay,
                &self.params.fenv2_sustain, &self.params.fenv2_release, &self.params.fenv2_amount))
            .push(self.envelope_section("Filter Env 3", &self.params.fenv3_attack, &self.params.fenv3_decay,
                &self.params.fenv3_sustain, &self.params.fenv3_release, &self.params.fenv3_amount))
            .spacing(15)
            .padding(10);

        // LFOs in a row
        let lfos = Row::new()
            .push(self.lfo_section(1, &self.params.lfo1_waveform, &self.params.lfo1_rate,
                &self.params.lfo1_depth, &self.params.lfo1_filter_amount))
            .push(self.lfo_section(2, &self.params.lfo2_waveform, &self.params.lfo2_rate,
                &self.params.lfo2_depth, &self.params.lfo2_filter_amount))
            .push(self.lfo_section(3, &self.params.lfo3_waveform, &self.params.lfo3_rate,
                &self.params.lfo3_depth, &self.params.lfo3_filter_amount))
            .spacing(15)
            .padding(10);

        // Velocity sensitivity
        let velocity_section = Column::new()
            .push(Text::new("Velocity Sensitivity").size(18))
            .push(param_row("Amplitude", &self.params.velocity_amp))
            .push(param_row("Filter", &self.params.velocity_filter))
            .push(param_row("Filter Env", &self.params.velocity_filter_env))
            .spacing(5)
            .padding(10);

        let content = Column::new()
            .push(title)
            .push(master_section)
            .push(Text::new("Oscillators").size(22))
            .push(oscillators)
            .push(Text::new("Filters").size(22))
            .push(filters)
            .push(Text::new("Filter Envelopes").size(22))
            .push(filter_envs)
            .push(Text::new("LFOs").size(22))
            .push(lfos)
            .push(velocity_section)
            .spacing(15)
            .padding(20);

        Scrollable::new(&mut self.scrollable_state)
            .push(content)
            .into()
    }
}

impl PluginGui {
    fn oscillator_section<'a>(
        &self,
        num: i32,
        waveform: &'a impl Param,
        pitch: &'a impl Param,
        detune: &'a impl Param,
        gain: &'a impl Param,
        pan: &'a impl Param,
        unison: &'a impl Param,
        unison_detune: &'a impl Param,
        shape: &'a impl Param,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        
        Column::new()
            .push(Text::new(format!("Oscillator {}", num)).size(18))
            .push(param_row("Waveform", waveform))
            .push(param_row("Pitch", pitch))
            .push(param_row("Detune", detune))
            .push(param_row("Gain", gain))
            .push(param_row("Pan", pan))
            .push(param_row("Unison", unison))
            .push(param_row("Uni Detune", unison_detune))
            .push(param_row("Shape", shape))
            .spacing(5)
            .padding(10)
    }

    fn filter_section<'a>(
        &self,
        num: i32,
        filter_type: &'a impl Param,
        cutoff: &'a impl Param,
        resonance: &'a impl Param,
        drive: &'a impl Param,
        amount: &'a impl Param,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        
        Column::new()
            .push(Text::new(format!("Filter {}", num)).size(18))
            .push(param_row("Type", filter_type))
            .push(param_row("Cutoff", cutoff))
            .push(param_row("Resonance", resonance))
            .push(param_row("Drive", drive))
            .push(param_row("Key Track", amount))
            .spacing(5)
            .padding(10)
    }

    fn envelope_section<'a>(
        &self,
        name: &str,
        attack: &'a impl Param,
        decay: &'a impl Param,
        sustain: &'a impl Param,
        release: &'a impl Param,
        amount: &'a impl Param,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        
        Column::new()
            .push(Text::new(name).size(18))
            .push(param_row("Attack", attack))
            .push(param_row("Decay", decay))
            .push(param_row("Sustain", sustain))
            .push(param_row("Release", release))
            .push(param_row("Amount", amount))
            .spacing(5)
            .padding(10)
    }

    fn lfo_section<'a>(
        &self,
        num: i32,
        waveform: &'a impl Param,
        rate: &'a impl Param,
        depth: &'a impl Param,
        filter_amount: &'a impl Param,
    ) -> widget::Column<'a, Message> {
        use widget::*;
        
        Column::new()
            .push(Text::new(format!("LFO {}", num)).size(18))
            .push(param_row("Waveform", waveform))
            .push(param_row("Rate", rate))
            .push(param_row("Depth", depth))
            .push(param_row("Filter Amt", filter_amount))
            .spacing(5)
            .padding(10)
    }
}

// Helper function to create a parameter row
fn param_row<'a, P: Param>(label: &str, param: &'a P, state: &'a mut nih_widgets::param_slider::State) -> widget::Row<'a, Message> {
    use widget::*;
    
    Row::new()
        .push(Text::new(label).width(Length::Units(90)))
        .push(
            nih_widgets::ParamSlider::new(state, param)
                .map(Message::ParamUpdate)
        )
        .spacing(10)
        .padding(3)
}

// Helper function for toggle parameters
fn param_row_toggle<'a, P: Param>(label: &str, param: &'a P, state: &'a mut nih_widgets::param_slider::State) -> widget::Row<'a, Message> {
    use widget::*;
    
    Row::new()
        .push(Text::new(label).width(Length::Units(90)))
        .push(
            nih_widgets::ParamSlider::new(state, param)
                .map(Message::ParamUpdate)
        )
        .spacing(10)
        .padding(3)
}
