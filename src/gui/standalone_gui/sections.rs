use crate::params::{DistortionType, FilterType, LFOWaveform, SynthParams, Waveform};
use iced::{
    widget::{column, pick_list, row, slider, text, Column},
    Element,
};

use super::additive_gui;
use super::messages::{
    ChorusMessage, DelayMessage, DistortionMessage, EnvelopeMessage, FilterMessage, LFOMessage,
    Message, OscTab, OscillatorMessage, ReverbMessage,
};

/// Build oscillator controls UI section
pub fn oscillator_controls<'a>(
    params: &'a SynthParams,
    index: usize,
    label: &'a str,
    active_tab: OscTab,
) -> Element<'a, Message> {
    let mut content = Column::new()
        .push(text(label).size(20))
        .push(additive_gui::oscillator_tab_buttons(index, active_tab))
        .spacing(10);

    // Render appropriate tab content
    match active_tab {
        OscTab::Basic => {
            content = content.extend(vec![basic_oscillator_section(params, index)]);
        }
        OscTab::Harmonics => {
            content = content.push(additive_gui::harmonics_tab_content(params, index));
        }
    }

    content.into()
}

/// Render the basic oscillator section (existing controls)
fn basic_oscillator_section<'a>(
    params: &'a SynthParams,
    index: usize,
) -> Element<'a, Message> {
    let osc = &params.oscillators[index];
    let filter = &params.filters[index];
    let lfo = &params.lfos[index];

    let waveforms = vec![
        Waveform::Sine,
        Waveform::Saw,
        Waveform::Square,
        Waveform::Triangle,
        Waveform::Pulse,
        Waveform::WhiteNoise,
        Waveform::PinkNoise,
        Waveform::Additive,
    ];

    let filter_types = vec![
        FilterType::Lowpass,
        FilterType::Highpass,
        FilterType::Bandpass,
    ];

    let lfo_waveforms = vec![
        LFOWaveform::Sine,
        LFOWaveform::Triangle,
        LFOWaveform::Square,
        LFOWaveform::Saw,
    ];

    Column::new()
        // Solo button
        .push(
            iced::widget::button(if osc.solo {
                "SOLO [ON]"
            } else {
                "SOLO [OFF]"
            })
            .on_press(Message::Oscillator(
                index,
                OscillatorMessage::SoloToggled(!osc.solo),
            )),
        )
        // Oscillator controls
        .push(text("Waveform:"))
        .push(pick_list(waveforms, Some(osc.waveform), move |w| {
            Message::Oscillator(index, OscillatorMessage::WaveformChanged(w))
        }))
        .push(text("Pitch (semitones):"))
        .push(
            slider(-24.0..=24.0, osc.pitch, move |p| {
                Message::Oscillator(index, OscillatorMessage::PitchChanged(p))
            })
            .step(1.0),
        )
        .push(text(format!("{:.0}", osc.pitch)))
        .push(text("Detune (cents):"))
        .push(
            slider(-50.0..=50.0, osc.detune, move |d| {
                Message::Oscillator(index, OscillatorMessage::DetuneChanged(d))
            })
            .step(1.0),
        )
        .push(text(format!("{:.0}", osc.detune)))
        .push(text("Gain:"))
        .push(
            slider(0.0..=1.0, osc.gain, move |g| {
                Message::Oscillator(index, OscillatorMessage::GainChanged(g))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", osc.gain)))
        .push(text("Pan:"))
        .push(
            slider(-1.0..=1.0, osc.pan, move |p| {
                Message::Oscillator(index, OscillatorMessage::PanChanged(p))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", osc.pan)))
        .push(text("Unison:"))
        .push(
            slider(1.0..=7.0, osc.unison as f32, move |v| {
                Message::Oscillator(index, OscillatorMessage::UnisonChanged(v as usize))
            })
            .step(1.0),
        )
        .push(text(format!("{}", osc.unison)))
        .push(text("Unison Detune (cents):"))
        .push(
            slider(0.0..=50.0, osc.unison_detune, move |d| {
                Message::Oscillator(index, OscillatorMessage::UnisonDetuneChanged(d))
            })
            .step(1.0),
        )
        .push(text(format!("{:.0}", osc.unison_detune)))
        .push(text("Phase:"))
        .push(
            slider(0.0..=1.0, osc.phase, move |p| {
                Message::Oscillator(index, OscillatorMessage::PhaseChanged(p))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", osc.phase)))
        .push(text("Shape:"))
        .push(
            slider(-1.0..=1.0, osc.shape, move |s| {
                Message::Oscillator(index, OscillatorMessage::ShapeChanged(s))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", osc.shape)))
        // FM Synthesis controls
        .push(text("--- FM Synthesis ---").size(18))
        .push(text("FM Source:"))
        .push({
            // Create a simple list for FM source selection
            // -1 = None, 0-2 = Osc 1-3
            let fm_sources = vec![-1, 0, 1, 2];
            let current_source = osc.fm_source.map(|s| s as i32).unwrap_or(-1);
            
            pick_list(fm_sources, Some(current_source), move |src| {
                let opt_src = if src < 0 { None } else { Some(src as usize) };
                Message::Oscillator(index, OscillatorMessage::FmSourceChanged(opt_src))
            })
        })
        .push(text(match osc.fm_source {
            None => "None".to_string(),
            Some(0) => "Osc 1".to_string(),
            Some(1) => "Osc 2".to_string(),
            Some(2) => "Osc 3".to_string(),
            _ => "Invalid".to_string(),
        }))
        .push(text("FM Amount:"))
        .push(
            slider(0.0..=10.0, osc.fm_amount, move |a| {
                Message::Oscillator(index, OscillatorMessage::FmAmountChanged(a))
            })
            .step(0.1),
        )
        .push(text(format!("{:.1}", osc.fm_amount)))
        // Filter controls
        .push(text("--- Filter ---").size(18))
        .push(text("Type:"))
        .push(pick_list(
            filter_types,
            Some(filter.filter_type),
            move |t| Message::Filter(index, FilterMessage::TypeChanged(t)),
        ))
        .push(text("Cutoff (Hz):"))
        .push(
            slider(20.0..=20000.0, filter.cutoff, move |c| {
                Message::Filter(index, FilterMessage::CutoffChanged(c))
            })
            .step(10.0),
        )
        .push(text(format!("{:.0}", filter.cutoff)))
        .push(text("Resonance:"))
        .push(
            slider(0.5..=10.0, filter.resonance, move |r| {
                Message::Filter(index, FilterMessage::ResonanceChanged(r))
            })
            .step(0.1),
        )
        .push(text(format!("{:.1}", filter.resonance)))
        .push(text("Bandwidth (octaves):"))
        .push(
            slider(0.1..=4.0, filter.bandwidth, move |b| {
                Message::Filter(index, FilterMessage::BandwidthChanged(b))
            })
            .step(0.1),
        )
        .push(text(format!("{:.1}", filter.bandwidth)))
        .push(text("Key Tracking:"))
        .push(
            slider(0.0..=1.0, filter.key_tracking, move |k| {
                Message::Filter(index, FilterMessage::KeyTrackingChanged(k))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", filter.key_tracking)))
        // LFO controls
        .push(text("--- LFO ---").size(18))
        .push(text("Waveform:"))
        .push(pick_list(lfo_waveforms, Some(lfo.waveform), move |w| {
            Message::LFO(index, LFOMessage::WaveformChanged(w))
        }))
        .push(text("Rate (Hz):"))
        .push(
            slider(0.01..=20.0, lfo.rate, move |r| {
                Message::LFO(index, LFOMessage::RateChanged(r))
            })
            .step(0.1),
        )
        .push(text(format!("{:.2}", lfo.rate)))
        .push(text("Depth:"))
        .push(
            slider(0.0..=1.0, lfo.depth, move |d| {
                Message::LFO(index, LFOMessage::DepthChanged(d))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", lfo.depth)))
        .push(text("Filter Amount (Hz):"))
        .push(
            slider(0.0..=5000.0, lfo.filter_amount, move |a| {
                Message::LFO(index, LFOMessage::FilterAmountChanged(a))
            })
            .step(50.0),
        )
        .push(text(format!("{:.0}", lfo.filter_amount)))
        .push(text("Pitch Amount (cents):"))
        .push(
            slider(0.0..=100.0, lfo.pitch_amount, move |p| {
                Message::LFO(index, LFOMessage::PitchAmountChanged(p))
            })
            .step(1.0),
        )
        .push(text(format!("{:.1}", lfo.pitch_amount)))
        .push(text("Gain Amount:"))
        .push(
            slider(0.0..=1.0, lfo.gain_amount, move |g| {
                Message::LFO(index, LFOMessage::GainAmountChanged(g))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", lfo.gain_amount)))
        .push(text("Pan Amount:"))
        .push(
            slider(0.0..=1.0, lfo.pan_amount, move |p| {
                Message::LFO(index, LFOMessage::PanAmountChanged(p))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", lfo.pan_amount)))
        .push(text("PWM Amount:"))
        .push(
            slider(0.0..=1.0, lfo.pwm_amount, move |p| {
                Message::LFO(index, LFOMessage::PwmAmountChanged(p))
            })
            .step(0.01),
        )
        .push(text(format!("{:.2}", lfo.pwm_amount)))
        .spacing(5)
        .padding(10)
        .into()
}

/// Build effects controls UI section
pub fn effects_controls<'a>(params: &'a SynthParams) -> Element<'a, Message> {
    let effects = &params.effects;

    let distortion_types = vec![
        DistortionType::Tanh,
        DistortionType::SoftClip,
        DistortionType::HardClip,
        DistortionType::Cubic,
    ];

    // Distortion controls
    let distortion_section = column![
        text("DISTORTION").size(18),
        text("Type:"),
        pick_list(distortion_types, Some(effects.distortion.dist_type), |t| {
            Message::Distortion(DistortionMessage::TypeChanged(t))
        }),
        text("Drive:"),
        slider(0.0..=1.0, effects.distortion.drive, |v| {
            Message::Distortion(DistortionMessage::DriveChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.distortion.drive)),
        text("Mix:"),
        slider(0.0..=1.0, effects.distortion.mix, |v| {
            Message::Distortion(DistortionMessage::MixChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.distortion.mix)),
    ]
    .spacing(5)
    .padding(10);

    // Chorus controls
    let chorus_section = column![
        text("CHORUS").size(18),
        text("Rate (Hz):"),
        slider(0.1..=5.0, effects.chorus.rate, |v| {
            Message::Chorus(ChorusMessage::RateChanged(v))
        })
        .step(0.1),
        text(format!("{:.1}", effects.chorus.rate)),
        text("Depth:"),
        slider(0.0..=1.0, effects.chorus.depth, |v| {
            Message::Chorus(ChorusMessage::DepthChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.chorus.depth)),
        text("Mix:"),
        slider(0.0..=1.0, effects.chorus.mix, |v| {
            Message::Chorus(ChorusMessage::MixChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.chorus.mix)),
    ]
    .spacing(5)
    .padding(10);

    // Delay controls
    let delay_section = column![
        text("DELAY").size(18),
        text("Time (ms):"),
        slider(1.0..=2000.0, effects.delay.time_ms, |v| {
            Message::Delay(DelayMessage::TimeChanged(v))
        })
        .step(1.0),
        text(format!("{:.0}", effects.delay.time_ms)),
        text("Feedback:"),
        slider(0.0..=0.95, effects.delay.feedback, |v| {
            Message::Delay(DelayMessage::FeedbackChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.delay.feedback)),
        text("Wet:"),
        slider(0.0..=1.0, effects.delay.wet, |v| {
            Message::Delay(DelayMessage::WetChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.delay.wet)),
        text("Dry:"),
        slider(0.0..=1.0, effects.delay.dry, |v| {
            Message::Delay(DelayMessage::DryChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.delay.dry)),
    ]
    .spacing(5)
    .padding(10);

    // Reverb controls
    let reverb_section = column![
        text("REVERB").size(18),
        text("Room Size:"),
        slider(0.0..=1.0, effects.reverb.room_size, |v| {
            Message::Reverb(ReverbMessage::RoomSizeChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.reverb.room_size)),
        text("Damping:"),
        slider(0.0..=1.0, effects.reverb.damping, |v| {
            Message::Reverb(ReverbMessage::DampingChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.reverb.damping)),
        text("Wet:"),
        slider(0.0..=1.0, effects.reverb.wet, |v| {
            Message::Reverb(ReverbMessage::WetChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.reverb.wet)),
        text("Dry:"),
        slider(0.0..=1.0, effects.reverb.dry, |v| {
            Message::Reverb(ReverbMessage::DryChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.reverb.dry)),
        text("Width:"),
        slider(0.0..=1.0, effects.reverb.width, |v| {
            Message::Reverb(ReverbMessage::WidthChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", effects.reverb.width)),
    ]
    .spacing(5)
    .padding(10);

    // Combine all effects in a row
    column![
        text("=== EFFECTS CHAIN ===").size(20),
        text("Signal Flow: Distortion → Chorus → Delay → Reverb").size(14),
        row![
            distortion_section,
            chorus_section,
            delay_section,
            reverb_section
        ]
        .spacing(20)
    ]
    .spacing(10)
    .padding(10)
    .into()
}

/// Build envelope controls UI section
pub fn envelope_controls<'a>(params: &'a SynthParams) -> Element<'a, Message> {
    let env = &params.envelope;

    column![
        text("=== ENVELOPE (ADSR) ===").size(20),
        text("Attack (s):"),
        slider(0.001..=5.0, env.attack, |v| {
            Message::Envelope(EnvelopeMessage::AttackChanged(v))
        })
        .step(0.001),
        text(format!("{:.3}", env.attack)),
        text("Decay (s):"),
        slider(0.001..=5.0, env.decay, |v| {
            Message::Envelope(EnvelopeMessage::DecayChanged(v))
        })
        .step(0.001),
        text(format!("{:.3}", env.decay)),
        text("Sustain (level):"),
        slider(0.0..=1.0, env.sustain, |v| {
            Message::Envelope(EnvelopeMessage::SustainChanged(v))
        })
        .step(0.01),
        text(format!("{:.2}", env.sustain)),
        text("Release (s):"),
        slider(0.001..=5.0, env.release, |v| {
            Message::Envelope(EnvelopeMessage::ReleaseChanged(v))
        })
        .step(0.001),
        text(format!("{:.3}", env.release)),
    ]
    .spacing(5)
    .padding(10)
    .into()
}
