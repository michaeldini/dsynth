/// Parameter Update Mechanism for CLAP
///
/// This module provides thread-safe parameter synchronization between the CLAP host
/// and the audio processing thread using the triple-buffer pattern.
///
/// Design:
/// - GUI/Host Thread → triple-buffer → Audio Thread (lock-free parameter updates)
/// - Audio Thread → automation queue → Host (for modulation visualization)  
/// - Update interval: ~32 samples (~0.7ms at 44.1kHz) for responsiveness
///
/// Note: This is a simplified version for CLAP that works with the existing
/// triple-buffer infrastructure. Parameter updates from the host should be
/// applied to a SynthParams struct and written to the triple-buffer producer.
use super::param_descriptor::ParamId;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Holds parameter updates with timing information
#[derive(Clone)]
pub struct ParamUpdate {
    pub param_id: ParamId,
    /// Normalized value (0.0-1.0)
    pub normalized_value: f32,
    /// Sample index within the processing buffer
    pub sample_index: u32,
}

/// Thread-safe parameter update buffer
///
/// Manages outbound automation updates (audio→host for modulation visualization).
/// For inbound updates (host→audio), use the existing triple-buffer pattern directly.
pub struct ParamUpdateBuffer {
    /// Queue for parameter changes to report back to host
    /// Used for parameter modulation (e.g., LFO modulating cutoff)
    automation_queue: Arc<Mutex<VecDeque<ParamUpdate>>>,
}

impl ParamUpdateBuffer {
    /// Create a new parameter update buffer
    pub fn new() -> Self {
        Self {
            automation_queue: Arc::new(Mutex::new(VecDeque::with_capacity(64))),
        }
    }

    /// Queue an automation update to be sent back to the host
    ///
    /// Called from the audio thread when a parameter is modulated by LFO, envelope, etc.
    /// The host can use this for automation recording or visualization.
    pub fn queue_automation(&self, param_id: ParamId, normalized_value: f32, sample_index: u32) {
        if let Ok(mut queue) = self.automation_queue.lock() {
            // Limit queue size to prevent unbounded growth
            if queue.len() < 1000 {
                queue.push_back(ParamUpdate {
                    param_id,
                    normalized_value,
                    sample_index,
                });
            }
        }
    }

    /// Poll automation updates to send to host
    ///
    /// Called from the host processing callback to retrieve any parameter
    /// changes that occurred during audio processing.
    pub fn poll_automation_updates(&self) -> Vec<ParamUpdate> {
        if let Ok(mut queue) = self.automation_queue.lock() {
            queue.drain(..).collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for ParamUpdateBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for applying parameter changes to SynthParams
pub mod param_apply {
    use super::super::param_descriptor::*;
    use super::ParamId;
    use crate::params::SynthParams;

    /// Apply a normalized parameter value (0.0-1.0) to a SynthParams struct
    pub fn apply_param(params: &mut SynthParams, param_id: ParamId, normalized: f32) {
        let denorm = {
            let registry = super::super::param_registry::get_registry();
            if let Some(desc) = registry.get(param_id) {
                desc.denormalize(normalized)
            } else {
                normalized // Fallback
            }
        };

        match param_id {
            // Master
            PARAM_MASTER_GAIN => params.master_gain = denorm,
            PARAM_MONOPHONIC => params.monophonic = denorm > 0.5,

            // Oscillator 1
            PARAM_OSC1_WAVEFORM => {
                if let Some(wf) = denorm_to_waveform(denorm) {
                    params.oscillators[0].waveform = wf;
                }
            }
            PARAM_OSC1_PITCH => params.oscillators[0].pitch = denorm.clamp(-24.0, 24.0),
            PARAM_OSC1_DETUNE => params.oscillators[0].detune = denorm,
            PARAM_OSC1_GAIN => params.oscillators[0].gain = denorm,
            PARAM_OSC1_PAN => params.oscillators[0].pan = denorm,
            PARAM_OSC1_UNISON => params.oscillators[0].unison = (denorm as usize).max(1).min(7),
            PARAM_OSC1_UNISON_DETUNE => params.oscillators[0].unison_detune = denorm,
            PARAM_OSC1_PHASE => params.oscillators[0].phase = denorm,
            PARAM_OSC1_SHAPE => params.oscillators[0].shape = denorm,
            PARAM_OSC1_FM_SOURCE => {
                // Integer param: 0 = None (no FM), 1 = Some(0) (Osc 1 self-mod),
                // 2 = Some(1) (Osc 2 → Osc 1 feedback), 3 = Some(2) (Osc 3 → Osc 1 feedback)
                let val = denorm.round() as i32;
                params.oscillators[0].fm_source = match val {
                    0 => None,    // No FM
                    1 => Some(0), // Osc 1 self-modulation (1-sample feedback)
                    2 => Some(1), // Osc 2 modulates Osc 1 (feedback)
                    3 => Some(2), // Osc 3 modulates Osc 1 (feedback)
                    _ => None,
                };
            }
            PARAM_OSC1_FM_AMOUNT => params.oscillators[0].fm_amount = denorm,
            PARAM_OSC1_H1..=PARAM_OSC1_H8 => {
                let idx = (param_id - PARAM_OSC1_H1) as usize;
                if idx < 8 {
                    params.oscillators[0].additive_harmonics[idx] = denorm;
                }
            }
            PARAM_OSC1_SOLO => params.oscillators[0].solo = denorm > 0.5,
            PARAM_OSC1_WAVETABLE_INDEX => {
                params.oscillators[0].wavetable_index = denorm.round() as usize;
            }
            PARAM_OSC1_WAVETABLE_POSITION => {
                params.oscillators[0].wavetable_position = denorm.clamp(0.0, 1.0);
            }

            // Oscillator 2
            PARAM_OSC2_WAVEFORM => {
                if let Some(wf) = denorm_to_waveform(denorm) {
                    params.oscillators[1].waveform = wf;
                }
            }
            PARAM_OSC2_PITCH => params.oscillators[1].pitch = denorm.clamp(-24.0, 24.0),
            PARAM_OSC2_DETUNE => params.oscillators[1].detune = denorm,
            PARAM_OSC2_GAIN => params.oscillators[1].gain = denorm,
            PARAM_OSC2_PAN => params.oscillators[1].pan = denorm,
            PARAM_OSC2_UNISON => params.oscillators[1].unison = (denorm as usize).max(1).min(7),
            PARAM_OSC2_UNISON_DETUNE => params.oscillators[1].unison_detune = denorm,
            PARAM_OSC2_PHASE => params.oscillators[1].phase = denorm,
            PARAM_OSC2_SHAPE => params.oscillators[1].shape = denorm,
            PARAM_OSC2_FM_SOURCE => {
                // Integer param: 0 = None (no FM), 1 = Some(0) (use Osc 1),
                // 2 = Some(1) (Osc 2 self-mod), 3 = Some(2) (Osc 3 → Osc 2 feedback)
                let val = denorm.round() as i32;
                params.oscillators[1].fm_source = match val {
                    0 => None,    // No FM
                    1 => Some(0), // Osc 1 modulates Osc 2
                    2 => Some(1), // Osc 2 self-modulation (1-sample feedback)
                    3 => Some(2), // Osc 3 modulates Osc 2 (feedback)
                    _ => None,
                };
            }
            PARAM_OSC2_FM_AMOUNT => params.oscillators[1].fm_amount = denorm,
            PARAM_OSC2_H1..=PARAM_OSC2_H8 => {
                let idx = (param_id - PARAM_OSC2_H1) as usize;
                if idx < 8 {
                    params.oscillators[1].additive_harmonics[idx] = denorm;
                }
            }
            PARAM_OSC2_SOLO => params.oscillators[1].solo = denorm > 0.5,
            PARAM_OSC2_WAVETABLE_INDEX => {
                params.oscillators[1].wavetable_index = denorm.round() as usize;
            }
            PARAM_OSC2_WAVETABLE_POSITION => {
                params.oscillators[1].wavetable_position = denorm.clamp(0.0, 1.0);
            }

            // Oscillator 3
            PARAM_OSC3_WAVEFORM => {
                if let Some(wf) = denorm_to_waveform(denorm) {
                    params.oscillators[2].waveform = wf;
                }
            }
            PARAM_OSC3_PITCH => params.oscillators[2].pitch = denorm.clamp(-24.0, 24.0),
            PARAM_OSC3_DETUNE => params.oscillators[2].detune = denorm,
            PARAM_OSC3_GAIN => params.oscillators[2].gain = denorm,
            PARAM_OSC3_PAN => params.oscillators[2].pan = denorm,
            PARAM_OSC3_UNISON => params.oscillators[2].unison = (denorm as usize).max(1).min(7),
            PARAM_OSC3_UNISON_DETUNE => params.oscillators[2].unison_detune = denorm,
            PARAM_OSC3_PHASE => params.oscillators[2].phase = denorm,
            PARAM_OSC3_SHAPE => params.oscillators[2].shape = denorm,
            PARAM_OSC3_FM_SOURCE => {
                // Integer param: 0 = None (no FM), 1 = Some(0) (use Osc 1),
                // 2 = Some(1) (use Osc 2), 3 = Some(2) (Osc 3 self-mod)
                let val = denorm.round() as i32;
                params.oscillators[2].fm_source = match val {
                    0 => None,    // No FM
                    1 => Some(0), // Osc 1 modulates Osc 3
                    2 => Some(1), // Osc 2 modulates Osc 3
                    3 => Some(2), // Osc 3 self-modulation (1-sample feedback)
                    _ => None,
                };
            }
            PARAM_OSC3_FM_AMOUNT => params.oscillators[2].fm_amount = denorm,
            PARAM_OSC3_H1..=PARAM_OSC3_H8 => {
                let idx = (param_id - PARAM_OSC3_H1) as usize;
                if idx < 8 {
                    params.oscillators[2].additive_harmonics[idx] = denorm;
                }
            }
            PARAM_OSC3_SOLO => params.oscillators[2].solo = denorm > 0.5,
            PARAM_OSC3_WAVETABLE_INDEX => {
                params.oscillators[2].wavetable_index = denorm.round() as usize;
            }
            PARAM_OSC3_WAVETABLE_POSITION => {
                params.oscillators[2].wavetable_position = denorm.clamp(0.0, 1.0);
            }

            // Filters
            PARAM_FILTER1_TYPE => {
                if let Some(ft) = denorm_to_filter_type(denorm) {
                    params.filters[0].filter_type = ft;
                }
            }
            PARAM_FILTER1_CUTOFF => params.filters[0].cutoff = denorm,
            PARAM_FILTER1_RESONANCE => params.filters[0].resonance = denorm,
            PARAM_FILTER1_BANDWIDTH => params.filters[0].bandwidth = denorm,
            PARAM_FILTER1_KEY_TRACKING => params.filters[0].key_tracking = denorm,

            PARAM_FILTER2_TYPE => {
                if let Some(ft) = denorm_to_filter_type(denorm) {
                    params.filters[1].filter_type = ft;
                }
            }
            PARAM_FILTER2_CUTOFF => params.filters[1].cutoff = denorm,
            PARAM_FILTER2_RESONANCE => params.filters[1].resonance = denorm,
            PARAM_FILTER2_BANDWIDTH => params.filters[1].bandwidth = denorm,
            PARAM_FILTER2_KEY_TRACKING => params.filters[1].key_tracking = denorm,

            PARAM_FILTER3_TYPE => {
                if let Some(ft) = denorm_to_filter_type(denorm) {
                    params.filters[2].filter_type = ft;
                }
            }
            PARAM_FILTER3_CUTOFF => params.filters[2].cutoff = denorm,
            PARAM_FILTER3_RESONANCE => params.filters[2].resonance = denorm,
            PARAM_FILTER3_BANDWIDTH => params.filters[2].bandwidth = denorm,
            PARAM_FILTER3_KEY_TRACKING => params.filters[2].key_tracking = denorm,

            // Filter 1 Envelope
            PARAM_FILTER1_ENV_ATTACK => params.filters[0].envelope.attack = denorm,
            PARAM_FILTER1_ENV_DECAY => params.filters[0].envelope.decay = denorm,
            PARAM_FILTER1_ENV_SUSTAIN => params.filters[0].envelope.sustain = denorm,
            PARAM_FILTER1_ENV_RELEASE => params.filters[0].envelope.release = denorm,
            PARAM_FILTER1_ENV_AMOUNT => params.filters[0].envelope.amount = denorm,

            // Filter 2 Envelope
            PARAM_FILTER2_ENV_ATTACK => params.filters[1].envelope.attack = denorm,
            PARAM_FILTER2_ENV_DECAY => params.filters[1].envelope.decay = denorm,
            PARAM_FILTER2_ENV_SUSTAIN => params.filters[1].envelope.sustain = denorm,
            PARAM_FILTER2_ENV_RELEASE => params.filters[1].envelope.release = denorm,
            PARAM_FILTER2_ENV_AMOUNT => params.filters[1].envelope.amount = denorm,

            // Filter 3 Envelope
            PARAM_FILTER3_ENV_ATTACK => params.filters[2].envelope.attack = denorm,
            PARAM_FILTER3_ENV_DECAY => params.filters[2].envelope.decay = denorm,
            PARAM_FILTER3_ENV_SUSTAIN => params.filters[2].envelope.sustain = denorm,
            PARAM_FILTER3_ENV_RELEASE => params.filters[2].envelope.release = denorm,
            PARAM_FILTER3_ENV_AMOUNT => params.filters[2].envelope.amount = denorm,

            // LFOs
            PARAM_LFO1_WAVEFORM => {
                if let Some(lw) = denorm_to_lfo_waveform(denorm) {
                    params.lfos[0].waveform = lw;
                }
            }
            PARAM_LFO1_RATE => params.lfos[0].rate = denorm,
            PARAM_LFO1_DEPTH => params.lfos[0].depth = denorm,
            PARAM_LFO1_FILTER_AMOUNT => params.lfos[0].filter_amount = denorm,
            PARAM_LFO1_PITCH_AMOUNT => params.lfos[0].pitch_amount = denorm,
            PARAM_LFO1_GAIN_AMOUNT => params.lfos[0].gain_amount = denorm,
            PARAM_LFO1_PAN_AMOUNT => params.lfos[0].pan_amount = denorm,
            PARAM_LFO1_PWM_AMOUNT => params.lfos[0].pwm_amount = denorm,

            PARAM_LFO2_WAVEFORM => {
                if let Some(lw) = denorm_to_lfo_waveform(denorm) {
                    params.lfos[1].waveform = lw;
                }
            }
            PARAM_LFO2_RATE => params.lfos[1].rate = denorm,
            PARAM_LFO2_DEPTH => params.lfos[1].depth = denorm,
            PARAM_LFO2_FILTER_AMOUNT => params.lfos[1].filter_amount = denorm,
            PARAM_LFO2_PITCH_AMOUNT => params.lfos[1].pitch_amount = denorm,
            PARAM_LFO2_GAIN_AMOUNT => params.lfos[1].gain_amount = denorm,
            PARAM_LFO2_PAN_AMOUNT => params.lfos[1].pan_amount = denorm,
            PARAM_LFO2_PWM_AMOUNT => params.lfos[1].pwm_amount = denorm,

            PARAM_LFO3_WAVEFORM => {
                if let Some(lw) = denorm_to_lfo_waveform(denorm) {
                    params.lfos[2].waveform = lw;
                }
            }
            PARAM_LFO3_RATE => params.lfos[2].rate = denorm,
            PARAM_LFO3_DEPTH => params.lfos[2].depth = denorm,
            PARAM_LFO3_FILTER_AMOUNT => params.lfos[2].filter_amount = denorm,
            PARAM_LFO3_PITCH_AMOUNT => params.lfos[2].pitch_amount = denorm,
            PARAM_LFO3_GAIN_AMOUNT => params.lfos[2].gain_amount = denorm,
            PARAM_LFO3_PAN_AMOUNT => params.lfos[2].pan_amount = denorm,
            PARAM_LFO3_PWM_AMOUNT => params.lfos[2].pwm_amount = denorm,

            // Envelope
            PARAM_ENVELOPE_ATTACK => params.envelope.attack = denorm,
            PARAM_ENVELOPE_DECAY => params.envelope.decay = denorm,
            PARAM_ENVELOPE_SUSTAIN => params.envelope.sustain = denorm,
            PARAM_ENVELOPE_RELEASE => params.envelope.release = denorm,

            // Velocity
            PARAM_VELOCITY_AMP => params.velocity.amp_sensitivity = denorm,
            PARAM_VELOCITY_FILTER => params.velocity.filter_sensitivity = denorm,

            // Effects
            PARAM_REVERB_ROOM_SIZE => params.effects.reverb.room_size = denorm,
            PARAM_REVERB_DAMPING => params.effects.reverb.damping = denorm,
            PARAM_REVERB_WET => params.effects.reverb.wet = denorm,
            PARAM_REVERB_DRY => params.effects.reverb.dry = denorm,
            PARAM_REVERB_WIDTH => params.effects.reverb.width = denorm,
            PARAM_DELAY_TIME_MS => params.effects.delay.time_ms = denorm,
            PARAM_DELAY_FEEDBACK => params.effects.delay.feedback = denorm,
            PARAM_DELAY_WET => params.effects.delay.wet = denorm,
            PARAM_DELAY_DRY => params.effects.delay.dry = denorm,
            PARAM_CHORUS_RATE => params.effects.chorus.rate = denorm,
            PARAM_CHORUS_DEPTH => params.effects.chorus.depth = denorm,
            PARAM_CHORUS_MIX => params.effects.chorus.mix = denorm,
            PARAM_DISTORTION_TYPE => {
                if let Some(dt) = denorm_to_distortion_type(denorm) {
                    params.effects.distortion.dist_type = dt;
                }
            }
            PARAM_DISTORTION_DRIVE => params.effects.distortion.drive = denorm,
            PARAM_DISTORTION_MIX => params.effects.distortion.mix = denorm,

            // Multiband Distortion
            PARAM_MB_DIST_LOW_MID_FREQ => params.effects.multiband_distortion.low_mid_freq = denorm,
            PARAM_MB_DIST_MID_HIGH_FREQ => {
                params.effects.multiband_distortion.mid_high_freq = denorm
            }
            PARAM_MB_DIST_DRIVE_LOW => params.effects.multiband_distortion.drive_low = denorm,
            PARAM_MB_DIST_DRIVE_MID => params.effects.multiband_distortion.drive_mid = denorm,
            PARAM_MB_DIST_DRIVE_HIGH => params.effects.multiband_distortion.drive_high = denorm,
            PARAM_MB_DIST_GAIN_LOW => params.effects.multiband_distortion.gain_low = denorm,
            PARAM_MB_DIST_GAIN_MID => params.effects.multiband_distortion.gain_mid = denorm,
            PARAM_MB_DIST_GAIN_HIGH => params.effects.multiband_distortion.gain_high = denorm,
            PARAM_MB_DIST_MIX => params.effects.multiband_distortion.mix = denorm,

            // Stereo Widener
            PARAM_WIDENER_HAAS_DELAY => params.effects.stereo_widener.haas_delay_ms = denorm,
            PARAM_WIDENER_HAAS_MIX => params.effects.stereo_widener.haas_mix = denorm,
            PARAM_WIDENER_WIDTH => params.effects.stereo_widener.width = denorm,
            PARAM_WIDENER_MID_GAIN => params.effects.stereo_widener.mid_gain = denorm,
            PARAM_WIDENER_SIDE_GAIN => params.effects.stereo_widener.side_gain = denorm,

            // Unison Normalization toggles
            PARAM_OSC1_UNISON_NORMALIZE => params.oscillators[0].unison_normalize = denorm > 0.5,
            PARAM_OSC2_UNISON_NORMALIZE => params.oscillators[1].unison_normalize = denorm > 0.5,
            PARAM_OSC3_UNISON_NORMALIZE => params.oscillators[2].unison_normalize = denorm > 0.5,

            _ => {} // Unknown parameter, ignore
        }
    }

    fn denorm_to_waveform(denorm: f32) -> Option<crate::params::Waveform> {
        use crate::params::Waveform;
        // denorm is already the enum index (0-8) from registry.denormalize()
        match denorm.round() as i32 {
            0 => Some(Waveform::Sine),
            1 => Some(Waveform::Saw),
            2 => Some(Waveform::Square),
            3 => Some(Waveform::Triangle),
            4 => Some(Waveform::Pulse),
            5 => Some(Waveform::WhiteNoise),
            6 => Some(Waveform::PinkNoise),
            7 => Some(Waveform::Additive),
            8 => Some(Waveform::Wavetable),
            _ => None,
        }
    }

    fn denorm_to_filter_type(denorm: f32) -> Option<crate::params::FilterType> {
        use crate::params::FilterType;
        // denorm is already the enum index (0-2) from registry.denormalize()
        match denorm.round() as i32 {
            0 => Some(FilterType::Lowpass),
            1 => Some(FilterType::Highpass),
            2 => Some(FilterType::Bandpass),
            _ => None,
        }
    }

    fn denorm_to_lfo_waveform(denorm: f32) -> Option<crate::params::LFOWaveform> {
        use crate::params::LFOWaveform;
        // denorm is already the enum index (0-3) from registry.denormalize()
        match denorm.round() as i32 {
            0 => Some(LFOWaveform::Sine),
            1 => Some(LFOWaveform::Triangle),
            2 => Some(LFOWaveform::Square),
            3 => Some(LFOWaveform::Saw),
            _ => None,
        }
    }

    fn denorm_to_distortion_type(denorm: f32) -> Option<crate::params::DistortionType> {
        use crate::params::DistortionType;
        // denorm is already the enum index (0-3) from registry.denormalize()
        match denorm.round() as i32 {
            0 => Some(DistortionType::Tanh),
            1 => Some(DistortionType::SoftClip),
            2 => Some(DistortionType::HardClip),
            3 => Some(DistortionType::Cubic),
            _ => None,
        }
    }
}

/// Helper functions for reading parameter values from SynthParams
pub mod param_get {
    use super::super::param_descriptor::*;
    use super::ParamId;
    use crate::params::SynthParams;

    /// Get a parameter value (denormalized) from a SynthParams struct
    pub fn get_param(params: &SynthParams, param_id: ParamId) -> f32 {
        match param_id {
            // Master
            PARAM_MASTER_GAIN => params.master_gain,
            PARAM_MONOPHONIC => {
                if params.monophonic {
                    1.0
                } else {
                    0.0
                }
            }

            // Oscillator 1
            PARAM_OSC1_WAVEFORM => waveform_to_denorm(params.oscillators[0].waveform),
            PARAM_OSC1_PITCH => params.oscillators[0].pitch,
            PARAM_OSC1_DETUNE => params.oscillators[0].detune,
            PARAM_OSC1_GAIN => params.oscillators[0].gain,
            PARAM_OSC1_PAN => params.oscillators[0].pan,
            PARAM_OSC1_UNISON => params.oscillators[0].unison as f32,
            PARAM_OSC1_UNISON_DETUNE => params.oscillators[0].unison_detune,
            PARAM_OSC1_PHASE => params.oscillators[0].phase,
            PARAM_OSC1_SHAPE => params.oscillators[0].shape,
            PARAM_OSC1_FM_SOURCE => match params.oscillators[0].fm_source {
                None => 0.0,    // No FM
                Some(0) => 1.0, // Osc 1 self-modulation
                Some(1) => 2.0, // Osc 2 → Osc 1 (feedback)
                Some(2) => 3.0, // Osc 3 → Osc 1 (feedback)
                _ => 0.0,
            },
            PARAM_OSC1_FM_AMOUNT => params.oscillators[0].fm_amount,
            PARAM_OSC1_H1..=PARAM_OSC1_H8 => {
                let idx = (param_id - PARAM_OSC1_H1) as usize;
                if idx < 8 {
                    params.oscillators[0].additive_harmonics[idx]
                } else {
                    0.0
                }
            }
            PARAM_OSC1_SOLO => {
                if params.oscillators[0].solo {
                    1.0
                } else {
                    0.0
                }
            }
            PARAM_OSC1_WAVETABLE_INDEX => params.oscillators[0].wavetable_index as f32,
            PARAM_OSC1_WAVETABLE_POSITION => params.oscillators[0].wavetable_position,

            // Oscillator 2
            PARAM_OSC2_WAVEFORM => waveform_to_denorm(params.oscillators[1].waveform),
            PARAM_OSC2_PITCH => params.oscillators[1].pitch,
            PARAM_OSC2_DETUNE => params.oscillators[1].detune,
            PARAM_OSC2_GAIN => params.oscillators[1].gain,
            PARAM_OSC2_PAN => params.oscillators[1].pan,
            PARAM_OSC2_UNISON => params.oscillators[1].unison as f32,
            PARAM_OSC2_UNISON_DETUNE => params.oscillators[1].unison_detune,
            PARAM_OSC2_PHASE => params.oscillators[1].phase,
            PARAM_OSC2_SHAPE => params.oscillators[1].shape,
            PARAM_OSC2_FM_SOURCE => match params.oscillators[1].fm_source {
                None => 0.0,    // No FM
                Some(0) => 1.0, // Osc 1 → Osc 2
                Some(1) => 2.0, // Osc 2 self-modulation
                Some(2) => 3.0, // Osc 3 → Osc 2 (feedback)
                _ => 0.0,
            },
            PARAM_OSC2_FM_AMOUNT => params.oscillators[1].fm_amount,
            PARAM_OSC2_H1..=PARAM_OSC2_H8 => {
                let idx = (param_id - PARAM_OSC2_H1) as usize;
                if idx < 8 {
                    params.oscillators[1].additive_harmonics[idx]
                } else {
                    0.0
                }
            }
            PARAM_OSC2_SOLO => {
                if params.oscillators[1].solo {
                    1.0
                } else {
                    0.0
                }
            }
            PARAM_OSC2_WAVETABLE_INDEX => params.oscillators[1].wavetable_index as f32,
            PARAM_OSC2_WAVETABLE_POSITION => params.oscillators[1].wavetable_position,

            // Oscillator 3
            PARAM_OSC3_WAVEFORM => waveform_to_denorm(params.oscillators[2].waveform),
            PARAM_OSC3_PITCH => params.oscillators[2].pitch,
            PARAM_OSC3_DETUNE => params.oscillators[2].detune,
            PARAM_OSC3_GAIN => params.oscillators[2].gain,
            PARAM_OSC3_PAN => params.oscillators[2].pan,
            PARAM_OSC3_UNISON => params.oscillators[2].unison as f32,
            PARAM_OSC3_UNISON_DETUNE => params.oscillators[2].unison_detune,
            PARAM_OSC3_PHASE => params.oscillators[2].phase,
            PARAM_OSC3_SHAPE => params.oscillators[2].shape,
            PARAM_OSC3_FM_SOURCE => match params.oscillators[2].fm_source {
                None => 0.0,    // No FM
                Some(0) => 1.0, // Osc 1 → Osc 3
                Some(1) => 2.0, // Osc 2 → Osc 3
                Some(2) => 3.0, // Osc 3 self-modulation
                _ => 0.0,
            },
            PARAM_OSC3_FM_AMOUNT => params.oscillators[2].fm_amount,
            PARAM_OSC3_H1..=PARAM_OSC3_H8 => {
                let idx = (param_id - PARAM_OSC3_H1) as usize;
                if idx < 8 {
                    params.oscillators[2].additive_harmonics[idx]
                } else {
                    0.0
                }
            }
            PARAM_OSC3_SOLO => {
                if params.oscillators[2].solo {
                    1.0
                } else {
                    0.0
                }
            }
            PARAM_OSC3_WAVETABLE_INDEX => params.oscillators[2].wavetable_index as f32,
            PARAM_OSC3_WAVETABLE_POSITION => params.oscillators[2].wavetable_position,

            // Filters
            PARAM_FILTER1_TYPE => filter_type_to_denorm(params.filters[0].filter_type),
            PARAM_FILTER1_CUTOFF => params.filters[0].cutoff,
            PARAM_FILTER1_RESONANCE => params.filters[0].resonance,
            PARAM_FILTER1_BANDWIDTH => params.filters[0].bandwidth,
            PARAM_FILTER1_KEY_TRACKING => params.filters[0].key_tracking,

            PARAM_FILTER2_TYPE => filter_type_to_denorm(params.filters[1].filter_type),
            PARAM_FILTER2_CUTOFF => params.filters[1].cutoff,
            PARAM_FILTER2_RESONANCE => params.filters[1].resonance,
            PARAM_FILTER2_BANDWIDTH => params.filters[1].bandwidth,
            PARAM_FILTER2_KEY_TRACKING => params.filters[1].key_tracking,

            PARAM_FILTER3_TYPE => filter_type_to_denorm(params.filters[2].filter_type),
            PARAM_FILTER3_CUTOFF => params.filters[2].cutoff,
            PARAM_FILTER3_RESONANCE => params.filters[2].resonance,
            PARAM_FILTER3_BANDWIDTH => params.filters[2].bandwidth,
            PARAM_FILTER3_KEY_TRACKING => params.filters[2].key_tracking,

            // LFOs
            PARAM_LFO1_WAVEFORM => lfo_waveform_to_denorm(params.lfos[0].waveform),
            PARAM_LFO1_RATE => params.lfos[0].rate,
            PARAM_LFO1_DEPTH => params.lfos[0].depth,
            PARAM_LFO1_FILTER_AMOUNT => params.lfos[0].filter_amount,
            PARAM_LFO1_PITCH_AMOUNT => params.lfos[0].pitch_amount,
            PARAM_LFO1_GAIN_AMOUNT => params.lfos[0].gain_amount,
            PARAM_LFO1_PAN_AMOUNT => params.lfos[0].pan_amount,
            PARAM_LFO1_PWM_AMOUNT => params.lfos[0].pwm_amount,

            PARAM_LFO2_WAVEFORM => lfo_waveform_to_denorm(params.lfos[1].waveform),
            PARAM_LFO2_RATE => params.lfos[1].rate,
            PARAM_LFO2_DEPTH => params.lfos[1].depth,
            PARAM_LFO2_FILTER_AMOUNT => params.lfos[1].filter_amount,
            PARAM_LFO2_PITCH_AMOUNT => params.lfos[1].pitch_amount,
            PARAM_LFO2_GAIN_AMOUNT => params.lfos[1].gain_amount,
            PARAM_LFO2_PAN_AMOUNT => params.lfos[1].pan_amount,
            PARAM_LFO2_PWM_AMOUNT => params.lfos[1].pwm_amount,

            PARAM_LFO3_WAVEFORM => lfo_waveform_to_denorm(params.lfos[2].waveform),
            PARAM_LFO3_RATE => params.lfos[2].rate,
            PARAM_LFO3_DEPTH => params.lfos[2].depth,
            PARAM_LFO3_FILTER_AMOUNT => params.lfos[2].filter_amount,
            PARAM_LFO3_PITCH_AMOUNT => params.lfos[2].pitch_amount,
            PARAM_LFO3_GAIN_AMOUNT => params.lfos[2].gain_amount,
            PARAM_LFO3_PAN_AMOUNT => params.lfos[2].pan_amount,
            PARAM_LFO3_PWM_AMOUNT => params.lfos[2].pwm_amount,

            // Envelope
            PARAM_ENVELOPE_ATTACK => params.envelope.attack,
            PARAM_ENVELOPE_DECAY => params.envelope.decay,
            PARAM_ENVELOPE_SUSTAIN => params.envelope.sustain,
            PARAM_ENVELOPE_RELEASE => params.envelope.release,

            // Velocity
            PARAM_VELOCITY_AMP => params.velocity.amp_sensitivity,
            PARAM_VELOCITY_FILTER => params.velocity.filter_sensitivity,

            // Effects - Reverb
            PARAM_REVERB_ROOM_SIZE => params.effects.reverb.room_size,
            PARAM_REVERB_DAMPING => params.effects.reverb.damping,
            PARAM_REVERB_WET => params.effects.reverb.wet,
            PARAM_REVERB_DRY => params.effects.reverb.dry,
            PARAM_REVERB_WIDTH => params.effects.reverb.width,

            // Effects - Delay
            PARAM_DELAY_TIME_MS => params.effects.delay.time_ms,
            PARAM_DELAY_FEEDBACK => params.effects.delay.feedback,
            PARAM_DELAY_WET => params.effects.delay.wet,
            PARAM_DELAY_DRY => params.effects.delay.dry,

            // Effects - Chorus
            PARAM_CHORUS_RATE => params.effects.chorus.rate,
            PARAM_CHORUS_DEPTH => params.effects.chorus.depth,
            PARAM_CHORUS_MIX => params.effects.chorus.mix,

            // Effects - Distortion
            PARAM_DISTORTION_TYPE => distortion_type_to_denorm(params.effects.distortion.dist_type),
            PARAM_DISTORTION_DRIVE => params.effects.distortion.drive,
            PARAM_DISTORTION_MIX => params.effects.distortion.mix,

            // Effects - Multiband Distortion
            PARAM_MB_DIST_LOW_MID_FREQ => params.effects.multiband_distortion.low_mid_freq,
            PARAM_MB_DIST_MID_HIGH_FREQ => params.effects.multiband_distortion.mid_high_freq,
            PARAM_MB_DIST_DRIVE_LOW => params.effects.multiband_distortion.drive_low,
            PARAM_MB_DIST_DRIVE_MID => params.effects.multiband_distortion.drive_mid,
            PARAM_MB_DIST_DRIVE_HIGH => params.effects.multiband_distortion.drive_high,
            PARAM_MB_DIST_GAIN_LOW => params.effects.multiband_distortion.gain_low,
            PARAM_MB_DIST_GAIN_MID => params.effects.multiband_distortion.gain_mid,
            PARAM_MB_DIST_GAIN_HIGH => params.effects.multiband_distortion.gain_high,
            PARAM_MB_DIST_MIX => params.effects.multiband_distortion.mix,

            // Effects - Stereo Widener
            PARAM_WIDENER_HAAS_DELAY => params.effects.stereo_widener.haas_delay_ms,
            PARAM_WIDENER_HAAS_MIX => params.effects.stereo_widener.haas_mix,
            PARAM_WIDENER_WIDTH => params.effects.stereo_widener.width,
            PARAM_WIDENER_MID_GAIN => params.effects.stereo_widener.mid_gain,
            PARAM_WIDENER_SIDE_GAIN => params.effects.stereo_widener.side_gain,

            // Unison Normalization
            PARAM_OSC1_UNISON_NORMALIZE => {
                if params.oscillators[0].unison_normalize {
                    1.0
                } else {
                    0.0
                }
            }
            PARAM_OSC2_UNISON_NORMALIZE => {
                if params.oscillators[1].unison_normalize {
                    1.0
                } else {
                    0.0
                }
            }
            PARAM_OSC3_UNISON_NORMALIZE => {
                if params.oscillators[2].unison_normalize {
                    1.0
                } else {
                    0.0
                }
            }

            _ => 0.0,
        }
    }

    fn distortion_type_to_denorm(dt: crate::params::DistortionType) -> f32 {
        use crate::params::DistortionType;
        match dt {
            DistortionType::Tanh => 0.0,
            DistortionType::SoftClip => 1.0,
            DistortionType::HardClip => 2.0,
            DistortionType::Cubic => 3.0,
        }
    }

    fn waveform_to_denorm(wf: crate::params::Waveform) -> f32 {
        use crate::params::Waveform;
        // Return enum index (0, 1, 2, ...) which will be normalized by CLAP
        match wf {
            Waveform::Sine => 0.0,
            Waveform::Saw => 1.0,
            Waveform::Square => 2.0,
            Waveform::Triangle => 3.0,
            Waveform::Pulse => 4.0,
            Waveform::WhiteNoise => 5.0,
            Waveform::PinkNoise => 6.0,
            Waveform::Additive => 7.0,
            Waveform::Wavetable => 8.0,
        }
    }

    fn filter_type_to_denorm(ft: crate::params::FilterType) -> f32 {
        use crate::params::FilterType;
        // Return enum index (0, 1, 2) which will be normalized by CLAP
        match ft {
            FilterType::Lowpass => 0.0,
            FilterType::Highpass => 1.0,
            FilterType::Bandpass => 2.0,
        }
    }

    fn lfo_waveform_to_denorm(lw: crate::params::LFOWaveform) -> f32 {
        use crate::params::LFOWaveform;
        // Return enum index (0, 1, 2, 3) which will be normalized by CLAP
        match lw {
            LFOWaveform::Sine => 0.0,
            LFOWaveform::Triangle => 1.0,
            LFOWaveform::Square => 2.0,
            LFOWaveform::Saw => 3.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::SynthParams;

    #[test]
    fn test_param_update_buffer_creation() {
        let buffer = ParamUpdateBuffer::new();
        assert!(buffer.automation_queue.lock().unwrap().is_empty());
    }

    #[test]
    fn test_automation_queue() {
        let buffer = ParamUpdateBuffer::new();

        // Queue some automation updates
        buffer.queue_automation(0x12345678, 0.5, 0);
        buffer.queue_automation(0x87654321, 0.75, 10);

        // Poll them back
        let updates = buffer.poll_automation_updates();
        assert_eq!(updates.len(), 2);
        assert!((updates[0].normalized_value - 0.5).abs() < 0.01);
        assert!((updates[1].normalized_value - 0.75).abs() < 0.01);

        // Queue should be empty now
        let updates2 = buffer.poll_automation_updates();
        assert_eq!(updates2.len(), 0);
    }

    #[test]
    fn test_param_apply() {
        use super::super::param_descriptor::PARAM_MASTER_GAIN;
        let mut params = SynthParams::default();
        param_apply::apply_param(&mut params, PARAM_MASTER_GAIN, 0.5);
        // Should not crash
    }
}
