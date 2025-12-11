pub mod controls;

use crate::audio::engine::SynthEngine;
use crate::params::{FilterType, SynthParams, Waveform};
use iced::{
    Element, Length, Task, event, keyboard,
    widget::{Column, button, column, container, pick_list, row, scrollable, slider, text},
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use triple_buffer::Input;

pub struct SynthGui {
    params: SynthParams,
    param_producer: Option<Input<SynthParams>>,
    engine: Option<Arc<Mutex<SynthEngine>>>,
    pressed_keys: HashSet<keyboard::Key>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Oscillator 1
    Osc1WaveformChanged(Waveform),
    Osc1PitchChanged(f32),
    Osc1DetuneChanged(f32),
    Osc1GainChanged(f32),
    Osc1PanChanged(f32),
    Osc1UnisonChanged(usize),
    Osc1UnisonDetuneChanged(f32),
    Osc1PhaseChanged(f32),

    // Oscillator 2
    Osc2WaveformChanged(Waveform),
    Osc2PitchChanged(f32),
    Osc2DetuneChanged(f32),
    Osc2GainChanged(f32),
    Osc2PanChanged(f32),
    Osc2UnisonChanged(usize),
    Osc2UnisonDetuneChanged(f32),
    Osc2PhaseChanged(f32),

    // Oscillator 3
    Osc3WaveformChanged(Waveform),
    Osc3PitchChanged(f32),
    Osc3DetuneChanged(f32),
    Osc3GainChanged(f32),
    Osc3PanChanged(f32),
    Osc3UnisonChanged(usize),
    Osc3UnisonDetuneChanged(f32),
    Osc3PhaseChanged(f32),

    // Filter 1
    Filter1TypeChanged(FilterType),
    Filter1CutoffChanged(f32),
    Filter1ResonanceChanged(f32),
    Filter1DriveChanged(f32),

    // Filter 2
    Filter2TypeChanged(FilterType),
    Filter2CutoffChanged(f32),
    Filter2ResonanceChanged(f32),
    Filter2DriveChanged(f32),

    // Filter 3
    Filter3TypeChanged(FilterType),
    Filter3CutoffChanged(f32),
    Filter3ResonanceChanged(f32),
    Filter3DriveChanged(f32),

    // Master
    MasterGainChanged(f32),
    PanicPressed,

    // Keyboard events
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
}

impl SynthGui {
    pub fn new(
        param_producer: Option<Input<SynthParams>>,
        engine: Option<Arc<Mutex<SynthEngine>>>,
    ) -> Self {
        Self {
            params: SynthParams::default(),
            param_producer,
            engine,
            pressed_keys: HashSet::new(),
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
            // Oscillator 1
            Message::Osc1WaveformChanged(w) => self.params.oscillators[0].waveform = w,
            Message::Osc1PitchChanged(p) => self.params.oscillators[0].pitch = p,
            Message::Osc1DetuneChanged(d) => self.params.oscillators[0].detune = d,
            Message::Osc1GainChanged(g) => self.params.oscillators[0].gain = g,
            Message::Osc1PanChanged(p) => self.params.oscillators[0].pan = p,
            Message::Osc1UnisonChanged(u) => self.params.oscillators[0].unison = u,
            Message::Osc1UnisonDetuneChanged(d) => self.params.oscillators[0].unison_detune = d,
            Message::Osc1PhaseChanged(p) => self.params.oscillators[0].phase = p,

            // Oscillator 2
            Message::Osc2WaveformChanged(w) => self.params.oscillators[1].waveform = w,
            Message::Osc2PitchChanged(p) => self.params.oscillators[1].pitch = p,
            Message::Osc2DetuneChanged(d) => self.params.oscillators[1].detune = d,
            Message::Osc2GainChanged(g) => self.params.oscillators[1].gain = g,
            Message::Osc2PanChanged(p) => self.params.oscillators[1].pan = p,
            Message::Osc2UnisonChanged(u) => self.params.oscillators[1].unison = u,
            Message::Osc2UnisonDetuneChanged(d) => self.params.oscillators[1].unison_detune = d,
            Message::Osc2PhaseChanged(p) => self.params.oscillators[1].phase = p,

            // Oscillator 3
            Message::Osc3WaveformChanged(w) => self.params.oscillators[2].waveform = w,
            Message::Osc3PitchChanged(p) => self.params.oscillators[2].pitch = p,
            Message::Osc3DetuneChanged(d) => self.params.oscillators[2].detune = d,
            Message::Osc3GainChanged(g) => self.params.oscillators[2].gain = g,
            Message::Osc3PanChanged(p) => self.params.oscillators[2].pan = p,
            Message::Osc3UnisonChanged(u) => self.params.oscillators[2].unison = u,
            Message::Osc3UnisonDetuneChanged(d) => self.params.oscillators[2].unison_detune = d,
            Message::Osc3PhaseChanged(p) => self.params.oscillators[2].phase = p,

            // Filter 1
            Message::Filter1TypeChanged(t) => self.params.filters[0].filter_type = t,
            Message::Filter1CutoffChanged(c) => self.params.filters[0].cutoff = c,
            Message::Filter1ResonanceChanged(r) => self.params.filters[0].resonance = r,
            Message::Filter1DriveChanged(d) => self.params.filters[0].drive = d,

            // Filter 2
            Message::Filter2TypeChanged(t) => self.params.filters[1].filter_type = t,
            Message::Filter2CutoffChanged(c) => self.params.filters[1].cutoff = c,
            Message::Filter2ResonanceChanged(r) => self.params.filters[1].resonance = r,
            Message::Filter2DriveChanged(d) => self.params.filters[1].drive = d,

            // Filter 3
            Message::Filter3TypeChanged(t) => self.params.filters[2].filter_type = t,
            Message::Filter3CutoffChanged(c) => self.params.filters[2].cutoff = c,
            Message::Filter3ResonanceChanged(r) => self.params.filters[2].resonance = r,
            Message::Filter3DriveChanged(d) => self.params.filters[2].drive = d,

            // Master
            Message::MasterGainChanged(g) => self.params.master_gain = g,
            Message::PanicPressed => {
                // Send all notes off to engine
                if let Some(engine) = &self.engine {
                    if let Ok(mut eng) = engine.lock() {
                        eng.all_notes_off();
                    }
                }
                self.pressed_keys.clear();
            }

            // Keyboard events
            Message::KeyPressed(key) => {
                if !self.pressed_keys.contains(&key) {
                    if let Some(note) = Self::key_to_midi_note(&key) {
                        if let Some(engine) = &self.engine {
                            if let Ok(mut eng) = engine.lock() {
                                eng.note_on(note, 0.8); // Fixed velocity
                            }
                        }
                        self.pressed_keys.insert(key);
                    }
                }
            }
            Message::KeyReleased(key) => {
                if self.pressed_keys.remove(&key) {
                    if let Some(note) = Self::key_to_midi_note(&key) {
                        if let Some(engine) = &self.engine {
                            if let Ok(mut eng) = engine.lock() {
                                eng.note_off(note);
                            }
                        }
                    }
                }
            }
        }

        // Write updated parameters to triple buffer
        if let Some(producer) = &mut self.param_producer {
            producer.write(self.params);
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let title = text("DSynth - Digital Synthesizer").size(32);

        let keyboard_help = text("Keyboard: AWSEDFTGYHUJKOLP (C4-D#5) | ZXCVBNM (C3-B3)").size(14);

        let osc1_section = self.oscillator_controls(0, "Oscillator 1");
        let osc2_section = self.oscillator_controls(1, "Oscillator 2");
        let osc3_section = self.oscillator_controls(2, "Oscillator 3");

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
            button("PANIC").on_press(Message::PanicPressed).padding(10),
        ]
        .spacing(10)
        .padding(10);

        let content = column![
            title,
            keyboard_help,
            scrollable(row![osc1_section, osc2_section, osc3_section].spacing(20)),
            master_controls,
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

        let waveforms = vec![
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ];

        let filter_types = vec![
            FilterType::Lowpass,
            FilterType::Highpass,
            FilterType::Bandpass,
        ];

        let (
            wave_msg,
            pitch_msg,
            detune_msg,
            gain_msg,
            pan_msg,
            unison_msg,
            unison_detune_msg,
            phase_msg,
            ftype_msg,
            cutoff_msg,
            res_msg,
            drive_msg,
        ) = match index {
            0 => (
                Message::Osc1WaveformChanged as fn(Waveform) -> Message,
                Message::Osc1PitchChanged as fn(f32) -> Message,
                Message::Osc1DetuneChanged as fn(f32) -> Message,
                Message::Osc1GainChanged as fn(f32) -> Message,
                Message::Osc1PanChanged as fn(f32) -> Message,
                Message::Osc1UnisonChanged as fn(usize) -> Message,
                Message::Osc1UnisonDetuneChanged as fn(f32) -> Message,
                Message::Osc1PhaseChanged as fn(f32) -> Message,
                Message::Filter1TypeChanged as fn(FilterType) -> Message,
                Message::Filter1CutoffChanged as fn(f32) -> Message,
                Message::Filter1ResonanceChanged as fn(f32) -> Message,
                Message::Filter1DriveChanged as fn(f32) -> Message,
            ),
            1 => (
                Message::Osc2WaveformChanged as fn(Waveform) -> Message,
                Message::Osc2PitchChanged as fn(f32) -> Message,
                Message::Osc2DetuneChanged as fn(f32) -> Message,
                Message::Osc2GainChanged as fn(f32) -> Message,
                Message::Osc2PanChanged as fn(f32) -> Message,
                Message::Osc2UnisonChanged as fn(usize) -> Message,
                Message::Osc2UnisonDetuneChanged as fn(f32) -> Message,
                Message::Osc2PhaseChanged as fn(f32) -> Message,
                Message::Filter2TypeChanged as fn(FilterType) -> Message,
                Message::Filter2CutoffChanged as fn(f32) -> Message,
                Message::Filter2ResonanceChanged as fn(f32) -> Message,
                Message::Filter2DriveChanged as fn(f32) -> Message,
            ),
            _ => (
                Message::Osc3WaveformChanged as fn(Waveform) -> Message,
                Message::Osc3PitchChanged as fn(f32) -> Message,
                Message::Osc3DetuneChanged as fn(f32) -> Message,
                Message::Osc3GainChanged as fn(f32) -> Message,
                Message::Osc3PanChanged as fn(f32) -> Message,
                Message::Osc3UnisonChanged as fn(usize) -> Message,
                Message::Osc3UnisonDetuneChanged as fn(f32) -> Message,
                Message::Osc3PhaseChanged as fn(f32) -> Message,
                Message::Filter3TypeChanged as fn(FilterType) -> Message,
                Message::Filter3CutoffChanged as fn(f32) -> Message,
                Message::Filter3ResonanceChanged as fn(f32) -> Message,
                Message::Filter3DriveChanged as fn(f32) -> Message,
            ),
        };

        Column::new()
            .push(text(label).size(20))
            .push(text("Waveform:"))
            .push(pick_list(waveforms, Some(osc.waveform), wave_msg))
            .push(text("Pitch (semitones):"))
            .push(slider(-24.0..=24.0, osc.pitch, pitch_msg).step(1.0))
            .push(text(format!("{:.0}", osc.pitch)))
            .push(text("Detune (cents):"))
            .push(slider(-50.0..=50.0, osc.detune, detune_msg).step(1.0))
            .push(text(format!("{:.0}", osc.detune)))
            .push(text("Gain:"))
            .push(slider(0.0..=1.0, osc.gain, gain_msg).step(0.01))
            .push(text(format!("{:.2}", osc.gain)))
            .push(text("Pan:"))
            .push(slider(-1.0..=1.0, osc.pan, pan_msg).step(0.01))
            .push(text(format!("{:.2}", osc.pan)))
            .push(text("Unison:"))
            .push(
                slider(1.0..=7.0, osc.unison as f32, move |v| {
                    unison_msg(v as usize)
                })
                .step(1.0),
            )
            .push(text(format!("{}", osc.unison)))
            .push(text("Unison Detune (cents):"))
            .push(slider(0.0..=50.0, osc.unison_detune, unison_detune_msg).step(1.0))
            .push(text(format!("{:.0}", osc.unison_detune)))
            .push(text("Phase:"))
            .push(slider(0.0..=1.0, osc.phase, phase_msg).step(0.01))
            .push(text(format!("{:.2}", osc.phase)))
            .push(text("--- Filter ---").size(18))
            .push(text("Type:"))
            .push(pick_list(filter_types, Some(filter.filter_type), ftype_msg))
            .push(text("Cutoff (Hz):"))
            .push(slider(20.0..=20000.0, filter.cutoff, cutoff_msg).step(10.0))
            .push(text(format!("{:.0}", filter.cutoff)))
            .push(text("Resonance:"))
            .push(slider(0.5..=10.0, filter.resonance, res_msg).step(0.1))
            .push(text(format!("{:.1}", filter.resonance)))
            .push(text("Drive:"))
            .push(slider(1.0..=10.0, filter.drive, drive_msg).step(0.1))
            .push(text(format!("{:.1}", filter.drive)))
            .spacing(5)
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
}

pub fn run_gui(
    param_producer: Input<SynthParams>,
    engine: Arc<Mutex<SynthEngine>>,
) -> Result<(), Box<dyn std::error::Error>> {
    iced::application("DSynth", SynthGui::update, SynthGui::view)
        .subscription(SynthGui::subscription)
        .run_with(move || {
            let gui = SynthGui::new(Some(param_producer), Some(engine));
            (gui, Task::none())
        })
        .map_err(|e| format!("GUI error: {:?}", e).into())
}
