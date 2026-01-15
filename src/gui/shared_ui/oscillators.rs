// Oscillator sections: main controls, additive harmonics, wavetable

use super::helpers::{current_normalized, default_normalized};
use super::traits::{IndexedSection, ParameterLayout};
use crate::gui::widgets::{
    fm_source_button, oscillator_waveform_button, param_checkbox, param_knob, param_vslider,
};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

/// Parameter set for an oscillator instance
pub struct OscillatorParams {
    pub waveform: u32,
    pub pitch: u32,
    pub detune: u32,
    pub gain: u32,
    pub pan: u32,
    pub unison: u32,
    pub unison_detune: u32,
    pub phase: u32,
    pub shape: u32,
    pub fm_source: u32,
    pub fm_amount: u32,
    pub solo: u32,
    pub unison_normalize: u32,
    pub saturation: u32,
    pub h1: u32,
    pub h2: u32,
    pub h3: u32,
    pub h4: u32,
    pub h5: u32,
    pub h6: u32,
    pub h7: u32,
    pub h8: u32,
    pub wavetable_index: u32,
    pub wavetable_position: u32,
}

/// Oscillator UI section builder
pub struct OscillatorSection;

impl IndexedSection for OscillatorSection {
    type Params = OscillatorParams;

    fn get_params(&self, index: usize) -> Self::Params {
        match index {
            1 => OscillatorParams {
                waveform: PARAM_OSC1_WAVEFORM,
                pitch: PARAM_OSC1_PITCH,
                detune: PARAM_OSC1_DETUNE,
                gain: PARAM_OSC1_GAIN,
                pan: PARAM_OSC1_PAN,
                unison: PARAM_OSC1_UNISON,
                unison_detune: PARAM_OSC1_UNISON_DETUNE,
                phase: PARAM_OSC1_PHASE,
                shape: PARAM_OSC1_SHAPE,
                fm_source: PARAM_OSC1_FM_SOURCE,
                fm_amount: PARAM_OSC1_FM_AMOUNT,
                solo: PARAM_OSC1_SOLO,
                unison_normalize: PARAM_OSC1_UNISON_NORMALIZE,
                saturation: PARAM_OSC1_SATURATION,
                h1: PARAM_OSC1_H1,
                h2: PARAM_OSC1_H2,
                h3: PARAM_OSC1_H3,
                h4: PARAM_OSC1_H4,
                h5: PARAM_OSC1_H5,
                h6: PARAM_OSC1_H6,
                h7: PARAM_OSC1_H7,
                h8: PARAM_OSC1_H8,
                wavetable_index: PARAM_OSC1_WAVETABLE_INDEX,
                wavetable_position: PARAM_OSC1_WAVETABLE_POSITION,
            },
            2 => OscillatorParams {
                waveform: PARAM_OSC2_WAVEFORM,
                pitch: PARAM_OSC2_PITCH,
                detune: PARAM_OSC2_DETUNE,
                gain: PARAM_OSC2_GAIN,
                pan: PARAM_OSC2_PAN,
                unison: PARAM_OSC2_UNISON,
                unison_detune: PARAM_OSC2_UNISON_DETUNE,
                phase: PARAM_OSC2_PHASE,
                shape: PARAM_OSC2_SHAPE,
                fm_source: PARAM_OSC2_FM_SOURCE,
                fm_amount: PARAM_OSC2_FM_AMOUNT,
                solo: PARAM_OSC2_SOLO,
                unison_normalize: PARAM_OSC2_UNISON_NORMALIZE,
                saturation: PARAM_OSC2_SATURATION,
                h1: PARAM_OSC2_H1,
                h2: PARAM_OSC2_H2,
                h3: PARAM_OSC2_H3,
                h4: PARAM_OSC2_H4,
                h5: PARAM_OSC2_H5,
                h6: PARAM_OSC2_H6,
                h7: PARAM_OSC2_H7,
                h8: PARAM_OSC2_H8,
                wavetable_index: PARAM_OSC2_WAVETABLE_INDEX,
                wavetable_position: PARAM_OSC2_WAVETABLE_POSITION,
            },
            _ => OscillatorParams {
                waveform: PARAM_OSC3_WAVEFORM,
                pitch: PARAM_OSC3_PITCH,
                detune: PARAM_OSC3_DETUNE,
                gain: PARAM_OSC3_GAIN,
                pan: PARAM_OSC3_PAN,
                unison: PARAM_OSC3_UNISON,
                unison_detune: PARAM_OSC3_UNISON_DETUNE,
                phase: PARAM_OSC3_PHASE,
                shape: PARAM_OSC3_SHAPE,
                fm_source: PARAM_OSC3_FM_SOURCE,
                fm_amount: PARAM_OSC3_FM_AMOUNT,
                solo: PARAM_OSC3_SOLO,
                unison_normalize: PARAM_OSC3_UNISON_NORMALIZE,
                saturation: PARAM_OSC3_SATURATION,
                h1: PARAM_OSC3_H1,
                h2: PARAM_OSC3_H2,
                h3: PARAM_OSC3_H3,
                h4: PARAM_OSC3_H4,
                h5: PARAM_OSC3_H5,
                h6: PARAM_OSC3_H6,
                h7: PARAM_OSC3_H7,
                h8: PARAM_OSC3_H8,
                wavetable_index: PARAM_OSC3_WAVETABLE_INDEX,
                wavetable_position: PARAM_OSC3_WAVETABLE_POSITION,
            },
        }
    }

    fn build(&self, cx: &mut Context, index: usize) {
        let p = self.get_params(index);

        VStack::new(cx, |cx| {
            // Header row
            HStack::new(cx, |cx| {
                Label::new(cx, &format!("Osc {}", index))
                    .font_size(14.0)
                    .color(Color::rgb(200, 200, 210));
                oscillator_waveform_button(cx, p.waveform, index - 1);
                fm_source_button(cx, p.fm_source, index - 1);

                let solo_v = current_normalized(cx, p.solo);
                param_checkbox(cx, p.solo, "Solo", solo_v > 0.5);
            })
            .height(Units::Auto)
            .gap(Pixels(10.0));

            // Main parameters
            Self::build_param_row(cx, |cx| {
                let pitch_v = current_normalized(cx, p.pitch);
                let detune_v = current_normalized(cx, p.detune);
                let gain_v = current_normalized(cx, p.gain);
                let pan_v = current_normalized(cx, p.pan);
                let saturation_v = current_normalized(cx, p.saturation);

                param_knob(cx, p.pitch, "Pitch", pitch_v, default_normalized(p.pitch));
                param_knob(
                    cx,
                    p.detune,
                    "Detune",
                    detune_v,
                    default_normalized(p.detune),
                );
                param_knob(cx, p.gain, "Gain", gain_v, default_normalized(p.gain));
                param_knob(cx, p.pan, "Pan", pan_v, default_normalized(p.pan));
                param_knob(
                    cx,
                    p.saturation,
                    "Sat",
                    saturation_v,
                    default_normalized(p.saturation),
                );
            });

            // Modulation & unison parameters
            Self::build_param_row(cx, |cx| {
                let fm_amount_v = current_normalized(cx, p.fm_amount);
                let unison_v = current_normalized(cx, p.unison);
                let unison_detune_v = current_normalized(cx, p.unison_detune);
                let shape_v = current_normalized(cx, p.shape);
                let unison_normalize_v = current_normalized(cx, p.unison_normalize);

                param_knob(
                    cx,
                    p.fm_amount,
                    "FM Amt",
                    fm_amount_v,
                    default_normalized(p.fm_amount),
                );
                param_knob(
                    cx,
                    p.unison,
                    "Unison",
                    unison_v,
                    default_normalized(p.unison),
                );
                param_knob(
                    cx,
                    p.unison_detune,
                    "UDet",
                    unison_detune_v,
                    default_normalized(p.unison_detune),
                );
                param_knob(cx, p.shape, "Shape", shape_v, default_normalized(p.shape));
                param_checkbox(cx, p.unison_normalize, "UNorm", unison_normalize_v > 0.5);
            });
        })
        .height(Units::Auto)
        .gap(Pixels(10.0));
    }

    fn section_name(&self) -> &'static str {
        "Oscillator"
    }
}

// Helper functions using the parameter struct
fn build_additive_section(cx: &mut Context, p: &OscillatorParams, index: usize) {
    VStack::new(cx, |cx| {
        Label::new(cx, &format!("Osc {} Harmonics", index))
            .font_size(14.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(22.0));

        HStack::new(cx, |cx| {
            let h1_v = current_normalized(cx, p.h1);
            let h2_v = current_normalized(cx, p.h2);
            let h3_v = current_normalized(cx, p.h3);
            let h4_v = current_normalized(cx, p.h4);
            let h5_v = current_normalized(cx, p.h5);
            let h6_v = current_normalized(cx, p.h6);
            let h7_v = current_normalized(cx, p.h7);
            let h8_v = current_normalized(cx, p.h8);

            param_vslider(cx, p.h1, "H1", h1_v, default_normalized(p.h1));
            param_vslider(cx, p.h2, "H2", h2_v, default_normalized(p.h2));
            param_vslider(cx, p.h3, "H3", h3_v, default_normalized(p.h3));
            param_vslider(cx, p.h4, "H4", h4_v, default_normalized(p.h4));
            param_vslider(cx, p.h5, "H5", h5_v, default_normalized(p.h5));
            param_vslider(cx, p.h6, "H6", h6_v, default_normalized(p.h6));
            param_vslider(cx, p.h7, "H7", h7_v, default_normalized(p.h7));
            param_vslider(cx, p.h8, "H8", h8_v, default_normalized(p.h8));
        })
        .height(Units::Auto)
        .gap(Pixels(2.0));
    })
    .height(Units::Auto)
    .gap(Pixels(12.0));
}

fn build_wavetable_section(cx: &mut Context, p: &OscillatorParams, index: usize) {
    VStack::new(cx, |cx| {
        Label::new(cx, &format!("Wavetable Controls (Osc {})", index))
            .font_size(12.0)
            .color(Color::rgb(200, 200, 210))
            .width(Stretch(1.0))
            .height(Pixels(22.0));

        HStack::new(cx, |cx| {
            let wt_idx_v = current_normalized(cx, p.wavetable_index);
            let wt_pos_v = current_normalized(cx, p.wavetable_position);

            param_knob(
                cx,
                p.wavetable_index,
                "Wavetable",
                wt_idx_v,
                default_normalized(p.wavetable_index),
            );
            param_knob(
                cx,
                p.wavetable_position,
                "Position",
                wt_pos_v,
                default_normalized(p.wavetable_position),
            );
        })
        .height(Units::Auto)
        .gap(Pixels(6.0));
    })
    .height(Units::Auto)
    .gap(Pixels(12.0));
}

// Public API functions for backward compatibility

// Public API functions for backward compatibility
pub fn build_osc_section(cx: &mut Context, osc_index: usize) {
    OscillatorSection.build(cx, osc_index);
}

pub fn build_waveform_specific_section(cx: &mut Context, osc_index: usize) {
    VStack::new(cx, |cx| {
        let p = OscillatorSection.get_params(osc_index);
        build_additive_section(cx, &p, osc_index);
        build_wavetable_section(cx, &p, osc_index);
    })
    .height(Pixels(125.0))
    .gap(Pixels(10.0));
}
