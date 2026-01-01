// Multiband effects: multiband distortion, stereo widener

use super::super::helpers::{current_normalized, default_normalized};
use crate::gui::widgets::{param_checkbox, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_multiband_distortion_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Multiband Distortion")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_MB_DIST_ENABLED);
            param_checkbox(cx, PARAM_MB_DIST_ENABLED, "On", enabled > 0.5);
        })
        .height(Units::Auto)
        .gap(Pixels(8.0));

        // Crossover frequencies
        HStack::new(cx, |cx| {
            let low_mid_v = current_normalized(cx, PARAM_MB_DIST_LOW_MID_FREQ);
            let mid_high_v = current_normalized(cx, PARAM_MB_DIST_MID_HIGH_FREQ);
            let mix_v = current_normalized(cx, PARAM_MB_DIST_MIX);

            param_knob(
                cx,
                PARAM_MB_DIST_LOW_MID_FREQ,
                "Lo/Mid",
                low_mid_v,
                default_normalized(PARAM_MB_DIST_LOW_MID_FREQ),
            );
            param_knob(
                cx,
                PARAM_MB_DIST_MID_HIGH_FREQ,
                "Mid/Hi",
                mid_high_v,
                default_normalized(PARAM_MB_DIST_MID_HIGH_FREQ),
            );
            param_knob(
                cx,
                PARAM_MB_DIST_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_MB_DIST_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));

        // Per-band drive and gain
        HStack::new(cx, |cx| {
            let drive_low_v = current_normalized(cx, PARAM_MB_DIST_DRIVE_LOW);
            let drive_mid_v = current_normalized(cx, PARAM_MB_DIST_DRIVE_MID);
            let drive_high_v = current_normalized(cx, PARAM_MB_DIST_DRIVE_HIGH);
            let gain_low_v = current_normalized(cx, PARAM_MB_DIST_GAIN_LOW);
            let gain_mid_v = current_normalized(cx, PARAM_MB_DIST_GAIN_MID);
            let gain_high_v = current_normalized(cx, PARAM_MB_DIST_GAIN_HIGH);

            param_knob(
                cx,
                PARAM_MB_DIST_DRIVE_LOW,
                "DrLo",
                drive_low_v,
                default_normalized(PARAM_MB_DIST_DRIVE_LOW),
            );
            param_knob(
                cx,
                PARAM_MB_DIST_DRIVE_MID,
                "DrMid",
                drive_mid_v,
                default_normalized(PARAM_MB_DIST_DRIVE_MID),
            );
            param_knob(
                cx,
                PARAM_MB_DIST_DRIVE_HIGH,
                "DrHi",
                drive_high_v,
                default_normalized(PARAM_MB_DIST_DRIVE_HIGH),
            );
            param_knob(
                cx,
                PARAM_MB_DIST_GAIN_LOW,
                "GnLo",
                gain_low_v,
                default_normalized(PARAM_MB_DIST_GAIN_LOW),
            );
            param_knob(
                cx,
                PARAM_MB_DIST_GAIN_MID,
                "GnMid",
                gain_mid_v,
                default_normalized(PARAM_MB_DIST_GAIN_MID),
            );
            param_knob(
                cx,
                PARAM_MB_DIST_GAIN_HIGH,
                "GnHi",
                gain_high_v,
                default_normalized(PARAM_MB_DIST_GAIN_HIGH),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_stereo_widener_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Stereo Widener")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_WIDENER_ENABLED);
            param_checkbox(cx, PARAM_WIDENER_ENABLED, "On", enabled > 0.5);
        })
        .height(Units::Auto)
        .gap(Pixels(8.0));

        HStack::new(cx, |cx| {
            let haas_delay_v = current_normalized(cx, PARAM_WIDENER_HAAS_DELAY);
            let haas_mix_v = current_normalized(cx, PARAM_WIDENER_HAAS_MIX);
            let width_v = current_normalized(cx, PARAM_WIDENER_WIDTH);
            let mid_gain_v = current_normalized(cx, PARAM_WIDENER_MID_GAIN);
            let side_gain_v = current_normalized(cx, PARAM_WIDENER_SIDE_GAIN);

            param_knob(
                cx,
                PARAM_WIDENER_HAAS_DELAY,
                "Haas",
                haas_delay_v,
                default_normalized(PARAM_WIDENER_HAAS_DELAY),
            );
            param_knob(
                cx,
                PARAM_WIDENER_HAAS_MIX,
                "HMix",
                haas_mix_v,
                default_normalized(PARAM_WIDENER_HAAS_MIX),
            );
            param_knob(
                cx,
                PARAM_WIDENER_WIDTH,
                "Width",
                width_v,
                default_normalized(PARAM_WIDENER_WIDTH),
            );
            param_knob(
                cx,
                PARAM_WIDENER_MID_GAIN,
                "Mid",
                mid_gain_v,
                default_normalized(PARAM_WIDENER_MID_GAIN),
            );
            param_knob(
                cx,
                PARAM_WIDENER_SIDE_GAIN,
                "Side",
                side_gain_v,
                default_normalized(PARAM_WIDENER_SIDE_GAIN),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}
