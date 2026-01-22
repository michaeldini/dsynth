// Modulation Effects - Chorus, flanger, phaser, tremolo, and related effects

pub mod chorus;
pub mod flanger;
pub mod phaser;
pub mod tremolo;
pub mod auto_pan;
pub mod ring_modulator;

pub use chorus::Chorus;
pub use flanger::Flanger;
pub use phaser::Phaser;
pub use tremolo::Tremolo;
pub use auto_pan::AutoPan;
pub use ring_modulator::RingModulator;
