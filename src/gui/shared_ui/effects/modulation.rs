// Modulation effects: phaser, flanger, tremolo, auto-pan

use super::super::helpers::{current_normalized, default_normalized};
use crate::gui::widgets::{param_checkbox, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_phaser_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Phaser")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_PHASER_ENABLED);
            param_checkbox(cx, PARAM_PHASER_ENABLED, "On", enabled > 0.5);
        })
        .height(Units::Auto)
        .gap(Pixels(8.0));

        HStack::new(cx, |cx| {
            let rate_v = current_normalized(cx, PARAM_PHASER_RATE);
            let depth_v = current_normalized(cx, PARAM_PHASER_DEPTH);
            let fb_v = current_normalized(cx, PARAM_PHASER_FEEDBACK);
            let mix_v = current_normalized(cx, PARAM_PHASER_MIX);

            param_knob(
                cx,
                PARAM_PHASER_RATE,
                "Rate",
                rate_v,
                default_normalized(PARAM_PHASER_RATE),
            );
            param_knob(
                cx,
                PARAM_PHASER_DEPTH,
                "Depth",
                depth_v,
                default_normalized(PARAM_PHASER_DEPTH),
            );
            param_knob(
                cx,
                PARAM_PHASER_FEEDBACK,
                "FB",
                fb_v,
                default_normalized(PARAM_PHASER_FEEDBACK),
            );
            param_knob(
                cx,
                PARAM_PHASER_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_PHASER_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .padding(Pixels(6.0))
    .gap(Pixels(6.0));
}

pub fn build_flanger_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Flanger")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_FLANGER_ENABLED);
            param_checkbox(cx, PARAM_FLANGER_ENABLED, "On", enabled > 0.5);
        })
        .height(Units::Auto)
        .gap(Pixels(8.0));

        HStack::new(cx, |cx| {
            let rate_v = current_normalized(cx, PARAM_FLANGER_RATE);
            let depth_v = current_normalized(cx, PARAM_FLANGER_DEPTH);
            let fb_v = current_normalized(cx, PARAM_FLANGER_FEEDBACK);
            let mix_v = current_normalized(cx, PARAM_FLANGER_MIX);

            param_knob(
                cx,
                PARAM_FLANGER_RATE,
                "Rate",
                rate_v,
                default_normalized(PARAM_FLANGER_RATE),
            );
            param_knob(
                cx,
                PARAM_FLANGER_DEPTH,
                "Depth",
                depth_v,
                default_normalized(PARAM_FLANGER_DEPTH),
            );
            param_knob(
                cx,
                PARAM_FLANGER_FEEDBACK,
                "FB",
                fb_v,
                default_normalized(PARAM_FLANGER_FEEDBACK),
            );
            param_knob(
                cx,
                PARAM_FLANGER_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_FLANGER_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_tremolo_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Tremolo")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_TREMOLO_ENABLED);
            param_checkbox(cx, PARAM_TREMOLO_ENABLED, "On", enabled > 0.5);
        })
        .height(Units::Auto)
        .gap(Pixels(8.0));

        HStack::new(cx, |cx| {
            let rate_v = current_normalized(cx, PARAM_TREMOLO_RATE);
            let depth_v = current_normalized(cx, PARAM_TREMOLO_DEPTH);

            param_knob(
                cx,
                PARAM_TREMOLO_RATE,
                "Rate",
                rate_v,
                default_normalized(PARAM_TREMOLO_RATE),
            );
            param_knob(
                cx,
                PARAM_TREMOLO_DEPTH,
                "Depth",
                depth_v,
                default_normalized(PARAM_TREMOLO_DEPTH),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_autopan_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Auto-Pan")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_AUTOPAN_ENABLED);
            param_checkbox(cx, PARAM_AUTOPAN_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0))
        .height(Units::Auto);

        HStack::new(cx, |cx| {
            let rate_v = current_normalized(cx, PARAM_AUTOPAN_RATE);
            let depth_v = current_normalized(cx, PARAM_AUTOPAN_DEPTH);

            param_knob(
                cx,
                PARAM_AUTOPAN_RATE,
                "Rate",
                rate_v,
                default_normalized(PARAM_AUTOPAN_RATE),
            );
            param_knob(
                cx,
                PARAM_AUTOPAN_DEPTH,
                "Depth",
                depth_v,
                default_normalized(PARAM_AUTOPAN_DEPTH),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}
