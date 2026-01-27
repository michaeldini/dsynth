// Modulation Effects - Chorus, flanger, phaser, tremolo, and related effects

pub mod auto_pan;
pub mod chorus;
pub mod flanger;
pub mod phaser;
pub mod ring_modulator;
pub mod tremolo;

pub use auto_pan::AutoPan;
pub use chorus::Chorus;
pub use flanger::Flanger;
pub use phaser::Phaser;
pub use ring_modulator::RingModulator;
pub use tremolo::Tremolo;
