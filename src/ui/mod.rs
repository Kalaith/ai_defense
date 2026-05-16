//! UI helpers and reusable components for Last Assembly.

use crate::data::TowerDef;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

pub fn draw_resource_bar(x: f32, y: f32, w: f32, h: f32, current: f32, max: f32, color: Color) {
    draw_rectangle(x, y, w, h, dark::PANEL);
    let fill = (current / max).clamp(0.0, 1.0);
    draw_rectangle(x, y, w * fill, h, color);
    draw_rectangle_lines(x, y, w, h, 1.0, dark::TEXT_DIM);
}

/// Draw tower build panel on the left sidebar. Returns tower id if one was clicked.
pub fn draw_build_panel(x: f32, y: f32, w: f32, tower_defs: &[TowerDef], scrap: f32, power: f32) -> Option<String> {
    let mut clicked_id = None;

    draw_rectangle(x, y, w, screen_height() - y, Color::new(0.1, 0.1, 0.12, 0.9));
    draw_text("BUILD", x + 10.0, y + 20.0, 18.0, dark::ACCENT);

    let mut btn_y = y + 35.0;
    let btn_h = 50.0;
    let padding = 5.0;

    for def in tower_defs {
        let can_afford = scrap >= def.cost_scrap && power >= def.cost_power;
        let label = format!("{} ({}s/{}p)", def.name, def.cost_scrap as i32, def.cost_power as i32);

        let color = if can_afford {
            dark::ACCENT
        } else {
            dark::TEXT_DIM
        };

        draw_rectangle(x + 5.0, btn_y, w - 10.0, btn_h, dark::PANEL);
        draw_rectangle_lines(x + 5.0, btn_y, w - 10.0, btn_h, 1.0, color);
        draw_text(&label, x + 10.0, btn_y + 20.0, 14.0, color);
        draw_text(&def.description, x + 10.0, btn_y + 38.0, 11.0, dark::TEXT_DIM);

        if can_afford {
            let (mx, my) = mouse_position();
            if mx >= x + 5.0 && mx <= x + w - 5.0 && my >= btn_y && my <= btn_y + btn_h {
                draw_rectangle(x + 5.0, btn_y, w - 10.0, btn_h, Color::new(0.3, 0.4, 0.6, 0.2));
                if is_mouse_button_pressed(MouseButton::Left) {
                    clicked_id = Some(def.id.clone());
                }
            }
        }

        btn_y += btn_h + padding;
    }

    clicked_id
}

