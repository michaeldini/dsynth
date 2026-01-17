/// Voice Enhancer Parameters
///
/// Audio processing chain for vocal enhancement with pitch-tracked sub oscillator:
/// 1. Input Gain
/// 2. Pitch Detection (mono sum, auto-detected)
/// 3. Noise Gate (remove background noise)
/// 4. Parametric EQ (4-band vocal shaping)
/// 5. Compressor (dynamics control)
/// 6. De-Esser (sibilance reduction)
/// 7. Sub Oscillator (pitch-tracked bass enhancement with amplitude ramping)
/// 8. Exciter (harmonic enhancement)
/// 9. Lookahead Limiter (safety ceiling)
/// 10. Output Gain & Dry/Wet Mix
use serde::{Deserialize, Serialize};

/// Waveform types for sub oscillator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SubOscWaveform {
    Sine,
    Triangle,
    Square,
    Saw,
}

impl Default for SubOscWaveform {
    fn default() -> Self {
        Self::Sine
    }
}

impl SubOscWaveform {
    pub fn to_index(self) -> usize {
        match self {
            Self::Sine => 0,
            Self::Triangle => 1,
            Self::Square => 2,
            Self::Saw => 3,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Sine,
            1 => Self::Triangle,
            2 => Self::Square,
            3 => Self::Saw,
            _ => Self::Sine,
        }
    }
}

/// Complete parameter set for voice enhancement plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceParams {
    // Input/Output (2 params)
    pub input_gain: f32,  // -12dB to +12dB
    pub output_gain: f32, // -12dB to +12dB

    // Noise Gate (5 params)
    pub gate_threshold: f32, // -80dB to -20dB
    pub gate_ratio: f32,     // 1.0 to 10.0 (expansion ratio)
    pub gate_attack: f32,    // 0.1ms to 50ms
    pub gate_release: f32,   // 10ms to 500ms
    pub gate_hold: f32,      // 0ms to 200ms

    // Parametric EQ - 4 bands (13 params)
    pub eq_band1_freq: f32, // 20Hz to 500Hz (low shelf)
    pub eq_band1_gain: f32, // -12dB to +12dB
    pub eq_band1_q: f32,    // 0.1 to 10.0

    pub eq_band2_freq: f32, // 100Hz to 2kHz (bell)
    pub eq_band2_gain: f32, // -12dB to +12dB
    pub eq_band2_q: f32,    // 0.1 to 10.0

    pub eq_band3_freq: f32, // 1kHz to 8kHz (bell)
    pub eq_band3_gain: f32, // -12dB to +12dB
    pub eq_band3_q: f32,    // 0.1 to 10.0

    pub eq_band4_freq: f32, // 2kHz to 20kHz (high shelf)
    pub eq_band4_gain: f32, // -12dB to +12dB
    pub eq_band4_q: f32,    // 0.1 to 10.0

    pub eq_master_gain: f32, // -12dB to +12dB (output trim)

    // Compressor (6 params)
    pub comp_threshold: f32,   // -40dB to 0dB
    pub comp_ratio: f32,       // 1.0 to 20.0
    pub comp_attack: f32,      // 0.1ms to 100ms
    pub comp_release: f32,     // 10ms to 1000ms
    pub comp_knee: f32,        // 0dB to 12dB (soft knee width)
    pub comp_makeup_gain: f32, // 0dB to +24dB

    // De-Esser (4 params)
    pub deess_threshold: f32, // -40dB to 0dB
    pub deess_frequency: f32, // 4kHz to 10kHz (center frequency)
    pub deess_ratio: f32,     // 1.0 to 10.0
    pub deess_amount: f32,    // 0.0 to 1.0 (dry/wet)

    // Pitch Detector (2 params)
    pub pitch_smoothing: f32,            // 0.0 to 1.0 (smoothing factor)
    pub pitch_confidence_threshold: f32, // 0.0 to 1.0 (minimum confidence)

    // Sub Oscillator (4 params)
    pub sub_octave: f32,              // -2 to 0 (octaves below detected pitch)
    pub sub_level: f32,               // 0.0 to 1.0 (mix level)
    pub sub_waveform: SubOscWaveform, // Sine, Triangle, Square, Saw
    pub sub_ramp_time: f32,           // 1ms to 100ms (amplitude ramping to avoid clicks)

    // Exciter (4 params)
    pub exciter_amount: f32,    // 0.0 to 1.0 (drive amount)
    pub exciter_frequency: f32, // 2kHz to 10kHz (high-pass cutoff)
    pub exciter_harmonics: f32, // 0.0 to 1.0 (harmonic generation)
    pub exciter_mix: f32,       // 0.0 to 1.0 (dry/wet)

    // Master (1 param)
    pub dry_wet: f32, // 0.0 to 1.0 (processed vs original)
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            // Input/Output - unity gain defaults
            input_gain: 0.0,
            output_gain: 0.0,

            // Noise Gate - moderate settings
            gate_threshold: -50.0,
            gate_ratio: 5.0,
            gate_attack: 5.0,
            gate_release: 100.0,
            gate_hold: 50.0,

            // Parametric EQ - flat response (0dB gain)
            eq_band1_freq: 80.0,
            eq_band1_gain: 0.0,
            eq_band1_q: 1.0,

            eq_band2_freq: 400.0,
            eq_band2_gain: 0.0,
            eq_band2_q: 1.0,

            eq_band3_freq: 3000.0,
            eq_band3_gain: 0.0,
            eq_band3_q: 1.0,

            eq_band4_freq: 8000.0,
            eq_band4_gain: 0.0,
            eq_band4_q: 1.0,

            eq_master_gain: 0.0,

            // Compressor - gentle compression
            comp_threshold: -20.0,
            comp_ratio: 3.0,
            comp_attack: 10.0,
            comp_release: 100.0,
            comp_knee: 6.0,
            comp_makeup_gain: 0.0,

            // De-Esser - moderate reduction
            deess_threshold: -25.0,
            deess_frequency: 6000.0,
            deess_ratio: 4.0,
            deess_amount: 0.5,

            // Pitch Detector - balanced settings
            pitch_smoothing: 0.7,
            pitch_confidence_threshold: 0.6,

            // Sub Oscillator - 1 octave down, sine wave
            sub_octave: -1.0,
            sub_level: 0.3,
            sub_waveform: SubOscWaveform::Sine,
            sub_ramp_time: 10.0,

            // Exciter - subtle enhancement
            exciter_amount: 0.3,
            exciter_frequency: 4000.0,
            exciter_harmonics: 0.5,
            exciter_mix: 0.3,

            // Master - 100% wet (fully processed)
            dry_wet: 1.0,
        }
    }
}

impl VoiceParams {
    /// Create a new VoiceParams with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Preset: "Clean Vocal" - transparent processing with light enhancement
    pub fn preset_clean_vocal() -> Self {
        Self {
            input_gain: 0.0,
            output_gain: 0.0,

            // Gentle gate
            gate_threshold: -55.0,
            gate_ratio: 3.0,
            gate_attack: 3.0,
            gate_release: 80.0,
            gate_hold: 30.0,

            // Subtle EQ sculpting
            eq_band1_freq: 100.0,
            eq_band1_gain: -2.0, // Reduce low mud
            eq_band1_q: 1.0,

            eq_band2_freq: 300.0,
            eq_band2_gain: -1.0, // Slight mid reduction
            eq_band2_q: 1.5,

            eq_band3_freq: 3500.0,
            eq_band3_gain: 2.0, // Presence boost
            eq_band3_q: 1.2,

            eq_band4_freq: 10000.0,
            eq_band4_gain: 1.5, // Air
            eq_band4_q: 0.7,

            eq_master_gain: 0.0,

            // Light compression
            comp_threshold: -18.0,
            comp_ratio: 2.5,
            comp_attack: 8.0,
            comp_release: 120.0,
            comp_knee: 8.0,
            comp_makeup_gain: 3.0,

            // Moderate de-essing
            deess_threshold: -22.0,
            deess_frequency: 6500.0,
            deess_ratio: 4.0,
            deess_amount: 0.6,

            pitch_smoothing: 0.8,
            pitch_confidence_threshold: 0.7,

            // Subtle sub
            sub_octave: -1.0,
            sub_level: 0.2,
            sub_waveform: SubOscWaveform::Sine,
            sub_ramp_time: 15.0,

            // Light exciter
            exciter_amount: 0.25,
            exciter_frequency: 5000.0,
            exciter_harmonics: 0.3,
            exciter_mix: 0.25,

            dry_wet: 1.0,
        }
    }

    /// Preset: "Radio Voice" - aggressive processing for broadcast sound
    pub fn preset_radio_voice() -> Self {
        Self {
            input_gain: 3.0,
            output_gain: 0.0,

            // Aggressive gate
            gate_threshold: -45.0,
            gate_ratio: 8.0,
            gate_attack: 2.0,
            gate_release: 60.0,
            gate_hold: 40.0,

            // Sculpted EQ for radio
            eq_band1_freq: 120.0,
            eq_band1_gain: -4.0, // Cut low rumble
            eq_band1_q: 1.0,

            eq_band2_freq: 250.0,
            eq_band2_gain: -3.0, // Reduce boxiness
            eq_band2_q: 2.0,

            eq_band3_freq: 2500.0,
            eq_band3_gain: 4.0, // Strong presence
            eq_band3_q: 1.5,

            eq_band4_freq: 8000.0,
            eq_band4_gain: 3.0, // Clarity
            eq_band4_q: 0.8,

            eq_master_gain: 2.0,

            // Heavy compression
            comp_threshold: -25.0,
            comp_ratio: 6.0,
            comp_attack: 5.0,
            comp_release: 80.0,
            comp_knee: 4.0,
            comp_makeup_gain: 8.0,

            // Strong de-essing
            deess_threshold: -20.0,
            deess_frequency: 7000.0,
            deess_ratio: 6.0,
            deess_amount: 0.8,

            pitch_smoothing: 0.6,
            pitch_confidence_threshold: 0.65,

            // Noticeable sub
            sub_octave: -1.0,
            sub_level: 0.4,
            sub_waveform: SubOscWaveform::Sine,
            sub_ramp_time: 8.0,

            // Strong exciter
            exciter_amount: 0.5,
            exciter_frequency: 3500.0,
            exciter_harmonics: 0.6,
            exciter_mix: 0.5,

            dry_wet: 1.0,
        }
    }

    /// Preset: "Deep Bass" - maximize sub oscillator for bass enhancement
    pub fn preset_deep_bass() -> Self {
        Self {
            input_gain: 0.0,
            output_gain: 0.0,

            // Tight gate
            gate_threshold: -50.0,
            gate_ratio: 6.0,
            gate_attack: 4.0,
            gate_release: 90.0,
            gate_hold: 45.0,

            // Bass-focused EQ
            eq_band1_freq: 60.0,
            eq_band1_gain: 3.0, // Boost sub bass
            eq_band1_q: 0.7,

            eq_band2_freq: 200.0,
            eq_band2_gain: -2.0, // Reduce muddiness
            eq_band2_q: 1.5,

            eq_band3_freq: 3000.0,
            eq_band3_gain: 0.0, // Neutral mids
            eq_band3_q: 1.0,

            eq_band4_freq: 12000.0,
            eq_band4_gain: -1.0, // Slight high cut
            eq_band4_q: 0.7,

            eq_master_gain: 0.0,

            // Moderate compression
            comp_threshold: -22.0,
            comp_ratio: 3.5,
            comp_attack: 12.0,
            comp_release: 150.0,
            comp_knee: 7.0,
            comp_makeup_gain: 4.0,

            // Light de-essing
            deess_threshold: -28.0,
            deess_frequency: 6000.0,
            deess_ratio: 3.0,
            deess_amount: 0.4,

            pitch_smoothing: 0.85, // Smooth pitch tracking
            pitch_confidence_threshold: 0.7,

            // Strong sub oscillator - 2 octaves down
            sub_octave: -2.0,
            sub_level: 0.6,
            sub_waveform: SubOscWaveform::Sine,
            sub_ramp_time: 20.0, // Slower ramp for bass

            // Minimal exciter
            exciter_amount: 0.15,
            exciter_frequency: 6000.0,
            exciter_harmonics: 0.2,
            exciter_mix: 0.15,

            dry_wet: 1.0,
        }
    }

    /// Convert dB to linear gain
    pub fn db_to_gain(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Convert linear gain to dB
    pub fn gain_to_db(gain: f32) -> f32 {
        20.0 * gain.log10()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = VoiceParams::default();
        assert_eq!(params.input_gain, 0.0);
        assert_eq!(params.output_gain, 0.0);
        assert_eq!(params.gate_threshold, -50.0);
        assert_eq!(params.sub_octave, -1.0);
        assert_eq!(params.dry_wet, 1.0);
    }

    #[test]
    fn test_preset_clean_vocal() {
        let params = VoiceParams::preset_clean_vocal();
        assert!(params.gate_threshold < 0.0);
        assert!(params.comp_ratio >= 1.0);
        assert!(params.sub_level > 0.0);
    }

    #[test]
    fn test_preset_radio_voice() {
        let params = VoiceParams::preset_radio_voice();
        assert!(params.comp_ratio > 5.0); // Heavy compression
        assert!(params.eq_band3_gain > 0.0); // Presence boost
    }

    #[test]
    fn test_preset_deep_bass() {
        let params = VoiceParams::preset_deep_bass();
        assert_eq!(params.sub_octave, -2.0); // 2 octaves down
        assert!(params.sub_level > 0.5); // Strong sub
    }

    #[test]
    fn test_db_to_gain() {
        assert!((VoiceParams::db_to_gain(0.0) - 1.0).abs() < 0.001);
        assert!((VoiceParams::db_to_gain(6.0) - 2.0).abs() < 0.01);
        assert!((VoiceParams::db_to_gain(-6.0) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_gain_to_db() {
        assert!((VoiceParams::gain_to_db(1.0) - 0.0).abs() < 0.001);
        assert!((VoiceParams::gain_to_db(2.0) - 6.0).abs() < 0.1);
        assert!((VoiceParams::gain_to_db(0.5) - (-6.0)).abs() < 0.1);
    }

    #[test]
    fn test_sub_waveform_conversion() {
        assert_eq!(SubOscWaveform::Sine.to_index(), 0);
        assert_eq!(SubOscWaveform::Triangle.to_index(), 1);
        assert_eq!(SubOscWaveform::Square.to_index(), 2);
        assert_eq!(SubOscWaveform::Saw.to_index(), 3);

        assert_eq!(SubOscWaveform::from_index(0), SubOscWaveform::Sine);
        assert_eq!(SubOscWaveform::from_index(1), SubOscWaveform::Triangle);
        assert_eq!(SubOscWaveform::from_index(2), SubOscWaveform::Square);
        assert_eq!(SubOscWaveform::from_index(3), SubOscWaveform::Saw);
    }

    #[test]
    fn test_parameter_ranges() {
        let params = VoiceParams::default();

        // Verify all parameters are within expected ranges
        assert!(params.input_gain >= -12.0 && params.input_gain <= 12.0);
        assert!(params.gate_threshold >= -80.0 && params.gate_threshold <= -20.0);
        assert!(params.gate_ratio >= 1.0 && params.gate_ratio <= 10.0);
        assert!(params.comp_ratio >= 1.0 && params.comp_ratio <= 20.0);
        assert!(params.sub_octave >= -2.0 && params.sub_octave <= 0.0);
        assert!(params.sub_level >= 0.0 && params.sub_level <= 1.0);
        assert!(params.dry_wet >= 0.0 && params.dry_wet <= 1.0);
    }
}
