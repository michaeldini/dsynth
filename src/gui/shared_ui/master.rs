// Master, envelope, and velocity sections

use super::helpers::{current_normalized, default_normalized};
use crate::gui::widgets::{param_checkbox, param_knob, EnvelopeEditor};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_master_section(cx: &mut Context) {
    HStack::new(cx, |cx| {
        let gain = current_normalized(cx, PARAM_MASTER_GAIN);
        let gain_def = default_normalized(PARAM_MASTER_GAIN);
        let mono = current_normalized(cx, PARAM_MONOPHONIC);
        let hard_sync = current_normalized(cx, PARAM_HARD_SYNC);
        let amp_v = current_normalized(cx, PARAM_VELOCITY_AMP);
        let filter_v = current_normalized(cx, PARAM_VELOCITY_FILTER);

        param_knob(cx, PARAM_MASTER_GAIN, "Gain", gain, gain_def);
        param_checkbox(cx, PARAM_MONOPHONIC, "Mono", mono > 0.5);
        param_checkbox(cx, PARAM_HARD_SYNC, "Hard Sync", hard_sync > 0.5);
        param_knob(
            cx,
            PARAM_VELOCITY_AMP,
            "Vel→Amp",
            amp_v,
            default_normalized(PARAM_VELOCITY_AMP),
        );
        param_knob(
            cx,
            PARAM_VELOCITY_FILTER,
            "Vel→Flt",
            filter_v,
            default_normalized(PARAM_VELOCITY_FILTER),
        );

        // Randomize button
        Button::new(cx, |cx| Label::new(cx, "🎲 Randomize"))
            .on_press(|cx| cx.emit(crate::gui::GuiMessage::Randomize))
            .width(Pixels(100.0))
            .height(Pixels(32.0))
            .background_color(Color::rgb(60, 60, 70))
            .corner_radius(Pixels(4.0))
            .cursor(CursorIcon::Hand);
    })
    .gap(Pixels(6.0));
}

pub fn build_envelope_section(cx: &mut Context) {
    HStack::new(cx, |cx| {
        let attack = current_normalized(cx, PARAM_ENVELOPE_ATTACK);
        let decay = current_normalized(cx, PARAM_ENVELOPE_DECAY);
        let sustain = current_normalized(cx, PARAM_ENVELOPE_SUSTAIN);
        let release = current_normalized(cx, PARAM_ENVELOPE_RELEASE);
        let attack_curve_norm = current_normalized(cx, PARAM_ENVELOPE_ATTACK_CURVE);
        let decay_curve_norm = current_normalized(cx, PARAM_ENVELOPE_DECAY_CURVE);
        let release_curve_norm = current_normalized(cx, PARAM_ENVELOPE_RELEASE_CURVE);

        // Convert normalized curve values (0.0-1.0) to actual curve range (-1.0 to +1.0)
        // ONLY for EnvelopeEditor visualization, NOT for the knobs
        let attack_curve = attack_curve_norm * 2.0 - 1.0;
        let decay_curve = decay_curve_norm * 2.0 - 1.0;
        let release_curve = release_curve_norm * 2.0 - 1.0;

        // Visual envelope editor
        EnvelopeEditor::new(
            cx,
            attack,
            decay,
            sustain,
            release,
            attack_curve,
            decay_curve,
            release_curve,
            PARAM_ENVELOPE_ATTACK,
            PARAM_ENVELOPE_DECAY,
            PARAM_ENVELOPE_SUSTAIN,
            PARAM_ENVELOPE_RELEASE,
            PARAM_ENVELOPE_ATTACK_CURVE,
            PARAM_ENVELOPE_DECAY_CURVE,
            PARAM_ENVELOPE_RELEASE_CURVE,
        )
        .background_color(Color::rgb(25, 25, 30))
        .border_width(Pixels(1.0))
        .border_color(Color::rgb(60, 60, 70))
        .corner_radius(Pixels(4.0));

        // Curve control knobs
        HStack::new(cx, |cx| {
            param_knob(
                cx,
                PARAM_ENVELOPE_ATTACK_CURVE,
                "Atk Curve",
                attack_curve_norm, // Use normalized value for knob
                default_normalized(PARAM_ENVELOPE_ATTACK_CURVE),
            );
            param_knob(
                cx,
                PARAM_ENVELOPE_DECAY_CURVE,
                "Dec Curve",
                decay_curve_norm, // Use normalized value for knob
                default_normalized(PARAM_ENVELOPE_DECAY_CURVE),
            );
            param_knob(
                cx,
                PARAM_ENVELOPE_RELEASE_CURVE,
                "Rel Curve",
                release_curve_norm, // Use normalized value for knob
                default_normalized(PARAM_ENVELOPE_RELEASE_CURVE),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(8.0));
}

pub fn build_velocity_section(cx: &mut Context) {
    HStack::new(cx, |cx| {
        let amp_v = current_normalized(cx, PARAM_VELOCITY_AMP);
        let filter_v = current_normalized(cx, PARAM_VELOCITY_FILTER);

        param_knob(
            cx,
            PARAM_VELOCITY_AMP,
            "Amp",
            amp_v,
            default_normalized(PARAM_VELOCITY_AMP),
        );
        param_knob(
            cx,
            PARAM_VELOCITY_FILTER,
            "Filter",
            filter_v,
            default_normalized(PARAM_VELOCITY_FILTER),
        );
    })
    .height(Units::Auto)
    .gap(Pixels(6.0));
}
