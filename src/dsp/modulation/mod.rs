// Modulation Components - Envelopes, LFOs, and envelope followers

pub mod envelope;
pub mod envelope_follower;
pub mod lfo;

pub use envelope::Envelope;
pub use envelope_follower::{EnvelopeFollower, EnvelopeMode};
pub use lfo::LFO;
