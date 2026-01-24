// === Core DSP Modules ===

// Synthesis Components - Oscillators, waveforms, wavetables
pub mod synthesis;

// Modulation Components - Envelopes, LFOs, envelope followers
pub mod modulation;

// Filter Components - Biquad filters and utilities
pub mod filters;

// Analysis Components - Pitch detection, formant analysis, signal classification
pub mod analysis;

// Signal Analyzer - Unified signal analysis for intelligent audio processing
pub mod signal_analyzer;

// Effects - Audio processing effects organized by category
pub mod effects;

// === Re-exports for backwards compatibility ===

// Synthesis
pub use synthesis::{Downsampler, Oscillator, Wavetable, WavetableLibrary};

// Modulation
pub use modulation::{Envelope, EnvelopeFollower, EnvelopeMode, LFO};

// Filters
pub use filters::BiquadFilter;

// Analysis
pub use analysis::{
    FormantDetector, PitchDetectionResult, PitchDetector, PitchQuantizer, RootNote, ScaleType,
    SibilanceDetector, SignalType, SpectralCentroid, TransientDetector, VowelEstimate, ZcrDetector,
    PITCH_BUFFER_SIZE,
};

// Signal Analyzer
pub use signal_analyzer::{SignalAnalysis, SignalAnalyzer};
