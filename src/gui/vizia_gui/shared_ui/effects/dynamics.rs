// Dynamics effects: compressor

use super::super::helpers::{current_normalized, default_normalized};
use crate::gui::vizia_gui::widgets::{param_checkbox, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_compressor_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Compressor")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_COMPRESSOR_ENABLED);
            param_checkbox(cx, PARAM_COMPRESSOR_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0))
        .height(Units::Auto);

        HStack::new(cx, |cx| {
            let thresh_v = current_normalized(cx, PARAM_COMPRESSOR_THRESHOLD);
            let ratio_v = current_normalized(cx, PARAM_COMPRESSOR_RATIO);
            let attack_v = current_normalized(cx, PARAM_COMPRESSOR_ATTACK);
            let release_v = current_normalized(cx, PARAM_COMPRESSOR_RELEASE);

            param_knob(
                cx,
                PARAM_COMPRESSOR_THRESHOLD,
                "Thresh",
                thresh_v,
                default_normalized(PARAM_COMPRESSOR_THRESHOLD),
            );
            param_knob(
                cx,
                PARAM_COMPRESSOR_RATIO,
                "Ratio",
                ratio_v,
                default_normalized(PARAM_COMPRESSOR_RATIO),
            );
            param_knob(
                cx,
                PARAM_COMPRESSOR_ATTACK,
                "Attack",
                attack_v,
                default_normalized(PARAM_COMPRESSOR_ATTACK),
            );
            param_knob(
                cx,
                PARAM_COMPRESSOR_RELEASE,
                "Release",
                release_v,
                default_normalized(PARAM_COMPRESSOR_RELEASE),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}
