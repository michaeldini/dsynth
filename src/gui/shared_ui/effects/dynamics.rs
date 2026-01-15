// Dynamics effects: compressor

use super::super::helpers::{current_normalized, default_normalized, effect_header};
use crate::gui::widgets::{param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_compressor_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        effect_header(cx, PARAM_COMPRESSOR_ENABLED, "Compressor");

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
