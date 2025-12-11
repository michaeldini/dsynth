use crate::audio::engine::SynthEngine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioOutput {
    _stream: Stream,
    sample_rate: f32,
}

impl AudioOutput {
    /// Create and start audio output
    ///
    /// # Arguments
    /// * `engine` - Synth engine wrapped in Arc<Mutex<>>
    ///
    /// # Returns
    /// Result containing AudioOutput or error message
    pub fn new(engine: Arc<Mutex<SynthEngine>>) -> Result<Self, String> {
        let host = cpal::default_host();
        
        let device = host
            .default_output_device()
            .ok_or_else(|| "No output device available".to_string())?;
        
        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;
        
        let sample_rate = config.sample_rate().0 as f32;
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), engine)?,
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), engine)?,
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), engine)?,
            _ => return Err("Unsupported sample format".to_string()),
        };
        
        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        
        Ok(Self {
            _stream: stream,
            sample_rate,
        })
    }

    /// Build audio stream for specific sample type
    fn build_stream<T>(
        device: &Device,
        config: &StreamConfig,
        engine: Arc<Mutex<SynthEngine>>,
    ) -> Result<Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = config.channels as usize;
        
        let err_fn = |err| eprintln!("Audio stream error: {}", err);
        
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    let mut engine_lock = engine.lock().unwrap();
                    
                    for frame in data.chunks_mut(channels) {
                        let sample = engine_lock.process();
                        let sample_t = cpal::Sample::from_sample(sample);
                        
                        // Write to all channels (mono to stereo/multi-channel)
                        for channel_sample in frame.iter_mut() {
                            *channel_sample = sample_t;
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("Failed to build output stream: {}", e))?;
        
        Ok(stream)
    }

    /// Get the sample rate of the audio output
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

/// List available audio output devices
pub fn list_output_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;
    
    let mut device_names = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            device_names.push(name);
        }
    }
    
    Ok(device_names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::engine::create_parameter_buffer;

    #[test]
    fn test_list_output_devices() {
        // Should not panic
        let result = list_output_devices();
        
        // On most systems, there should be at least one output device
        if let Ok(devices) = result {
            println!("Found {} audio output devices", devices.len());
        }
    }

    #[test]
    fn test_audio_output_creation() {
        let (_producer, consumer) = create_parameter_buffer();
        let engine = SynthEngine::new(44100.0, consumer);
        let engine_arc = Arc::new(Mutex::new(engine));
        
        // Try to create audio output
        // This may fail in test environments without audio devices
        let result = AudioOutput::new(engine_arc);
        
        match result {
            Ok(output) => {
                assert!(output.sample_rate() > 0.0);
                println!("Audio output created successfully at {} Hz", output.sample_rate());
            }
            Err(e) => {
                println!("Audio output creation failed (expected in CI): {}", e);
            }
        }
    }
}

