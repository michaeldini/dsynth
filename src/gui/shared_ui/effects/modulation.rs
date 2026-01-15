// Modulation effects: phaser, flanger, tremolo, auto-pan

use super::super::helpers::{current_normalized, default_normalized, effect_header};
use crate::gui::widgets::{param_knob, tempo_sync_button};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_phaser_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        effect_header(cx, PARAM_PHASER_ENABLED, "Phaser");

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
            tempo_sync_button(cx, PARAM_PHASER_TEMPO_SYNC);
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
        effect_header(cx, PARAM_FLANGER_ENABLED, "Flanger");

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
            tempo_sync_button(cx, PARAM_FLANGER_TEMPO_SYNC);
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
        effect_header(cx, PARAM_TREMOLO_ENABLED, "Tremolo");

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
            tempo_sync_button(cx, PARAM_TREMOLO_TEMPO_SYNC);
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
        effect_header(cx, PARAM_AUTOPAN_ENABLED, "Auto-Pan");

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
            tempo_sync_button(cx, PARAM_AUTOPAN_TEMPO_SYNC);
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
