//! Plugin state management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin state for save/load
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginState {
    /// Parameter values (param_id -> value)
    pub params: HashMap<u32, f32>,

    /// Custom binary data (e.g., wavetables, samples)
    #[serde(skip)]
    pub binary_data: Vec<u8>,

    /// Version for forward/backward compatibility
    pub version: u32,
}

impl PluginState {
    /// Create a new state
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Set parameter value
    pub fn set_param(&mut self, id: u32, value: f32) {
        self.params.insert(id, value);
    }

    /// Get parameter value
    pub fn get_param(&self, id: u32) -> Option<f32> {
        self.params.get(&id).copied()
    }
}
