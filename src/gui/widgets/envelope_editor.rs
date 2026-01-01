use vizia::prelude::*;

/// Visual ADSR envelope editor with draggable control points
///
/// Displays the envelope curve using VIZIA's canvas drawing API
#[derive(Lens)]
pub struct EnvelopeEditor {
    /// Attack time (normalized 0.0 to 1.0)
    attack_normalized: f32,
    /// Decay time (normalized 0.0 to 1.0)
    decay_normalized: f32,
    /// Sustain level (normalized 0.0 to 1.0)
    sustain_normalized: f32,
    /// Release time (normalized 0.0 to 1.0)
    release_normalized: f32,

    /// Parameter IDs for each ADSR parameter
    attack_param_id: u32,
    decay_param_id: u32,
    sustain_param_id: u32,
    release_param_id: u32,

    /// Currently dragged handle (if any)
    #[lens(ignore)]
    drag_state: Option<DraggedHandle>,

    /// Hovered handle for visual feedback
    #[lens(ignore)]
    hovered_handle: Option<DraggedHandle>,

    /// Mouse position at start of drag
    #[lens(ignore)]
    drag_start_pos: (f32, f32),

    /// Parameter values at start of drag
    #[lens(ignore)]
    drag_start_values: (f32, f32, f32, f32), // (attack, decay, sustain, release)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DraggedHandle {
    Attack,
    Decay,
    Release,
}

impl EnvelopeEditor {
    pub fn new(
        cx: &mut Context,
        attack_value: f32,
        decay_value: f32,
        sustain_value: f32,
        release_value: f32,
        attack_param_id: u32,
        decay_param_id: u32,
        sustain_param_id: u32,
        release_param_id: u32,
    ) -> Handle<'_, Self> {
        Self {
            attack_normalized: attack_value.clamp(0.0, 1.0),
            decay_normalized: decay_value.clamp(0.0, 1.0),
            sustain_normalized: sustain_value.clamp(0.0, 1.0),
            release_normalized: release_value.clamp(0.0, 1.0),
            attack_param_id,
            decay_param_id,
            sustain_param_id,
            release_param_id,
            drag_state: None,
            hovered_handle: None,
            drag_start_pos: (0.0, 0.0),
            drag_start_values: (0.0, 0.0, 0.0, 0.0),
        }
        .build(cx, |_cx| {})
        .width(Pixels(300.0))
        .height(Pixels(150.0))
    }

    /// Calculate the envelope curve points in normalized coordinates (0.0 to 1.0)
    /// Uses THREE EQUAL SECTIONS for attack (0-1/3), decay (1/3-2/3), and release (2/3-1)
    #[allow(dead_code)]
    fn calculate_envelope_points(&self) -> Vec<(f32, f32)> {
        let mut points = Vec::new();

        // Divide the view into three equal sections
        // Section 1: Attack (0.0 to 0.333)
        // Section 2: Decay (0.333 to 0.666)
        // Section 3: Release (0.666 to 1.0)

        let section_width = 1.0 / 3.0;

        // Attack spans from 0 to section_width (0.333)
        let attack_end_x = self.attack_normalized * section_width;

        // Decay spans from section_width to 2*section_width (0.333 to 0.666)
        // let decay_start_x = section_width;
        let decay_end_x = section_width + (self.decay_normalized * section_width);

        // Release spans from 2*section_width to 1.0 (0.666 to 1.0)
        let release_start_x = 2.0 * section_width;
        let release_end_x = release_start_x + (self.release_normalized * section_width);

        // Build the ADSR curve
        // Start at origin (0, 0)
        points.push((0.0, 0.0));

        // Attack: ramp up to peak (1.0)
        points.push((attack_end_x, 1.0));

        // Decay: goes directly from attack peak to sustain level
        points.push((decay_end_x, self.sustain_normalized));

        // Sustain: hold at sustain level until release section starts
        if decay_end_x < release_start_x {
            points.push((release_start_x, self.sustain_normalized));
        }

        // Release: decay to zero
        points.push((release_end_x, 0.0));

        points
    }

    /// Check if mouse position is over a control handle
    fn get_handle_at_position(
        &self,
        mouse_pos: (f32, f32),
        bounds: BoundingBox,
    ) -> Option<DraggedHandle> {
        let points = self.calculate_envelope_points();
        let pixel_points: Vec<(f32, f32)> = points
            .iter()
            .map(|(nx, ny)| self.normalized_to_pixels(*nx, *ny, bounds))
            .collect();

        let hit_radius = 10.0; // Larger hit area for easier interaction

        // Check attack peak (point 1)
        if pixel_points.len() > 1 {
            let (px, py) = pixel_points[1];
            let dx = mouse_pos.0 - px;
            let dy = mouse_pos.1 - py;
            if (dx * dx + dy * dy).sqrt() <= hit_radius {
                return Some(DraggedHandle::Attack);
            }
        }

        // Check decay endpoint - find the point at decay_end_x with sustain level
        let section_width = 1.0 / 3.0;
        let decay_end_x = section_width + (self.decay_normalized * section_width);
        for (i, &(px, py)) in pixel_points.iter().enumerate() {
            let (nx, ny) = points[i];
            // Match the decay endpoint: at decay_end_x with sustain level
            if (nx - decay_end_x).abs() < 0.001 && (ny - self.sustain_normalized).abs() < 0.001 {
                let dx = mouse_pos.0 - px;
                let dy = mouse_pos.1 - py;
                if (dx * dx + dy * dy).sqrt() <= hit_radius {
                    return Some(DraggedHandle::Decay);
                }
            }
        }

        // Check release endpoint (last point)
        if let Some(&(px, py)) = pixel_points.last() {
            let dx = mouse_pos.0 - px;
            let dy = mouse_pos.1 - py;
            if (dx * dx + dy * dy).sqrt() <= hit_radius {
                return Some(DraggedHandle::Release);
            }
        }

        None
    }

    /// Handle dragging a control point
    fn handle_drag(
        &mut self,
        cx: &mut EventContext,
        mouse_pos: (f32, f32),
        bounds: BoundingBox,
        handle: DraggedHandle,
    ) {
        let padding = 10.0;
        let width = bounds.width() - 2.0 * padding;
        let height = bounds.height() - 2.0 * padding;

        // Convert mouse position to normalized coordinates
        let nx = ((mouse_pos.0 - bounds.x - padding) / width).clamp(0.0, 1.0);
        let ny = 1.0 - ((mouse_pos.1 - bounds.y - padding) / height).clamp(0.0, 1.0);

        match handle {
            DraggedHandle::Attack => {
                // Attack is constrained to first third (0.0 to 0.333)
                // Map position within first third to normalized parameter (0.0 to 1.0)
                let section_width = 1.0 / 3.0;
                let relative_pos = (nx / section_width).clamp(0.0, 1.0);
                self.attack_normalized = relative_pos;
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.attack_param_id,
                    self.attack_normalized,
                ));
            }

            DraggedHandle::Decay => {
                // Sustain level (vertical) - direct mapping
                self.sustain_normalized = ny;
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.sustain_param_id,
                    self.sustain_normalized,
                ));

                // Decay is constrained to second third (0.333 to 0.666)
                // Map position within second third to normalized parameter (0.0 to 1.0)
                let section_width = 1.0 / 3.0;
                let section_start = section_width;
                let relative_pos = ((nx - section_start) / section_width).clamp(0.0, 1.0);
                self.decay_normalized = relative_pos;
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.decay_param_id,
                    self.decay_normalized,
                ));
            }

            DraggedHandle::Release => {
                // Release is constrained to third third (0.666 to 1.0)
                // Map position within third third to normalized parameter (0.0 to 1.0)
                let section_width = 1.0 / 3.0;
                let section_start = 2.0 * section_width;
                let relative_pos = ((nx - section_start) / section_width).clamp(0.0, 1.0);
                self.release_normalized = relative_pos;
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.release_param_id,
                    self.release_normalized,
                ));
            }
        }
    }
}

impl View for EnvelopeEditor {
    fn element(&self) -> Option<&'static str> {
        Some("envelope-editor")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // Handle parameter sync messages from the audio thread
        event.map(
            |gui_msg: &crate::gui::GuiMessage, _meta| match gui_msg {
                crate::gui::GuiMessage::SyncKnobValue(param_id, normalized) => {
                    let value = normalized.clamp(0.0, 1.0);
                    if *param_id == self.attack_param_id {
                        self.attack_normalized = value;
                    } else if *param_id == self.decay_param_id {
                        self.decay_normalized = value;
                    } else if *param_id == self.sustain_param_id {
                        self.sustain_normalized = value;
                    } else if *param_id == self.release_param_id {
                        self.release_normalized = value;
                    }
                }
                _ => {}
            },
        );

        // Handle mouse events for interaction
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseMove(x, y) => {
                let bounds = cx.cache.get_bounds(cx.current());
                let mouse_pos = (*x, *y);

                // Check if we're dragging
                if let Some(handle) = self.drag_state {
                    self.handle_drag(cx, mouse_pos, bounds, handle);
                    meta.consume();
                } else {
                    // Check for hover
                    let old_hover = self.hovered_handle;
                    self.hovered_handle = self.get_handle_at_position(mouse_pos, bounds);

                    // Request redraw if hover state changed
                    if old_hover != self.hovered_handle {
                        cx.needs_redraw();
                    }
                }
            }

            WindowEvent::MouseDown(MouseButton::Left) => {
                let bounds = cx.cache.get_bounds(cx.current());
                let mouse_pos = (cx.mouse().cursor_x, cx.mouse().cursor_y);

                if let Some(handle) = self.get_handle_at_position(mouse_pos, bounds) {
                    self.drag_state = Some(handle);
                    self.drag_start_pos = mouse_pos;
                    self.drag_start_values = (
                        self.attack_normalized,
                        self.decay_normalized,
                        self.sustain_normalized,
                        self.release_normalized,
                    );

                    cx.capture();
                    cx.set_active(true);
                    meta.consume();
                }
            }

            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.drag_state.is_some() {
                    self.drag_state = None;
                    cx.release();
                    cx.set_active(false);
                    meta.consume();
                }
            }

            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        let padding = 10.0;

        // Draw background
        self.draw_background(canvas, bounds);

        // Draw grid lines
        self.draw_grid(canvas, bounds, padding);

        // Draw the envelope curve
        self.draw_envelope_curve(canvas, bounds);

        // TODO: Add text labels once we figure out the Font API
        // self.draw_labels(canvas, bounds, padding);
    }
}

impl EnvelopeEditor {
    /// Draw the background rectangle
    fn draw_background(&self, canvas: &Canvas, bounds: BoundingBox) {
        use vizia::vg::{Paint, Path, Rect};

        let mut path = Path::new();
        let rect = Rect::from_xywh(bounds.x, bounds.y, bounds.width(), bounds.height());
        path.add_rect(rect, None);

        let mut paint = Paint::default();
        paint.set_color(vizia::vg::Color::from_rgb(30, 30, 35));
        paint.set_anti_alias(true);

        canvas.draw_path(&path, &paint);
    }

    /// Draw grid lines for visual reference
    fn draw_grid(&self, canvas: &Canvas, bounds: BoundingBox, padding: f32) {
        use vizia::vg::{Paint, Path};

        let width = bounds.width() - 2.0 * padding;
        let height = bounds.height() - 2.0 * padding;

        let mut paint = Paint::default();
        paint.set_color(vizia::vg::Color::from_argb(100, 100, 100, 110));
        paint.set_anti_alias(true);
        paint.set_stroke_width(1.0);
        paint.set_style(vizia::vg::paint::Style::Stroke);

        // Vertical grid lines (3 divisions)
        for i in 1..3 {
            let x = bounds.x + padding + (i as f32 / 3.0) * width;
            let mut path = Path::new();
            path.move_to((x, bounds.y + padding));
            path.line_to((x, bounds.y + padding + height));
            canvas.draw_path(&path, &paint);
        }

        // Horizontal grid lines (4 divisions)
        for i in 1..4 {
            let y = bounds.y + padding + (i as f32 / 4.0) * height;
            let mut path = Path::new();
            path.move_to((bounds.x + padding, y));
            path.line_to((bounds.x + padding + width, y));
            canvas.draw_path(&path, &paint);
        }
    }

    /// Draw the ADSR envelope curve
    fn draw_envelope_curve(&self, canvas: &Canvas, bounds: BoundingBox) {
        use vizia::vg::{Paint, Path};

        let points = self.calculate_envelope_points();
        if points.len() < 2 {
            return;
        }

        // Convert to pixel coordinates
        let pixel_points: Vec<(f32, f32)> = points
            .iter()
            .map(|(nx, ny)| self.normalized_to_pixels(*nx, *ny, bounds))
            .collect();

        // Draw the envelope line
        let mut path = Path::new();
        let first = pixel_points[0];
        path.move_to(first);

        for &point in pixel_points.iter().skip(1) {
            path.line_to(point);
        }

        // Stroke the path with a bright color
        let mut paint = Paint::default();
        paint.set_color(vizia::vg::Color::from_rgb(100, 200, 255));
        paint.set_anti_alias(true);
        paint.set_stroke_width(2.5);
        paint.set_style(vizia::vg::paint::Style::Stroke);

        canvas.draw_path(&path, &paint);

        // Draw control point handles
        self.draw_handles(canvas, &pixel_points);
    }

    /// Draw draggable handles at control points
    fn draw_handles(&self, canvas: &Canvas, pixel_points: &[(f32, f32)]) {
        use vizia::vg::{Paint, Path};

        let handle_radius = 5.0;

        // Calculate which points should have handles
        let points = self.calculate_envelope_points();
        let section_width = 1.0 / 3.0;
        let decay_end_x = section_width + (self.decay_normalized * section_width);

        // Draw handles at:
        // - Point 1: Attack peak
        // - Point matching decay_end_x: Decay endpoint
        // - Last point: Release endpoint

        for (i, &(px, py)) in pixel_points.iter().enumerate() {
            let (nx, ny) = points[i];

            // Determine if this point should have a handle
            let handle_type = if i == 1 {
                Some(DraggedHandle::Attack)
            } else if (nx - decay_end_x).abs() < 0.001
                && (ny - self.sustain_normalized).abs() < 0.001
            {
                Some(DraggedHandle::Decay)
            } else if i == points.len() - 1 {
                Some(DraggedHandle::Release)
            } else {
                None
            };

            if let Some(handle_type) = handle_type {
                let mut path = Path::new();

                // Check if this handle is being dragged or hovered
                let is_dragging = self.drag_state == Some(handle_type);
                let is_hovered = self.hovered_handle == Some(handle_type);

                // Increase size if dragging or hovering
                let radius = if is_dragging {
                    handle_radius * 1.4
                } else if is_hovered {
                    handle_radius * 1.2
                } else {
                    handle_radius
                };

                path.add_circle((px, py), radius, None);

                // Fill the handle with color based on state
                let mut fill_paint = Paint::default();
                let fill_color = if is_dragging {
                    vizia::vg::Color::from_rgb(255, 200, 100) // Orange when dragging
                } else if is_hovered {
                    vizia::vg::Color::from_rgb(220, 240, 255) // Lighter blue when hovered
                } else {
                    vizia::vg::Color::from_rgb(200, 220, 255) // Default light blue
                };
                fill_paint.set_color(fill_color);
                fill_paint.set_anti_alias(true);
                canvas.draw_path(&path, &fill_paint);

                // Add a border
                let mut stroke_paint = Paint::default();
                let border_color = if is_dragging {
                    vizia::vg::Color::from_rgb(200, 100, 0) // Dark orange when dragging
                } else {
                    vizia::vg::Color::from_rgb(50, 50, 60) // Default dark gray
                };
                stroke_paint.set_color(border_color);
                stroke_paint.set_anti_alias(true);
                stroke_paint.set_stroke_width(if is_dragging { 2.0 } else { 1.5 });
                stroke_paint.set_style(vizia::vg::paint::Style::Stroke);
                canvas.draw_path(&path, &stroke_paint);
            }
        }
    }

    /// Draw text labels for each envelope stage
    /// Draw text labels for each envelope stage
    /// TODO: Fix Font API usage - currently disabled
    #[allow(dead_code)]
    fn draw_labels(&self, _canvas: &Canvas, _bounds: BoundingBox, _padding: f32) {
        // Skia Font API is complex - will implement later
        // For now, the visual curve is more important
    }

    /// Convert normalized coordinates to pixel coordinates
    fn normalized_to_pixels(&self, nx: f32, ny: f32, bounds: BoundingBox) -> (f32, f32) {
        let padding = 10.0;
        let width = bounds.width() - 2.0 * padding;
        let height = bounds.height() - 2.0 * padding;

        let px = bounds.x + padding + nx * width;
        // Y is inverted (0 is top, 1 is bottom) so we flip it
        let py = bounds.y + padding + (1.0 - ny) * height;

        (px, py)
    }
}
