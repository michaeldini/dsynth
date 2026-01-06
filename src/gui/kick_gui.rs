/// Simplified GUI for Kick Drum Synthesizer
/// Focused interface for kick drum parameters only
use crate::params_kick::KickParams;
use parking_lot::RwLock;
use std::sync::Arc;
use vizia::prelude::*;

// Import Application and WindowModifiers directly from vizia_winit
use vizia_winit::application::Application;
use vizia_winit::window_modifiers::WindowModifiers;

const WINDOW_WIDTH: u32 = 600;
const WINDOW_HEIGHT: u32 = 400;

/// GUI state for kick synthesizer
#[derive(Lens)]
pub struct KickGuiState {
    params: Arc<RwLock<KickParams>>,
}

impl KickGuiState {
    pub fn new(params: Arc<RwLock<KickParams>>) -> Self {
        Self { params }
    }
}

impl Model for KickGuiState {}

/// Run standalone VIZIA GUI for kick synth
pub fn run_kick_gui(
    params: Arc<RwLock<KickParams>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = Application::new(move |cx| {
        // Initialize GUI state
        KickGuiState::new(params.clone()).build(cx);
        
        // Build UI
        build_kick_ui(cx);
    })
    .title("DSynth Kick")
    .inner_size((WINDOW_WIDTH, WINDOW_HEIGHT))
    .run();
    
    Ok(())
}

/// Build the kick synth UI layout (simplified placeholder)
fn build_kick_ui(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Title
        Label::new(cx, "DSynth Kick")
            .height(Pixels(40.0))
            .font_size(24.0)
            .color(Color::white());
        
        // Info text
        Label::new(cx, "Kick Drum Synthesizer")
            .font_size(14.0)
            .color(Color::rgba(170, 170, 170, 255));
        
        Label::new(cx, "Play MIDI notes to trigger kicks")
            .font_size(12.0)
            .color(Color::rgba(120, 120, 120, 255));
        
        // Preset buttons (placeholder)
        HStack::new(cx, |cx| {
            Button::new(cx, |cx| Label::new(cx, "808"))
                .width(Pixels(100.0))
                .height(Pixels(40.0));
            
            Button::new(cx, |cx| Label::new(cx, "Techno"))
                .width(Pixels(100.0))
                .height(Pixels(40.0));
            
            Button::new(cx, |cx| Label::new(cx, "Sub"))
                .width(Pixels(100.0))
                .height(Pixels(40.0));
        });
    })
    .background_color(Color::rgb(20, 20, 25));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gui_state_creation() {
        let params = Arc::new(RwLock::new(KickParams::default()));
        let _state = KickGuiState::new(params);
        // Just verify it can be created
    }
}
