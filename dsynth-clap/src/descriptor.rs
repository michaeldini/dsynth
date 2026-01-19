//! Plugin descriptor and metadata

use crate::{NotePortConfig, PortConfig};

/// Plugin descriptor containing metadata and configuration
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
    pub audio_ports: PortConfig,
    pub note_ports: NotePortConfig,
}

impl PluginDescriptor {
    /// Create a new instrument plugin (synthesizer)
    pub fn instrument(name: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            vendor: "DSynth".to_string(),
            url: "https://github.com/yourusername/dsynth".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            features: vec!["instrument".to_string()],
            audio_ports: PortConfig::Instrument,
            note_ports: NotePortConfig::Input,
        }
    }

    /// Create a new audio effect plugin
    pub fn effect(name: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            vendor: "DSynth".to_string(),
            url: "https://github.com/yourusername/dsynth".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            features: vec!["audio-effect".to_string()],
            audio_ports: PortConfig::Effect,
            note_ports: NotePortConfig::None,
        }
    }

    /// Set the plugin version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the plugin description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add CLAP features
    pub fn with_features(mut self, features: &[&str]) -> Self {
        self.features.extend(features.iter().map(|s| s.to_string()));
        self
    }

    /// Set audio port configuration
    pub fn audio_ports(mut self, config: PortConfig) -> Self {
        self.audio_ports = config;
        self
    }

    /// Set MIDI note port configuration
    pub fn note_ports(mut self, config: NotePortConfig) -> Self {
        self.note_ports = config;
        self
    }

    /// Set the vendor name
    pub fn vendor(mut self, vendor: impl Into<String>) -> Self {
        self.vendor = vendor.into();
        self
    }

    /// Set the project URL
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }
}
