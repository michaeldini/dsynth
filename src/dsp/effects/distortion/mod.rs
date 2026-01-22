// Distortion Effects - Waveshaping, saturation, and harmonic generation

pub mod distortion;
pub mod waveshaper;
pub mod multiband_distortion;
pub mod bitcrusher;

pub use distortion::{Distortion, DistortionType};
pub use waveshaper::Waveshaper;
pub use multiband_distortion::MultibandDistortion;
pub use bitcrusher::Bitcrusher;
