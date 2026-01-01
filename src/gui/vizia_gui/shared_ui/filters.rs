// Filter sections with envelope controls

use super::helpers::{current_normalized, default_normalized};
use crate::gui::vizia_gui::widgets::{filter_type_button, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

pub fn build_filter_section(cx: &mut Context, filter_index: usize) {
    let (ft, cutoff, res, bw, kt, drive, post_drive, env_amt, env_att, env_dec, env_sus, env_rel) =
        match filter_index {
            1 => (
                PARAM_FILTER1_TYPE,
                PARAM_FILTER1_CUTOFF,
                PARAM_FILTER1_RESONANCE,
                PARAM_FILTER1_BANDWIDTH,
                PARAM_FILTER1_KEY_TRACKING,
                PARAM_FILTER1_DRIVE,
                PARAM_FILTER1_POST_DRIVE,
                PARAM_FILTER1_ENV_AMOUNT,
                PARAM_FILTER1_ENV_ATTACK,
                PARAM_FILTER1_ENV_DECAY,
                PARAM_FILTER1_ENV_SUSTAIN,
                PARAM_FILTER1_ENV_RELEASE,
            ),
            2 => (
                PARAM_FILTER2_TYPE,
                PARAM_FILTER2_CUTOFF,
                PARAM_FILTER2_RESONANCE,
                PARAM_FILTER2_BANDWIDTH,
                PARAM_FILTER2_KEY_TRACKING,
                PARAM_FILTER2_DRIVE,
                PARAM_FILTER2_POST_DRIVE,
                PARAM_FILTER2_ENV_AMOUNT,
                PARAM_FILTER2_ENV_ATTACK,
                PARAM_FILTER2_ENV_DECAY,
                PARAM_FILTER2_ENV_SUSTAIN,
                PARAM_FILTER2_ENV_RELEASE,
            ),
            _ => (
                PARAM_FILTER3_TYPE,
                PARAM_FILTER3_CUTOFF,
                PARAM_FILTER3_RESONANCE,
                PARAM_FILTER3_BANDWIDTH,
                PARAM_FILTER3_KEY_TRACKING,
                PARAM_FILTER3_DRIVE,
                PARAM_FILTER3_POST_DRIVE,
                PARAM_FILTER3_ENV_AMOUNT,
                PARAM_FILTER3_ENV_ATTACK,
                PARAM_FILTER3_ENV_DECAY,
                PARAM_FILTER3_ENV_SUSTAIN,
                PARAM_FILTER3_ENV_RELEASE,
            ),
        };

    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, &format!("Filter {}", filter_index))
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210));
            filter_type_button(cx, ft, filter_index - 1);
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));

        HStack::new(cx, |cx| {
            let cutoff_v = current_normalized(cx, cutoff);
            let res_v = current_normalized(cx, res);
            let bw_v = current_normalized(cx, bw);
            let kt_v = current_normalized(cx, kt);
            let drive_v = current_normalized(cx, drive);
            let post_drive_v = current_normalized(cx, post_drive);

            param_knob(cx, cutoff, "Cutoff", cutoff_v, default_normalized(cutoff));
            param_knob(cx, res, "Res", res_v, default_normalized(res));
            param_knob(cx, bw, "BW", bw_v, default_normalized(bw));
            param_knob(cx, kt, "KeyTrk", kt_v, default_normalized(kt));
            param_knob(cx, drive, "Drive", drive_v, default_normalized(drive));
            param_knob(
                cx,
                post_drive,
                "PostDrv",
                post_drive_v,
                default_normalized(post_drive),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));

        // Filter Envelope Section
        Label::new(cx, "Filter Envelope")
            .font_size(12.0)
            .color(Color::rgb(180, 180, 190))
            .top(Pixels(4.0));

        HStack::new(cx, |cx| {
            let env_amt_v = current_normalized(cx, env_amt);
            let env_att_v = current_normalized(cx, env_att);
            let env_dec_v = current_normalized(cx, env_dec);
            let env_sus_v = current_normalized(cx, env_sus);
            let env_rel_v = current_normalized(cx, env_rel);

            param_knob(cx, env_amt, "Amt", env_amt_v, default_normalized(env_amt));
            param_knob(cx, env_att, "A", env_att_v, default_normalized(env_att));
            param_knob(cx, env_dec, "D", env_dec_v, default_normalized(env_dec));
            param_knob(cx, env_sus, "S", env_sus_v, default_normalized(env_sus));
            param_knob(cx, env_rel, "R", env_rel_v, default_normalized(env_rel));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(12.0));
}
