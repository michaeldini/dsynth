// Filter and pitch effects: comb filter, ring modulator

use super::super::helpers::{current_normalized, default_normalized};
use crate::gui::vizia_gui::widgets::{param_checkbox, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_combfilter_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Comb Filter")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_COMB_ENABLED);
            param_checkbox(cx, PARAM_COMB_ENABLED, "On", enabled > 0.5);
        })
        .height(Units::Auto)
        .gap(Pixels(8.0));

        HStack::new(cx, |cx| {
            let freq_v = current_normalized(cx, PARAM_COMB_FREQUENCY);
            let fb_v = current_normalized(cx, PARAM_COMB_FEEDBACK);
            let mix_v = current_normalized(cx, PARAM_COMB_MIX);

            param_knob(
                cx,
                PARAM_COMB_FREQUENCY,
                "Freq",
                freq_v,
                default_normalized(PARAM_COMB_FREQUENCY),
            );
            param_knob(
                cx,
                PARAM_COMB_FEEDBACK,
                "FB",
                fb_v,
                default_normalized(PARAM_COMB_FEEDBACK),
            );
            param_knob(
                cx,
                PARAM_COMB_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_COMB_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_ringmod_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Ring Mod")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_RINGMOD_ENABLED);
            param_checkbox(cx, PARAM_RINGMOD_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0))
        .height(Units::Auto);

        HStack::new(cx, |cx| {
            let freq_v = current_normalized(cx, PARAM_RINGMOD_FREQUENCY);
            let depth_v = current_normalized(cx, PARAM_RINGMOD_DEPTH);

            param_knob(
                cx,
                PARAM_RINGMOD_FREQUENCY,
                "Freq",
                freq_v,
                default_normalized(PARAM_RINGMOD_FREQUENCY),
            );
            param_knob(
                cx,
                PARAM_RINGMOD_DEPTH,
                "Depth",
                depth_v,
                default_normalized(PARAM_RINGMOD_DEPTH),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}
