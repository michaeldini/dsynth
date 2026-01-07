use crate::gui::theme;
use vizia::prelude::*;

/// A simple reactive rotary knob widget with optional label
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

    /// Optional label
    label: Option<String>,
}

impl Knob {
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
            drag_start_y: 0.0,
            drag_start_value: 0.0,
            label: None,
        }
        .build(cx, |cx| {
            // Outer circle (knob body) with indicator line inside
            ZStack::new(cx, |cx| {
                // Background circle
                Element::new(cx)
                    .class("knob-body")
                    .width(Pixels(theme::KNOB_SIZE))
                    .height(Pixels(theme::KNOB_SIZE))
                    .background_color(theme::WIDGET_BG)
                    .border_width(Pixels(2.0))
                    .border_color(theme::WIDGET_BORDER)
                    .corner_radius(Percentage(50.0));

                // Indicator line - positioned to rotate around knob center
                // The line is 20px tall, positioned so its bottom is at knob center
                Element::new(cx)
                    .class("knob-indicator")
                    .width(Pixels(3.0))
                    .height(Pixels(20.0))
                    .background_color(theme::WIDGET_ACCENT)
                    .corner_radius(Pixels(1.5))
                    .left(Pixels(theme::KNOB_SIZE / 2.0 - 1.5))
                    .top(Pixels(theme::KNOB_SIZE / 2.0 - 20.0))
                    .translate((Pixels(0.0), Pixels(10.0)))
                    .rotate(Knob::normalized_value.map(|v| Angle::Deg(v * 270.0 - 135.0)));
            })
            .width(Pixels(theme::KNOB_SIZE))
            .height(Pixels(theme::KNOB_SIZE));
        })
        .width(Pixels(theme::KNOB_SIZE))
        .height(Pixels(theme::KNOB_SIZE))
    }

    /// Builder method to add a label above the knob
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
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

/// Helper function to create a labeled knob (label + knob in VStack)
pub fn param_knob(
    cx: &mut Context,
    param_id: u32,
    label: &str,
    initial_normalized: f32,
    default_normalized: f32,
) {
    VStack::new(cx, move |cx| {
        // Label at top
        Label::new(cx, label)
            .font_size(11.0)
            // .background_color(theme::BG_DARK)
            .color(theme::TEXT_SECONDARY)
            .width(Pixels(theme::KNOB_CELL_WIDTH))
            .height(Pixels(theme::LABEL_HEIGHT))
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
        .width(Pixels(theme::KNOB_SIZE))
        .height(Pixels(theme::KNOB_SIZE));
    })
    .width(Pixels(theme::KNOB_CELL_WIDTH))
    .height(Pixels(theme::LABEL_HEIGHT + 4.0 + theme::KNOB_SIZE)) // label + gap + knob
    .gap(Pixels(4.0));
}
