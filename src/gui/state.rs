// GUI state for VIZIA - shared between plugin and standalone

#[cfg(feature = "standalone")]
use crate::audio::output::EngineEvent;
use crate::gui::messages::UiTab;
use crate::gui::GuiMessage;
use crate::params::SynthParams;
use crate::plugin::gui_param_change::GuiParamChange;
#[cfg(feature = "standalone")]
use crossbeam_channel::Sender;
use parking_lot::{Mutex, RwLock};
use std::collections::HashSet;
use std::sync::Arc;
use triple_buffer::Input;
use vizia::prelude::*;

/// GUI state that holds synth parameters and provides VIZIA lens access
///
/// This intentionally stays simple: we store the shared Arc and apply updates directly.
/// If performance ever becomes an issue, we can add reactive caching.
#[derive(Clone, Lens)]
pub struct GuiState {
    /// Shared synthesizer parameters (Arc for cross-thread access)
    pub synth_params: Arc<RwLock<SynthParams>>,

    /// GUI -> audio thread lock-free param change producer (for single param changes)
    pub gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,

    /// GUI -> audio thread producer for full SynthParams (for standalone)
    /// This is what the audio engine actually reads from
    #[lens(ignore)]
    pub params_producer: Option<Arc<Mutex<Input<SynthParams>>>>,

    /// UI feedback string (e.g. last changed param/value)
    pub last_param_text: String,

    /// Event sender for standalone features (MIDI, panic) - None for plugin
    #[lens(ignore)]
    #[cfg(feature = "standalone")]
    pub event_sender: Option<Sender<EngineEvent>>,

    /// Track pressed keys to prevent key-repeat note retriggering (standalone only)
    #[lens(ignore)]
    pub pressed_keys: HashSet<u8>,

    /// Cached waveform values for conditional rendering (0=Sine, 7=Additive, 8=Wavetable)
    pub osc1_waveform: i32,
    pub osc2_waveform: i32,
    pub osc3_waveform: i32,

    /// Active UI tab
    pub active_tab: UiTab,
}

impl GuiState {
    /// Create new GUI state for plugin (no event sender)
    pub fn new(
        synth_params: Arc<RwLock<SynthParams>>,
        gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,
    ) -> Self {
        Self {
            synth_params,
            gui_param_producer,
            params_producer: None,
            last_param_text: String::new(),
            #[cfg(feature = "standalone")]
            event_sender: None,
            pressed_keys: HashSet::new(),
            osc1_waveform: 0, // Default: Sine
            osc2_waveform: 0,
            osc3_waveform: 0,
            active_tab: UiTab::Oscillator,
        }
    }

    /// Create new GUI state for standalone (with event sender for MIDI/panic)
    #[cfg(feature = "standalone")]
    pub fn new_standalone(
        synth_params: Arc<RwLock<SynthParams>>,
        gui_param_producer: Arc<Mutex<Input<GuiParamChange>>>,
        params_producer: Arc<Mutex<Input<SynthParams>>>,
        event_sender: Sender<EngineEvent>,
    ) -> Self {
        Self {
            synth_params,
            gui_param_producer,
            params_producer: Some(params_producer),
            last_param_text: String::new(),
            event_sender: Some(event_sender),
            pressed_keys: HashSet::new(),
            osc1_waveform: 0, // Default: Sine
            osc2_waveform: 0,
            osc3_waveform: 0,
            active_tab: UiTab::Oscillator,
        }
    }

    /// Update parameter value and write to synth_params
    pub fn update_param(&mut self, param_id: u32, normalized_value: f32) {
        // Write to synth_params
        let mut params = self.synth_params.write();
        crate::plugin::param_update::param_apply::apply_param(
            &mut params,
            param_id,
            normalized_value,
        );

        // Sync waveform fields for conditional rendering
        use crate::plugin::param_descriptor::*;
        match param_id {
            PARAM_OSC1_WAVEFORM => {
                self.osc1_waveform = params.oscillators[0].waveform as i32;
            }
            PARAM_OSC2_WAVEFORM => {
                self.osc2_waveform = params.oscillators[1].waveform as i32;
            }
            PARAM_OSC3_WAVEFORM => {
                self.osc3_waveform = params.oscillators[2].waveform as i32;
            }
            _ => {}
        }

        // For standalone: Write full SynthParams to the engine's triple-buffer
        if let Some(ref producer) = self.params_producer {
            let mut p = producer.lock();
            p.write(*params);
        }

        // Also send single param change (for plugin use via ClapProcessor)
        let mut producer = self.gui_param_producer.lock();
        producer.write(GuiParamChange {
            param_id,
            normalized: normalized_value.clamp(0.0, 1.0),
        });
    }
}

impl Model for GuiState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|msg, meta| match msg {
            GuiMessage::SetActiveTab(tab) => {
                self.active_tab = *tab;
                cx.needs_redraw();
                meta.consume();
            }
            GuiMessage::ParamChanged(param_id, normalized) => {
                self.update_param(*param_id, *normalized);

                // Provide immediate visual feedback in the UI with formatted values
                let registry = crate::plugin::param_registry::get_registry();
                if let Some(desc) = registry.get(*param_id) {
                    let formatted_value = desc.format_value(*normalized);
                    self.last_param_text = format!("{}: {}", desc.name, formatted_value);
                } else {
                    self.last_param_text = format!(
                        "Param 0x{:08X}: {:.0}%",
                        *param_id,
                        (normalized.clamp(0.0, 1.0) * 100.0)
                    );
                }

                cx.needs_redraw();
                meta.consume();
            }
            GuiMessage::Randomize => {
                self.apply_randomized_params();
                self.last_param_text = "🎲 Randomized!".to_string();

                // Emit SyncKnobValue for all parameters to update knob visuals
                self.emit_all_param_syncs(cx);

                cx.needs_redraw();
                meta.consume();
            }
            _ => {}
        });
    }
}

impl GuiState {
    /// Apply randomized parameters to all synth parameters
    fn apply_randomized_params(&mut self) {
        use crate::randomize::randomize_synth_params;

        // Generate randomized parameters
        let mut rng = rand::thread_rng();
        let randomized = randomize_synth_params(&mut rng);

        // Write randomized params to shared state and audio thread
        let mut params = self.synth_params.write();
        // Copy randomized values to the shared params
        *params = randomized;

        // For standalone: Write full SynthParams to the engine's triple-buffer
        if let Some(ref producer) = self.params_producer {
            let mut p = producer.lock();
            p.write(*params);
        }

        // For plugin: Send a dummy param change to trigger ClapProcessor to re-read
        // (The plugin's ClapProcessor reads GuiParamChange and applies to its params)
        // Use a random normalized value to avoid duplicate detection
        let mut producer = self.gui_param_producer.lock();
        // Send a "full sync" signal - param_id 0xFFFFFFFF is a special marker
        // Use random normalized to ensure each sync is unique (avoids duplicate detection)
        producer.write(crate::plugin::gui_param_change::GuiParamChange {
            param_id: 0xFFFFFFFF,
            normalized: rand::random::<f32>(),
        });
    }

    /// Emit SyncKnobValue messages for all parameters to update UI visuals
    fn emit_all_param_syncs(&self, cx: &mut EventContext) {
        use crate::plugin::param_registry::get_registry;
        use crate::plugin::param_update::param_get;
        use vizia::prelude::Propagation;

        let registry = get_registry();

        let params = self.synth_params.read();
        for param_id in registry.iter_ids() {
            // Get denormalized value from params
            let denorm = param_get::get_param(&params, param_id);

            // Normalize it
            let normalized = if let Some(desc) = registry.get(param_id) {
                desc.normalize_value(denorm)
            } else {
                0.0
            };

            // Emit sync message with Subtree propagation to reach all knobs
            cx.emit_custom(
                Event::new(GuiMessage::SyncKnobValue(param_id, normalized))
                    .propagate(Propagation::Subtree),
            );
        }
    }
}
