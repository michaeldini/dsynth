// Thin wrapper so `crate::plugin` stays stable while the implementation
// lives in the extracted module files under src/plugin/.

#[path = "plugin/mod.rs"]
mod extracted;

#[allow(unused_imports)]
pub use extracted::{DSynthParams, DSynthPlugin};
