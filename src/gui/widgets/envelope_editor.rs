use crate::gui::GuiState;
use crate::plugin::{param_registry, param_update::param_get};
use vizia::prelude::*;

/// Visual ADSR envelope editor with draggable control points
///
/// Reads parameter values from shared SynthParams for consistency with knobs/automation.
/// Dragging uses delta-from-start to avoid cursor drift and handle reset.
#[derive(Lens)]
pub struct EnvelopeEditor {
    // Parameter IDs
    attack_param_id: u32,
    decay_param_id: u32,
    sustain_param_id: u32,
    release_param_id: u32,
    attack_curve_param_id: u32,
    decay_curve_param_id: u32,
    release_curve_param_id: u32,

    // Interaction state
    #[lens(ignore)]
    drag_state: Option<DraggedHandle>,
    #[lens(ignore)]
    hovered_handle: Option<DraggedHandle>,

    // Drag capture
    #[lens(ignore)]
    drag_start_mouse: (f32, f32),
    #[lens(ignore)]
    drag_start_attack: f32,
    #[lens(ignore)]
    drag_start_decay: f32,
    #[lens(ignore)]
    drag_start_sustain: f32,
    #[lens(ignore)]
    drag_start_release: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DraggedHandle {
    Attack,
    Decay,
    Release,
}

impl EnvelopeEditor {
    #[allow(clippy::too_many_arguments)] // UI builder; params map directly to UI controls.
    pub fn new(
        cx: &mut Context,
        _attack_value: f32,
        _decay_value: f32,
        _sustain_value: f32,
        _release_value: f32,
        _attack_curve_value: f32,
        _decay_curve_value: f32,
        _release_curve_value: f32,
        attack_param_id: u32,
        decay_param_id: u32,
        sustain_param_id: u32,
        release_param_id: u32,
        attack_curve_param_id: u32,
        decay_curve_param_id: u32,
        release_curve_param_id: u32,
    ) -> Handle<'_, Self> {
        Self {
            attack_param_id,
            decay_param_id,
            sustain_param_id,
            release_param_id,
            attack_curve_param_id,
            decay_curve_param_id,
            release_curve_param_id,
            drag_state: None,
            hovered_handle: None,
            drag_start_mouse: (0.0, 0.0),
            drag_start_attack: 0.0,
            drag_start_decay: 0.0,
            drag_start_sustain: 0.0,
            drag_start_release: 0.0,
        }
        .build(cx, |_cx| {})
        .width(Pixels(300.0))
        .height(Pixels(150.0))
    }

    // --- Parameter helpers ---
    fn get_normalized_param(cx: &DrawContext, param_id: u32) -> f32 {
        let arc = GuiState::synth_params.get(cx);
        let params = arc.read();
        let denorm = param_get::get_param(&params, param_id);
        Self::denorm_to_normalized(param_id, denorm)
    }

    fn get_normalized_param_event(cx: &EventContext, param_id: u32) -> f32 {
        let arc = GuiState::synth_params.get(cx);
        let params = arc.read();
        let denorm = param_get::get_param(&params, param_id);
        Self::denorm_to_normalized(param_id, denorm)
    }

    fn denorm_to_normalized(param_id: u32, denorm: f32) -> f32 {
        let registry = param_registry::get_registry();
        registry
            .get(param_id)
            .map(|desc| desc.normalize_value(denorm))
            .unwrap_or(0.0)
    }

    // --- Geometry helpers ---
    fn apply_curve(progress: f32, curve: f32) -> f32 {
        if curve.abs() < 0.01 {
            progress
        } else {
            progress.powf(1.0 - curve * 0.67)
        }
    }

    fn calculate_envelope_points(
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> Vec<(f32, f32)> {
        let mut points = Vec::with_capacity(5);
        let section_width = 1.0 / 3.0;

        let attack_end_x = attack * section_width;
        let decay_end_x = section_width + (decay * section_width);
        let release_start_x = 2.0 * section_width;
        let release_end_x = release_start_x + (release * section_width);

        points.push((0.0, 0.0));
        points.push((attack_end_x, 1.0));
        points.push((decay_end_x, sustain));
        if decay_end_x < release_start_x {
            points.push((release_start_x, sustain));
        }
        points.push((release_end_x, 0.0));
        points
    }

    fn normalized_to_pixels(nx: f32, ny: f32, bounds: BoundingBox) -> (f32, f32) {
        let padding = 10.0;
        let width = bounds.width() - 2.0 * padding;
        let height = bounds.height() - 2.0 * padding;
        let px = bounds.x + padding + nx * width;
        let py = bounds.y + padding + (1.0 - ny) * height;
        (px, py)
    }

    fn get_handle_positions(&self, cx: &EventContext, bounds: BoundingBox) -> [(f32, f32); 3] {
        let attack = Self::get_normalized_param_event(cx, self.attack_param_id);
        let decay = Self::get_normalized_param_event(cx, self.decay_param_id);
        let sustain = Self::get_normalized_param_event(cx, self.sustain_param_id);
        let release = Self::get_normalized_param_event(cx, self.release_param_id);

        let points = Self::calculate_envelope_points(attack, decay, sustain, release);
        let pixels: Vec<(f32, f32)> = points
            .iter()
            .map(|(nx, ny)| Self::normalized_to_pixels(*nx, *ny, bounds))
            .collect();

        let attack_px = if pixels.len() > 1 {
            pixels[1]
        } else {
            (0.0_f32, 0.0_f32)
        };

        let decay_px = if pixels.len() > 2 {
            pixels[2]
        } else {
            (0.0_f32, 0.0_f32)
        };

        let release_px = pixels.last().cloned().unwrap_or((0.0_f32, 0.0_f32));

        [attack_px, decay_px, release_px]
    }

    // --- Hit test ---
    fn get_handle_at_position(
        &self,
        cx: &EventContext,
        mouse_pos: (f32, f32),
        bounds: BoundingBox,
    ) -> Option<DraggedHandle> {
        let handles = self.get_handle_positions(cx, bounds);
        let hit_radius = 12.0;
        for (i, &(hx, hy)) in handles.iter().enumerate() {
            let dx = mouse_pos.0 - hx;
            let dy = mouse_pos.1 - hy;
            if (dx * dx + dy * dy).sqrt() <= hit_radius {
                return match i {
                    0 => Some(DraggedHandle::Attack),
                    1 => Some(DraggedHandle::Decay),
                    2 => Some(DraggedHandle::Release),
                    _ => None,
                };
            }
        }
        None
    }

    // --- Drag handling ---
    fn handle_drag(
        &self,
        cx: &mut EventContext,
        mouse_pos: (f32, f32),
        bounds: BoundingBox,
        handle: DraggedHandle,
    ) {
        let padding = 10.0;
        let width = bounds.width() - 2.0 * padding;
        let height = bounds.height() - 2.0 * padding;
        let section_width = 1.0 / 3.0;

        // Delta from drag start in pixels
        let delta_px = mouse_pos.0 - self.drag_start_mouse.0;
        let delta_py = mouse_pos.1 - self.drag_start_mouse.1;

        // Convert to normalized deltas
        let delta_nx = delta_px / width;
        let delta_ny = -delta_py / height; // invert Y

        // Map envelope-x delta to parameter delta (each stage is 1/3 wide)
        let param_dx = delta_nx / section_width;

        match handle {
            DraggedHandle::Attack => {
                let val = (self.drag_start_attack + param_dx).clamp(0.0, 1.0);
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.attack_param_id,
                    val,
                ));
            }
            DraggedHandle::Decay => {
                let decay = (self.drag_start_decay + param_dx).clamp(0.0, 1.0);
                let sustain = (self.drag_start_sustain + delta_ny).clamp(0.0, 1.0);
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.decay_param_id,
                    decay,
                ));
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.sustain_param_id,
                    sustain,
                ));
            }
            DraggedHandle::Release => {
                let val = (self.drag_start_release + param_dx).clamp(0.0, 1.0);
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.release_param_id,
                    val,
                ));
            }
        }
        cx.needs_redraw();
    }
}

impl View for EnvelopeEditor {
    fn element(&self) -> Option<&'static str> {
        Some("envelope-editor")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // Redraw when parameters change elsewhere
        event.map(|gui_msg: &crate::gui::GuiMessage, _meta| {
            if let crate::gui::GuiMessage::ParamChanged(param_id, _)
            | crate::gui::GuiMessage::SyncKnobValue(param_id, _) = gui_msg
            {
                if *param_id == self.attack_param_id
                    || *param_id == self.decay_param_id
                    || *param_id == self.sustain_param_id
                    || *param_id == self.release_param_id
                    || *param_id == self.attack_curve_param_id
                    || *param_id == self.decay_curve_param_id
                    || *param_id == self.release_curve_param_id
                {
                    cx.needs_redraw();
                }
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseMove(x, y) => {
                let bounds = cx.cache.get_bounds(cx.current());
                let mouse_pos = (*x, *y);

                if let Some(handle) = self.drag_state {
                    self.handle_drag(cx, mouse_pos, bounds, handle);
                    meta.consume();
                } else {
                    let old_hover = self.hovered_handle;
                    self.hovered_handle = self.get_handle_at_position(cx, mouse_pos, bounds);
                    if old_hover != self.hovered_handle {
                        cx.needs_redraw();
                    }
                }
            }

            WindowEvent::MouseDown(MouseButton::Left) => {
                let bounds = cx.cache.get_bounds(cx.current());
                let mouse_pos = (cx.mouse().cursor_x, cx.mouse().cursor_y);

                if let Some(handle) = self.get_handle_at_position(cx, mouse_pos, bounds) {
                    self.drag_state = Some(handle);
                    self.drag_start_mouse = mouse_pos;

                    // Capture starting parameter values
                    self.drag_start_attack =
                        Self::get_normalized_param_event(cx, self.attack_param_id);
                    self.drag_start_decay =
                        Self::get_normalized_param_event(cx, self.decay_param_id);
                    self.drag_start_sustain =
                        Self::get_normalized_param_event(cx, self.sustain_param_id);
                    self.drag_start_release =
                        Self::get_normalized_param_event(cx, self.release_param_id);

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

        self.draw_background(canvas, bounds);
        self.draw_grid(canvas, bounds, padding);

        let attack = Self::get_normalized_param(cx, self.attack_param_id);
        let decay = Self::get_normalized_param(cx, self.decay_param_id);
        let sustain = Self::get_normalized_param(cx, self.sustain_param_id);
        let release = Self::get_normalized_param(cx, self.release_param_id);

        let attack_curve = (Self::get_normalized_param(cx, self.attack_curve_param_id) * 2.0 - 1.0)
            .clamp(-1.0, 1.0);
        let decay_curve = (Self::get_normalized_param(cx, self.decay_curve_param_id) * 2.0 - 1.0)
            .clamp(-1.0, 1.0);
        let release_curve = (Self::get_normalized_param(cx, self.release_curve_param_id) * 2.0
            - 1.0)
            .clamp(-1.0, 1.0);

        self.draw_envelope_curve(
            canvas,
            bounds,
            attack,
            decay,
            sustain,
            release,
            attack_curve,
            decay_curve,
            release_curve,
        );
    }
}

impl EnvelopeEditor {
    fn draw_background(&self, canvas: &Canvas, bounds: BoundingBox) {
        use vizia::vg::{Paint, Path, Rect};
        let mut path = Path::new();
        path.add_rect(
            Rect::from_xywh(bounds.x, bounds.y, bounds.width(), bounds.height()),
            None,
        );
        let mut paint = Paint::default();
        paint.set_color(vizia::vg::Color::from_rgb(30, 30, 35));
        paint.set_anti_alias(true);
        canvas.draw_path(&path, &paint);
    }

    fn draw_grid(&self, canvas: &Canvas, bounds: BoundingBox, padding: f32) {
        use vizia::vg::{Paint, Path};
        let width = bounds.width() - 2.0 * padding;
        let height = bounds.height() - 2.0 * padding;
        let mut paint = Paint::default();
        paint.set_color(vizia::vg::Color::from_argb(100, 100, 100, 110));
        paint.set_anti_alias(true);
        paint.set_stroke_width(1.0);
        paint.set_style(vizia::vg::paint::Style::Stroke);

        for i in 1..3 {
            let x = bounds.x + padding + (i as f32 / 3.0) * width;
            let mut path = Path::new();
            path.move_to((x, bounds.y + padding));
            path.line_to((x, bounds.y + padding + height));
            canvas.draw_path(&path, &paint);
        }

        for i in 1..4 {
            let y = bounds.y + padding + (i as f32 / 4.0) * height;
            let mut path = Path::new();
            path.move_to((bounds.x + padding, y));
            path.line_to((bounds.x + padding + width, y));
            canvas.draw_path(&path, &paint);
        }
    }

    #[allow(clippy::too_many_arguments)] // Rendering helper; clarity over bundling.
    fn draw_envelope_curve(
        &self,
        canvas: &Canvas,
        bounds: BoundingBox,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
        attack_curve: f32,
        decay_curve: f32,
        release_curve: f32,
    ) {
        use vizia::vg::{Paint, Path};

        let points = Self::calculate_envelope_points(attack, decay, sustain, release);
        if points.len() < 2 {
            return;
        }

        let pixels: Vec<(f32, f32)> = points
            .iter()
            .map(|(nx, ny)| Self::normalized_to_pixels(*nx, *ny, bounds))
            .collect();

        let mut path = Path::new();
        path.move_to(pixels[0]);

        if pixels.len() > 1 {
            Self::draw_curved_segment(&mut path, pixels[0], pixels[1], attack_curve);
        }
        if pixels.len() > 2 {
            Self::draw_curved_segment(&mut path, pixels[1], pixels[2], decay_curve);
        }
        if pixels.len() > 3 {
            path.line_to(pixels[3]);
        }
        let release_start = if pixels.len() > 4 { 3 } else { 2 };
        let release_end = release_start + 1;
        if pixels.len() > release_end {
            Self::draw_curved_segment(
                &mut path,
                pixels[release_start],
                pixels[release_end],
                release_curve,
            );
        }

        let mut paint = Paint::default();
        paint.set_color(vizia::vg::Color::from_rgb(100, 200, 255));
        paint.set_anti_alias(true);
        paint.set_stroke_width(2.5);
        paint.set_style(vizia::vg::paint::Style::Stroke);
        canvas.draw_path(&path, &paint);

        self.draw_handles(canvas, &pixels);
    }

    fn draw_curved_segment(
        path: &mut vizia::vg::Path,
        start: (f32, f32),
        end: (f32, f32),
        curve: f32,
    ) {
        if curve.abs() < 0.01 {
            path.line_to(end);
            return;
        }
        let steps = 24;
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let tc = Self::apply_curve(t, curve);
            path.line_to((start.0 + dx * t, start.1 + dy * tc));
        }
    }

    fn draw_handles(&self, canvas: &Canvas, pixels: &[(f32, f32)]) {
        use vizia::vg::{Paint, Path};
        let handle_radius = 5.0;
        let handle_indices = [
            (1, DraggedHandle::Attack),
            (2, DraggedHandle::Decay),
            (pixels.len().saturating_sub(1), DraggedHandle::Release),
        ];

        for (idx, handle_type) in handle_indices {
            if idx >= pixels.len() {
                continue;
            }
            let (px, py) = pixels[idx];
            let is_dragging = self.drag_state == Some(handle_type);
            let is_hovered = self.hovered_handle == Some(handle_type);
            let radius = if is_dragging {
                handle_radius * 1.4
            } else if is_hovered {
                handle_radius * 1.2
            } else {
                handle_radius
            };

            let mut path = Path::new();
            path.add_circle((px, py), radius, None);

            let mut fill = Paint::default();
            fill.set_color(if is_dragging {
                vizia::vg::Color::from_rgb(255, 200, 100)
            } else if is_hovered {
                vizia::vg::Color::from_rgb(220, 240, 255)
            } else {
                vizia::vg::Color::from_rgb(200, 220, 255)
            });
            fill.set_anti_alias(true);
            canvas.draw_path(&path, &fill);

            let mut stroke = Paint::default();
            stroke.set_color(if is_dragging {
                vizia::vg::Color::from_rgb(200, 100, 0)
            } else {
                vizia::vg::Color::from_rgb(50, 50, 60)
            });
            stroke.set_anti_alias(true);
            stroke.set_stroke_width(if is_dragging { 2.0 } else { 1.5 });
            stroke.set_style(vizia::vg::paint::Style::Stroke);
            canvas.draw_path(&path, &stroke);
        }
    }
}
