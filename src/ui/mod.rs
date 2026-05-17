//! UI helpers and reusable components for Last Assembly.

use crate::data::TowerDef;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

pub fn draw_resource_bar(x: f32, y: f32, w: f32, h: f32, current: f32, max: f32, color: Color) {
    macroquad_toolkit::ui::progress_bar(x, y, w, h, current, max, color);
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
    draw_text("BUILD", x + 10.0, y + 20.0, 18.0, dark::ACCENT);

    let mut btn_y = y + 35.0;
    let btn_h = 50.0;
    let padding = 5.0;

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
        let button_surface =
            macroquad_toolkit::ui::SurfaceStyle::new(dark::PANEL).with_border(1.0, color);
        macroquad_toolkit::ui::draw_surface(button_rect, &button_surface);
        draw_text(&label, x + 10.0, btn_y + 20.0, 14.0, color);
        draw_text(
            &def.description,
            x + 10.0,
            btn_y + 38.0,
            11.0,
            dark::TEXT_DIM,
        );

        if can_afford {
            let (mx, my) = mouse_position();
            if mx >= x + 5.0 && mx <= x + w - 5.0 && my >= btn_y && my <= btn_y + btn_h {
                let hover_surface =
                    macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.3, 0.4, 0.6, 0.2));
                macroquad_toolkit::ui::draw_surface(button_rect, &hover_surface);
                if is_mouse_button_pressed(MouseButton::Left) {
                    clicked_id = Some(def.id.clone());
                }
            }
        }

        btn_y += btn_h + padding;
    }

    clicked_id
}
