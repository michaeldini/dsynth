#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod audio;
pub mod dsp;
#[cfg(any(feature = "standalone", feature = "vst"))]
pub mod gui;
pub mod midi;
pub mod params;
pub mod preset;

#[cfg(feature = "vst")]
#[path = "plugin.rs"]
mod plugin;
