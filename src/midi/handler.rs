use crossbeam_channel::{Receiver, unbounded};
use midir::{MidiInput, MidiInputConnection};
use std::error::Error;

/// MIDI message events
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiEvent {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    ControlChange { controller: u8, value: u8 },
}

/// MIDI input handler
pub struct MidiHandler {
    _connection: Option<MidiInputConnection<()>>,
    event_receiver: Receiver<MidiEvent>,
}

impl MidiHandler {
    /// Create a new MIDI handler
    ///
    /// # Returns
    /// Result containing (MidiHandler, event_receiver) or error
    pub fn new() -> Result<(Self, Receiver<MidiEvent>), Box<dyn Error>> {
        let (sender, receiver) = unbounded();
        
        let midi_in = MidiInput::new("DSynth MIDI Input")?;
        
        // Try to connect to the first available MIDI input port
        let ports = midi_in.ports();
        
        let connection = if let Some(port) = ports.first() {
            let port_name = midi_in.port_name(port)?;
            println!("Connecting to MIDI port: {}", port_name);
            
            let connection = midi_in.connect(
                port,
                "dsynth-input",
                move |_timestamp, message, _| {
                    if let Some(event) = Self::parse_midi_message(message) {
                        let _ = sender.send(event);
                    }
                },
                (),
            )?;
            
            Some(connection)
        } else {
            println!("No MIDI input ports available");
            None
        };
        
        let handler = Self {
            _connection: connection,
            event_receiver: receiver.clone(),
        };
        
        Ok((handler, receiver))
    }

    /// Parse MIDI message bytes into MidiEvent
    fn parse_midi_message(message: &[u8]) -> Option<MidiEvent> {
        if message.len() < 2 {
            return None;
        }
        
        let status = message[0];
        let _channel = status & 0x0F;
        let message_type = status & 0xF0;
        
        match message_type {
            0x90 => {
                // Note On
                let note = message[1];
                let velocity = message[2];
                
                // MIDI spec: Note On with velocity 0 is actually Note Off
                if velocity == 0 {
                    Some(MidiEvent::NoteOff { note })
                } else {
                    Some(MidiEvent::NoteOn { note, velocity })
                }
            }
            0x80 => {
                // Note Off
                let note = message[1];
                Some(MidiEvent::NoteOff { note })
            }
            0xB0 => {
                // Control Change
                let controller = message[1];
                let value = message[2];
                Some(MidiEvent::ControlChange { controller, value })
            }
            _ => None,
        }
    }

    /// Get event receiver for processing MIDI events
    pub fn event_receiver(&self) -> &Receiver<MidiEvent> {
        &self.event_receiver
    }

    /// List available MIDI input ports
    pub fn list_ports() -> Result<Vec<String>, Box<dyn Error>> {
        let midi_in = MidiInput::new("DSynth MIDI Input")?;
        let ports = midi_in.ports();
        
        let mut port_names = Vec::new();
        for port in &ports {
            let name = midi_in.port_name(port)?;
            port_names.push(name);
        }
        
        Ok(port_names)
    }
}

/// Helper function to convert MIDI velocity (0-127) to normalized value (0.0-1.0)
pub fn velocity_to_float(velocity: u8) -> f32 {
    velocity as f32 / 127.0
}

/// Helper function to convert MIDI CC value (0-127) to normalized value (0.0-1.0)
pub fn cc_to_float(value: u8) -> f32 {
    value as f32 / 127.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note_on() {
        let message = [0x90, 60, 100]; // Note On, C4, velocity 100
        let event = MidiHandler::parse_midi_message(&message);
        
        assert_eq!(
            event,
            Some(MidiEvent::NoteOn { note: 60, velocity: 100 })
        );
    }

    #[test]
    fn test_parse_note_off() {
        let message = [0x80, 60, 0]; // Note Off, C4
        let event = MidiHandler::parse_midi_message(&message);
        
        assert_eq!(event, Some(MidiEvent::NoteOff { note: 60 }));
    }

    #[test]
    fn test_parse_note_on_zero_velocity() {
        // Note On with velocity 0 should be treated as Note Off
        let message = [0x90, 60, 0];
        let event = MidiHandler::parse_midi_message(&message);
        
        assert_eq!(event, Some(MidiEvent::NoteOff { note: 60 }));
    }

    #[test]
    fn test_parse_control_change() {
        let message = [0xB0, 7, 100]; // CC, controller 7 (volume), value 100
        let event = MidiHandler::parse_midi_message(&message);
        
        assert_eq!(
            event,
            Some(MidiEvent::ControlChange { controller: 7, value: 100 })
        );
    }

    #[test]
    fn test_parse_invalid_message() {
        let message = [0xFF]; // Invalid/incomplete message
        let event = MidiHandler::parse_midi_message(&message);
        
        assert_eq!(event, None);
    }

    #[test]
    fn test_parse_unsupported_message() {
        let message = [0xE0, 0, 64]; // Pitch bend (not supported)
        let event = MidiHandler::parse_midi_message(&message);
        
        assert_eq!(event, None);
    }

    #[test]
    fn test_velocity_to_float() {
        assert_eq!(velocity_to_float(0), 0.0);
        assert_eq!(velocity_to_float(127), 1.0);
        assert!((velocity_to_float(64) - 0.504).abs() < 0.01);
    }

    #[test]
    fn test_cc_to_float() {
        assert_eq!(cc_to_float(0), 0.0);
        assert_eq!(cc_to_float(127), 1.0);
        assert!((cc_to_float(64) - 0.504).abs() < 0.01);
    }

    #[test]
    fn test_list_ports() {
        // Should not panic
        let result = MidiHandler::list_ports();
        
        match result {
            Ok(ports) => {
                println!("Found {} MIDI input ports", ports.len());
            }
            Err(e) => {
                println!("Failed to list MIDI ports: {}", e);
            }
        }
    }

    #[test]
    fn test_midi_handler_creation() {
        // Try to create MIDI handler
        // This may fail in test environments without MIDI devices
        let result = MidiHandler::new();
        
        match result {
            Ok((_handler, receiver)) => {
                println!("MIDI handler created successfully");
                assert!(receiver.is_empty());
            }
            Err(e) => {
                println!("MIDI handler creation failed (expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_midi_event_channel_independence() {
        // Different channels should parse the same
        let message_ch0 = [0x90, 60, 100]; // Channel 0
        let message_ch15 = [0x9F, 60, 100]; // Channel 15
        
        let event0 = MidiHandler::parse_midi_message(&message_ch0);
        let event15 = MidiHandler::parse_midi_message(&message_ch15);
        
        // Both should produce the same event (we don't track channel separately)
        assert_eq!(event0, event15);
    }
}

