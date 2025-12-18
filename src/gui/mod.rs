pub mod controls;

#[cfg(feature = "vst")]
#[path = "plugin_gui.rs"]
pub mod plugin_gui;

#[cfg(feature = "standalone")]
use crate::params::{DistortionType, FilterType, LFOWaveform, SynthParams, Waveform};

#[cfg(feature = "standalone")]
use crate::preset::Preset;

#[cfg(feature = "standalone")]
use crate::audio::output::EngineEvent;

#[cfg(feature = "standalone")]
use iced::{
    Element, Length, Task, event, keyboard,
    widget::{
        Column, button, column, container, pick_list, row, scrollable, slider, text, text_input,
    },
};
#[cfg(feature = "standalone")]
use std::collections::HashSet;

#[cfg(feature = "standalone")]
use crossbeam_channel::Sender;

#[cfg(feature = "standalone")]
use triple_buffer::Input;

#[cfg(feature = "standalone")]
pub struct SynthGui {
    params: SynthParams,
    param_producer: Option<Input<SynthParams>>,
    event_sender: Option<Sender<EngineEvent>>,
    pressed_keys: HashSet<keyboard::Key>,
    preset_name: String,
}

/// Hierarchical message types to reduce boilerplate
#[cfg(feature = "standalone")]
#[derive(Debug, Clone)]
pub enum OscillatorMessage {
    WaveformChanged(Waveform),
    PitchChanged(f32),
    DetuneChanged(f32),
    GainChanged(f32),
    PanChanged(f32),
    UnisonChanged(usize),
    UnisonDetuneChanged(f32),
    PhaseChanged(f32),
    ShapeChanged(f32),
    SoloToggled(bool),
}

#[cfg(feature = "standalone")]
#[derive(Debug, Clone)]
pub enum FilterMessage {
    TypeChanged(FilterType),
    CutoffChanged(f32),
    ResonanceChanged(f32),
    BandwidthChanged(f32),
    KeyTrackingChanged(f32),
}

#[cfg(feature = "standalone")]
#[derive(Debug, Clone)]
pub enum LFOMessage {
    WaveformChanged(LFOWaveform),
    RateChanged(f32),
    DepthChanged(f32),
    FilterAmountChanged(f32),
}

#[cfg(feature = "standalone")]
#[derive(Debug, Clone)]
pub enum ReverbMessage {
    RoomSizeChanged(f32),
    DampingChanged(f32),
    WetChanged(f32),
    DryChanged(f32),
    WidthChanged(f32),
}

#[cfg(feature = "standalone")]
#[derive(Debug, Clone)]
pub enum DelayMessage {
    TimeChanged(f32),
    FeedbackChanged(f32),
    WetChanged(f32),
    DryChanged(f32),
}

#[cfg(feature = "standalone")]
#[derive(Debug, Clone)]
pub enum ChorusMessage {
    RateChanged(f32),
    DepthChanged(f32),
    MixChanged(f32),
}

#[cfg(feature = "standalone")]
#[derive(Debug, Clone)]
pub enum DistortionMessage {
    DriveChanged(f32),
    MixChanged(f32),
    TypeChanged(DistortionType),
}

#[cfg(feature = "standalone")]
#[derive(Debug, Clone)]
pub enum Message {
    // Indexed parameter groups
    Oscillator(usize, OscillatorMessage),
    Filter(usize, FilterMessage),
    LFO(usize, LFOMessage),

    // Effects
    Reverb(ReverbMessage),
    Delay(DelayMessage),
    Chorus(ChorusMessage),
    Distortion(DistortionMessage),

    // Velocity Sensitivity
    VelocityAmpChanged(f32),
    VelocityFilterChanged(f32),

    // Master
    MasterGainChanged(f32),
    MonophonicToggled(bool),
    PanicPressed,

    // Keyboard events
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),

    // Preset management
    PresetNameChanged(String),
    SavePreset,
    LoadPreset,
    PresetLoaded(Box<Result<SynthParams, String>>),
    Randomize,
}

#[cfg(feature = "standalone")]
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

    /// Map keyboard key to MIDI note number
    /// Using two rows: AWSEDFTGYHUJK for white keys, and QRTYUOP for black keys
    fn key_to_midi_note(key: &keyboard::Key) -> Option<u8> {
        use keyboard::Key;

        match key {
            // Bottom row (white keys) - C to B
            Key::Character(c) => match c.as_str() {
                "a" => Some(60), // C4
                "w" => Some(61), // C#4
                "s" => Some(62), // D4
                "e" => Some(63), // D#4
                "d" => Some(64), // E4
                "f" => Some(65), // F4
                "t" => Some(66), // F#4
                "g" => Some(67), // G4
                "y" => Some(68), // G#4
                "h" => Some(69), // A4
                "u" => Some(70), // A#4
                "j" => Some(71), // B4
                "k" => Some(72), // C5
                "o" => Some(73), // C#5
                "l" => Some(74), // D5
                "p" => Some(75), // D#5

                // Top row (one octave up)
                "z" => Some(48), // C3
                "x" => Some(50), // D3
                "c" => Some(52), // E3
                "v" => Some(53), // F3
                "b" => Some(55), // G3
                "n" => Some(57), // A3
                "m" => Some(59), // B3

                _ => None,
            },
            _ => None,
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
            Message::Reverb(msg) => {
                match msg {
                    ReverbMessage::RoomSizeChanged(v) => self.params.effects.reverb.room_size = v,
                    ReverbMessage::DampingChanged(v) => self.params.effects.reverb.damping = v,
                    ReverbMessage::WetChanged(v) => self.params.effects.reverb.wet = v,
                    ReverbMessage::DryChanged(v) => self.params.effects.reverb.dry = v,
                    ReverbMessage::WidthChanged(v) => self.params.effects.reverb.width = v,
                }
            }
            Message::Delay(msg) => {
                match msg {
                    DelayMessage::TimeChanged(v) => self.params.effects.delay.time_ms = v,
                    DelayMessage::FeedbackChanged(v) => self.params.effects.delay.feedback = v,
                    DelayMessage::WetChanged(v) => self.params.effects.delay.wet = v,
                    DelayMessage::DryChanged(v) => self.params.effects.delay.dry = v,
                }
            }
            Message::Chorus(msg) => {
                match msg {
                    ChorusMessage::RateChanged(v) => self.params.effects.chorus.rate = v,
                    ChorusMessage::DepthChanged(v) => self.params.effects.chorus.depth = v,
                    ChorusMessage::MixChanged(v) => self.params.effects.chorus.mix = v,
                }
            }
            Message::Distortion(msg) => {
                match msg {
                    DistortionMessage::DriveChanged(v) => self.params.effects.distortion.drive = v,
                    DistortionMessage::MixChanged(v) => self.params.effects.distortion.mix = v,
                    DistortionMessage::TypeChanged(t) => self.params.effects.distortion.dist_type = t,
                }
            }

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
                    && let Some(note) = Self::key_to_midi_note(&key)
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
                    && let Some(note) = Self::key_to_midi_note(&key)
                    && let Some(sender) = &self.event_sender
                {
                    let _ = sender.try_send(EngineEvent::NoteOff { note });
                }
            }

            // Preset management
            Message::PresetNameChanged(name) => self.preset_name = name,
            Message::SavePreset => {
                return Task::perform(
                    Self::save_preset_dialog(self.preset_name.clone(), self.params),
                    |_| Message::PanicPressed,
                );
            }
            Message::LoadPreset => {
                return Task::perform(Self::load_preset_dialog(), |result| {
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
        let title = text("DSynth - Digital Synthesizer").size(32);

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

        let osc1_section = self.oscillator_controls(0, "Oscillator 1");
        let osc2_section = self.oscillator_controls(1, "Oscillator 2");
        let osc3_section = self.oscillator_controls(2, "Oscillator 3");

        let effects_section = self.effects_controls();

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
            row![title, keyboard_help, preset_controls],
            row![master_controls, velocity_controls],
            scrollable(
                column![
                    row![osc1_section, osc2_section, osc3_section].spacing(20),
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

    fn oscillator_controls<'a>(&'a self, index: usize, label: &'a str) -> Element<'a, Message> {
        let osc = &self.params.oscillators[index];
        let filter = &self.params.filters[index];
        let lfo = &self.params.lfos[index];

        let waveforms = vec![
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
            Waveform::Pulse,
        ];

        let filter_types = vec![
            FilterType::Lowpass,
            FilterType::Highpass,
            FilterType::Bandpass,
        ];

        let lfo_waveforms = vec![
            LFOWaveform::Sine,
            LFOWaveform::Triangle,
            LFOWaveform::Square,
            LFOWaveform::Saw,
        ];

        Column::new()
            .push(text(label).size(20))
            // Solo button
            .push(
                button(if osc.solo { "SOLO [ON]" } else { "SOLO [OFF]" }).on_press(
                    Message::Oscillator(index, OscillatorMessage::SoloToggled(!osc.solo)),
                ),
            )
            // Oscillator controls
            .push(text("Waveform:"))
            .push(pick_list(waveforms, Some(osc.waveform), move |w| {
                Message::Oscillator(index, OscillatorMessage::WaveformChanged(w))
            }))
            .push(text("Pitch (semitones):"))
            .push(
                slider(-24.0..=24.0, osc.pitch, move |p| {
                    Message::Oscillator(index, OscillatorMessage::PitchChanged(p))
                })
                .step(1.0),
            )
            .push(text(format!("{:.0}", osc.pitch)))
            .push(text("Detune (cents):"))
            .push(
                slider(-50.0..=50.0, osc.detune, move |d| {
                    Message::Oscillator(index, OscillatorMessage::DetuneChanged(d))
                })
                .step(1.0),
            )
            .push(text(format!("{:.0}", osc.detune)))
            .push(text("Gain:"))
            .push(
                slider(0.0..=1.0, osc.gain, move |g| {
                    Message::Oscillator(index, OscillatorMessage::GainChanged(g))
                })
                .step(0.01),
            )
            .push(text(format!("{:.2}", osc.gain)))
            .push(text("Pan:"))
            .push(
                slider(-1.0..=1.0, osc.pan, move |p| {
                    Message::Oscillator(index, OscillatorMessage::PanChanged(p))
                })
                .step(0.01),
            )
            .push(text(format!("{:.2}", osc.pan)))
            .push(text("Unison:"))
            .push(
                slider(1.0..=7.0, osc.unison as f32, move |v| {
                    Message::Oscillator(index, OscillatorMessage::UnisonChanged(v as usize))
                })
                .step(1.0),
            )
            .push(text(format!("{}", osc.unison)))
            .push(text("Unison Detune (cents):"))
            .push(
                slider(0.0..=50.0, osc.unison_detune, move |d| {
                    Message::Oscillator(index, OscillatorMessage::UnisonDetuneChanged(d))
                })
                .step(1.0),
            )
            .push(text(format!("{:.0}", osc.unison_detune)))
            .push(text("Phase:"))
            .push(
                slider(0.0..=1.0, osc.phase, move |p| {
                    Message::Oscillator(index, OscillatorMessage::PhaseChanged(p))
                })
                .step(0.01),
            )
            .push(text(format!("{:.2}", osc.phase)))
            .push(text("Shape:"))
            .push(
                slider(-1.0..=1.0, osc.shape, move |s| {
                    Message::Oscillator(index, OscillatorMessage::ShapeChanged(s))
                })
                .step(0.01),
            )
            .push(text(format!("{:.2}", osc.shape)))
            // Filter controls
            .push(text("--- Filter ---").size(18))
            .push(text("Type:"))
            .push(pick_list(
                filter_types,
                Some(filter.filter_type),
                move |t| Message::Filter(index, FilterMessage::TypeChanged(t)),
            ))
            .push(text("Cutoff (Hz):"))
            .push(
                slider(20.0..=20000.0, filter.cutoff, move |c| {
                    Message::Filter(index, FilterMessage::CutoffChanged(c))
                })
                .step(10.0),
            )
            .push(text(format!("{:.0}", filter.cutoff)))
            .push(text("Resonance:"))
            .push(
                slider(0.5..=10.0, filter.resonance, move |r| {
                    Message::Filter(index, FilterMessage::ResonanceChanged(r))
                })
                .step(0.1),
            )
            .push(text(format!("{:.1}", filter.resonance)))
            .push(text("Bandwidth (octaves):"))
            .push(
                slider(0.1..=4.0, filter.bandwidth, move |b| {
                    Message::Filter(index, FilterMessage::BandwidthChanged(b))
                })
                .step(0.1),
            )
            .push(text(format!("{:.1}", filter.bandwidth)))
            .push(text("Key Tracking:"))
            .push(
                slider(0.0..=1.0, filter.key_tracking, move |k| {
                    Message::Filter(index, FilterMessage::KeyTrackingChanged(k))
                })
                .step(0.01),
            )
            .push(text(format!("{:.2}", filter.key_tracking)))
            // LFO controls
            .push(text("--- LFO ---").size(18))
            .push(text("Waveform:"))
            .push(pick_list(lfo_waveforms, Some(lfo.waveform), move |w| {
                Message::LFO(index, LFOMessage::WaveformChanged(w))
            }))
            .push(text("Rate (Hz):"))
            .push(
                slider(0.01..=20.0, lfo.rate, move |r| {
                    Message::LFO(index, LFOMessage::RateChanged(r))
                })
                .step(0.1),
            )
            .push(text(format!("{:.2}", lfo.rate)))
            .push(text("Depth:"))
            .push(
                slider(0.0..=1.0, lfo.depth, move |d| {
                    Message::LFO(index, LFOMessage::DepthChanged(d))
                })
                .step(0.01),
            )
            .push(text(format!("{:.2}", lfo.depth)))
            .push(text("Filter Amount (Hz):"))
            .push(
                slider(0.0..=5000.0, lfo.filter_amount, move |a| {
                    Message::LFO(index, LFOMessage::FilterAmountChanged(a))
                })
                .step(50.0),
            )
            .push(text(format!("{:.0}", lfo.filter_amount)))
            .spacing(5)
            .padding(10)
            .into()
    }

    fn effects_controls<'a>(&'a self) -> Element<'a, Message> {
        let effects = &self.params.effects;

        let distortion_types = vec![
            DistortionType::Tanh,
            DistortionType::SoftClip,
            DistortionType::HardClip,
            DistortionType::Cubic,
        ];

        // Distortion controls
        let distortion_section = column![
            text("DISTORTION").size(18),
            text("Type:"),
            pick_list(
                distortion_types,
                Some(effects.distortion.dist_type),
                |t| Message::Distortion(DistortionMessage::TypeChanged(t))
            ),
            text("Drive:"),
            slider(0.0..=1.0, effects.distortion.drive, |v| {
                Message::Distortion(DistortionMessage::DriveChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.distortion.drive)),
            text("Mix:"),
            slider(0.0..=1.0, effects.distortion.mix, |v| {
                Message::Distortion(DistortionMessage::MixChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.distortion.mix)),
        ]
        .spacing(5)
        .padding(10);

        // Chorus controls
        let chorus_section = column![
            text("CHORUS").size(18),
            text("Rate (Hz):"),
            slider(0.1..=5.0, effects.chorus.rate, |v| {
                Message::Chorus(ChorusMessage::RateChanged(v))
            })
            .step(0.1),
            text(format!("{:.1}", effects.chorus.rate)),
            text("Depth:"),
            slider(0.0..=1.0, effects.chorus.depth, |v| {
                Message::Chorus(ChorusMessage::DepthChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.chorus.depth)),
            text("Mix:"),
            slider(0.0..=1.0, effects.chorus.mix, |v| {
                Message::Chorus(ChorusMessage::MixChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.chorus.mix)),
        ]
        .spacing(5)
        .padding(10);

        // Delay controls
        let delay_section = column![
            text("DELAY").size(18),
            text("Time (ms):"),
            slider(1.0..=2000.0, effects.delay.time_ms, |v| {
                Message::Delay(DelayMessage::TimeChanged(v))
            })
            .step(1.0),
            text(format!("{:.0}", effects.delay.time_ms)),
            text("Feedback:"),
            slider(0.0..=0.95, effects.delay.feedback, |v| {
                Message::Delay(DelayMessage::FeedbackChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.delay.feedback)),
            text("Wet:"),
            slider(0.0..=1.0, effects.delay.wet, |v| {
                Message::Delay(DelayMessage::WetChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.delay.wet)),
            text("Dry:"),
            slider(0.0..=1.0, effects.delay.dry, |v| {
                Message::Delay(DelayMessage::DryChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.delay.dry)),
        ]
        .spacing(5)
        .padding(10);

        // Reverb controls
        let reverb_section = column![
            text("REVERB").size(18),
            text("Room Size:"),
            slider(0.0..=1.0, effects.reverb.room_size, |v| {
                Message::Reverb(ReverbMessage::RoomSizeChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.reverb.room_size)),
            text("Damping:"),
            slider(0.0..=1.0, effects.reverb.damping, |v| {
                Message::Reverb(ReverbMessage::DampingChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.reverb.damping)),
            text("Wet:"),
            slider(0.0..=1.0, effects.reverb.wet, |v| {
                Message::Reverb(ReverbMessage::WetChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.reverb.wet)),
            text("Dry:"),
            slider(0.0..=1.0, effects.reverb.dry, |v| {
                Message::Reverb(ReverbMessage::DryChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.reverb.dry)),
            text("Width:"),
            slider(0.0..=1.0, effects.reverb.width, |v| {
                Message::Reverb(ReverbMessage::WidthChanged(v))
            })
            .step(0.01),
            text(format!("{:.2}", effects.reverb.width)),
        ]
        .spacing(5)
        .padding(10);

        // Combine all effects in a row
        column![
            text("=== EFFECTS CHAIN ===").size(20),
            text("Signal Flow: Distortion → Chorus → Delay → Reverb").size(14),
            row![
                distortion_section,
                chorus_section,
                delay_section,
                reverb_section
            ]
            .spacing(20)
        ]
        .spacing(10)
        .padding(10)
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

    async fn save_preset_dialog(name: String, params: SynthParams) -> Result<(), String> {
        use rfd::AsyncFileDialog;

        let file = AsyncFileDialog::new()
            .set_title("Save Preset")
            .set_file_name(format!("{}.json", name))
            .add_filter("JSON", &["json"])
            .save_file()
            .await;

        if let Some(file) = file {
            let preset = Preset::new(name, params);
            preset
                .save(file.path())
                .map_err(|e| format!("Failed to save preset: {}", e))?;
        }

        Ok(())
    }

    async fn load_preset_dialog() -> Result<SynthParams, String> {
        use rfd::AsyncFileDialog;

        let file = AsyncFileDialog::new()
            .set_title("Load Preset")
            .add_filter("JSON", &["json"])
            .pick_file()
            .await;

        if let Some(file) = file {
            let preset =
                Preset::load(file.path()).map_err(|e| format!("Failed to load preset: {}", e))?;
            return Ok(preset.params);
        }

        Err("No file selected".to_string())
    }
}

#[cfg(feature = "standalone")]
pub fn run_gui(
    param_producer: Input<SynthParams>,
    event_sender: Sender<EngineEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    use iced::window;

    // Load window icon from embedded PNG
    let icon_bytes = include_bytes!("../../assets/icon.png");
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
