//! UI helpers and reusable components for Last Assembly.

use crate::data::TowerDef;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

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
    draw_text(label, x, y, 17.0, color);
    if !sublabel.is_empty() {
        draw_text(sublabel, x, y + 16.0, 10.0, dark::TEXT_DIM);
    }
}

pub fn draw_status_pill(x: f32, y: f32, label: &str, color: Color) {
    let font_size = 11.0;
    let h = 18.0;
    let w = measure_text(label, None, font_size as u16, 1.0).width + 16.0;
    draw_rectangle(x, y, w, h, Color::new(color.r, color.g, color.b, 0.16));
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(color.r, color.g, color.b, 0.82));
    draw_text(label, x + 8.0, y + 12.5, font_size, color);
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
    let dims = measure_text(&bounded, None, font_size as u16, 1.0);
    draw_text(
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
pub fn draw_build_panel(
    x: f32,
    y: f32,
    w: f32,
    tower_defs: &[TowerDef],
    scrap: f32,
    power: f32,
) -> Option<String> {
    let mut clicked_id = None;

    let sidebar = Rect::new(x, y, w, screen_height() - y);
    let sidebar_surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.1, 0.1, 0.12, 0.9));
    macroquad_toolkit::ui::draw_surface(sidebar, &sidebar_surface);
    draw_text("TOWERS", x + 10.0, y + 20.0, 18.0, dark::ACCENT);
    draw_text(
        "Select, then place on powered pads",
        x + 10.0,
        y + 36.0,
        11.0,
        dark::TEXT_DIM,
    );

    let mut btn_y = y + 54.0;
    let btn_h = 68.0;
    let padding = 8.0;

    for def in tower_defs {
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

    clicked_id
}

fn draw_bounded_text(text: &str, x: f32, y: f32, max_w: f32, font_size: f32, color: Color) {
    let bounded = truncate_text(text, max_w, font_size as u16);
    draw_text(&bounded, x, y, font_size, color);
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
    if measure_text(text, None, font_size, 1.0).width <= max_w {
        return text.to_string();
    }

    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}...", out, ch);
        if measure_text(&candidate, None, font_size, 1.0).width > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{}...", out)
}
