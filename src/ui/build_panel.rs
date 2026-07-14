//! Left sidebar: the tower build list and the map glyph legend.

use crate::data::TowerDef;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::math::pulse_range;
use macroquad_toolkit::ui::draw_ui_text;

use super::text::draw_bounded_text;
use super::widgets::draw_button_corners;

/// Per-tower unlock state, parallel to the `tower_defs` slice, so the build
/// panel can show gated towers greyed-out with their unlock requirement.
pub struct TowerButtonInfo {
    pub unlocked: bool,
    /// Human-readable requirement (e.g. "Research Core"); shown when locked.
    pub requirement: String,
}

/// Draw the tower build panel on the left sidebar. Returns the tower id if one
/// was clicked.
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

        if !unlocked {
            let requirement = infos
                .get(i)
                .map(|info| info.requirement.as_str())
                .unwrap_or("");
            btn_y += draw_locked_row(x, btn_y, w, &def.name, requirement) + padding;
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
        let pulse = pulse_range(3.2, 0.55, 0.77);
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

/// Locked towers render as slim reference rows — name + requirement — instead
/// of full-size cards, so the panel doesn't shout about towers the player can't
/// build yet. Returns the row height consumed.
fn draw_locked_row(x: f32, btn_y: f32, w: f32, name: &str, requirement: &str) -> f32 {
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
    draw_bounded_text(name, x + 15.0, btn_y + 14.0, w - 30.0, 12.0, dark::TEXT_DIM);
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
    row_h
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
