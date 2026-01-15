// Helper functions for parameter normalization

use crate::gui::widgets::param_checkbox;
use crate::gui::GuiState;
use crate::plugin::param_registry;
use crate::plugin::param_update::param_get;
use vizia::prelude::*;

pub fn current_normalized(cx: &mut Context, param_id: u32) -> f32 {
    let arc = GuiState::synth_params.get(cx);
    let params = arc.read();
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

// Common effect header: checkbox first, then title label
pub fn effect_header(cx: &mut Context, enabled_param: u32, title: &str) {
    HStack::new(cx, |cx| {
        let enabled = current_normalized(cx, enabled_param);
        param_checkbox(cx, enabled_param, "On", enabled > 0.5);
        Label::new(cx, title)
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(30.0));
    })
    .height(Units::Auto)
    .gap(Pixels(8.0));
}

// Common effect row wrapper: full-width VStack with standard gap and explicit height.
// Usage: effect_row(cx, 125.0, |cx| core::build_distortion_section(cx))
pub fn effect_row<F>(cx: &mut Context, height_px: f32, builder: F)
where
    F: FnOnce(&mut Context),
{
    VStack::new(cx, |cx| builder(cx))
        .width(Stretch(1.0))
        .height(Pixels(height_px))
        .gap(Pixels(6.0));
}
