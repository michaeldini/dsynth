// === Audio Effects - Organized by Category ===

// Dynamics Effects - Compression, limiting, gating, level control
pub mod dynamics;

// Distortion Effects - Waveshaping, saturation, harmonic generation
pub mod distortion;

// Saturator - Simple saturation/harmonic enhancement
pub mod saturator;

// Modulation Effects - Chorus, flanger, phaser, tremolo
pub mod modulation;

// Time-Based Effects - Delay, reverb, acoustic space simulation
pub mod time_based;

// Delay Effects - Smart delay with signal-aware processing
pub mod delay;

// Spectral Effects - Frequency-domain processing and EQ
pub mod spectral;

// Stereo Effects - Stereo field manipulation and imaging
pub mod stereo;

// Vocal Effects - Voice-specific processing and enhancement
pub mod vocal;

// === Re-exports for backwards compatibility ===

// Dynamics
pub use dynamics::{
    Clipper, Compressor, DeEsser, LookAheadLimiter, LookaheadLimiter, MultibandCompressor,
    NoiseGate,
};

// Distortion
pub use distortion::{Bitcrusher, Distortion, DistortionType, MultibandDistortion, Waveshaper};

// Modulation
pub use modulation::{AutoPan, Chorus, Flanger, Phaser, RingModulator, Tremolo};

// Time-Based
pub use time_based::{CombFilter, Delay, Reverb, StereoDelay};

// Delay
pub use delay::SmartDelay;

// Saturator
pub use saturator::Saturator;

// Spectral
pub use spectral::{
    Crossover, EQBand, EqBand, Exciter, LR2Crossover, ParametricEQ, ParametricEq, PitchShifter,
};

// Stereo
pub use stereo::StereoWidener;

// Vocal
pub use vocal::{VocalChoir, VocalDoubler};
