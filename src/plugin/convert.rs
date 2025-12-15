use crate::params::{FilterParams, LFOParams, OscillatorParams, SynthParams, VelocityParams};

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
                },
                LFOParams {
                    waveform: p.lfo2_waveform.value(),
                    rate: p.lfo2_rate.value(),
                    depth: p.lfo2_depth.value(),
                    filter_amount: p.lfo2_filter_amount.value(), // `SynthParams::lfos[].filter_amount` is in Hz (same as the standalone engine params)
                },
                LFOParams {
                    waveform: p.lfo3_waveform.value(),
                    rate: p.lfo3_rate.value(),
                    depth: p.lfo3_depth.value(),
                    filter_amount: p.lfo3_filter_amount.value(), // `SynthParams::lfos[].filter_amount` is in Hz (same as the standalone engine params)
                },
            ],

            velocity: VelocityParams {
                amp_sensitivity: p.velocity_amp.value(),
                filter_sensitivity: p.velocity_filter.value(),
            },
        }
    }
}
