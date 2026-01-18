use crate::params::SynthParams;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub params: SynthParams,
}

impl Preset {
    pub fn new(name: String, params: SynthParams) -> Self {
        Self { name, params }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        let preset = serde_json::from_str(&json)?;
        Ok(preset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_preset_round_trip() {
        let params = SynthParams::default();
        let preset = Preset::new("Test Preset".to_string(), params);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_owned();

        preset.save(&path).unwrap();
        let loaded = Preset::load(&path).unwrap();

        assert_eq!(preset.name, loaded.name);
        assert_eq!(preset.params, loaded.params);
    }
}
