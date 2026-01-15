// Core effects: distortion, chorus, delay, reverb

use super::super::helpers::{current_normalized, default_normalized, effect_header};
use crate::gui::widgets::{distortion_type_button, param_knob, tempo_sync_button};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_distortion_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        effect_header(cx, PARAM_DISTORTION_ENABLED, "Distortion");

        HStack::new(cx, |cx| {
            let drive_v = current_normalized(cx, PARAM_DISTORTION_DRIVE);
            let mix_v = current_normalized(cx, PARAM_DISTORTION_MIX);

            distortion_type_button(cx, PARAM_DISTORTION_TYPE);
            param_knob(
                cx,
                PARAM_DISTORTION_DRIVE,
                "Drive",
                drive_v,
                default_normalized(PARAM_DISTORTION_DRIVE),
            );
            param_knob(
                cx,
                PARAM_DISTORTION_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_DISTORTION_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_chorus_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        effect_header(cx, PARAM_CHORUS_ENABLED, "Chorus");

        HStack::new(cx, |cx| {
            let rate_v = current_normalized(cx, PARAM_CHORUS_RATE);
            let depth_v = current_normalized(cx, PARAM_CHORUS_DEPTH);
            let mix_v = current_normalized(cx, PARAM_CHORUS_MIX);

            param_knob(
                cx,
                PARAM_CHORUS_RATE,
                "Rate",
                rate_v,
                default_normalized(PARAM_CHORUS_RATE),
            );
            tempo_sync_button(cx, PARAM_CHORUS_TEMPO_SYNC);
            param_knob(
                cx,
                PARAM_CHORUS_DEPTH,
                "Depth",
                depth_v,
                default_normalized(PARAM_CHORUS_DEPTH),
            );
            param_knob(
                cx,
                PARAM_CHORUS_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_CHORUS_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_delay_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        effect_header(cx, PARAM_DELAY_ENABLED, "Delay");

        HStack::new(cx, |cx| {
            let time_v = current_normalized(cx, PARAM_DELAY_TIME_MS);
            let fb_v = current_normalized(cx, PARAM_DELAY_FEEDBACK);
            let wet_v = current_normalized(cx, PARAM_DELAY_WET);
            let dry_v = current_normalized(cx, PARAM_DELAY_DRY);

            param_knob(
                cx,
                PARAM_DELAY_TIME_MS,
                "Time",
                time_v,
                default_normalized(PARAM_DELAY_TIME_MS),
            );
            param_knob(
                cx,
                PARAM_DELAY_FEEDBACK,
                "FB",
                fb_v,
                default_normalized(PARAM_DELAY_FEEDBACK),
            );
            param_knob(
                cx,
                PARAM_DELAY_WET,
                "Wet",
                wet_v,
                default_normalized(PARAM_DELAY_WET),
            );
            param_knob(
                cx,
                PARAM_DELAY_DRY,
                "Dry",
                dry_v,
                default_normalized(PARAM_DELAY_DRY),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .height(Units::Auto)
    .gap(Pixels(6.0));
}

pub fn build_reverb_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        effect_header(cx, PARAM_REVERB_ENABLED, "Reverb");

        HStack::new(cx, |cx| {
            let room_v = current_normalized(cx, PARAM_REVERB_ROOM_SIZE);
            let damp_v = current_normalized(cx, PARAM_REVERB_DAMPING);
            let wet_v = current_normalized(cx, PARAM_REVERB_WET);
            let dry_v = current_normalized(cx, PARAM_REVERB_DRY);
            let width_v = current_normalized(cx, PARAM_REVERB_WIDTH);

            param_knob(
                cx,
                PARAM_REVERB_ROOM_SIZE,
                "Room",
                room_v,
                default_normalized(PARAM_REVERB_ROOM_SIZE),
            );
            param_knob(
                cx,
                PARAM_REVERB_DAMPING,
                "Damp",
                damp_v,
                default_normalized(PARAM_REVERB_DAMPING),
            );
            param_knob(
                cx,
                PARAM_REVERB_WET,
                "Wet",
                wet_v,
                default_normalized(PARAM_REVERB_WET),
            );
            param_knob(
                cx,
                PARAM_REVERB_DRY,
                "Dry",
                dry_v,
                default_normalized(PARAM_REVERB_DRY),
            );
            param_knob(
                cx,
                PARAM_REVERB_WIDTH,
                "Width",
                width_v,
                default_normalized(PARAM_REVERB_WIDTH),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}
