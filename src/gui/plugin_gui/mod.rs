use nih_plug::prelude::*;
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;

mod helpers;
mod randomize;
mod sections;

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
    phase: nih_widgets::param_slider::State,
    shape: nih_widgets::param_slider::State,
    fm_source: nih_widgets::param_slider::State,
    fm_amount: nih_widgets::param_slider::State,
    // Additive harmonics
    h1: nih_widgets::param_slider::State,
    h2: nih_widgets::param_slider::State,
    h3: nih_widgets::param_slider::State,
    h4: nih_widgets::param_slider::State,
    h5: nih_widgets::param_slider::State,
    h6: nih_widgets::param_slider::State,
    h7: nih_widgets::param_slider::State,
    h8: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct FilterStates {
    filter_type: nih_widgets::param_slider::State,
    cutoff: nih_widgets::param_slider::State,
    resonance: nih_widgets::param_slider::State,
    bandwidth: nih_widgets::param_slider::State,
    key_tracking: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct LfoStates {
    waveform: nih_widgets::param_slider::State,
    rate: nih_widgets::param_slider::State,
    depth: nih_widgets::param_slider::State,
    filter_amount: nih_widgets::param_slider::State,
    pitch_amount: nih_widgets::param_slider::State,
    gain_amount: nih_widgets::param_slider::State,
    pan_amount: nih_widgets::param_slider::State,
    pwm_amount: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct VelocityStates {
    amp: nih_widgets::param_slider::State,
    filter: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct EnvelopeStates {
    attack: nih_widgets::param_slider::State,
    decay: nih_widgets::param_slider::State,
    sustain: nih_widgets::param_slider::State,
    release: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct EffectsStates {
    // Reverb
    reverb_room_size: nih_widgets::param_slider::State,
    reverb_damping: nih_widgets::param_slider::State,
    reverb_wet: nih_widgets::param_slider::State,
    reverb_dry: nih_widgets::param_slider::State,
    reverb_width: nih_widgets::param_slider::State,
    // Delay
    delay_time_ms: nih_widgets::param_slider::State,
    delay_feedback: nih_widgets::param_slider::State,
    delay_wet: nih_widgets::param_slider::State,
    delay_dry: nih_widgets::param_slider::State,
    // Chorus
    chorus_rate: nih_widgets::param_slider::State,
    chorus_depth: nih_widgets::param_slider::State,
    chorus_mix: nih_widgets::param_slider::State,
    // Distortion
    distortion_type: nih_widgets::param_slider::State,
    distortion_drive: nih_widgets::param_slider::State,
    distortion_mix: nih_widgets::param_slider::State,
}

#[derive(Default)]
struct ParamStates {
    master_gain: nih_widgets::param_slider::State,

    osc1: OscStates,
    osc2: OscStates,
    osc3: OscStates,

    filter1: FilterStates,
    filter2: FilterStates,
    filter3: FilterStates,

    lfo1: LfoStates,
    lfo2: LfoStates,
    lfo3: LfoStates,

    velocity: VelocityStates,
    envelope: EnvelopeStates,
    effects: EffectsStates,
}

pub struct PluginGui {
    params: Arc<crate::plugin::DSynthParams>,
    context: Arc<dyn GuiContext>,
    scrollable_state: scrollable::State,
    randomize_button_state: button::State,

    param_states: ParamStates,
}

#[derive(Debug, Clone)]
pub enum Message {
    ParamUpdate(nih_widgets::ParamMessage),
    Randomize,
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
            randomize_button_state: Default::default(),

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
            Message::Randomize => {
                self.randomize_params();
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        use widget::*;

        let params = self.params.as_ref();

        let title = Text::new("DSynth").size(26);
        let header = Row::new()
            .push(title)
            .push(Space::with_width(Length::Fill))
            .push(
                Button::new(&mut self.randomize_button_state, Text::new("Randomize"))
                    .on_press(Message::Randomize)
                    .padding(8),
            )
            .spacing(10)
            .padding(5);

        // Master controls
        let master = Column::new()
            .push(Text::new("Master").size(18))
            .push(helpers::param_row(
                "Gain",
                &params.master_gain,
                &mut self.param_states.master_gain,
            ))
            .push(helpers::param_checkbox("Monophonic", &params.monophonic))
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

        // LFOs
        let lfo1 = Self::lfo1_section("LFO 1", params, &mut self.param_states.lfo1);
        let lfo2 = Self::lfo2_section("LFO 2", params, &mut self.param_states.lfo2);
        let lfo3 = Self::lfo3_section("LFO 3", params, &mut self.param_states.lfo3);

        let lfos = Row::new().push(lfo1).push(lfo2).push(lfo3).spacing(10);

        // Velocity sensitivity
        let velocity = Column::new()
            .push(Text::new("Velocity").size(18))
            .push(helpers::param_row(
                "Amp",
                &params.velocity_amp,
                &mut self.param_states.velocity.amp,
            ))
            .push(helpers::param_row(
                "Filter",
                &params.velocity_filter,
                &mut self.param_states.velocity.filter,
            ))
            .spacing(5)
            .padding(10);

        // Envelope (ADSR)
        let envelope = Self::envelope_section(params, &mut self.param_states.envelope);

        // Effects
        let effects = Self::effects_section(params, &mut self.param_states.effects);

        // Main layout
        let content = Column::new()
            .push(header)
            .push(master)
            .push(oscillators)
            .push(filters)
            .push(lfos)
            .push(velocity)
            .push(envelope)
            .push(effects)
            .spacing(5);

        Scrollable::new(&mut self.scrollable_state)
            .push(content)
            .into()
    }
}

impl PluginGui {
    fn handle_param_message(&mut self, message: nih_widgets::ParamMessage) {
        // Handle parameter updates from sliders and other widgets
        // This is the standard pattern from nih_plug_iced's IcedEditor trait
        match message {
            nih_widgets::ParamMessage::BeginSetParameter(ptr) => unsafe {
                self.context.raw_begin_set_parameter(ptr)
            },
            nih_widgets::ParamMessage::SetParameterNormalized(ptr, value) => unsafe {
                self.context.raw_set_parameter_normalized(ptr, value)
            },
            nih_widgets::ParamMessage::EndSetParameter(ptr) => unsafe {
                self.context.raw_end_set_parameter(ptr)
            },
        }
    }
}
