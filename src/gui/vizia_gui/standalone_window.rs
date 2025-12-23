// VIZIA standalone window for desktop application

use crate::audio::output::EngineEvent;
use crate::gui::vizia_gui::GuiState;
use crate::gui::vizia_gui::shared_ui;
use crate::params::SynthParams;
use crate::plugin::gui_param_change::GuiParamChange;
use crossbeam_channel::Sender;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use triple_buffer::Input;
use vizia::prelude::*;

// Import Application and WindowModifiers directly from vizia_winit since both backends are enabled
// (vizia's prelude only exports Application when exactly one backend is enabled)
use vizia_winit::application::Application;
use vizia_winit::window_modifiers::WindowModifiers;

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 800;

/// Run standalone VIZIA GUI (blocking call - runs until window closes)
pub fn run_standalone_gui(
    synth_params: Arc<RwLock<SynthParams>>,
    gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,
    event_sender: Sender<EngineEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = Application::new(move |cx| {
        // Initialize GUI state with event sender for standalone features
        GuiState::new_standalone(
            synth_params.clone(),
            gui_param_producer.clone(),
            event_sender.clone(),
        )
        .build(cx);

        // Create root container that captures keyboard events
        KeyboardCapture::new(cx, event_sender.clone(), |cx| {
            // Build shared UI layout inside the keyboard-capturing container
            shared_ui::build_ui(cx);
        });
    })
    .title("DSynth")
    .inner_size((WINDOW_WIDTH, WINDOW_HEIGHT))
    .run();

    Ok(())
}

/// Container view that captures keyboard events and converts them to MIDI notes
struct KeyboardCapture {
    event_sender: Sender<EngineEvent>,
    /// Track which notes are currently held to filter out key repeat events
    pressed_notes: HashSet<u8>,
}

impl KeyboardCapture {
    fn new<F>(cx: &mut Context, event_sender: Sender<EngineEvent>, content: F) -> Handle<'_, Self>
    where
        F: FnOnce(&mut Context),
    {
        Self {
            event_sender,
            pressed_notes: HashSet::new(),
        }
        .build(cx, content)
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .focusable(true)
    }
}

impl View for KeyboardCapture {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::KeyDown(code, _) => {
                if let Some(note) = key_code_to_midi_note(code) {
                    // Only send NoteOn if this note isn't already pressed (filter key repeat)
                    if !self.pressed_notes.contains(&note) {
                        self.pressed_notes.insert(note);
                        let _ = self.event_sender.try_send(EngineEvent::NoteOn {
                            note,
                            velocity: 1.0,
                        });
                    }
                    meta.consume();
                }
            }
            WindowEvent::KeyUp(code, _) => {
                if let Some(note) = key_code_to_midi_note(code) {
                    // Only send NoteOff if the note was actually pressed
                    if self.pressed_notes.remove(&note) {
                        let _ = self.event_sender.try_send(EngineEvent::NoteOff { note });
                    }
                    meta.consume();
                }
            }
            WindowEvent::MouseDown(_) => {
                // Focus this element when clicked anywhere, to ensure keyboard works
                cx.focus();
            }
            _ => {}
        });
    }
}

/// Map keyboard code to MIDI note number
/// Using QWERTY layout: AWSEDFTGYHUJKOLP (white + black keys)
fn key_code_to_midi_note(code: &Code) -> Option<u8> {
    match code {
        // Main keyboard piano layout
        Code::KeyA => Some(60), // C4
        Code::KeyW => Some(61), // C#4
        Code::KeyS => Some(62), // D4
        Code::KeyE => Some(63), // D#4
        Code::KeyD => Some(64), // E4
        Code::KeyF => Some(65), // F4
        Code::KeyT => Some(66), // F#4
        Code::KeyG => Some(67), // G4
        Code::KeyY => Some(68), // G#4
        Code::KeyH => Some(69), // A4
        Code::KeyU => Some(70), // A#4
        Code::KeyJ => Some(71), // B4
        Code::KeyK => Some(72), // C5
        Code::KeyO => Some(73), // C#5
        Code::KeyL => Some(74), // D5
        Code::KeyP => Some(75), // D#5

        // Lower octave
        Code::KeyZ => Some(48), // C3
        Code::KeyX => Some(50), // D3
        Code::KeyC => Some(52), // E3
        Code::KeyV => Some(53), // F3
        Code::KeyB => Some(55), // G3
        Code::KeyN => Some(57), // A3
        Code::KeyM => Some(59), // B3

        _ => None,
    }
}
