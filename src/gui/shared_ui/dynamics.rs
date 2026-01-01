// Voice dynamics: compressor and transient shaper

use super::helpers::{current_normalized, default_normalized};
use crate::gui::widgets::{param_checkbox, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_voice_compressor_section(cx: &mut Context) {
    HStack::new(cx, |cx| {
        let enabled = current_normalized(cx, PARAM_VOICE_COMP_ENABLED);
        let threshold = current_normalized(cx, PARAM_VOICE_COMP_THRESHOLD);
        let ratio = current_normalized(cx, PARAM_VOICE_COMP_RATIO);
        let attack = current_normalized(cx, PARAM_VOICE_COMP_ATTACK);
        let release = current_normalized(cx, PARAM_VOICE_COMP_RELEASE);
        let knee = current_normalized(cx, PARAM_VOICE_COMP_KNEE);
        let makeup = current_normalized(cx, PARAM_VOICE_COMP_MAKEUP);

        param_checkbox(cx, PARAM_VOICE_COMP_ENABLED, "On", enabled > 0.5);
        param_knob(
            cx,
            PARAM_VOICE_COMP_THRESHOLD,
            "Thresh",
            threshold,
            default_normalized(PARAM_VOICE_COMP_THRESHOLD),
        );
        param_knob(
            cx,
            PARAM_VOICE_COMP_RATIO,
            "Ratio",
            ratio,
            default_normalized(PARAM_VOICE_COMP_RATIO),
        );
        param_knob(
            cx,
            PARAM_VOICE_COMP_ATTACK,
            "Attack",
            attack,
            default_normalized(PARAM_VOICE_COMP_ATTACK),
        );
        param_knob(
            cx,
            PARAM_VOICE_COMP_RELEASE,
            "Release",
            release,
            default_normalized(PARAM_VOICE_COMP_RELEASE),
        );
        param_knob(
            cx,
            PARAM_VOICE_COMP_KNEE,
            "Knee",
            knee,
            default_normalized(PARAM_VOICE_COMP_KNEE),
        );
        param_knob(
            cx,
            PARAM_VOICE_COMP_MAKEUP,
            "Makeup",
            makeup,
            default_normalized(PARAM_VOICE_COMP_MAKEUP),
        );
    })
    .height(Units::Auto)
    .gap(Pixels(6.0));
}

pub fn build_transient_shaper_section(cx: &mut Context) {
    HStack::new(cx, |cx| {
        let enabled = current_normalized(cx, PARAM_TRANSIENT_ENABLED);
        let attack = current_normalized(cx, PARAM_TRANSIENT_ATTACK);
        let sustain = current_normalized(cx, PARAM_TRANSIENT_SUSTAIN);

        param_checkbox(cx, PARAM_TRANSIENT_ENABLED, "On", enabled > 0.5);
        param_knob(
            cx,
            PARAM_TRANSIENT_ATTACK,
            "Attack",
            attack,
            default_normalized(PARAM_TRANSIENT_ATTACK),
        );
        param_knob(
            cx,
            PARAM_TRANSIENT_SUSTAIN,
            "Sustain",
            sustain,
            default_normalized(PARAM_TRANSIENT_SUSTAIN),
        );
    })
    .height(Units::Auto)
    .gap(Pixels(6.0));
}
