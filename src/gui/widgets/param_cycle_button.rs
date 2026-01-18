use crate::gui::theme;
use vizia::prelude::*;

#[derive(Clone, Copy, Debug)]
enum EnumPopupEvent {
    Toggle,
    Select(usize),
}

#[derive(Lens)]
struct EnumPopupButton {
    param_id: u32,
    label: String,
    options: &'static [&'static str],
    is_open: bool,
}

impl EnumPopupButton {
    fn new<'a>(
        cx: &'a mut Context,
        param_id: u32,
        label: impl Into<String>,
        options: &'static [&'static str],
    ) -> Handle<'a, Self> {
        const BUTTON_HEIGHT: f32 = 24.0;
        const POPUP_MAX_WIDTH: f32 = 300.0;
        const POPUP_ROW_HEIGHT: f32 = 22.0;
        const POPUP_ROW_GAP: f32 = 4.0;
        const POPUP_COL_GAP: f32 = 8.0;
        const POPUP_PADDING: f32 = 8.0;
        const POPUP_ROWS_PER_COL: usize = 8;
        const POPUP_GAP: f32 = 6.0;
        const OPTION_FONT_SIZE: f32 = 12.0;
        const OPTION_CHAR_WIDTH: f32 = 7.0;
        const OPTION_HORIZONTAL_PADDING: f32 = 10.0;

        Self {
            param_id,
            label: label.into(),
            options,
            is_open: false,
        }
        .build(cx, move |cx| {
            let param_id = EnumPopupButton::param_id.get(cx);
            let options = EnumPopupButton::options.get(cx);
            let label_text = EnumPopupButton::label.get(cx);

            VStack::new(cx, move |cx| {
                Label::new(cx, label_text)
                    .font_size(11.0)
                    .color(theme::TEXT_SECONDARY)
                    .width(Pixels(theme::CYCLE_BUTTON_CELL_WIDTH))
                    .height(Pixels(theme::LABEL_HEIGHT))
                    .text_align(TextAlign::Center)
                    .text_wrap(false)
                    .text_overflow(TextOverflow::Ellipsis);

                ZStack::new(cx, move |cx| {
                    Button::new(cx, move |cx| {
                        let current_value =
                            crate::gui::GuiState::synth_params.map(move |params_arc| {
                                let params = params_arc.read();
                                crate::plugin::param_update::param_get::get_param(&params, param_id)
                            });

                        let option_text = current_value.map(move |v| {
                            let index = (*v).round() as usize;
                            let clamped_index = index.min(options.len().saturating_sub(1));
                            options[clamped_index]
                        });

                        Label::new(cx, option_text)
                            .font_size(12.0)
                            .color(theme::TEXT_BRIGHT)
                            .text_align(TextAlign::Center)
                            .text_wrap(false)
                            .text_overflow(TextOverflow::Ellipsis)
                    })
                    .on_press(move |cx| {
                        cx.emit(EnumPopupEvent::Toggle);
                    })
                    .width(Pixels(theme::CYCLE_BUTTON_CELL_WIDTH))
                    .height(Pixels(BUTTON_HEIGHT))
                    .background_color(theme::WIDGET_BG)
                    .border_width(Pixels(1.0))
                    .border_color(theme::WIDGET_BORDER)
                    .corner_radius(Pixels(4.0))
                    .cursor(CursorIcon::Hand);

                    Binding::new(cx, EnumPopupButton::is_open, move |cx, is_open| {
                        if !is_open.get(cx) {
                            return;
                        }

                        let num_options = options.len();
                        let num_cols = num_options.div_ceil(POPUP_ROWS_PER_COL);

                        let max_len =
                            options.iter().map(|s| s.chars().count()).max().unwrap_or(0) as f32;
                        let column_width = (max_len * OPTION_CHAR_WIDTH
                            + OPTION_HORIZONTAL_PADDING * 2.0)
                            .min(POPUP_MAX_WIDTH);

                        let rows_in_col = num_options.min(POPUP_ROWS_PER_COL) as f32;
                        let panel_width = (column_width * num_cols as f32)
                            + POPUP_COL_GAP * (num_cols.saturating_sub(1)) as f32
                            + POPUP_PADDING * 2.0;
                        let row_gaps = if rows_in_col > 1.0 {
                            rows_in_col - 1.0
                        } else {
                            0.0
                        };
                        let panel_height = (rows_in_col * POPUP_ROW_HEIGHT)
                            + POPUP_ROW_GAP * row_gaps
                            + POPUP_PADDING * 2.0;

                        Popup::new(cx, move |cx| {
                            ZStack::new(cx, move |cx| {
                                Element::new(cx)
                                    .width(Pixels(panel_width))
                                    .height(Pixels(panel_height))
                                    .background_color(Color::rgba(0, 0, 0, 140))
                                    .corner_radius(Pixels(6.0))
                                    .translate((Pixels(3.0), Pixels(3.0)));

                                VStack::new(cx, move |cx| {
                                    HStack::new(cx, move |cx| {
                                        for col in 0..num_cols {
                                            VStack::new(cx, move |cx| {
                                                for row in 0..POPUP_ROWS_PER_COL {
                                                    let option_index =
                                                        col * POPUP_ROWS_PER_COL + row;
                                                    if option_index >= num_options {
                                                        break;
                                                    }

                                                    let option_label = options[option_index];

                                                    Button::new(cx, move |cx| {
                                                        Label::new(cx, option_label)
                                                            .font_size(OPTION_FONT_SIZE)
                                                            .color(theme::TEXT_PRIMARY)
                                                            .text_align(TextAlign::Left)
                                                            .text_wrap(false)
                                                            .text_overflow(TextOverflow::Ellipsis)
                                                            .width(Stretch(1.0))
                                                    })
                                                    .on_press(move |cx| {
                                                        cx.emit(EnumPopupEvent::Select(
                                                            option_index,
                                                        ));
                                                    })
                                                    .width(Pixels(column_width))
                                                    .height(Pixels(POPUP_ROW_HEIGHT))
                                                    .background_color(theme::BG_SECTION)
                                                    .border_width(Pixels(1.0))
                                                    .border_color(theme::WIDGET_BORDER)
                                                    .corner_radius(Pixels(4.0))
                                                    .cursor(CursorIcon::Hand);
                                                }
                                            })
                                            .width(Pixels(column_width))
                                            .gap(Pixels(POPUP_ROW_GAP));
                                        }
                                    })
                                    .gap(Pixels(POPUP_COL_GAP));
                                })
                                .padding(Pixels(POPUP_PADDING))
                                .width(Pixels(panel_width))
                                .height(Pixels(panel_height))
                                .background_color(theme::BG_PANEL)
                                .border_width(Pixels(1.0))
                                .border_color(theme::ACTIVE_BORDER)
                                .corner_radius(Pixels(6.0));
                            })
                            .width(Pixels(panel_width))
                            .height(Pixels(panel_height));
                        })
                        .show_arrow(false)
                        .arrow_size(Pixels(POPUP_GAP))
                        .placement(Placement::Bottom)
                        .z_index(200);
                    });
                })
                .width(Pixels(theme::CYCLE_BUTTON_CELL_WIDTH))
                .height(Pixels(BUTTON_HEIGHT));
            })
            .width(Pixels(theme::CYCLE_BUTTON_CELL_WIDTH))
            .height(Pixels(theme::LABEL_HEIGHT + 4.0 + BUTTON_HEIGHT))
            .gap(Pixels(4.0));
        })
        .width(Pixels(theme::CYCLE_BUTTON_CELL_WIDTH))
        .height(Pixels(theme::LABEL_HEIGHT + 4.0 + BUTTON_HEIGHT))
    }
}

impl View for EnumPopupButton {
    fn element(&self) -> Option<&'static str> {
        Some("enum-popup-button")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|popup_msg: &EnumPopupEvent, meta| match *popup_msg {
            EnumPopupEvent::Toggle => {
                self.is_open = true;
                meta.consume();
            }
            EnumPopupEvent::Select(index) => {
                let num_options = self.options.len().max(1);
                let normalized_value = if num_options > 1 {
                    index as f32 / (num_options - 1) as f32
                } else {
                    0.0
                };

                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.param_id,
                    normalized_value,
                ));
                self.is_open = false;
                meta.consume();
            }
        });
    }
}

fn param_enum_popup_button(
    cx: &mut Context,
    param_id: u32,
    label: &str,
    options: &'static [&'static str],
) {
    EnumPopupButton::new(cx, param_id, label, options);
}

/// A simple cycling button widget for enum parameters
///
/// IMPORTANT: The parameter system uses DENORMALIZED values for enums.
/// get_param() returns the enum index directly (0, 1, 2, ...) NOT normalized 0.0-1.0
/// We need to apply the parameter and receive denormalized values as indices.
pub fn param_cycle_button(
    cx: &mut Context,
    param_id: u32,
    label: &str,
    options: &'static [&'static str],
) {
    use crate::gui::{GuiMessage, GuiState};

    const CELL_WIDTH: f32 = 80.0;
    let num_options = options.len();

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

        // Button that shows current selection and cycles when clicked
        Button::new(cx, move |cx| {
            // Get current parameter value from the GUI state
            // get_param returns DENORMALIZED value (enum index: 0, 1, 2, ...)
            let current_value = GuiState::synth_params.map(move |params_arc| {
                let params = params_arc.read();
                crate::plugin::param_update::param_get::get_param(&params, param_id)
            });

            // The denormalized value IS the index directly
            let option_text = current_value.map(move |v| {
                let index = (*v).round() as usize;
                let clamped_index = index.min(num_options - 1);
                options[clamped_index]
            });

            Label::new(cx, option_text)
                .font_size(12.0)
                .color(Color::rgb(240, 240, 240))
                .text_align(TextAlign::Center)
        })
        .on_press(move |cx| {
            // Get current state and cycle to next option
            let arc = GuiState::synth_params.get(cx);
            let current_value = {
                let params = arc.read();
                crate::plugin::param_update::param_get::get_param(&params, param_id)
            };

            // current_value is the denormalized index (0, 1, 2, ...)
            let current_index = current_value.round() as usize;
            let next_index = (current_index + 1) % num_options;

            // Convert to normalized value (0.0-1.0) for the ParamChanged message
            // Normalized = index / (num_options - 1)
            let normalized_value = if num_options > 1 {
                next_index as f32 / (num_options - 1) as f32
            } else {
                0.0
            };

            cx.emit(GuiMessage::ParamChanged(param_id, normalized_value));
        })
        .width(Pixels(CELL_WIDTH))
        .height(Pixels(24.0))
        .background_color(Color::rgb(60, 60, 65))
        .border_width(Pixels(1.0))
        .border_color(Color::rgb(100, 100, 110));
    })
    .width(Pixels(CELL_WIDTH))
    .height(Pixels(60.0))
    .gap(Pixels(4.0));
}

// Helper function for oscillator waveforms (8 options in registry)
pub fn oscillator_waveform_button(cx: &mut Context, param_id: u32, _osc_index: usize) {
    const OPTIONS: &[&str] = &[
        "Sine",
        "Saw",
        "Square",
        "Triangle",
        "Pulse",
        "White",
        "Pink",
        "Add",
        "Wavetable",
    ];
    param_enum_popup_button(cx, param_id, "Waveform", OPTIONS);
}

// Helper function for filter types
pub fn filter_type_button(cx: &mut Context, param_id: u32, _filter_index: usize) {
    const OPTIONS: &[&str] = &["Lowpass", "Highpass", "Bandpass"];
    param_enum_popup_button(cx, param_id, "Filter Type", OPTIONS);
}

// Helper function for FM source
pub fn fm_source_button(cx: &mut Context, param_id: u32, _osc_index: usize) {
    const OPTIONS: &[&str] = &["None", "Osc1", "Osc2", "Osc3"];
    param_enum_popup_button(cx, param_id, "FM Source", OPTIONS);
}

// Helper function for LFO waveforms (order from denorm_to_lfo_waveform)
pub fn lfo_waveform_button(cx: &mut Context, param_id: u32, _lfo_index: usize) {
    const OPTIONS: &[&str] = &["Sine", "Triangle", "Square", "Saw"];
    param_enum_popup_button(cx, param_id, "LFO Wave", OPTIONS);
}

// Helper function for distortion types (order from DistortionType enum)
pub fn distortion_type_button(cx: &mut Context, param_id: u32) {
    const OPTIONS: &[&str] = &[
        "Tanh",
        "SoftClip",
        "HardClip",
        "Cubic",
        "Foldback",
        "Asymmetric",
        "SineShaper",
        "Bitcrush",
        "Diode",
    ];
    param_enum_popup_button(cx, param_id, "Type", OPTIONS);
}

// Helper function for tempo sync modes (order from TempoSync enum)
pub fn tempo_sync_button(cx: &mut Context, param_id: u32) {
    const OPTIONS: &[&str] = &[
        "Hz", "1/1", "1/2", "1/4", "1/8", "1/16", "1/32", "1/4T", "1/8T", "1/16T", "1/4D", "1/8D",
        "1/16D",
    ];
    param_enum_popup_button(cx, param_id, "Sync", OPTIONS);
}
