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
        effect_row(cx, 125.0, |cx| core::build_distortion_section(cx));
        effect_row(cx, 125.0, |cx| core::build_chorus_section(cx));
        effect_row(cx, 125.0, |cx| core::build_delay_section(cx));
        effect_row(cx, 125.0, |cx| core::build_reverb_section(cx));

        effect_row(cx, 200.0, |cx| {
            multiband::build_multiband_distortion_section(cx)
        });
        effect_row(cx, 125.0, |cx| multiband::build_stereo_widener_section(cx));

        effect_row(cx, 125.0, |cx| modulation::build_phaser_section(cx));
        effect_row(cx, 125.0, |cx| modulation::build_flanger_section(cx));
        effect_row(cx, 125.0, |cx| modulation::build_tremolo_section(cx));
        effect_row(cx, 125.0, |cx| modulation::build_autopan_section(cx));

        effect_row(cx, 125.0, |cx| filter_pitch::build_combfilter_section(cx));
        effect_row(cx, 125.0, |cx| filter_pitch::build_ringmod_section(cx));
        effect_row(cx, 125.0, |cx| dynamics::build_compressor_section(cx));

        effect_row(cx, 125.0, |cx| lofi::build_bitcrusher_section(cx));
        effect_row(cx, 125.0, |cx| lofi::build_waveshaper_section(cx));
        effect_row(cx, 125.0, |cx| lofi::build_exciter_section(cx));
    })
    .width(Stretch(1.0))
    .height(Units::Auto)
    .background_color(Color::rgb(35, 35, 40))
    .padding(Pixels(10.0))
    .gap(Pixels(12.0));
}
