/// Plugin state serialization (CLAP state + user presets)
///
/// This module handles saving/loading parameter state for the CLAP state extension
/// (binary) and for user presets (JSON).
///
/// Note: any mention of "migration" in this module refers to *state schema/version*
/// migration (e.g. preset format v0 → v1), not the historical CLAP integration migration.
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

use crate::params::SynthParams;

/// Current plugin state format version
const STATE_VERSION: u32 = 1;

/// Maximum state size (10MB safety limit)
const MAX_STATE_SIZE: usize = 10 * 1024 * 1024;

/// Plugin state serialization error
#[derive(Debug)]
pub enum StateError {
    SerializationError(String),
    DeserializationError(String),
    VersionMismatch { expected: u32, got: u32 },
    StateTooLarge { size: usize, max: usize },
    InvalidData(String),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            StateError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
            StateError::VersionMismatch { expected, got } => {
                write!(f, "Version mismatch: expected {}, got {}", expected, got)
            }
            StateError::StateTooLarge { size, max } => {
                write!(f, "State too large: {} bytes (max {})", size, max)
            }
            StateError::InvalidData(e) => write!(f, "Invalid data: {}", e),
        }
    }
}

impl Error for StateError {}

/// Binary plugin state format
///
/// This is the format used by CLAP's state save/load interface.
/// Includes version for forward/backward compatibility.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginState {
    /// State format version
    version: u32,
    /// Actual parameter values
    params: SynthParams,
    /// Optional preset name
    preset_name: Option<String>,
}

impl PluginState {
    /// Create a new plugin state from current parameters
    pub fn from_params(params: SynthParams, preset_name: Option<String>) -> Self {
        Self {
            version: STATE_VERSION,
            params,
            preset_name,
        }
    }

    /// Get the contained parameters
    pub fn params(&self) -> &SynthParams {
        &self.params
    }

    /// Get mutable reference to parameters
    pub fn params_mut(&mut self) -> &mut SynthParams {
        &mut self.params
    }

    /// Get the preset name if available
    pub fn preset_name(&self) -> Option<&str> {
        self.preset_name.as_deref()
    }

    /// Serialize state to binary format (for CLAP)
    pub fn to_bytes(&self) -> Result<Vec<u8>, StateError> {
        bincode::serialize(self)
            .map_err(|e| StateError::SerializationError(e.to_string()))
            .and_then(|bytes| {
                if bytes.len() > MAX_STATE_SIZE {
                    Err(StateError::StateTooLarge {
                        size: bytes.len(),
                        max: MAX_STATE_SIZE,
                    })
                } else {
                    Ok(bytes)
                }
            })
    }

    /// Deserialize state from binary format (from CLAP)
    pub fn from_bytes(data: &[u8]) -> Result<Self, StateError> {
        if data.len() > MAX_STATE_SIZE {
            return Err(StateError::StateTooLarge {
                size: data.len(),
                max: MAX_STATE_SIZE,
            });
        }

        let state: PluginState = bincode::deserialize(data)
            .map_err(|e| StateError::DeserializationError(e.to_string()))?;

        // Check version for forward compatibility
        if state.version != STATE_VERSION {
            return Err(StateError::VersionMismatch {
                expected: STATE_VERSION,
                got: state.version,
            });
        }

        Ok(state)
    }

    /// Serialize state to JSON (for user presets)
    pub fn to_json(&self) -> Result<String, StateError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| StateError::SerializationError(e.to_string()))
    }

    /// Deserialize state from JSON (from user presets)
    pub fn from_json(json: &str) -> Result<Self, StateError> {
        let state: PluginState = serde_json::from_str(json)
            .map_err(|e| StateError::DeserializationError(e.to_string()))?;

        // Check version for backward compatibility.
        // If you want to accept older/newer formats, use `PresetMigration::migrate_if_needed()`.
        if state.version != STATE_VERSION {
            // For now, reject version mismatches; schema migration is handled externally.
            return Err(StateError::VersionMismatch {
                expected: STATE_VERSION,
                got: state.version,
            });
        }

        Ok(state)
    }
}

/// Helper for preset schema/version migration (if needed)
pub struct PresetMigration;

impl PresetMigration {
    /// Migrate a preset JSON string to the current `PluginState` schema.
    ///
    /// Currently only provides a best-effort path for v0 → v1.
    pub fn migrate_if_needed(json: &str) -> Result<PluginState, StateError> {
        // Try to parse as current version first
        if let Ok(state) = PluginState::from_json(json) {
            return Ok(state);
        }

        // If that fails, try to parse as generic JSON and infer version
        let v: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| StateError::DeserializationError(e.to_string()))?;

        let version = v.get("version").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        match version {
            1 => {
                // Already current version, just deserialize
                PluginState::from_json(json)
            }
            0 | 2..=u32::MAX => {
                // Either no version or future version - try to coerce into v1.
                Self::upgrade_to_v1(v)
            }
        }
    }

    /// Upgrade a v0 preset to v1 format
    fn upgrade_to_v1(value: serde_json::Value) -> Result<PluginState, StateError> {
        // Extract parameters from v0 format
        // This is a placeholder - actual migration would depend on v0 schema
        let params_value = value
            .get("params")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let params: SynthParams = serde_json::from_value(params_value).map_err(|e| {
            StateError::DeserializationError(format!("Failed to migrate preset: {}", e))
        })?;

        let preset_name = value
            .get("preset_name")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(PluginState {
            version: STATE_VERSION,
            params,
            preset_name,
        })
    }
}

/// Preset manager for loading/saving user presets
pub struct PresetManager;

impl PresetManager {
    /// Load a preset from file
    pub fn load_preset(path: &str) -> Result<PluginState, StateError> {
        std::fs::read_to_string(path)
            .map_err(|e| {
                StateError::DeserializationError(format!("Failed to read preset file: {}", e))
            })
            .and_then(|contents| PresetMigration::migrate_if_needed(&contents))
    }

    /// Save a preset to file
    pub fn save_preset(state: &PluginState, path: &str) -> Result<(), StateError> {
        let json = state.to_json()?;
        std::fs::write(path, json).map_err(|e| {
            StateError::SerializationError(format!("Failed to write preset file: {}", e))
        })
    }

    /// Get default preset
    pub fn default_preset() -> PluginState {
        PluginState::from_params(SynthParams::default(), Some("Default".to_string()))
    }

    /// List all presets in a directory
    pub fn list_presets(dir: &str) -> Result<Vec<String>, StateError> {
        Ok(std::fs::read_dir(dir)
            .map_err(|e| {
                StateError::InvalidData(format!("Failed to read preset directory: {}", e))
            })?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.extension().map(|ext| ext == "json").unwrap_or(false) {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_state_creation() {
        let params = SynthParams::default();
        let state = PluginState::from_params(params, Some("Test".to_string()));

        assert_eq!(state.version, STATE_VERSION);
        assert_eq!(state.preset_name(), Some("Test"));
        assert_eq!(state.params().master_gain, params.master_gain);
    }

    #[test]
    fn test_binary_serialization() {
        let state =
            PluginState::from_params(SynthParams::default(), Some("Binary Test".to_string()));

        // Serialize
        let bytes = state.to_bytes().expect("Serialization failed");
        assert!(!bytes.is_empty());
        assert!(bytes.len() < MAX_STATE_SIZE);

        // Deserialize
        let restored = PluginState::from_bytes(&bytes).expect("Deserialization failed");
        assert_eq!(restored.version, state.version);
        assert_eq!(restored.preset_name(), state.preset_name());
        assert_eq!(restored.params().master_gain, state.params().master_gain);
    }

    #[test]
    fn test_json_serialization() {
        let state = PluginState::from_params(SynthParams::default(), Some("JSON Test".to_string()));

        // Serialize to JSON
        let json = state.to_json().expect("JSON serialization failed");
        assert!(json.contains("version"));
        assert!(json.contains("JSON Test"));

        // Deserialize from JSON
        let restored = PluginState::from_json(&json).expect("JSON deserialization failed");
        assert_eq!(restored.version, state.version);
        assert_eq!(restored.preset_name(), state.preset_name());
    }

    #[test]
    fn test_version_mismatch() {
        let mut state = PluginState::from_params(SynthParams::default(), None);
        state.version = 999; // Set to wrong version

        let bytes = state.to_bytes().expect("Serialization should work");
        let result = PluginState::from_bytes(&bytes);

        assert!(result.is_err());
        if let Err(StateError::VersionMismatch { expected, got }) = result {
            assert_eq!(expected, STATE_VERSION);
            assert_eq!(got, 999);
        } else {
            panic!("Expected VersionMismatch error");
        }
    }

    #[test]
    fn test_preset_default() {
        let preset = PresetManager::default_preset();
        assert_eq!(preset.preset_name(), Some("Default"));
        assert_eq!(preset.version, STATE_VERSION);
    }

    #[test]
    fn test_parameter_serialization_roundtrip() {
        let mut state =
            PluginState::from_params(SynthParams::default(), Some("Roundtrip".to_string()));

        // Modify some parameters
        state.params_mut().master_gain = 0.75;
        state.params_mut().oscillators[0].pitch = 12.0;
        state.params_mut().filters[0].cutoff = 5000.0;

        // Serialize to JSON and back
        let json = state.to_json().expect("JSON serialization failed");
        let restored = PluginState::from_json(&json).expect("JSON deserialization failed");

        assert!((restored.params().master_gain - 0.75).abs() < 0.0001);
        assert!((restored.params().oscillators[0].pitch - 12.0).abs() < 0.0001);
        assert!((restored.params().filters[0].cutoff - 5000.0).abs() < 0.0001);
    }
}
