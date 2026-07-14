//! Width-bounded text drawing shared by every console surface.

use macroquad::prelude::Color;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

/// Draw `text` at `x`/`y`, ellipsizing it to fit within `max_w`.
pub fn draw_bounded_text(text: &str, x: f32, y: f32, max_w: f32, font_size: f32, color: Color) {
    let bounded = truncate_text(text, max_w, font_size as u16);
    draw_ui_text(&bounded, x, y, font_size, color);
}

/// Shorten `text` with a trailing ellipsis until it measures under `max_w`.
pub fn truncate_text(text: &str, max_w: f32, font_size: u16) -> String {
    if measure_ui_text(text, None, font_size, 1.0).width <= max_w {
        return text.to_string();
    }

    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}...", out, ch);
        if measure_ui_text(&candidate, None, font_size, 1.0).width > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{}...", out)
}
