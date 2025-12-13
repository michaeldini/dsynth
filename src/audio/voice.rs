use crate::dsp::{envelope::Envelope, filter::BiquadFilter, lfo::LFO, oscillator::Oscillator};
use crate::params::{
    FilterEnvelopeParams, FilterParams, LFOParams, OscillatorParams, VelocityParams,
};

/// A single voice combining 3 oscillators, 3 filters, and an envelope
/// Includes RMS tracking for voice-stealing decisions
/// Now supports unison voices per oscillator for thicker sound
/// Filter envelopes and LFOs for modulation
pub struct Voice {
    note: u8,
    velocity: f32,
    is_active: bool,
    sample_rate: f32,

    // DSP components - each oscillator can have up to 7 unison voices
    oscillators: Vec<Vec<Oscillator>>, // [osc_index][unison_index]
    filters: [BiquadFilter; 3],
    envelope: Envelope,
    filter_envelopes: [Envelope; 3],
    lfos: [LFO; 3],

    // RMS tracking for voice stealing using exponential moving average
    // Stores squared samples (before sqrt) for efficiency
    rms_squared_ema: f32,
}

impl Voice {
    /// Create a new voice
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        // Initialize with single oscillator per slot (unison will add more)
        let mut oscillators = Vec::with_capacity(3);
        for _ in 0..3 {
            let mut osc_group = Vec::with_capacity(7); // Max 7 unison voices
            osc_group.push(Oscillator::new(sample_rate));
            oscillators.push(osc_group);
        }

        Self {
            note: 0,
            velocity: 0.0,
            is_active: false,
            sample_rate,
            oscillators,
            filters: [
                BiquadFilter::new(sample_rate),
                BiquadFilter::new(sample_rate),
                BiquadFilter::new(sample_rate),
            ],
            envelope: Envelope::new(sample_rate),
            filter_envelopes: [
                Envelope::new(sample_rate),
                Envelope::new(sample_rate),
                Envelope::new(sample_rate),
            ],
            lfos: [
                LFO::new(sample_rate),
                LFO::new(sample_rate),
                LFO::new(sample_rate),
            ],
            rms_squared_ema: 0.0,
        }
    }

    /// Trigger a note on this voice
    ///
    /// # Arguments
    /// * `note` - MIDI note number (0-127)
    /// * `velocity` - Note velocity (0.0-1.0)
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.note = note;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.is_active = true;

        // Trigger envelopes
        self.envelope.note_on();
        for filter_env in &mut self.filter_envelopes {
            filter_env.note_on();
        }

        // Reset LFOs
        for lfo in &mut self.lfos {
            lfo.reset();
        }

        // Reset RMS tracking
        self.rms_squared_ema = 0.0;
    }

    /// Release this voice
    pub fn note_off(&mut self) {
        self.envelope.note_off();
        for filter_env in &mut self.filter_envelopes {
            filter_env.note_off();
        }
    }

    /// Update oscillator and filter parameters
    pub fn update_parameters(
        &mut self,
        osc_params: &[OscillatorParams; 3],
        filter_params: &[FilterParams; 3],
        filter_env_params: &[FilterEnvelopeParams; 3],
        lfo_params: &[LFOParams; 3],
    ) {
        // Convert MIDI note to frequency
        let base_freq = Self::midi_note_to_freq(self.note);

        for i in 0..3 {
            let param = &osc_params[i];

            // Ensure we have the right number of unison voices
            let target_unison = param.unison.clamp(1, 7);
            while self.oscillators[i].len() < target_unison {
                self.oscillators[i].push(Oscillator::new(self.sample_rate));
            }
            while self.oscillators[i].len() > target_unison {
                self.oscillators[i].pop();
            }

            // Calculate base frequency with pitch and detune
            let pitch_mult = 2.0_f32.powf(param.pitch / 12.0);
            let detune_mult = 2.0_f32.powf(param.detune / 1200.0);
            let base_osc_freq = base_freq * pitch_mult * detune_mult;

            // Get unison count before iteration
            let unison_count = self.oscillators[i].len();

            // Configure each unison voice with spread
            for (unison_idx, osc) in self.oscillators[i].iter_mut().enumerate() {
                // Calculate unison detune spread
                let unison_detune = if unison_count > 1 {
                    let spread = param.unison_detune / 100.0; // cents to ratio
                    let offset = (unison_idx as f32 - (unison_count - 1) as f32 / 2.0)
                        / (unison_count - 1) as f32;
                    2.0_f32.powf(offset * spread / 12.0)
                } else {
                    1.0
                };

                let freq = base_osc_freq * unison_detune;
                osc.set_frequency(freq);
                osc.set_waveform(param.waveform);
                osc.set_shape(param.shape);

                // Set phase offset for unison spread
                let phase_offset = param.phase + (unison_idx as f32 / unison_count.max(1) as f32);
                osc.set_phase(phase_offset % 1.0);
            }

            // Update filter parameters (base cutoff, will be modulated during process())
            self.filters[i].set_filter_type(filter_params[i].filter_type);
            self.filters[i].set_cutoff(filter_params[i].cutoff);
            self.filters[i].set_resonance(filter_params[i].resonance);

            // Update filter envelope parameters
            let fenv = &filter_env_params[i];
            self.filter_envelopes[i].set_attack(fenv.attack);
            self.filter_envelopes[i].set_decay(fenv.decay);
            self.filter_envelopes[i].set_sustain(fenv.sustain);
            self.filter_envelopes[i].set_release(fenv.release);

            // Update LFO parameters
            let lfo = &lfo_params[i];
            self.lfos[i].set_rate(lfo.rate);
            self.lfos[i].set_waveform(lfo.waveform);
        }
    }

    /// Convert MIDI note number to frequency in Hz
    fn midi_note_to_freq(note: u8) -> f32 {
        440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
    }

    /// Process one sample
    ///
    /// # Arguments
    /// * `osc_params` - Oscillator parameters for gain control
    /// * `filter_params` - Filter parameters for cutoff
    /// * `filter_env_params` - Filter envelope parameters
    /// * `lfo_params` - LFO parameters
    /// * `velocity_params` - Velocity sensitivity parameters
    ///
    /// # Returns
    /// Mixed output sample
    pub fn process(
        &mut self,
        osc_params: &[OscillatorParams; 3],
        filter_params: &[FilterParams; 3],
        filter_env_params: &[FilterEnvelopeParams; 3],
        lfo_params: &[LFOParams; 3],
        velocity_params: &VelocityParams,
    ) -> f32 {
        if !self.is_active {
            return 0.0;
        }

        let env_value = self.envelope.process();

        // Check if envelope has finished
        if !self.envelope.is_active() {
            self.is_active = false;
            return 0.0;
        }

        // Calculate velocity-sensitive amplitude using standardized formula:
        // output = 1.0 + sensitivity * (velocity - 0.5)
        // At velocity 0.0: 1.0 - 0.5 * sensitivity (quieter)
        // At velocity 0.5: 1.0 (no change)
        // At velocity 1.0: 1.0 + 0.5 * sensitivity (louder)
        let velocity_factor = 1.0 + velocity_params.amp_sensitivity * (self.velocity - 0.5);

        // Process each oscillator group through its filter
        let mut output = 0.0;

        // Check if any oscillator is soloed
        let any_soloed = osc_params.iter().any(|o| o.solo);

        for (i, osc_group) in self.oscillators.iter_mut().enumerate() {
            if osc_group.is_empty() || i >= 3 {
                continue;
            }

            // Skip this oscillator if solo mode is active and this osc is not soloed
            if any_soloed && !osc_params[i].solo {
                continue;
            }

            // Mix all unison voices for this oscillator
            let mut osc_sum = 0.0;
            let unison_count = osc_group.len() as f32;

            for osc in osc_group.iter_mut() {
                osc_sum += osc.process();
            }

            // Normalize by unison count to prevent clipping
            let osc_out = osc_sum / unison_count.sqrt();

            // Get filter envelope and LFO values
            let filter_env_value = self.filter_envelopes[i].process();
            let lfo_value = self.lfos[i].process();

            // Calculate modulated filter cutoff
            let base_cutoff = filter_params[i].cutoff;

            // Apply key tracking
            let key_tracking_offset = if filter_params[i].key_tracking > 0.0 {
                let base_note = 60.0; // C4 as reference
                let note_offset = self.note as f32 - base_note;
                let cents_per_note = 100.0; // One semitone = 100 cents
                let cutoff_mult = 2.0_f32
                    .powf((note_offset * cents_per_note * filter_params[i].key_tracking) / 1200.0);
                base_cutoff * (cutoff_mult - 1.0)
            } else {
                0.0
            };

            // Apply velocity to filter envelope amount using standardized formula:
            // output = 1.0 + sensitivity * (velocity - 0.5)
            let env_amount = filter_env_params[i].amount
                * (1.0 + velocity_params.filter_env_sensitivity * (self.velocity - 0.5));

            // Apply velocity to filter cutoff using standardized formula
            // Velocity offset is proportional to base cutoff frequency
            // At velocity 0.0: cutoff is reduced, at velocity 1.0: cutoff is raised
            let velocity_cutoff_offset =
                base_cutoff * velocity_params.filter_sensitivity * (self.velocity - 0.5);

            // Combine all modulations
            let modulated_cutoff = (base_cutoff
                + key_tracking_offset
                + velocity_cutoff_offset
                + filter_env_value * env_amount
                + lfo_value * lfo_params[i].filter_amount * lfo_params[i].depth)
                .clamp(20.0, 20000.0);

            // Update filter cutoff with modulation
            self.filters[i].set_cutoff(modulated_cutoff);

            // Apply filter with drive
            let filtered = self.filters[i].process_with_drive(osc_out, filter_params[i].drive);

            // Apply stereo panning
            let pan = osc_params[i].pan;
            let left_gain = ((1.0 - pan) / 2.0).sqrt();
            let right_gain = ((1.0 + pan) / 2.0).sqrt();
            let panned = filtered * (left_gain + right_gain) * osc_params[i].gain;

            output += panned;
        }

        // Apply envelope and velocity-sensitive amplitude
        output = output * env_value * velocity_factor;

        // Update RMS using exponential moving average
        // Alpha = 0.01 gives ~100 sample window, good balance of responsiveness and smoothness
        const RMS_ALPHA: f32 = 0.01;
        let squared = output * output;
        self.rms_squared_ema = self.rms_squared_ema * (1.0 - RMS_ALPHA) + squared * RMS_ALPHA;

        output
    }

    /// Get current RMS level for voice stealing
    /// Returns the square root of the exponentially-weighted squared samples
    pub fn get_rms(&self) -> f32 {
        self.rms_squared_ema.sqrt()
    }

    /// Check if voice is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get the MIDI note number for this voice
    pub fn note(&self) -> u8 {
        self.note
    }

    /// Reset voice state
    pub fn reset(&mut self) {
        self.is_active = false;
        self.note = 0;
        self.velocity = 0.0;
        self.envelope.reset();

        for filter_env in &mut self.filter_envelopes {
            filter_env.reset();
        }

        for lfo in &mut self.lfos {
            lfo.reset();
        }

        for osc_group in &mut self.oscillators {
            for osc in osc_group.iter_mut() {
                osc.reset();
            }
        }

        for filter in &mut self.filters {
            filter.reset();
        }

        self.rms_squared_ema = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn default_osc_params() -> [OscillatorParams; 3] {
        [OscillatorParams::default(); 3]
    }

    fn default_filter_params() -> [FilterParams; 3] {
        [FilterParams::default(); 3]
    }

    fn default_filter_env_params() -> [FilterEnvelopeParams; 3] {
        [FilterEnvelopeParams::default(); 3]
    }

    fn default_lfo_params() -> [LFOParams; 3] {
        [LFOParams::default(); 3]
    }

    fn default_velocity_params() -> VelocityParams {
        VelocityParams::default()
    }

    #[test]
    fn test_voice_creation() {
        let voice = Voice::new(44100.0);
        assert!(!voice.is_active());
        assert_eq!(voice.note(), 0);
    }

    #[test]
    fn test_note_on_activates_voice() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60, 0.8);

        assert!(voice.is_active());
        assert_eq!(voice.note(), 60);
        assert_relative_eq!(voice.velocity, 0.8, epsilon = 0.001);
    }

    #[test]
    fn test_velocity_clamping() {
        let mut voice = Voice::new(44100.0);

        voice.note_on(60, 1.5);
        assert_eq!(voice.velocity, 1.0);

        voice.note_on(60, -0.5);
        assert_eq!(voice.velocity, 0.0);
    }

    #[test]
    fn test_midi_note_to_freq() {
        // A4 = 440 Hz
        assert_relative_eq!(Voice::midi_note_to_freq(69), 440.0, epsilon = 0.01);

        // C4 = ~261.63 Hz
        assert_relative_eq!(Voice::midi_note_to_freq(60), 261.63, epsilon = 0.01);

        // A5 = 880 Hz (one octave up)
        assert_relative_eq!(Voice::midi_note_to_freq(81), 880.0, epsilon = 0.01);
    }

    #[test]
    fn test_voice_produces_output() {
        let mut voice = Voice::new(44100.0);
        let osc_params = default_osc_params();
        let filter_params = default_filter_params();
        let filter_env_params = default_filter_env_params();
        let lfo_params = default_lfo_params();
        let velocity_params = default_velocity_params();

        voice.note_on(60, 0.8);
        voice.update_parameters(&osc_params, &filter_params, &filter_env_params, &lfo_params);

        // Process some samples
        let mut found_nonzero = false;
        for _ in 0..1000 {
            let sample = voice.process(
                &osc_params,
                &filter_params,
                &filter_env_params,
                &lfo_params,
                &velocity_params,
            );
            if sample.abs() > 0.001 {
                found_nonzero = true;
                break;
            }
        }

        assert!(found_nonzero, "Voice should produce non-zero output");
    }

    #[test]
    fn test_voice_stops_after_release() {
        let mut voice = Voice::new(44100.0);
        let osc_params = default_osc_params();
        let filter_params = default_filter_params();
        let filter_env_params = default_filter_env_params();
        let lfo_params = default_lfo_params();
        let velocity_params = default_velocity_params();

        voice.note_on(60, 0.8);
        voice.update_parameters(&osc_params, &filter_params, &filter_env_params, &lfo_params);

        // Process to sustain
        for _ in 0..5000 {
            voice.process(
                &osc_params,
                &filter_params,
                &filter_env_params,
                &lfo_params,
                &velocity_params,
            );
        }

        assert!(voice.is_active());

        // Release
        voice.note_off();

        // Process through release (should eventually become inactive)
        let mut became_inactive = false;
        for _ in 0..20000 {
            voice.process(
                &osc_params,
                &filter_params,
                &filter_env_params,
                &lfo_params,
                &velocity_params,
            );
            if !voice.is_active() {
                became_inactive = true;
                break;
            }
        }

        assert!(
            became_inactive,
            "Voice should become inactive after release"
        );
    }

    #[test]
    fn test_inactive_voice_produces_no_output() {
        let voice = Voice::new(44100.0);
        let osc_params = default_osc_params();
        let filter_params = default_filter_params();
        let filter_env_params = default_filter_env_params();
        let lfo_params = default_lfo_params();
        let velocity_params = default_velocity_params();

        // Inactive voice should produce zero
        let mut voice_mut = voice;
        let output = voice_mut.process(
            &osc_params,
            &filter_params,
            &filter_env_params,
            &lfo_params,
            &velocity_params,
        );
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_rms_tracking() {
        let mut voice = Voice::new(44100.0);
        let osc_params = default_osc_params();
        let filter_params = default_filter_params();
        let filter_env_params = default_filter_env_params();
        let lfo_params = default_lfo_params();
        let velocity_params = default_velocity_params();

        voice.note_on(60, 0.8);
        voice.update_parameters(&osc_params, &filter_params, &filter_env_params, &lfo_params);

        // Process enough samples for RMS to update
        for _ in 0..256 {
            voice.process(
                &osc_params,
                &filter_params,
                &filter_env_params,
                &lfo_params,
                &velocity_params,
            );
        }

        // RMS should be non-zero for active voice
        assert!(voice.get_rms() > 0.0, "RMS should be > 0 for active voice");
    }

    #[test]
    fn test_reset() {
        let mut voice = Voice::new(44100.0);
        let osc_params = default_osc_params();
        let filter_params = default_filter_params();
        let filter_env_params = default_filter_env_params();
        let lfo_params = default_lfo_params();
        let velocity_params = default_velocity_params();

        voice.note_on(60, 0.8);
        voice.update_parameters(&osc_params, &filter_params, &filter_env_params, &lfo_params);

        for _ in 0..100 {
            voice.process(
                &osc_params,
                &filter_params,
                &filter_env_params,
                &lfo_params,
                &velocity_params,
            );
        }

        voice.reset();

        assert!(!voice.is_active());
        assert_eq!(voice.note(), 0);
        assert_eq!(voice.velocity, 0.0);
        assert_eq!(voice.get_rms(), 0.0);
    }
}
