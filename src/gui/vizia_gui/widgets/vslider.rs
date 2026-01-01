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
        const SLIDER_WIDTH: f32 = 18.0;
        const SLIDER_HEIGHT: f32 = 90.0;
        const HANDLE_HEIGHT: f32 = 8.0;

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
                    .width(Pixels(SLIDER_WIDTH))
                    .height(Pixels(SLIDER_HEIGHT))
                    .background_color(Color::rgb(55, 55, 62))
                    .border_width(Pixels(2.0))
                    .border_color(Color::rgb(90, 90, 100))
                    .corner_radius(Pixels(4.0));

                // Fill (from bottom)
                Element::new(cx)
                    .width(Pixels(SLIDER_WIDTH - 4.0))
                    .height(VSlider::normalized_value.map(move |v| Pixels(v * SLIDER_HEIGHT)))
                    .bottom(Pixels(2.0))
                    .background_color(Color::rgb(200, 200, 210))
                    .corner_radius(Pixels(3.0));

                // Handle
                Element::new(cx)
                    .width(Pixels(SLIDER_WIDTH + 6.0))
                    .height(Pixels(HANDLE_HEIGHT))
                    .left(Pixels(-(6.0 / 2.0)))
                    .top(
                        VSlider::normalized_value.map(move |v| {
                            Pixels((1.0 - v) * (SLIDER_HEIGHT - HANDLE_HEIGHT) + 2.0)
                        }),
                    )
                    .background_color(Color::rgb(120, 120, 130))
                    .corner_radius(Pixels(3.0));
            })
            .width(Pixels(SLIDER_WIDTH))
            .height(Pixels(SLIDER_HEIGHT));
        })
        .width(Pixels(SLIDER_WIDTH))
        .height(Pixels(SLIDER_HEIGHT))
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
        event.map(|gui_msg: &crate::gui::vizia_gui::GuiMessage, _meta| {
            if let crate::gui::vizia_gui::GuiMessage::SyncKnobValue(param_id, normalized) = gui_msg
            {
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

                cx.emit(crate::gui::vizia_gui::GuiMessage::ParamChanged(
                    self.param_id,
                    self.normalized_value,
                ));

                meta.consume();
            }

            WindowEvent::MouseMove(_x, y) => {
                if self.is_dragging {
                    let bounds = cx.cache.get_bounds(cx.current());
                    self.update_from_y(bounds, *y);

                    cx.emit(crate::gui::vizia_gui::GuiMessage::ParamChanged(
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
                cx.emit(crate::gui::vizia_gui::GuiMessage::ParamChanged(
                    self.param_id,
                    self.normalized_value,
                ));
                meta.consume();
            }

            _ => {}
        });
    }
}
