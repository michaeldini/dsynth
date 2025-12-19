use crate::audio::output::EngineEvent;
use crate::params::SynthParams;
use crossbeam_channel::Sender;
use iced::{
    Element, Length, Task, event, keyboard,
    widget::{button, column, container, row, scrollable, slider, text, text_input},
};
use std::collections::HashSet;
use triple_buffer::Input;

use super::keyboard::key_to_midi_note;
use super::messages::*;
use super::preset_manager;
use super::sections;

pub struct SynthGui {
    params: SynthParams,
    param_producer: Option<Input<SynthParams>>,
    event_sender: Option<Sender<EngineEvent>>,
    pressed_keys: HashSet<keyboard::Key>,
    preset_name: String,
}

impl SynthGui {
    pub fn new(
        param_producer: Option<Input<SynthParams>>,
        event_sender: Option<Sender<EngineEvent>>,
    ) -> Self {
        Self {
            params: SynthParams::default(),
            param_producer,
            event_sender,
            pressed_keys: HashSet::new(),
            preset_name: String::from("My Preset"),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Oscillator parameters
            Message::Oscillator(idx, msg) => {
                if idx < 3 {
                    let osc = &mut self.params.oscillators[idx];
                    match msg {
                        OscillatorMessage::WaveformChanged(w) => osc.waveform = w,
                        OscillatorMessage::PitchChanged(p) => osc.pitch = p,
                        OscillatorMessage::DetuneChanged(d) => osc.detune = d,
                        OscillatorMessage::GainChanged(g) => osc.gain = g,
                        OscillatorMessage::PanChanged(p) => osc.pan = p,
                        OscillatorMessage::UnisonChanged(u) => osc.unison = u,
                        OscillatorMessage::UnisonDetuneChanged(d) => osc.unison_detune = d,
                        OscillatorMessage::PhaseChanged(p) => osc.phase = p,
                        OscillatorMessage::ShapeChanged(s) => osc.shape = s,
                        OscillatorMessage::SoloToggled(s) => osc.solo = s,
                        OscillatorMessage::FmSourceChanged(src) => osc.fm_source = src,
                        OscillatorMessage::FmAmountChanged(amt) => osc.fm_amount = amt,
                    }
                }
            }

            // Filter parameters
            Message::Filter(idx, msg) => {
                if idx < 3 {
                    let filter = &mut self.params.filters[idx];
                    match msg {
                        FilterMessage::TypeChanged(t) => filter.filter_type = t,
                        FilterMessage::CutoffChanged(c) => filter.cutoff = c,
                        FilterMessage::ResonanceChanged(r) => filter.resonance = r,
                        FilterMessage::BandwidthChanged(b) => filter.bandwidth = b,
                        FilterMessage::KeyTrackingChanged(k) => filter.key_tracking = k,
                    }
                }
            }

            // LFO parameters
            Message::LFO(idx, msg) => {
                if idx < 3 {
                    let lfo = &mut self.params.lfos[idx];
                    match msg {
                        LFOMessage::WaveformChanged(w) => lfo.waveform = w,
                        LFOMessage::RateChanged(r) => lfo.rate = r,
                        LFOMessage::DepthChanged(d) => lfo.depth = d,
                        LFOMessage::FilterAmountChanged(a) => lfo.filter_amount = a,
                    }
                }
            }

            // Effects
            Message::Reverb(msg) => match msg {
                ReverbMessage::RoomSizeChanged(v) => self.params.effects.reverb.room_size = v,
                ReverbMessage::DampingChanged(v) => self.params.effects.reverb.damping = v,
                ReverbMessage::WetChanged(v) => self.params.effects.reverb.wet = v,
                ReverbMessage::DryChanged(v) => self.params.effects.reverb.dry = v,
                ReverbMessage::WidthChanged(v) => self.params.effects.reverb.width = v,
            },
            Message::Delay(msg) => match msg {
                DelayMessage::TimeChanged(v) => self.params.effects.delay.time_ms = v,
                DelayMessage::FeedbackChanged(v) => self.params.effects.delay.feedback = v,
                DelayMessage::WetChanged(v) => self.params.effects.delay.wet = v,
                DelayMessage::DryChanged(v) => self.params.effects.delay.dry = v,
            },
            Message::Chorus(msg) => match msg {
                ChorusMessage::RateChanged(v) => self.params.effects.chorus.rate = v,
                ChorusMessage::DepthChanged(v) => self.params.effects.chorus.depth = v,
                ChorusMessage::MixChanged(v) => self.params.effects.chorus.mix = v,
            },
            Message::Distortion(msg) => match msg {
                DistortionMessage::DriveChanged(v) => self.params.effects.distortion.drive = v,
                DistortionMessage::MixChanged(v) => self.params.effects.distortion.mix = v,
                DistortionMessage::TypeChanged(t) => self.params.effects.distortion.dist_type = t,
            },

            // Envelope (ADSR)
            Message::Envelope(msg) => match msg {
                EnvelopeMessage::AttackChanged(v) => self.params.envelope.attack = v,
                EnvelopeMessage::DecayChanged(v) => self.params.envelope.decay = v,
                EnvelopeMessage::SustainChanged(v) => self.params.envelope.sustain = v,
                EnvelopeMessage::ReleaseChanged(v) => self.params.envelope.release = v,
            },

            // Velocity Sensitivity
            Message::VelocityAmpChanged(v) => self.params.velocity.amp_sensitivity = v,
            Message::VelocityFilterChanged(v) => self.params.velocity.filter_sensitivity = v,

            // Master
            Message::MasterGainChanged(g) => self.params.master_gain = g,
            Message::MonophonicToggled(mono) => self.params.monophonic = mono,
            Message::PanicPressed => {
                if let Some(sender) = &self.event_sender {
                    let _ = sender.try_send(EngineEvent::AllNotesOff);
                }
                self.pressed_keys.clear();
            }

            // Keyboard events
            Message::KeyPressed(key) => {
                if !self.pressed_keys.contains(&key)
                    && let Some(note) = key_to_midi_note(&key)
                    && let Some(sender) = &self.event_sender
                {
                    let _ = sender.try_send(EngineEvent::NoteOn {
                        note,
                        velocity: 0.8,
                    });
                    self.pressed_keys.insert(key);
                }
            }
            Message::KeyReleased(key) => {
                if self.pressed_keys.remove(&key)
                    && let Some(note) = key_to_midi_note(&key)
                    && let Some(sender) = &self.event_sender
                {
                    let _ = sender.try_send(EngineEvent::NoteOff { note });
                }
            }

            // Preset management
            Message::PresetNameChanged(name) => self.preset_name = name,
            Message::SavePreset => {
                return Task::perform(
                    preset_manager::save_preset_dialog(self.preset_name.clone(), self.params),
                    |_| Message::PanicPressed,
                );
            }
            Message::LoadPreset => {
                return Task::perform(preset_manager::load_preset_dialog(), |result| {
                    Message::PresetLoaded(Box::new(result))
                });
            }
            Message::PresetLoaded(result) => match *result {
                Ok(params) => self.params = params,
                Err(e) => eprintln!("Failed to load preset: {}", e),
            },
            Message::Randomize => {
                self.params = Self::randomize_params();
            }
        }

        // Write updated parameters to triple buffer
        if let Some(producer) = &mut self.param_producer {
            producer.write(self.params);
        }

        Task::none()
    }

    /// Generate random parameters for sound design exploration
    fn randomize_params() -> SynthParams {
        crate::randomize::randomize_synth_params(&mut rand::thread_rng())
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let title = text("DSynth").size(32);

        let keyboard_help = text("Keyboard: AWSEDFTGYHUJKOLP (C4-D#5) | ZXCVBNM (C3-B3)").size(14);

        let preset_controls = row![
            text("Preset:").width(60),
            text_input("Preset name", &self.preset_name)
                .on_input(Message::PresetNameChanged)
                .width(200),
            button("Save").on_press(Message::SavePreset).padding(10),
            button("Load").on_press(Message::LoadPreset).padding(10),
            button("🎲 Randomize")
                .on_press(Message::Randomize)
                .padding(10),
        ]
        .spacing(10)
        .padding(10);

        let osc1_section = sections::oscillator_controls(&self.params, 0, "Oscillator 1");
        let osc2_section = sections::oscillator_controls(&self.params, 1, "Oscillator 2");
        let osc3_section = sections::oscillator_controls(&self.params, 2, "Oscillator 3");

        let envelope_section = sections::envelope_controls(&self.params);
        let effects_section = sections::effects_controls(&self.params);

        let velocity_controls = column![
            text("--- Velocity Sensitivity ---").size(18),
            text("Amplitude:"),
            slider(
                0.0..=1.0,
                self.params.velocity.amp_sensitivity,
                Message::VelocityAmpChanged
            )
            .step(0.01),
            text(format!("{:.2}", self.params.velocity.amp_sensitivity)),
            text("Filter Cutoff:"),
            slider(
                0.0..=1.0,
                self.params.velocity.filter_sensitivity,
                Message::VelocityFilterChanged
            )
            .step(0.01),
            text(format!("{:.2}", self.params.velocity.filter_sensitivity)),
        ]
        .spacing(5)
        .padding(10);

        let master_controls = row![
            text("Master Gain:").width(100),
            slider(
                0.0..=1.0,
                self.params.master_gain,
                Message::MasterGainChanged
            )
            .step(0.01)
            .width(200),
            text(format!("{:.2}", self.params.master_gain)).width(50),
            button(if self.params.monophonic {
                "MONO [ON]"
            } else {
                "MONO [OFF]"
            })
            .on_press(Message::MonophonicToggled(!self.params.monophonic))
            .padding(10),
            button("PANIC").on_press(Message::PanicPressed).padding(10),
        ]
        .spacing(10)
        .padding(10);

        let content = column![
            row![title, preset_controls],
            row![master_controls],
            row![keyboard_help],
            scrollable(
                column![
                    row![osc1_section, osc2_section, osc3_section].spacing(20),
                    row![envelope_section, velocity_controls].padding(10),
                    row![effects_section].padding(10),
                ]
                .spacing(20)
            ),
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        event::listen_with(|event, _status, _id| match event {
            event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                Some(Message::KeyPressed(key))
            }
            event::Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                Some(Message::KeyReleased(key))
            }
            _ => None,
        })
    }
}

/// Run the standalone GUI application
pub fn run_gui(
    param_producer: Input<SynthParams>,
    event_sender: Sender<EngineEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    use iced::window;

    // Load window icon from embedded PNG
    let icon_bytes = include_bytes!("../../../assets/icon.png");
    println!("Loading icon... {} bytes", icon_bytes.len());

    let icon = match image::load_from_memory(icon_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            println!("Icon loaded: {}x{}", width, height);
            match window::icon::from_rgba(rgba.into_raw(), width, height) {
                Ok(icon) => {
                    println!("✓ Icon created successfully");
                    Some(icon)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create icon: {:?}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load icon image: {}", e);
            None
        }
    };

    let mut settings = iced::application("DSynth", SynthGui::update, SynthGui::view)
        .subscription(SynthGui::subscription);

    // Apply icon if loaded successfully
    if let Some(icon) = icon {
        println!("Applying icon to window settings...");
        settings = settings.window(window::Settings {
            icon: Some(icon),
            ..window::Settings::default()
        });
    } else {
        println!(
            "Note: Window icon not available (this is normal on macOS - use app bundle icon instead)"
        );
    }

    settings
        .run_with(move || {
            let gui = SynthGui::new(Some(param_producer), Some(event_sender));
            (gui, Task::none())
        })
        .map_err(|e| format!("GUI error: {:?}", e).into())
}
