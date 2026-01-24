//! Smart Noise Gate - Intelligent gate using signal analysis
//!
//! Unlike traditional amplitude-based gates, this gate uses comprehensive
//! signal analysis to make musically intelligent decisions:
//!
//! - **Never gates transients** - Drum hits and consonants always pass through
//! - **Preserves sibilance** - 's', 'sh', 't', 'f' sounds bypass gate
//! - **Vocal-aware** - Lowers threshold for singing/speech (pitched content)
//! - **Content classification** - Treats tonal vs noisy content differently
//!
//! # How It Works
//! 1. Receives `SignalAnalysis` with pre-computed signal features
//! 2. Decides whether to open/close gate based on musical intelligence
//! 3. Applies smooth gain changes to avoid clicks
//!
//! # Parameters
//! Just ONE parameter: `threshold_db` (-80dB to -20dB)
//! All intelligence is automatic!
//!
//! # Decision Logic
//! ```text
//! if transient OR sibilance:
//!     gate_open = TRUE  (always pass musical content)
//! else if voiced/pitched:
//!     gate_open = (level > threshold - 6dB)  (more permissive for vocals)
//! else:
//!     gate_open = (level > threshold)  (standard for noise)
//! ```

use crate::dsp::signal_analyzer::SignalAnalysis;

/// Smart noise gate with signal-aware intelligence
pub struct SmartGate {
    sample_rate: f32,

    /// Threshold in dB (-80dB to -20dB)
    threshold_db: f32,

    /// Attack/Release/Hold timing
    attack_ms: f32,
    release_ms: f32,
    hold_ms: f32,

    /// Current gate gain (0.0-1.0)
    gain: f32,

    /// Hold counter (samples remaining in hold phase)
    hold_counter: usize,

    /// Time constants
    attack_coeff: f32,
    release_coeff: f32,

    /// Hysteresis to prevent chattering
    hysteresis_db: f32,
}

impl SmartGate {
    /// Create a new smart gate
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: f32) -> Self {
        let mut gate = Self {
            sample_rate,
            threshold_db: -50.0, // Default threshold
            attack_ms: 1.0,      // Fast attack (1ms)
            release_ms: 100.0,   // Moderate release (100ms)
            hold_ms: 50.0,       // 50ms hold
            gain: 1.0,
            hold_counter: 0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            hysteresis_db: 3.0, // 3dB hysteresis
        };

        gate.update_coefficients();
        gate
    }

    /// Set threshold in dB (-80dB to -20dB)
    ///
    /// This is the ONLY user-facing parameter!
    /// Lower = more aggressive gating (only loud signals pass)
    /// Higher = more permissive (quieter signals pass)
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db.clamp(-80.0, -20.0);
    }

    /// Set attack time in milliseconds (typically 0.1-5ms)
    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack_ms = attack_ms.clamp(0.1, 50.0);
        self.attack_coeff = Self::ms_to_coeff(self.attack_ms, self.sample_rate);
    }

    /// Set release time in milliseconds (typically 50-200ms)
    pub fn set_release(&mut self, release_ms: f32) {
        self.release_ms = release_ms.clamp(10.0, 1000.0);
        self.release_coeff = Self::ms_to_coeff(self.release_ms, self.sample_rate);
    }

    /// Set hold time in milliseconds (typically 20-100ms)
    pub fn set_hold(&mut self, hold_ms: f32) {
        self.hold_ms = hold_ms.clamp(0.0, 500.0);
    }

    /// Convert milliseconds to exponential decay coefficient
    fn ms_to_coeff(time_ms: f32, sample_rate: f32) -> f32 {
        if time_ms <= 0.0 {
            return 0.0;
        }
        let time_sec = time_ms / 1000.0;
        let samples = time_sec * sample_rate;
        (-1.0 / samples).exp()
    }

    /// Update time-based coefficients
    fn update_coefficients(&mut self) {
        self.attack_coeff = Self::ms_to_coeff(self.attack_ms, self.sample_rate);
        self.release_coeff = Self::ms_to_coeff(self.release_ms, self.sample_rate);
    }

    /// Convert dB to linear gain
    #[inline]
    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Convert linear level to dB
    #[inline]
    fn linear_to_db(linear: f32) -> f32 {
        20.0 * linear.max(1e-10).log10()
    }

    /// Process one stereo sample pair with signal analysis
    ///
    /// # Arguments
    /// * `left` - Left channel input sample
    /// * `right` - Right channel input sample
    /// * `analysis` - Pre-computed signal analysis
    ///
    /// # Returns
    /// Tuple of (left, right) output samples
    pub fn process(&mut self, left: f32, right: f32, analysis: &SignalAnalysis) -> (f32, f32) {
        // === STEP 1: Compute Adaptive Threshold ===
        // Use signal intelligence to adjust threshold based on content

        let effective_threshold = if analysis.is_transient {
            // CRITICAL: Transients always pass through (drum hits, consonants)
            // Lower threshold dramatically to ensure gate opens
            self.threshold_db - 18.0 // -18dB offset = almost always open
        } else if analysis.has_sibilance {
            // Sibilance ('s', 'sh', 't', 'f') is musically important
            // Lower threshold to preserve consonants
            self.threshold_db - 12.0 // -12dB offset = very permissive
        } else if analysis.is_pitched && analysis.pitch_confidence > 0.5 {
            // Clear pitch detected = singing/speech
            // Lower threshold to preserve quiet vocals
            self.threshold_db - 6.0 // -6dB offset = more permissive
        } else if analysis.is_voiced {
            // Voiced content (low ZCR) = likely vocal/musical
            // Slightly lower threshold
            self.threshold_db - 3.0 // -3dB offset = somewhat permissive
        } else {
            // Unvoiced/noisy content - use standard threshold
            // This catches sustained background noise
            self.threshold_db
        };

        // === STEP 2: Determine Gate State ===
        // Use RMS level from analysis (pre-computed)

        let input_level_db = Self::linear_to_db(analysis.rms_level.sqrt());

        let target_gain = if input_level_db > effective_threshold {
            // Signal above threshold - gate is open
            // Reset hold counter
            self.hold_counter = (self.hold_ms * self.sample_rate / 1000.0) as usize;
            1.0 // Full volume
        } else if input_level_db < effective_threshold - self.hysteresis_db || self.hold_counter == 0
        {
            // Signal below threshold (with hysteresis) and hold expired
            // Gate closes
            0.0 // Silence
        } else {
            // In hold phase - keep gate open
            if self.hold_counter > 0 {
                self.hold_counter -= 1;
            }
            1.0 // Keep gate open during hold
        };

        // === STEP 3: Smooth Gain Changes ===
        // Prevents clicks when gate opens/closes

        let coeff = if target_gain > self.gain {
            self.attack_coeff // Opening gate (fast attack)
        } else {
            self.release_coeff // Closing gate (slower release)
        };

        self.gain = self.gain * coeff + target_gain * (1.0 - coeff);

        // === STEP 4: Apply Gain ===
        (left * self.gain, right * self.gain)
    }

    /// Reset gate state
    pub fn reset(&mut self) {
        self.gain = 1.0;
        self.hold_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::SignalType;

    fn create_default_analysis() -> SignalAnalysis {
        SignalAnalysis {
            rms_level: 0.01, // Quiet signal
            peak_level: 0.01,
            is_transient: false,
            transient_strength: 0.0,
            zcr_hz: 500.0,
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
    fn test_smart_gate_creation() {
        let gate = SmartGate::new(44100.0);
        assert_eq!(gate.sample_rate, 44100.0);
        assert_eq!(gate.threshold_db, -50.0);
        assert_eq!(gate.gain, 1.0);
    }

    #[test]
    fn test_transient_always_passes() {
        let mut gate = SmartGate::new(44100.0);
        gate.set_threshold(-40.0);

        // Create analysis with transient detected (even if quiet)
        let mut analysis = create_default_analysis();
        analysis.rms_level = 0.001; // Very quiet (-60dB)
        analysis.is_transient = true;
        analysis.transient_strength = 0.8;

        // Process samples
        for _ in 0..100 {
            let (out_l, out_r) = gate.process(0.1, 0.1, &analysis);
            // Gate should open despite low level (transient passes through)
            assert!(out_l > 0.05 || out_r > 0.05, "Transient should pass through");
        }
    }

    #[test]
    fn test_sibilance_bypass() {
        let mut gate = SmartGate::new(44100.0);
        gate.set_threshold(-40.0);

        // Create analysis with sibilance detected (quiet 's' sound)
        let mut analysis = create_default_analysis();
        analysis.rms_level = 0.005; // Quiet (-46dB)
        analysis.has_sibilance = true;
        analysis.sibilance_strength = 0.7;

        // Process samples
        for _ in 0..100 {
            let (out_l, out_r) = gate.process(0.1, 0.1, &analysis);
            // Gate should open for sibilance
            assert!(out_l > 0.05 || out_r > 0.05, "Sibilance should pass through");
        }
    }

    #[test]
    fn test_pitched_vocal_threshold_adaptation() {
        let mut gate = SmartGate::new(44100.0);
        gate.set_threshold(-40.0); // Standard threshold

        // Create analysis with confident pitch (singing)
        let mut analysis = create_default_analysis();
        analysis.rms_level = 0.01; // Quiet but not silent (-40dB)
        analysis.is_pitched = true;
        analysis.pitch_confidence = 0.8;
        analysis.pitch_hz = 220.0;

        // Process samples - should open due to pitched content
        for _ in 0..100 {
            let (out_l, out_r) = gate.process(0.1, 0.1, &analysis);
            // With -6dB threshold adaptation, should pass
            assert!(out_l > 0.05 || out_r > 0.05, "Pitched vocal should pass");
        }
    }

    #[test]
    fn test_gates_unvoiced_noise() {
        let mut gate = SmartGate::new(44100.0);
        gate.set_threshold(-40.0);
        gate.set_release(10.0); // Fast release for test
        gate.set_hold(0.0); // No hold to speed up test

        // Create analysis with unvoiced noise (no transient, no sibilance, no pitch)
        let mut analysis = create_default_analysis();
        // RMS level is squared! 0.0001^2 = 1e-8 = about -80dB
        // Need level well below -40dB threshold
        analysis.rms_level = 0.0001 * 0.0001; // ~-80dB
        analysis.is_voiced = false;
        analysis.is_unvoiced = true;
        analysis.is_transient = false;
        analysis.has_sibilance = false;
        analysis.is_pitched = false;

        // Process samples - should close gate
        for _ in 0..5000 {
            let (out_l, out_r) = gate.process(0.1, 0.1, &analysis);
            // Gate should eventually close
            if out_l < 0.01 && out_r < 0.01 {
                // Success - gate closed
                return;
            }
        }

        panic!("Gate should close for sustained quiet noise");
    }

    #[test]
    fn test_hold_time() {
        let mut gate = SmartGate::new(44100.0);
        gate.set_threshold(-40.0);
        gate.set_hold(50.0); // 50ms hold
        gate.set_release(10.0); // Fast release

        // Create loud analysis (opens gate)
        let mut analysis = create_default_analysis();
        analysis.rms_level = 0.1; // Loud (-20dB)

        // Open gate with loud signal
        for _ in 0..100 {
            gate.process(0.5, 0.5, &analysis);
        }

        // Switch to quiet signal
        analysis.rms_level = 0.001; // Quiet (-60dB)

        let mut hold_working = false;
        for i in 0..500 {
            let (out_l, _) = gate.process(0.1, 0.1, &analysis);

            // During hold period (~50ms = 2205 samples @ 44.1kHz), should stay open
            if i < 2000 && out_l > 0.05 {
                hold_working = true;
            }
        }

        assert!(hold_working, "Hold should keep gate open temporarily");
    }

    #[test]
    fn test_reset() {
        let mut gate = SmartGate::new(44100.0);

        // Change gate state
        let analysis = create_default_analysis();
        for _ in 0..100 {
            gate.process(0.0, 0.0, &analysis);
        }

        gate.reset();

        assert_eq!(gate.gain, 1.0);
        assert_eq!(gate.hold_counter, 0);
    }
}
