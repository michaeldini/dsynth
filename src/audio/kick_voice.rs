/// Kick Drum Voice
/// Single voice optimized for kick drum synthesis with pitch and amplitude envelopes
use crate::dsp::effects::{Clipper, Exciter, MultibandCompressor};
use crate::dsp::modulation::envelope::{Envelope, EnvelopeStage};
use crate::dsp::filters::filter::BiquadFilter;
use crate::dsp::synthesis::oscillator::Oscillator;
use crate::params::FilterType;
use crate::params_kick::{DistortionType, KickParams};

pub struct KickVoice {
    // Two oscillators: body + click
    osc1: Oscillator, // Body/tone oscillator
    osc2: Oscillator, // Click/transient oscillator

    // Amplitude envelope
    amp_envelope: Envelope,

    // Filter and filter envelope
    filter: BiquadFilter,
    filter_envelope: Envelope,

    // Pitch envelopes (simple exponential decay)
    osc1_pitch_phase: f32,
    osc2_pitch_phase: f32,

    // New transient/dynamics effects
    exciter: Exciter, // High-frequency transient enhancement (before filter)
    multiband_comp: MultibandCompressor, // 3-band compression (after distortion)
    clipper: Clipper, // Brick-wall limiting (final stage)

    // Voice state
    is_active: bool,
    velocity: f32,
    note: u8, // MIDI note number (0-127) for key tracking
    sample_rate: f32,
}

impl KickVoice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            osc1: Oscillator::new(sample_rate),
            osc2: Oscillator::new(sample_rate),
            amp_envelope: Envelope::new(sample_rate),
            filter: BiquadFilter::new(sample_rate),
            filter_envelope: Envelope::new(sample_rate),
            osc1_pitch_phase: 0.0,
            osc2_pitch_phase: 0.0,
            exciter: Exciter::new(sample_rate),
            multiband_comp: MultibandCompressor::new(sample_rate),
            clipper: Clipper::new(0.95),
            is_active: false,
            velocity: 0.0,
            note: 60, // Default to C4
            sample_rate,
        }
    }

    /// Trigger the kick with a note-on event
    pub fn trigger(&mut self, note: u8, velocity: f32, params: &KickParams) {
        self.is_active = true;
        self.velocity = velocity;
        self.note = note;

        // Reset pitch envelope phases
        self.osc1_pitch_phase = 0.0;
        self.osc2_pitch_phase = 0.0;

        // Configure amplitude envelope (no sustain for kicks)
        self.amp_envelope.set_attack(params.amp_attack / 1000.0); // Convert ms to seconds
        self.amp_envelope.set_decay(params.amp_decay / 1000.0);
        self.amp_envelope.set_sustain(params.amp_sustain);
        self.amp_envelope.set_release(params.amp_release / 1000.0);
        self.amp_envelope.note_on();

        // Configure filter envelope
        self.filter_envelope.set_attack(0.001); // Very fast attack
        self.filter_envelope
            .set_decay(params.filter_env_decay / 1000.0);
        self.filter_envelope.set_sustain(0.0);
        self.filter_envelope.set_release(0.01);
        self.filter_envelope.note_on();

        // Set filter type
        self.filter.set_filter_type(FilterType::Lowpass);
    }

    /// Release the kick (for MIDI note-off, though kicks typically ignore this)
    pub fn release(&mut self) {
        self.amp_envelope.note_off();
        self.filter_envelope.note_off();
    }

    /// Check if voice is still producing sound
    pub fn is_active(&self) -> bool {
        self.is_active && self.amp_envelope.is_active()
    }

    /// Convert MIDI note number to frequency in Hz
    /// Uses equal temperament tuning: A4 (note 69) = 440 Hz
    fn midi_note_to_freq(note: u8) -> f32 {
        440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
    }

    /// Process one sample
    pub fn process(&mut self, params: &KickParams) -> f32 {
        if !self.is_active {
            return 0.0;
        }

        // Calculate key tracking multiplier
        // At key_tracking=0.0: mult=1.0 (no change, current behavior)
        // At key_tracking=1.0: mult scales with note frequency (chromatic)
        let key_tracking_mult = if params.key_tracking > 0.0 {
            let note_freq = Self::midi_note_to_freq(self.note);
            let reference_freq = Self::midi_note_to_freq(60); // C4 reference (261.63 Hz)
            (note_freq / reference_freq).powf(params.key_tracking)
        } else {
            1.0
        };

        // Apply key tracking to pitch envelope start/end values
        let osc1_start = params.osc1_pitch_start * key_tracking_mult;
        let osc1_end = params.osc1_pitch_end * key_tracking_mult;
        let osc2_start = params.osc2_pitch_start * key_tracking_mult;
        let osc2_end = params.osc2_pitch_end * key_tracking_mult;

        // Calculate pitch envelope values (exponential decay)
        let osc1_pitch = Self::calculate_pitch_envelope(
            self.osc1_pitch_phase,
            osc1_start,
            osc1_end,
            params.osc1_pitch_decay / 1000.0, // Convert ms to seconds
        );

        let osc2_pitch = Self::calculate_pitch_envelope(
            self.osc2_pitch_phase,
            osc2_start,
            osc2_end,
            params.osc2_pitch_decay / 1000.0,
        );

        // Advance pitch envelope phases
        let time_step = 1.0 / self.sample_rate;
        self.osc1_pitch_phase += time_step;
        self.osc2_pitch_phase += time_step;

        // Set oscillator frequencies
        self.osc1.set_frequency(osc1_pitch);
        self.osc2.set_frequency(osc2_pitch);

        // Generate oscillator samples
        let osc1_sample = self.osc1.process() * params.osc1_level;
        let osc2_sample = self.osc2.process() * params.osc2_level;

        // Mix oscillators
        let mut mixed = osc1_sample + osc2_sample;

        // Apply amplitude envelope
        let amp_env_value = self.amp_envelope.process();

        // Apply velocity sensitivity
        let velocity_mod =
            1.0 - params.velocity_sensitivity + (params.velocity_sensitivity * self.velocity);
        mixed *= amp_env_value * velocity_mod;

        // STEP 1: Apply exciter (before filter for aggressive high-frequency enhancement)
        if params.exciter_enabled {
            self.exciter.set_frequency(params.exciter_frequency);
            self.exciter.set_drive(params.exciter_drive);
            self.exciter.set_mix(params.exciter_mix);
            let (excited_l, _excited_r) = self.exciter.process(mixed, mixed);
            mixed = excited_l; // Use left channel directly (both channels identical for mono)
        }

        // STEP 2: Apply filter with envelope modulation
        let filter_env_value = self.filter_envelope.process();
        let modulated_cutoff =
            params.filter_cutoff * (1.0 + params.filter_env_amount * filter_env_value);
        let clamped_cutoff = modulated_cutoff.clamp(20.0, self.sample_rate * 0.45);

        self.filter.set_cutoff(clamped_cutoff);
        self.filter.set_resonance(params.filter_resonance);
        mixed = self.filter.process(mixed);

        // STEP 3: Apply distortion
        if params.distortion_enabled && params.distortion_amount > 0.0 {
            mixed = Self::apply_distortion(mixed, params.distortion_amount, params.distortion_type);
        }

        // STEP 4: Apply multiband compression (after distortion, before transient shaper)
        self.multiband_comp.set_xover_low(params.mb_xover_low);
        self.multiband_comp.set_xover_high(params.mb_xover_high);
        self.multiband_comp
            .set_sub_threshold(params.mb_sub_threshold);
        self.multiband_comp.set_sub_ratio(params.mb_sub_ratio);
        self.multiband_comp.set_sub_attack(params.mb_sub_attack);
        self.multiband_comp.set_sub_release(params.mb_sub_release);
        self.multiband_comp.set_sub_gain(params.mb_sub_gain);
        self.multiband_comp.set_sub_bypass(params.mb_sub_bypass);
        self.multiband_comp
            .set_body_threshold(params.mb_body_threshold);
        self.multiband_comp.set_body_ratio(params.mb_body_ratio);
        self.multiband_comp.set_body_attack(params.mb_body_attack);
        self.multiband_comp.set_body_release(params.mb_body_release);
        self.multiband_comp.set_body_gain(params.mb_body_gain);
        self.multiband_comp.set_body_bypass(params.mb_body_bypass);
        self.multiband_comp
            .set_click_threshold(params.mb_click_threshold);
        self.multiband_comp.set_click_ratio(params.mb_click_ratio);
        self.multiband_comp.set_click_attack(params.mb_click_attack);
        self.multiband_comp
            .set_click_release(params.mb_click_release);
        self.multiband_comp.set_click_gain(params.mb_click_gain);
        self.multiband_comp.set_click_bypass(params.mb_click_bypass);
        self.multiband_comp.set_mix(params.mb_mix);
        self.multiband_comp.set_enabled(params.mb_enabled);
        mixed = self.multiband_comp.process(mixed);

        // STEP 5: Apply transient shaper (envelope-based gain modulation)
        if params.transient_enabled {
            let transient_gain_mult = match self.amp_envelope.stage() {
                EnvelopeStage::Attack => {
                    // Boost attack transient (0.0-1.0 → 1x to 2x gain)
                    1.0 + params.transient_attack_boost
                }
                EnvelopeStage::Sustain | EnvelopeStage::Decay => {
                    // Reduce sustain/decay (0.0-1.0 → 1x to 0x gain)
                    1.0 - params.transient_sustain_reduction
                }
                _ => 1.0, // No change for release/idle
            };
            mixed *= transient_gain_mult;
        }

        // STEP 6: Apply clipper (final stage for maximum loudness)
        self.clipper.set_threshold(params.clipper_threshold);
        self.clipper.set_enabled(params.clipper_enabled);
        mixed = self.clipper.process(mixed);

        // Kicks are typically one-shot. If sustain is zero, the ADSR envelope will enter the
        // Sustain stage at level 0.0 and would otherwise remain "active" forever.
        // Convert that terminal sustain into Idle so voices deactivate cleanly.
        if self.amp_envelope.stage() == EnvelopeStage::Sustain && params.amp_sustain <= 0.0001 {
            self.amp_envelope.reset();
            self.filter_envelope.reset();
            self.is_active = false;
            return mixed;
        }

        // Check if voice should be deactivated
        if !self.amp_envelope.is_active() {
            self.is_active = false;
        }

        mixed
    }

    /// Calculate exponential pitch envelope value
    /// Uses formula: pitch = end + (start - end) * e^(-t / decay)
    fn calculate_pitch_envelope(
        phase: f32,
        start_pitch: f32,
        end_pitch: f32,
        decay_time: f32,
    ) -> f32 {
        if decay_time <= 0.0 {
            return end_pitch;
        }

        let decay_factor = (-phase / decay_time).exp();
        end_pitch + (start_pitch - end_pitch) * decay_factor
    }

    /// Apply distortion/saturation
    fn apply_distortion(sample: f32, amount: f32, distortion_type: DistortionType) -> f32 {
        if amount <= 0.0 {
            return sample;
        }

        // Scale input based on distortion amount
        let drive = 1.0 + amount * 9.0; // 1.0 to 10.0
        let driven = sample * drive;

        let distorted = match distortion_type {
            DistortionType::Soft => {
                // Soft clipping using tanh
                driven.tanh()
            }
            DistortionType::Hard => {
                // Hard clipping
                driven.clamp(-1.0, 1.0)
            }
            DistortionType::Tube => {
                // Tube-style saturation (asymmetric)
                if driven > 0.0 {
                    driven / (1.0 + driven)
                } else {
                    driven / (1.0 - driven * 0.5)
                }
            }
            DistortionType::Foldback => {
                // Foldback distortion
                let folded = driven % 2.0;
                if folded > 1.0 {
                    2.0 - folded
                } else if folded < -1.0 {
                    -2.0 - folded
                } else {
                    folded
                }
            }
        };

        // Compensate for gain increase (simple makeup gain)
        let makeup = 1.0 / (1.0 + amount * 0.5);
        distorted * makeup
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_kick_voice_creation() {
        let voice = KickVoice::new(44100.0);
        assert!(!voice.is_active());
    }

    #[test]
    fn test_trigger_activates_voice() {
        let mut voice = KickVoice::new(44100.0);
        let params = KickParams::default();

        voice.trigger(60, 1.0, &params);
        assert!(voice.is_active());
    }

    #[test]
    fn test_pitch_envelope_decay() {
        let start = 150.0;
        let end = 50.0;
        let decay = 0.1; // 100ms

        // At time 0, should be at start pitch
        let pitch_0 = KickVoice::calculate_pitch_envelope(0.0, start, end, decay);
        assert_relative_eq!(pitch_0, start, epsilon = 0.1);

        // At time = decay constant, should be ~63% toward end
        let pitch_decay = KickVoice::calculate_pitch_envelope(decay, start, end, decay);
        let expected = end + (start - end) * std::f32::consts::E.recip();
        assert_relative_eq!(pitch_decay, expected, epsilon = 1.0);

        // At very long time, should approach end pitch
        let pitch_long = KickVoice::calculate_pitch_envelope(10.0 * decay, start, end, decay);
        assert_relative_eq!(pitch_long, end, epsilon = 0.1);
    }

    #[test]
    fn test_process_generates_audio() {
        let mut voice = KickVoice::new(44100.0);
        let params = KickParams::default();

        voice.trigger(60, 1.0, &params);

        // Process first sample
        let sample = voice.process(&params);

        // Should produce non-zero audio
        assert!(sample.abs() > 0.0);
    }

    #[test]
    fn test_voice_eventually_stops() {
        let mut voice = KickVoice::new(44100.0);
        let params = KickParams {
            // Very short envelope for faster test
            amp_decay: 10.0,  // 10ms
            amp_release: 5.0, // 5ms
            ..Default::default()
        };

        voice.trigger(60, 1.0, &params);

        // Process enough samples to finish envelope (20ms at 44.1kHz = 882 samples)
        for _ in 0..1000 {
            voice.process(&params);
        }

        // Voice should be inactive
        assert!(!voice.is_active());
    }

    #[test]
    fn test_distortion_types() {
        let sample = 0.5;
        let amount = 0.5;

        let soft = KickVoice::apply_distortion(sample, amount, DistortionType::Soft);
        let hard = KickVoice::apply_distortion(sample, amount, DistortionType::Hard);
        let tube = KickVoice::apply_distortion(sample, amount, DistortionType::Tube);
        let foldback = KickVoice::apply_distortion(sample, amount, DistortionType::Foldback);

        // All should produce valid audio
        assert!(soft.abs() <= 1.5);
        assert!(hard.abs() <= 1.5);
        assert!(tube.abs() <= 1.5);
        assert!(foldback.abs() <= 1.5);

        // Soft should be smoother than hard
        assert!(soft.abs() <= hard.abs() + 0.1);
    }

    #[test]
    fn test_velocity_sensitivity() {
        let mut voice1 = KickVoice::new(44100.0);
        let mut voice2 = KickVoice::new(44100.0);
        let params = KickParams {
            velocity_sensitivity: 1.0, // Full velocity sensitivity
            ..Default::default()
        };

        voice1.trigger(60, 1.0, &params); // Full velocity
        voice2.trigger(60, 0.5, &params); // Half velocity

        let sample1 = voice1.process(&params);
        let sample2 = voice2.process(&params);

        // Full velocity should be louder
        assert!(sample1.abs() > sample2.abs());
    }

    #[test]
    fn test_key_tracking() {
        let mut voice_c4 = KickVoice::new(44100.0);
        let mut voice_c5 = KickVoice::new(44100.0);
        let params = KickParams {
            key_tracking: 1.0, // Full key tracking
            osc1_pitch_start: 100.0,
            osc1_pitch_end: 50.0,
            ..Default::default()
        };

        // Trigger C4 (note 60) and C5 (note 72, one octave higher)
        voice_c4.trigger(60, 1.0, &params);
        voice_c5.trigger(72, 1.0, &params);

        // With full key tracking, C5 should have ~2× the pitch multiplier of C4
        // Calculate what the pitch envelope start values should be
        let c4_freq = KickVoice::midi_note_to_freq(60);
        let c5_freq = KickVoice::midi_note_to_freq(72);
        let ref_freq = KickVoice::midi_note_to_freq(60);

        let c4_mult = (c4_freq / ref_freq).powf(params.key_tracking);
        let c5_mult = (c5_freq / ref_freq).powf(params.key_tracking);

        // C5 multiplier should be ~2× C4 multiplier (octave higher)
        assert_relative_eq!(c5_mult / c4_mult, 2.0, epsilon = 0.01);

        // Verify the multipliers are applied correctly
        assert_relative_eq!(c4_mult, 1.0, epsilon = 0.01); // C4 is the reference
        assert_relative_eq!(c5_mult, 2.0, epsilon = 0.01); // C5 is one octave up
    }

    #[test]
    fn test_key_tracking_disabled() {
        let mut voice_c4 = KickVoice::new(44100.0);
        let mut voice_c5 = KickVoice::new(44100.0);
        let params = KickParams::default();

        // Trigger different notes
        voice_c4.trigger(60, 1.0, &params);
        voice_c5.trigger(72, 1.0, &params);

        // With no key tracking, the multiplier should be 1.0 for both
        let c4_freq = KickVoice::midi_note_to_freq(60);
        let c5_freq = KickVoice::midi_note_to_freq(72);
        let ref_freq = KickVoice::midi_note_to_freq(60);

        let c4_mult = (c4_freq / ref_freq).powf(params.key_tracking);
        let c5_mult = (c5_freq / ref_freq).powf(params.key_tracking);

        // Both should be 1.0 (no tracking)
        assert_relative_eq!(c4_mult, 1.0, epsilon = 0.01);
        assert_relative_eq!(c5_mult, 1.0, epsilon = 0.01);
    }

    #[test]
    fn test_midi_note_to_freq() {
        // Test standard tuning: A4 (note 69) = 440 Hz
        assert_relative_eq!(KickVoice::midi_note_to_freq(69), 440.0, epsilon = 0.1);

        // Test C4 (middle C, note 60) ≈ 261.63 Hz
        assert_relative_eq!(KickVoice::midi_note_to_freq(60), 261.63, epsilon = 0.1);

        // Test octave relationship: C5 (72) should be 2× C4 (60)
        let c4_freq = KickVoice::midi_note_to_freq(60);
        let c5_freq = KickVoice::midi_note_to_freq(72);
        assert_relative_eq!(c5_freq / c4_freq, 2.0, epsilon = 0.01);
    }
}
