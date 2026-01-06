/// Kick Drum Synthesizer - Standalone Application Entry Point
/// Simplified monophonic kick synth with GUI, audio output, and MIDI input
use dsynth::audio::kick_engine::{KickEngine, MidiEvent};
use dsynth::gui::kick_gui::run_kick_gui;
use dsynth::params_kick::KickParams;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use std::thread;

// Audio output using cpal
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};

// MIDI input using midir
use midir::{Ignore, MidiInput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting DSynth Kick...");
    
    // Create shared parameters
    let params = Arc::new(Mutex::new(KickParams::default()));
    
    // Create audio engine
    let sample_rate = 44100.0;
    let engine = Arc::new(Mutex::new(KickEngine::new(sample_rate, Arc::clone(&params))));
    
    // Get MIDI note queue for MIDI thread
    let note_queue = engine.lock().get_note_queue();
    
    // Start MIDI input thread
    start_midi_thread(note_queue.clone())?;
    
    // Start audio output thread
    start_audio_thread(Arc::clone(&engine))?;
    
    // Run GUI (blocking)
    println!("Starting GUI...");
    run_kick_gui_wrapper(params)?;
    
    Ok(())
}

// Wrapper to convert Mutex to RwLock for GUI compatibility
fn run_kick_gui_wrapper(params: Arc<Mutex<KickParams>>) -> Result<(), Box<dyn std::error::Error>> {
    // For simplicity, just use the params - GUI is placeholder anyway
    let params_rw = Arc::new(RwLock::new(params.lock().clone()));
    run_kick_gui(params_rw)
}

/// Start MIDI input thread
fn start_midi_thread(
    note_queue: Arc<Mutex<Vec<MidiEvent>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    thread::spawn(move || {
        let mut midi_in = match MidiInput::new("DSynth Kick MIDI Input") {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to create MIDI input: {}", e);
                return;
            }
        };
        
        midi_in.ignore(Ignore::None);
        
        // Get available MIDI ports
        let ports = midi_in.ports();
        if ports.is_empty() {
            println!("No MIDI input ports available");
            return;
        }
        
        // Connect to first available port
        let port = &ports[0];
        let port_name = midi_in.port_name(port).unwrap_or_else(|_| "Unknown".to_string());
        println!("Connecting to MIDI port: {}", port_name);
        
        let _connection = midi_in.connect(
            port,
            "dsynth-kick-midi",
            move |_timestamp, message, _| {
                handle_midi_message(message, &note_queue);
            },
            (),
        );
        
        if let Err(e) = _connection {
            eprintln!("Failed to connect to MIDI port: {}", e);
            return;
        }
        
        println!("MIDI input connected");
        
        // Keep thread alive
        loop {
            thread::sleep(std::time::Duration::from_millis(100));
        }
    });
    
    Ok(())
}

/// Handle incoming MIDI message
fn handle_midi_message(message: &[u8], note_queue: &Mutex<Vec<MidiEvent>>) {
    if message.len() < 2 {
        return;
    }
    
    let status = message[0];
    let note = message[1];
    let velocity = if message.len() >= 3 { message[2] } else { 0 };
    
    match status & 0xF0 {
        0x90 if velocity > 0 => {
            // Note On
            let vel_normalized = velocity as f32 / 127.0;
            let mut queue = note_queue.lock();
            queue.push(MidiEvent::NoteOn {
                note,
                velocity: vel_normalized,
            });
        }
        0x80 | 0x90 => {
            // Note Off (0x80) or Note On with velocity 0 (0x90)
            let mut queue = note_queue.lock();
            queue.push(MidiEvent::NoteOff { note });
        }
        _ => {}
    }
}

/// Start audio output thread
fn start_audio_thread(
    engine: Arc<Mutex<KickEngine>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get audio host
    let host = cpal::default_host();
    
    // Get default output device
    let device = host
        .default_output_device()
        .ok_or("No output device available")?;
    
    println!("Using audio device: {}", device.name()?);
    
    // Get default output config
    let config = device.default_output_config()?;
    println!("Audio config: {:?}", config);
    
    // Build output stream
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), engine)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), engine)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), engine)?,
        sample_format => {
            return Err(format!("Unsupported sample format: {:?}", sample_format).into())
        }
    };
    
    // Start the stream
    stream.play()?;
    
    println!("Audio output started");
    
    // Leak the stream to keep it alive for the lifetime of the program
    // This is necessary because cpal Stream is not Send
    std::mem::forget(stream);
    
    Ok(())
}

/// Build audio output stream
fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    engine: Arc<Mutex<KickEngine>>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: Sample + FromSample<f32> + SizedSample,
{
    let channels = config.channels as usize;
    
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut engine = engine.lock();
            
            // Process audio in frames (interleaved stereo)
            for frame in data.chunks_mut(channels) {
                let sample = engine.process_sample();
                
                // Duplicate mono to all channels
                for output in frame.iter_mut() {
                    *output = T::from_sample(sample);
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;
    
    Ok(stream)
}
