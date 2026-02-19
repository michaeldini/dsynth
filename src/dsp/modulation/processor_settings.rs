/// Complete multiband processor settings for perceptual voice processing
///
/// These structures contain the technical parameters that processors need,
/// computed from perceptual controls by the ParameterMapper.

#[derive(Debug, Clone)]
pub struct ProcessorSettings {
    pub saturation: MultibandSaturation,
    pub compression: MultibandCompression,
    pub transient_shaping: MultibandTransientShaping,
    pub harmonic_enhancement: MultibandHarmonicEnhancement,
    pub dynamic_eq: MultibandDynamicEq,
    pub vocal_doubler: VocalDoublerSettings,
    pub intelligent_exciter: IntelligentExciterSettings,
}

/// Saturation drive amounts for each frequency band
#[derive(Debug, Clone)]
pub struct MultibandSaturation {
    pub bass_drive: f32,     // 0.0 to 1.0
    pub mid_drive: f32,      // 0.0 to 1.0
    pub presence_drive: f32, // 0.0 to 1.0
    pub air_drive: f32,      // 0.0 to 1.0
}

/// Compression settings for each frequency band
#[derive(Debug, Clone)]
pub struct MultibandCompression {
    pub bass_ratio: f32,         // 1.0 to 6.0
    pub bass_threshold: f32,     // -30.0 to 0.0 dB
    pub mid_ratio: f32,          // 1.0 to 6.0
    pub mid_threshold: f32,      // -30.0 to 0.0 dB
    pub presence_ratio: f32,     // 1.0 to 6.0
    pub presence_threshold: f32, // -30.0 to 0.0 dB
    pub air_ratio: f32,          // 1.0 to 4.0
    pub air_threshold: f32,      // -20.0 to 0.0 dB
}

/// Transient shaping amounts for each frequency band
#[derive(Debug, Clone)]
pub struct MultibandTransientShaping {
    pub bass_attack: f32,     // -1.0 to +1.0 (soften ← → punch)
    pub mid_attack: f32,      // -1.0 to +1.0
    pub presence_attack: f32, // -1.0 to +1.0
    pub air_attack: f32,      // -1.0 to +1.0
}

/// Harmonic enhancement amounts for each frequency band
#[derive(Debug, Clone)]
pub struct MultibandHarmonicEnhancement {
    pub bass_harmonics: f32,     // 0.0 to 1.0 (subtle harmonic generation)
    pub mid_harmonics: f32,      // 0.0 to 1.0
    pub presence_harmonics: f32, // 0.0 to 1.0
    pub air_harmonics: f32,      // 0.0 to 1.0
}

/// Dynamic EQ adjustments for each frequency band
#[derive(Debug, Clone)]
pub struct MultibandDynamicEq {
    pub bass_gain_db: f32,     // -6.0 to +6.0 dB (level-dependent)
    pub mid_gain_db: f32,      // -6.0 to +6.0 dB
    pub presence_gain_db: f32, // -6.0 to +6.0 dB
    pub air_gain_db: f32,      // -6.0 to +6.0 dB
}

/// Vocal doubler settings controlled by perceptual parameters
#[derive(Debug, Clone)]
pub struct VocalDoublerSettings {
    pub amount: f32,       // 0.0 to 1.0 overall intensity
    pub stereo_width: f32, // 0.0 to 1.0 stereo spread
}

/// Intelligent exciter settings for pitch-tracked vocal sparkle
#[derive(Debug, Clone)]
pub struct IntelligentExciterSettings {
    pub amount: f32, // 0.0 to 1.0 harmonic generation amount
    pub mix: f32,    // 0.0 to 1.0 wet/dry balance
}

impl Default for ProcessorSettings {
    fn default() -> Self {
        Self {
            saturation: MultibandSaturation {
                bass_drive: 0.3,
                mid_drive: 0.25,
                presence_drive: 0.2,
                air_drive: 0.1,
            },
            compression: MultibandCompression {
                bass_ratio: 2.2,
                bass_threshold: -18.0,
                mid_ratio: 3.0,
                mid_threshold: -16.0,
                presence_ratio: 2.5,
                presence_threshold: -12.0,
                air_ratio: 1.8,
                air_threshold: -10.0,
            },
            transient_shaping: MultibandTransientShaping {
                bass_attack: 0.0,
                mid_attack: 0.1,
                presence_attack: 0.2,
                air_attack: 0.15,
            },
            harmonic_enhancement: MultibandHarmonicEnhancement {
                bass_harmonics: 0.2,
                mid_harmonics: 0.15,
                presence_harmonics: 0.25,
                air_harmonics: 0.3,
            },
            dynamic_eq: MultibandDynamicEq {
                bass_gain_db: 0.0,
                mid_gain_db: 0.0,
                presence_gain_db: 0.0,
                air_gain_db: 0.0,
            },
            vocal_doubler: VocalDoublerSettings {
                amount: 0.4,       // Moderate default doubling
                stereo_width: 0.6, // Nice stereo spread
            },
            intelligent_exciter: IntelligentExciterSettings {
                amount: 0.3, // Moderate harmonic generation
                mix: 0.4,    // Balanced wet/dry mix
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_settings_creation() {
        let settings = ProcessorSettings::default();
        assert!(settings.saturation.bass_drive >= 0.0);
        assert!(settings.saturation.bass_drive <= 1.0);
        assert!(settings.compression.bass_ratio >= 1.0);
        assert!(settings.transient_shaping.bass_attack >= -1.0);
        assert!(settings.transient_shaping.bass_attack <= 1.0);
    }

    #[test]
    fn test_multiband_saturation() {
        let saturation = MultibandSaturation {
            bass_drive: 0.5,
            mid_drive: 0.4,
            presence_drive: 0.6,
            air_drive: 0.2,
        };

        // All drives should be in valid range
        assert!(saturation.bass_drive >= 0.0 && saturation.bass_drive <= 1.0);
        assert!(saturation.mid_drive >= 0.0 && saturation.mid_drive <= 1.0);
        assert!(saturation.presence_drive >= 0.0 && saturation.presence_drive <= 1.0);
        assert!(saturation.air_drive >= 0.0 && saturation.air_drive <= 1.0);
    }

    #[test]
    fn test_multiband_compression() {
        let compression = MultibandCompression {
            bass_ratio: 2.0,
            bass_threshold: -18.0,
            mid_ratio: 3.0,
            mid_threshold: -16.0,
            presence_ratio: 2.5,
            presence_threshold: -12.0,
            air_ratio: 1.5,
            air_threshold: -10.0,
        };

        // Ratios should be >= 1.0
        assert!(compression.bass_ratio >= 1.0);
        assert!(compression.mid_ratio >= 1.0);
        assert!(compression.presence_ratio >= 1.0);
        assert!(compression.air_ratio >= 1.0);

        // Thresholds should be negative or zero
        assert!(compression.bass_threshold <= 0.0);
        assert!(compression.mid_threshold <= 0.0);
        assert!(compression.presence_threshold <= 0.0);
        assert!(compression.air_threshold <= 0.0);
    }
}
