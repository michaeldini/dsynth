//! Basic type check - validates the dsynth-clap API compiles

#[cfg(test)]
mod tests {
    use dsynth_clap::*;

    struct DummyPlugin;
    struct DummyProcessor;
    struct DummyParams;

    impl ClapPlugin for DummyPlugin {
        type Processor = DummyProcessor;
        type Params = DummyParams;

        fn descriptor() -> PluginDescriptor {
            PluginDescriptor {
                id: "test.plugin".to_string(),
                name: "Test".to_string(),
                vendor: "Test".to_string(),
                url: String::new(),
                version: "1.0.0".to_string(),
                description: String::new(),
                features: vec![],
                audio_ports: PortConfig::Instrument,
                note_ports: NotePortConfig::Input,
            }
        }

        fn clap_descriptor() -> &'static clap_sys::plugin::clap_plugin_descriptor {
            unimplemented!()
        }

        fn new() -> Self {
            Self
        }

        fn create_processor(&mut self, _sample_rate: f32) -> Self::Processor {
            DummyProcessor
        }
    }

    impl ClapProcessor for DummyProcessor {
        fn process(&mut self, _audio: &mut AudioBuffers, _events: &Events) -> ProcessStatus {
            ProcessStatus::Continue
        }

        fn activate(&mut self, _sample_rate: f32) {}
    }

    impl PluginParams for DummyParams {
        fn param_count() -> u32 {
            0
        }

        fn param_descriptor(_index: u32) -> Option<ParamDescriptor> {
            None
        }

        fn param_descriptor_by_id(_id: ParamId) -> Option<ParamDescriptor> {
            None
        }

        fn get_param(_id: ParamId) -> Option<f32> {
            None
        }

        fn set_param(_id: ParamId, _value: f32) {}

        fn save_state() -> PluginState {
            PluginState::default()
        }

        fn load_state(_state: &PluginState) {}
    }

    #[test]
    fn test_plugin_creation() {
        let _plugin = DummyPlugin::new();
    }

    #[test]
    fn test_descriptor() {
        let desc = DummyPlugin::descriptor();
        assert_eq!(desc.id, "test.plugin");
    }
}
