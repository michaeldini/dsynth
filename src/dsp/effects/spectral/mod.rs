// Spectral Effects - Frequency-domain processing and EQ

pub mod pitch_shifter;
pub mod exciter;
pub mod parametric_eq;
pub mod crossover;

pub use pitch_shifter::PitchShifter;
pub use exciter::Exciter;
pub use parametric_eq::{ParametricEQ, EQBand};
pub use crossover::LR2Crossover;

// Aliases for convenience
pub use parametric_eq::ParametricEQ as ParametricEq;
pub use parametric_eq::EQBand as EqBand;
pub use crossover::LR2Crossover as Crossover;
