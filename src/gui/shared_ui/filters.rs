// Filter sections with envelope controls

use super::helpers::{current_normalized, default_normalized};
use super::traits::{IndexedSection, ParameterLayout};
use crate::gui::widgets::{filter_type_button, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

/// Parameter set for a filter instance
pub struct FilterParams {
    pub filter_type: u32,
    pub cutoff: u32,
    pub resonance: u32,
    pub bandwidth: u32,
    pub key_tracking: u32,
    pub drive: u32,
    pub post_drive: u32,
    pub env_amount: u32,
    pub env_attack: u32,
    pub env_decay: u32,
    pub env_sustain: u32,
    pub env_release: u32,
}

/// Filter UI section builder
pub struct FilterSection;

impl IndexedSection for FilterSection {
    type Params = FilterParams;

    fn get_params(&self, index: usize) -> Self::Params {
        match index {
            1 => FilterParams {
                filter_type: PARAM_FILTER1_TYPE,
                cutoff: PARAM_FILTER1_CUTOFF,
                resonance: PARAM_FILTER1_RESONANCE,
                bandwidth: PARAM_FILTER1_BANDWIDTH,
                key_tracking: PARAM_FILTER1_KEY_TRACKING,
                drive: PARAM_FILTER1_DRIVE,
                post_drive: PARAM_FILTER1_POST_DRIVE,
                env_amount: PARAM_FILTER1_ENV_AMOUNT,
                env_attack: PARAM_FILTER1_ENV_ATTACK,
                env_decay: PARAM_FILTER1_ENV_DECAY,
                env_sustain: PARAM_FILTER1_ENV_SUSTAIN,
                env_release: PARAM_FILTER1_ENV_RELEASE,
            },
            2 => FilterParams {
                filter_type: PARAM_FILTER2_TYPE,
                cutoff: PARAM_FILTER2_CUTOFF,
                resonance: PARAM_FILTER2_RESONANCE,
                bandwidth: PARAM_FILTER2_BANDWIDTH,
                key_tracking: PARAM_FILTER2_KEY_TRACKING,
                drive: PARAM_FILTER2_DRIVE,
                post_drive: PARAM_FILTER2_POST_DRIVE,
                env_amount: PARAM_FILTER2_ENV_AMOUNT,
                env_attack: PARAM_FILTER2_ENV_ATTACK,
                env_decay: PARAM_FILTER2_ENV_DECAY,
                env_sustain: PARAM_FILTER2_ENV_SUSTAIN,
                env_release: PARAM_FILTER2_ENV_RELEASE,
            },
            _ => FilterParams {
                filter_type: PARAM_FILTER3_TYPE,
                cutoff: PARAM_FILTER3_CUTOFF,
                resonance: PARAM_FILTER3_RESONANCE,
                bandwidth: PARAM_FILTER3_BANDWIDTH,
                key_tracking: PARAM_FILTER3_KEY_TRACKING,
                drive: PARAM_FILTER3_DRIVE,
                post_drive: PARAM_FILTER3_POST_DRIVE,
                env_amount: PARAM_FILTER3_ENV_AMOUNT,
                env_attack: PARAM_FILTER3_ENV_ATTACK,
                env_decay: PARAM_FILTER3_ENV_DECAY,
                env_sustain: PARAM_FILTER3_ENV_SUSTAIN,
                env_release: PARAM_FILTER3_ENV_RELEASE,
            },
        }
    }

    fn build(&self, cx: &mut Context, index: usize) {
        let p = self.get_params(index);

        VStack::new(cx, |cx| {
            // Header
            HStack::new(cx, |cx| {
                Label::new(cx, &format!("Filter {}", index))
                    .font_size(14.0)
                    .color(Color::rgb(200, 200, 210));
                filter_type_button(cx, p.filter_type, index - 1);
            })
            .height(Units::Auto)
            .gap(Pixels(6.0));

            // Main parameters
            Self::build_param_row(cx, |cx| {
                let cutoff_v = current_normalized(cx, p.cutoff);
                let resonance_v = current_normalized(cx, p.resonance);
                let bandwidth_v = current_normalized(cx, p.bandwidth);
                let key_tracking_v = current_normalized(cx, p.key_tracking);
                let drive_v = current_normalized(cx, p.drive);
                let post_drive_v = current_normalized(cx, p.post_drive);

                param_knob(
                    cx,
                    p.cutoff,
                    "Cutoff",
                    cutoff_v,
                    default_normalized(p.cutoff),
                );
                param_knob(
                    cx,
                    p.resonance,
                    "Res",
                    resonance_v,
                    default_normalized(p.resonance),
                );
                param_knob(
                    cx,
                    p.bandwidth,
                    "BW",
                    bandwidth_v,
                    default_normalized(p.bandwidth),
                );
                param_knob(
                    cx,
                    p.key_tracking,
                    "KeyTrk",
                    key_tracking_v,
                    default_normalized(p.key_tracking),
                );
                param_knob(cx, p.drive, "Drive", drive_v, default_normalized(p.drive));
                param_knob(
                    cx,
                    p.post_drive,
                    "PostDrv",
                    post_drive_v,
                    default_normalized(p.post_drive),
                );
            });

            // Filter Envelope Section
            Label::new(cx, "Filter Envelope")
                .font_size(12.0)
                .color(Color::rgb(180, 180, 190))
                .top(Pixels(4.0));

            Self::build_param_row(cx, |cx| {
                let env_amount_v = current_normalized(cx, p.env_amount);
                let env_attack_v = current_normalized(cx, p.env_attack);
                let env_decay_v = current_normalized(cx, p.env_decay);
                let env_sustain_v = current_normalized(cx, p.env_sustain);
                let env_release_v = current_normalized(cx, p.env_release);

                param_knob(
                    cx,
                    p.env_amount,
                    "Amt",
                    env_amount_v,
                    default_normalized(p.env_amount),
                );
                param_knob(
                    cx,
                    p.env_attack,
                    "A",
                    env_attack_v,
                    default_normalized(p.env_attack),
                );
                param_knob(
                    cx,
                    p.env_decay,
                    "D",
                    env_decay_v,
                    default_normalized(p.env_decay),
                );
                param_knob(
                    cx,
                    p.env_sustain,
                    "S",
                    env_sustain_v,
                    default_normalized(p.env_sustain),
                );
                param_knob(
                    cx,
                    p.env_release,
                    "R",
                    env_release_v,
                    default_normalized(p.env_release),
                );
            });
        })
        .gap(Pixels(12.0));
    }

    fn section_name(&self) -> &'static str {
        "Filter"
    }
}

// Public API for backward compatibility

// Public API for backward compatibility
pub fn build_filter_section(cx: &mut Context, filter_index: usize) {
    FilterSection.build(cx, filter_index);
}
