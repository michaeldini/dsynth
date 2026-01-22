/// Pitch Quantizer - Snaps detected pitches to musical scales
///
/// This is the core of auto-tune/pitch correction. It takes a raw detected pitch
/// (from YIN algorithm) and quantizes it to the nearest note in the selected scale.
///
/// Features:
/// - Multiple scale types (Chromatic, Major, Minor, Pentatonic)
/// - Configurable root note (C, C#, D, etc.)
/// - Smooth pitch correction with adjustable retune speed
/// - Correction amount for blend between raw/corrected

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleType {
    Chromatic,       // All 12 semitones (no correction, just smoothing)
    Major,           // Major scale: 0,2,4,5,7,9,11 (Do-Re-Mi-Fa-Sol-La-Ti)
    Minor,           // Natural minor: 0,2,3,5,7,8,10 (La-Ti-Do-Re-Mi-Fa-Sol)
    Pentatonic,      // Major pentatonic: 0,2,4,7,9 (Do-Re-Mi-Sol-La)
    MinorPentatonic, // Minor pentatonic: 0,3,5,7,10
}

impl ScaleType {
    /// Returns the semitone intervals for this scale (within one octave)
    pub fn intervals(&self) -> &[u8] {
        match self {
            ScaleType::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            ScaleType::Major => &[0, 2, 4, 5, 7, 9, 11],
            ScaleType::Minor => &[0, 2, 3, 5, 7, 8, 10],
            ScaleType::Pentatonic => &[0, 2, 4, 7, 9],
            ScaleType::MinorPentatonic => &[0, 3, 5, 7, 10],
        }
    }
}

/// Root note for scale (C=0, C#=1, D=2, etc.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RootNote(pub u8); // 0-11 (C to B)

impl RootNote {
    pub const C: RootNote = RootNote(0);
    pub const C_SHARP: RootNote = RootNote(1);
    pub const D: RootNote = RootNote(2);
    pub const D_SHARP: RootNote = RootNote(3);
    pub const E: RootNote = RootNote(4);
    pub const F: RootNote = RootNote(5);
    pub const F_SHARP: RootNote = RootNote(6);
    pub const G: RootNote = RootNote(7);
    pub const G_SHARP: RootNote = RootNote(8);
    pub const A: RootNote = RootNote(9);
    pub const A_SHARP: RootNote = RootNote(10);
    pub const B: RootNote = RootNote(11);
}

pub struct PitchQuantizer {
    sample_rate: f32,

    // Quantization settings
    scale_type: ScaleType,
    root_note: RootNote,

    // Smoothing/correction settings
    retune_speed: f32,      // 0.0-1.0 (0=instant/robotic, 1=slow/natural)
    correction_amount: f32, // 0.0-1.0 (0=off, 1=full correction)

    // State
    current_corrected_pitch: f32, // Smoothed corrected pitch
    target_corrected_pitch: f32,  // Target quantized pitch
}

impl PitchQuantizer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            scale_type: ScaleType::Chromatic,
            root_note: RootNote::C,
            retune_speed: 0.5,      // Default: moderate correction speed
            correction_amount: 0.0, // Default: off (no correction)
            current_corrected_pitch: 440.0,
            target_corrected_pitch: 440.0,
        }
    }

    /// Set the scale type for quantization
    pub fn set_scale_type(&mut self, scale_type: ScaleType) {
        self.scale_type = scale_type;
    }

    /// Set the root note for the scale
    pub fn set_root_note(&mut self, root_note: RootNote) {
        self.root_note = root_note;
    }

    /// Set retune speed (0.0 = instant/robotic, 1.0 = very slow/natural)
    /// Typical values:
    /// - 0.0: T-Pain/Cher effect (instant snap)
    /// - 0.3-0.5: Noticeable correction (pop vocal)
    /// - 0.7-0.9: Subtle/natural correction
    pub fn set_retune_speed(&mut self, speed: f32) {
        self.retune_speed = speed.clamp(0.0, 1.0);
    }

    /// Set correction amount (0.0 = no correction, 1.0 = full correction)
    pub fn set_correction_amount(&mut self, amount: f32) {
        self.correction_amount = amount.clamp(0.0, 1.0);
    }

    /// Quantize a detected pitch to the nearest scale note
    ///
    /// Algorithm:
    /// 1. Convert frequency to MIDI note number
    /// 2. Find nearest scale degree
    /// 3. Convert back to frequency
    /// 4. Smooth towards target pitch at retune speed
    /// 5. Blend with raw pitch based on correction amount
    pub fn quantize(&mut self, raw_pitch_hz: f32) -> f32 {
        if self.correction_amount <= 0.0001 {
            // No correction - pass through
            return raw_pitch_hz;
        }

        // 1. Convert Hz to MIDI note number (fractional)
        let raw_midi_note = Self::hz_to_midi(raw_pitch_hz);

        // 2. Find nearest scale note
        let quantized_midi_note = self.quantize_to_scale(raw_midi_note);

        // 3. Convert back to Hz
        self.target_corrected_pitch = Self::midi_to_hz(quantized_midi_note);

        // 4. Smooth towards target at retune speed
        // Convert retune speed to time constant: 0.0 = instant, 1.0 = very slow
        // Use exponential smoothing with time-dependent coefficient
        let time_constant_ms = self.retune_speed * 200.0; // 0-200ms
        let smoothing_coefficient = if time_constant_ms < 0.1 {
            0.0 // Instant snap
        } else {
            // Calculate alpha for exponential moving average
            // alpha = 1 - exp(-dt / tau), where dt = sample period, tau = time constant
            let dt_ms = 1000.0 / self.sample_rate;
            let tau_ms = time_constant_ms;
            (-dt_ms / tau_ms).exp()
        };

        // Smooth current towards target
        self.current_corrected_pitch = smoothing_coefficient * self.current_corrected_pitch
            + (1.0 - smoothing_coefficient) * self.target_corrected_pitch;

        // 5. Blend between raw and corrected based on correction amount
        let corrected_pitch = self.correction_amount * self.current_corrected_pitch
            + (1.0 - self.correction_amount) * raw_pitch_hz;

        corrected_pitch
    }

    /// Quantize MIDI note to nearest scale degree
    fn quantize_to_scale(&self, raw_midi_note: f32) -> f32 {
        // Get scale intervals
        let intervals = self.scale_type.intervals();

        // Extract octave and note within octave
        let octave = (raw_midi_note / 12.0).floor();
        let note_in_octave = raw_midi_note - octave * 12.0;

        // Adjust for root note
        let adjusted_note = (note_in_octave - self.root_note.0 as f32 + 12.0) % 12.0;

        // Find nearest scale degree
        let mut nearest_interval = intervals[0] as f32;
        let mut min_distance = (adjusted_note - nearest_interval).abs();

        for &interval in intervals.iter() {
            let distance = (adjusted_note - interval as f32).abs();
            if distance < min_distance {
                min_distance = distance;
                nearest_interval = interval as f32;
            }

            // Also check wrapped distance (e.g., 11 semitones vs. -1 semitone)
            let wrapped_distance = (adjusted_note - (interval as f32 + 12.0)).abs();
            if wrapped_distance < min_distance {
                min_distance = wrapped_distance;
                nearest_interval = interval as f32 + 12.0;
            }
        }

        // Convert back to absolute MIDI note
        let quantized_note_in_octave = (nearest_interval + self.root_note.0 as f32) % 12.0;
        let quantized_midi_note = octave * 12.0 + quantized_note_in_octave;

        quantized_midi_note
    }

    /// Convert frequency (Hz) to MIDI note number (A4 = 440Hz = MIDI 69)
    fn hz_to_midi(hz: f32) -> f32 {
        69.0 + 12.0 * (hz / 440.0).log2()
    }

    /// Convert MIDI note number to frequency (Hz)
    fn midi_to_hz(midi: f32) -> f32 {
        440.0 * 2.0_f32.powf((midi - 69.0) / 12.0)
    }

    /// Reset internal state
    pub fn reset(&mut self) {
        self.current_corrected_pitch = 440.0;
        self.target_corrected_pitch = 440.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_hz_to_midi_conversion() {
        // A4 = 440Hz = MIDI 69
        assert_relative_eq!(PitchQuantizer::hz_to_midi(440.0), 69.0, epsilon = 0.01);

        // C4 = 261.63Hz = MIDI 60
        assert_relative_eq!(PitchQuantizer::hz_to_midi(261.63), 60.0, epsilon = 0.01);

        // A3 = 220Hz = MIDI 57
        assert_relative_eq!(PitchQuantizer::hz_to_midi(220.0), 57.0, epsilon = 0.01);
    }

    #[test]
    fn test_midi_to_hz_conversion() {
        // MIDI 69 = A4 = 440Hz
        assert_relative_eq!(PitchQuantizer::midi_to_hz(69.0), 440.0, epsilon = 0.01);

        // MIDI 60 = C4 = 261.63Hz
        assert_relative_eq!(PitchQuantizer::midi_to_hz(60.0), 261.63, epsilon = 0.01);

        // MIDI 81 = A5 = 880Hz (octave up)
        assert_relative_eq!(PitchQuantizer::midi_to_hz(81.0), 880.0, epsilon = 0.01);
    }

    #[test]
    fn test_chromatic_scale_no_correction() {
        let mut quantizer = PitchQuantizer::new(44100.0);
        quantizer.set_scale_type(ScaleType::Chromatic);
        quantizer.set_correction_amount(1.0);
        quantizer.set_retune_speed(0.0); // Instant

        // Chromatic scale should quantize to nearest semitone
        let input_hz = 445.0; // Slightly sharp A4
        let corrected = quantizer.quantize(input_hz);

        // Should snap to A4 (440Hz)
        assert_relative_eq!(corrected, 440.0, epsilon = 1.0);
    }

    #[test]
    fn test_major_scale_quantization() {
        let mut quantizer = PitchQuantizer::new(44100.0);
        quantizer.set_scale_type(ScaleType::Major);
        quantizer.set_root_note(RootNote::C);
        quantizer.set_correction_amount(1.0);
        quantizer.set_retune_speed(0.0); // Instant

        // C# (277.18Hz, MIDI 61) should snap to either C (261.63Hz) or D (293.66Hz)
        // It's closer to C (0.5 semitones down vs 1.5 semitones up)
        let c_sharp_hz = 277.18;
        let corrected = quantizer.quantize(c_sharp_hz);

        // Should snap to C4 (261.63Hz) or D4 (293.66Hz) - both are in C major
        let c4_hz = 261.63;
        let d4_hz = 293.66;
        assert!(
            (corrected - c4_hz).abs() < 2.0 || (corrected - d4_hz).abs() < 2.0,
            "Expected {} to be close to {} or {}, got {}",
            c_sharp_hz,
            c4_hz,
            d4_hz,
            corrected
        );
    }

    #[test]
    fn test_correction_amount_blending() {
        let mut quantizer = PitchQuantizer::new(44100.0);
        quantizer.set_scale_type(ScaleType::Chromatic);
        quantizer.set_retune_speed(0.0); // Instant

        let raw_pitch = 445.0; // Slightly sharp A4

        // 0% correction = pass through
        quantizer.set_correction_amount(0.0);
        let no_correction = quantizer.quantize(raw_pitch);
        assert_relative_eq!(no_correction, raw_pitch, epsilon = 0.1);

        // 100% correction = full snap to 440Hz
        quantizer.reset();
        quantizer.set_correction_amount(1.0);
        let full_correction = quantizer.quantize(raw_pitch);
        assert_relative_eq!(full_correction, 440.0, epsilon = 1.0);
    }

    #[test]
    fn test_retune_speed_smoothing() {
        let mut quantizer = PitchQuantizer::new(44100.0);
        quantizer.set_scale_type(ScaleType::Chromatic);
        quantizer.set_correction_amount(1.0);
        quantizer.set_retune_speed(0.9); // Very slow

        // Start at A4, move to A5 (octave up)
        quantizer.quantize(440.0); // Initialize state

        // Jump to A5 - with slow retune, should not snap instantly
        let first_frame = quantizer.quantize(880.0);
        assert!(
            first_frame < 880.0,
            "Should not snap instantly with slow retune"
        );
        assert!(first_frame > 440.0, "Should start moving towards target");
    }
}
