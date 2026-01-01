// LFO sections with modulation routing

use super::helpers::{current_normalized, default_normalized};
use super::traits::{IndexedSection, ParameterLayout};
use crate::gui::widgets::{lfo_waveform_button, param_knob};
use crate::plugin::param_descriptor::*;
use vizia::prelude::*;

/// Parameter set for an LFO instance
pub struct LfoParams {
    pub waveform: u32,
    pub rate: u32,
    pub depth: u32,
    pub filter_amount: u32,
    pub pitch_amount: u32,
    pub gain_amount: u32,
    pub pan_amount: u32,
    pub pwm_amount: u32,
}

/// LFO UI section builder
pub struct LfoSection;

impl IndexedSection for LfoSection {
    type Params = LfoParams;
    
    fn get_params(&self, index: usize) -> Self::Params {
        match index {
            1 => LfoParams {
                waveform: PARAM_LFO1_WAVEFORM,
                rate: PARAM_LFO1_RATE,
                depth: PARAM_LFO1_DEPTH,
                filter_amount: PARAM_LFO1_FILTER_AMOUNT,
                pitch_amount: PARAM_LFO1_PITCH_AMOUNT,
                gain_amount: PARAM_LFO1_GAIN_AMOUNT,
                pan_amount: PARAM_LFO1_PAN_AMOUNT,
                pwm_amount: PARAM_LFO1_PWM_AMOUNT,
            },
            2 => LfoParams {
                waveform: PARAM_LFO2_WAVEFORM,
                rate: PARAM_LFO2_RATE,
                depth: PARAM_LFO2_DEPTH,
                filter_amount: PARAM_LFO2_FILTER_AMOUNT,
                pitch_amount: PARAM_LFO2_PITCH_AMOUNT,
                gain_amount: PARAM_LFO2_GAIN_AMOUNT,
                pan_amount: PARAM_LFO2_PAN_AMOUNT,
                pwm_amount: PARAM_LFO2_PWM_AMOUNT,
            },
            _ => LfoParams {
                waveform: PARAM_LFO3_WAVEFORM,
                rate: PARAM_LFO3_RATE,
                depth: PARAM_LFO3_DEPTH,
                filter_amount: PARAM_LFO3_FILTER_AMOUNT,
                pitch_amount: PARAM_LFO3_PITCH_AMOUNT,
                gain_amount: PARAM_LFO3_GAIN_AMOUNT,
                pan_amount: PARAM_LFO3_PAN_AMOUNT,
                pwm_amount: PARAM_LFO3_PWM_AMOUNT,
            },
        }
    }
    
    fn build(&self, cx: &mut Context, index: usize) {
        let p = self.get_params(index);
        
        VStack::new(cx, move |cx| {
            // Header
            HStack::new(cx, move |cx| {
                Label::new(cx, &format!("LFO {}", index))
                    .font_size(14.0)
                    .color(Color::rgb(200, 200, 210));
                lfo_waveform_button(cx, p.waveform, index - 1);
            })
            .height(Units::Auto)
            .gap(Pixels(10.0));

            // Main parameters
            Self::build_param_row(cx, move |cx| {
                let rate_v = current_normalized(cx, p.rate);
                let depth_v = current_normalized(cx, p.depth);
                let filter_amount_v = current_normalized(cx, p.filter_amount);
                
                param_knob(cx, p.rate, "Rate", rate_v, default_normalized(p.rate));
                param_knob(cx, p.depth, "Depth", depth_v, default_normalized(p.depth));
                param_knob(cx, p.filter_amount, "Filter", filter_amount_v, default_normalized(p.filter_amount));
            });

            // Modulation targets
            Self::build_param_row(cx, move |cx| {
                let pitch_amount_v = current_normalized(cx, p.pitch_amount);
                let gain_amount_v = current_normalized(cx, p.gain_amount);
                let pan_amount_v = current_normalized(cx, p.pan_amount);
                let pwm_amount_v = current_normalized(cx, p.pwm_amount);
                
                param_knob(cx, p.pitch_amount, "Pitch", pitch_amount_v, default_normalized(p.pitch_amount));
                param_knob(cx, p.gain_amount, "Gain", gain_amount_v, default_normalized(p.gain_amount));
                param_knob(cx, p.pan_amount, "Pan", pan_amount_v, default_normalized(p.pan_amount));
                param_knob(cx, p.pwm_amount, "PWM", pwm_amount_v, default_normalized(p.pwm_amount));
            });
        })
        .height(Pixels(350.0))
        .gap(Pixels(10.0));
    }
    
    fn section_name(&self) -> &'static str {
        "LFO"
    }
}

// Public API for backward compatibility

// Public API for backward compatibility
pub fn build_lfo_section(cx: &mut Context, lfo_index: usize) {
    LfoSection.build(cx, lfo_index);
}

