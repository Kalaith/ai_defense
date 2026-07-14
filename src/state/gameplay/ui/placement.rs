//! World-space tower placement ghost: valid pads, blocked pads, and the range
//! and route coverage of the pad under the cursor.

use crate::data::GameData;
use crate::state::gameplay::GameplayState;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::math::pulse_range;
use macroquad_toolkit::ui::draw_ui_text;

impl GameplayState {
    pub fn draw_placement_ghost(&self, data: &GameData) {
        let world_mouse = self.screen_to_world(vec2(mouse_position().0, mouse_position().1));
        let Some(preview) = self.placement_preview(data, world_mouse) else {
            return;
        };
        let Some(def) = data.tower_def_by_id(&preview.tower_id) else {
            return;
        };

        let range = self.effective_tower_range(def.base_range);
        let c = def.color();

        for path in self.map_state.active_paths_limited() {
            for pair in path.points.windows(2) {
                draw_line(
                    pair[0].x,
                    pair[0].y,
                    pair[1].x,
                    pair[1].y,
                    9.0,
                    Color::new(1.0, 0.48, 0.08, 0.24),
                );
                draw_line(
                    pair[0].x,
                    pair[0].y,
                    pair[1].x,
                    pair[1].y,
                    4.0,
                    Color::new(1.0, 0.7, 0.22, 0.8),
                );
            }
        }

        for idx in preview.valid_slots {
            if let Some(slot) = self.map_state.slots.get(idx) {
                let pulse = pulse_range(5.0, 0.55, 0.83);
                draw_circle(
                    slot.position.x,
                    slot.position.y,
                    19.0,
                    Color::new(0.06, 0.25, 0.22, 0.34),
                );
                draw_circle_lines(
                    slot.position.x,
                    slot.position.y,
                    23.0,
                    3.0,
                    Color::new(0.26, 0.95, 0.82, pulse),
                );
            }
        }

        for (idx, reason) in preview.invalid_slots {
            if let Some(slot) = self.map_state.slots.get(idx) {
                draw_circle(
                    slot.position.x,
                    slot.position.y,
                    17.0,
                    Color::new(0.02, 0.025, 0.03, 0.58),
                );
                draw_circle_lines(
                    slot.position.x,
                    slot.position.y,
                    18.0,
                    1.5,
                    Color::new(0.45, 0.48, 0.5, 0.32),
                );
                if preview.hovered_slot == Some(idx) {
                    draw_ui_text(
                        reason.label(),
                        slot.position.x + 18.0,
                        slot.position.y + 4.0,
                        12.0,
                        dark::TEXT_DIM,
                    );
                }
            }
        }

        let Some(idx) = preview.hovered_slot else {
            return;
        };
        let Some(slot) = self.map_state.slots.get(idx) else {
            return;
        };
        let ghost_color = Color::new(c.r, c.g, c.b, 0.5);
        let range_color = Color::new(c.r, c.g, c.b, 0.32);
        draw_circle(
            slot.position.x,
            slot.position.y,
            self.constants.ui.tower_base_radius,
            ghost_color,
        );
        draw_circle_lines(slot.position.x, slot.position.y, range, 2.0, range_color);
        for path_id in &preview.covered_paths {
            if let Some(path) = self.map_state.paths.iter().find(|p| p.id == *path_id) {
                for pair in path.points.windows(2) {
                    draw_line(
                        pair[0].x,
                        pair[0].y,
                        pair[1].x,
                        pair[1].y,
                        6.0,
                        Color::new(c.r, c.g, c.b, 0.42),
                    );
                }
            }
        }
    }
}
