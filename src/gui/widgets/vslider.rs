use crate::gui::theme;
use vizia::prelude::*;

/// A simple reactive vertical slider widget.
///
/// - Click/drag to set value
/// - Double-click to reset to default
/// - Listens to `GuiMessage::SyncKnobValue` for UI sync after randomize/preset load
#[derive(Lens)]
pub struct VSlider {
    /// Current normalized value (0.0 to 1.0)
    normalized_value: f32,

    /// Default normalized value for double-click reset
    default_value: f32,

    /// Parameter ID for event emission
    param_id: u32,

    /// Is the slider currently being dragged?
    is_dragging: bool,
}

impl VSlider {
    pub fn new(
        cx: &mut Context,
        initial_value: f32,
        param_id: u32,
        default_value: f32,
    ) -> Handle<'_, Self> {
        Self {
            normalized_value: initial_value.clamp(0.0, 1.0),
            default_value: default_value.clamp(0.0, 1.0),
            param_id,
            is_dragging: false,
        }
        .build(cx, move |cx| {
            ZStack::new(cx, move |cx| {
                // Track
                Element::new(cx)
                    .width(Pixels(theme::SLIDER_WIDTH))
                    .height(Pixels(theme::SLIDER_HEIGHT))
                    .background_color(theme::WIDGET_BG)
                    .border_width(Pixels(2.0))
                    .border_color(theme::WIDGET_BORDER)
                    .corner_radius(Pixels(4.0));

                // Fill (from bottom)
                Element::new(cx)
                    .width(Pixels(theme::SLIDER_WIDTH - 4.0))
                    .height(
                        VSlider::normalized_value.map(move |v| Pixels(v * theme::SLIDER_HEIGHT)),
                    )
                    .bottom(Pixels(2.0))
                    .background_color(theme::WIDGET_ACCENT)
                    .corner_radius(Pixels(3.0));

                // Handle
                Element::new(cx)
                    .width(Pixels(theme::SLIDER_WIDTH + 6.0))
                    .height(Pixels(theme::SLIDER_HANDLE_HEIGHT))
                    .left(Pixels(-(6.0 / 2.0)))
                    .top(VSlider::normalized_value.map(move |v| {
                        Pixels(
                            (1.0 - v) * (theme::SLIDER_HEIGHT - theme::SLIDER_HANDLE_HEIGHT) + 2.0,
                        )
                    }))
                    .background_color(theme::WIDGET_TRACK)
                    .corner_radius(Pixels(3.0));
            })
            .width(Pixels(theme::SLIDER_WIDTH))
            .height(Pixels(theme::SLIDER_HEIGHT));
        })
        .width(Pixels(theme::SLIDER_WIDTH))
        .height(Pixels(theme::SLIDER_HEIGHT))
        .cursor(CursorIcon::Hand)
    }

    fn update_from_y(&mut self, bounds: BoundingBox, y: f32) {
        let height = bounds.height().max(1.0);
        let rel = (y - bounds.y).clamp(0.0, height);

        // Top = 1.0, bottom = 0.0
        self.normalized_value = (1.0 - (rel / height)).clamp(0.0, 1.0);
    }
}

impl View for VSlider {
    fn element(&self) -> Option<&'static str> {
        Some("vslider")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // Sync message (re-using existing knob sync message)
        event.map(|gui_msg: &crate::gui::GuiMessage, _meta| {
            if let crate::gui::GuiMessage::SyncKnobValue(param_id, normalized) = gui_msg {
                if *param_id == self.param_id {
                    self.normalized_value = normalized.clamp(0.0, 1.0);
                }
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                self.is_dragging = true;
                let bounds = cx.cache.get_bounds(cx.current());
                self.update_from_y(bounds, cx.mouse().cursor_y);

                cx.capture();
                cx.set_active(true);

                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.param_id,
                    self.normalized_value,
                ));

                meta.consume();
            }

            WindowEvent::MouseMove(_x, y) => {
                if self.is_dragging {
                    let bounds = cx.cache.get_bounds(cx.current());
                    self.update_from_y(bounds, *y);

                    cx.emit(crate::gui::GuiMessage::ParamChanged(
                        self.param_id,
                        self.normalized_value,
                    ));

                    meta.consume();
                }
            }

            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.is_dragging {
                    self.is_dragging = false;
                    cx.release();
                    cx.set_active(false);
                    meta.consume();
                }
            }

            WindowEvent::MouseDoubleClick(MouseButton::Left) => {
                self.normalized_value = self.default_value;
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.param_id,
                    self.normalized_value,
                ));
                meta.consume();
            }

            _ => {}
        });
    }
}

/// Helper function to create a labeled vertical slider (label + slider in VStack)
pub fn param_vslider(
    cx: &mut Context,
    param_id: u32,
    label: &str,
    initial_normalized: f32,
    default_normalized: f32,
) {
    VStack::new(cx, move |cx| {
        Label::new(cx, label)
            .font_size(11.0)
            .color(theme::TEXT_SECONDARY)
            .width(Pixels(theme::SLIDER_CELL_WIDTH))
            .height(Pixels(theme::LABEL_HEIGHT))
            .text_align(TextAlign::Center)
            .text_wrap(false)
            .text_overflow(TextOverflow::Ellipsis);

        VSlider::new(
            cx,
            initial_normalized.clamp(0.0, 1.0),
            param_id,
            default_normalized.clamp(0.0, 1.0),
        )
        .height(Pixels(theme::SLIDER_HEIGHT));
    })
    .width(Pixels(theme::SLIDER_CELL_WIDTH))
    .height(Pixels(theme::LABEL_HEIGHT + 4.0 + theme::SLIDER_HEIGHT))
    .gap(Pixels(4.0));
}
