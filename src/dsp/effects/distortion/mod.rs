// Distortion Effects - Waveshaping, saturation, and harmonic generation

pub mod bitcrusher;
pub mod distortion;
pub mod multiband_distortion;
pub mod waveshaper;

pub use bitcrusher::Bitcrusher;
pub use distortion::{Distortion, DistortionType};
pub use multiband_distortion::MultibandDistortion;
pub use waveshaper::Waveshaper;
