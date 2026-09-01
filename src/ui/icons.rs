//! Vector console glyphs, drawn from primitives so they scale with the UI.

use crate::data::BuildingBoon;
use macroquad::prelude::*;

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
    Pause,
}

/// Pick the glyph that represents a building's headline output, falling back to
/// its risk when it produces nothing.
pub fn icon_for_boon(boon: &BuildingBoon, risk: f32) -> ConsoleIcon {
    if boon.scrap_per_sec > 0.0 {
        ConsoleIcon::Scrap
    } else if boon.food_per_sec > 0.0 {
        ConsoleIcon::Food
    } else if boon.water_per_sec > 0.0 {
        ConsoleIcon::Water
    } else if boon.power_per_sec > 0.0 {
        ConsoleIcon::Power
    } else if risk > 0.0 {
        ConsoleIcon::Risk
    } else {
        ConsoleIcon::Locked
    }
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
        ConsoleIcon::Pause => {
            draw_rectangle(
                cx - size * 0.28,
                y + size * 0.18,
                size * 0.16,
                size * 0.64,
                color,
            );
            draw_rectangle(
                cx + size * 0.12,
                y + size * 0.18,
                size * 0.16,
                size * 0.64,
                color,
            );
        }
    }
}
