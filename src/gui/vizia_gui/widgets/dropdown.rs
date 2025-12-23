use vizia::prelude::*;

/// A dropdown parameter widget for enum parameters
#[derive(Lens)]
pub struct Dropdown {
    /// Current selected index
    current_index: usize,

    /// Parameter ID for event emission
    param_id: u32,

    /// Number of options
    num_options: usize,

    /// Display strings for each option
    options: Vec<String>,
    
    /// Current option display text
    current_text: String,
}

impl Dropdown {
    pub fn new(
        cx: &mut Context,
        param_id: u32,
        options: Vec<String>,
        initial_index: usize,
    ) -> Handle<'_, Self> {
        let num_options = options.len();
        let current_index = initial_index.min(num_options.saturating_sub(1));
        let current_text = if current_index < options.len() {
            options[current_index].clone()
        } else {
            "Unknown".to_string()
        };
        
        // Debug: Log dropdown creation
        let debug_msg = format!("DEBUG: Creating Dropdown widget for param_id: 0x{:08X}, num_options: {}\n", param_id, num_options);
        let _ = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/dsynth_debug.log").and_then(|mut f| std::io::Write::write_all(&mut f, debug_msg.as_bytes()));

        Self {
            current_index,
            param_id,
            num_options,
            options,
            current_text,
        }
        .build(cx, |cx| {
            // Display the current selection text
            Label::new(cx, Dropdown::current_text)
            .font_size(11.0)
            .color(Color::rgb(200, 200, 210))
            .text_align(TextAlign::Center);
        })
        .width(Pixels(80.0))
        .height(Pixels(20.0))
        .background_color(Color::rgb(60, 60, 65))
        .border_width(Pixels(1.0))
        .border_color(Color::rgb(100, 100, 110))
    }
}

impl View for Dropdown {
    fn element(&self) -> Option<&'static str> {
        Some("dropdown")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                // Debug: Log click event
                let debug_msg = format!("DEBUG: Dropdown clicked! param_id: 0x{:08X}, current_index: {}\n", self.param_id, self.current_index);
                let _ = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/dsynth_debug.log").and_then(|mut f| std::io::Write::write_all(&mut f, debug_msg.as_bytes()));

                // Cycle to next option
                self.current_index = (self.current_index + 1) % self.num_options;
                self.current_text = if self.current_index < self.options.len() {
                    self.options[self.current_index].clone()
                } else {
                    "Unknown".to_string()
                };
                
                // Convert index to normalized value (0.0 to 1.0)
                let normalized = if self.num_options > 1 {
                    self.current_index as f32 / (self.num_options - 1) as f32
                } else {
                    0.0
                };

                // Emit parameter change event
                cx.emit(crate::gui::vizia_gui::GuiMessage::ParamChanged(
                    self.param_id,
                    normalized,
                ));

                meta.consume();
            }
            _ => {}
        });
    }
}