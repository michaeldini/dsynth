// Oscillator sections: main controls, additive harmonics, wavetable

use super::helpers::{current_normalized, default_normalized};
use crate::gui::vizia_gui::widgets::{
    fm_source_button, oscillator_waveform_button, param_checkbox, param_knob, param_vslider,
};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

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
        saturation,
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
            PARAM_OSC1_SATURATION,
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
            PARAM_OSC2_SATURATION,
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
            PARAM_OSC3_SATURATION,
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
            let saturation_v = current_normalized(cx, saturation);

            param_knob(cx, pitch, "Pitch", pitch_v, default_normalized(pitch));
            param_knob(cx, detune, "Detune", detune_v, default_normalized(detune));
            param_knob(cx, gain, "Gain", gain_v, default_normalized(gain));
            param_knob(cx, pan, "Pan", pan_v, default_normalized(pan));
            param_knob(
                cx,
                saturation,
                "Sat",
                saturation_v,
                default_normalized(saturation),
            );
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

pub fn build_waveform_specific_section(cx: &mut Context, osc_index: usize) {
    VStack::new(cx, |cx| {
        build_additive_osc_section(cx, osc_index);
        build_wavetable_osc_section(cx, osc_index);
    })
    .height(Pixels(125.0))
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
            let h5_v = current_normalized(cx, h5);
            let h6_v = current_normalized(cx, h6);
            let h7_v = current_normalized(cx, h7);
            let h8_v = current_normalized(cx, h8);

            param_vslider(cx, h1, "H1", h1_v, default_normalized(h1));
            param_vslider(cx, h2, "H2", h2_v, default_normalized(h2));
            param_vslider(cx, h3, "H3", h3_v, default_normalized(h3));
            param_vslider(cx, h4, "H4", h4_v, default_normalized(h4));
            param_vslider(cx, h5, "H5", h5_v, default_normalized(h5));
            param_vslider(cx, h6, "H6", h6_v, default_normalized(h6));
            param_vslider(cx, h7, "H7", h7_v, default_normalized(h7));
            param_vslider(cx, h8, "H8", h8_v, default_normalized(h8));
        })
        .height(Units::Auto)
        .gap(Pixels(2.0));
    })
    .height(Units::Auto)
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
    })
    .height(Units::Auto)
    .gap(Pixels(12.0));
}
