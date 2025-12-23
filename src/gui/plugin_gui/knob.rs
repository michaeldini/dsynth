/// Knob Widget Component
///
/// A reusable rotary knob control built on top of iced's Slider widget.
/// This component provides:
/// - Visual knob-style control with value display
/// - Parameter ID binding for CLAP integration
/// - Unit labels (Hz, dB, %, etc.)
/// - Value formatting with precision control
/// - Optional double-click to reset to default
///
/// Design philosophy:
/// - Uses native iced::widget::Slider (vertical) as the base
/// - Wraps it with labels, value display, and proper spacing
/// - Returns Message::ParamChanged with normalized 0-1 values
/// - Normalization happens in the closure passed to the knob
use iced_baseview::widget::{Column, Container, Slider, Text};
use iced_baseview::{Element, Length};

/// Knob widget builder
#[derive(Debug, Clone)]
pub struct Knob {
    /// Parameter name to display above knob
    label: String,
    /// Current value (in parameter's native range, not normalized)
    value: f32,
    /// Minimum value (native range)
    min: f32,
    /// Maximum value (native range)
    max: f32,
    /// Unit to display (Hz, dB, %, st, etc.)
    unit: Option<String>,
    /// Number of decimal places for value display
    precision: usize,
    /// Default value for double-click reset (native range)
    default: Option<f32>,
    /// Width of the knob control
    width: f32,
}

impl Knob {
    /// Create a new knob
    pub fn new(label: impl Into<String>, value: f32, min: f32, max: f32) -> Self {
        Self {
            label: label.into(),
            value,
            min,
            max,
            unit: None,
            precision: 2,
            default: None,
            width: 80.0,
        }
    }

    /// Set the unit to display after the value
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the number of decimal places
    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Set the default value for reset
    pub fn default(mut self, default: f32) -> Self {
        self.default = Some(default);
        self
    }

    /// Set the width of the knob
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Build the knob widget
    ///
    /// The `on_change` closure receives the **native value** (not normalized)
    /// and should return a Message. The caller is responsible for normalization.
    pub fn view<'a, Message: 'a + Clone>(
        self,
        on_change: impl Fn(f32) -> Message + 'a,
    ) -> Element<'a, Message> {
        let value_text = if let Some(unit) = &self.unit {
            format!("{:.prec$} {}", self.value, unit, prec = self.precision)
        } else {
            format!("{:.prec$}", self.value, prec = self.precision)
        };

        // Build the knob column: label, slider, value
        let knob_column = Column::new()
            .spacing(4)
            .align_x(iced_baseview::Alignment::Center)
            .push(
                // Label
                Text::new(self.label.clone())
                    .size(12)
                    .width(Length::Fixed(self.width)),
            )
            .push(
                // Vertical slider (will look like knob with styling)
                Slider::new(self.min..=self.max, self.value, on_change)
                    .width(Length::Fixed(self.width))
                    .step(0.001), // Fine-grained control
            )
            .push(
                // Value display
                Text::new(value_text)
                    .size(11)
                    .width(Length::Fixed(self.width)),
            );

        Container::new(knob_column)
            .width(Length::Fixed(self.width + 10.0))
            .center_x(Length::Fill)
            .into()
    }
}

/// Helper function to create a linear knob (0-1 range parameters)
pub fn linear_knob<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    value: f32,
    param_id: u32,
    unit: Option<&str>,
    precision: usize,
    on_change: impl Fn(u32, f32) -> Message + 'a,
) -> Element<'a, Message> {
    let mut knob = Knob::new(label, value, 0.0, 1.0).precision(precision);
    if let Some(u) = unit {
        knob = knob.unit(u);
    }
    knob.view(move |v| on_change(param_id, v))
}

/// Helper function to create a logarithmic frequency knob (20-20000 Hz)
pub fn frequency_knob<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    value: f32,
    param_id: u32,
    on_change: impl Fn(u32, f32) -> Message + 'a,
) -> Element<'a, Message> {
    Knob::new(label, value, 20.0, 20000.0)
        .unit("Hz")
        .precision(0)
        .view(move |v| {
            // Logarithmic normalization for frequency
            let norm = (v.ln() - 20.0f32.ln()) / (20000.0f32.ln() - 20.0f32.ln());
            on_change(param_id, norm)
        })
}

/// Helper function to create a pitch knob (-24 to +24 semitones)
pub fn pitch_knob<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    value: f32,
    param_id: u32,
    on_change: impl Fn(u32, f32) -> Message + 'a,
) -> Element<'a, Message> {
    Knob::new(label, value, -24.0, 24.0)
        .unit("st")
        .precision(1)
        .default(0.0)
        .view(move |v| {
            // Linear normalization: -24..24 → 0..1
            let norm = (v + 24.0) / 48.0;
            on_change(param_id, norm)
        })
}

/// Helper function to create a detune knob (-50 to +50 cents)
pub fn detune_knob<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    value: f32,
    param_id: u32,
    on_change: impl Fn(u32, f32) -> Message + 'a,
) -> Element<'a, Message> {
    Knob::new(label, value, -50.0, 50.0)
        .unit("¢")
        .precision(1)
        .default(0.0)
        .view(move |v| {
            // Linear normalization: -50..50 → 0..1
            let norm = (v + 50.0) / 100.0;
            on_change(param_id, norm)
        })
}

/// Helper function to create a time knob (0.001 to 5.0 seconds, logarithmic)
pub fn time_knob<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    value: f32,
    param_id: u32,
    on_change: impl Fn(u32, f32) -> Message + 'a,
) -> Element<'a, Message> {
    Knob::new(label, value, 0.001, 5.0)
        .unit("s")
        .precision(3)
        .view(move |v| {
            // Logarithmic normalization for time
            let norm = (v.ln() - 0.001f32.ln()) / (5.0f32.ln() - 0.001f32.ln());
            on_change(param_id, norm)
        })
}

/// Helper function to create a percentage knob (0-100%)
pub fn percent_knob<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    value: f32, // Expects 0.0-1.0 internal value
    param_id: u32,
    on_change: impl Fn(u32, f32) -> Message + 'a,
) -> Element<'a, Message> {
    Knob::new(label, value * 100.0, 0.0, 100.0)
        .unit("%")
        .precision(1)
        .view(move |v| on_change(param_id, v / 100.0))
}
