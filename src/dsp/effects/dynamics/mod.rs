// Dynamics Effects - Compression, limiting, gating, and level control

pub mod compressor;
pub mod multiband_compressor;
pub mod lookahead_limiter;
pub mod noise_gate;
pub mod de_esser;
pub mod clipper;

pub use compressor::Compressor;
pub use multiband_compressor::MultibandCompressor;
pub use lookahead_limiter::LookAheadLimiter;
pub use noise_gate::NoiseGate;
pub use de_esser::DeEsser;
pub use clipper::Clipper;

// Alias for convenience
pub use lookahead_limiter::LookAheadLimiter as LookaheadLimiter;
