/// Voice Enhancer Parameters
///
/// Audio processing chain for vocal enhancement:
/// 1. Input Gain
/// 2. Pitch Detection (mono sum, auto-detected with adaptive smoothing)
/// 3. Noise Gate (remove background noise)
/// 4. Parametric EQ (4-band vocal shaping)
/// 5. Compressor (dynamics control)
/// 6. De-Esser (sibilance reduction)
/// 7. Exciter (harmonic enhancement)
/// 8. Lookahead Limiter (safety ceiling)
/// 9. Output Gain & Dry/Wet Mix
use serde::{Deserialize, Serialize};

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

    // Pitch Detector (1 param - smoothing is now adaptive)
    pub pitch_confidence_threshold: f32, // 0.0 to 1.0 (minimum confidence)

    // Pitch Correction / Auto-Tune (5 params)
    pub pitch_correction_enable: bool, // Enable/disable pitch correction
    pub pitch_correction_scale: u8, // Scale type: 0=Chromatic, 1=Major, 2=Minor, 3=Pentatonic, 4=MinorPentatonic
    pub pitch_correction_root: u8,  // Root note: 0=C, 1=C#, 2=D, ... 11=B
    pub pitch_correction_speed: f32, // 0.0 to 1.0 (0=instant/robotic, 1=slow/natural)
    pub pitch_correction_amount: f32, // 0.0 to 1.0 (0=off, 1=full correction)

    // Exciter (6 params)
    pub exciter_amount: f32,         // 0.0 to 1.0 (drive amount)
    pub exciter_frequency: f32,      // 2kHz to 10kHz (high-pass cutoff)
    pub exciter_harmonics: f32,      // 0.0 to 1.0 (harmonic generation)
    pub exciter_mix: f32,            // 0.0 to 1.0 (dry/wet)
    pub exciter_follow_enable: bool, // Enable pitch tracking
    pub exciter_follow_amount: f32,  // 1.0 to 4.0× (multiply factor above pitch)

    // Vocal Doubler (5 params)
    pub doubler_enable: bool,      // Enable/disable doubler
    pub doubler_delay: f32,        // 5.0 to 15.0ms (delay time)
    pub doubler_detune: f32,       // 0.0 to 10.0 cents (pitch variation)
    pub doubler_stereo_width: f32, // 0.0 to 1.0 (stereo spread)
    pub doubler_mix: f32,          // 0.0 to 1.0 (dry/wet)

    // Vocal Choir (5 params)
    pub choir_enable: bool,       // Enable/disable choir
    pub choir_num_voices: usize,  // 2 to 8 voices
    pub choir_detune: f32,        // 0.0 to 30.0 cents (total spread)
    pub choir_delay_spread: f32,  // 10.0 to 40.0ms (delay range)
    pub choir_stereo_spread: f32, // 0.0 to 1.0 (panning width)
    pub choir_mix: f32,           // 0.0 to 1.0 (dry/wet)

    // Multiband Distortion (9 params)
    pub mb_dist_enable: bool,       // Enable/disable multiband distortion
    pub mb_dist_low_mid_freq: f32,  // 50.0 to 500.0 Hz (bass/mid crossover)
    pub mb_dist_mid_high_freq: f32, // 1000.0 to 8000.0 Hz (mid/high crossover)
    pub mb_dist_drive_low: f32,     // 0.0 to 1.0 (bass drive)
    pub mb_dist_drive_mid: f32,     // 0.0 to 1.0 (mid drive)
    pub mb_dist_drive_high: f32,    // 0.0 to 1.0 (high drive)
    pub mb_dist_gain_low: f32,      // 0.0 to 2.0 (bass output gain)
    pub mb_dist_gain_mid: f32,      // 0.0 to 2.0 (mid output gain)
    pub mb_dist_gain_high: f32,     // 0.0 to 2.0 (high output gain)
    pub mb_dist_mix: f32,           // 0.0 to 1.0 (dry/wet)

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

            // Pitch Detector - balanced settings (smoothing is now adaptive)
            pitch_confidence_threshold: 0.6,

            // Pitch Correction / Auto-Tune - off by default
            pitch_correction_enable: false,
            pitch_correction_scale: 0,    // Chromatic (no scale correction)
            pitch_correction_root: 0,     // C
            pitch_correction_speed: 0.5,  // Moderate retune speed (noticeable but not robotic)
            pitch_correction_amount: 0.0, // Off by default

            // Exciter - subtle enhancement
            exciter_amount: 0.3,
            exciter_frequency: 4000.0,
            exciter_harmonics: 0.5,
            exciter_mix: 0.3,

            // Vocal Doubler - off by default
            doubler_enable: false,
            doubler_delay: 10.0,       // 10ms delay
            doubler_detune: 5.0,       // 5 cents detune
            doubler_stereo_width: 0.7, // 70% stereo width
            doubler_mix: 0.5,          // 50% mix

            // Vocal Choir - off by default
            choir_enable: false,
            choir_num_voices: 4,      // 4 voices
            choir_detune: 15.0,       // 15 cents spread
            choir_delay_spread: 25.0, // 25ms delay spread
            choir_stereo_spread: 0.8, // 80% stereo spread
            choir_mix: 0.5,           // 50% mix
            // Multiband Distortion - off by default
            mb_dist_enable: false,
            mb_dist_low_mid_freq: 200.0,   // 200Hz bass/mid split
            mb_dist_mid_high_freq: 2000.0, // 2kHz mid/high split
            mb_dist_drive_low: 0.3,        // Moderate bass drive
            mb_dist_drive_mid: 0.2,        // Light mid drive
            mb_dist_drive_high: 0.1,       // Subtle high drive
            mb_dist_gain_low: 1.0,         // Unity gain
            mb_dist_gain_mid: 1.0,         // Unity gain
            mb_dist_gain_high: 1.0,        // Unity gain
            mb_dist_mix: 0.5,              // 50% mix
            // Exciter follow
            exciter_follow_enable: false, // Pitch tracking off by default
            exciter_follow_amount: 1.5,   // 1.5× = major 3rd above pitch
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

            pitch_confidence_threshold: 0.7,

            // Pitch correction off for clean vocal
            pitch_correction_enable: false,
            pitch_correction_scale: 0, // Chromatic
            pitch_correction_root: 0,  // C
            pitch_correction_speed: 0.5,
            pitch_correction_amount: 0.0,

            // Light exciter
            exciter_amount: 0.25,
            exciter_frequency: 5000.0,
            exciter_harmonics: 0.3,
            exciter_mix: 0.25,
            // Doubler - subtle for clean vocal
            doubler_enable: false,
            doubler_delay: 10.0,
            doubler_detune: 3.0, // Subtle 3 cents
            doubler_stereo_width: 0.6,
            doubler_mix: 0.3,

            // Choir - off for clean vocal
            choir_enable: false,
            choir_num_voices: 4,
            choir_detune: 12.0,
            choir_delay_spread: 20.0,
            choir_stereo_spread: 0.7,
            choir_mix: 0.4,

            // Multiband Distortion - off for clean vocal
            mb_dist_enable: false,
            mb_dist_low_mid_freq: 200.0,
            mb_dist_mid_high_freq: 2000.0,
            mb_dist_drive_low: 0.2,
            mb_dist_drive_mid: 0.1,
            mb_dist_drive_high: 0.05,
            mb_dist_gain_low: 1.0,
            mb_dist_gain_mid: 1.0,
            mb_dist_gain_high: 1.0,
            mb_dist_mix: 0.3,

            // Exciter follow - off by default
            exciter_follow_enable: false,
            exciter_follow_amount: 1.5,

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

            pitch_confidence_threshold: 0.65,

            // Pitch correction off for radio voice
            pitch_correction_enable: false,
            pitch_correction_scale: 0,
            pitch_correction_root: 0,
            pitch_correction_speed: 0.3, // Faster for radio effect
            pitch_correction_amount: 0.0,

            // Strong exciter
            exciter_amount: 0.5,
            exciter_frequency: 3500.0,
            exciter_harmonics: 0.6,
            exciter_mix: 0.5,
            // Doubler - off for radio (already compressed/processed)
            doubler_enable: false,
            doubler_delay: 10.0,
            doubler_detune: 5.0,
            doubler_stereo_width: 0.7,
            doubler_mix: 0.5,

            // Choir - off for radio
            choir_enable: false,
            choir_num_voices: 4,
            choir_detune: 15.0,
            choir_delay_spread: 25.0,
            choir_stereo_spread: 0.8,
            choir_mix: 0.5,

            // Multiband Distortion - moderate for radio
            mb_dist_enable: false,
            mb_dist_low_mid_freq: 250.0,
            mb_dist_mid_high_freq: 2500.0,
            mb_dist_drive_low: 0.4,
            mb_dist_drive_mid: 0.3,
            mb_dist_drive_high: 0.2,
            mb_dist_gain_low: 1.0,
            mb_dist_gain_mid: 1.0,
            mb_dist_gain_high: 1.0,
            mb_dist_mix: 0.5,

            // Exciter follow - off by default
            exciter_follow_enable: false,
            exciter_follow_amount: 1.5,

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

            pitch_confidence_threshold: 0.7,

            // Pitch correction off for deep bass
            pitch_correction_enable: false,
            pitch_correction_scale: 0,
            pitch_correction_root: 0,
            pitch_correction_speed: 0.5,
            pitch_correction_amount: 0.0,

            // Minimal exciter
            exciter_amount: 0.15,
            exciter_frequency: 6000.0,
            exciter_harmonics: 0.2,
            exciter_mix: 0.15,
            // Doubler - wider for deep bass texture
            doubler_enable: false,
            doubler_delay: 12.0,
            doubler_detune: 7.0, // More detune for bass
            doubler_stereo_width: 0.8,
            doubler_mix: 0.4,

            // Choir - larger ensemble for deep bass
            choir_enable: false,
            choir_num_voices: 6,
            choir_detune: 20.0, // Wider spread for bass
            choir_delay_spread: 30.0,
            choir_stereo_spread: 0.9,
            choir_mix: 0.5,
            // Multiband Distortion - heavy bass for deep_bass
            mb_dist_enable: false,
            mb_dist_low_mid_freq: 150.0, // Lower split for more bass
            mb_dist_mid_high_freq: 1500.0,
            mb_dist_drive_low: 0.6, // Heavy bass saturation
            mb_dist_drive_mid: 0.3,
            mb_dist_drive_high: 0.1,
            mb_dist_gain_low: 1.2, // Boost bass
            mb_dist_gain_mid: 1.0,
            mb_dist_gain_high: 0.9,
            mb_dist_mix: 0.6,

            // Exciter follow - higher multiplier for bass enhancement
            exciter_follow_enable: false,
            exciter_follow_amount: 2.0, // 2× = octave above for bass presence

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
        assert_eq!(params.dry_wet, 1.0);
    }

    #[test]
    fn test_preset_clean_vocal() {
        let params = VoiceParams::preset_clean_vocal();
        assert!(params.gate_threshold < 0.0);
        assert!(params.comp_ratio >= 1.0);
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
        assert!(params.eq_band1_gain > 0.0); // Bass boost
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
    fn test_parameter_ranges() {
        let params = VoiceParams::default();

        // Verify all parameters are within expected ranges
        assert!(params.input_gain >= -12.0 && params.input_gain <= 12.0);
        assert!(params.gate_threshold >= -80.0 && params.gate_threshold <= -20.0);
        assert!(params.gate_ratio >= 1.0 && params.gate_ratio <= 10.0);
        assert!(params.comp_ratio >= 1.0 && params.comp_ratio <= 20.0);
        assert!(params.dry_wet >= 0.0 && params.dry_wet <= 1.0);
    }
}
