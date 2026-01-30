// Dynamics Effects - Compression, limiting, gating, and level control

pub mod adaptive_compression_limiter;
pub mod adaptive_compressor;
pub mod clipper;
pub mod compressor;
pub mod de_esser;
pub mod lookahead_limiter;
pub mod multiband_compressor;
pub mod noise_gate;
pub mod smart_gate;
pub mod transient_shaper;

pub use adaptive_compression_limiter::AdaptiveCompressionLimiter;
pub use adaptive_compressor::AdaptiveCompressor;
pub use clipper::Clipper;
pub use compressor::Compressor;
pub use de_esser::DeEsser;
pub use lookahead_limiter::LookAheadLimiter;
pub use multiband_compressor::MultibandCompressor;
pub use noise_gate::NoiseGate;
pub use smart_gate::SmartGate;
pub use transient_shaper::TransientShaper;

// Alias for convenience
pub use lookahead_limiter::LookAheadLimiter as LookaheadLimiter;
