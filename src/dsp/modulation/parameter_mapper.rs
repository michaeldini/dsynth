use super::processor_settings::*;
/// Lightweight parameter mapper for real-time perceptual → technical conversion
///
/// Maps 4 perceptual controls (character, intensity, presence, dynamics) to
/// comprehensive multiband processor settings based on signal analysis.
///
/// Optimized for <100 CPU operations per call to maintain real-time safety.
use crate::dsp::signal_analyzer::SignalAnalysis;
use crate::params_voice::VoiceParams;

pub struct ParameterMapper;

impl ParameterMapper {
    /// Map perceptual parameters to comprehensive multiband settings
    ///
    /// # Performance Target
    /// < 100 CPU operations per call for real-time safety
    pub fn map_parameters(params: &VoiceParams, analysis: &SignalAnalysis) -> ProcessorSettings {
        // Pre-compute common values to avoid repeated calculations
        let character = params.character;
        let intensity = params.intensity;
        let presence = params.presence;
        let dynamics = params.dynamics;

        // Signal-aware scaling factors (computed once)
        let transient_factor = if analysis.is_transient && analysis.transient_strength > 0.6 {
            0.8 // Gentler on strong transients
        } else {
            1.0
        };

        let signal_brightness = match analysis.signal_type {
            crate::dsp::analysis::SignalType::Tonal => 1.0,
            crate::dsp::analysis::SignalType::Mixed => 0.9,
            crate::dsp::analysis::SignalType::Noisy => 0.7,
            crate::dsp::analysis::SignalType::VeryNoisy => 0.5,
        };

        ProcessorSettings {
            saturation: Self::map_saturation(character, intensity, transient_factor),
            compression: Self::map_compression(presence, dynamics, analysis.rms_level),
            transient_shaping: Self::map_transient_shaping(character, presence, analysis),
            harmonic_enhancement: Self::map_harmonic_enhancement(
                character,
                intensity,
                signal_brightness,
            ),
            dynamic_eq: Self::map_dynamic_eq(character, presence, analysis.rms_level),
            vocal_doubler: Self::map_vocal_doubler(character, intensity, presence, analysis),
            intelligent_exciter: Self::map_intelligent_exciter(presence, character, analysis),
        }
    }

    /// Map character + intensity → multiband saturation with band-specific curves
    fn map_saturation(
        character: f32,
        intensity: f32,
        transient_factor: f32,
    ) -> MultibandSaturation {
        // Base drive from intensity (0.0-1.0 → 0.05-0.85 range)
        let base_drive = 0.05 + intensity * 0.8;

        MultibandSaturation {
            // Bass: Warm = more bass saturation, Bright = controlled bass
            bass_drive: (base_drive
                * if character < 0.0 {
                    1.0 + (-character) * 0.6 // Up to 60% more when warm
                } else {
                    1.0 - character * 0.4 // Up to 40% less when bright
                }
                * transient_factor)
                .clamp(0.0, 1.0),

            // Mid: Always moderate, slight character influence
            mid_drive: (base_drive * 0.8 * (1.0 + character.abs() * 0.2)).clamp(0.0, 1.0),

            // Presence: Bright = more presence saturation
            presence_drive: (base_drive
                * if character > 0.0 {
                    1.0 + character * 0.7 // Up to 70% more when bright
                } else {
                    1.0 + character * 0.1 // Slight reduction when warm
                }
                * transient_factor)
                .clamp(0.0, 1.0),

            // Air: Always gentle, brightness-dependent
            air_drive: (base_drive * 0.4 * (1.0 + character.max(0.0) * 0.5)).clamp(0.0, 0.6),
        }
    }

    /// Map presence + dynamics → intelligent multiband compression
    fn map_compression(presence: f32, dynamics: f32, rms_level: f32) -> MultibandCompression {
        // Dynamics: -1=heavy compression, +1=light compression
        let compression_intensity = 0.5 - dynamics * 0.35; // 0.15 to 0.85

        // Presence affects mid/presence band compression
        let mid_compression_scale = if presence > 0.0 {
            1.0 - presence * 0.3 // Less mid compression when intimate
        } else {
            1.0 + (-presence) * 0.4 // More mid compression when distant
        };

        // Adaptive thresholds based on input level
        let threshold_offset = if rms_level > 0.6 {
            -3.0 // Lower thresholds for hot signals
        } else if rms_level < 0.2 {
            2.0 // Higher thresholds for quiet signals
        } else {
            0.0
        };

        MultibandCompression {
            bass_ratio: (1.5 + compression_intensity * 2.0).clamp(1.1, 4.0), // 1.5:1 to 3.5:1
            bass_threshold: (-18.0_f32 + threshold_offset).clamp(-30.0, 0.0),

            mid_ratio: ((2.0 + compression_intensity * 2.5) * mid_compression_scale)
                .clamp(1.1, 6.0),
            mid_threshold: (-16.0_f32 + threshold_offset).clamp(-30.0, 0.0),

            presence_ratio: (1.8 + compression_intensity * 1.7).clamp(1.1, 4.0), // Gentler on presence
            presence_threshold: (-12.0_f32 + threshold_offset).clamp(-30.0, 0.0),

            air_ratio: (1.3 + compression_intensity * 1.2).clamp(1.1, 2.5), // Very gentle on air
            air_threshold: (-8.0_f32 + threshold_offset).clamp(-20.0, 0.0),
        }
    }

    /// Map character + presence → transient shaping per band
    fn map_transient_shaping(
        character: f32,
        presence: f32,
        analysis: &SignalAnalysis,
    ) -> MultibandTransientShaping {
        // Base transient intensity from presence and character
        let base_attack = presence * 0.5 + character * 0.3;

        // Reduce transient shaping on already transient-heavy material
        let transient_reduction = if analysis.is_transient && analysis.transient_strength > 0.7 {
            0.6
        } else {
            1.0
        };

        MultibandTransientShaping {
            // Bass: Gentle transient shaping to preserve power
            bass_attack: (base_attack * 0.3 * transient_reduction).clamp(-1.0, 1.0),

            // Mid: Moderate transient shaping for clarity
            mid_attack: (base_attack * 0.6 * transient_reduction).clamp(-1.0, 1.0),

            // Presence: Most aggressive transient shaping for clarity/punch
            presence_attack: (base_attack * 0.9 * transient_reduction).clamp(-1.0, 1.0),

            // Air: Subtle transient shaping to preserve sparkle
            air_attack: (base_attack * 0.4 * transient_reduction).clamp(-1.0, 1.0),
        }
    }

    /// Map character + intensity → harmonic enhancement per band
    fn map_harmonic_enhancement(
        character: f32,
        intensity: f32,
        signal_brightness: f32,
    ) -> MultibandHarmonicEnhancement {
        // Base harmonic generation from intensity
        let base_harmonics = intensity * 0.4; // 0.0 to 0.4 range

        MultibandHarmonicEnhancement {
            // Bass: Even-order harmonics for warmth (more when warm)
            bass_harmonics: (base_harmonics
                * if character < 0.0 {
                    1.0 + (-character) * 0.5 // More bass harmonics when warm
                } else {
                    1.0 - character * 0.3 // Less when bright
                })
            .clamp(0.0, 1.0),

            // Mid: Moderate harmonic enhancement
            mid_harmonics: (base_harmonics * 0.7).clamp(0.0, 1.0),

            // Presence: Odd-order harmonics for brightness (more when bright)
            presence_harmonics: (base_harmonics
                * if character > 0.0 {
                    1.0 + character * 0.8 // More presence harmonics when bright
                } else {
                    1.0 + character * 0.2 // Slight reduction when warm
                }
                * signal_brightness)
                .clamp(0.0, 1.0),

            // Air: Always gentle, brightness-dependent
            air_harmonics: (base_harmonics
                * 0.5
                * (1.0 + character.max(0.0) * 0.6)
                * signal_brightness)
                .clamp(0.0, 1.0),
        }
    }

    /// Map character + presence → dynamic EQ adjustments
    fn map_dynamic_eq(character: f32, presence: f32, rms_level: f32) -> MultibandDynamicEq {
        // Level-dependent EQ adjustments
        let level_factor = rms_level.clamp(0.1, 1.0);

        MultibandDynamicEq {
            // Bass: Warm character boosts bass, reduce bass when signal is hot
            bass_gain_db: (if character < 0.0 {
                -character * 2.0 // Up to +2dB boost when warm
            } else {
                0.0
            } - if level_factor > 0.7 {
                (level_factor - 0.7) * 6.0
            } else {
                0.0
            })
            .clamp(-4.0, 3.0),

            // Mid: Slight reduction when very loud to prevent muddiness
            mid_gain_db: (-if level_factor > 0.8 {
                (level_factor - 0.8) * 5.0
            } else {
                0.0
            })
            .clamp(-2.0, 0.0),

            // Presence: Bright character boosts presence, intimate presence boosts
            presence_gain_db: (character * 1.5 + presence * 1.0
                - if level_factor > 0.75 {
                    (level_factor - 0.75) * 8.0
                } else {
                    0.0
                })
            .clamp(-3.0, 4.0),

            // Air: Bright character and intimate presence boost air
            air_gain_db: ((character.max(0.0) * 1.2 + presence.max(0.0) * 0.8)
                - if level_factor > 0.8 {
                    (level_factor - 0.8) * 4.0
                } else {
                    0.0
                })
            .clamp(-2.0, 3.0),
        }
    }

    /// Map character + intensity + presence → vocal doubler settings
    fn map_vocal_doubler(
        character: f32,
        intensity: f32,
        presence: f32,
        analysis: &SignalAnalysis,
    ) -> VocalDoublerSettings {
        // Base doubling amount from intensity (main control)
        let base_amount = intensity * 0.8; // Scale down from max 1.0 to 0.8 for musicality

        // Character influences doubler behavior:
        // Warm character (-1) = more doubling for thickness
        // Bright character (+1) = less doubling to maintain clarity
        let character_scaling = if character < 0.0 {
            1.0 + (-character * 0.4) // Up to +40% more doubling when warm
        } else {
            1.0 - (character * 0.3) // Up to -30% less doubling when bright
        };

        // Reduce doubling on transients to preserve punch
        let transient_reduction = if analysis.is_transient && analysis.transient_strength > 0.6 {
            0.6 // Reduce to 60% on strong transients
        } else {
            1.0
        };

        // Final doubling amount
        let final_amount = (base_amount * character_scaling * transient_reduction).clamp(0.0, 1.0);

        // Stereo width from presence:
        // Distant (-1) = narrow stereo image
        // Intimate (+1) = wide stereo image for immersive effect
        let stereo_width = (0.5 + presence * 0.4).clamp(0.1, 0.9); // 0.1 to 0.9 range

        VocalDoublerSettings {
            amount: final_amount,
            stereo_width,
        }
    }

    /// Map presence + character → intelligent exciter settings
    fn map_intelligent_exciter(
        presence: f32,
        character: f32,
        analysis: &SignalAnalysis,
    ) -> IntelligentExciterSettings {
        // Presence is primary control for vocal air & sparkle:
        // Distant (-1) = minimal sparkle, Intimate (+1) = maximum sparkle
        let base_amount = (0.1 + (presence + 1.0) * 0.25).clamp(0.0, 0.6); // 0.1 to 0.6 range
        let base_mix = (0.2 + (presence + 1.0) * 0.25).clamp(0.1, 0.7); // 0.2 to 0.7 range

        // Character influences the harmonic richness:
        // Bright character (+1) = more harmonics, Warm character (-1) = gentler
        let character_scaling = 1.0 + character * 0.3; // 0.7x to 1.3x scaling

        // Reduce enhancement on already bright/sibilant content
        let content_reduction = if analysis.has_sibilance && analysis.sibilance_strength > 0.5 {
            0.6 // Much gentler on sibilants
        } else if analysis.signal_type == crate::dsp::analysis::SignalType::VeryNoisy {
            0.8 // Gentler on noisy content
        } else {
            1.0
        };

        let final_amount = (base_amount * character_scaling * content_reduction).clamp(0.0, 0.6);
        let final_mix = (base_mix * content_reduction).clamp(0.1, 0.7);

        IntelligentExciterSettings {
            amount: final_amount,
            mix: final_mix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::SignalType;

    fn create_test_analysis() -> SignalAnalysis {
        SignalAnalysis {
            rms_level: 0.4,
            peak_level: 0.6,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 200.0,
            signal_type: SignalType::Tonal,
            is_voiced: true,
            is_unvoiced: false,
            has_sibilance: false,
            sibilance_strength: 0.0,
            pitch_hz: 220.0,
            pitch_confidence: 0.0,
            is_pitched: false,
        }
    }

    #[test]
    fn test_parameter_mapping() {
        let params = VoiceParams::default();
        let analysis = create_test_analysis();

        let settings = ParameterMapper::map_parameters(&params, &analysis);

        // Verify all settings are in valid ranges
        assert!(settings.saturation.bass_drive >= 0.0 && settings.saturation.bass_drive <= 1.0);
        assert!(settings.compression.bass_ratio >= 1.0);
        assert!(
            settings.transient_shaping.bass_attack >= -1.0
                && settings.transient_shaping.bass_attack <= 1.0
        );
        assert!(
            settings.harmonic_enhancement.bass_harmonics >= 0.0
                && settings.harmonic_enhancement.bass_harmonics <= 1.0
        );
        assert!(
            settings.dynamic_eq.bass_gain_db >= -6.0 && settings.dynamic_eq.bass_gain_db <= 6.0
        );
    }

    #[test]
    fn test_warm_vs_bright_character() {
        let mut warm_params = VoiceParams::default();
        warm_params.character = -0.8; // Very warm

        let mut bright_params = VoiceParams::default();
        bright_params.character = 0.8; // Very bright

        let analysis = create_test_analysis();

        let warm_settings = ParameterMapper::map_parameters(&warm_params, &analysis);
        let bright_settings = ParameterMapper::map_parameters(&bright_params, &analysis);

        // Warm should have more bass saturation, less presence
        assert!(warm_settings.saturation.bass_drive > bright_settings.saturation.bass_drive);
        assert!(
            warm_settings.saturation.presence_drive < bright_settings.saturation.presence_drive
        );

        // Warm should boost bass EQ, bright should boost presence/air
        assert!(warm_settings.dynamic_eq.bass_gain_db > bright_settings.dynamic_eq.bass_gain_db);
        assert!(
            warm_settings.dynamic_eq.presence_gain_db < bright_settings.dynamic_eq.presence_gain_db
        );
    }

    #[test]
    fn test_transient_adaptation() {
        let params = VoiceParams::default();

        let mut transient_analysis = create_test_analysis();
        transient_analysis.is_transient = true;
        transient_analysis.transient_strength = 0.8;

        let non_transient_analysis = create_test_analysis();

        let transient_settings = ParameterMapper::map_parameters(&params, &transient_analysis);
        let normal_settings = ParameterMapper::map_parameters(&params, &non_transient_analysis);

        // Transient material should get gentler saturation
        assert!(transient_settings.saturation.bass_drive < normal_settings.saturation.bass_drive);
        assert!(
            transient_settings.saturation.presence_drive
                < normal_settings.saturation.presence_drive
        );
    }

    #[test]
    fn test_intensity_scaling() {
        let mut gentle_params = VoiceParams::default();
        gentle_params.intensity = 0.1;

        let mut aggressive_params = VoiceParams::default();
        aggressive_params.intensity = 0.9;

        let analysis = create_test_analysis();

        let gentle_settings = ParameterMapper::map_parameters(&gentle_params, &analysis);
        let aggressive_settings = ParameterMapper::map_parameters(&aggressive_params, &analysis);

        // Higher intensity should increase saturation and harmonics
        assert!(aggressive_settings.saturation.bass_drive > gentle_settings.saturation.bass_drive);
        assert!(
            aggressive_settings.harmonic_enhancement.presence_harmonics
                > gentle_settings.harmonic_enhancement.presence_harmonics
        );
    }

    #[test]
    fn test_dynamics_compression_mapping() {
        let mut compressed_params = VoiceParams::default();
        compressed_params.dynamics = -0.8; // Very compressed

        let mut dynamic_params = VoiceParams::default();
        dynamic_params.dynamics = 0.8; // Very dynamic

        let analysis = create_test_analysis();

        let compressed_settings = ParameterMapper::map_parameters(&compressed_params, &analysis);
        let dynamic_settings = ParameterMapper::map_parameters(&dynamic_params, &analysis);

        // Compressed setting should have higher compression ratios
        assert!(
            compressed_settings.compression.bass_ratio > dynamic_settings.compression.bass_ratio
        );
        assert!(compressed_settings.compression.mid_ratio > dynamic_settings.compression.mid_ratio);
    }

    #[test]
    fn test_vocal_doubler_mapping() {
        let mut low_intensity = VoiceParams::default();
        low_intensity.intensity = 0.1; // Low doubling
        low_intensity.character = 0.5; // Bright (less doubling)
        low_intensity.presence = -0.5; // Distant (narrow stereo)

        let mut high_intensity = VoiceParams::default();
        high_intensity.intensity = 0.9; // High doubling
        high_intensity.character = -0.5; // Warm (more doubling)
        high_intensity.presence = 0.5; // Intimate (wide stereo)

        let analysis = create_test_analysis();

        let low_settings = ParameterMapper::map_parameters(&low_intensity, &analysis);
        let high_settings = ParameterMapper::map_parameters(&high_intensity, &analysis);

        // High intensity should have more doubling
        assert!(high_settings.vocal_doubler.amount > low_settings.vocal_doubler.amount);

        // Intimate presence should have wider stereo
        assert!(high_settings.vocal_doubler.stereo_width > low_settings.vocal_doubler.stereo_width);

        // Values should be in valid range
        assert!(
            low_settings.vocal_doubler.amount >= 0.0 && low_settings.vocal_doubler.amount <= 1.0
        );
        assert!(
            high_settings.vocal_doubler.amount >= 0.0 && high_settings.vocal_doubler.amount <= 1.0
        );
        assert!(
            low_settings.vocal_doubler.stereo_width >= 0.1
                && low_settings.vocal_doubler.stereo_width <= 0.9
        );
        assert!(
            high_settings.vocal_doubler.stereo_width >= 0.1
                && high_settings.vocal_doubler.stereo_width <= 0.9
        );
    }

    #[test]
    fn test_intelligent_exciter_mapping() {
        let mut distant = VoiceParams::default();
        distant.presence = -1.0; // Distant (minimal sparkle)
        distant.character = -0.5; // Warm (gentler harmonics)

        let mut intimate = VoiceParams::default();
        intimate.presence = 1.0; // Intimate (maximum sparkle)
        intimate.character = 0.5; // Bright (more harmonics)

        let analysis = create_test_analysis();

        let distant_settings = ParameterMapper::map_parameters(&distant, &analysis);
        let intimate_settings = ParameterMapper::map_parameters(&intimate, &analysis);

        // Intimate presence should have more sparkle
        assert!(
            intimate_settings.intelligent_exciter.amount
                > distant_settings.intelligent_exciter.amount
        );
        assert!(
            intimate_settings.intelligent_exciter.mix > distant_settings.intelligent_exciter.mix
        );

        // Values should be in valid ranges
        assert!(
            distant_settings.intelligent_exciter.amount >= 0.0
                && distant_settings.intelligent_exciter.amount <= 0.6
        );
        assert!(
            intimate_settings.intelligent_exciter.amount >= 0.0
                && intimate_settings.intelligent_exciter.amount <= 0.6
        );
        assert!(
            distant_settings.intelligent_exciter.mix >= 0.1
                && distant_settings.intelligent_exciter.mix <= 0.7
        );
        assert!(
            intimate_settings.intelligent_exciter.mix >= 0.1
                && intimate_settings.intelligent_exciter.mix <= 0.7
        );
    }
}
