// Existing effects
pub mod chorus;
pub mod delay;
pub mod distortion;
pub mod multiband_distortion;
pub mod reverb;
pub mod stereo_widener;

// New effects
pub mod auto_pan;
pub mod bitcrusher;
pub mod clipper;
pub mod comb_filter;
pub mod compressor;
pub mod crossover;
pub mod de_esser;
pub mod exciter;
pub mod flanger;
pub mod multiband_compressor;
pub mod noise_gate;
pub mod parametric_eq;
pub mod phaser;
pub mod ring_modulator;
pub mod tremolo;
pub mod waveshaper;

// Existing effect exports
pub use chorus::Chorus;
pub use delay::StereoDelay;
pub use distortion::Distortion;
pub use multiband_distortion::MultibandDistortion;
pub use reverb::Reverb;
pub use stereo_widener::StereoWidener;

// New effect exports
pub use auto_pan::AutoPan;
pub use bitcrusher::Bitcrusher;
pub use clipper::Clipper;
pub use comb_filter::CombFilter;
pub use compressor::Compressor;
pub use crossover::LR2Crossover;
pub use de_esser::DeEsser;
pub use exciter::Exciter;
pub use flanger::Flanger;
pub use multiband_compressor::MultibandCompressor;
pub use noise_gate::NoiseGate;
pub use parametric_eq::ParametricEQ;
pub use phaser::Phaser;
pub use ring_modulator::RingModulator;
pub use tremolo::Tremolo;
pub use waveshaper::Waveshaper;
