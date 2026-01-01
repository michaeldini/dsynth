use vizia::prelude::*;

/// A checkbox parameter control widget for boolean parameters
pub fn param_checkbox(cx: &mut Context, param_id: u32, label: &str, _initial_checked: bool) {
    use crate::gui::{GuiMessage, GuiState};

    const CELL_WIDTH: f32 = 80.0;

    HStack::new(cx, move |cx| {
        // Reactive button that acts like a checkbox - gets current state from GuiState
        Button::new(cx, move |cx| {
            // Get current parameter value from the GUI state to determine if checked
            let current_value = GuiState::synth_params.map(move |params_arc| {
                if let Ok(params) = params_arc.read() {
                    crate::plugin::param_update::param_get::get_param(&params, param_id)
                } else {
                    0.0
                }
            });

            let is_checked = current_value.map(|v| *v > 0.5);

            Label::new(
                cx,
                is_checked.map(|checked| if *checked { "✓" } else { "✗" }),
            )
            .font_size(14.0)
            .color(is_checked.map(|checked| {
                if *checked {
                    Color::rgb(100, 255, 100) // Green when ON (checked)
                } else {
                    Color::rgb(255, 100, 100) // Red when OFF (unchecked)
                }
            }))
        })
        .on_press(move |cx| {
            // Get current state and toggle it
            let arc = GuiState::synth_params.get(cx);
            let current_value = if let Ok(params) = arc.read() {
                crate::plugin::param_update::param_get::get_param(&params, param_id)
            } else {
                0.0
            };

            // Toggle: if currently > 0.5 (ON), turn OFF (0.0), otherwise turn ON (1.0)
            let new_value = if current_value > 0.5 { 0.0 } else { 1.0 };

            cx.emit(GuiMessage::ParamChanged(param_id, new_value));
        })
        .width(Pixels(20.0))
        .height(Pixels(20.0))
        .background_color(Color::rgb(60, 60, 65))
        .border_width(Pixels(1.0))
        .border_color(Color::rgb(100, 100, 110));

        // Label at right
        Label::new(cx, label)
            .font_size(11.0)
            .color(Color::rgb(200, 200, 210))
            // .width(Pixels(CELL_WIDTH))
            .height(Pixels(16.0))
            .text_align(TextAlign::Center)
            .text_wrap(false)
            .text_overflow(TextOverflow::Ellipsis);
    })
    .width(Pixels(CELL_WIDTH))
    .gap(Pixels(4.0));
}
