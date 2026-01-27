// Analysis Components - Pitch detection, formant analysis, and signal classification

pub mod formant_detector;
pub mod pitch_detector;
pub mod pitch_quantizer;
pub mod sibilance_detector;
pub mod spectral_centroid;
pub mod transient_detector;
pub mod zcr_detector;

pub use formant_detector::{FormantDetector, VowelEstimate};
pub use pitch_detector::{PitchDetectionResult, PitchDetector, PITCH_BUFFER_SIZE};
pub use pitch_quantizer::{PitchQuantizer, RootNote, ScaleType};
pub use sibilance_detector::SibilanceDetector;
pub use spectral_centroid::SpectralCentroid;
pub use transient_detector::TransientDetector;
pub use zcr_detector::{SignalType, ZcrDetector};
