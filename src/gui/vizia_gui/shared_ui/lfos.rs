// LFO sections with modulation routing

use super::helpers::{current_normalized, default_normalized};
use crate::gui::vizia_gui::widgets::{lfo_waveform_button, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_lfo_section(cx: &mut Context, lfo_index: usize) {
    let (wf, rate, depth, filt, pitch, gain, pan, pwm) = match lfo_index {
        1 => (
            PARAM_LFO1_WAVEFORM,
            PARAM_LFO1_RATE,
            PARAM_LFO1_DEPTH,
            PARAM_LFO1_FILTER_AMOUNT,
            PARAM_LFO1_PITCH_AMOUNT,
            PARAM_LFO1_GAIN_AMOUNT,
            PARAM_LFO1_PAN_AMOUNT,
            PARAM_LFO1_PWM_AMOUNT,
        ),
        2 => (
            PARAM_LFO2_WAVEFORM,
            PARAM_LFO2_RATE,
            PARAM_LFO2_DEPTH,
            PARAM_LFO2_FILTER_AMOUNT,
            PARAM_LFO2_PITCH_AMOUNT,
            PARAM_LFO2_GAIN_AMOUNT,
            PARAM_LFO2_PAN_AMOUNT,
            PARAM_LFO2_PWM_AMOUNT,
        ),
        _ => (
            PARAM_LFO3_WAVEFORM,
            PARAM_LFO3_RATE,
            PARAM_LFO3_DEPTH,
            PARAM_LFO3_FILTER_AMOUNT,
            PARAM_LFO3_PITCH_AMOUNT,
            PARAM_LFO3_GAIN_AMOUNT,
            PARAM_LFO3_PAN_AMOUNT,
            PARAM_LFO3_PWM_AMOUNT,
        ),
    };

    VStack::new(cx, move |cx| {
        HStack::new(cx, move |cx| {
            Label::new(cx, &format!("LFO {}", lfo_index))
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210));
            lfo_waveform_button(cx, wf, lfo_index - 1);
        })
        .height(Units::Auto)
        .gap(Pixels(10.0));

        HStack::new(cx, move |cx| {
            let rate_v = current_normalized(cx, rate);
            let depth_v = current_normalized(cx, depth);
            let filt_v = current_normalized(cx, filt);

            // lfo_waveform_button(cx, wf, lfo_index - 1);
            param_knob(cx, rate, "Rate", rate_v, default_normalized(rate));
            param_knob(cx, depth, "Depth", depth_v, default_normalized(depth));
            param_knob(cx, filt, "Filter", filt_v, default_normalized(filt));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));

        HStack::new(cx, move |cx| {
            let pitch_v = current_normalized(cx, pitch);
            let gain_v = current_normalized(cx, gain);
            let pan_v = current_normalized(cx, pan);
            let pwm_v = current_normalized(cx, pwm);

            param_knob(cx, pitch, "Pitch", pitch_v, default_normalized(pitch));
            param_knob(cx, gain, "Gain", gain_v, default_normalized(gain));
            param_knob(cx, pan, "Pan", pan_v, default_normalized(pan));
            param_knob(cx, pwm, "PWM", pwm_v, default_normalized(pwm));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .height(Pixels(350.0))
    .gap(Pixels(10.0));
}
