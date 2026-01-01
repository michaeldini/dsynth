use vizia::prelude::*;

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
                if let Ok(params) = params_arc.read() {
                    crate::plugin::param_update::param_get::get_param(&params, param_id)
                } else {
                    0.0
                }
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
            let current_value = if let Ok(params) = arc.read() {
                crate::plugin::param_update::param_get::get_param(&params, param_id)
            } else {
                0.0
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
        "Sine", "Saw", "Square", "Triangle", "Pulse", "White", "Pink", "Add", "Wavetable",
    ];
    param_cycle_button(cx, param_id, "Waveform", OPTIONS);
}

// Helper function for filter types
pub fn filter_type_button(cx: &mut Context, param_id: u32, _filter_index: usize) {
    const OPTIONS: &[&str] = &["Lowpass", "Highpass", "Bandpass"];
    param_cycle_button(cx, param_id, "Filter Type", OPTIONS);
}

// Helper function for FM source
pub fn fm_source_button(cx: &mut Context, param_id: u32, _osc_index: usize) {
    const OPTIONS: &[&str] = &["None", "Osc1", "Osc2", "Osc3"];
    param_cycle_button(cx, param_id, "FM Source", OPTIONS);
}

// Helper function for LFO waveforms (order from denorm_to_lfo_waveform)
pub fn lfo_waveform_button(cx: &mut Context, param_id: u32, _lfo_index: usize) {
    const OPTIONS: &[&str] = &["Sine", "Triangle", "Square", "Saw"];
    param_cycle_button(cx, param_id, "LFO Wave", OPTIONS);
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
    param_cycle_button(cx, param_id, "Type", OPTIONS);
}
