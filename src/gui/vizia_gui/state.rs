// GUI state for VIZIA - shared between plugin and standalone

use crate::gui::vizia_gui::GuiMessage;
use crate::params::SynthParams;
use crate::plugin::gui_param_change::GuiParamChange;
use std::sync::{Arc, Mutex, RwLock};
use triple_buffer::Input;
use vizia::prelude::*;

/// GUI state that holds synth parameters and provides VIZIA lens access
/// 
/// For Phase 1, we keep this simple - just store the Arc and handle updates directly.
/// In Phase 2, we'll add reactive caching for better performance.
#[derive(Clone, Lens)]
pub struct GuiState {
    /// Shared synthesizer parameters (Arc for cross-thread access)
    pub synth_params: Arc<RwLock<SynthParams>>,

    /// GUI -> audio thread lock-free param change producer
    pub gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,

    /// UI feedback string (e.g. last changed param/value)
    pub last_param_text: String,
}

impl GuiState {
    /// Create new GUI state from synth parameters
    pub fn new(
        synth_params: Arc<RwLock<SynthParams>>,
        gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,
    ) -> Self {
        Self {
            synth_params,
            gui_param_producer,
            last_param_text: String::new(),
        }
    }
    
    /// Update parameter value and write to synth_params
    pub fn update_param(&mut self, param_id: u32, normalized_value: f32) {
        // Write to synth_params (will be picked up by audio thread via triple-buffer)
        if let Ok(mut params) = self.synth_params.write() {
            crate::plugin::param_update::param_apply::apply_param(&mut params, param_id, normalized_value);

            // Send a single param change to the audio thread (lock-free once written).
            if let Ok(mut producer) = self.gui_param_producer.lock() {
                producer.write(GuiParamChange {
                    param_id,
                    normalized: normalized_value.clamp(0.0, 1.0),
                });
            }
        }
    }
}

impl Model for GuiState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|msg, meta| match msg {
            GuiMessage::ParamChanged(param_id, normalized) => {
                let debug_msg = format!("DEBUG: GuiMessage::ParamChanged received - param_id: 0x{:08X}, normalized: {}\n", param_id, normalized);
                let _ = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/dsynth_debug.log").and_then(|mut f| std::io::Write::write_all(&mut f, debug_msg.as_bytes()));
                
                self.update_param(*param_id, *normalized);

                // Provide immediate visual feedback in the UI.
                let registry = crate::plugin::param_registry::get_registry();
                if let Some(desc) = registry.get(*param_id) {
                    self.last_param_text = format!(
                        "{}: {:.0}%",
                        desc.name,
                        (normalized.clamp(0.0, 1.0) * 100.0)
                    );
                } else {
                    self.last_param_text = format!(
                        "Param 0x{:08X}: {:.0}%",
                        *param_id,
                        (normalized.clamp(0.0, 1.0) * 100.0)
                    );
                }

                cx.needs_redraw();
                meta.consume();
            }
            _ => {}
        });
    }
}

