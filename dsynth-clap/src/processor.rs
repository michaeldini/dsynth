//! Audio processor trait

use clap_sys::events::{clap_input_events, clap_output_events};

/// Errors that can occur while wrapping CLAP audio buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBuffersError {
    /// The host passed a null `clap_process` pointer.
    NullProcess,
}

/// Stereo input (L,R) + stereo output (L,R) view.
pub type StereoIo<'a> = (&'a [f32], &'a [f32], &'a mut [f32], &'a mut [f32]);

/// Process status return codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Continue processing
    Continue,
    /// Continue processing even if output is silent
    ContinueIfNotQuiet,
    /// Processing generated tail (reverb, delay, etc.)
    Tail,
    /// No more processing needed, plugin can sleep
    Sleep,
}

/// Audio buffer wrapper for safe access
pub struct AudioBuffers {
    input_ports: Vec<InputPort>,
    output_ports: Vec<OutputPort>,
    frames_count: u32,
}

struct InputPort {
    channels: Vec<*const f32>,
}

struct OutputPort {
    channels: Vec<*mut f32>,
}

impl AudioBuffers {
    /// Create from CLAP process struct
    ///
    /// # Safety
    /// `process` must be a valid pointer to a CLAP `clap_process` provided by the host.
    pub unsafe fn from_clap_process(
        process: *const clap_sys::process::clap_process,
    ) -> Result<Self, AudioBuffersError> {
        if process.is_null() {
            return Err(AudioBuffersError::NullProcess);
        }

        let process = &*process;
        let frames_count = process.frames_count;

        // CRITICAL DEBUG: Write to a file to verify this is being called
        use std::fs::OpenOptions;
        use std::io::Write;
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/dsynth_voice_debug.log")
            .and_then(|mut f| {
                writeln!(
                    f,
                    "[AudioBuffers] inputs={}, outputs={}, frames={}",
                    process.audio_inputs_count, process.audio_outputs_count, frames_count
                )
            });

        // Extract input ports (const pointers)
        let mut input_ports = Vec::new();
        if !process.audio_inputs.is_null() {
            for i in 0..process.audio_inputs_count {
                let port = &*process.audio_inputs.add(i as usize);
                let mut channels = Vec::new();
                for ch in 0..port.channel_count {
                    // Input buffers are const in CLAP - don't cast to mut!
                    let ptr = *port.data32.add(ch as usize);
                    channels.push(ptr);
                }
                input_ports.push(InputPort { channels });
            }
        }

        // Extract output ports (mutable pointers)
        let mut output_ports = Vec::new();
        if !process.audio_outputs.is_null() {
            for i in 0..process.audio_outputs_count {
                let port = &*process.audio_outputs.add(i as usize);
                let mut channels = Vec::new();
                for ch in 0..port.channel_count {
                    // Output buffers: data32 is **const f32, but we can write to the data
                    let ptr = *port.data32.add(ch as usize) as *mut f32;
                    channels.push(ptr);
                }
                output_ports.push(OutputPort { channels });
            }
        }

        Ok(Self {
            input_ports,
            output_ports,
            frames_count,
        })
    }

    /// Get number of input ports
    pub fn input_port_count(&self) -> usize {
        self.input_ports.len()
    }

    /// Get number of output ports
    pub fn output_port_count(&self) -> usize {
        self.output_ports.len()
    }

    /// Get number of frames in this buffer
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    /// Get input channel as slice
    ///
    /// # Safety
    /// The host must provide valid audio buffer pointers for the duration of this call.
    pub unsafe fn input_channel(&self, port: usize, channel: usize) -> Option<&[f32]> {
        let port = self.input_ports.get(port)?;
        let ptr = *port.channels.get(channel)?;
        if ptr.is_null() {
            return None;
        }
        Some(std::slice::from_raw_parts(ptr, self.frames_count as usize))
    }

    /// Get output channel as mutable slice
    ///
    /// # Safety
    /// The host must provide valid audio buffer pointers for the duration of this call.
    pub unsafe fn output_channel_mut(&mut self, port: usize, channel: usize) -> Option<&mut [f32]> {
        let port = self.output_ports.get(port)?;
        let ptr = *port.channels.get(channel)?;
        if ptr.is_null() {
            return None;
        }
        Some(std::slice::from_raw_parts_mut(
            ptr,
            self.frames_count as usize,
        ))
    }

    /// Get input stereo pair (L, R) as slices
    ///
    /// # Safety
    /// The host must provide at least 2 input channels for `port`.
    pub unsafe fn input_stereo(&self, port: usize) -> Option<(&[f32], &[f32])> {
        Some((self.input_channel(port, 0)?, self.input_channel(port, 1)?))
    }

    /// Get output stereo pair (L, R) as mutable slices
    ///
    /// # Safety
    /// The host must provide at least 2 output channels for `port`.
    pub unsafe fn output_stereo_mut(&mut self, port: usize) -> Option<(&mut [f32], &mut [f32])> {
        let port_data = self.output_ports.get(port)?;
        let left_ptr = *port_data.channels.first()?;
        let right_ptr = *port_data.channels.get(1)?;

        if left_ptr.is_null() || right_ptr.is_null() {
            return None;
        }

        let frames = self.frames_count as usize;
        let left = std::slice::from_raw_parts_mut(left_ptr, frames);
        let right = std::slice::from_raw_parts_mut(right_ptr, frames);

        Some((left, right))
    }

    /// Get stereo input (L,R) and stereo output (L,R) in one call.
    ///
    /// This avoids borrow-checker conflicts in processors that need both at once.
    ///
    /// Note: hosts may provide in-place buffers. Processors must be prepared for
    /// input/output aliasing (process sample-by-sample if needed).
    ///
    /// # Safety
    /// The host must provide valid input/output pointers for the requested ports and at least 2
    /// channels per port.
    pub unsafe fn io_stereo_mut(
        &mut self,
        input_port: usize,
        output_port: usize,
    ) -> Option<StereoIo<'_>> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let in_port = self.input_ports.get(input_port);
        let out_port = self.output_ports.get(output_port);

        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/dsynth_voice_debug.log")
            .and_then(|mut f| {
                writeln!(
                    f,
                    "[io_stereo_mut] in_port={}, out_port={}",
                    in_port.is_some(),
                    out_port.is_some()
                )
            });

        let in_port = in_port?;
        let out_port = out_port?;

        let in_l_ptr = *in_port.channels.first()?;
        let in_r_ptr = *in_port.channels.get(1)?;
        let out_l_ptr = *out_port.channels.first()?;
        let out_r_ptr = *out_port.channels.get(1)?;

        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/dsynth_voice_debug.log")
            .and_then(|mut f| {
                writeln!(
                    f,
                    "[io_stereo_mut] ptrs: in_l={:?}, in_r={:?}, out_l={:?}, out_r={:?}",
                    in_l_ptr, in_r_ptr, out_l_ptr, out_r_ptr
                )
            });

        if in_l_ptr.is_null() || in_r_ptr.is_null() || out_l_ptr.is_null() || out_r_ptr.is_null() {
            let _ = OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/dsynth_voice_debug.log")
                .and_then(|mut f| writeln!(f, "[io_stereo_mut] NULL POINTER DETECTED!"));
            return None;
        }

        let frames = self.frames_count as usize;
        let in_l = std::slice::from_raw_parts(in_l_ptr, frames);
        let in_r = std::slice::from_raw_parts(in_r_ptr, frames);
        let out_l = std::slice::from_raw_parts_mut(out_l_ptr, frames);
        let out_r = std::slice::from_raw_parts_mut(out_r_ptr, frames);

        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/dsynth_voice_debug.log")
            .and_then(|mut f| writeln!(f, "[io_stereo_mut] SUCCESS! frames={}", frames));

        Some((in_l, in_r, out_l, out_r))
    }
}

/// CLAP events wrapper
pub struct Events {
    input_events: *const clap_input_events,
    output_events: *const clap_output_events,
}

impl Events {
    /// Create from CLAP process struct
    ///
    /// # Safety
    /// `process` must be a valid pointer to a CLAP `clap_process` provided by the host.
    pub unsafe fn from_clap_process(process: *const clap_sys::process::clap_process) -> Self {
        let (input_events, output_events) = if !process.is_null() {
            let p = &*process;
            (p.in_events, p.out_events)
        } else {
            (std::ptr::null(), std::ptr::null())
        };

        Self {
            input_events,
            output_events,
        }
    }

    /// Get number of input events
    ///
    /// # Safety
    /// The host must provide a valid input events pointer for the duration of this call.
    pub unsafe fn input_event_count(&self) -> u32 {
        if self.input_events.is_null() {
            return 0;
        }
        let events = &*self.input_events;
        if let Some(size_fn) = events.size {
            size_fn(events)
        } else {
            0
        }
    }

    /// Get input event at index
    ///
    /// # Safety
    /// The host must provide a valid input events pointer for the duration of this call.
    pub unsafe fn input_event(&self, index: u32) -> Option<&clap_sys::events::clap_event_header> {
        if self.input_events.is_null() {
            return None;
        }
        let events = &*self.input_events;
        if let Some(get_fn) = events.get {
            let event = get_fn(events, index);
            if event.is_null() {
                None
            } else {
                Some(&*event)
            }
        } else {
            None
        }
    }

    /// Try to push an output event
    ///
    /// # Safety
    /// The host must provide a valid output events pointer for the duration of this call.
    pub unsafe fn try_push_output(&self, event: &clap_sys::events::clap_event_header) -> bool {
        if self.output_events.is_null() {
            return false;
        }
        let events = &*self.output_events;
        if let Some(try_push_fn) = events.try_push {
            try_push_fn(events, event)
        } else {
            false
        }
    }

    /// Get raw input events pointer (for advanced use)
    pub fn input_events_ptr(&self) -> *const clap_input_events {
        self.input_events
    }

    /// Get raw output events pointer (for advanced use)
    pub fn output_events_ptr(&self) -> *const clap_output_events {
        self.output_events
    }
}

/// Audio processor trait - implement this for real-time DSP
pub trait ClapProcessor: Send + 'static {
    /// Process audio block
    ///
    /// # Arguments
    /// * `audio` - Input/output audio buffers
    /// * `events` - CLAP events (MIDI, parameter automation)
    ///
    /// # Safety
    /// This is called from the real-time audio thread:
    /// - No allocations
    /// - No blocking operations
    /// - No lock contention
    fn process(&mut self, audio: &mut AudioBuffers, events: &Events) -> ProcessStatus;

    /// Activate processing at a given sample rate
    fn activate(&mut self, sample_rate: f32);

    /// Deactivate processing
    fn deactivate(&mut self) {
        // Default implementation does nothing
    }

    /// Reset processor state
    fn reset(&mut self) {
        // Default implementation does nothing
    }

    /// Get processing latency in samples
    fn latency(&self) -> u32 {
        0
    }

    /// Get tail length in samples (for effects like reverb)
    fn tail(&self) -> u32 {
        0
    }
}
