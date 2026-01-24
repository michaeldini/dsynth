// Spectral Effects - Frequency-domain processing and EQ

pub mod crossover;
pub mod de_esser;
pub mod exciter;
pub mod intelligent_exciter;
pub mod parametric_eq;
pub mod pitch_shifter;

pub use crossover::LR2Crossover;
pub use de_esser::DeEsser;
pub use exciter::Exciter;
pub use intelligent_exciter::IntelligentExciter;
pub use parametric_eq::{EQBand, ParametricEQ};
pub use pitch_shifter::PitchShifter;

// Aliases for convenience
pub use crossover::LR2Crossover as Crossover;
pub use parametric_eq::EQBand as EqBand;
pub use parametric_eq::ParametricEQ as ParametricEq;
