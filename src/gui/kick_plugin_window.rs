// CLAP kick plugin window integration with VIZIA (baseview)
//
// This module is only compiled when the "kick-clap" feature is enabled.

use crate::gui::theme;
use crate::gui::widgets::param_knob;
use crate::gui::GuiMessage;
use crate::params_kick::KickParams;
use crate::plugin::kick_param_registry::{get_kick_registry, ParamId};
use parking_lot::Mutex;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle as RawHandle};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vizia::prelude::*;

use vizia_baseview::{Application, WindowHandle as ViziaBaseviewWindowHandle};

const KICK_BG_JPG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/synthofchrist.jpg"
));

const KICK_BG_NAME: &str = "dsynth-kick-bg";

/// Wrapper to make RawWindowHandle implement HasRawWindowHandle
struct WindowHandleWrapper(RawHandle);

unsafe impl HasRawWindowHandle for WindowHandleWrapper {
    fn raw_window_handle(&self) -> RawHandle {
        self.0
    }
}

#[derive(Lens)]
pub struct KickGuiState {
    params: Arc<Mutex<KickParams>>,
    last_param_text: String,

    // Host automation -> GUI sync
    #[lens(ignore)]
    last_seen: HashMap<ParamId, f32>,
    #[lens(ignore)]
    last_poll: Instant,
}

impl KickGuiState {
    pub fn new(params: Arc<Mutex<KickParams>>) -> Self {
        let registry = get_kick_registry();
        let snapshot = params.lock().clone();

        let mut last_seen = HashMap::with_capacity(registry.param_count());
        for &param_id in registry.param_ids() {
            last_seen.insert(param_id, registry.get_param(&snapshot, param_id) as f32);
        }

        Self {
            params,
            last_param_text: String::new(),
            last_seen,
            last_poll: Instant::now(),
        }
    }

    fn update_param(&mut self, param_id: ParamId, normalized: f32) {
        let registry = get_kick_registry();
        let mut params = self.params.lock();
        let normalized = normalized.clamp(0.0, 1.0);
        registry.apply_param(&mut params, param_id, normalized as f64);
        self.last_seen.insert(param_id, normalized);

        if let Some(desc) = registry.get_descriptor(param_id) {
            self.last_param_text = format!("{}: {}", desc.name, desc.format_value(normalized));
        } else {
            self.last_param_text = format!(
                "Param 0x{:08X}: {:.0}%",
                param_id,
                normalized.clamp(0.0, 1.0) * 100.0
            );
        }
    }

    fn poll_host_changes(&mut self, cx: &mut EventContext) {
        // Throttle to reduce GUI<->audio thread lock contention.
        const POLL_INTERVAL: Duration = Duration::from_millis(50);
        if self.last_poll.elapsed() < POLL_INTERVAL {
            // Keep the redraw loop alive so automation updates propagate.
            cx.needs_redraw();
            return;
        }
        self.last_poll = Instant::now();

        let registry = get_kick_registry();
        let snapshot = self.params.lock().clone();

        for &param_id in registry.param_ids() {
            let normalized = (registry.get_param(&snapshot, param_id) as f32).clamp(0.0, 1.0);

            let changed = match self.last_seen.get(&param_id) {
                Some(prev) => (prev - normalized).abs() > 1e-4,
                None => true,
            };

            if changed {
                self.last_seen.insert(param_id, normalized);
                cx.emit_custom(
                    Event::new(GuiMessage::SyncKnobValue(param_id, normalized))
                        .propagate(Propagation::Subtree),
                );
            }
        }

        // Keep polling while the editor is open.
        cx.needs_redraw();
    }
}

impl Model for KickGuiState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|msg, meta| {
            if let GuiMessage::ParamChanged(param_id, normalized) = msg {
                self.update_param(*param_id, *normalized);
                cx.needs_redraw();
                meta.consume();
            }
        });

        // Continuous (throttled) polling so DAW automation updates knob visuals.
        event.map(|window_event, _meta| {
            if let WindowEvent::Redraw = window_event {
                self.poll_host_changes(cx);
            }
        });
    }
}

/// Opens a VIZIA-based kick plugin editor window.
///
/// Important: the returned handle must be kept alive by the plugin instance.
pub fn open_editor(
    parent_window: raw_window_handle::RawWindowHandle,
    kick_params: Arc<Mutex<KickParams>>,
    width: u32,
    height: u32,
) -> Option<EditorWindowHandle> {
    let window_wrapper = WindowHandleWrapper(parent_window);

    let width = width.max(1);
    let height = height.max(1);

    let handle = Application::new(move |cx| {
        // Embed the background image directly in the binary (plugin-friendly: no runtime paths).
        cx.load_image(KICK_BG_NAME, KICK_BG_JPG, ImageRetentionPolicy::Forever);

        // This VIZIA version doesn't expose a `background_size()` style modifier,
        // so configure background sizing via a small stylesheet.
        let _ = cx.add_stylesheet(CSS::from_string(
            r#"
.kick-bg {
    background-size: cover;
}
"#,
        ));

        KickGuiState::new(kick_params.clone()).build(cx);
        build_kick_ui(cx, kick_params.clone());
    })
    .inner_size((width, height))
    .open_parented(&window_wrapper);

    Some(EditorWindowHandle { _inner: handle })
}

/// Wrapper which keeps the baseview window handle alive.
pub struct EditorWindowHandle {
    _inner: ViziaBaseviewWindowHandle,
}

#[derive(Lens)]
struct KickToggle {
    checked: bool,
    default_checked: bool,
    param_id: ParamId,
}

impl KickToggle {
    fn new(
        cx: &mut Context,
        initial_checked: bool,
        param_id: ParamId,
        default_checked: bool,
    ) -> Handle<'_, Self> {
        Self {
            checked: initial_checked,
            default_checked,
            param_id,
        }
        .build(cx, |cx| {
            Binding::new(cx, KickToggle::checked, |cx, checked| {
                let is_checked = checked.get(cx);
                Label::new(cx, if is_checked { "✓" } else { "✗" })
                    .font_size(18.0)
                    .color(if is_checked {
                        theme::TEXT_PRIMARY
                    } else {
                        theme::TEXT_TERTIARY
                    })
                    .text_align(TextAlign::Center)
                    .width(Stretch(1.0))
                    .height(Stretch(1.0));
            });
        })
        .width(Pixels(theme::KNOB_SIZE))
        .height(Pixels(theme::KNOB_SIZE))
        .background_color(theme::WIDGET_BG)
        .border_width(Pixels(2.0))
        .border_color(theme::WIDGET_BORDER)
        .corner_radius(Pixels(6.0))
        .cursor(CursorIcon::Hand)
    }
}

impl View for KickToggle {
    fn element(&self) -> Option<&'static str> {
        Some("kick-toggle")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|gui_msg: &crate::gui::GuiMessage, _meta| {
            if let crate::gui::GuiMessage::SyncKnobValue(param_id, normalized) = gui_msg {
                if *param_id == self.param_id {
                    self.checked = *normalized >= 0.5;
                }
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                self.checked = !self.checked;
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.param_id,
                    if self.checked { 1.0 } else { 0.0 },
                ));
                meta.consume();
            }

            WindowEvent::MouseDoubleClick(MouseButton::Left) => {
                self.checked = self.default_checked;
                cx.emit(crate::gui::GuiMessage::ParamChanged(
                    self.param_id,
                    if self.checked { 1.0 } else { 0.0 },
                ));
                meta.consume();
            }

            _ => {}
        });
    }
}

fn build_section(cx: &mut Context, title: &str, content: impl FnOnce(&mut Context) + 'static) {
    // Baseview + ScrollView can mis-measure Auto-sized children, leading to
    // collapsed sections (0 height), overlapping headers, and no scrolling.
    // Since each section here is exactly one knob row, give the section a
    // deterministic height so layout is stable.
    const TITLE_H: f32 = 22.0;
    const SECTION_PAD: f32 = 12.0;
    const SECTION_GAP: f32 = 10.0;
    let row_h = theme::LABEL_HEIGHT + 4.0 + theme::KNOB_SIZE;
    let section_h = TITLE_H + SECTION_GAP + row_h + (SECTION_PAD * 2.0);

    VStack::new(cx, move |cx| {
        Label::new(cx, title)
            .font_size(14.0)
            .color(theme::TEXT_SECONDARY)
            // .background_color(theme::BG_DARK)
            .height(Pixels(TITLE_H))
            .width(Stretch(1.0));
        content(cx);
    })
    .width(Stretch(1.0))
    .height(Pixels(section_h))
    .padding(Pixels(12.0))
    .gap(Pixels(10.0))
    .background_color(theme::BG_SECTION)
    .corner_radius(Pixels(6.0));
}

fn build_knob_row(cx: &mut Context, items: &[(ParamId, &str, f32, f32)]) {
    HStack::new(cx, move |cx| {
        for (param_id, label, initial, default) in items {
            param_knob(cx, *param_id, label, *initial, *default);
        }
    })
    .width(Stretch(1.0))
    // Critical: without an explicit row height, the flex layout can treat this as
    // having ~0 height in some backends, causing subsequent section headers to
    // overlap the knob content (and breaking scroll height calculation).
    .height(Pixels(theme::LABEL_HEIGHT + 4.0 + theme::KNOB_SIZE))
    .gap(Pixels(10.0));
}

#[allow(dead_code)]
fn build_toggle_row(cx: &mut Context, items: Vec<(ParamId, &'static str, bool, bool)>) {
    HStack::new(cx, move |cx| {
        for item in items.iter() {
            let (param_id, label, initial_checked, default_checked) = *item;
            VStack::new(cx, move |cx| {
                Label::new(cx, label)
                    .font_size(11.0)
                    .background_color(theme::BG_DARK)
                    .color(theme::TEXT_SECONDARY)
                    .width(Pixels(theme::KNOB_CELL_WIDTH))
                    .height(Pixels(theme::LABEL_HEIGHT))
                    .text_align(TextAlign::Center)
                    .text_wrap(false)
                    .text_overflow(TextOverflow::Ellipsis);

                KickToggle::new(cx, initial_checked, param_id, default_checked);
            })
            .width(Pixels(theme::KNOB_CELL_WIDTH))
            .height(Pixels(theme::LABEL_HEIGHT + 4.0 + theme::KNOB_SIZE))
            .gap(Pixels(4.0));
        }
    })
    .width(Stretch(1.0))
    .height(Pixels(theme::LABEL_HEIGHT + 4.0 + theme::KNOB_SIZE))
    .gap(Pixels(10.0));
}

fn build_mixed_row(
    cx: &mut Context,
    knobs: Vec<(ParamId, &'static str, f32, f32)>,
    toggles: Vec<(ParamId, &'static str, bool, bool)>,
) {
    HStack::new(cx, move |cx| {
        for item in knobs.iter() {
            let (param_id, label, initial, default) = *item;
            param_knob(cx, param_id, label, initial, default);
        }
        for item in toggles.iter() {
            let (param_id, label, initial_checked, default_checked) = *item;
            VStack::new(cx, move |cx| {
                Label::new(cx, label)
                    .font_size(11.0)
                    .background_color(theme::BG_DARK)
                    .color(theme::TEXT_SECONDARY)
                    .width(Pixels(theme::KNOB_CELL_WIDTH))
                    .height(Pixels(theme::LABEL_HEIGHT))
                    .text_align(TextAlign::Center)
                    .text_wrap(false)
                    .text_overflow(TextOverflow::Ellipsis);

                KickToggle::new(cx, initial_checked, param_id, default_checked);
            })
            .width(Pixels(theme::KNOB_CELL_WIDTH))
            .height(Pixels(theme::LABEL_HEIGHT + 4.0 + theme::KNOB_SIZE))
            .gap(Pixels(4.0));
        }
    })
    .width(Stretch(1.0))
    .height(Pixels(theme::LABEL_HEIGHT + 4.0 + theme::KNOB_SIZE))
    .gap(Pixels(10.0));
}

fn default_normalized(
    registry: &crate::plugin::kick_param_registry::KickParamRegistry,
    id: ParamId,
) -> f32 {
    registry
        .get_descriptor(id)
        .map(|d| d.normalize_value(d.default))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

fn build_kick_ui(cx: &mut Context, kick_params: Arc<Mutex<KickParams>>) {
    let registry = get_kick_registry();
    let params_snapshot = kick_params.lock().clone();

    let item = |id: ParamId, label: &'static str| -> (ParamId, &'static str, f32, f32) {
        let initial = registry.get_param(&params_snapshot, id) as f32;
        let default = default_normalized(registry, id);
        (id, label, initial, default)
    };

    let toggle = |id: ParamId, label: &'static str| -> (ParamId, &'static str, bool, bool) {
        let initial = (registry.get_param(&params_snapshot, id) as f32).clamp(0.0, 1.0);
        let default = default_normalized(registry, id);
        (id, label, initial >= 0.5, default >= 0.5)
    };

    let body_row = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_OSC1_PITCH_START,
            "Start",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_OSC1_PITCH_END,
            "End",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_OSC1_PITCH_DECAY,
            "Decay",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_OSC1_LEVEL,
            "Level",
        ),
    ];

    let click_row = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_OSC2_PITCH_START,
            "Start",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_OSC2_PITCH_END,
            "End",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_OSC2_PITCH_DECAY,
            "Decay",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_OSC2_LEVEL,
            "Level",
        ),
    ];

    let env_row = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_AMP_ATTACK,
            "Attack",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_AMP_DECAY,
            "Decay",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_AMP_SUSTAIN,
            "Sustain",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_AMP_RELEASE,
            "Release",
        ),
    ];

    let filter_row = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_FILTER_CUTOFF,
            "Cutoff",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_FILTER_RESONANCE,
            "Reso",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_FILTER_ENV_AMOUNT,
            "Env",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_FILTER_ENV_DECAY,
            "Decay",
        ),
    ];

    let dist_knobs = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_DISTORTION_AMOUNT,
            "Amount",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_DISTORTION_TYPE,
            "Type",
        ),
    ];

    let dist_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_DISTORTION_ENABLED,
        "Enable",
    )];

    let master_knobs = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MASTER_LEVEL,
            "Level",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_VELOCITY_SENSITIVITY,
            "Vel",
        ),
    ];

    // Treat key tracking as an on/off toggle in the UI (0.0 = off, 1.0 = on)
    let master_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_KEY_TRACKING,
        "KeyTrack",
    )];

    // Multiband Compression - Crossovers
    let mb_xover_knobs = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_XOVER_LOW,
            "Low Xover",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_XOVER_HIGH,
            "High Xover",
        ),
        item(crate::plugin::kick_param_registry::PARAM_KICK_MB_MIX, "Mix"),
    ];

    let mb_xover_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_MB_ENABLED,
        "Enable",
    )];

    // Multiband Compression - Sub Band
    let mb_sub_knobs = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_SUB_THRESHOLD,
            "Thresh",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_SUB_RATIO,
            "Ratio",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_SUB_ATTACK,
            "Attack",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_SUB_RELEASE,
            "Release",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_SUB_GAIN,
            "Gain",
        ),
    ];

    let mb_sub_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_MB_SUB_BYPASS,
        "Bypass",
    )];

    // Multiband Compression - Body Band
    let mb_body_knobs = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_BODY_THRESHOLD,
            "Thresh",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_BODY_RATIO,
            "Ratio",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_BODY_ATTACK,
            "Attack",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_BODY_RELEASE,
            "Release",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_BODY_GAIN,
            "Gain",
        ),
    ];

    let mb_body_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_MB_BODY_BYPASS,
        "Bypass",
    )];

    // Multiband Compression - Click Band
    let mb_click_knobs = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_CLICK_THRESHOLD,
            "Thresh",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_CLICK_RATIO,
            "Ratio",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_CLICK_ATTACK,
            "Attack",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_CLICK_RELEASE,
            "Release",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_MB_CLICK_GAIN,
            "Gain",
        ),
    ];

    let mb_click_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_MB_CLICK_BYPASS,
        "Bypass",
    )];

    // Exciter
    let exciter_knobs = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_EXCITER_FREQUENCY,
            "Frequency",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_EXCITER_DRIVE,
            "Drive",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_EXCITER_MIX,
            "Mix",
        ),
    ];

    let exciter_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_EXCITER_ENABLED,
        "Enable",
    )];

    // Transient Shaper
    let transient_knobs = [
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_TRANSIENT_ATTACK_BOOST,
            "Attack Boost",
        ),
        item(
            crate::plugin::kick_param_registry::PARAM_KICK_TRANSIENT_SUSTAIN_REDUCTION,
            "Sustain Reduce",
        ),
    ];

    let transient_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_TRANSIENT_ENABLED,
        "Enable",
    )];

    // Clipper
    let clipper_knobs = [item(
        crate::plugin::kick_param_registry::PARAM_KICK_CLIPPER_THRESHOLD,
        "Threshold",
    )];

    let clipper_toggles = [toggle(
        crate::plugin::kick_param_registry::PARAM_KICK_CLIPPER_ENABLED,
        "Enable",
    )];

    ZStack::new(cx, move |cx| {
        // Background image layer.
        Element::new(cx)
            .background_image(format!("'{}'", KICK_BG_NAME).as_str())
            .class("kick-bg")
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .hoverable(false);

        // Foreground UI layer.
        VStack::new(cx, move |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "Synth of Christ Kick Drum")
                    .font_size(18.0)
                    .color(theme::TEXT_PRIMARY);

                Binding::new(cx, KickGuiState::last_param_text, |cx, text| {
                    Label::new(cx, text.get(cx))
                        .font_size(16.0)
                        .color(theme::TEXT_TERTIARY)
                        .width(Stretch(1.0))
                        .text_align(TextAlign::Right)
                        .text_wrap(false)
                        .text_overflow(TextOverflow::Ellipsis);
                });
            })
            .height(Pixels(42.0))
            .padding(Pixels(10.0))
            .background_color(theme::BG_DARK);

            ScrollView::new(cx, move |cx| {
                VStack::new(cx, move |cx| {
                    build_section(cx, "Body Osc", move |cx| {
                        build_knob_row(cx, &body_row);
                    });

                    build_section(cx, "Click Osc", move |cx| {
                        build_knob_row(cx, &click_row);
                    });

                    build_section(cx, "Envelope", move |cx| {
                        build_knob_row(cx, &env_row);
                    });

                    build_section(cx, "Filter", move |cx| {
                        build_knob_row(cx, &filter_row);
                    });

                    build_section(cx, "Distortion", move |cx| {
                        build_mixed_row(cx, dist_knobs.to_vec(), dist_toggles.to_vec());
                    });

                    build_section(cx, "Master", move |cx| {
                        build_mixed_row(cx, master_knobs.to_vec(), master_toggles.to_vec());
                    });

                    // Multiband Compression Section
                    build_section(cx, "Multiband Comp - Crossovers", move |cx| {
                        build_mixed_row(cx, mb_xover_knobs.to_vec(), mb_xover_toggles.to_vec());
                    });

                    build_section(cx, "MB Comp - Sub Band (40-150Hz)", move |cx| {
                        build_mixed_row(cx, mb_sub_knobs.to_vec(), mb_sub_toggles.to_vec());
                    });

                    build_section(cx, "MB Comp - Body Band (150-800Hz)", move |cx| {
                        build_mixed_row(cx, mb_body_knobs.to_vec(), mb_body_toggles.to_vec());
                    });

                    build_section(cx, "MB Comp - Click Band (800Hz+)", move |cx| {
                        build_mixed_row(cx, mb_click_knobs.to_vec(), mb_click_toggles.to_vec());
                    });

                    // Exciter Section
                    build_section(cx, "Exciter", move |cx| {
                        build_mixed_row(cx, exciter_knobs.to_vec(), exciter_toggles.to_vec());
                    });

                    // Transient Shaper Section
                    build_section(cx, "Transient Shaper", move |cx| {
                        build_mixed_row(cx, transient_knobs.to_vec(), transient_toggles.to_vec());
                    });

                    // Clipper Section
                    build_section(cx, "Clipper", move |cx| {
                        build_mixed_row(cx, clipper_knobs.to_vec(), clipper_toggles.to_vec());
                    });
                })
                .width(Stretch(1.0))
                .height(Units::Auto)
                .min_height(Pixels(0.0))
                .gap(Pixels(18.0))
                .padding(Pixels(12.0));
            })
            .show_horizontal_scrollbar(false)
            .show_vertical_scrollbar(true)
            .background_color(theme::BG_DARK)
            // Ensure the scroll area fills the remaining editor space.
            // Without this, the content can be clipped instead of scrollable.
            .width(Stretch(1.0))
            .height(Stretch(1.0));
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0));
    })
    .width(Stretch(1.0))
    .height(Stretch(1.0));
}
