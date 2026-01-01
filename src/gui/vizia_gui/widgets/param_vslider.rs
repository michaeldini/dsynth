use vizia::prelude::*;

use crate::gui::vizia_gui::widgets::VSlider;

/// Parameter control: label + vertical slider.
pub fn param_vslider(
    cx: &mut Context,
    param_id: u32,
    label: &str,
    initial_normalized: f32,
    default_normalized: f32,
) {
    const CELL_WIDTH: f32 = 42.0;
    const LABEL_HEIGHT: f32 = 16.0;
    const SLIDER_HEIGHT: f32 = 90.0;

    VStack::new(cx, move |cx| {
        Label::new(cx, label)
            .font_size(11.0)
            .color(Color::rgb(200, 200, 210))
            .width(Pixels(CELL_WIDTH))
            .height(Pixels(LABEL_HEIGHT))
            .text_align(TextAlign::Center)
            .text_wrap(false)
            .text_overflow(TextOverflow::Ellipsis);

        VSlider::new(
            cx,
            initial_normalized.clamp(0.0, 1.0),
            param_id,
            default_normalized.clamp(0.0, 1.0),
        )
        .height(Pixels(SLIDER_HEIGHT));
    })
    .width(Pixels(CELL_WIDTH))
    .height(Pixels(LABEL_HEIGHT + 4.0 + SLIDER_HEIGHT))
    .gap(Pixels(4.0));
}
