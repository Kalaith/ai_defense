//! Console chrome: panels, headers, pills, and buttons.

use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::math::pulse_range;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::text::truncate_text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleButtonState {
    Disabled,
    Affordable,
    Dangerous,
    Recommended,
}

pub fn draw_resource_bar(x: f32, y: f32, w: f32, h: f32, current: f32, max: f32, color: Color) {
    macroquad_toolkit::ui::progress_bar(x, y, w, h, current, max, color);
}

pub fn draw_console_panel(rect: Rect, accent: Color) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.035, 0.045, 0.052, 0.96),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, accent);

    let mut y = rect.y + 5.0;
    while y < rect.y + rect.h {
        draw_line(
            rect.x + 2.0,
            y,
            rect.x + rect.w - 2.0,
            y,
            1.0,
            Color::new(1.0, 1.0, 1.0, 0.025),
        );
        y += 6.0;
    }
}

pub fn draw_console_header(x: f32, y: f32, label: &str, sublabel: &str, color: Color) {
    draw_ui_text(label, x, y, 17.0, color);
    if !sublabel.is_empty() {
        draw_ui_text(sublabel, x, y + 16.0, 10.0, dark::TEXT_DIM);
    }
}

pub fn draw_status_pill(x: f32, y: f32, label: &str, color: Color) {
    let font_size = 11.0;
    let h = 18.0;
    let w = measure_ui_text(label, None, font_size as u16, 1.0).width + 16.0;
    draw_rectangle(x, y, w, h, Color::new(color.r, color.g, color.b, 0.16));
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(color.r, color.g, color.b, 0.82));
    draw_ui_text(label, x + 8.0, y + 12.5, font_size, color);
}

pub fn draw_console_button(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    state: ConsoleButtonState,
) -> bool {
    let pulse = pulse_range(3.2, 0.55, 0.77);
    let (base_fill, accent, text) = match state {
        ConsoleButtonState::Disabled => (
            Color::new(0.03, 0.035, 0.04, 0.62),
            Color::new(0.28, 0.32, 0.34, 0.55),
            dark::TEXT_DIM,
        ),
        ConsoleButtonState::Affordable => (
            Color::new(0.025, 0.055, 0.075, 0.7),
            Color::new(0.28, 0.74, 1.0, 0.78 + pulse * 0.08),
            Color::new(0.86, 0.96, 1.0, 1.0),
        ),
        ConsoleButtonState::Dangerous => (
            Color::new(0.09, 0.035, 0.025, 0.74),
            Color::new(0.95, 0.18, 0.15, 0.82 + pulse * 0.12),
            Color::new(1.0, 0.68, 0.38, 1.0),
        ),
        ConsoleButtonState::Recommended => (
            Color::new(0.025, 0.075, 0.055, 0.74),
            Color::new(0.28, 0.95, 0.65, 0.78 + pulse * 0.16),
            Color::new(0.74, 1.0, 0.88, 1.0),
        ),
    };

    let (mx, my) = mouse_position();
    let hovered = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let enabled = !matches!(state, ConsoleButtonState::Disabled);
    let hover_boost = if hovered && enabled { 0.08 } else { 0.0 };
    let fill = Color::new(
        (base_fill.r + hover_boost).min(1.0),
        (base_fill.g + hover_boost).min(1.0),
        (base_fill.b + hover_boost).min(1.0),
        if hovered && enabled {
            (base_fill.a + 0.08).min(0.92)
        } else {
            base_fill.a
        },
    );

    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 1.8, accent);
    draw_line(
        x,
        y + h - 2.0,
        x + w,
        y + h - 2.0,
        2.0,
        Color::new(
            accent.r,
            accent.g,
            accent.b,
            if enabled { 0.58 } else { 0.22 },
        ),
    );
    draw_rectangle(
        x,
        y,
        if w >= 110.0 { 4.0 } else { 3.0 },
        h,
        Color::new(
            accent.r,
            accent.g,
            accent.b,
            if enabled { 0.78 } else { 0.24 },
        ),
    );

    if hovered && enabled {
        draw_rectangle_lines(
            x - 2.0,
            y - 2.0,
            w + 4.0,
            h + 4.0,
            1.0,
            Color::new(accent.r, accent.g, accent.b, 0.42),
        );
    }

    if w >= 128.0 && h >= 36.0 {
        let marker_x = x + w - 28.0;
        let marker_y = y + h * 0.5;
        draw_circle_lines(marker_x, marker_y, 10.0, 1.3, accent);
        if enabled {
            draw_triangle(
                vec2(marker_x - 3.0, marker_y - 5.0),
                vec2(marker_x - 3.0, marker_y + 5.0),
                vec2(marker_x + 5.0, marker_y),
                accent,
            );
        } else {
            draw_line(
                marker_x - 5.0,
                marker_y,
                marker_x + 5.0,
                marker_y,
                1.3,
                accent,
            );
        }
    }

    if h >= 36.0 {
        draw_button_corners(Rect::new(x, y, w, h), accent);
    }

    let font_size = if label.len() > 18 {
        11.0
    } else if h >= 40.0 {
        13.0
    } else {
        12.0
    };
    let text_pad = if w >= 128.0 && h >= 36.0 { 42.0 } else { 14.0 };
    let bounded = truncate_text(label, w - text_pad, font_size as u16);
    let dims = measure_ui_text(&bounded, None, font_size as u16, 1.0);
    draw_ui_text(
        &bounded,
        x + (w - text_pad - dims.width) * 0.5 + 8.0,
        y + (h + font_size * 0.55) * 0.5,
        font_size,
        text,
    );

    if !enabled {
        return false;
    }
    // Deliberately fires on press, not release like toolkit::input::was_clicked
    // — these are high-stakes console actions (start/shutdown beacon, place
    // tower) where instant feedback on press reads as more responsive, and
    // there's no drag-to-cancel gesture in this UI to protect against.
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

/// Bracket the corners of `rect` with short accent ticks.
pub(super) fn draw_button_corners(rect: Rect, color: Color) {
    let len = 10.0_f32.min(rect.w * 0.18).min(rect.h * 0.32);
    let inset = 3.0;
    let c = Color::new(color.r, color.g, color.b, color.a.min(0.86));

    draw_line(
        rect.x + inset,
        rect.y + inset,
        rect.x + inset + len,
        rect.y + inset,
        1.3,
        c,
    );
    draw_line(
        rect.x + inset,
        rect.y + inset,
        rect.x + inset,
        rect.y + inset + len,
        1.3,
        c,
    );
    draw_line(
        rect.x + rect.w - inset - len,
        rect.y + inset,
        rect.x + rect.w - inset,
        rect.y + inset,
        1.3,
        c,
    );
    draw_line(
        rect.x + rect.w - inset,
        rect.y + inset,
        rect.x + rect.w - inset,
        rect.y + inset + len,
        1.3,
        c,
    );
    draw_line(
        rect.x + inset,
        rect.y + rect.h - inset,
        rect.x + inset + len,
        rect.y + rect.h - inset,
        1.3,
        c,
    );
    draw_line(
        rect.x + inset,
        rect.y + rect.h - inset - len,
        rect.x + inset,
        rect.y + rect.h - inset,
        1.3,
        c,
    );
    draw_line(
        rect.x + rect.w - inset - len,
        rect.y + rect.h - inset,
        rect.x + rect.w - inset,
        rect.y + rect.h - inset,
        1.3,
        c,
    );
    draw_line(
        rect.x + rect.w - inset,
        rect.y + rect.h - inset - len,
        rect.x + rect.w - inset,
        rect.y + rect.h - inset,
        1.3,
        c,
    );
}
