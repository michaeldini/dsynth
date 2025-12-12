pub mod controls;

use crate::audio::engine::SynthEngine;
use crate::params::{FilterType, LFOWaveform, SynthParams, Waveform};
use crate::preset::Preset;
use iced::{
    Element, Length, Task, event, keyboard,
    widget::{
        Column, button, column, container, pick_list, row, scrollable, slider, text, text_input,
    },
};
use rand::Rng;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use triple_buffer::Input;

pub struct SynthGui {
    params: SynthParams,
    param_producer: Option<Input<SynthParams>>,
    engine: Option<Arc<Mutex<SynthEngine>>>,
    pressed_keys: HashSet<keyboard::Key>,
    preset_name: String,
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
    Osc1ShapeChanged(f32),

    // Oscillator 2
    Osc2WaveformChanged(Waveform),
    Osc2PitchChanged(f32),
    Osc2DetuneChanged(f32),
    Osc2GainChanged(f32),
    Osc2PanChanged(f32),
    Osc2UnisonChanged(usize),
    Osc2UnisonDetuneChanged(f32),
    Osc2PhaseChanged(f32),
    Osc2ShapeChanged(f32),

    // Oscillator 3
    Osc3WaveformChanged(Waveform),
    Osc3PitchChanged(f32),
    Osc3DetuneChanged(f32),
    Osc3GainChanged(f32),
    Osc3PanChanged(f32),
    Osc3UnisonChanged(usize),
    Osc3UnisonDetuneChanged(f32),
    Osc3PhaseChanged(f32),
    Osc3ShapeChanged(f32),

    // Filter 1
    Filter1TypeChanged(FilterType),
    Filter1CutoffChanged(f32),
    Filter1ResonanceChanged(f32),
    Filter1DriveChanged(f32),
    Filter1KeyTrackingChanged(f32),

    // Filter 2
    Filter2TypeChanged(FilterType),
    Filter2CutoffChanged(f32),
    Filter2ResonanceChanged(f32),
    Filter2DriveChanged(f32),
    Filter2KeyTrackingChanged(f32),

    // Filter 3
    Filter3TypeChanged(FilterType),
    Filter3CutoffChanged(f32),
    Filter3ResonanceChanged(f32),
    Filter3DriveChanged(f32),
    Filter3KeyTrackingChanged(f32),

    // Filter Envelope 1
    FilterEnv1AttackChanged(f32),
    FilterEnv1DecayChanged(f32),
    FilterEnv1SustainChanged(f32),
    FilterEnv1ReleaseChanged(f32),
    FilterEnv1AmountChanged(f32),

    // Filter Envelope 2
    FilterEnv2AttackChanged(f32),
    FilterEnv2DecayChanged(f32),
    FilterEnv2SustainChanged(f32),
    FilterEnv2ReleaseChanged(f32),
    FilterEnv2AmountChanged(f32),

    // Filter Envelope 3
    FilterEnv3AttackChanged(f32),
    FilterEnv3DecayChanged(f32),
    FilterEnv3SustainChanged(f32),
    FilterEnv3ReleaseChanged(f32),
    FilterEnv3AmountChanged(f32),

    // LFO 1
    LFO1WaveformChanged(LFOWaveform),
    LFO1RateChanged(f32),
    LFO1DepthChanged(f32),
    LFO1FilterAmountChanged(f32),

    // LFO 2
    LFO2WaveformChanged(LFOWaveform),
    LFO2RateChanged(f32),
    LFO2DepthChanged(f32),
    LFO2FilterAmountChanged(f32),

    // LFO 3
    LFO3WaveformChanged(LFOWaveform),
    LFO3RateChanged(f32),
    LFO3DepthChanged(f32),
    LFO3FilterAmountChanged(f32),

    // Velocity Sensitivity
    VelocityAmpChanged(f32),
    VelocityFilterChanged(f32),
    VelocityFilterEnvChanged(f32),

    // Master
    MasterGainChanged(f32),
    PanicPressed,

    // Keyboard events
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),

    // Preset management
    PresetNameChanged(String),
    SavePreset,
    LoadPreset,
    PresetLoaded(Result<SynthParams, String>),
    Randomize,
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
            // Oscillator 1
            Message::Osc1WaveformChanged(w) => self.params.oscillators[0].waveform = w,
            Message::Osc1PitchChanged(p) => self.params.oscillators[0].pitch = p,
            Message::Osc1DetuneChanged(d) => self.params.oscillators[0].detune = d,
            Message::Osc1GainChanged(g) => self.params.oscillators[0].gain = g,
            Message::Osc1PanChanged(p) => self.params.oscillators[0].pan = p,
            Message::Osc1UnisonChanged(u) => self.params.oscillators[0].unison = u,
            Message::Osc1UnisonDetuneChanged(d) => self.params.oscillators[0].unison_detune = d,
            Message::Osc1PhaseChanged(p) => self.params.oscillators[0].phase = p,
            Message::Osc1ShapeChanged(s) => self.params.oscillators[0].shape = s,

            // Oscillator 2
            Message::Osc2WaveformChanged(w) => self.params.oscillators[1].waveform = w,
            Message::Osc2PitchChanged(p) => self.params.oscillators[1].pitch = p,
            Message::Osc2DetuneChanged(d) => self.params.oscillators[1].detune = d,
            Message::Osc2GainChanged(g) => self.params.oscillators[1].gain = g,
            Message::Osc2PanChanged(p) => self.params.oscillators[1].pan = p,
            Message::Osc2UnisonChanged(u) => self.params.oscillators[1].unison = u,
            Message::Osc2UnisonDetuneChanged(d) => self.params.oscillators[1].unison_detune = d,
            Message::Osc2PhaseChanged(p) => self.params.oscillators[1].phase = p,
            Message::Osc2ShapeChanged(s) => self.params.oscillators[1].shape = s,

            // Oscillator 3
            Message::Osc3WaveformChanged(w) => self.params.oscillators[2].waveform = w,
            Message::Osc3PitchChanged(p) => self.params.oscillators[2].pitch = p,
            Message::Osc3DetuneChanged(d) => self.params.oscillators[2].detune = d,
            Message::Osc3GainChanged(g) => self.params.oscillators[2].gain = g,
            Message::Osc3PanChanged(p) => self.params.oscillators[2].pan = p,
            Message::Osc3UnisonChanged(u) => self.params.oscillators[2].unison = u,
            Message::Osc3UnisonDetuneChanged(d) => self.params.oscillators[2].unison_detune = d,
            Message::Osc3PhaseChanged(p) => self.params.oscillators[2].phase = p,
            Message::Osc3ShapeChanged(s) => self.params.oscillators[2].shape = s,

            // Filter 1
            Message::Filter1TypeChanged(t) => self.params.filters[0].filter_type = t,
            Message::Filter1CutoffChanged(c) => self.params.filters[0].cutoff = c,
            Message::Filter1ResonanceChanged(r) => self.params.filters[0].resonance = r,
            Message::Filter1DriveChanged(d) => self.params.filters[0].drive = d,
            Message::Filter1KeyTrackingChanged(k) => self.params.filters[0].key_tracking = k,

            // Filter 2
            Message::Filter2TypeChanged(t) => self.params.filters[1].filter_type = t,
            Message::Filter2CutoffChanged(c) => self.params.filters[1].cutoff = c,
            Message::Filter2ResonanceChanged(r) => self.params.filters[1].resonance = r,
            Message::Filter2DriveChanged(d) => self.params.filters[1].drive = d,
            Message::Filter2KeyTrackingChanged(k) => self.params.filters[1].key_tracking = k,

            // Filter 3
            Message::Filter3TypeChanged(t) => self.params.filters[2].filter_type = t,
            Message::Filter3CutoffChanged(c) => self.params.filters[2].cutoff = c,
            Message::Filter3ResonanceChanged(r) => self.params.filters[2].resonance = r,
            Message::Filter3DriveChanged(d) => self.params.filters[2].drive = d,
            Message::Filter3KeyTrackingChanged(k) => self.params.filters[2].key_tracking = k,

            // Filter Envelope 1
            Message::FilterEnv1AttackChanged(a) => self.params.filter_envelopes[0].attack = a,
            Message::FilterEnv1DecayChanged(d) => self.params.filter_envelopes[0].decay = d,
            Message::FilterEnv1SustainChanged(s) => self.params.filter_envelopes[0].sustain = s,
            Message::FilterEnv1ReleaseChanged(r) => self.params.filter_envelopes[0].release = r,
            Message::FilterEnv1AmountChanged(a) => self.params.filter_envelopes[0].amount = a,

            // Filter Envelope 2
            Message::FilterEnv2AttackChanged(a) => self.params.filter_envelopes[1].attack = a,
            Message::FilterEnv2DecayChanged(d) => self.params.filter_envelopes[1].decay = d,
            Message::FilterEnv2SustainChanged(s) => self.params.filter_envelopes[1].sustain = s,
            Message::FilterEnv2ReleaseChanged(r) => self.params.filter_envelopes[1].release = r,
            Message::FilterEnv2AmountChanged(a) => self.params.filter_envelopes[1].amount = a,

            // Filter Envelope 3
            Message::FilterEnv3AttackChanged(a) => self.params.filter_envelopes[2].attack = a,
            Message::FilterEnv3DecayChanged(d) => self.params.filter_envelopes[2].decay = d,
            Message::FilterEnv3SustainChanged(s) => self.params.filter_envelopes[2].sustain = s,
            Message::FilterEnv3ReleaseChanged(r) => self.params.filter_envelopes[2].release = r,
            Message::FilterEnv3AmountChanged(a) => self.params.filter_envelopes[2].amount = a,

            // LFO 1
            Message::LFO1WaveformChanged(w) => self.params.lfos[0].waveform = w,
            Message::LFO1RateChanged(r) => self.params.lfos[0].rate = r,
            Message::LFO1DepthChanged(d) => self.params.lfos[0].depth = d,
            Message::LFO1FilterAmountChanged(a) => self.params.lfos[0].filter_amount = a,

            // LFO 2
            Message::LFO2WaveformChanged(w) => self.params.lfos[1].waveform = w,
            Message::LFO2RateChanged(r) => self.params.lfos[1].rate = r,
            Message::LFO2DepthChanged(d) => self.params.lfos[1].depth = d,
            Message::LFO2FilterAmountChanged(a) => self.params.lfos[1].filter_amount = a,

            // LFO 3
            Message::LFO3WaveformChanged(w) => self.params.lfos[2].waveform = w,
            Message::LFO3RateChanged(r) => self.params.lfos[2].rate = r,
            Message::LFO3DepthChanged(d) => self.params.lfos[2].depth = d,
            Message::LFO3FilterAmountChanged(a) => self.params.lfos[2].filter_amount = a,

            // Velocity Sensitivity
            Message::VelocityAmpChanged(v) => self.params.velocity.amp_sensitivity = v,
            Message::VelocityFilterChanged(v) => self.params.velocity.filter_sensitivity = v,
            Message::VelocityFilterEnvChanged(v) => self.params.velocity.filter_env_sensitivity = v,

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

            // Preset management
            Message::PresetNameChanged(name) => {
                self.preset_name = name;
            }
            Message::SavePreset => {
                return Task::perform(
                    Self::save_preset_dialog(self.preset_name.clone(), self.params),
                    |_| Message::PanicPressed, // Dummy message after save
                );
            }
            Message::LoadPreset => {
                return Task::perform(Self::load_preset_dialog(), Message::PresetLoaded);
            }
            Message::PresetLoaded(result) => match result {
                Ok(params) => {
                    self.params = params;
                }
                Err(e) => {
                    eprintln!("Failed to load preset: {}", e);
                }
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
        let mut rng = rand::thread_rng();

        let waveforms = [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
            Waveform::Pulse,
        ];
        let filter_types = [
            FilterType::Lowpass,
            FilterType::Highpass,
            FilterType::Bandpass,
        ];
        let lfo_waveforms = [
            LFOWaveform::Sine,
            LFOWaveform::Triangle,
            LFOWaveform::Square,
            LFOWaveform::Saw,
        ];

        let mut params = SynthParams::default();

        // Randomize oscillators
        for osc in &mut params.oscillators {
            osc.waveform = waveforms[rng.gen_range(0..waveforms.len())];
            osc.pitch = (rng.gen_range(-24.0..=24.0) as f32).round();
            osc.detune = (rng.gen_range(-50.0..=50.0) as f32).round();
            osc.gain = rng.gen_range(0.2..=0.8); // Keep reasonable gain range
            osc.pan = rng.gen_range(-1.0..=1.0);
            osc.unison = rng.gen_range(1..=7);
            osc.unison_detune = rng.gen_range(0.0..=50.0);
            osc.phase = rng.gen_range(0.0..=1.0);
            osc.shape = rng.gen_range(-0.8..=0.8); // Avoid extreme shaping
        }

        // Randomize filters
        for filter in &mut params.filters {
            filter.filter_type = filter_types[rng.gen_range(0..filter_types.len())];
            filter.cutoff = rng.gen_range(200.0..=10000.0); // Musical range
            filter.resonance = rng.gen_range(0.5..=5.0); // Avoid extreme resonance
            filter.drive = rng.gen_range(1.0..=5.0);
            filter.key_tracking = rng.gen_range(0.0..=1.0);
        }

        // Randomize filter envelopes
        for fenv in &mut params.filter_envelopes {
            fenv.attack = rng.gen_range(0.001..=2.0);
            fenv.decay = rng.gen_range(0.01..=2.0);
            fenv.sustain = rng.gen_range(0.0..=1.0);
            fenv.release = rng.gen_range(0.01..=2.0);
            fenv.amount = rng.gen_range(-5000.0..=5000.0);
        }

        // Randomize LFOs
        for lfo in &mut params.lfos {
            lfo.waveform = lfo_waveforms[rng.gen_range(0..lfo_waveforms.len())];
            lfo.rate = rng.gen_range(0.1..=10.0);
            lfo.depth = rng.gen_range(0.0..=1.0);
            lfo.filter_amount = rng.gen_range(0.0..=3000.0);
        }

        // Randomize velocity
        params.velocity.amp_sensitivity = rng.gen_range(0.3..=1.0);
        params.velocity.filter_sensitivity = rng.gen_range(0.0..=0.8);
        params.velocity.filter_env_sensitivity = rng.gen_range(0.0..=0.8);

        // Keep master gain reasonable
        params.master_gain = rng.gen_range(0.4..=0.7);

        params
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
            text("Filter Envelope:"),
            slider(
                0.0..=1.0,
                self.params.velocity.filter_env_sensitivity,
                Message::VelocityFilterEnvChanged
            )
            .step(0.01),
            text(format!(
                "{:.2}",
                self.params.velocity.filter_env_sensitivity
            )),
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
            button("PANIC").on_press(Message::PanicPressed).padding(10),
        ]
        .spacing(10)
        .padding(10);

        let content = column![
            row![title, keyboard_help, preset_controls],
            row![master_controls, velocity_controls],
            scrollable(row![osc1_section, osc2_section, osc3_section].spacing(20)),
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
        let filter_env = &self.params.filter_envelopes[index];
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

        let (
            wave_msg,
            pitch_msg,
            detune_msg,
            gain_msg,
            pan_msg,
            unison_msg,
            unison_detune_msg,
            phase_msg,
            shape_msg,
            ftype_msg,
            cutoff_msg,
            res_msg,
            drive_msg,
            key_track_msg,
            fenv_attack_msg,
            fenv_decay_msg,
            fenv_sustain_msg,
            fenv_release_msg,
            fenv_amount_msg,
            lfo_wave_msg,
            lfo_rate_msg,
            lfo_depth_msg,
            lfo_filter_msg,
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
                Message::Osc1ShapeChanged as fn(f32) -> Message,
                Message::Filter1TypeChanged as fn(FilterType) -> Message,
                Message::Filter1CutoffChanged as fn(f32) -> Message,
                Message::Filter1ResonanceChanged as fn(f32) -> Message,
                Message::Filter1DriveChanged as fn(f32) -> Message,
                Message::Filter1KeyTrackingChanged as fn(f32) -> Message,
                Message::FilterEnv1AttackChanged as fn(f32) -> Message,
                Message::FilterEnv1DecayChanged as fn(f32) -> Message,
                Message::FilterEnv1SustainChanged as fn(f32) -> Message,
                Message::FilterEnv1ReleaseChanged as fn(f32) -> Message,
                Message::FilterEnv1AmountChanged as fn(f32) -> Message,
                Message::LFO1WaveformChanged as fn(LFOWaveform) -> Message,
                Message::LFO1RateChanged as fn(f32) -> Message,
                Message::LFO1DepthChanged as fn(f32) -> Message,
                Message::LFO1FilterAmountChanged as fn(f32) -> Message,
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
                Message::Osc2ShapeChanged as fn(f32) -> Message,
                Message::Filter2TypeChanged as fn(FilterType) -> Message,
                Message::Filter2CutoffChanged as fn(f32) -> Message,
                Message::Filter2ResonanceChanged as fn(f32) -> Message,
                Message::Filter2DriveChanged as fn(f32) -> Message,
                Message::Filter2KeyTrackingChanged as fn(f32) -> Message,
                Message::FilterEnv2AttackChanged as fn(f32) -> Message,
                Message::FilterEnv2DecayChanged as fn(f32) -> Message,
                Message::FilterEnv2SustainChanged as fn(f32) -> Message,
                Message::FilterEnv2ReleaseChanged as fn(f32) -> Message,
                Message::FilterEnv2AmountChanged as fn(f32) -> Message,
                Message::LFO2WaveformChanged as fn(LFOWaveform) -> Message,
                Message::LFO2RateChanged as fn(f32) -> Message,
                Message::LFO2DepthChanged as fn(f32) -> Message,
                Message::LFO2FilterAmountChanged as fn(f32) -> Message,
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
                Message::Osc3ShapeChanged as fn(f32) -> Message,
                Message::Filter3TypeChanged as fn(FilterType) -> Message,
                Message::Filter3CutoffChanged as fn(f32) -> Message,
                Message::Filter3ResonanceChanged as fn(f32) -> Message,
                Message::Filter3DriveChanged as fn(f32) -> Message,
                Message::Filter3KeyTrackingChanged as fn(f32) -> Message,
                Message::FilterEnv3AttackChanged as fn(f32) -> Message,
                Message::FilterEnv3DecayChanged as fn(f32) -> Message,
                Message::FilterEnv3SustainChanged as fn(f32) -> Message,
                Message::FilterEnv3ReleaseChanged as fn(f32) -> Message,
                Message::FilterEnv3AmountChanged as fn(f32) -> Message,
                Message::LFO3WaveformChanged as fn(LFOWaveform) -> Message,
                Message::LFO3RateChanged as fn(f32) -> Message,
                Message::LFO3DepthChanged as fn(f32) -> Message,
                Message::LFO3FilterAmountChanged as fn(f32) -> Message,
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
            .push(text("Shape:"))
            .push(slider(-1.0..=1.0, osc.shape, shape_msg).step(0.01))
            .push(text(format!("{:.2}", osc.shape)))
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
            .push(text("Key Tracking:"))
            .push(slider(0.0..=1.0, filter.key_tracking, key_track_msg).step(0.01))
            .push(text(format!("{:.2}", filter.key_tracking)))
            .push(text("--- Filter Envelope ---").size(18))
            .push(text("Attack (s):"))
            .push(slider(0.001..=5.0, filter_env.attack, fenv_attack_msg).step(0.01))
            .push(text(format!("{:.3}", filter_env.attack)))
            .push(text("Decay (s):"))
            .push(slider(0.001..=5.0, filter_env.decay, fenv_decay_msg).step(0.01))
            .push(text(format!("{:.3}", filter_env.decay)))
            .push(text("Sustain:"))
            .push(slider(0.0..=1.0, filter_env.sustain, fenv_sustain_msg).step(0.01))
            .push(text(format!("{:.2}", filter_env.sustain)))
            .push(text("Release (s):"))
            .push(slider(0.001..=5.0, filter_env.release, fenv_release_msg).step(0.01))
            .push(text(format!("{:.3}", filter_env.release)))
            .push(text("Amount (Hz):"))
            .push(slider(-10000.0..=10000.0, filter_env.amount, fenv_amount_msg).step(100.0))
            .push(text(format!("{:.0}", filter_env.amount)))
            .push(text("--- LFO ---").size(18))
            .push(text("Waveform:"))
            .push(pick_list(lfo_waveforms, Some(lfo.waveform), lfo_wave_msg))
            .push(text("Rate (Hz):"))
            .push(slider(0.01..=20.0, lfo.rate, lfo_rate_msg).step(0.1))
            .push(text(format!("{:.2}", lfo.rate)))
            .push(text("Depth:"))
            .push(slider(0.0..=1.0, lfo.depth, lfo_depth_msg).step(0.01))
            .push(text(format!("{:.2}", lfo.depth)))
            .push(text("Filter Amount (Hz):"))
            .push(slider(0.0..=5000.0, lfo.filter_amount, lfo_filter_msg).step(50.0))
            .push(text(format!("{:.0}", lfo.filter_amount)))
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

    async fn save_preset_dialog(name: String, params: SynthParams) -> Result<(), String> {
        use rfd::AsyncFileDialog;

        let file = AsyncFileDialog::new()
            .set_title("Save Preset")
            .set_file_name(&format!("{}.json", name))
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
