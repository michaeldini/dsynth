// Additive synthesis GUI components
use crate::params::{SynthParams, Waveform};
use iced::{
    widget::{button, column, container, row, slider, text},
    Alignment, Color, Element, Length,
};

use super::messages::{Message, OscTab, OscillatorMessage};

/// Render tab buttons for oscillator
pub fn oscillator_tab_buttons(index: usize, active_tab: OscTab) -> Element<'static, Message> {
    row![
        button(if active_tab == OscTab::Basic {
            "● Basic"
        } else {
            "○ Basic"
        })
        .on_press(Message::OscTabChanged(index, OscTab::Basic))
        .padding(8),
        button(if active_tab == OscTab::Harmonics {
            "● Harmonics"
        } else {
            "○ Harmonics"
        })
        .on_press(Message::OscTabChanged(index, OscTab::Harmonics))
        .padding(8),
    ]
    .spacing(10)
    .into()
}

/// Render harmonics editor controls for additive synthesis
pub fn harmonics_tab_content(params: &SynthParams, index: usize) -> Element<'static, Message> {
    let osc = &params.oscillators[index];

    // Check if additive waveform is selected
    if osc.waveform != Waveform::Additive {
        return column![text("Select 'Additive' waveform to edit harmonics").size(16),]
            .spacing(10)
            .padding(10)
            .into();
    }

    let mut content = column![
        text("Harmonic Amplitudes").size(18),
        text("Tip: H1 = fundamental, H2-H8 = overtones. Shape morphs the balance.").size(12),
    ]
    .spacing(5);

    // 8 harmonic sliders
    for h_idx in 0..8 {
        let amp = osc.additive_harmonics[h_idx];
        content = content.push(
            row![
                text(format!("H{}:", h_idx + 1)).width(Length::Fixed(40.0)),
                slider(0.0..=1.0, amp, move |v| {
                    Message::Oscillator(index, OscillatorMessage::AdditiveHarmonicChanged(h_idx, v))
                })
                .step(0.01)
                .width(Length::Fill),
                text(format!("{:.2}", amp)).width(Length::Fixed(45.0)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        );
    }

    // Spectrum visualizer
    content = content
        .push(text("Spectrum:").size(16))
        .push(spectrum_visualizer(osc.additive_harmonics));

    content.padding(10).into()
}

/// Simple spectrum visualizer
fn spectrum_visualizer(harmonics: [f32; 8]) -> Element<'static, Message> {
    let bars: Vec<Element<'static, Message>> = harmonics
        .iter()
        .enumerate()
        .map(|(i, &amp)| {
            let height = (amp * 80.0).max(4.0);
            column![
                container(text(""))
                    .width(Length::Fixed(25.0))
                    .height(Length::Fixed(height))
                    .style(|_theme: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.6, 0.9))),
                        border: iced::Border::default(),
                        ..Default::default()
                    }),
                text(format!("H{}", i + 1)).size(10)
            ]
            .align_x(Alignment::Center)
            .spacing(2)
            .into()
        })
        .collect();

    row(bars).spacing(5).padding(10).into()
}
