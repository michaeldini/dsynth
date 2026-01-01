// Helper functions for parameter normalization

use crate::gui::vizia_gui::GuiState;
use crate::plugin::param_registry;
use crate::plugin::param_update::param_get;
use vizia::prelude::*;

pub fn current_normalized(cx: &mut Context, param_id: u32) -> f32 {
    let arc = GuiState::synth_params.get(cx);
    let params = arc.read().unwrap();
    let denorm = param_get::get_param(&params, param_id);
    let registry = param_registry::get_registry();
    if let Some(desc) = registry.get(param_id) {
        match &desc.param_type {
            crate::plugin::param_descriptor::ParamType::Float { min, max, .. } => {
                if *max > *min {
                    ((denorm - *min) / (*max - *min)).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            crate::plugin::param_descriptor::ParamType::Bool => {
                if denorm > 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                if variants.len() > 1 {
                    (denorm / (variants.len() - 1) as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            crate::plugin::param_descriptor::ParamType::Int { min, max } => {
                let range = (*max - *min) as f32;
                if range > 0.0 {
                    ((denorm - *min as f32) / range).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
        }
    } else {
        0.0
    }
}

pub fn default_normalized(param_id: u32) -> f32 {
    let registry = param_registry::get_registry();
    registry.get(param_id).map(|d| d.default).unwrap_or(0.0)
}
