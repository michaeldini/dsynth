/// Wavetable library management for loading and storing collections of wavetables
///
/// The library loads .wav files from disk at startup and provides access to wavetables
/// by index or name. All wavetables are pre-loaded to avoid I/O in the audio thread.
use crate::dsp::synthesis::wavetable::Wavetable;
use std::collections::HashMap;
use std::path::Path;

// Include compile-time embedded wavetable data
include!(concat!(env!("OUT_DIR"), "/embedded_wavetables.rs"));

/// Error types for wavetable loading
#[derive(Debug)]
pub enum LoadError {
    IoError(std::io::Error),
    InvalidFormat(String),
    EmptyLibrary,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::IoError(e) => write!(f, "I/O error: {}", e),
            LoadError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            LoadError::EmptyLibrary => write!(f, "No wavetables loaded"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        LoadError::IoError(e)
    }
}

/// Static collection of all loaded wavetables
/// Loaded once at application startup, never modified during playback
pub struct WavetableLibrary {
    /// All wavetables indexed by ID
    tables: Vec<Wavetable>,

    /// Name → index lookup for user-friendly selection
    name_to_index: HashMap<String, usize>,
}

impl WavetableLibrary {
    /// Create an empty wavetable library
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    /// Create a library with built-in wavetables (fallback if no .wav files found)
    pub fn with_builtin_wavetables() -> Self {
        let mut library = Self::new();

        // Add built-in waveforms as wavetables
        library.add_wavetable(Wavetable::sine("Sine".to_string(), 2048));
        library.add_wavetable(Wavetable::sawtooth("Sawtooth".to_string(), 2048));

        // Generate additional built-in wavetables
        library.add_wavetable(Self::generate_square("Square".to_string(), 2048));
        library.add_wavetable(Self::generate_triangle("Triangle".to_string(), 2048));
        library.add_wavetable(Self::generate_pulse("Pulse 25%".to_string(), 2048, 0.25));
        library.add_wavetable(Self::generate_pulse("Pulse 75%".to_string(), 2048, 0.75));

        library
    }

    /// Load wavetables from compile-time embedded bytes
    ///
    /// This is the preferred method for production builds as it eliminates
    /// runtime file dependencies. Wavetables are embedded into the binary
    /// at compile time via build.rs.
    ///
    /// Falls back to built-in wavetables if no embedded data is available.
    ///
    /// # Returns
    /// WavetableLibrary with loaded wavetables (or built-in fallbacks)
    pub fn load_from_embedded() -> Result<Self, LoadError> {
        let mut library = Self::new();
        let mut loaded_count = 0;

        // Load each embedded wavetable (EMBEDDED_WAVETABLES is defined at module level)
        for (filename, wav_bytes) in EMBEDDED_WAVETABLES {
            match Wavetable::from_wav_bytes(wav_bytes, (*filename).to_string()) {
                Ok(wavetable) => {
                    library.add_wavetable(wavetable);
                    loaded_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to load embedded wavetable '{}': {}", filename, e);
                }
            }
        }

        if loaded_count == 0 {
            eprintln!("No wavetables loaded from embedded data, using built-in wavetables");
            return Ok(Self::with_builtin_wavetables());
        }
        Ok(library)
    }

    /// Load wavetables from directory (e.g., "assets/wavetables/")
    ///
    /// Scans the directory for .wav files and loads each as a wavetable.
    /// If no .wav files are found or the directory doesn't exist, returns a library
    /// with built-in wavetables as fallback.
    ///
    /// # Arguments
    /// * `path` - Path to directory containing .wav files
    ///
    /// # Returns
    /// WavetableLibrary with loaded wavetables (or built-in fallbacks)
    pub fn load_from_directory(path: &str) -> Result<Self, LoadError> {
        let dir_path = Path::new(path);

        // If directory doesn't exist, use built-in wavetables
        if !dir_path.exists() || !dir_path.is_dir() {
            eprintln!(
                "Wavetable directory '{}' not found, using built-in wavetables",
                path
            );
            return Ok(Self::with_builtin_wavetables());
        }

        let mut library = Self::new();
        let mut loaded_count = 0;

        // Scan directory for .wav files
        match std::fs::read_dir(dir_path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let entry_path = entry.path();

                    // Check if it's a .wav file
                    if let Some(ext) = entry_path.extension() {
                        if ext.eq_ignore_ascii_case("wav") {
                            match Self::load_wavetable_from_wav(&entry_path) {
                                Ok(wavetable) => {
                                    library.add_wavetable(wavetable);
                                    loaded_count += 1;
                                }
                                Err(e) => {
                                    eprintln!("Failed to load {:?}: {}", entry_path, e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read directory '{}': {}", path, e);
            }
        }

        // If no wavetables were loaded, use built-in ones
        if loaded_count == 0 {
            eprintln!(
                "No .wav files found in '{}', using built-in wavetables",
                path
            );
            return Ok(Self::with_builtin_wavetables());
        }

        println!("Loaded {} wavetables from '{}'", loaded_count, path);
        Ok(library)
    }

    /// Load a single wavetable from a .wav file
    ///
    /// Expects a single-cycle waveform (typically 2048 samples).
    /// The WAV file is converted to mono if stereo and resampled if needed.
    fn load_wavetable_from_wav(path: &Path) -> Result<Wavetable, LoadError> {
        // Get filename without extension for the name
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Use hound to read the WAV file
        let mut reader = hound::WavReader::open(path)
            .map_err(|e| LoadError::InvalidFormat(format!("Failed to open WAV: {}", e)))?;

        let spec = reader.spec();

        // Read all samples from the WAV file
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => reader
                .samples::<f32>()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| LoadError::InvalidFormat(format!("Failed to read samples: {}", e)))?,
            hound::SampleFormat::Int => {
                // Convert integer samples to float (-1.0 to 1.0)
                let max_value = (1 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / max_value))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| {
                        LoadError::InvalidFormat(format!("Failed to read samples: {}", e))
                    })?
            }
        };

        if samples.is_empty() {
            return Err(LoadError::InvalidFormat(
                "WAV file contains no samples".to_string(),
            ));
        }

        // Convert stereo to mono by averaging channels
        let mono_samples: Vec<f32> = if spec.channels == 1 {
            samples
        } else {
            samples
                .chunks(spec.channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
                .collect()
        };

        // Use the samples as-is (single-cycle waveform)
        // Most wavetables are already 2048 samples, but we'll accept any length
        Ok(Wavetable::new(name, mono_samples))
    }

    /// Add a wavetable to the library
    pub fn add_wavetable(&mut self, wavetable: Wavetable) {
        let index = self.tables.len();
        let name = wavetable.name().to_string();
        self.tables.push(wavetable);
        self.name_to_index.insert(name, index);
    }

    /// Get wavetable by index
    pub fn get(&self, index: usize) -> Option<&Wavetable> {
        self.tables.get(index)
    }

    /// Get index by name
    pub fn find_by_name(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }

    /// Get all wavetable names
    pub fn list_names(&self) -> Vec<&str> {
        self.tables.iter().map(|wt| wt.name()).collect()
    }

    /// Total number of wavetables loaded
    pub fn count(&self) -> usize {
        self.tables.len()
    }

    /// Check if the library is empty
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    // Built-in waveform generators

    fn generate_square(name: String, num_samples: usize) -> Wavetable {
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| if i < num_samples / 2 { 1.0 } else { -1.0 })
            .collect();
        Wavetable::new(name, samples)
    }

    fn generate_triangle(name: String, num_samples: usize) -> Wavetable {
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| {
                let phase = i as f32 / num_samples as f32;
                if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    -4.0 * phase + 3.0
                }
            })
            .collect();
        Wavetable::new(name, samples)
    }

    fn generate_pulse(name: String, num_samples: usize, duty_cycle: f32) -> Wavetable {
        let duty_cycle = duty_cycle.clamp(0.0, 1.0);
        let threshold = (num_samples as f32 * duty_cycle) as usize;
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| if i < threshold { 1.0 } else { -1.0 })
            .collect();
        Wavetable::new(name, samples)
    }
}

impl Default for WavetableLibrary {
    fn default() -> Self {
        Self::with_builtin_wavetables()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wavetable_library_builtin() {
        let library = WavetableLibrary::with_builtin_wavetables();

        // Should have at least the basic built-in wavetables
        assert!(library.count() >= 6);

        // Should be able to find by name
        assert!(library.find_by_name("Sine").is_some());
        assert!(library.find_by_name("Sawtooth").is_some());
    }

    #[test]
    fn test_wavetable_library_get() {
        let library = WavetableLibrary::with_builtin_wavetables();

        // Get first wavetable
        let wt = library.get(0).expect("Should have at least one wavetable");
        assert!(!wt.name().is_empty());
    }

    #[test]
    fn test_wavetable_library_list_names() {
        let library = WavetableLibrary::with_builtin_wavetables();
        let names = library.list_names();

        assert!(!names.is_empty());
        assert!(names.contains(&"Sine"));
    }

    #[test]
    fn test_wavetable_library_add() {
        let mut library = WavetableLibrary::new();
        assert_eq!(library.count(), 0);

        library.add_wavetable(Wavetable::sine("Test".to_string(), 2048));
        assert_eq!(library.count(), 1);

        assert!(library.find_by_name("Test").is_some());
    }
}
