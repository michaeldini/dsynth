// Lo-fi effects: bitcrusher, waveshaper, exciter

use super::super::helpers::{current_normalized, default_normalized};
use crate::gui::widgets::{param_checkbox, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_bitcrusher_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Bitcrusher")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_BITCRUSHER_ENABLED);
            param_checkbox(cx, PARAM_BITCRUSHER_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0))
        .height(Units::Auto);

        HStack::new(cx, |cx| {
            let rate_v = current_normalized(cx, PARAM_BITCRUSHER_RATE);
            let bits_v = current_normalized(cx, PARAM_BITCRUSHER_BITS);

            param_knob(
                cx,
                PARAM_BITCRUSHER_RATE,
                "Rate",
                rate_v,
                default_normalized(PARAM_BITCRUSHER_RATE),
            );
            param_knob(
                cx,
                PARAM_BITCRUSHER_BITS,
                "Bits",
                bits_v,
                default_normalized(PARAM_BITCRUSHER_BITS),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_waveshaper_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Waveshaper")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_WAVESHAPER_ENABLED);
            param_checkbox(cx, PARAM_WAVESHAPER_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0))
        .height(Units::Auto);

        HStack::new(cx, |cx| {
            let drive_v = current_normalized(cx, PARAM_WAVESHAPER_DRIVE);
            let mix_v = current_normalized(cx, PARAM_WAVESHAPER_MIX);

            param_knob(
                cx,
                PARAM_WAVESHAPER_DRIVE,
                "Drive",
                drive_v,
                default_normalized(PARAM_WAVESHAPER_DRIVE),
            );
            param_knob(
                cx,
                PARAM_WAVESHAPER_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_WAVESHAPER_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_exciter_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Exciter")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_EXCITER_ENABLED);
            param_checkbox(cx, PARAM_EXCITER_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0))
        .height(Units::Auto);

        HStack::new(cx, |cx| {
            let freq_v = current_normalized(cx, PARAM_EXCITER_FREQUENCY);
            let drive_v = current_normalized(cx, PARAM_EXCITER_DRIVE);
            let mix_v = current_normalized(cx, PARAM_EXCITER_MIX);

            param_knob(
                cx,
                PARAM_EXCITER_FREQUENCY,
                "Freq",
                freq_v,
                default_normalized(PARAM_EXCITER_FREQUENCY),
            );
            param_knob(
                cx,
                PARAM_EXCITER_DRIVE,
                "Drive",
                drive_v,
                default_normalized(PARAM_EXCITER_DRIVE),
            );
            param_knob(
                cx,
                PARAM_EXCITER_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_EXCITER_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}
