// GUI messages for VIZIA event handling

use std::path::PathBuf;
use vizia::prelude::Data;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum UiTab {
    Oscillator,
    FilterLfo,
    Effects,
}

#[derive(Debug, Clone)]
pub enum GuiMessage {
    // Parameter changes: (parameter_id, normalized_value)
    ParamChanged(u32, f32),

    // Sync knob visual to match model value (param_id, normalized_value)
    // Used after randomization or preset load to update UI
    SyncKnobValue(u32, f32),

    // Preset management
    PresetLoad(PathBuf),
    PresetSave(PathBuf),

    // Randomization
    Randomize,
    RandomizeOscillators,
    RandomizeFilters,
    RandomizeEnvelope,
    RandomizeEffects,

    // UI navigation
    SetActiveTab(UiTab),
}
