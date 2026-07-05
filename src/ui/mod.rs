//! UI helpers and reusable components for Last Assembly.

use crate::data::TowerDef;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleButtonState {
    Disabled,
    Affordable,
    Dangerous,
    Recommended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleIcon {
    Scrap,
    Water,
    Food,
    Power,
    Risk,
    Locked,
    Tower,
    People,
    Morale,
    Health,
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
    let pulse = 0.55 + 0.22 * (get_time() as f32 * 3.2).sin().abs();
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
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

pub fn draw_icon(icon: ConsoleIcon, x: f32, y: f32, size: f32, color: Color) {
    let cx = x + size * 0.5;
    let cy = y + size * 0.5;
    match icon {
        ConsoleIcon::Scrap => {
            draw_circle_lines(cx, cy, size * 0.28, 2.0, color);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let inner = vec2(cx + a.cos() * size * 0.29, cy + a.sin() * size * 0.29);
                let outer = vec2(cx + a.cos() * size * 0.42, cy + a.sin() * size * 0.42);
                draw_line(inner.x, inner.y, outer.x, outer.y, 2.0, color);
            }
        }
        ConsoleIcon::Water => {
            draw_circle(
                cx,
                cy + size * 0.14,
                size * 0.25,
                Color::new(color.r, color.g, color.b, 0.35),
            );
            draw_triangle(
                vec2(cx, y + size * 0.12),
                vec2(cx - size * 0.22, cy + size * 0.1),
                vec2(cx + size * 0.22, cy + size * 0.1),
                Color::new(color.r, color.g, color.b, 0.55),
            );
            draw_circle_lines(cx, cy + size * 0.14, size * 0.25, 1.5, color);
        }
        ConsoleIcon::Food => {
            draw_line(
                cx - size * 0.12,
                y + size * 0.18,
                cx - size * 0.12,
                y + size * 0.82,
                2.0,
                color,
            );
            draw_line(
                cx + size * 0.1,
                y + size * 0.22,
                cx + size * 0.1,
                y + size * 0.82,
                2.0,
                color,
            );
            draw_line(
                cx - size * 0.22,
                y + size * 0.18,
                cx - size * 0.02,
                y + size * 0.18,
                2.0,
                color,
            );
        }
        ConsoleIcon::Power => {
            let p1 = vec2(cx + size * 0.08, y + size * 0.1);
            let p2 = vec2(cx - size * 0.2, cy + size * 0.08);
            let p3 = vec2(cx + size * 0.02, cy + size * 0.08);
            let p4 = vec2(cx - size * 0.08, y + size * 0.9);
            let p5 = vec2(cx + size * 0.22, cy - size * 0.08);
            let p6 = vec2(cx, cy - size * 0.08);
            draw_triangle(p1, p2, p3, color);
            draw_triangle(p4, p5, p6, color);
        }
        ConsoleIcon::Risk => {
            draw_circle_lines(cx - size * 0.22, cy, size * 0.08, 2.0, color);
            for i in 1..=3 {
                draw_circle_lines(
                    cx - size * 0.22,
                    cy,
                    size * (0.12 + i as f32 * 0.1),
                    1.5,
                    Color::new(color.r, color.g, color.b, 0.55),
                );
            }
        }
        ConsoleIcon::Locked => {
            draw_rectangle_lines(
                x + size * 0.25,
                y + size * 0.44,
                size * 0.5,
                size * 0.34,
                2.0,
                color,
            );
            draw_circle_lines(cx, y + size * 0.45, size * 0.2, 2.0, color);
        }
        ConsoleIcon::Tower => {
            draw_rectangle(
                x + size * 0.28,
                y + size * 0.58,
                size * 0.44,
                size * 0.2,
                Color::new(color.r, color.g, color.b, 0.35),
            );
            draw_circle_lines(cx, cy, size * 0.22, 2.0, color);
            draw_line(cx, cy, x + size * 0.86, y + size * 0.28, 3.0, color);
        }
        ConsoleIcon::People => {
            draw_circle(cx - size * 0.14, cy - size * 0.08, size * 0.1, color);
            draw_circle(cx + size * 0.14, cy - size * 0.08, size * 0.1, color);
            draw_rectangle(
                cx - size * 0.26,
                cy + size * 0.06,
                size * 0.52,
                size * 0.24,
                Color::new(color.r, color.g, color.b, 0.5),
            );
        }
        ConsoleIcon::Morale => {
            draw_circle_lines(cx, cy, size * 0.34, 2.0, color);
            draw_circle(cx - size * 0.12, cy - size * 0.08, size * 0.03, color);
            draw_circle(cx + size * 0.12, cy - size * 0.08, size * 0.03, color);
            draw_line(
                cx - size * 0.14,
                cy + size * 0.12,
                cx + size * 0.14,
                cy + size * 0.12,
                2.0,
                color,
            );
        }
        ConsoleIcon::Health => {
            draw_rectangle(
                cx - size * 0.08,
                y + size * 0.18,
                size * 0.16,
                size * 0.64,
                color,
            );
            draw_rectangle(
                x + size * 0.18,
                cy - size * 0.08,
                size * 0.64,
                size * 0.16,
                color,
            );
        }
    }
}

/// Draw tower build panel on the left sidebar. Returns tower id if one was clicked.
/// Per-tower unlock state, parallel to the `tower_defs` slice, so the build
/// panel can show gated towers greyed-out with their unlock requirement.
pub struct TowerButtonInfo {
    pub unlocked: bool,
    /// Human-readable requirement (e.g. "Research Core"); shown when locked.
    pub requirement: String,
}

pub fn draw_build_panel(
    x: f32,
    y: f32,
    w: f32,
    tower_defs: &[TowerDef],
    infos: &[TowerButtonInfo],
    scrap: f32,
    power: f32,
) -> Option<String> {
    let mut clicked_id = None;

    let sidebar = Rect::new(x, y, w, screen_height() - y);
    let sidebar_surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.1, 0.1, 0.12, 0.9));
    macroquad_toolkit::ui::draw_surface(sidebar, &sidebar_surface);
    draw_ui_text("TOWERS", x + 10.0, y + 20.0, 18.0, dark::ACCENT);
    draw_ui_text(
        "Select, then place on powered pads",
        x + 10.0,
        y + 36.0,
        11.0,
        dark::TEXT_DIM,
    );

    let mut btn_y = y + 54.0;
    let btn_h = 68.0;
    let padding = 8.0;

    for (i, def) in tower_defs.iter().enumerate() {
        let unlocked = infos.get(i).map(|info| info.unlocked).unwrap_or(true);

        // Locked towers render as slim reference rows — name + requirement —
        // instead of full-size cards, so the panel doesn't shout about towers
        // the player can't build yet.
        if !unlocked {
            let row_h = 34.0;
            let row = Rect::new(x + 5.0, btn_y, w - 10.0, row_h);
            draw_rectangle(
                row.x,
                row.y,
                row.w,
                row.h,
                Color::new(0.02, 0.03, 0.04, 0.5),
            );
            draw_rectangle(row.x, row.y, 3.0, row.h, Color::new(0.28, 0.32, 0.34, 0.4));
            draw_bounded_text(
                &def.name,
                x + 15.0,
                btn_y + 14.0,
                w - 30.0,
                12.0,
                dark::TEXT_DIM,
            );
            let requirement = infos
                .get(i)
                .map(|info| info.requirement.as_str())
                .unwrap_or("");
            let req_line = if requirement.is_empty() {
                "Locked".to_string()
            } else {
                format!("Needs {}", requirement)
            };
            draw_bounded_text(
                &req_line,
                x + 15.0,
                btn_y + 27.0,
                w - 30.0,
                10.0,
                Color::new(0.42, 0.5, 0.52, 0.8),
            );
            btn_y += row_h + padding;
            continue;
        }

        let can_afford = scrap >= def.cost_scrap && power >= def.cost_power;
        let label = format!(
            "{} ({}s/{}p)",
            def.name, def.cost_scrap as i32, def.cost_power as i32
        );

        let color = if can_afford {
            dark::ACCENT
        } else {
            dark::TEXT_DIM
        };

        let button_rect = Rect::new(x + 5.0, btn_y, w - 10.0, btn_h);
        let (mx, my) = mouse_position();
        let hovered = mx >= button_rect.x
            && mx <= button_rect.x + button_rect.w
            && my >= btn_y
            && my <= btn_y + btn_h;
        let pulse = 0.55 + 0.22 * (get_time() as f32 * 3.2).sin().abs();
        let accent = if can_afford {
            Color::new(0.28, 0.74, 1.0, 0.78 + pulse * 0.08)
        } else {
            Color::new(0.28, 0.32, 0.34, 0.55)
        };
        let fill = if hovered && can_afford {
            Color::new(0.09, 0.12, 0.14, 0.82)
        } else {
            Color::new(0.02, 0.03, 0.04, 0.68)
        };
        draw_rectangle(
            button_rect.x,
            button_rect.y,
            button_rect.w,
            button_rect.h,
            fill,
        );
        draw_rectangle_lines(
            button_rect.x,
            button_rect.y,
            button_rect.w,
            button_rect.h,
            1.8,
            accent,
        );
        draw_rectangle(
            button_rect.x,
            button_rect.y,
            4.0,
            button_rect.h,
            Color::new(
                accent.r,
                accent.g,
                accent.b,
                if can_afford { 0.78 } else { 0.24 },
            ),
        );
        draw_line(
            button_rect.x,
            button_rect.y + button_rect.h - 2.0,
            button_rect.x + button_rect.w,
            button_rect.y + button_rect.h - 2.0,
            2.0,
            Color::new(
                accent.r,
                accent.g,
                accent.b,
                if can_afford { 0.58 } else { 0.22 },
            ),
        );
        if hovered && can_afford {
            draw_rectangle_lines(
                button_rect.x - 2.0,
                button_rect.y - 2.0,
                button_rect.w + 4.0,
                button_rect.h + 4.0,
                1.0,
                Color::new(accent.r, accent.g, accent.b, 0.42),
            );
        }
        draw_button_corners(button_rect, accent);
        draw_bounded_text(&label, x + 15.0, btn_y + 23.0, w - 50.0, 15.0, color);
        let afford_line = if can_afford {
            "Affordable".to_string()
        } else if scrap < def.cost_scrap {
            format!("Need {:.0} more scrap", def.cost_scrap - scrap)
        } else {
            format!("Need {:.0} power buffer", def.cost_power - power)
        };
        draw_bounded_text(
            &afford_line,
            x + 12.0,
            btn_y + 40.0,
            w - 24.0,
            11.0,
            if can_afford {
                dark::POSITIVE
            } else {
                dark::WARNING
            },
        );
        draw_bounded_text(
            &def.description,
            x + 12.0,
            btn_y + 57.0,
            w - 24.0,
            10.0,
            dark::TEXT_DIM,
        );

        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            clicked_id = Some(def.id.clone());
        }

        btn_y += btn_h + padding;
    }

    draw_map_key(x, btn_y, w);

    clicked_id
}

/// Legend for the map's glyph vocabulary, anchored to the bottom of the build
/// panel's otherwise-empty space. New players had no way to know what the
/// circles, crosses, and boxes on the circuit board meant.
fn draw_map_key(x: f32, list_bottom: f32, w: f32) {
    const ROW_H: f32 = 22.0;
    const ROWS: usize = 6;
    let key_h = 30.0 + ROWS as f32 * ROW_H;
    let key_y = screen_height() - key_h - 10.0;
    // Skip the legend entirely if the tower list already fills the panel
    // (small windows) — never overlap real controls.
    if key_y < list_bottom + 8.0 {
        return;
    }

    draw_line(
        x + 10.0,
        key_y,
        x + w - 10.0,
        key_y,
        1.0,
        Color::new(0.25, 0.35, 0.38, 0.6),
    );
    draw_ui_text("MAP KEY", x + 10.0, key_y + 18.0, 12.0, dark::TEXT_DIM);

    let glyph_x = x + 19.0;
    let text_x = x + 36.0;
    let mut row_y = key_y + 30.0;
    let label = |text: &str, y: f32| {
        draw_bounded_text(
            text,
            text_x,
            y + 5.0,
            w - (text_x - x) - 10.0,
            11.0,
            dark::TEXT_DIM,
        );
    };

    // Powered pad: green ring + plus (same glyph as the map).
    let cy = row_y + ROW_H * 0.5 - 4.0;
    draw_circle(glyph_x, cy, 7.0, Color::new(0.04, 0.2, 0.08, 0.9));
    draw_circle_lines(glyph_x, cy, 9.0, 1.8, Color::new(0.25, 1.0, 0.45, 0.8));
    draw_line(
        glyph_x - 3.5,
        cy,
        glyph_x + 3.5,
        cy,
        1.6,
        Color::new(0.5, 1.0, 0.6, 0.8),
    );
    draw_line(
        glyph_x,
        cy - 3.5,
        glyph_x,
        cy + 3.5,
        1.6,
        Color::new(0.5, 1.0, 0.6, 0.8),
    );
    label("Powered pad — build towers here", cy);
    row_y += ROW_H;

    // Unpowered pad: blue ring + dot.
    let cy = row_y + ROW_H * 0.5 - 4.0;
    draw_circle(glyph_x, cy, 7.0, Color::new(0.05, 0.12, 0.2, 0.75));
    draw_circle_lines(glyph_x, cy, 9.0, 1.8, Color::new(0.32, 0.68, 1.0, 0.82));
    draw_circle(glyph_x, cy, 2.4, Color::new(0.45, 0.82, 1.0, 0.7));
    label("Unpowered pad — pay to power", cy);
    row_y += ROW_H;

    // Debris: brown circle + orange cross.
    let cy = row_y + ROW_H * 0.5 - 4.0;
    draw_circle(glyph_x, cy, 7.0, Color::new(0.28, 0.17, 0.08, 0.86));
    draw_circle_lines(glyph_x, cy, 9.0, 1.6, Color::new(0.85, 0.52, 0.16, 0.78));
    let s = 3.4;
    draw_line(
        glyph_x - s,
        cy - s,
        glyph_x + s,
        cy + s,
        1.6,
        Color::new(1.0, 0.66, 0.2, 0.68),
    );
    draw_line(
        glyph_x + s,
        cy - s,
        glyph_x - s,
        cy + s,
        1.6,
        Color::new(1.0, 0.66, 0.2, 0.68),
    );
    label("Debris — clear for a new pad", cy);
    row_y += ROW_H;

    // Machine: small box.
    let cy = row_y + ROW_H * 0.5 - 4.0;
    draw_rectangle(
        glyph_x - 8.0,
        cy - 5.5,
        16.0,
        11.0,
        Color::new(0.28, 0.05, 0.05, 0.9),
    );
    draw_rectangle_lines(
        glyph_x - 8.0,
        cy - 5.5,
        16.0,
        11.0,
        1.6,
        Color::new(0.78, 0.18, 0.12, 0.86),
    );
    label("Machine — repair, then power", cy);
    row_y += ROW_H;

    // Enemy entrance: red pulse circle.
    let cy = row_y + ROW_H * 0.5 - 4.0;
    draw_circle(glyph_x, cy, 6.0, Color::new(0.92, 0.18, 0.08, 0.88));
    draw_circle_lines(glyph_x, cy, 9.0, 1.8, Color::new(1.0, 0.46, 0.12, 0.7));
    label("Enemy entrance", cy);
    row_y += ROW_H;

    // Route: orange line with a chevron.
    let cy = row_y + ROW_H * 0.5 - 4.0;
    draw_line(
        glyph_x - 9.0,
        cy,
        glyph_x + 9.0,
        cy,
        4.0,
        Color::new(1.0, 0.56, 0.12, 0.9),
    );
    draw_triangle(
        vec2(glyph_x + 9.0, cy),
        vec2(glyph_x + 3.0, cy - 4.0),
        vec2(glyph_x + 3.0, cy + 4.0),
        Color::new(1.0, 0.78, 0.22, 0.95),
    );
    label("Attack route to your core", cy);
}

fn draw_bounded_text(text: &str, x: f32, y: f32, max_w: f32, font_size: f32, color: Color) {
    let bounded = truncate_text(text, max_w, font_size as u16);
    draw_ui_text(&bounded, x, y, font_size, color);
}

fn draw_button_corners(rect: Rect, color: Color) {
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

fn truncate_text(text: &str, max_w: f32, font_size: u16) -> String {
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

/// Result of a settings-overlay frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsAction {
    None,
    Close,
}

/// Shared modal settings overlay, reused by the main menu and the in-game pause
/// menu. Mutates and persists `settings` in place; returns `Close` when the
/// player dismisses it.
pub fn draw_settings_overlay(settings: &mut crate::save::Settings) -> SettingsAction {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.7));

    let pw = 480.0_f32.min(sw - 40.0);
    let ph = 430.0_f32.min(sh - 40.0);
    let px = (sw - pw) * 0.5;
    let py = (sh - ph) * 0.5;
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.98))
        .with_border(1.6, Color::new(0.24, 0.58, 0.62, 0.9));
    macroquad_toolkit::ui::draw_surface(Rect::new(px, py, pw, ph), &surface);

    draw_ui_text("SETTINGS", px + 24.0, py + 42.0, 26.0, dark::ACCENT);

    let row_x = px + 24.0;
    let row_w = pw - 48.0;
    let row_h = 34.0;
    let gap = 14.0;
    let mut y = py + 74.0;
    let mut changed = false;

    if toggle_row(row_x, y, row_w, row_h, "Autosave", settings.autosave) {
        settings.autosave = !settings.autosave;
        changed = true;
    }
    y += row_h + gap;
    if toggle_row(
        row_x,
        y,
        row_w,
        row_h,
        "Start runs at 2x speed",
        settings.default_fast_speed,
    ) {
        settings.default_fast_speed = !settings.default_fast_speed;
        changed = true;
    }
    y += row_h + gap;
    if let Some(v) = slider_row(
        row_x,
        y,
        row_w,
        row_h,
        "Master volume",
        settings.master_volume,
    ) {
        settings.master_volume = v;
        changed = true;
    }
    y += row_h + gap;
    if let Some(v) = slider_row(row_x, y, row_w, row_h, "SFX volume", settings.sfx_volume) {
        settings.sfx_volume = v;
        changed = true;
    }
    y += row_h + gap;
    let tut_label = if settings.tutorial_seen {
        "Replay tutorial on next new run"
    } else {
        "Tutorial will show on next new run"
    };
    if macroquad_toolkit::ui::button(row_x, y, row_w, row_h, tut_label) && settings.tutorial_seen {
        settings.tutorial_seen = false;
        changed = true;
    }

    if changed {
        let _ = settings.save();
    }

    let close_w = 150.0;
    if macroquad_toolkit::ui::button(
        px + (pw - close_w) * 0.5,
        py + ph - 48.0,
        close_w,
        34.0,
        "Close",
    ) {
        return SettingsAction::Close;
    }
    SettingsAction::None
}

fn toggle_row(x: f32, y: f32, w: f32, h: f32, label: &str, on: bool) -> bool {
    draw_ui_text(label, x + 4.0, y + h * 0.5 + 5.0, 15.0, dark::TEXT);
    let bw = 92.0;
    macroquad_toolkit::ui::button(x + w - bw, y, bw, h, if on { "On" } else { "Off" })
}

fn slider_row(x: f32, y: f32, w: f32, h: f32, label: &str, value: f32) -> Option<f32> {
    draw_ui_text(label, x + 4.0, y + h * 0.5 + 5.0, 15.0, dark::TEXT);
    let bw = 34.0;
    let val_w = 60.0;
    let mut out = None;
    let minus_x = x + w - bw * 2.0 - val_w;
    if macroquad_toolkit::ui::button(minus_x, y, bw, h, "-") {
        out = Some((value - 0.1).clamp(0.0, 1.0));
    }
    let pct = format!("{:.0}%", value * 100.0);
    let dims = measure_ui_text(&pct, None, 15, 1.0);
    draw_ui_text(
        &pct,
        minus_x + bw + (val_w - dims.width) * 0.5,
        y + h * 0.5 + 5.0,
        15.0,
        dark::TEXT_BRIGHT,
    );
    let plus_x = x + w - bw;
    if macroquad_toolkit::ui::button(plus_x, y, bw, h, "+") {
        out = Some((value + 0.1).clamp(0.0, 1.0));
    }
    out
}
