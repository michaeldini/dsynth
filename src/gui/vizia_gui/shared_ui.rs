// Shared UI layout for both CLAP plugin and standalone VIZIA GUI

use crate::gui::vizia_gui::GuiState;
use crate::gui::vizia_gui::widgets::{
    distortion_type_button, filter_type_button, fm_source_button, lfo_waveform_button,
    oscillator_waveform_button, param_checkbox, param_knob,
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
                .font_size(24.0)
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

                // Row 2: Oscillators (with conditional waveform-specific sections)
                HStack::new(cx, |cx| {
                    // Oscillator 1 column
                    VStack::new(cx, |cx| {
                        build_osc_section(cx, 1);
                        build_waveform_specific_section(cx, 1);
                    })
                    .width(Pixels(OSC_COL_WIDTH))
                    .height(Units::Auto)
                    .padding(Pixels(10.0))
                    .gap(Pixels(10.0))
                    .background_color(Color::rgb(35, 35, 40));

                    // Oscillator 2 column
                    VStack::new(cx, |cx| {
                        build_osc_section(cx, 2);
                        build_waveform_specific_section(cx, 2);
                    })
                    .width(Pixels(OSC_COL_WIDTH))
                    .height(Units::Auto)
                    .padding(Pixels(10.0))
                    .gap(Pixels(10.0))
                    .background_color(Color::rgb(35, 35, 40));

                    // Oscillator 3 column
                    VStack::new(cx, |cx| {
                        build_osc_section(cx, 3);
                        build_waveform_specific_section(cx, 3);
                    })
                    .width(Pixels(OSC_COL_WIDTH))
                    .height(Units::Auto)
                    .padding(Pixels(10.0))
                    .gap(Pixels(10.0))
                    .background_color(Color::rgb(35, 35, 40));
                })
                .height(Units::Auto)
                .gap(Pixels(COL_GAP));

                // Row 3: Filters
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| build_filter_section(cx, 1))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto);
                        VStack::new(cx, |cx| build_filter_section(cx, 2))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto);
                        VStack::new(cx, |cx| build_filter_section(cx, 3))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto);
                    })
                    .height(Units::Auto)
                    .gap(Pixels(COL_GAP));
                })
                .background_color(Color::rgb(35, 35, 40))
                .height(Pixels(275.0));

                // Row 4: LFOs
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| build_lfo_section(cx, 1))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto);
                        VStack::new(cx, |cx| build_lfo_section(cx, 2))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto);
                        VStack::new(cx, |cx| build_lfo_section(cx, 3))
                            .width(Pixels(OSC_COL_WIDTH))
                            .height(Units::Auto);
                    })
                    .height(Units::Auto)
                    .gap(Pixels(COL_GAP));
                })
                .height(Pixels(250.0));

                // Row 5: Effects
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

        // Randomize button
        Button::new(cx, |cx| Label::new(cx, "🎲 Randomize"))
            .on_press(|cx| cx.emit(crate::gui::vizia_gui::GuiMessage::Randomize))
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
    let (
        wf,
        pitch,
        detune,
        gain,
        pan,
        unison,
        unison_detune,
        phase,
        shape,
        fm_src,
        fm_amt,
        solo,
        unison_norm,
    ) = match osc_index {
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
            PARAM_OSC1_SOLO,
            PARAM_OSC1_UNISON_NORMALIZE,
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
            PARAM_OSC2_SOLO,
            PARAM_OSC2_UNISON_NORMALIZE,
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
            PARAM_OSC3_SOLO,
            PARAM_OSC3_UNISON_NORMALIZE,
        ),
    };

    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, &format!("Osc {}", osc_index))
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210));
            oscillator_waveform_button(cx, wf, osc_index - 1);
            fm_source_button(cx, fm_src, osc_index - 1);

            let solo_v = current_normalized(cx, solo);
            param_checkbox(cx, solo, "Solo", solo_v > 0.5);
        })
        .height(Units::Auto)
        .gap(Pixels(10.0));
        HStack::new(cx, |cx| {
            let pitch_v = current_normalized(cx, pitch);
            let detune_v = current_normalized(cx, detune);
            let gain_v = current_normalized(cx, gain);
            let pan_v = current_normalized(cx, pan);

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
            let unison_norm_v = current_normalized(cx, unison_norm);

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
            param_checkbox(cx, unison_norm, "UNorm", unison_norm_v > 0.5);
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .height(Units::Auto)
    .gap(Pixels(10.0));
}

/// Build conditionally-rendered section based on selected waveform
/// Shows additive harmonics for Additive waveform (7), wavetable controls for Wavetable (8)
pub fn build_waveform_specific_section(cx: &mut Context, osc_index: usize) {
    match osc_index {
        1 => {
            // Additive harmonics section - visible when waveform == 7 (Additive)
            VStack::new(cx, |cx| {
                build_additive_osc_section(cx, 1);
            })
            .height(Units::Auto)
            .visibility(GuiState::osc1_waveform.map(|wf| {
                if *wf == 7 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            }));
        }
        2 => {
            // Additive harmonics section - visible when waveform == 7 (Additive)
            VStack::new(cx, |cx| {
                build_additive_osc_section(cx, 2);
            })
            .height(Units::Auto)
            .visibility(GuiState::osc2_waveform.map(|wf| {
                if *wf == 7 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            }));
        }
        _ => {
            // Additive harmonics section - visible when waveform == 7 (Additive)
            VStack::new(cx, |cx| {
                build_additive_osc_section(cx, 3);
            })
            .height(Units::Auto)
            .visibility(GuiState::osc3_waveform.map(|wf| {
                if *wf == 7 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            }));
        }
    }

    // Wavetable controls section - visible when waveform == 8 (Wavetable)
    match osc_index {
        1 => {
            VStack::new(cx, |cx| {
                build_wavetable_osc_section(cx, 1);
            })
            .height(Pixels(220.0))
            .visibility(GuiState::osc1_waveform.map(|wf| {
                if *wf == 8 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            }));
        }
        2 => {
            VStack::new(cx, |cx| {
                build_wavetable_osc_section(cx, 2);
            })
            .height(Pixels(220.0))
            .visibility(GuiState::osc2_waveform.map(|wf| {
                if *wf == 8 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            }));
        }
        _ => {
            VStack::new(cx, |cx| {
                build_wavetable_osc_section(cx, 3);
            })
            .height(Units::Auto)
            .visibility(GuiState::osc3_waveform.map(|wf| {
                if *wf == 8 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            }));
        }
    }
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

pub fn build_wavetable_osc_section(cx: &mut Context, osc_index: usize) {
    let (wt_idx, wt_pos) = match osc_index {
        1 => (PARAM_OSC1_WAVETABLE_INDEX, PARAM_OSC1_WAVETABLE_POSITION),
        2 => (PARAM_OSC2_WAVETABLE_INDEX, PARAM_OSC2_WAVETABLE_POSITION),
        _ => (PARAM_OSC3_WAVETABLE_INDEX, PARAM_OSC3_WAVETABLE_POSITION),
    };

    VStack::new(cx, |cx| {
        // Section header
        Label::new(cx, &format!("Wavetable Controls (Osc {})", osc_index))
            .font_size(12.0)
            .color(Color::rgb(200, 200, 210))
            .width(Stretch(1.0))
            .height(Pixels(22.0));

        // Wavetable selector and position knobs
        HStack::new(cx, |cx| {
            let wt_idx_v = current_normalized(cx, wt_idx);
            let wt_pos_v = current_normalized(cx, wt_pos);

            param_knob(
                cx,
                wt_idx,
                "Wavetable",
                wt_idx_v,
                default_normalized(wt_idx),
            );
            param_knob(cx, wt_pos, "Position", wt_pos_v, default_normalized(wt_pos));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));

        // Info text about wavetable names
        Label::new(cx, "Built-in: Sine, Saw, Square, Triangle")
            .font_size(10.0)
            .color(Color::rgb(160, 160, 170))
            .width(Stretch(1.0))
            .height(Pixels(16.0))
            .text_wrap(true);
    })
    .gap(Pixels(12.0));
}

pub fn build_filter_section(cx: &mut Context, filter_index: usize) {
    let (ft, cutoff, res, bw, kt, env_amt, env_att, env_dec, env_sus, env_rel) = match filter_index
    {
        1 => (
            PARAM_FILTER1_TYPE,
            PARAM_FILTER1_CUTOFF,
            PARAM_FILTER1_RESONANCE,
            PARAM_FILTER1_BANDWIDTH,
            PARAM_FILTER1_KEY_TRACKING,
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

            param_knob(cx, cutoff, "Cutoff", cutoff_v, default_normalized(cutoff));
            param_knob(cx, res, "Res", res_v, default_normalized(res));
            param_knob(cx, bw, "BW", bw_v, default_normalized(bw));
            param_knob(cx, kt, "KeyTrk", kt_v, default_normalized(kt));
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
        HStack::new(cx, move |cx| {
            Label::new(cx, &format!("LFO {}", lfo_index))
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210));
            lfo_waveform_button(cx, wf, lfo_index - 1);
        })
        .height(Units::Auto)
        .gap(Pixels(10.0));

        HStack::new(cx, move |cx| {
            let rate_v = current_normalized(cx, rate);
            let depth_v = current_normalized(cx, depth);
            let filt_v = current_normalized(cx, filt);

            // lfo_waveform_button(cx, wf, lfo_index - 1);
            param_knob(cx, rate, "Rate", rate_v, default_normalized(rate));
            param_knob(cx, depth, "Depth", depth_v, default_normalized(depth));
            param_knob(cx, filt, "Filter", filt_v, default_normalized(filt));
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));

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
    .height(Pixels(350.0))
    .gap(Pixels(10.0));
}

pub fn build_effects_section(cx: &mut Context) {
    const EFFECT_COL_WIDTH: f32 = 280.0;

    VStack::new(cx, |cx| {
        Label::new(cx, "Effects")
            .font_size(16.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(24.0));

        // Row 1: Standard distortion, chorus, delay
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
        })
        .height(Pixels(150.0))
        .gap(Pixels(12.0));

        // Row 2: Multiband distortion, stereo widener, reverb
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| build_multiband_distortion_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH + 100.0))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_stereo_widener_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH + 20.0))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_reverb_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
        })
        .height(Pixels(200.0))
        .gap(Pixels(12.0));

        // Row 3: Modulation effects (phaser, flanger, tremolo, auto-pan)
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| build_phaser_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_flanger_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_tremolo_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_autopan_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
        })
        .height(Pixels(150.0))
        .gap(Pixels(12.0));

        // Row 4: Filter/pitch effects + dynamics (comb, ring mod, compressor)
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| build_combfilter_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_ringmod_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_compressor_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH + 20.0))
                .gap(Pixels(6.0));
        })
        .height(Pixels(150.0))
        .gap(Pixels(12.0));

        // Row 5: Lo-fi effects (bitcrusher, waveshaper)
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| build_bitcrusher_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| build_waveshaper_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
        })
        .height(Pixels(150.0))
        .gap(Pixels(12.0));
    })
    .width(Stretch(1.0))
    .height(Units::Auto)
    .background_color(Color::rgb(35, 35, 40))
    .padding(Pixels(10.0))
    .gap(Pixels(6.0));
}

pub fn build_distortion_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Distortion")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_DISTORTION_ENABLED);
            param_checkbox(cx, PARAM_DISTORTION_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0));

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
        HStack::new(cx, |cx| {
            Label::new(cx, "Chorus")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_CHORUS_ENABLED);
            param_checkbox(cx, PARAM_CHORUS_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0));

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
        HStack::new(cx, |cx| {
            Label::new(cx, "Delay")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_DELAY_ENABLED);
            param_checkbox(cx, PARAM_DELAY_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0));

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
        HStack::new(cx, |cx| {
            Label::new(cx, "Reverb")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_REVERB_ENABLED);
            param_checkbox(cx, PARAM_REVERB_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0));

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
// New effect GUI builder functions to be appended to shared_ui.rs

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
        .gap(Pixels(8.0));

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

pub fn build_combfilter_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Comb Filter")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_COMB_ENABLED);
            param_checkbox(cx, PARAM_COMB_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0));

        HStack::new(cx, |cx| {
            let freq_v = current_normalized(cx, PARAM_COMB_FREQUENCY);
            let fb_v = current_normalized(cx, PARAM_COMB_FEEDBACK);
            let mix_v = current_normalized(cx, PARAM_COMB_MIX);

            param_knob(
                cx,
                PARAM_COMB_FREQUENCY,
                "Freq",
                freq_v,
                default_normalized(PARAM_COMB_FREQUENCY),
            );
            param_knob(
                cx,
                PARAM_COMB_FEEDBACK,
                "FB",
                fb_v,
                default_normalized(PARAM_COMB_FEEDBACK),
            );
            param_knob(
                cx,
                PARAM_COMB_MIX,
                "Mix",
                mix_v,
                default_normalized(PARAM_COMB_MIX),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_ringmod_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Ring Mod")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_RINGMOD_ENABLED);
            param_checkbox(cx, PARAM_RINGMOD_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0));

        HStack::new(cx, |cx| {
            let freq_v = current_normalized(cx, PARAM_RINGMOD_FREQUENCY);
            let depth_v = current_normalized(cx, PARAM_RINGMOD_DEPTH);

            param_knob(
                cx,
                PARAM_RINGMOD_FREQUENCY,
                "Freq",
                freq_v,
                default_normalized(PARAM_RINGMOD_FREQUENCY),
            );
            param_knob(
                cx,
                PARAM_RINGMOD_DEPTH,
                "Depth",
                depth_v,
                default_normalized(PARAM_RINGMOD_DEPTH),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

pub fn build_compressor_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Compressor")
                .font_size(14.0)
                .color(Color::rgb(200, 200, 210))
                .height(Pixels(22.0));
            let enabled = current_normalized(cx, PARAM_COMPRESSOR_ENABLED);
            param_checkbox(cx, PARAM_COMPRESSOR_ENABLED, "On", enabled > 0.5);
        })
        .gap(Pixels(8.0));

        HStack::new(cx, |cx| {
            let thresh_v = current_normalized(cx, PARAM_COMPRESSOR_THRESHOLD);
            let ratio_v = current_normalized(cx, PARAM_COMPRESSOR_RATIO);
            let attack_v = current_normalized(cx, PARAM_COMPRESSOR_ATTACK);
            let release_v = current_normalized(cx, PARAM_COMPRESSOR_RELEASE);

            param_knob(
                cx,
                PARAM_COMPRESSOR_THRESHOLD,
                "Thresh",
                thresh_v,
                default_normalized(PARAM_COMPRESSOR_THRESHOLD),
            );
            param_knob(
                cx,
                PARAM_COMPRESSOR_RATIO,
                "Ratio",
                ratio_v,
                default_normalized(PARAM_COMPRESSOR_RATIO),
            );
            param_knob(
                cx,
                PARAM_COMPRESSOR_ATTACK,
                "Attack",
                attack_v,
                default_normalized(PARAM_COMPRESSOR_ATTACK),
            );
            param_knob(
                cx,
                PARAM_COMPRESSOR_RELEASE,
                "Release",
                release_v,
                default_normalized(PARAM_COMPRESSOR_RELEASE),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .gap(Pixels(6.0));
}

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
        .gap(Pixels(8.0));

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
        .gap(Pixels(8.0));

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
