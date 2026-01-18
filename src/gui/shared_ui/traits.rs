// Trait-based abstractions for UI sections

use vizia::prelude::*;

/// Represents a UI section with multiple indexed instances (e.g., Osc 1, Osc 2, Osc 3)
pub trait IndexedSection {
    /// Get the parameter IDs for this instance
    fn get_params(&self, index: usize) -> Self::Params;

    /// Associated type for the parameter set
    type Params;

    /// Build the UI section for a specific index (1-based)
    fn build(&self, cx: &mut Context, index: usize);

    /// Get the display name for this section type
    fn section_name(&self) -> &'static str;
}

/// Represents a parameter-based effect section with enable toggle
pub trait EffectSection {
    /// Get all parameter IDs for this effect
    fn get_params(&self) -> Self::Params;

    /// Associated type for the parameter set
    type Params;

    /// Build the effect UI section
    fn build(&self, cx: &mut Context);

    /// Get the display name
    fn name(&self) -> &'static str;

    /// Get the enable parameter ID
    fn enable_param(&self) -> u32;
}

/// Helper trait for building common parameter control layouts
pub trait ParameterLayout {
    /// Build a standard header with label and enable checkbox
    fn build_header(cx: &mut Context, title: &str, enable_param: Option<u32>, enabled: bool) {
        HStack::new(cx, |cx| {
            Label::new(cx, title)
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));

            if let Some(param_id) = enable_param {
                crate::gui::widgets::param_checkbox(cx, param_id, "On", enabled);
            }
        })
        .height(Units::Auto)
        .gap(Pixels(8.0));
    }

    /// Build a horizontal parameter knob row
    fn build_param_row<F>(cx: &mut Context, builder: F)
    where
        F: FnOnce(&mut Context),
    {
        HStack::new(cx, builder)
            .height(Units::Auto)
            .gap(Pixels(6.0));
    }
}

// Blanket implementation for all types
impl<T> ParameterLayout for T {}
