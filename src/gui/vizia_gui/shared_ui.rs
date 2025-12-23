// Shared UI layout for both CLAP plugin and standalone VIZIA GUI

use crate::gui::vizia_gui::GuiState;
use crate::gui::vizia_gui::widgets::{
    distortion_type_dropdown, filter_type_dropdown, fm_source_dropdown, lfo_waveform_dropdown,
    oscillator_waveform_dropdown, param_checkbox, param_knob,
};
use crate::plugin::param_descriptor::*;
use crate::plugin::param_registry;
use crate::plugin::param_update::param_get;
use vizia::prelude::*;

/// Build the main UI layout - shared by plugin and standalone
pub fn build_ui(cx: &mut Context) {
    const LEFT_COL_WIDTH: f32 = 300.0;
    const OSC_COL_WIDTH: f32 = 360.0;
    const ROW_GAP: f32 = 12.0;
    const COL_GAP: f32 = 12.0;

    VStack::new(cx, |cx| {
        // Title bar + live status text
        HStack::new(cx, |cx| {
            Label::new(cx, "DSynth - VIZIA GUI")
                .font_size(20.0)
                .color(Color::rgb(220, 220, 230));

            Label::new(cx, GuiState::last_param_text)
                .font_size(12.0)
                .color(Color::rgb(180, 180, 190))
                .width(Stretch(1.0))
                .text_align(TextAlign::Right)
                .text_wrap(false)
                .text_overflow(TextOverflow::Ellipsis);
        })
        .height(Pixels(50.0))
        .width(Stretch(1.0))
        .padding(Pixels(10.0))
        .background_color(Color::rgb(25, 25, 30));

        // Scrollable content area
        ScrollView::new(cx, |cx| {
            VStack::new(cx, |cx| {
                // Row 1: Master + Envelope + Velocity
                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Master")
                            .font_size(16.0)
                            .color(Color::rgb(200, 200, 210))
                            .height(Pixels(24.0));
                        build_master_section(cx);
                    })
                    .width(Pixels(LEFT_COL_WIDTH))
                    .padding(Pixels(10.0))
                    .gap(Pixels(6.0))
                    .background_color(Color::rgb(35, 35, 40));

                    VStack::new(cx, |cx| {
                        Label::new(cx, "Envelope")
                            .font_size(16.0)
                            .color(Color::rgb(200, 200, 210))
                            .height(Pixels(24.0));
                        build_envelope_section(cx);
                    })
                    .width(Stretch(1.0))
                    .padding(Pixels(10.0))
                    .gap(Pixels(6.0))
                    .background_color(Color::rgb(35, 35, 40));

                    VStack::new(cx, |cx| {
                        Label::new(cx, "Velocity")
                            .font_size(16.0)
                            .color(Color::rgb(200, 200, 210))
                            .height(Pixels(24.0));
                        build_velocity_section(cx);
                    })
                    .width(Stretch(1.0))
                    .padding(Pixels(10.0))
                    .gap(Pixels(6.0))
                    .background_color(Color::rgb(35, 35, 40));
                })
                .gap(Pixels(COL_GAP))
                .height(Pixels(125.0));

                // Row 2: Oscillators
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| build_osc_section(cx, 1))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Pixels(0.0))
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                        VStack::new(cx, |cx| build_osc_section(cx, 2))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Pixels(0.0))
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                        VStack::new(cx, |cx| build_osc_section(cx, 3))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Pixels(0.0))
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                    })
                    .height(Pixels(0.0))
                    .gap(Pixels(COL_GAP));
                })
                .padding(Pixels(10.0))
                .gap(Pixels(10.0))
                .height(Pixels(250.0));

                // Row 3: Harmonics
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| build_additive_osc_section(cx, 1))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                        VStack::new(cx, |cx| build_additive_osc_section(cx, 2))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                        VStack::new(cx, |cx| build_additive_osc_section(cx, 3))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                    })
                    .height(Units::Auto)
                    .gap(Pixels(COL_GAP));
                })
                .padding(Pixels(10.0))
                .gap(Pixels(10.0))
                .background_color(Color::rgb(35, 35, 40))
                .height(Pixels(250.0));

                // Row 4: Filters
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| build_filter_section(cx, 1))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                        VStack::new(cx, |cx| build_filter_section(cx, 2))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                        VStack::new(cx, |cx| build_filter_section(cx, 3))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                    })
                    .height(Units::Auto)
                    .gap(Pixels(COL_GAP));
                })
                .padding(Pixels(10.0))
                .gap(Pixels(10.0))
                .background_color(Color::rgb(35, 35, 40))
                .height(Pixels(150.0));

                // Row 5: LFOs
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| build_lfo_section(cx, 1))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                        VStack::new(cx, |cx| build_lfo_section(cx, 2))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                        VStack::new(cx, |cx| build_lfo_section(cx, 3))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(12.0))
                            .gap(Pixels(12.0));
                    })
                    .height(Units::Auto)
                    .gap(Pixels(COL_GAP));
                })
                .padding(Pixels(10.0))
                .gap(Pixels(10.0))
                .background_color(Color::rgb(35, 35, 40))
                .height(Pixels(250.0));

                // Row 6: Effects
                build_effects_section(cx);
            })
            .width(Stretch(1.0))
            .height(Units::Auto)
            .min_height(Pixels(0.0))
            .padding(Pixels(10.0))
            .gap(Pixels(ROW_GAP));
        })
        .show_horizontal_scrollbar(false)
        .show_vertical_scrollbar(true)
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .background_color(Color::rgb(30, 30, 35));
    })
    .width(Stretch(1.0))
    .height(Stretch(1.0))
    .background_color(Color::rgb(30, 30, 35));
}

pub fn build_master_section(cx: &mut Context) {
    HStack::new(cx, |cx| {
        let gain = current_normalized(cx, PARAM_MASTER_GAIN);
        let gain_def = default_normalized(PARAM_MASTER_GAIN);
        let mono = current_normalized(cx, PARAM_MONOPHONIC);

        param_knob(cx, PARAM_MASTER_GAIN, "Gain", gain, gain_def);
        param_checkbox(cx, PARAM_MONOPHONIC, "Mono", mono > 0.5);
    })
    .gap(Pixels(6.0));
}

pub fn build_envelope_section(cx: &mut Context) {
    HStack::new(cx, |cx| {
        let attack = current_normalized(cx, PARAM_ENVELOPE_ATTACK);
        let decay = current_normalized(cx, PARAM_ENVELOPE_DECAY);
        let sustain = current_normalized(cx, PARAM_ENVELOPE_SUSTAIN);
        let release = current_normalized(cx, PARAM_ENVELOPE_RELEASE);

        let attack_def = default_normalized(PARAM_ENVELOPE_ATTACK);
        let decay_def = default_normalized(PARAM_ENVELOPE_DECAY);
        let sustain_def = default_normalized(PARAM_ENVELOPE_SUSTAIN);
        let release_def = default_normalized(PARAM_ENVELOPE_RELEASE);

        param_knob(cx, PARAM_ENVELOPE_ATTACK, "Attack", attack, attack_def);
        param_knob(cx, PARAM_ENVELOPE_DECAY, "Decay", decay, decay_def);
        param_knob(cx, PARAM_ENVELOPE_SUSTAIN, "Sustain", sustain, sustain_def);
        param_knob(cx, PARAM_ENVELOPE_RELEASE, "Release", release, release_def);
    })
    .height(Pixels(100.0))
    .gap(Pixels(6.0));
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

pub fn build_osc_section(cx: &mut Context, osc_index: usize) {
    let (wf, pitch, detune, gain, pan, unison, unison_detune, phase, shape, fm_src, fm_amt) =
        match osc_index {
            1 => (
                PARAM_OSC1_WAVEFORM,
                PARAM_OSC1_PITCH,
                PARAM_OSC1_DETUNE,
                PARAM_OSC1_GAIN,
                PARAM_OSC1_PAN,
                PARAM_OSC1_UNISON,
                PARAM_OSC1_UNISON_DETUNE,
                PARAM_OSC1_PHASE,
                PARAM_OSC1_SHAPE,
                PARAM_OSC1_FM_SOURCE,
                PARAM_OSC1_FM_AMOUNT,
            ),
            2 => (
                PARAM_OSC2_WAVEFORM,
                PARAM_OSC2_PITCH,
                PARAM_OSC2_DETUNE,
                PARAM_OSC2_GAIN,
                PARAM_OSC2_PAN,
                PARAM_OSC2_UNISON,
                PARAM_OSC2_UNISON_DETUNE,
                PARAM_OSC2_PHASE,
                PARAM_OSC2_SHAPE,
                PARAM_OSC2_FM_SOURCE,
                PARAM_OSC2_FM_AMOUNT,
            ),
            _ => (
                PARAM_OSC3_WAVEFORM,
                PARAM_OSC3_PITCH,
                PARAM_OSC3_DETUNE,
                PARAM_OSC3_GAIN,
                PARAM_OSC3_PAN,
                PARAM_OSC3_UNISON,
                PARAM_OSC3_UNISON_DETUNE,
                PARAM_OSC3_PHASE,
                PARAM_OSC3_SHAPE,
                PARAM_OSC3_FM_SOURCE,
                PARAM_OSC3_FM_AMOUNT,
            ),
        };

    VStack::new(cx, |cx| {
        Label::new(cx, &format!("Osc {}", osc_index))
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210));

        HStack::new(cx, |cx| {
            let pitch_v = current_normalized(cx, pitch);
            let detune_v = current_normalized(cx, detune);
            let gain_v = current_normalized(cx, gain);
            let pan_v = current_normalized(cx, pan);

            oscillator_waveform_dropdown(cx, wf, osc_index - 1);
            param_knob(cx, pitch, "Pitch", pitch_v, default_normalized(pitch));
            param_knob(cx, detune, "Detune", detune_v, default_normalized(detune));
            param_knob(cx, gain, "Gain", gain_v, default_normalized(gain));
            param_knob(cx, pan, "Pan", pan_v, default_normalized(pan));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));

        HStack::new(cx, |cx| {
            let unison_v = current_normalized(cx, unison);
            let unison_detune_v = current_normalized(cx, unison_detune);
            let _phase_v = current_normalized(cx, phase);
            let shape_v = current_normalized(cx, shape);
            let fm_amt_v = current_normalized(cx, fm_amt);

            fm_source_dropdown(cx, fm_src, osc_index - 1);
            param_knob(cx, fm_amt, "FM Amt", fm_amt_v, default_normalized(fm_amt));
            param_knob(cx, unison, "Unison", unison_v, default_normalized(unison));
            param_knob(
                cx,
                unison_detune,
                "UDet",
                unison_detune_v,
                default_normalized(unison_detune),
            );
            param_knob(cx, shape, "Shape", shape_v, default_normalized(shape));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(10.0));
}

pub fn build_additive_osc_section(cx: &mut Context, osc_index: usize) {
    let (h1, h2, h3, h4, h5, h6, h7, h8) = match osc_index {
        1 => (
            PARAM_OSC1_H1,
            PARAM_OSC1_H2,
            PARAM_OSC1_H3,
            PARAM_OSC1_H4,
            PARAM_OSC1_H5,
            PARAM_OSC1_H6,
            PARAM_OSC1_H7,
            PARAM_OSC1_H8,
        ),
        2 => (
            PARAM_OSC2_H1,
            PARAM_OSC2_H2,
            PARAM_OSC2_H3,
            PARAM_OSC2_H4,
            PARAM_OSC2_H5,
            PARAM_OSC2_H6,
            PARAM_OSC2_H7,
            PARAM_OSC2_H8,
        ),
        _ => (
            PARAM_OSC3_H1,
            PARAM_OSC3_H2,
            PARAM_OSC3_H3,
            PARAM_OSC3_H4,
            PARAM_OSC3_H5,
            PARAM_OSC3_H6,
            PARAM_OSC3_H7,
            PARAM_OSC3_H8,
        ),
    };

    VStack::new(cx, |cx| {
        Label::new(cx, &format!("Osc {} Harmonics", osc_index))
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(22.0));

        HStack::new(cx, |cx| {
            let h1_v = current_normalized(cx, h1);
            let h2_v = current_normalized(cx, h2);
            let h3_v = current_normalized(cx, h3);
            let h4_v = current_normalized(cx, h4);

            param_knob(cx, h1, "H1", h1_v, default_normalized(h1));
            param_knob(cx, h2, "H2", h2_v, default_normalized(h2));
            param_knob(cx, h3, "H3", h3_v, default_normalized(h3));
            param_knob(cx, h4, "H4", h4_v, default_normalized(h4));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));

        HStack::new(cx, |cx| {
            let h5_v = current_normalized(cx, h5);
            let h6_v = current_normalized(cx, h6);
            let h7_v = current_normalized(cx, h7);
            let h8_v = current_normalized(cx, h8);

            param_knob(cx, h5, "H5", h5_v, default_normalized(h5));
            param_knob(cx, h6, "H6", h6_v, default_normalized(h6));
            param_knob(cx, h7, "H7", h7_v, default_normalized(h7));
            param_knob(cx, h8, "H8", h8_v, default_normalized(h8));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(12.0));
}

pub fn build_filter_section(cx: &mut Context, filter_index: usize) {
    let (ft, cutoff, res, bw, kt) = match filter_index {
        1 => (
            PARAM_FILTER1_TYPE,
            PARAM_FILTER1_CUTOFF,
            PARAM_FILTER1_RESONANCE,
            PARAM_FILTER1_BANDWIDTH,
            PARAM_FILTER1_KEY_TRACKING,
        ),
        2 => (
            PARAM_FILTER2_TYPE,
            PARAM_FILTER2_CUTOFF,
            PARAM_FILTER2_RESONANCE,
            PARAM_FILTER2_BANDWIDTH,
            PARAM_FILTER2_KEY_TRACKING,
        ),
        _ => (
            PARAM_FILTER3_TYPE,
            PARAM_FILTER3_CUTOFF,
            PARAM_FILTER3_RESONANCE,
            PARAM_FILTER3_BANDWIDTH,
            PARAM_FILTER3_KEY_TRACKING,
        ),
    };

    VStack::new(cx, |cx| {
        Label::new(cx, &format!("Filter {}", filter_index))
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(22.0));

        HStack::new(cx, |cx| {
            let cutoff_v = current_normalized(cx, cutoff);
            let res_v = current_normalized(cx, res);
            let bw_v = current_normalized(cx, bw);
            let kt_v = current_normalized(cx, kt);

            filter_type_dropdown(cx, ft, filter_index - 1);
            param_knob(cx, cutoff, "Cutoff", cutoff_v, default_normalized(cutoff));
            param_knob(cx, res, "Res", res_v, default_normalized(res));
            param_knob(cx, bw, "BW", bw_v, default_normalized(bw));
            param_knob(cx, kt, "KeyTrk", kt_v, default_normalized(kt));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(12.0));
}

pub fn build_lfo_section(cx: &mut Context, lfo_index: usize) {
    let (wf, rate, depth, filt, pitch, gain, pan, pwm) = match lfo_index {
        1 => (
            PARAM_LFO1_WAVEFORM,
            PARAM_LFO1_RATE,
            PARAM_LFO1_DEPTH,
            PARAM_LFO1_FILTER_AMOUNT,
            PARAM_LFO1_PITCH_AMOUNT,
            PARAM_LFO1_GAIN_AMOUNT,
            PARAM_LFO1_PAN_AMOUNT,
            PARAM_LFO1_PWM_AMOUNT,
        ),
        2 => (
            PARAM_LFO2_WAVEFORM,
            PARAM_LFO2_RATE,
            PARAM_LFO2_DEPTH,
            PARAM_LFO2_FILTER_AMOUNT,
            PARAM_LFO2_PITCH_AMOUNT,
            PARAM_LFO2_GAIN_AMOUNT,
            PARAM_LFO2_PAN_AMOUNT,
            PARAM_LFO2_PWM_AMOUNT,
        ),
        _ => (
            PARAM_LFO3_WAVEFORM,
            PARAM_LFO3_RATE,
            PARAM_LFO3_DEPTH,
            PARAM_LFO3_FILTER_AMOUNT,
            PARAM_LFO3_PITCH_AMOUNT,
            PARAM_LFO3_GAIN_AMOUNT,
            PARAM_LFO3_PAN_AMOUNT,
            PARAM_LFO3_PWM_AMOUNT,
        ),
    };

    VStack::new(cx, move |cx| {
        Label::new(cx, &format!("LFO {}", lfo_index))
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(22.0));

        HStack::new(cx, move |cx| {
            let rate_v = current_normalized(cx, rate);
            let depth_v = current_normalized(cx, depth);
            let filt_v = current_normalized(cx, filt);

            lfo_waveform_dropdown(cx, wf, lfo_index - 1);
            param_knob(cx, rate, "Rate", rate_v, default_normalized(rate));
            param_knob(cx, depth, "Depth", depth_v, default_normalized(depth));
            param_knob(cx, filt, "Filter", filt_v, default_normalized(filt));
        })
        .height(Units::Auto)
        .gap(Pixels(18.0));

        HStack::new(cx, move |cx| {
            let pitch_v = current_normalized(cx, pitch);
            let gain_v = current_normalized(cx, gain);
            let pan_v = current_normalized(cx, pan);
            let pwm_v = current_normalized(cx, pwm);

            param_knob(cx, pitch, "Pitch", pitch_v, default_normalized(pitch));
            param_knob(cx, gain, "Gain", gain_v, default_normalized(gain));
            param_knob(cx, pan, "Pan", pan_v, default_normalized(pan));
            param_knob(cx, pwm, "PWM", pwm_v, default_normalized(pwm));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(18.0));
}

pub fn build_effects_section(cx: &mut Context) {
    const EFFECT_COL_WIDTH: f32 = 280.0;

    VStack::new(cx, |cx| {
        Label::new(cx, "Effects")
            .font_size(16.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(24.0));

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| build_distortion_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_chorus_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_delay_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_reverb_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
        })
        .height(Units::Auto)
        .gap(Pixels(12.0));
    })
    .width(Stretch(1.0))
    .height(Pixels(150.0))
    .background_color(Color::rgb(35, 35, 40))
    .padding(Pixels(10.0))
    .gap(Pixels(6.0));
}

pub fn build_distortion_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "Distortion")
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(22.0));

        HStack::new(cx, |cx| {
            let drive_v = current_normalized(cx, PARAM_DISTORTION_DRIVE);
            let mix_v = current_normalized(cx, PARAM_DISTORTION_MIX);

            distortion_type_dropdown(cx, PARAM_DISTORTION_TYPE);
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
        Label::new(cx, "Chorus")
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(22.0));

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
        Label::new(cx, "Delay")
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(22.0));

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
    .gap(Pixels(6.0));
}

pub fn build_reverb_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "Reverb")
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(22.0));

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

// Helper functions for parameter normalization

pub fn current_normalized(cx: &mut Context, param_id: u32) -> f32 {
    let arc = GuiState::synth_params.get(cx);
    let params = arc.read().unwrap();
    let denorm = param_get::get_param(&params, param_id);
    let registry = param_registry::get_registry();
    if let Some(desc) = registry.get(param_id) {
        match &desc.param_type {
            crate::plugin::param_descriptor::ParamType::Float { min, max, .. } => {
                if *max > *min {
                    ((denorm - *min) / (*max - *min)).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            crate::plugin::param_descriptor::ParamType::Bool => {
                if denorm > 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            crate::plugin::param_descriptor::ParamType::Enum { variants } => {
                if variants.len() > 1 {
                    (denorm / (variants.len() - 1) as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            crate::plugin::param_descriptor::ParamType::Int { min, max } => {
                let range = (*max - *min) as f32;
                if range > 0.0 {
                    ((denorm - *min as f32) / range).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
        }
    } else {
        0.0
    }
}

pub fn default_normalized(param_id: u32) -> f32 {
    let registry = param_registry::get_registry();
    registry.get(param_id).map(|d| d.default).unwrap_or(0.0)
}
