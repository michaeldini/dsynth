/// Parameter Registry for CLAP Plugin
///
/// Centralized registry that maps parameter IDs to descriptors and manages
/// parameter value conversion between normalized (0.0-1.0) and internal ranges.
///
/// This registry is the single source of truth for:
/// - Parameter metadata (name, range, unit, automation flags)
/// - Normalized ↔ internal value conversion
/// - Parameter iteration and lookup
/// - Consistency checks across the plugin
use super::param_descriptor::*;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Global parameter registry (lazy-initialized, thread-safe)
static REGISTRY: OnceLock<ParamRegistry> = OnceLock::new();

/// Get the global parameter registry
pub fn get_registry() -> &'static ParamRegistry {
    REGISTRY.get_or_init(ParamRegistry::new)
}

/// Complete parameter registry for DSynth
pub struct ParamRegistry {
    /// Map of parameter ID → descriptor
    descriptors: HashMap<ParamId, ParamDescriptor>,
    /// Sorted list of all parameter IDs (for iteration)
    param_ids: Vec<ParamId>,
}

impl ParamRegistry {
    /// Create a new parameter registry with all DSynth parameters
    pub fn new() -> Self {
        let mut descriptors = HashMap::new();
        let mut param_ids = Vec::new();

        // Helper macro to add a parameter
        macro_rules! add_param {
            ($id:expr, $desc:expr) => {
                descriptors.insert($id, $desc);
                param_ids.push($id);
            };
        }

        // Master parameters
        add_param!(
            PARAM_MASTER_GAIN,
            ParamDescriptor::float(
                PARAM_MASTER_GAIN,
                "Gain",
                "Master",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );
        add_param!(
            PARAM_MONOPHONIC,
            ParamDescriptor::bool(PARAM_MONOPHONIC, "Monophonic", "Master", false)
        );
        add_param!(
            PARAM_HARD_SYNC,
            ParamDescriptor::bool(PARAM_HARD_SYNC, "Hard Sync (1→2→3)", "Master", false)
        );

        // Oscillator 1
        add_param!(
            PARAM_OSC1_WAVEFORM,
            ParamDescriptor::enum_param(
                PARAM_OSC1_WAVEFORM,
                "Waveform",
                "Oscillator 1",
                vec![
                    "Sine".into(),
                    "Saw".into(),
                    "Square".into(),
                    "Triangle".into(),
                    "Pulse".into(),
                    "White Noise".into(),
                    "Pink Noise".into(),
                    "Additive".into(),
                    "Wavetable".into(),
                ],
                0 // Default: Sine
            )
        );
        add_param!(
            PARAM_OSC1_PITCH,
            ParamDescriptor::float(
                PARAM_OSC1_PITCH,
                "Pitch",
                "Oscillator 1",
                -24.0,
                24.0,
                0.0,
                Some("semitones")
            )
        );
        add_param!(
            PARAM_OSC1_DETUNE,
            ParamDescriptor::float(
                PARAM_OSC1_DETUNE,
                "Detune",
                "Oscillator 1",
                -50.0,
                50.0,
                0.0,
                Some("cents")
            )
        );
        add_param!(
            PARAM_OSC1_GAIN,
            ParamDescriptor::float(
                PARAM_OSC1_GAIN,
                "Gain",
                "Oscillator 1",
                0.0,
                1.0,
                0.25,
                Some("lin")
            )
        );
        add_param!(
            PARAM_OSC1_PAN,
            ParamDescriptor::float(
                PARAM_OSC1_PAN,
                "Pan",
                "Oscillator 1",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC1_UNISON,
            ParamDescriptor::int(PARAM_OSC1_UNISON, "Unison", "Oscillator 1", 1, 7, 1)
        );
        add_param!(
            PARAM_OSC1_UNISON_DETUNE,
            ParamDescriptor::float(
                PARAM_OSC1_UNISON_DETUNE,
                "Unison Detune",
                "Oscillator 1",
                0.0,
                100.0,
                10.0,
                Some("cents")
            )
        );
        add_param!(
            PARAM_OSC1_PHASE,
            ParamDescriptor::float(
                PARAM_OSC1_PHASE,
                "Phase",
                "Oscillator 1",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC1_SHAPE,
            ParamDescriptor::float(
                PARAM_OSC1_SHAPE,
                "Shape",
                "Oscillator 1",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC1_FM_SOURCE,
            ParamDescriptor::int(PARAM_OSC1_FM_SOURCE, "FM Source", "Oscillator 1", 0, 3, 0)
        );
        add_param!(
            PARAM_OSC1_FM_AMOUNT,
            ParamDescriptor::float(
                PARAM_OSC1_FM_AMOUNT,
                "FM Amount",
                "Oscillator 1",
                0.0,
                10.0,
                0.0,
                Some("lin")
            )
        );

        // Oscillator 1 additive harmonics
        for i in 0..8 {
            let param_id = PARAM_OSC1_H1 + i as u32;
            add_param!(
                param_id,
                ParamDescriptor::float(
                    param_id,
                    format!("Harmonic {}", i + 1),
                    "Oscillator 1 Additive",
                    0.0,
                    1.0,
                    if i == 0 { 1.0 } else { 0.0 }, // First harmonic on by default
                    Some("")
                )
            );
        }

        add_param!(
            PARAM_OSC1_SOLO,
            ParamDescriptor::bool(PARAM_OSC1_SOLO, "Solo", "Oscillator 1", false)
        );

        add_param!(
            PARAM_OSC1_SATURATION,
            ParamDescriptor::float(
                PARAM_OSC1_SATURATION,
                "Saturation",
                "Oscillator 1",
                0.0,
                1.0,
                0.0,
                Some("%")
            )
        );

        // Oscillator 2 (same structure as Osc1)
        add_param!(
            PARAM_OSC2_WAVEFORM,
            ParamDescriptor::enum_param(
                PARAM_OSC2_WAVEFORM,
                "Waveform",
                "Oscillator 2",
                vec![
                    "Sine".into(),
                    "Saw".into(),
                    "Square".into(),
                    "Triangle".into(),
                    "Pulse".into(),
                    "White Noise".into(),
                    "Pink Noise".into(),
                    "Additive".into(),
                    "Wavetable".into(),
                ],
                1 // Default: Saw
            )
        );
        add_param!(
            PARAM_OSC2_PITCH,
            ParamDescriptor::float(
                PARAM_OSC2_PITCH,
                "Pitch",
                "Oscillator 2",
                -24.0,
                24.0,
                0.0,
                Some("semitones")
            )
        );
        add_param!(
            PARAM_OSC2_DETUNE,
            ParamDescriptor::float(
                PARAM_OSC2_DETUNE,
                "Detune",
                "Oscillator 2",
                -50.0,
                50.0,
                0.0,
                Some("cents")
            )
        );
        add_param!(
            PARAM_OSC2_GAIN,
            ParamDescriptor::float(
                PARAM_OSC2_GAIN,
                "Gain",
                "Oscillator 2",
                0.0,
                1.0,
                0.0,
                Some("lin")
            )
        );
        add_param!(
            PARAM_OSC2_PAN,
            ParamDescriptor::float(
                PARAM_OSC2_PAN,
                "Pan",
                "Oscillator 2",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC2_UNISON,
            ParamDescriptor::int(PARAM_OSC2_UNISON, "Unison", "Oscillator 2", 1, 7, 1)
        );
        add_param!(
            PARAM_OSC2_UNISON_DETUNE,
            ParamDescriptor::float(
                PARAM_OSC2_UNISON_DETUNE,
                "Unison Detune",
                "Oscillator 2",
                0.0,
                100.0,
                10.0,
                Some("cents")
            )
        );
        add_param!(
            PARAM_OSC2_PHASE,
            ParamDescriptor::float(
                PARAM_OSC2_PHASE,
                "Phase",
                "Oscillator 2",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC2_SHAPE,
            ParamDescriptor::float(
                PARAM_OSC2_SHAPE,
                "Shape",
                "Oscillator 2",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC2_FM_SOURCE,
            ParamDescriptor::int(PARAM_OSC2_FM_SOURCE, "FM Source", "Oscillator 2", 0, 3, 0)
        );
        add_param!(
            PARAM_OSC2_FM_AMOUNT,
            ParamDescriptor::float(
                PARAM_OSC2_FM_AMOUNT,
                "FM Amount",
                "Oscillator 2",
                0.0,
                10.0,
                0.0,
                Some("lin")
            )
        );

        // Oscillator 2 additive harmonics
        for i in 0..8 {
            let param_id = PARAM_OSC2_H1 + i as u32;
            add_param!(
                param_id,
                ParamDescriptor::float(
                    param_id,
                    format!("Harmonic {}", i + 1),
                    "Oscillator 2 Additive",
                    0.0,
                    1.0,
                    if i == 0 { 1.0 } else { 0.0 },
                    Some("")
                )
            );
        }

        add_param!(
            PARAM_OSC2_SOLO,
            ParamDescriptor::bool(PARAM_OSC2_SOLO, "Solo", "Oscillator 2", false)
        );

        add_param!(
            PARAM_OSC2_SATURATION,
            ParamDescriptor::float(
                PARAM_OSC2_SATURATION,
                "Saturation",
                "Oscillator 2",
                0.0,
                1.0,
                0.0,
                Some("%")
            )
        );

        // Oscillator 3 (same structure)
        add_param!(
            PARAM_OSC3_WAVEFORM,
            ParamDescriptor::enum_param(
                PARAM_OSC3_WAVEFORM,
                "Waveform",
                "Oscillator 3",
                vec![
                    "Sine".into(),
                    "Saw".into(),
                    "Square".into(),
                    "Triangle".into(),
                    "Pulse".into(),
                    "White Noise".into(),
                    "Pink Noise".into(),
                    "Additive".into(),
                    "Wavetable".into(),
                ],
                2 // Default: Square
            )
        );
        add_param!(
            PARAM_OSC3_PITCH,
            ParamDescriptor::float(
                PARAM_OSC3_PITCH,
                "Pitch",
                "Oscillator 3",
                -24.0,
                24.0,
                0.0,
                Some("semitones")
            )
        );
        add_param!(
            PARAM_OSC3_DETUNE,
            ParamDescriptor::float(
                PARAM_OSC3_DETUNE,
                "Detune",
                "Oscillator 3",
                -50.0,
                50.0,
                0.0,
                Some("cents")
            )
        );
        add_param!(
            PARAM_OSC3_GAIN,
            ParamDescriptor::float(
                PARAM_OSC3_GAIN,
                "Gain",
                "Oscillator 3",
                0.0,
                1.0,
                0.0,
                Some("lin")
            )
        );
        add_param!(
            PARAM_OSC3_PAN,
            ParamDescriptor::float(
                PARAM_OSC3_PAN,
                "Pan",
                "Oscillator 3",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC3_UNISON,
            ParamDescriptor::int(PARAM_OSC3_UNISON, "Unison", "Oscillator 3", 1, 7, 1)
        );
        add_param!(
            PARAM_OSC3_UNISON_DETUNE,
            ParamDescriptor::float(
                PARAM_OSC3_UNISON_DETUNE,
                "Unison Detune",
                "Oscillator 3",
                0.0,
                100.0,
                10.0,
                Some("cents")
            )
        );
        add_param!(
            PARAM_OSC3_PHASE,
            ParamDescriptor::float(
                PARAM_OSC3_PHASE,
                "Phase",
                "Oscillator 3",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC3_SHAPE,
            ParamDescriptor::float(
                PARAM_OSC3_SHAPE,
                "Shape",
                "Oscillator 3",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_OSC3_FM_SOURCE,
            ParamDescriptor::int(PARAM_OSC3_FM_SOURCE, "FM Source", "Oscillator 3", 0, 3, 0)
        );
        add_param!(
            PARAM_OSC3_FM_AMOUNT,
            ParamDescriptor::float(
                PARAM_OSC3_FM_AMOUNT,
                "FM Amount",
                "Oscillator 3",
                0.0,
                10.0,
                0.0,
                Some("lin")
            )
        );

        // Oscillator 3 additive harmonics
        for i in 0..8 {
            let param_id = PARAM_OSC3_H1 + i as u32;
            add_param!(
                param_id,
                ParamDescriptor::float(
                    param_id,
                    format!("Harmonic {}", i + 1),
                    "Oscillator 3 Additive",
                    0.0,
                    1.0,
                    if i == 0 { 1.0 } else { 0.0 },
                    Some("")
                )
            );
        }

        add_param!(
            PARAM_OSC3_SOLO,
            ParamDescriptor::bool(PARAM_OSC3_SOLO, "Solo", "Oscillator 3", false)
        );

        add_param!(
            PARAM_OSC3_SATURATION,
            ParamDescriptor::float(
                PARAM_OSC3_SATURATION,
                "Saturation",
                "Oscillator 3",
                0.0,
                1.0,
                0.0,
                Some("%")
            )
        );

        // Filters (Lowpass, Highpass, Bandpass)
        for filter_idx in 0..3 {
            let base_id = match filter_idx {
                0 => PARAM_FILTER1_TYPE,
                1 => PARAM_FILTER2_TYPE,
                2 => PARAM_FILTER3_TYPE,
                _ => unreachable!(),
            };
            let module = format!("Filter {}", filter_idx + 1);

            add_param!(
                base_id,
                ParamDescriptor::enum_param(
                    base_id,
                    "Type",
                    &module,
                    vec!["Lowpass".into(), "Highpass".into(), "Bandpass".into()],
                    0 // Default: Lowpass
                )
            );

            add_param!(
                base_id + 1,
                ParamDescriptor::float_log(
                    base_id + 1,
                    "Cutoff",
                    &module,
                    20.0,
                    20000.0,
                    1000.0,
                    Some("Hz")
                )
            );

            add_param!(
                base_id + 2,
                ParamDescriptor::float(
                    base_id + 2,
                    "Resonance",
                    &module,
                    0.5,
                    50.0,
                    0.707,
                    Some("")
                )
            );

            add_param!(
                base_id + 3,
                ParamDescriptor::float(
                    base_id + 3,
                    "Bandwidth",
                    &module,
                    0.1,
                    4.0,
                    1.0,
                    Some("octaves")
                )
            );

            add_param!(
                base_id + 4,
                ParamDescriptor::float(
                    base_id + 4,
                    "Key Tracking",
                    &module,
                    0.0,
                    1.0,
                    0.0,
                    Some("")
                )
            );

            add_param!(
                base_id + 5,
                ParamDescriptor::float_log(
                    base_id + 5,
                    "Env Attack",
                    &module,
                    0.001,
                    5.0,
                    0.01,
                    Some("ms")
                )
            );

            add_param!(
                base_id + 6,
                ParamDescriptor::float_log(
                    base_id + 6,
                    "Env Decay",
                    &module,
                    0.001,
                    5.0,
                    0.1,
                    Some("ms")
                )
            );

            add_param!(
                base_id + 7,
                ParamDescriptor::float(
                    base_id + 7,
                    "Env Sustain",
                    &module,
                    0.0,
                    1.0,
                    0.5,
                    Some("")
                )
            );

            add_param!(
                base_id + 8,
                ParamDescriptor::float_log(
                    base_id + 8,
                    "Env Release",
                    &module,
                    0.001,
                    5.0,
                    0.2,
                    Some("ms")
                )
            );

            add_param!(
                base_id + 9,
                ParamDescriptor::float(
                    base_id + 9,
                    "Env Amount",
                    &module,
                    -10000.0,
                    10000.0,
                    0.0,
                    Some("Hz")
                )
            );

            add_param!(
                base_id + 10,
                ParamDescriptor::float(base_id + 10, "Drive", &module, 0.0, 1.0, 0.0, Some("%"))
            );

            add_param!(
                base_id + 11,
                ParamDescriptor::float(
                    base_id + 11,
                    "Post Drive",
                    &module,
                    0.0,
                    1.0,
                    0.0,
                    Some("%")
                )
            );
        }

        // LFOs
        for lfo_idx in 0..3 {
            let base_id = match lfo_idx {
                0 => PARAM_LFO1_WAVEFORM,
                1 => PARAM_LFO2_WAVEFORM,
                2 => PARAM_LFO3_WAVEFORM,
                _ => unreachable!(),
            };
            let module = format!("LFO {}", lfo_idx + 1);

            add_param!(
                base_id,
                ParamDescriptor::enum_param(
                    base_id,
                    "Waveform",
                    &module,
                    vec![
                        "Sine".into(),
                        "Triangle".into(),
                        "Square".into(),
                        "Saw".into()
                    ],
                    0 // Default: Sine
                )
            );

            add_param!(
                base_id + 1,
                ParamDescriptor::float_log(
                    base_id + 1,
                    "Rate",
                    &module,
                    0.01,
                    20.0,
                    2.0,
                    Some("Hz")
                )
            );

            add_param!(
                base_id + 2,
                ParamDescriptor::enum_param(
                    base_id + 2,
                    "Tempo Sync",
                    &module,
                    vec![
                        "Hz".into(),
                        "1/1".into(),
                        "1/2".into(),
                        "1/4".into(),
                        "1/8".into(),
                        "1/16".into(),
                        "1/32".into(),
                        "1/4T".into(),
                        "1/8T".into(),
                        "1/16T".into(),
                        "1/4D".into(),
                        "1/8D".into(),
                        "1/16D".into(),
                    ],
                    0 // Default: Hz (free-running)
                )
            );

            add_param!(
                base_id + 3,
                ParamDescriptor::float(base_id + 3, "Depth", &module, 0.0, 1.0, 0.0, Some(""))
            );

            add_param!(
                base_id + 4,
                ParamDescriptor::float(
                    base_id + 4,
                    "Filter Amount",
                    &module,
                    -5000.0,
                    5000.0,
                    0.0,
                    Some("Hz")
                )
            );

            add_param!(
                base_id + 5,
                ParamDescriptor::float(
                    base_id + 5,
                    "Pitch Amount",
                    &module,
                    -100.0,
                    100.0,
                    0.0,
                    Some("cents")
                )
            );

            add_param!(
                base_id + 6,
                ParamDescriptor::float(
                    base_id + 6,
                    "Gain Amount",
                    &module,
                    -1.0,
                    1.0,
                    0.0,
                    Some("")
                )
            );

            add_param!(
                base_id + 7,
                ParamDescriptor::float(base_id + 7, "Pan Amount", &module, 0.0, 1.0, 0.0, Some(""))
            );

            add_param!(
                base_id + 8,
                ParamDescriptor::float(base_id + 8, "PWM Amount", &module, 0.0, 1.0, 0.0, Some(""))
            );

            add_param!(
                base_id + 9,
                ParamDescriptor::enum_param(
                    base_id + 9,
                    "Destination",
                    &module,
                    vec!["Global".into(), "Osc1".into(), "Osc2".into(), "Osc3".into(),],
                    0 // Default: Global
                )
            );
        }

        // Envelope (ADSR)
        add_param!(
            PARAM_ENVELOPE_ATTACK,
            ParamDescriptor::float_log(
                PARAM_ENVELOPE_ATTACK,
                "Attack",
                "Envelope",
                0.001,
                5.0,
                0.01,
                Some("s")
            )
        );
        add_param!(
            PARAM_ENVELOPE_DECAY,
            ParamDescriptor::float_log(
                PARAM_ENVELOPE_DECAY,
                "Decay",
                "Envelope",
                0.001,
                5.0,
                0.1,
                Some("s")
            )
        );
        add_param!(
            PARAM_ENVELOPE_SUSTAIN,
            ParamDescriptor::float(
                PARAM_ENVELOPE_SUSTAIN,
                "Sustain",
                "Envelope",
                0.0,
                1.0,
                0.7,
                Some("")
            )
        );
        add_param!(
            PARAM_ENVELOPE_RELEASE,
            ParamDescriptor::float_log(
                PARAM_ENVELOPE_RELEASE,
                "Release",
                "Envelope",
                0.001,
                5.0,
                0.2,
                Some("s")
            )
        );
        add_param!(
            PARAM_ENVELOPE_ATTACK_CURVE,
            ParamDescriptor::float(
                PARAM_ENVELOPE_ATTACK_CURVE,
                "Attack Curve",
                "Envelope",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_ENVELOPE_DECAY_CURVE,
            ParamDescriptor::float(
                PARAM_ENVELOPE_DECAY_CURVE,
                "Decay Curve",
                "Envelope",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_ENVELOPE_RELEASE_CURVE,
            ParamDescriptor::float(
                PARAM_ENVELOPE_RELEASE_CURVE,
                "Release Curve",
                "Envelope",
                -1.0,
                1.0,
                0.0,
                Some("")
            )
        );

        // Velocity
        add_param!(
            PARAM_VELOCITY_AMP,
            ParamDescriptor::float(
                PARAM_VELOCITY_AMP,
                "Amplitude",
                "Velocity",
                0.0,
                1.0,
                0.7,
                Some("")
            )
        );
        add_param!(
            PARAM_VELOCITY_FILTER,
            ParamDescriptor::float(
                PARAM_VELOCITY_FILTER,
                "Filter",
                "Velocity",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );

        // Effects
        add_param!(
            PARAM_REVERB_ROOM_SIZE,
            ParamDescriptor::float(
                PARAM_REVERB_ROOM_SIZE,
                "Room Size",
                "Reverb",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );
        add_param!(
            PARAM_REVERB_DAMPING,
            ParamDescriptor::float(
                PARAM_REVERB_DAMPING,
                "Damping",
                "Reverb",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );
        add_param!(
            PARAM_REVERB_WET,
            ParamDescriptor::float(PARAM_REVERB_WET, "Wet", "Reverb", 0.0, 1.0, 0.33, Some(""))
        );
        add_param!(
            PARAM_REVERB_DRY,
            ParamDescriptor::float(PARAM_REVERB_DRY, "Dry", "Reverb", 0.0, 1.0, 0.67, Some(""))
        );
        add_param!(
            PARAM_REVERB_WIDTH,
            ParamDescriptor::float(
                PARAM_REVERB_WIDTH,
                "Width",
                "Reverb",
                0.0,
                1.0,
                1.0,
                Some("")
            )
        );

        add_param!(
            PARAM_DELAY_TIME_MS,
            ParamDescriptor::float_log(
                PARAM_DELAY_TIME_MS,
                "Time",
                "Delay",
                1.0,
                2000.0,
                500.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_DELAY_FEEDBACK,
            ParamDescriptor::float(
                PARAM_DELAY_FEEDBACK,
                "Feedback",
                "Delay",
                0.0,
                0.95,
                0.3,
                Some("")
            )
        );
        add_param!(
            PARAM_DELAY_WET,
            ParamDescriptor::float(PARAM_DELAY_WET, "Wet", "Delay", 0.0, 1.0, 0.3, Some(""))
        );
        add_param!(
            PARAM_DELAY_DRY,
            ParamDescriptor::float(PARAM_DELAY_DRY, "Dry", "Delay", 0.0, 1.0, 0.7, Some(""))
        );

        add_param!(
            PARAM_CHORUS_RATE,
            ParamDescriptor::float_log(
                PARAM_CHORUS_RATE,
                "Rate",
                "Chorus",
                0.1,
                5.0,
                0.5,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_CHORUS_TEMPO_SYNC,
            ParamDescriptor::enum_param(
                PARAM_CHORUS_TEMPO_SYNC,
                "Sync",
                "Chorus",
                vec![
                    "Hz".into(),
                    "1/1".into(),
                    "1/2".into(),
                    "1/4".into(),
                    "1/8".into(),
                    "1/16".into(),
                    "1/32".into(),
                    "1/4T".into(),
                    "1/8T".into(),
                    "1/16T".into(),
                    "1/4D".into(),
                    "1/8D".into(),
                    "1/16D".into(),
                ],
                0 // Default: Hz
            )
        );
        add_param!(
            PARAM_CHORUS_DEPTH,
            ParamDescriptor::float(
                PARAM_CHORUS_DEPTH,
                "Depth",
                "Chorus",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );
        add_param!(
            PARAM_CHORUS_MIX,
            ParamDescriptor::float(PARAM_CHORUS_MIX, "Mix", "Chorus", 0.0, 1.0, 0.5, Some(""))
        );

        add_param!(
            PARAM_DISTORTION_TYPE,
            ParamDescriptor::enum_param(
                PARAM_DISTORTION_TYPE,
                "Type",
                "Distortion",
                vec![
                    "Tanh".into(),
                    "Soft Clip".into(),
                    "Hard Clip".into(),
                    "Cubic".into(),
                    "Foldback".into(),
                    "Asymmetric".into(),
                    "Sine Shaper".into(),
                    "Bitcrush".into(),
                    "Diode".into(),
                ],
                0 // Default: Tanh
            )
        );
        add_param!(
            PARAM_DISTORTION_DRIVE,
            ParamDescriptor::float(
                PARAM_DISTORTION_DRIVE,
                "Drive",
                "Distortion",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_DISTORTION_MIX,
            ParamDescriptor::float(
                PARAM_DISTORTION_MIX,
                "Mix",
                "Distortion",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );

        // Multiband Distortion parameters
        add_param!(
            PARAM_MB_DIST_LOW_MID_FREQ,
            ParamDescriptor::float_log(
                PARAM_MB_DIST_LOW_MID_FREQ,
                "Low-Mid Freq",
                "Multiband Dist",
                50.0,
                500.0,
                200.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_MB_DIST_MID_HIGH_FREQ,
            ParamDescriptor::float_log(
                PARAM_MB_DIST_MID_HIGH_FREQ,
                "Mid-High Freq",
                "Multiband Dist",
                1000.0,
                8000.0,
                2000.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_MB_DIST_DRIVE_LOW,
            ParamDescriptor::float(
                PARAM_MB_DIST_DRIVE_LOW,
                "Bass Drive",
                "Multiband Dist",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_MB_DIST_DRIVE_MID,
            ParamDescriptor::float(
                PARAM_MB_DIST_DRIVE_MID,
                "Mid Drive",
                "Multiband Dist",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_MB_DIST_DRIVE_HIGH,
            ParamDescriptor::float(
                PARAM_MB_DIST_DRIVE_HIGH,
                "High Drive",
                "Multiband Dist",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_MB_DIST_GAIN_LOW,
            ParamDescriptor::float(
                PARAM_MB_DIST_GAIN_LOW,
                "Bass Gain",
                "Multiband Dist",
                0.0,
                2.0,
                1.0,
                Some("")
            )
        );
        add_param!(
            PARAM_MB_DIST_GAIN_MID,
            ParamDescriptor::float(
                PARAM_MB_DIST_GAIN_MID,
                "Mid Gain",
                "Multiband Dist",
                0.0,
                2.0,
                1.0,
                Some("")
            )
        );
        add_param!(
            PARAM_MB_DIST_GAIN_HIGH,
            ParamDescriptor::float(
                PARAM_MB_DIST_GAIN_HIGH,
                "High Gain",
                "Multiband Dist",
                0.0,
                2.0,
                1.0,
                Some("")
            )
        );
        add_param!(
            PARAM_MB_DIST_MIX,
            ParamDescriptor::float(
                PARAM_MB_DIST_MIX,
                "Mix",
                "Multiband Dist",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );

        // Stereo Widener parameters
        add_param!(
            PARAM_WIDENER_HAAS_DELAY,
            ParamDescriptor::float(
                PARAM_WIDENER_HAAS_DELAY,
                "Haas Delay",
                "Stereo Widener",
                0.0,
                30.0,
                0.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_WIDENER_HAAS_MIX,
            ParamDescriptor::float(
                PARAM_WIDENER_HAAS_MIX,
                "Haas Mix",
                "Stereo Widener",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );
        add_param!(
            PARAM_WIDENER_WIDTH,
            ParamDescriptor::float(
                PARAM_WIDENER_WIDTH,
                "Width",
                "Stereo Widener",
                0.0,
                2.0,
                1.0,
                Some("")
            )
        );
        add_param!(
            PARAM_WIDENER_MID_GAIN,
            ParamDescriptor::float(
                PARAM_WIDENER_MID_GAIN,
                "Mid Gain",
                "Stereo Widener",
                0.0,
                2.0,
                1.0,
                Some("")
            )
        );
        add_param!(
            PARAM_WIDENER_SIDE_GAIN,
            ParamDescriptor::float(
                PARAM_WIDENER_SIDE_GAIN,
                "Side Gain",
                "Stereo Widener",
                0.0,
                2.0,
                1.0,
                Some("")
            )
        );

        // Phaser parameters
        add_param!(
            PARAM_PHASER_RATE,
            ParamDescriptor::float_log(
                PARAM_PHASER_RATE,
                "Rate",
                "Phaser",
                0.1,
                10.0,
                0.5,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_PHASER_TEMPO_SYNC,
            ParamDescriptor::enum_param(
                PARAM_PHASER_TEMPO_SYNC,
                "Sync",
                "Phaser",
                vec![
                    "Hz".into(),
                    "1/1".into(),
                    "1/2".into(),
                    "1/4".into(),
                    "1/8".into(),
                    "1/16".into(),
                    "1/32".into(),
                    "1/4T".into(),
                    "1/8T".into(),
                    "1/16T".into(),
                    "1/4D".into(),
                    "1/8D".into(),
                    "1/16D".into(),
                ],
                0 // Default: Hz
            )
        );
        add_param!(
            PARAM_PHASER_DEPTH,
            ParamDescriptor::float(
                PARAM_PHASER_DEPTH,
                "Depth",
                "Phaser",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );
        add_param!(
            PARAM_PHASER_FEEDBACK,
            ParamDescriptor::float(
                PARAM_PHASER_FEEDBACK,
                "Feedback",
                "Phaser",
                0.0,
                0.95,
                0.5,
                Some("")
            )
        );
        add_param!(
            PARAM_PHASER_MIX,
            ParamDescriptor::float(PARAM_PHASER_MIX, "Mix", "Phaser", 0.0, 1.0, 0.5, Some(""))
        );

        // Flanger parameters
        add_param!(
            PARAM_FLANGER_RATE,
            ParamDescriptor::float_log(
                PARAM_FLANGER_RATE,
                "Rate",
                "Flanger",
                0.1,
                10.0,
                0.2,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_FLANGER_TEMPO_SYNC,
            ParamDescriptor::enum_param(
                PARAM_FLANGER_TEMPO_SYNC,
                "Sync",
                "Flanger",
                vec![
                    "Hz".into(),
                    "1/1".into(),
                    "1/2".into(),
                    "1/4".into(),
                    "1/8".into(),
                    "1/16".into(),
                    "1/32".into(),
                    "1/4T".into(),
                    "1/8T".into(),
                    "1/16T".into(),
                    "1/4D".into(),
                    "1/8D".into(),
                    "1/16D".into(),
                ],
                0 // Default: Hz
            )
        );
        add_param!(
            PARAM_FLANGER_DEPTH,
            ParamDescriptor::float(
                PARAM_FLANGER_DEPTH,
                "Depth",
                "Flanger",
                0.0,
                1.0,
                0.7,
                Some("")
            )
        );
        add_param!(
            PARAM_FLANGER_FEEDBACK,
            ParamDescriptor::float(
                PARAM_FLANGER_FEEDBACK,
                "Feedback",
                "Flanger",
                0.0,
                0.95,
                0.5,
                Some("")
            )
        );
        add_param!(
            PARAM_FLANGER_MIX,
            ParamDescriptor::float(PARAM_FLANGER_MIX, "Mix", "Flanger", 0.0, 1.0, 0.5, Some(""))
        );

        // Tremolo parameters
        add_param!(
            PARAM_TREMOLO_RATE,
            ParamDescriptor::float_log(
                PARAM_TREMOLO_RATE,
                "Rate",
                "Tremolo",
                0.1,
                20.0,
                4.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_TREMOLO_TEMPO_SYNC,
            ParamDescriptor::enum_param(
                PARAM_TREMOLO_TEMPO_SYNC,
                "Sync",
                "Tremolo",
                vec![
                    "Hz".into(),
                    "1/1".into(),
                    "1/2".into(),
                    "1/4".into(),
                    "1/8".into(),
                    "1/16".into(),
                    "1/32".into(),
                    "1/4T".into(),
                    "1/8T".into(),
                    "1/16T".into(),
                    "1/4D".into(),
                    "1/8D".into(),
                    "1/16D".into(),
                ],
                0 // Default: Hz
            )
        );
        add_param!(
            PARAM_TREMOLO_DEPTH,
            ParamDescriptor::float(
                PARAM_TREMOLO_DEPTH,
                "Depth",
                "Tremolo",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );

        // Auto-Pan parameters
        add_param!(
            PARAM_AUTOPAN_RATE,
            ParamDescriptor::float_log(
                PARAM_AUTOPAN_RATE,
                "Rate",
                "Auto-Pan",
                0.1,
                20.0,
                1.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_AUTOPAN_TEMPO_SYNC,
            ParamDescriptor::enum_param(
                PARAM_AUTOPAN_TEMPO_SYNC,
                "Sync",
                "Auto-Pan",
                vec![
                    "Hz".into(),
                    "1/1".into(),
                    "1/2".into(),
                    "1/4".into(),
                    "1/8".into(),
                    "1/16".into(),
                    "1/32".into(),
                    "1/4T".into(),
                    "1/8T".into(),
                    "1/16T".into(),
                    "1/4D".into(),
                    "1/8D".into(),
                    "1/16D".into(),
                ],
                0 // Default: Hz
            )
        );
        add_param!(
            PARAM_AUTOPAN_DEPTH,
            ParamDescriptor::float(
                PARAM_AUTOPAN_DEPTH,
                "Depth",
                "Auto-Pan",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );

        // Comb Filter parameters
        add_param!(
            PARAM_COMB_FREQUENCY,
            ParamDescriptor::float_log(
                PARAM_COMB_FREQUENCY,
                "Frequency",
                "Comb Filter",
                10.0,
                10000.0,
                100.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_COMB_FEEDBACK,
            ParamDescriptor::float(
                PARAM_COMB_FEEDBACK,
                "Feedback",
                "Comb Filter",
                0.0,
                0.99,
                0.5,
                Some("")
            )
        );
        add_param!(
            PARAM_COMB_MIX,
            ParamDescriptor::float(
                PARAM_COMB_MIX,
                "Mix",
                "Comb Filter",
                0.0,
                1.0,
                0.5,
                Some("")
            )
        );

        // Ring Modulator parameters
        add_param!(
            PARAM_RINGMOD_FREQUENCY,
            ParamDescriptor::float_log(
                PARAM_RINGMOD_FREQUENCY,
                "Frequency",
                "Ring Mod",
                20.0,
                10000.0,
                440.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_RINGMOD_DEPTH,
            ParamDescriptor::float(
                PARAM_RINGMOD_DEPTH,
                "Depth",
                "Ring Mod",
                0.0,
                1.0,
                1.0,
                Some("")
            )
        );

        // Compressor parameters
        add_param!(
            PARAM_COMPRESSOR_THRESHOLD,
            ParamDescriptor::float(
                PARAM_COMPRESSOR_THRESHOLD,
                "Threshold",
                "Compressor",
                -60.0,
                0.0,
                -20.0,
                Some("dB")
            )
        );
        add_param!(
            PARAM_COMPRESSOR_RATIO,
            ParamDescriptor::float(
                PARAM_COMPRESSOR_RATIO,
                "Ratio",
                "Compressor",
                1.0,
                20.0,
                4.0,
                Some(":1")
            )
        );
        add_param!(
            PARAM_COMPRESSOR_ATTACK,
            ParamDescriptor::float_log(
                PARAM_COMPRESSOR_ATTACK,
                "Attack",
                "Compressor",
                0.1,
                100.0,
                10.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_COMPRESSOR_RELEASE,
            ParamDescriptor::float_log(
                PARAM_COMPRESSOR_RELEASE,
                "Release",
                "Compressor",
                1.0,
                1000.0,
                100.0,
                Some("ms")
            )
        );

        // Bitcrusher parameters
        add_param!(
            PARAM_BITCRUSHER_RATE,
            ParamDescriptor::float_log(
                PARAM_BITCRUSHER_RATE,
                "Sample Rate",
                "Bitcrusher",
                100.0,
                44100.0,
                44100.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_BITCRUSHER_BITS,
            ParamDescriptor::int(PARAM_BITCRUSHER_BITS, "Bit Depth", "Bitcrusher", 1, 16, 16)
        );

        // Waveshaper parameters
        add_param!(
            PARAM_WAVESHAPER_DRIVE,
            ParamDescriptor::float(
                PARAM_WAVESHAPER_DRIVE,
                "Drive",
                "Waveshaper",
                0.1,
                10.0,
                1.0,
                Some("")
            )
        );
        add_param!(
            PARAM_WAVESHAPER_MIX,
            ParamDescriptor::float(
                PARAM_WAVESHAPER_MIX,
                "Mix",
                "Waveshaper",
                0.0,
                1.0,
                1.0,
                Some("")
            )
        );

        // Effect enable/disable toggles
        add_param!(
            PARAM_PHASER_ENABLED,
            ParamDescriptor::bool(PARAM_PHASER_ENABLED, "Enabled", "Phaser", false)
        );
        add_param!(
            PARAM_FLANGER_ENABLED,
            ParamDescriptor::bool(PARAM_FLANGER_ENABLED, "Enabled", "Flanger", false)
        );
        add_param!(
            PARAM_TREMOLO_ENABLED,
            ParamDescriptor::bool(PARAM_TREMOLO_ENABLED, "Enabled", "Tremolo", false)
        );
        add_param!(
            PARAM_AUTOPAN_ENABLED,
            ParamDescriptor::bool(PARAM_AUTOPAN_ENABLED, "Enabled", "Auto-Pan", false)
        );
        add_param!(
            PARAM_COMB_ENABLED,
            ParamDescriptor::bool(PARAM_COMB_ENABLED, "Enabled", "Comb Filter", false)
        );
        add_param!(
            PARAM_RINGMOD_ENABLED,
            ParamDescriptor::bool(PARAM_RINGMOD_ENABLED, "Enabled", "Ring Modulator", false)
        );
        add_param!(
            PARAM_COMPRESSOR_ENABLED,
            ParamDescriptor::bool(PARAM_COMPRESSOR_ENABLED, "Enabled", "Compressor", false)
        );
        add_param!(
            PARAM_BITCRUSHER_ENABLED,
            ParamDescriptor::bool(PARAM_BITCRUSHER_ENABLED, "Enabled", "Bitcrusher", false)
        );
        add_param!(
            PARAM_WAVESHAPER_ENABLED,
            ParamDescriptor::bool(PARAM_WAVESHAPER_ENABLED, "Enabled", "Waveshaper", false)
        );

        // Exciter
        add_param!(
            PARAM_EXCITER_FREQUENCY,
            ParamDescriptor::float_log(
                PARAM_EXCITER_FREQUENCY,
                "Frequency",
                "Exciter",
                2000.0,
                12000.0,
                5000.0,
                Some("Hz")
            )
        );
        add_param!(
            PARAM_EXCITER_DRIVE,
            ParamDescriptor::float(
                PARAM_EXCITER_DRIVE,
                "Drive",
                "Exciter",
                0.0,
                1.0,
                0.5,
                Some("%")
            )
        );
        add_param!(
            PARAM_EXCITER_MIX,
            ParamDescriptor::float(
                PARAM_EXCITER_MIX,
                "Mix",
                "Exciter",
                0.0,
                1.0,
                0.3,
                Some("%")
            )
        );
        add_param!(
            PARAM_EXCITER_ENABLED,
            ParamDescriptor::bool(PARAM_EXCITER_ENABLED, "Enabled", "Exciter", false)
        );

        // Effect enable/disable toggles
        add_param!(
            PARAM_REVERB_ENABLED,
            ParamDescriptor::bool(PARAM_REVERB_ENABLED, "Enabled", "Reverb", false)
        );
        add_param!(
            PARAM_DELAY_ENABLED,
            ParamDescriptor::bool(PARAM_DELAY_ENABLED, "Enabled", "Delay", false)
        );
        add_param!(
            PARAM_CHORUS_ENABLED,
            ParamDescriptor::bool(PARAM_CHORUS_ENABLED, "Enabled", "Chorus", false)
        );
        add_param!(
            PARAM_DISTORTION_ENABLED,
            ParamDescriptor::bool(PARAM_DISTORTION_ENABLED, "Enabled", "Distortion", false)
        );
        add_param!(
            PARAM_MB_DIST_ENABLED,
            ParamDescriptor::bool(
                PARAM_MB_DIST_ENABLED,
                "Enabled",
                "Multiband Distortion",
                false
            )
        );
        add_param!(
            PARAM_WIDENER_ENABLED,
            ParamDescriptor::bool(PARAM_WIDENER_ENABLED, "Enabled", "Stereo Widener", false)
        );

        // Voice Compressor parameters (per-voice transient control)
        add_param!(
            PARAM_VOICE_COMP_ENABLED,
            ParamDescriptor::bool(
                PARAM_VOICE_COMP_ENABLED,
                "Enabled",
                "Voice Compressor",
                false
            )
        );
        add_param!(
            PARAM_VOICE_COMP_THRESHOLD,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_THRESHOLD,
                "Threshold",
                "Voice Compressor",
                -60.0,
                0.0,
                -12.0,
                Some("dB")
            )
        );
        add_param!(
            PARAM_VOICE_COMP_RATIO,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_RATIO,
                "Ratio",
                "Voice Compressor",
                1.0,
                20.0,
                3.0,
                Some(":1")
            )
        );
        add_param!(
            PARAM_VOICE_COMP_ATTACK,
            ParamDescriptor::float_log(
                PARAM_VOICE_COMP_ATTACK,
                "Attack",
                "Voice Compressor",
                0.1,
                50.0,
                1.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_VOICE_COMP_RELEASE,
            ParamDescriptor::float_log(
                PARAM_VOICE_COMP_RELEASE,
                "Release",
                "Voice Compressor",
                10.0,
                200.0,
                50.0,
                Some("ms")
            )
        );
        add_param!(
            PARAM_VOICE_COMP_KNEE,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_KNEE,
                "Knee",
                "Voice Compressor",
                0.0,
                20.0,
                3.0,
                Some("dB")
            )
        );
        add_param!(
            PARAM_VOICE_COMP_MAKEUP,
            ParamDescriptor::float(
                PARAM_VOICE_COMP_MAKEUP,
                "Makeup Gain",
                "Voice Compressor",
                0.0,
                30.0,
                0.0,
                Some("dB")
            )
        );

        // Transient Shaper
        add_param!(
            PARAM_TRANSIENT_ENABLED,
            ParamDescriptor::bool(
                PARAM_TRANSIENT_ENABLED,
                "Enabled",
                "Transient Shaper",
                false
            )
        );
        add_param!(
            PARAM_TRANSIENT_ATTACK,
            ParamDescriptor::float(
                PARAM_TRANSIENT_ATTACK,
                "Attack Boost",
                "Transient Shaper",
                0.0,
                1.0,
                0.0,
                Some("%")
            )
        );
        add_param!(
            PARAM_TRANSIENT_SUSTAIN,
            ParamDescriptor::float(
                PARAM_TRANSIENT_SUSTAIN,
                "Sustain Reduction",
                "Transient Shaper",
                0.0,
                1.0,
                0.0,
                Some("%")
            )
        );

        // Wavetable parameters (Oscillator 1)
        add_param!(
            PARAM_OSC1_WAVETABLE_INDEX,
            ParamDescriptor::int(
                PARAM_OSC1_WAVETABLE_INDEX,
                "Wavetable",
                "Oscillator 1",
                0,
                63, // Support up to 64 wavetables (built-in + custom)
                0
            )
        );
        add_param!(
            PARAM_OSC1_WAVETABLE_POSITION,
            ParamDescriptor::float(
                PARAM_OSC1_WAVETABLE_POSITION,
                "WT Position",
                "Oscillator 1",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );

        // Wavetable parameters (Oscillator 2)
        add_param!(
            PARAM_OSC2_WAVETABLE_INDEX,
            ParamDescriptor::int(
                PARAM_OSC2_WAVETABLE_INDEX,
                "Wavetable",
                "Oscillator 2",
                0,
                63,
                0
            )
        );
        add_param!(
            PARAM_OSC2_WAVETABLE_POSITION,
            ParamDescriptor::float(
                PARAM_OSC2_WAVETABLE_POSITION,
                "WT Position",
                "Oscillator 2",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );

        // Wavetable parameters (Oscillator 3)
        add_param!(
            PARAM_OSC3_WAVETABLE_INDEX,
            ParamDescriptor::int(
                PARAM_OSC3_WAVETABLE_INDEX,
                "Wavetable",
                "Oscillator 3",
                0,
                63,
                0
            )
        );
        add_param!(
            PARAM_OSC3_WAVETABLE_POSITION,
            ParamDescriptor::float(
                PARAM_OSC3_WAVETABLE_POSITION,
                "WT Position",
                "Oscillator 3",
                0.0,
                1.0,
                0.0,
                Some("")
            )
        );

        // Unison normalization toggles
        add_param!(
            PARAM_OSC1_UNISON_NORMALIZE,
            ParamDescriptor::bool(
                PARAM_OSC1_UNISON_NORMALIZE,
                "Unison Norm",
                "Oscillator 1",
                true
            )
        );
        add_param!(
            PARAM_OSC2_UNISON_NORMALIZE,
            ParamDescriptor::bool(
                PARAM_OSC2_UNISON_NORMALIZE,
                "Unison Norm",
                "Oscillator 2",
                true
            )
        );
        add_param!(
            PARAM_OSC3_UNISON_NORMALIZE,
            ParamDescriptor::bool(
                PARAM_OSC3_UNISON_NORMALIZE,
                "Unison Norm",
                "Oscillator 3",
                true
            )
        );

        // Sort parameter IDs for consistent iteration
        param_ids.sort();

        Self {
            descriptors,
            param_ids,
        }
    }

    /// Get descriptor for a parameter ID
    pub fn get(&self, param_id: ParamId) -> Option<&ParamDescriptor> {
        self.descriptors.get(&param_id)
    }

    /// Get descriptor by index (for iteration)
    pub fn get_by_index(&self, index: usize) -> Option<&ParamDescriptor> {
        self.param_ids
            .get(index)
            .and_then(|id| self.descriptors.get(id))
    }

    /// Get parameter ID by index
    pub fn get_id_by_index(&self, index: usize) -> Option<ParamId> {
        self.param_ids.get(index).copied()
    }

    /// Total number of parameters
    pub fn count(&self) -> usize {
        self.param_ids.len()
    }

    /// Iterate over all parameter IDs
    pub fn iter_ids(&self) -> impl Iterator<Item = ParamId> + '_ {
        self.param_ids.iter().copied()
    }

    /// Iterate over all descriptors
    pub fn iter_descriptors(&self) -> impl Iterator<Item = &ParamDescriptor> {
        self.param_ids
            .iter()
            .filter_map(|id| self.descriptors.get(id))
    }

    /// Find parameters by module name
    pub fn find_by_module(&self, module: &str) -> Vec<&ParamDescriptor> {
        self.descriptors
            .values()
            .filter(|desc| desc.module == module)
            .collect()
    }

    /// Find parameters by name (partial match)
    pub fn find_by_name(&self, name: &str) -> Vec<&ParamDescriptor> {
        let name_lower = name.to_lowercase();
        self.descriptors
            .values()
            .filter(|desc| desc.name.to_lowercase().contains(&name_lower))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_initialization() {
        let registry = get_registry();
        // Expect at least 45 parameters (3 oscs × 19 + 2 master + 15 filter/lfo + 4 env + 2 vel + 15 effects)
        assert!(
            registry.count() >= 100,
            "Registry should have many parameters"
        );
    }

    #[test]
    fn test_all_params_accessible() {
        let registry = get_registry();

        // Test master parameters
        assert!(registry.get(PARAM_MASTER_GAIN).is_some());
        assert!(registry.get(PARAM_MONOPHONIC).is_some());

        // Test oscillator parameters
        assert!(registry.get(PARAM_OSC1_PITCH).is_some());
        assert!(registry.get(PARAM_OSC2_DETUNE).is_some());
        assert!(registry.get(PARAM_OSC3_GAIN).is_some());

        // Test filter parameters
        assert!(registry.get(PARAM_FILTER1_CUTOFF).is_some());
        assert!(registry.get(PARAM_FILTER3_RESONANCE).is_some());

        // Test effects
        assert!(registry.get(PARAM_REVERB_WET).is_some());
        assert!(registry.get(PARAM_DELAY_TIME_MS).is_some());
        assert!(registry.get(PARAM_DISTORTION_DRIVE).is_some());
    }

    #[test]
    fn test_denormalization() {
        let registry = get_registry();

        // Test frequency parameter (logarithmic)
        if let Some(cutoff) = registry.get(PARAM_FILTER1_CUTOFF) {
            let at_normalized_half = cutoff.denormalize(0.5);
            // Should be around 632 Hz (geometric mean of 20 and 20000)
            assert!(at_normalized_half > 500.0 && at_normalized_half < 700.0);
        }

        // Test linear parameter
        if let Some(gain) = registry.get(PARAM_REVERB_WET) {
            let at_half = gain.denormalize(0.5);
            assert!((at_half - 0.5).abs() < 0.01);
        }
    }

    #[test]
    fn test_find_by_module() {
        let registry = get_registry();
        let osc1_params = registry.find_by_module("Oscillator 1");
        assert!(
            osc1_params.len() > 10,
            "Should find many Oscillator 1 parameters"
        );
    }

    #[test]
    fn test_find_by_name() {
        let registry = get_registry();
        let pitch_params = registry.find_by_name("pitch");
        assert!(
            pitch_params.len() >= 3,
            "Should find pitch parameters for all oscillators"
        );
    }
}
