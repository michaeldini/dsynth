use vizia::prelude::*;

use crate::gui::vizia_gui::widgets::Knob;

/// A simple parameter control widget (labels for now)
pub fn param_knob(
    cx: &mut Context,
    param_id: u32,
    label: &str,
    initial_normalized: f32,
    default_normalized: f32,
) {
    const KNOB_SIZE: f32 = 54.0;
    const CELL_WIDTH: f32 = 54.0;

    VStack::new(cx, move |cx| {
        // Label at top
        Label::new(cx, label)
            .font_size(11.0)
            .color(Color::rgb(200, 200, 210))
            .width(Pixels(CELL_WIDTH))
            .height(Pixels(16.0))
            .text_align(TextAlign::Center)
            .text_wrap(false)
            .text_overflow(TextOverflow::Ellipsis);

        // Interactive knob with visual feedback
        Knob::new(
            cx,
            initial_normalized.clamp(0.0, 1.0),
            param_id,
            default_normalized.clamp(0.0, 1.0),
        )
        .width(Pixels(KNOB_SIZE))
        .height(Pixels(KNOB_SIZE));
    })
    .width(Pixels(CELL_WIDTH))
    .height(Pixels(16.0 + 4.0 + KNOB_SIZE)) // label + gap + knob
    .gap(Pixels(4.0));
}
