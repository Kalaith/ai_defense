//! UI helpers and reusable components for Last Assembly.

use crate::data::TowerDef;
use crate::engine::factory::Factory;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::button;

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

/// Draw factory sector panel on the right sidebar. Returns sector id if unlock was clicked.
pub enum SectorPanelAction {
    Unlock(String),
    Repair(String),
}

pub fn draw_sector_panel(
    x: f32,
    y: f32,
    w: f32,
    factory: &Factory,
    scrap: f32,
    power_generation: f32,
) -> Option<SectorPanelAction> {
    let mut action = None;
    let sectors = &factory.sectors;

    draw_rectangle(x, y, w, screen_height() - y, Color::new(0.1, 0.1, 0.12, 0.9));
    draw_text("FACTORY", x + 10.0, y + 20.0, 18.0, dark::WARNING);

    let diff = factory.difficulty_modifier;
    if diff > 0.0 {
        draw_text(
            &format!("Difficulty: +{:.0}", diff),
            x + 80.0,
            y + 20.0,
            14.0,
            dark::NEGATIVE,
        );
    }

    let mut row_y = y + 35.0;
    let row_h = 45.0;
    let padding = 4.0;

    for sector in sectors {
        let is_disabled = sector.unlocked && sector.integrity <= 0.0;
        let border_color = if is_disabled {
            dark::NEGATIVE
        } else if sector.unlocked {
            dark::POSITIVE
        } else {
            dark::TEXT_DIM
        };

        draw_rectangle(x + 5.0, row_y, w - 10.0, row_h, dark::PANEL);
        draw_rectangle_lines(x + 5.0, row_y, w - 10.0, row_h, 1.0, border_color);

        if sector.unlocked {
            let integrity_pct = (sector.integrity / sector.max_integrity) * 100.0;
            let label = if sector.id == "power_core" {
                format!("{} (+{:.0}/s)", sector.name, power_generation)
            } else {
                format!("{}", sector.name)
            };

            draw_text(&label, x + 10.0, row_y + 16.0, 14.0, dark::TEXT);

            if is_disabled {
                draw_text("OFFLINE", x + 10.0, row_y + 30.0, 12.0, dark::NEGATIVE);
            } else {
                draw_text(&format!("{:.0}%", integrity_pct), x + 10.0, row_y + 30.0, 12.0, dark::TEXT_DIM);
            }

            // Integrity bar
            let bar_w = w - 20.0;
            let bar_color = if is_disabled { dark::NEGATIVE } else { dark::POSITIVE };
            draw_resource_bar(x + 10.0, row_y + 33.0, bar_w, 8.0, sector.integrity, sector.max_integrity, bar_color);

            if sector.integrity < sector.max_integrity {
                let repair_cost = 20;
                let can_afford = scrap >= repair_cost as f32;
                let btn_label = format!("Repair ({})", repair_cost);
                let btn_w = 80.0;
                let btn_h = 22.0;
                let btn_x = x + w - btn_w - 10.0;
                let btn_y = row_y + 10.0;
                if can_afford {
                    if button(btn_x, btn_y, btn_w, btn_h, &btn_label) {
                        action = Some(SectorPanelAction::Repair(sector.id.clone()));
                    }
                } else {
                    draw_rectangle(btn_x, btn_y, btn_w, btn_h, Color::new(0.2, 0.2, 0.2, 0.5));
                    let dims = measure_text(&btn_label, None, 12, 1.0);
                    draw_text(&btn_label, btn_x + (btn_w - dims.width) / 2.0, btn_y + 16.0, 12.0, dark::TEXT_DIM);
                }
            }
        } else {
            let cost = sector.unlock_cost as i32;
            let can_afford = scrap >= sector.unlock_cost;
            let label = format!("{}", sector.name);
            draw_text(&label, x + 10.0, row_y + 18.0, 14.0, dark::TEXT_DIM);

            let btn_label = format!("Unlock ({})", cost);
            let btn_x = x + w - 90.0;
            let btn_y_inner = row_y + 8.0;
            let btn_w = 80.0;
            let btn_h = 28.0;

            if can_afford {
                if button(btn_x, btn_y_inner, btn_w, btn_h, &btn_label) {
                    action = Some(SectorPanelAction::Unlock(sector.id.clone()));
                }
            } else {
                draw_rectangle(btn_x, btn_y_inner, btn_w, btn_h, Color::new(0.2, 0.2, 0.2, 0.5));
                let dims = measure_text(&btn_label, None, 12, 1.0);
                draw_text(&btn_label, btn_x + (btn_w - dims.width) / 2.0, btn_y_inner + 18.0, 12.0, dark::TEXT_DIM);
            }
        }

        row_y += row_h + padding;
    }

    action
}
