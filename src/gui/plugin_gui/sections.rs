use super::{
    EffectsStates, EnvelopeStates, FilterStates, LfoStates, Message, OscStates, PluginGui,
};
use crate::plugin::DSynthParams;
use nih_plug_iced::widget;

impl PluginGui {
    pub(super) fn osc1_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut OscStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.osc1_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Solo", &params.osc1_solo, &mut states.solo))
            .push(param_row("Pitch", &params.osc1_pitch, &mut states.pitch))
            .push(param_row("Detune", &params.osc1_detune, &mut states.detune))
            .push(param_row("Gain", &params.osc1_gain, &mut states.gain))
            .push(param_row("Pan", &params.osc1_pan, &mut states.pan))
            .push(param_row("Unison", &params.osc1_unison, &mut states.unison))
            .push(param_row(
                "UniDet",
                &params.osc1_unison_detune,
                &mut states.unison_detune,
            ))
            .push(param_row("Phase", &params.osc1_phase, &mut states.phase))
            .push(param_row("Shape", &params.osc1_shape, &mut states.shape))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn osc2_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut OscStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.osc2_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Solo", &params.osc2_solo, &mut states.solo))
            .push(param_row("Pitch", &params.osc2_pitch, &mut states.pitch))
            .push(param_row("Detune", &params.osc2_detune, &mut states.detune))
            .push(param_row("Gain", &params.osc2_gain, &mut states.gain))
            .push(param_row("Pan", &params.osc2_pan, &mut states.pan))
            .push(param_row("Unison", &params.osc2_unison, &mut states.unison))
            .push(param_row(
                "UniDet",
                &params.osc2_unison_detune,
                &mut states.unison_detune,
            ))
            .push(param_row("Phase", &params.osc2_phase, &mut states.phase))
            .push(param_row("Shape", &params.osc2_shape, &mut states.shape))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn osc3_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut OscStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.osc3_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Solo", &params.osc3_solo, &mut states.solo))
            .push(param_row("Pitch", &params.osc3_pitch, &mut states.pitch))
            .push(param_row("Detune", &params.osc3_detune, &mut states.detune))
            .push(param_row("Gain", &params.osc3_gain, &mut states.gain))
            .push(param_row("Pan", &params.osc3_pan, &mut states.pan))
            .push(param_row("Unison", &params.osc3_unison, &mut states.unison))
            .push(param_row(
                "UniDet",
                &params.osc3_unison_detune,
                &mut states.unison_detune,
            ))
            .push(param_row("Phase", &params.osc3_phase, &mut states.phase))
            .push(param_row("Shape", &params.osc3_shape, &mut states.shape))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn filter1_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut FilterStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &params.filter1_type,
                &mut states.filter_type,
            ))
            .push(param_row(
                "Cutoff",
                &params.filter1_cutoff,
                &mut states.cutoff,
            ))
            .push(param_row(
                "Reso",
                &params.filter1_resonance,
                &mut states.resonance,
            ))
            .push(param_row(
                "Width",
                &params.filter1_bandwidth,
                &mut states.bandwidth,
            ))
            .push(param_row(
                "KeyTrk",
                &params.filter1_key_tracking,
                &mut states.key_tracking,
            ))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn filter2_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut FilterStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &params.filter2_type,
                &mut states.filter_type,
            ))
            .push(param_row(
                "Cutoff",
                &params.filter2_cutoff,
                &mut states.cutoff,
            ))
            .push(param_row(
                "Reso",
                &params.filter2_resonance,
                &mut states.resonance,
            ))
            .push(param_row(
                "Width",
                &params.filter2_bandwidth,
                &mut states.bandwidth,
            ))
            .push(param_row(
                "KeyTrk",
                &params.filter2_key_tracking,
                &mut states.key_tracking,
            ))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn filter3_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut FilterStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Type",
                &params.filter3_type,
                &mut states.filter_type,
            ))
            .push(param_row(
                "Cutoff",
                &params.filter3_cutoff,
                &mut states.cutoff,
            ))
            .push(param_row(
                "Reso",
                &params.filter3_resonance,
                &mut states.resonance,
            ))
            .push(param_row(
                "Width",
                &params.filter3_bandwidth,
                &mut states.bandwidth,
            ))
            .push(param_row(
                "KeyTrk",
                &params.filter3_key_tracking,
                &mut states.key_tracking,
            ))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn lfo1_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut LfoStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.lfo1_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Rate", &params.lfo1_rate, &mut states.rate))
            .push(param_row("Depth", &params.lfo1_depth, &mut states.depth))
            .push(param_row(
                "F-Amt",
                &params.lfo1_filter_amount,
                &mut states.filter_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn lfo2_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut LfoStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.lfo2_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Rate", &params.lfo2_rate, &mut states.rate))
            .push(param_row("Depth", &params.lfo2_depth, &mut states.depth))
            .push(param_row(
                "F-Amt",
                &params.lfo2_filter_amount,
                &mut states.filter_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn lfo3_section<'a>(
        title: &str,
        params: &'a DSynthParams,
        states: &'a mut LfoStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new(title).size(16))
            .push(param_row(
                "Wave",
                &params.lfo3_waveform,
                &mut states.waveform,
            ))
            .push(param_row("Rate", &params.lfo3_rate, &mut states.rate))
            .push(param_row("Depth", &params.lfo3_depth, &mut states.depth))
            .push(param_row(
                "F-Amt",
                &params.lfo3_filter_amount,
                &mut states.filter_amount,
            ))
            .spacing(3)
            .padding(8)
    }

    pub(super) fn effects_section<'a>(
        params: &'a DSynthParams,
        states: &'a mut EffectsStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        // Distortion controls
        let distortion = Column::new()
            .push(Text::new("Distortion").size(16))
            .push(param_row(
                "Type",
                &params.distortion_type,
                &mut states.distortion_type,
            ))
            .push(param_row(
                "Drive",
                &params.distortion_drive,
                &mut states.distortion_drive,
            ))
            .push(param_row(
                "Mix",
                &params.distortion_mix,
                &mut states.distortion_mix,
            ))
            .spacing(3)
            .padding(8);

        // Chorus controls
        let chorus = Column::new()
            .push(Text::new("Chorus").size(16))
            .push(param_row(
                "Rate",
                &params.chorus_rate,
                &mut states.chorus_rate,
            ))
            .push(param_row(
                "Depth",
                &params.chorus_depth,
                &mut states.chorus_depth,
            ))
            .push(param_row("Mix", &params.chorus_mix, &mut states.chorus_mix))
            .spacing(3)
            .padding(8);

        // Delay controls
        let delay = Column::new()
            .push(Text::new("Delay").size(16))
            .push(param_row(
                "Time",
                &params.delay_time_ms,
                &mut states.delay_time_ms,
            ))
            .push(param_row(
                "Fdbk",
                &params.delay_feedback,
                &mut states.delay_feedback,
            ))
            .push(param_row("Wet", &params.delay_wet, &mut states.delay_wet))
            .push(param_row("Dry", &params.delay_dry, &mut states.delay_dry))
            .spacing(3)
            .padding(8);

        // Reverb controls
        let reverb = Column::new()
            .push(Text::new("Reverb").size(16))
            .push(param_row(
                "Room",
                &params.reverb_room_size,
                &mut states.reverb_room_size,
            ))
            .push(param_row(
                "Damp",
                &params.reverb_damping,
                &mut states.reverb_damping,
            ))
            .push(param_row("Wet", &params.reverb_wet, &mut states.reverb_wet))
            .push(param_row("Dry", &params.reverb_dry, &mut states.reverb_dry))
            .push(param_row(
                "Width",
                &params.reverb_width,
                &mut states.reverb_width,
            ))
            .spacing(3)
            .padding(8);

        Column::new()
            .push(Text::new("EFFECTS").size(18))
            .push(Text::new("Distortion → Chorus → Delay → Reverb").size(12))
            .push(
                Row::new()
                    .push(distortion)
                    .push(chorus)
                    .push(delay)
                    .push(reverb)
                    .spacing(10),
            )
            .spacing(5)
            .padding(10)
    }

    pub(super) fn envelope_section<'a>(
        params: &'a DSynthParams,
        states: &'a mut EnvelopeStates,
    ) -> widget::Column<'a, Message> {
        use super::helpers::param_row;
        use widget::*;

        Column::new()
            .push(Text::new("ENVELOPE (ADSR)").size(18))
            .push(param_row(
                "Attack",
                &params.envelope_attack,
                &mut states.attack,
            ))
            .push(param_row(
                "Decay",
                &params.envelope_decay,
                &mut states.decay,
            ))
            .push(param_row(
                "Sustain",
                &params.envelope_sustain,
                &mut states.sustain,
            ))
            .push(param_row(
                "Release",
                &params.envelope_release,
                &mut states.release,
            ))
            .spacing(5)
            .padding(10)
    }
}
