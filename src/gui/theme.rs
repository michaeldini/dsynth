// GUI Theme Constants - centralized layout and colors

use vizia::prelude::*;

// ========== Window Dimensions ==========
pub const WINDOW_WIDTH: u32 = 1200;
pub const WINDOW_HEIGHT: u32 = 800;

// ========== Layout Constants ==========
pub const OSC_COL_WIDTH: f32 = 360.0;
pub const ROW_GAP: f32 = 12.0;
pub const COL_GAP: f32 = 12.0;

// Widget sizing
pub const KNOB_SIZE: f32 = 54.0;
pub const KNOB_CELL_WIDTH: f32 = 54.0;

pub const SLIDER_WIDTH: f32 = 18.0;
pub const SLIDER_HEIGHT: f32 = 90.0;
pub const SLIDER_CELL_WIDTH: f32 = 42.0;
pub const SLIDER_HANDLE_HEIGHT: f32 = 8.0;

pub const CYCLE_BUTTON_CELL_WIDTH: f32 = 80.0;
pub const CHECKBOX_CELL_WIDTH: f32 = 80.0;
pub const LABEL_HEIGHT: f32 = 16.0;

// ========== Color Palette ==========
// Primary text
pub const TEXT_PRIMARY: Color = Color::rgb(220, 220, 230);
pub const TEXT_SECONDARY: Color = Color::rgb(200, 200, 210);
pub const TEXT_TERTIARY: Color = Color::rgb(180, 180, 190);
pub const TEXT_BRIGHT: Color = Color::rgb(240, 240, 240);

// Backgrounds
pub const BG_DARK: Color = Color::rgb(25, 25, 30);
pub const BG_SECTION: Color = Color::rgb(35, 35, 40);
pub const BG_PANEL: Color = Color::rgb(40, 40, 48);

// Interactive elements
pub const WIDGET_BG: Color = Color::rgb(55, 55, 62);
pub const WIDGET_BORDER: Color = Color::rgb(90, 90, 100);
pub const WIDGET_ACCENT: Color = Color::rgb(200, 200, 210);
pub const WIDGET_TRACK: Color = Color::rgb(120, 120, 130);

// Active states
pub const ACTIVE_BG: Color = Color::rgb(60, 60, 70);
pub const ACTIVE_BORDER: Color = Color::rgb(100, 100, 110);

// Button states
pub const BUTTON_BG_ACTIVE: Color = Color::rgb(60, 60, 70);
pub const BUTTON_TEXT_ACTIVE: Color = Color::rgb(220, 220, 230);
pub const BUTTON_BG_INACTIVE: Color = Color::rgb(40, 40, 48);
pub const BUTTON_TEXT_INACTIVE: Color = Color::rgb(180, 180, 190);
