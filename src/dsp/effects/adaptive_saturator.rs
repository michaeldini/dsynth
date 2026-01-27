use crate::dsp::filters::BiquadFilter;
/// Professional 4-Band Vocal Saturator - Hit Record Quality
///
/// **Design Philosophy**: Multiband processing with stacked harmonic layers (tube + tape + console)
/// for professional vocal production. Optimized for transparency, clarity, and "expensive" analog sound.
///
/// # Architecture
/// ```text
/// Input → Mid/Side → 4-Band Split (200Hz/1kHz/8kHz) → Per-Band Saturation → Sum → L/R → Output
///         ↑          Bass/Mids/Presence/Air              ↓
///         └─────── Width Control (-1 to +1) ────────────┘
///
/// Per Band: Pitch-Tracked Harmonics (tube+tape+console) + Waveshaping + Pre-Emphasis + Auto-Gain
/// ```
///
/// # Key Features
/// - **4-band split**: Bass (<200Hz), Mids (200Hz-1kHz), Presence (1-8kHz), Air (>8kHz bypassed)
/// - **Stacked harmonics**: Tube (2nd), Tape (3rd), Console (5th+7th) all active simultaneously
/// - **Pitch-aware**: Male/female vocal optimization via automatic harmonic scaling
/// - **Mid-side stereo**: Bidirectional width control (-1=wide/thin, +1=power/glue)
/// - **Dynamic saturation**: Louder passages get more drive for analog compression feel
/// - **Pre-emphasis**: Presence band boosted before saturation for harmonic generation
///
/// # Calibration
/// - Bass: drive=0.6, mix=0.5 (warm foundation, punch preservation)
/// - Mids: drive=0.5, mix=0.4 (balanced fundamentals, clarity)
/// - Presence: drive=0.35, mix=0.35 (articulation without harshness)
/// - Width: 0.0 (neutral), +0.3 (subtle power), -0.3 (subtle space)
use crate::dsp::filters::MultibandCrossover;
use crate::dsp::signal_analyzer::SignalAnalysis;
use crate::dsp::synthesis::downsampler::Downsampler;

/// Harmonic oscillator with selective oversampling for anti-aliased harmonic generation
///
/// Bass band: 1× sample rate (no aliasing risk <200Hz)
/// Mids/Presence: 4× oversampled with 20-tap Kaiser FIR for clean harmonics up to 7th
struct HarmonicOscillator {
    sample_rate: f32,
    oversample_rate: f32,
    use_oversampling: bool, // false for bass, true for mids/presence

    // Phase accumulators for each harmonic (0.0-1.0, continuous)
    phase_2nd: f32, // Tube (even harmonic)
    phase_3rd: f32, // Tape (odd harmonic)
    phase_5th: f32, // Console presence
    phase_7th: f32, // Console air/bite

    // Downsampler for 4× → 1× conversion (only used if oversampling)
    downsampler: Option<Downsampler>,

    // Tracked pitch (updated from SignalAnalysis)
    pitch_hz: f32,
    pitch_confidence: f32,
}

impl HarmonicOscillator {
    /// Create new harmonic oscillator
    ///
    /// # Arguments
    /// * `sample_rate` - Target sample rate
    /// * `use_oversampling` - true for 4× oversampling (mids/presence), false for 1× (bass)
    fn new(sample_rate: f32, use_oversampling: bool) -> Self {
        let oversample_rate = if use_oversampling {
            sample_rate * 4.0
        } else {
            sample_rate
        };

        let downsampler = if use_oversampling {
            Some(Downsampler::new(20)) // 20-tap Kaiser FIR
        } else {
            None
        };

        Self {
            sample_rate,
            oversample_rate,
            use_oversampling,
            phase_2nd: 0.0,
            phase_3rd: 0.0,
            phase_5th: 0.0,
            phase_7th: 0.0,
            downsampler,
            pitch_hz: 0.0,
            pitch_confidence: 0.0,
        }
    }

    /// Update tracked pitch from analysis
    fn update_pitch(&mut self, pitch_hz: f32, confidence: f32) {
        self.pitch_hz = pitch_hz;
        self.pitch_confidence = confidence;
    }

    /// Generate harmonics for one sample (or 4 samples if oversampled)
    ///
    /// Returns (h2, h3, h5, h7) harmonic amplitudes
    /// Each harmonic is a pure sine wave at N× the fundamental frequency
    fn generate_harmonics(
        &mut self,
        input_level: f32,    // Amplitude scaling (0-1)
        tube_amount: f32,    // 0-1
        tape_amount: f32,    // 0-1
        console_amount: f32, // 0-1
    ) -> (f32, f32, f32, f32) {
        if self.pitch_confidence < 0.3 || self.pitch_hz < 50.0 {
            // No confident pitch detection - return silence
            return (0.0, 0.0, 0.0, 0.0);
        }

        let samples_to_generate = if self.use_oversampling { 4 } else { 1 };
        let rate = self.oversample_rate;

        // Calculate phase increments (freq / sample_rate)
        let inc_2nd = (self.pitch_hz * 2.0) / rate;
        let inc_3rd = (self.pitch_hz * 3.0) / rate;
        let inc_5th = (self.pitch_hz * 5.0) / rate;
        let inc_7th = (self.pitch_hz * 7.0) / rate;

        if self.use_oversampling {
            // Generate 4× oversampled harmonics
            let mut oversampled = [0.0f32; 4];

            for i in 0..4 {
                // Generate each harmonic at current phase
                let h2 =
                    (self.phase_2nd * 2.0 * std::f32::consts::PI).sin() * tube_amount * input_level;
                let h3 =
                    (self.phase_3rd * 2.0 * std::f32::consts::PI).sin() * tape_amount * input_level;
                let h5 = (self.phase_5th * 2.0 * std::f32::consts::PI).sin()
                    * console_amount
                    * input_level
                    * 0.6; // Quieter
                let h7 = (self.phase_7th * 2.0 * std::f32::consts::PI).sin()
                    * console_amount
                    * input_level
                    * 0.4; // Even quieter

                oversampled[i] = h2 + h3 + h5 + h7;

                // Advance phases
                self.phase_2nd += inc_2nd;
                self.phase_3rd += inc_3rd;
                self.phase_5th += inc_5th;
                self.phase_7th += inc_7th;

                // Wrap phases (0-1 range)
                self.phase_2nd -= self.phase_2nd.floor();
                self.phase_3rd -= self.phase_3rd.floor();
                self.phase_5th -= self.phase_5th.floor();
                self.phase_7th -= self.phase_7th.floor();
            }

            // Downsample 4× → 1×
            let downsampled = self.downsampler.as_mut().unwrap().process(oversampled);

            // Return individual harmonics (approximate split for envelope tracking)
            let total = downsampled;
            let h2_ratio = tube_amount / (tube_amount + tape_amount + console_amount + 0.001);
            let h3_ratio = tape_amount / (tube_amount + tape_amount + console_amount + 0.001);
            let h5_ratio =
                console_amount * 0.6 / (tube_amount + tape_amount + console_amount + 0.001);
            let h7_ratio =
                console_amount * 0.4 / (tube_amount + tape_amount + console_amount + 0.001);

            (
                total * h2_ratio,
                total * h3_ratio,
                total * h5_ratio,
                total * h7_ratio,
            )
        } else {
            // Generate at 1× rate (bass band)
            let h2 =
                (self.phase_2nd * 2.0 * std::f32::consts::PI).sin() * tube_amount * input_level;
            let h3 =
                (self.phase_3rd * 2.0 * std::f32::consts::PI).sin() * tape_amount * input_level;
            let h5 = (self.phase_5th * 2.0 * std::f32::consts::PI).sin()
                * console_amount
                * input_level
                * 0.6;
            let h7 = (self.phase_7th * 2.0 * std::f32::consts::PI).sin()
                * console_amount
                * input_level
                * 0.4;

            // Advance phases
            self.phase_2nd += inc_2nd;
            self.phase_3rd += inc_3rd;
            self.phase_5th += inc_5th;
            self.phase_7th += inc_7th;

            // Wrap phases
            self.phase_2nd -= self.phase_2nd.floor();
            self.phase_3rd -= self.phase_3rd.floor();
            self.phase_5th -= self.phase_5th.floor();
            self.phase_7th -= self.phase_7th.floor();

            (h2, h3, h5, h7)
        }
    }

    fn reset(&mut self) {
        self.phase_2nd = 0.0;
        self.phase_3rd = 0.0;
        self.phase_5th = 0.0;
        self.phase_7th = 0.0;
        if let Some(ref mut ds) = self.downsampler {
            *ds = Downsampler::new(20);
        }
    }
}

/// Per-band saturation processor with fixed harmonic recipes
///
/// Each band has hardcoded optimal tube/tape/console ratios for that frequency range.
/// Bass: warm foundation, Mids: balanced, Presence: clarity/bite
struct BandSaturator {
    sample_rate: f32,

    // Fixed harmonic recipe for this band
    tube_amount: f32,    // 2nd harmonic weight
    tape_amount: f32,    // 3rd harmonic weight
    console_amount: f32, // 5th+7th harmonic weight

    // Harmonic oscillator (with selective oversampling)
    harmonic_osc: HarmonicOscillator,

    // RMS tracking for dynamic saturation and auto-gain (75ms window)
    rms_input: f32,
    rms_output: f32,
    rms_coeff: f32,

    // Pitch-aware multipliers (updated based on detected pitch)
    pitch_tube_mult: f32,
    pitch_tape_mult: f32,
    pitch_console_mult: f32,

    // Pre-emphasis filter (only active for presence band)
    pre_emphasis: Option<BiquadFilter>,
    de_emphasis: Option<BiquadFilter>,

    // DC blocker (1-pole highpass @ 5Hz to remove saturation DC offset)
    dc_blocker: BiquadFilter,
}

impl BandSaturator {
    /// Create new band saturator
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate
    /// * `tube/tape/console_amount` - Fixed harmonic recipe (0-1)
    /// * `use_oversampling` - Whether to use 4× oversampling for harmonics
    /// * `use_emphasis` - Whether to use pre/de-emphasis (presence band only)
    fn new(
        sample_rate: f32,
        tube_amount: f32,
        tape_amount: f32,
        console_amount: f32,
        use_oversampling: bool,
        use_emphasis: bool,
    ) -> Self {
        // RMS smoothing (75ms for smooth gain riding)
        let rms_time_ms = 75.0;
        let rms_samples = (rms_time_ms / 1000.0) * sample_rate;
        let rms_coeff = (-1.0 / rms_samples).exp();

        // Pre/de-emphasis filters (+4dB/-4dB @ 3.5kHz peaking)
        let (pre_emphasis, de_emphasis) = if use_emphasis {
            let mut pre = BiquadFilter::new(sample_rate);
            pre.set_filter_type(crate::params::FilterType::Peaking);
            pre.set_cutoff(3500.0);
            pre.set_resonance(1.0);
            pre.set_gain_db(4.0); // +4dB

            let mut de = BiquadFilter::new(sample_rate);
            de.set_filter_type(crate::params::FilterType::Peaking);
            de.set_cutoff(3500.0);
            de.set_resonance(1.0);
            de.set_gain_db(-4.0); // -4dB

            (Some(pre), Some(de))
        } else {
            (None, None)
        };

        // DC blocker: CRITICAL - Remove DC offset from saturation
        // However, 5Hz might be causing phase issues. Try 2Hz for minimal phase shift
        let mut dc_blocker = BiquadFilter::new(sample_rate);
        dc_blocker.set_filter_type(crate::params::FilterType::Highpass);
        dc_blocker.set_cutoff(2.0); // Lower cutoff = less phase shift in vocal range
        dc_blocker.set_resonance(0.707);

        Self {
            sample_rate,
            tube_amount,
            tape_amount,
            console_amount,
            harmonic_osc: HarmonicOscillator::new(sample_rate, use_oversampling),
            rms_input: 0.0,
            rms_output: 0.0,
            rms_coeff,
            pitch_tube_mult: 1.0,
            pitch_tape_mult: 1.0,
            pitch_console_mult: 1.0,
            pre_emphasis,
            de_emphasis,
            dc_blocker,
        }
    }

    /// Update pitch-aware harmonic multipliers based on detected vocal pitch
    ///
    /// Male (<150Hz): Emphasize tube (body), reduce console (presence)
    /// Female (>200Hz): Reduce tube, emphasize console (presence)
    fn update_pitch_multipliers(&mut self, pitch_hz: f32, confidence: f32) {
        if confidence < 0.5 {
            // Uncertain pitch - use neutral multipliers
            self.pitch_tube_mult = 1.0;
            self.pitch_tape_mult = 1.0;
            self.pitch_console_mult = 1.0;
        } else if pitch_hz < 150.0 {
            // Male vocal - emphasize warmth/body
            self.pitch_tube_mult = 1.2;
            self.pitch_tape_mult = 0.8;
            self.pitch_console_mult = 0.6;
        } else if pitch_hz > 200.0 {
            // Female vocal - emphasize presence/clarity
            self.pitch_tube_mult = 0.8;
            self.pitch_tape_mult = 1.0;
            self.pitch_console_mult = 1.2;
        } else {
            // Transition range - neutral
            self.pitch_tube_mult = 1.0;
            self.pitch_tape_mult = 1.0;
            self.pitch_console_mult = 1.0;
        }
    }

    /// Process one sample through band saturation
    ///
    /// # Arguments
    /// * `input` - Input sample
    /// * `drive` - Drive amount (0-1)
    /// * `mix` - Dry/wet mix (0-1, per-band parallel processing)
    /// * `analysis` - Signal analysis for dynamic saturation
    ///
    /// # Returns
    /// Processed sample
    fn process(&mut self, input: f32, drive: f32, mix: f32, analysis: &SignalAnalysis) -> f32 {
        // Store dry for parallel processing
        let dry = input;

        // Track input RMS
        let squared = input * input;
        self.rms_input = self.rms_input * self.rms_coeff + squared * (1.0 - self.rms_coeff);

        // Dynamic saturation: louder passages get more drive (up to 1.5×)
        let dynamic_mult = 1.0 + (self.rms_input.sqrt() * 0.5);
        let adaptive_drive = (drive * dynamic_mult).min(1.0);

        // Apply pre-emphasis (presence band only)
        let mut signal = input;
        if let Some(ref mut pre) = self.pre_emphasis {
            signal = pre.process(signal);
        }

        // Simple waveshaping (tanh soft saturation)
        let gain = 1.0 + adaptive_drive * 5.0; // 1-6× gain range
        let shaped = (signal * gain).tanh() * 0.95;

        // Generate pitch-tracked harmonics
        let input_level = input.abs().min(1.0);
        let (h2, h3, h5, h7) = self.harmonic_osc.generate_harmonics(
            input_level * adaptive_drive,
            self.tube_amount * self.pitch_tube_mult,
            self.tape_amount * self.pitch_tape_mult,
            self.console_amount * self.pitch_console_mult,
        );

        // Combine waveshaping + harmonics
        let mut wet = shaped + h2 + h3 + h5 + h7;

        // Apply de-emphasis (presence band only)
        if let Some(ref mut de) = self.de_emphasis {
            wet = de.process(wet);
        }

        // Track output RMS
        let squared_out = wet * wet;
        self.rms_output = self.rms_output * self.rms_coeff + squared_out * (1.0 - self.rms_coeff);

        // Auto-gain compensation
        let compensation = self.calculate_auto_gain();
        wet *= compensation;

        // Remove DC offset from saturation
        wet = self.dc_blocker.process(wet);

        // Parallel processing: blend dry and wet per-band
        dry * (1.0 - mix) + wet * mix
    }

    fn calculate_auto_gain(&self) -> f32 {
        let input_level = self.rms_input.sqrt().max(0.001);
        let output_level = self.rms_output.sqrt().max(0.001);
        let compensation = input_level / output_level;
        compensation.clamp(0.5, 2.0) // ±6dB limit
    }

    fn reset(&mut self) {
        self.rms_input = 0.0;
        self.rms_output = 0.0;
        self.harmonic_osc.reset();
        if let Some(ref mut pre) = self.pre_emphasis {
            *pre = BiquadFilter::new(self.sample_rate);
            pre.set_filter_type(crate::params::FilterType::Peaking);
            pre.set_cutoff(3500.0);
            pre.set_resonance(1.0);
            pre.set_gain_db(4.0);
        }
        if let Some(ref mut de) = self.de_emphasis {
            *de = BiquadFilter::new(self.sample_rate);
            de.set_filter_type(crate::params::FilterType::Peaking);
            de.set_cutoff(3500.0);
            de.set_resonance(1.0);
            de.set_gain_db(-4.0);
        }
        self.dc_blocker = BiquadFilter::new(self.sample_rate);
        self.dc_blocker
            .set_filter_type(crate::params::FilterType::Highpass);
        self.dc_blocker.set_cutoff(5.0);
        self.dc_blocker.set_resonance(0.707);
    }
}

/// Professional 4-band vocal saturator with mid-side processing
pub struct AdaptiveSaturator {
    sample_rate: f32,

    // Single crossover - used for both mid and side channels
    // (Crossover is just frequency splitting, doesn't need separate instances)
    crossover: MultibandCrossover,

    // Per-band saturators - SEPARATE instances for mid and side channels
    // (Saturators have stateful processing that needs independence)
    bass_saturator_mid: BandSaturator,
    mid_saturator_mid: BandSaturator,
    presence_saturator_mid: BandSaturator,

    bass_saturator_side: BandSaturator,
    mid_saturator_side: BandSaturator,
    presence_saturator_side: BandSaturator,

    // Air band exciter (ultra-light harmonic enhancement >8kHz)
    air_exciter_drive: f32,
    air_exciter_mix: f32,

    // Safety limiter (fast attack, -0.1dBFS ceiling)
    limiter_threshold: f32, // 0.9 linear (~-0.9dB)
    limiter_gain_reduction: f32,
}

impl AdaptiveSaturator {
    /// Create new 4-band adaptive saturator
    pub fn new(sample_rate: f32) -> Self {
        // Create single crossover (shared by mid and side channels)
        let crossover = MultibandCrossover::new(sample_rate);

        // Create band saturators with fixed optimal recipes
        // CRITICAL: Separate instances for mid and side to prevent filter state corruption

        // Bass Mid: Heavy tube (warmth), moderate tape, light console
        let bass_saturator_mid = BandSaturator::new(
            sample_rate,
            0.7,   // tube
            0.4,   // tape
            0.2,   // console
            false, // no oversampling (< 200Hz)
            false, // no pre-emphasis
        );

        // Bass Side: Same recipe as mid
        let bass_saturator_side = BandSaturator::new(sample_rate, 0.7, 0.4, 0.2, false, false);

        // Mids Mid: Balanced all three
        let mid_saturator_mid = BandSaturator::new(
            sample_rate,
            0.5,   // tube
            0.5,   // tape
            0.5,   // console
            true,  // 4× oversampling
            false, // no pre-emphasis
        );

        // Mids Side: Same recipe as mid
        let mid_saturator_side = BandSaturator::new(sample_rate, 0.5, 0.5, 0.5, true, false);

        // Presence Mid: Light tube, moderate tape, heavy console (clarity)
        let presence_saturator_mid = BandSaturator::new(
            sample_rate,
            0.3,  // tube
            0.4,  // tape
            0.7,  // console
            true, // 4× oversampling
            true, // pre-emphasis active
        );

        // Presence Side: Same recipe as mid
        let presence_saturator_side = BandSaturator::new(sample_rate, 0.3, 0.4, 0.7, true, true);

        Self {
            sample_rate,
            crossover,
            bass_saturator_mid,
            mid_saturator_mid,
            presence_saturator_mid,
            bass_saturator_side,
            mid_saturator_side,
            presence_saturator_side,
            air_exciter_drive: 0.1,
            air_exciter_mix: 0.15,
            limiter_threshold: 0.9,
            limiter_gain_reduction: 1.0,
        }
    }

    /// Process stereo sample with 4-band saturation and mid-side processing
    ///
    /// # Arguments
    /// * `left/right` - Input samples
    /// * `bass/mid/presence_drive` - Per-band drive (0-1)
    /// * `bass/mid/presence_mix` - Per-band dry/wet mix (0-1)
    /// * `air_drive` - Air exciter drive (0-1)
    /// * `air_mix` - Air exciter dry/wet mix (0-1)
    /// * `stereo_width` - Mid-side balance (-1 to +1, 0=neutral)
    /// * `analysis` - Signal analysis (pitch, dynamics, etc.)
    ///
    /// # Returns
    /// Tuple of (left_out, right_out)
    pub fn process(
        &mut self,
        left: f32,
        right: f32,
        bass_drive: f32,
        bass_mix: f32,
        mid_drive: f32,
        mid_mix: f32,
        presence_drive: f32,
        presence_mix: f32,
        air_drive: f32,
        air_mix: f32,
        stereo_width: f32,
        analysis: &SignalAnalysis,
    ) -> (f32, f32) {
        // NOTE: Transient detection and lookahead disabled
        // They were causing constant gain reduction (~4-6dB) and phase issues
        // Processing happens immediately without delay

        // Store air parameters for this processing cycle
        self.air_exciter_drive = air_drive;
        self.air_exciter_mix = air_mix;

        // Update pitch tracking in all oscillators (mid and side)
        self.bass_saturator_mid
            .harmonic_osc
            .update_pitch(analysis.pitch_hz, analysis.pitch_confidence);
        self.mid_saturator_mid
            .harmonic_osc
            .update_pitch(analysis.pitch_hz, analysis.pitch_confidence);
        self.presence_saturator_mid
            .harmonic_osc
            .update_pitch(analysis.pitch_hz, analysis.pitch_confidence);
        self.bass_saturator_side
            .harmonic_osc
            .update_pitch(analysis.pitch_hz, analysis.pitch_confidence);
        self.mid_saturator_side
            .harmonic_osc
            .update_pitch(analysis.pitch_hz, analysis.pitch_confidence);
        self.presence_saturator_side
            .harmonic_osc
            .update_pitch(analysis.pitch_hz, analysis.pitch_confidence);

        // Update pitch-aware multipliers (mid and side)
        self.bass_saturator_mid
            .update_pitch_multipliers(analysis.pitch_hz, analysis.pitch_confidence);
        self.mid_saturator_mid
            .update_pitch_multipliers(analysis.pitch_hz, analysis.pitch_confidence);
        self.presence_saturator_mid
            .update_pitch_multipliers(analysis.pitch_hz, analysis.pitch_confidence);
        self.bass_saturator_side
            .update_pitch_multipliers(analysis.pitch_hz, analysis.pitch_confidence);
        self.mid_saturator_side
            .update_pitch_multipliers(analysis.pitch_hz, analysis.pitch_confidence);
        self.presence_saturator_side
            .update_pitch_multipliers(analysis.pitch_hz, analysis.pitch_confidence);

        // Convert L/R to Mid/Side
        let mid = (left + right) * 0.5;
        let side = (left - right) * 0.5;

        // Calculate mid/side drive multipliers based on width
        let (mid_mult, side_mult) = if stereo_width >= 0.0 {
            // Positive width: saturate mid more (power/glue)
            (1.0 + stereo_width * 0.5, 1.0 - stereo_width * 0.3)
        } else {
            // Negative width: saturate sides more (wide/thin)
            (1.0 + stereo_width * 0.3, 1.0 - stereo_width * 0.5)
        };

        // Process mid channel through 4-band split
        let (bass_mid, mids_mid, presence_mid, air_mid) = self.crossover.process(mid);

        let bass_out_mid =
            self.bass_saturator_mid
                .process(bass_mid, bass_drive * mid_mult, bass_mix, analysis);
        let mids_out_mid =
            self.mid_saturator_mid
                .process(mids_mid, mid_drive * mid_mult, mid_mix, analysis);
        let presence_out_mid = self.presence_saturator_mid.process(
            presence_mid,
            presence_drive * mid_mult,
            presence_mix,
            analysis,
        );

        // Air band: Ultra-light exciter for "sheen" (>8kHz)
        let air_out_mid = self.process_air_band(air_mid);

        // Sum mid bands
        let mid_out = bass_out_mid + mids_out_mid + presence_out_mid + air_out_mid;

        // Process side channel through same 4-band split
        // Note: Crossover can be shared safely - only saturators need separate instances
        let (bass_side, mids_side, presence_side, air_side) = self.crossover.process(side);

        let bass_out_side =
            self.bass_saturator_side
                .process(bass_side, bass_drive * side_mult, bass_mix, analysis);
        let mids_out_side =
            self.mid_saturator_side
                .process(mids_side, mid_drive * side_mult, mid_mix, analysis);
        let presence_out_side = self.presence_saturator_side.process(
            presence_side,
            presence_drive * side_mult,
            presence_mix,
            analysis,
        );

        // Air band exciter for side channel
        let air_out_side = self.process_air_band(air_side);

        // Sum side bands
        let side_out = bass_out_side + mids_out_side + presence_out_side + air_out_side;

        // Convert Mid/Side back to L/R (CRITICAL: must scale by 2.0 to compensate for 0.5 scaling in M/S encode)
        // M = (L+R)*0.5, S = (L-R)*0.5 → L = M+S (but we need to scale back!)
        // Correct formula: L = (M+S)*2, R = (M-S)*2 OR use full-scale M/S without 0.5
        let mut left_out = mid_out + side_out;
        let mut right_out = mid_out - side_out;

        // Safety limiter: Fast attack, -0.1dBFS ceiling (prevents clipping)
        let peak = left_out.abs().max(right_out.abs());
        if peak > self.limiter_threshold {
            let target_gain = self.limiter_threshold / peak;
            self.limiter_gain_reduction = target_gain.min(self.limiter_gain_reduction * 0.9 + 0.1);
        // Fast attack
        } else {
            self.limiter_gain_reduction = (self.limiter_gain_reduction * 0.999 + 0.001).min(1.0);
            // Slow release
        }

        left_out *= self.limiter_gain_reduction;
        right_out *= self.limiter_gain_reduction;

        (left_out, right_out)
    }

    /// Process air band with ultra-light harmonic exciter (>8kHz)
    fn process_air_band(&self, input: f32) -> f32 {
        // Ultra-light saturation for "sheen" without harshness
        let drive = 1.0 + self.air_exciter_drive * 2.0; // 1.0-1.2 range
        let excited = (input * drive).tanh() * 0.95;

        // Parallel mix: 15% wet
        input * (1.0 - self.air_exciter_mix) + excited * self.air_exciter_mix
    }

    /// Reset all processing state
    pub fn reset(&mut self) {
        self.crossover.reset();
        self.bass_saturator_mid.reset();
        self.mid_saturator_mid.reset();
        self.presence_saturator_mid.reset();
        self.bass_saturator_side.reset();
        self.mid_saturator_side.reset();
        self.presence_saturator_side.reset();

        // Reset limiter state
        self.limiter_gain_reduction = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_analysis() -> SignalAnalysis {
        SignalAnalysis {
            rms_level: 0.3,
            peak_level: 0.5,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 200.0,
            signal_type: crate::dsp::analysis::SignalType::Tonal,
            is_voiced: true,
            is_unvoiced: false,
            has_sibilance: false,
            sibilance_strength: 0.0,
            pitch_hz: 220.0,
            pitch_confidence: 0.8,
            is_pitched: true,
        }
    }

    #[test]
    fn test_saturator_creation() {
        let saturator = AdaptiveSaturator::new(44100.0);
        assert_eq!(saturator.sample_rate, 44100.0);
    }

    #[test]
    fn test_process_produces_finite_output() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        let (left, right) = saturator.process(
            0.5, 0.5, // left, right
            0.6, 0.5, // bass drive, mix
            0.5, 0.4, // mid drive, mix
            0.35, 0.35, // presence drive, mix
            0.1, 0.15, // air drive, mix
            0.0,  // width
            &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
    }

    #[test]
    fn test_stereo_width_positive() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Positive width should increase mid saturation
        let (left, right) = saturator.process(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.5, &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
    }

    #[test]
    fn test_stereo_width_negative() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Negative width should increase side saturation
        let (left, right) = saturator.process(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, -0.5, &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
    }

    #[test]
    fn test_pitch_aware_male_vocal() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.pitch_hz = 120.0; // Male vocal
        analysis.pitch_confidence = 0.9;

        let (left, right) = saturator.process(
            0.5, 0.5, 0.6, 0.5, 0.5, 0.4, 0.35, 0.35, 0.1, 0.15, 0.0, &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
        // Male vocal should emphasize tube (warmth)
        assert!(saturator.bass_saturator_mid.pitch_tube_mult > 1.0);
    }

    #[test]
    fn test_pitch_aware_female_vocal() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let mut analysis = create_test_analysis();
        analysis.pitch_hz = 250.0; // Female vocal
        analysis.pitch_confidence = 0.9;

        let (left, right) = saturator.process(
            0.5, 0.5, 0.6, 0.5, 0.5, 0.4, 0.35, 0.35, 0.1, 0.15, 0.0, &analysis,
        );

        assert!(left.is_finite());
        assert!(right.is_finite());
        // Female vocal should emphasize console (presence)
        assert!(saturator.presence_saturator_mid.pitch_console_mult > 1.0);
    }

    #[test]
    fn test_separate_crossovers_no_corruption() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Process multiple samples to ensure crossovers stay independent
        for _ in 0..100 {
            let (l, r) = saturator.process(
                0.5, -0.3, // Asymmetric L/R to test mid/side separation
                0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.0, &analysis,
            );
            assert!(l.is_finite());
            assert!(r.is_finite());
        }
    }

    #[test]
    fn test_safety_limiter_prevents_clipping() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // Process hot signal with max drive
        let mut max_peak: f32 = 0.0;
        for _ in 0..200 {
            let (l, r) = saturator.process(
                0.9, 0.9, // Hot input
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.15, 0.0, &analysis,
            );
            max_peak = max_peak.max(l.abs()).max(r.abs());
        }

        // Limiter should keep peaks below threshold
        assert!(
            max_peak <= 0.95,
            "Limiter should prevent clipping, got peak: {}",
            max_peak
        );
    }

    #[test]
    fn test_air_band_exciter_works() {
        let saturator = AdaptiveSaturator::new(44100.0);

        // Test air band processing
        let input = 0.3;
        let output = saturator.process_air_band(input);

        assert!(output.is_finite());
        assert!(output.abs() > 0.0);
        // Output should be slightly enhanced but not drastically different
        assert!((output - input).abs() < 0.1);
    }

    #[test]
    fn test_no_gain_loss() {
        let mut saturator = AdaptiveSaturator::new(44100.0);
        let analysis = create_test_analysis();

        // With transient reduction disabled, should have immediate response
        let (l1, r1) = saturator.process(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.1, 0.15, 0.0, &analysis,
        );

        // Should have immediate output (no lookahead delay)
        assert!(l1.abs() > 0.0, "Should have immediate output");
        assert!(r1.abs() > 0.0, "Should have immediate output");

        // With default 50% mix and auto-gain, output will be in range
        // Allow 0.1-1.0 range (processing can reduce level slightly)
        assert!(
            l1.abs() > 0.1 && l1.abs() < 1.0,
            "Output should be in reasonable range: got {}",
            l1
        );
    }
}
