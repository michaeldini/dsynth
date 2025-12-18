use super::PluginGui;
use nih_plug::prelude::*;
use nih_plug_iced::IcedEditor;

impl PluginGui {
    pub(super) fn randomize_params(&self) {
        let params = crate::randomize::randomize_synth_params(&mut rand::thread_rng());
        self.apply_synth_params(&params);
    }

    pub(super) fn apply_synth_params(&self, params: &crate::params::SynthParams) {
        let setter = ParamSetter::new(self.context());
        let p = self.params.as_ref();

        macro_rules! set_param {
            ($param:expr, $value:expr) => {{
                setter.begin_set_parameter($param);
                setter.set_parameter($param, $value);
                setter.end_set_parameter($param);
            }};
        }

        set_param!(&p.master_gain, params.master_gain);
        set_param!(&p.monophonic, params.monophonic);

        for (i, osc) in params.oscillators.iter().enumerate() {
            match i {
                0 => {
                    set_param!(&p.osc1_waveform, osc.waveform);
                    set_param!(&p.osc1_pitch, osc.pitch);
                    set_param!(&p.osc1_detune, osc.detune);
                    set_param!(&p.osc1_gain, osc.gain);
                    set_param!(&p.osc1_pan, osc.pan);
                    set_param!(&p.osc1_unison, osc.unison as i32);
                    set_param!(&p.osc1_unison_detune, osc.unison_detune);
                    set_param!(&p.osc1_phase, osc.phase);
                    set_param!(&p.osc1_shape, osc.shape);
                    set_param!(&p.osc1_solo, osc.solo);
                }
                1 => {
                    set_param!(&p.osc2_waveform, osc.waveform);
                    set_param!(&p.osc2_pitch, osc.pitch);
                    set_param!(&p.osc2_detune, osc.detune);
                    set_param!(&p.osc2_gain, osc.gain);
                    set_param!(&p.osc2_pan, osc.pan);
                    set_param!(&p.osc2_unison, osc.unison as i32);
                    set_param!(&p.osc2_unison_detune, osc.unison_detune);
                    set_param!(&p.osc2_phase, osc.phase);
                    set_param!(&p.osc2_shape, osc.shape);
                    set_param!(&p.osc2_solo, osc.solo);
                }
                2 => {
                    set_param!(&p.osc3_waveform, osc.waveform);
                    set_param!(&p.osc3_pitch, osc.pitch);
                    set_param!(&p.osc3_detune, osc.detune);
                    set_param!(&p.osc3_gain, osc.gain);
                    set_param!(&p.osc3_pan, osc.pan);
                    set_param!(&p.osc3_unison, osc.unison as i32);
                    set_param!(&p.osc3_unison_detune, osc.unison_detune);
                    set_param!(&p.osc3_phase, osc.phase);
                    set_param!(&p.osc3_shape, osc.shape);
                    set_param!(&p.osc3_solo, osc.solo);
                }
                _ => {}
            }
        }

        for (i, filter) in params.filters.iter().enumerate() {
            match i {
                0 => {
                    set_param!(&p.filter1_type, filter.filter_type);
                    set_param!(&p.filter1_cutoff, filter.cutoff);
                    set_param!(&p.filter1_resonance, filter.resonance);
                    set_param!(&p.filter1_bandwidth, filter.bandwidth);
                    set_param!(&p.filter1_key_tracking, filter.key_tracking);
                }
                1 => {
                    set_param!(&p.filter2_type, filter.filter_type);
                    set_param!(&p.filter2_cutoff, filter.cutoff);
                    set_param!(&p.filter2_resonance, filter.resonance);
                    set_param!(&p.filter2_bandwidth, filter.bandwidth);
                    set_param!(&p.filter2_key_tracking, filter.key_tracking);
                }
                2 => {
                    set_param!(&p.filter3_type, filter.filter_type);
                    set_param!(&p.filter3_cutoff, filter.cutoff);
                    set_param!(&p.filter3_resonance, filter.resonance);
                    set_param!(&p.filter3_bandwidth, filter.bandwidth);
                    set_param!(&p.filter3_key_tracking, filter.key_tracking);
                }
                _ => {}
            }
        }

        for (i, lfo) in params.lfos.iter().enumerate() {
            match i {
                0 => {
                    set_param!(&p.lfo1_waveform, lfo.waveform);
                    set_param!(&p.lfo1_rate, lfo.rate);
                    set_param!(&p.lfo1_depth, lfo.depth);
                    set_param!(&p.lfo1_filter_amount, lfo.filter_amount);
                }
                1 => {
                    set_param!(&p.lfo2_waveform, lfo.waveform);
                    set_param!(&p.lfo2_rate, lfo.rate);
                    set_param!(&p.lfo2_depth, lfo.depth);
                    set_param!(&p.lfo2_filter_amount, lfo.filter_amount);
                }
                2 => {
                    set_param!(&p.lfo3_waveform, lfo.waveform);
                    set_param!(&p.lfo3_rate, lfo.rate);
                    set_param!(&p.lfo3_depth, lfo.depth);
                    set_param!(&p.lfo3_filter_amount, lfo.filter_amount);
                }
                _ => {}
            }
        }

        set_param!(&p.velocity_amp, params.velocity.amp_sensitivity);
        set_param!(&p.velocity_filter, params.velocity.filter_sensitivity);
    }
}
