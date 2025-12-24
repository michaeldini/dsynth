use vizia::prelude::*;

/// A simple reactive rotary knob widget
///
/// Visual feedback via CSS rotation of an indicator line.
/// Drag vertically to change value, double-click to reset.
#[derive(Lens)]
pub struct Knob {
    /// Current normalized value (0.0 to 1.0)
    normalized_value: f32,

    /// Default normalized value for double-click reset
    default_value: f32,

    /// Parameter ID for event emission
    param_id: u32,

    /// Is the knob currently being dragged?
    is_dragging: bool,

    /// Y position at start of drag
    drag_start_y: f32,

    /// Value at start of drag
    drag_start_value: f32,
}

impl Knob {
    pub fn new(
        cx: &mut Context,
        initial_value: f32,
        param_id: u32,
        default_value: f32,
    ) -> Handle<'_, Self> {
        const KNOB_SIZE: f32 = 54.0;

        Self {
            normalized_value: initial_value.clamp(0.0, 1.0),
            default_value: default_value.clamp(0.0, 1.0),
            param_id,
            is_dragging: false,
            drag_start_y: 0.0,
            drag_start_value: 0.0,
        }
        .build(cx, |cx| {
            // Outer circle (knob body) with indicator line inside
            ZStack::new(cx, |cx| {
                // Background circle
                Element::new(cx)
                    .class("knob-body")
                    .width(Pixels(KNOB_SIZE))
                    .height(Pixels(KNOB_SIZE))
                    .background_color(Color::rgb(55, 55, 62))
                    .border_width(Pixels(2.0))
                    .border_color(Color::rgb(90, 90, 100))
                    .corner_radius(Percentage(50.0));

                // Indicator line - positioned to rotate around knob center
                // The line is 20px tall, positioned so its bottom is at knob center
                Element::new(cx)
                    .class("knob-indicator")
                    .width(Pixels(3.0))
                    .height(Pixels(20.0))
                    .background_color(Color::rgb(200, 200, 210))
                    .corner_radius(Pixels(1.5))
                    .left(Pixels(KNOB_SIZE / 2.0 - 1.5))
                    .top(Pixels(KNOB_SIZE / 2.0 - 20.0))
                    .translate((Pixels(0.0), Pixels(10.0)))
                    .rotate(Knob::normalized_value.map(|v| Angle::Deg(v * 270.0 - 135.0)));
            })
            .width(Pixels(KNOB_SIZE))
            .height(Pixels(KNOB_SIZE));
        })
        .width(Pixels(KNOB_SIZE))
        .height(Pixels(KNOB_SIZE))
    }

    /// Update value from vertical drag
    fn update_from_drag(&mut self, delta_y: f32, sensitivity: f32) {
        let delta_value = -delta_y * sensitivity;
        self.normalized_value = (self.drag_start_value + delta_value).clamp(0.0, 1.0);
    }
}

impl View for Knob {
    fn element(&self) -> Option<&'static str> {
        Some("knob")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // Handle SyncKnobValue messages to update visual state
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
                self.drag_start_y = cx.mouse().cursor_y;
                self.drag_start_value = self.normalized_value;
                cx.capture();
                cx.set_active(true);
                meta.consume();
            }

            WindowEvent::MouseMove(_x, y) => {
                if self.is_dragging {
                    let delta_y = *y - self.drag_start_y;
                    self.update_from_drag(delta_y, 1.0 / 200.0);

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
