use iced::keyboard;

/// Map keyboard key to MIDI note number
/// Using two rows: AWSEDFTGYHUJK for white keys, and QRTYUOP for black keys
pub fn key_to_midi_note(key: &keyboard::Key) -> Option<u8> {
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
