//! Map rendering entry points: the circuit-board playfield, towers, shot
//! effects, particles, and enemies.

use crate::data::strings::text;
use crate::engine::enemy::EnemyType;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::GameplayState;
use super::assets::draw_frame;

mod circuit_board;

impl GameplayState {
    pub(super) fn draw_circuit_board(&self) {
        let world_mouse = self.screen_to_world(vec2(mouse_position().0, mouse_position().1));
        let hovered_slot = self.map_state.nearest_slot(world_mouse).map(|(idx, _)| idx);
        let hovered_building = self
            .map_state
            .nearest_building(world_mouse)
            .map(|(idx, _)| idx);
        let max_x = self.map_state.max_visible_x();

        self.draw_pcb_background();
        self.draw_sections_and_corridors();
        self.draw_traces();
        self.draw_paths_and_entrances(max_x);
        self.draw_tower_slots(hovered_slot, max_x);
        self.draw_map_buildings(hovered_building, max_x);
        self.draw_factory_core();
    }

    pub(super) fn draw_towers(&self) {
        for (idx, tower) in self.towers.iter().enumerate() {
            let mut col = tower.color();
            if !tower.is_active {
                col = Color::new(col.r, col.g, col.b, 0.5);
            }
            let outline_alpha = if tower.is_active { 1.0 } else { 0.4 };
            let radius = self.constants.ui.tower_base_radius
                + (tower.level.saturating_sub(1) as f32) * self.constants.ui.tower_level_radius_inc;
            let column = match tower.tower_type {
                crate::data::TowerType::Ballistic => 0,
                crate::data::TowerType::Laser => 1,
                crate::data::TowerType::Emp => 2,
                crate::data::TowerType::AreaDenial => 3,
                crate::data::TowerType::Subversion => 4,
            };
            let level = tower.level.saturating_sub(1).min(2) as usize;
            let frame = level * 5 + column;
            draw_frame(
                &self.assets.towers,
                frame,
                vec2(64.0, 64.0),
                tower.position,
                vec2(radius * 3.0, radius * 3.0),
                col,
            );
            let outline = Color::new(dark::TEXT.r, dark::TEXT.g, dark::TEXT.b, outline_alpha);
            draw_circle_lines(tower.position.x, tower.position.y, radius, 1.5, outline);

            if tower.level > 1 {
                let ring_radius = radius + self.constants.ui.tower_ring_offset;
                draw_circle_lines(tower.position.x, tower.position.y, ring_radius, 1.5, col);
            }

            if self.selected_tower == Some(idx) {
                let range_alpha = if tower.is_active { 0.3 } else { 0.15 };
                draw_circle_lines(
                    tower.position.x,
                    tower.position.y,
                    radius + 8.0,
                    3.0,
                    Color::new(0.45, 0.9, 1.0, 1.0),
                );
                draw_circle_lines(
                    tower.position.x,
                    tower.position.y,
                    self.effective_tower_range(tower.range),
                    2.0,
                    Color::new(col.r, col.g, col.b, range_alpha),
                );
                draw_label_tag(
                    &text().map.selected_tower,
                    tower.position + vec2(20.0, -24.0),
                    Color::new(0.45, 0.9, 1.0, 1.0),
                );
            }
        }
    }

    pub(super) fn draw_shot_effects(&self) {
        for effect in &self.shot_effects {
            let alpha = effect.alpha();
            match effect {
                crate::engine::tower::ShotEffect::Line {
                    from, to, color, ..
                } => {
                    let col = Color::new(color.r, color.g, color.b, alpha);
                    draw_line(from.x, from.y, to.x, to.y, 2.0, col);
                }
                crate::engine::tower::ShotEffect::Pulse {
                    center,
                    radius,
                    color,
                    ..
                } => {
                    let col = Color::new(color.r, color.g, color.b, alpha);
                    draw_circle_lines(center.x, center.y, *radius, 2.0, col);
                }
            }
        }
    }

    pub(super) fn draw_particles(&self) {
        for p in &self.particles {
            let alpha = (p.ttl / self.constants.particles.ttl).clamp(0.0, 1.0);
            let col = Color::new(1.0, 1.0, 1.0, alpha);
            draw_rectangle(
                p.position.x,
                p.position.y,
                self.constants.particles.size,
                self.constants.particles.size,
                col,
            );
        }
    }

    pub(super) fn draw_enemies(&self) {
        for enemy in &self.wave_manager.enemies {
            if !enemy.is_alive {
                continue;
            }

            let (asset, radius, color, frame_size, sprite_size) = match enemy.enemy_type {
                EnemyType::Scout => (0, 5.0, Color::new(0.4, 0.9, 0.4, 1.0), vec2(48.0, 48.0), vec2(38.0, 38.0)),
                EnemyType::Drone => (1, 7.0, Color::new(0.9, 0.5, 0.2, 1.0), vec2(48.0, 48.0), vec2(42.0, 42.0)),
                EnemyType::HeavyUnit => (2, 10.0, Color::new(0.8, 0.2, 0.2, 1.0), vec2(64.0, 64.0), vec2(56.0, 56.0)),
                EnemyType::Saboteur => (3, 6.0, Color::new(0.7, 0.3, 0.9, 1.0), vec2(48.0, 48.0), vec2(38.0, 38.0)),
                EnemyType::Commander => (4, 14.0, Color::new(1.0, 0.8, 0.0, 1.0), vec2(64.0, 64.0), vec2(58.0, 58.0)),
            };

            let mut col = color;
            if enemy.hit_flash_timer > 0.0 {
                col = Color::new(1.0, 1.0, 1.0, 1.0);
            } else if enemy.dodge_timer > 0.0 {
                col = Color::new(color.r, color.g, color.b, 0.4);
            }
            let frame = ((get_time() * 8.0) as usize) % 4;
            draw_frame(&self.assets.enemies[asset], frame, frame_size, enemy.position, sprite_size, col);

            let bar_w = radius * 3.0;
            let bar_h = 3.0;
            let bar_x = enemy.position.x - bar_w / 2.0;
            let bar_y = enemy.position.y - radius - 6.0;
            let fill = enemy.health / enemy.max_health;
            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.3, 0.0, 0.0, 0.8));
            draw_rectangle(bar_x, bar_y, bar_w * fill, bar_h, dark::POSITIVE);
        }
    }
}

fn draw_label_tag(text: &str, pos: Vec2, color: Color) {
    let font_size = 12.0;
    let dims = measure_ui_text(text, None, font_size as u16, 1.0);
    let w = dims.width + 16.0;
    let h = 20.0;
    draw_rectangle(pos.x, pos.y, w, h, Color::new(0.03, 0.05, 0.07, 0.88));
    draw_rectangle_lines(pos.x, pos.y, w, h, 1.5, color);
    draw_ui_text(text, pos.x + 8.0, pos.y + 14.0, font_size, color);
}
