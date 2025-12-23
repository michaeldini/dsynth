use super::param_descriptor::ParamId;

/// A single GUI-initiated parameter change.
///
/// This is sent from the GUI thread to the audio thread via a lock-free
/// triple-buffer. The audio thread applies it to `current_params` and then
/// publishes the updated `SynthParams` through the existing engine parameter
/// triple-buffer.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GuiParamChange {
    pub param_id: ParamId,
    /// Normalized value (0.0-1.0)
    pub normalized: f32,
}

impl Default for GuiParamChange {
    fn default() -> Self {
        Self {
            param_id: 0,
            normalized: 0.0,
        }
    }
}
