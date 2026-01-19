//! CLAP note ports extension implementation

use crate::{plugin::ClapPlugin, NotePortConfig};
use clap_sys::ext::note_ports::*;
use clap_sys::string_sizes::CLAP_NAME_SIZE;
use std::sync::OnceLock;

/// Get the note ports extension for a plugin type
pub fn get_extension<P: ClapPlugin>() -> &'static clap_plugin_note_ports {
    static EXT: OnceLock<clap_plugin_note_ports> = OnceLock::new();
    EXT.get_or_init(|| clap_plugin_note_ports {
        count: Some(note_ports_count::<P>),
        get: Some(note_ports_get::<P>),
    })
}

unsafe extern "C" fn note_ports_count<P: ClapPlugin>(
    _plugin: *const clap_sys::plugin::clap_plugin,
    is_input: bool,
) -> u32 {
    let descriptor = P::descriptor();
    match descriptor.note_ports {
        NotePortConfig::None => 0,
        NotePortConfig::Input => {
            // MIDI: 1 input, 0 outputs (typical for instruments/effects)
            if is_input {
                1
            } else {
                0
            }
        }
        NotePortConfig::Custom { inputs, outputs } => {
            if is_input {
                inputs
            } else {
                outputs
            }
        }
    }
}

unsafe extern "C" fn note_ports_get<P: ClapPlugin>(
    _plugin: *const clap_sys::plugin::clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_note_port_info,
) -> bool {
    if info.is_null() {
        return false;
    }

    let descriptor = P::descriptor();
    let count = match descriptor.note_ports {
        NotePortConfig::None => 0,
        NotePortConfig::Input => {
            if is_input {
                1
            } else {
                0
            }
        }
        NotePortConfig::Custom { inputs, outputs } => {
            if is_input {
                inputs
            } else {
                outputs
            }
        }
    };

    if index >= count {
        return false;
    }

    let info = &mut *info;

    // Set port ID
    info.id = if is_input {
        index
    } else {
        0x1000 + index // Offset output IDs
    };

    // Set port name
    let name = if is_input {
        format!("MIDI In {}\0", index + 1)
    } else {
        format!("MIDI Out {}\0", index + 1)
    };
    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(CLAP_NAME_SIZE - 1);
    std::ptr::copy_nonoverlapping(
        name_bytes.as_ptr(),
        info.name.as_mut_ptr() as *mut u8,
        copy_len,
    );
    info.name[copy_len] = 0;

    // Support MIDI, note expressions, and per-note automation
    info.supported_dialects =
        CLAP_NOTE_DIALECT_MIDI | CLAP_NOTE_DIALECT_CLAP | CLAP_NOTE_DIALECT_MIDI_MPE;

    info.preferred_dialect = CLAP_NOTE_DIALECT_MIDI;

    true
}
