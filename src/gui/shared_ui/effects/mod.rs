// Effects module exports

pub mod core;
pub mod dynamics;
pub mod filter_pitch;
pub mod lofi;
pub mod modulation;
pub mod multiband;

use vizia::prelude::*;

pub fn build_effects_section(cx: &mut Context) {
    const EFFECT_COL_WIDTH: f32 = 280.0;

    VStack::new(cx, |cx| {
        Label::new(cx, "Effects")
            .font_size(16.0)
            .color(Color::rgb(200, 200, 210))
            .height(Pixels(24.0));

        // Row 1: Standard distortion, chorus, delay
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| core::build_distortion_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| core::build_chorus_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| core::build_delay_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
        })
        .height(Pixels(125.0))
        .gap(Pixels(12.0));

        // Row 2: Multiband distortion, stereo widener, reverb
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| multiband::build_multiband_distortion_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH + 100.0))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| multiband::build_stereo_widener_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH + 20.0))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| core::build_reverb_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
        })
        .height(Pixels(200.0))
        .gap(Pixels(12.0));

        // Row 3: Modulation effects (phaser, flanger, tremolo, auto-pan)
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| modulation::build_phaser_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| modulation::build_flanger_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| modulation::build_tremolo_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| modulation::build_autopan_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
        })
        .height(Pixels(150.0))
        .gap(Pixels(12.0));

        // Row 4: Filter/pitch effects + dynamics (comb, ring mod, compressor)
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| filter_pitch::build_combfilter_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| filter_pitch::build_ringmod_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| dynamics::build_compressor_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH + 20.0))
                .gap(Pixels(6.0));
        })
        .height(Pixels(150.0))
        .gap(Pixels(12.0));

        // Row 5: Lo-fi effects (bitcrusher, waveshaper, exciter)
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| lofi::build_bitcrusher_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| lofi::build_waveshaper_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH / 1.5))
                .gap(Pixels(6.0));
            VStack::new(cx, |cx| lofi::build_exciter_section(cx))
                .width(Pixels(EFFECT_COL_WIDTH + 30.0))
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
