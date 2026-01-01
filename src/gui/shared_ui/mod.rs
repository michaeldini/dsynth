// Main UI module - shared by both CLAP plugin and standalone

pub mod dynamics;
pub mod effects;
pub mod filters;
pub mod helpers;
pub mod lfos;
pub mod master;
pub mod oscillators;
pub mod traits;

use crate::gui::messages::UiTab;
use crate::gui::theme;
use crate::gui::GuiState;
use vizia::prelude::*;

fn tab_button(cx: &mut Context, label: &str, tab: UiTab, active_tab: UiTab) {
    let is_active = tab == active_tab;
    Button::new(cx, move |cx| Label::new(cx, label))
        .on_press(move |cx| cx.emit(crate::gui::GuiMessage::SetActiveTab(tab)))
        .height(Pixels(32.0))
        .padding(Pixels(8.0))
        .color(if is_active {
            theme::BUTTON_TEXT_ACTIVE
        } else {
            theme::BUTTON_TEXT_INACTIVE
        })
        .background_color(if is_active {
            theme::BUTTON_BG_ACTIVE
        } else {
            theme::BUTTON_BG_INACTIVE
        })
        .corner_radius(Pixels(4.0))
        .cursor(CursorIcon::Hand);
}

/// Build the main UI layout - shared by plugin and standalone
pub fn build_ui(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Title bar + live status text
        HStack::new(cx, |cx| {
            Label::new(cx, "DSynth")
                .font_size(20.0)
                .color(theme::TEXT_PRIMARY);

            // Tab bar
            Binding::new(cx, GuiState::active_tab, |cx, active_tab| {
                let active_tab = active_tab.get(cx);
                HStack::new(cx, |cx| {
                    tab_button(cx, "Oscillator", UiTab::Oscillator, active_tab);
                    tab_button(cx, "Filter + LFO", UiTab::FilterLfo, active_tab);
                    tab_button(cx, "Effects", UiTab::Effects, active_tab);
                })
                .gap(Pixels(8.0))
                .width(Stretch(1.0))
                .padding(Pixels(10.0))
                .background_color(Color::rgb(25, 25, 30));
            });

            Label::new(cx, GuiState::last_param_text)
                .font_size(24.0)
                .color(theme::TEXT_TERTIARY)
                .width(Stretch(1.0))
                .text_align(TextAlign::Right)
                .text_wrap(false)
                .text_overflow(TextOverflow::Ellipsis);
        })
        .height(Pixels(50.0))
        .width(Stretch(1.0))
        .padding(Pixels(10.0))
        .background_color(theme::BG_DARK);

        // Scrollable content area
        ScrollView::new(cx, |cx| {
            Binding::new(cx, GuiState::active_tab, move |cx, active_tab| {
                let active_tab = active_tab.get(cx);
                VStack::new(cx, |cx| match active_tab {
                    UiTab::Oscillator => {
                        // Row 1: Master + Envelope
                        HStack::new(cx, |cx| {
                            VStack::new(cx, |cx| {
                                Label::new(cx, "Master")
                                    .font_size(16.0)
                                    .color(theme::TEXT_SECONDARY)
                                    .height(Pixels(24.0));
                                master::build_master_section(cx);
                            })
                            .width(Stretch(1.0))
                            .padding(Pixels(10.0))
                            .gap(Pixels(6.0))
                            .background_color(theme::BG_SECTION);

                            VStack::new(cx, |cx| {
                                Label::new(cx, "Envelope")
                                    .font_size(16.0)
                                    .color(theme::TEXT_SECONDARY)
                                    .height(Pixels(24.0));
                                master::build_envelope_section(cx);
                            })
                            .width(Stretch(1.0))
                            .padding(Pixels(10.0))
                            .gap(Pixels(6.0))
                            .background_color(theme::BG_SECTION);
                        })
                        .gap(Pixels(theme::COL_GAP))
                        .height(Pixels(225.0));

                        // Row 1.5: Voice Dynamics (Compressor + Transient Shaper)
                        HStack::new(cx, |cx| {
                            VStack::new(cx, |cx| {
                                Label::new(cx, "Voice Compressor")
                                    .font_size(16.0)
                                    .color(theme::TEXT_SECONDARY)
                                    .height(Pixels(24.0));
                                dynamics::build_voice_compressor_section(cx);
                            })
                            .width(Stretch(1.0))
                            .padding(Pixels(10.0))
                            .gap(Pixels(6.0))
                            .background_color(theme::BG_SECTION);

                            VStack::new(cx, |cx| {
                                Label::new(cx, "Transient Shaper")
                                    .font_size(16.0)
                                    .color(theme::TEXT_SECONDARY)
                                    .height(Pixels(24.0));
                                dynamics::build_transient_shaper_section(cx);
                            })
                            .width(Stretch(1.0))
                            .padding(Pixels(10.0))
                            .gap(Pixels(6.0))
                            .background_color(theme::BG_SECTION);
                        })
                        .gap(Pixels(theme::COL_GAP))
                        .height(Pixels(125.0));

                        // Row 2: Oscillators
                        HStack::new(cx, |cx| {
                            // Oscillator 1 column
                            VStack::new(cx, |cx| {
                                oscillators::build_osc_section(cx, 1);
                                oscillators::build_waveform_specific_section(cx, 1);
                            })
                            .width(Pixels(theme::OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(10.0))
                            .gap(Pixels(10.0))
                            .background_color(theme::BG_SECTION);

                            // Oscillator 2 column
                            VStack::new(cx, |cx| {
                                oscillators::build_osc_section(cx, 2);
                                oscillators::build_waveform_specific_section(cx, 2);
                            })
                            .width(Pixels(theme::OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(10.0))
                            .gap(Pixels(10.0))
                            .background_color(theme::BG_SECTION);

                            // Oscillator 3 column
                            VStack::new(cx, |cx| {
                                oscillators::build_osc_section(cx, 3);
                                oscillators::build_waveform_specific_section(cx, 3);
                            })
                            .width(Pixels(theme::OSC_COL_WIDTH))
                            .height(Units::Auto)
                            .padding(Pixels(10.0))
                            .gap(Pixels(10.0))
                            .background_color(theme::BG_SECTION);
                        })
                        .height(Pixels(550.0))
                        .gap(Pixels(theme::COL_GAP));
                    }
                    UiTab::FilterLfo => {
                        // Row 3: Filters
                        VStack::new(cx, |cx| {
                            HStack::new(cx, |cx| {
                                VStack::new(cx, |cx| filters::build_filter_section(cx, 1))
                                    .width(Stretch(1.0))
                                    .height(Units::Auto);
                                VStack::new(cx, |cx| filters::build_filter_section(cx, 2))
                                    .width(Stretch(1.0))
                                    .height(Units::Auto);
                                VStack::new(cx, |cx| filters::build_filter_section(cx, 3))
                                    .width(Stretch(1.0))
                                    .height(Units::Auto);
                            })
                            .height(Units::Auto)
                            .gap(Pixels(theme::COL_GAP));
                        })
                        .background_color(theme::BG_SECTION)
                        .height(Pixels(275.0));

                        // Row 4: LFOs
                        VStack::new(cx, |cx| {
                            HStack::new(cx, |cx| {
                                VStack::new(cx, |cx| lfos::build_lfo_section(cx, 1))
                                    .width(Stretch(1.0))
                                    .height(Units::Auto);
                                VStack::new(cx, |cx| lfos::build_lfo_section(cx, 2))
                                    .width(Stretch(1.0))
                                    .height(Units::Auto);
                                VStack::new(cx, |cx| lfos::build_lfo_section(cx, 3))
                                    .width(Stretch(1.0))
                                    .height(Units::Auto);
                            })
                            .height(Units::Auto)
                            .gap(Pixels(theme::COL_GAP));
                        })
                        .height(Pixels(250.0));
                    }
                    UiTab::Effects => {
                        // Row 5: Effects
                        effects::build_effects_section(cx);
                    }
                })
                .width(Stretch(1.0))
                .height(Units::Auto)
                .min_height(Pixels(0.0))
                .padding(Pixels(10.0))
                .gap(Pixels(theme::ROW_GAP));
            });
        })
        .show_horizontal_scrollbar(false)
        .show_vertical_scrollbar(true)
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .background_color(theme::BG_DARK);
    })
    .width(Stretch(1.0))
    .height(Stretch(1.0))
    .background_color(theme::BG_DARK);
}
