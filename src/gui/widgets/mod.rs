pub mod knob;
pub use knob::{Knob, param_knob};

// The remaining widgets are only used by the main poly synth UI.
#[cfg(any(feature = "clap", feature = "standalone"))]
pub mod envelope_editor;

#[cfg(any(feature = "clap", feature = "standalone"))]
pub mod vslider;

#[cfg(any(feature = "clap", feature = "standalone"))]
pub mod param_checkbox;

#[cfg(any(feature = "clap", feature = "standalone"))]
pub mod param_cycle_button;

#[cfg(any(feature = "clap", feature = "standalone"))]
pub use envelope_editor::EnvelopeEditor;

#[cfg(any(feature = "clap", feature = "standalone"))]
pub use vslider::{VSlider, param_vslider};

#[cfg(any(feature = "clap", feature = "standalone"))]
pub use param_checkbox::param_checkbox;

#[cfg(any(feature = "clap", feature = "standalone"))]
pub use param_cycle_button::{
    distortion_type_button, filter_type_button, fm_source_button, lfo_waveform_button,
    oscillator_waveform_button, tempo_sync_button,
};
