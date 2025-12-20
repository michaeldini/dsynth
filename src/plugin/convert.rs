use crate::params::{
    ChorusParams, DelayParams, DistortionParams, EffectsParams, EnvelopeParams, FilterParams,
    LFOParams, OscillatorParams, ReverbParams, SynthParams, VelocityParams,
};

use super::DSynthPlugin;

impl DSynthPlugin {
    /// Convert DSynthParams to SynthParams for the audio engine
    pub(super) fn convert_params(&self) -> SynthParams {
        let p = &self.params;

        SynthParams {
            master_gain: p.master_gain.value(),
            monophonic: p.monophonic.value(),

            oscillators: [
                OscillatorParams {
                    waveform: p.osc1_waveform.value(),
                    pitch: p.osc1_pitch.value(),
                    detune: p.osc1_detune.value(),
                    gain: p.osc1_gain.value(),
                    pan: p.osc1_pan.value(),
                    unison: p.osc1_unison.value() as usize,
                    unison_detune: p.osc1_unison_detune.value(),
                    phase: p.osc1_phase.value(),
                    shape: p.osc1_shape.value(),
                    solo: p.osc1_solo.value(),
                    fm_source: {
                        let source = p.osc1_fm_source.value();
                        if source < 0 { None } else { Some(source as usize) }
                    },
                    fm_amount: p.osc1_fm_amount.value(),
                    additive_harmonics: [
                        p.osc1_h1.value(),
                        p.osc1_h2.value(),
                        p.osc1_h3.value(),
                        p.osc1_h4.value(),
                        p.osc1_h5.value(),
                        p.osc1_h6.value(),
                        p.osc1_h7.value(),
                        p.osc1_h8.value(),
                    ],
                },
                OscillatorParams {
                    waveform: p.osc2_waveform.value(),
                    pitch: p.osc2_pitch.value(),
                    detune: p.osc2_detune.value(),
                    gain: p.osc2_gain.value(),
                    pan: p.osc2_pan.value(),
                    unison: p.osc2_unison.value() as usize,
                    unison_detune: p.osc2_unison_detune.value(),
                    phase: p.osc2_phase.value(),
                    shape: p.osc2_shape.value(),
                    solo: p.osc2_solo.value(),
                    fm_source: {
                        let source = p.osc2_fm_source.value();
                        if source < 0 { None } else { Some(source as usize) }
                    },
                    fm_amount: p.osc2_fm_amount.value(),
                    additive_harmonics: [
                        p.osc2_h1.value(),
                        p.osc2_h2.value(),
                        p.osc2_h3.value(),
                        p.osc2_h4.value(),
                        p.osc2_h5.value(),
                        p.osc2_h6.value(),
                        p.osc2_h7.value(),
                        p.osc2_h8.value(),
                    ],
                },
                OscillatorParams {
                    waveform: p.osc3_waveform.value(),
                    pitch: p.osc3_pitch.value(),
                    detune: p.osc3_detune.value(),
                    gain: p.osc3_gain.value(),
                    pan: p.osc3_pan.value(),
                    unison: p.osc3_unison.value() as usize,
                    unison_detune: p.osc3_unison_detune.value(),
                    phase: p.osc3_phase.value(),
                    shape: p.osc3_shape.value(),
                    solo: p.osc3_solo.value(),
                    fm_source: {
                        let source = p.osc3_fm_source.value();
                        if source < 0 { None } else { Some(source as usize) }
                    },
                    fm_amount: p.osc3_fm_amount.value(),
                    additive_harmonics: [
                        p.osc3_h1.value(),
                        p.osc3_h2.value(),
                        p.osc3_h3.value(),
                        p.osc3_h4.value(),
                        p.osc3_h5.value(),
                        p.osc3_h6.value(),
                        p.osc3_h7.value(),
                        p.osc3_h8.value(),
                    ],
                },
            ],

            filters: [
                FilterParams {
                    filter_type: p.filter1_type.value(),
                    cutoff: p.filter1_cutoff.value(),
                    resonance: p.filter1_resonance.value(),
                    bandwidth: p.filter1_bandwidth.value(),
                    key_tracking: p.filter1_key_tracking.value(),
                },
                FilterParams {
                    filter_type: p.filter2_type.value(),
                    cutoff: p.filter2_cutoff.value(),
                    resonance: p.filter2_resonance.value(),
                    bandwidth: p.filter2_bandwidth.value(),
                    key_tracking: p.filter2_key_tracking.value(),
                },
                FilterParams {
                    filter_type: p.filter3_type.value(),
                    cutoff: p.filter3_cutoff.value(),
                    resonance: p.filter3_resonance.value(),
                    bandwidth: p.filter3_bandwidth.value(),
                    key_tracking: p.filter3_key_tracking.value(),
                },
            ],

            lfos: [
                LFOParams {
                    waveform: p.lfo1_waveform.value(),
                    rate: p.lfo1_rate.value(),
                    depth: p.lfo1_depth.value(),
                    filter_amount: p.lfo1_filter_amount.value(), // `SynthParams::lfos[].filter_amount` is in Hz (same as the standalone engine params)
                    pitch_amount: p.lfo1_pitch_amount.value(),
                    gain_amount: p.lfo1_gain_amount.value(),
                    pan_amount: p.lfo1_pan_amount.value(),
                    pwm_amount: p.lfo1_pwm_amount.value(),
                },
                LFOParams {
                    waveform: p.lfo2_waveform.value(),
                    rate: p.lfo2_rate.value(),
                    depth: p.lfo2_depth.value(),
                    filter_amount: p.lfo2_filter_amount.value(), // `SynthParams::lfos[].filter_amount` is in Hz (same as the standalone engine params)
                    pitch_amount: p.lfo2_pitch_amount.value(),
                    gain_amount: p.lfo2_gain_amount.value(),
                    pan_amount: p.lfo2_pan_amount.value(),
                    pwm_amount: p.lfo2_pwm_amount.value(),
                },
                LFOParams {
                    waveform: p.lfo3_waveform.value(),
                    rate: p.lfo3_rate.value(),
                    depth: p.lfo3_depth.value(),
                    filter_amount: p.lfo3_filter_amount.value(), // `SynthParams::lfos[].filter_amount` is in Hz (same as the standalone engine params)
                    pitch_amount: p.lfo3_pitch_amount.value(),
                    gain_amount: p.lfo3_gain_amount.value(),
                    pan_amount: p.lfo3_pan_amount.value(),
                    pwm_amount: p.lfo3_pwm_amount.value(),
                },
            ],

            envelope: EnvelopeParams {
                attack: p.envelope_attack.value(),
                decay: p.envelope_decay.value(),
                sustain: p.envelope_sustain.value(),
                release: p.envelope_release.value(),
            },

            velocity: VelocityParams {
                amp_sensitivity: p.velocity_amp.value(),
                filter_sensitivity: p.velocity_filter.value(),
            },

            effects: EffectsParams {
                reverb: ReverbParams {
                    room_size: p.reverb_room_size.value(),
                    damping: p.reverb_damping.value(),
                    wet: p.reverb_wet.value(),
                    dry: p.reverb_dry.value(),
                    width: p.reverb_width.value(),
                },
                delay: DelayParams {
                    time_ms: p.delay_time_ms.value(),
                    feedback: p.delay_feedback.value(),
                    wet: p.delay_wet.value(),
                    dry: p.delay_dry.value(),
                },
                chorus: ChorusParams {
                    rate: p.chorus_rate.value(),
                    depth: p.chorus_depth.value(),
                    mix: p.chorus_mix.value(),
                },
                distortion: DistortionParams {
                    dist_type: p.distortion_type.value(),
                    drive: p.distortion_drive.value(),
                    mix: p.distortion_mix.value(),
                },
            },
        }
    }
}
