// Effects module exports

pub mod core;
pub mod dynamics;
pub mod filter_pitch;
pub mod lofi;
pub mod modulation;
pub mod multiband;

use vizia::prelude::*;

use super::helpers::effect_row;

pub fn build_effects_section(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "Effects")
            .font_size(16.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(24.0));

        // Single-column list: one effect per row
        // Order matches the audio processing chain in engine/mod.rs
        effect_row(cx, 125.0, dynamics::build_compressor_section);
        effect_row(cx, 125.0, core::build_distortion_section);
        effect_row(cx, 125.0, lofi::build_waveshaper_section);
        effect_row(cx, 125.0, lofi::build_bitcrusher_section);
        effect_row(cx, 200.0, multiband::build_multiband_distortion_section);
        effect_row(cx, 125.0, lofi::build_exciter_section);
        effect_row(cx, 125.0, filter_pitch::build_combfilter_section);
        effect_row(cx, 125.0, modulation::build_phaser_section);
        effect_row(cx, 125.0, modulation::build_flanger_section);
        effect_row(cx, 125.0, filter_pitch::build_ringmod_section);
        effect_row(cx, 125.0, modulation::build_tremolo_section);
        effect_row(cx, 125.0, core::build_chorus_section);
        effect_row(cx, 125.0, core::build_delay_section);
        effect_row(cx, 125.0, modulation::build_autopan_section);
        effect_row(cx, 125.0, multiband::build_stereo_widener_section);
        effect_row(cx, 125.0, core::build_reverb_section);
    })
    .width(Stretch(1.0))
    .height(Units::Auto)
    .background_color(Color::rgb(35, 35, 40))
    .padding(Pixels(10.0))
    .gap(Pixels(12.0));
}
