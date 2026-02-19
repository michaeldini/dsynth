// Modulation Components - Envelopes, LFOs, envelope followers, and parameter mapping

pub mod envelope;
pub mod envelope_follower;
pub mod lfo;
pub mod parameter_mapper;
pub mod processor_settings;

pub use envelope::Envelope;
pub use envelope_follower::{EnvelopeFollower, EnvelopeMode};
pub use lfo::LFO;
pub use parameter_mapper::ParameterMapper;
pub use processor_settings::*;
