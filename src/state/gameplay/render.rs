use crate::data::GameData;
use crate::engine::enemy::EnemyType;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use super::helpers::{beacon_color, enemy_label, tower_label};
use super::GameplayState;

impl GameplayState {
    pub fn draw(&mut self, data: &GameData) {
        self.draw_grid();
        self.draw_path();
        self.draw_towers();
        self.draw_enemies();
        self.draw_shot_effects();
        self.draw_particles();
        self.draw_placement_ghost(data);
        self.draw_hud();
        self.draw_wave_start_flash();
        self.draw_wave_status();
        self.draw_notifications();
        self.draw_shutdown_ui();
        self.draw_build_panel(data);
        self.draw_sector_panel();
        self.handle_placement_click(data);
        self.handle_selection_click();
        self.draw_selected_tower_panel(data);
    }

    fn draw_grid(&self) {
        let grid_color = Color::new(0.15, 0.15, 0.18, 1.0);
        let step = self.constants.placement.grid;
        let mut x = 0.0;
        while x < screen_width() {
            draw_line(x, 0.0, x, screen_height(), 1.0, grid_color);
            x += step;
        }
        let mut y = 0.0;
        while y < screen_height() {
            draw_line(0.0, y, screen_width(), y, 1.0, grid_color);
            y += step;
        }
    }

    fn draw_path(&self) {
        if self.map_path.len() < 2 {
            return;
        }

        for i in 0..self.map_path.len() - 1 {
            let a = self.map_path[i];
            let b = self.map_path[i + 1];
            draw_line(a.x, a.y, b.x, b.y, 6.0, Color::new(0.3, 0.25, 0.2, 0.8));
            draw_line(a.x, a.y, b.x, b.y, 2.0, Color::new(0.5, 0.45, 0.35, 0.6));
        }

        for (i, p) in self.map_path.iter().enumerate() {
            let color = if i == 0 {
                dark::POSITIVE
            } else if i == self.map_path.len() - 1 {
                dark::NEGATIVE
            } else {
                dark::TEXT_DIM
            };
            draw_circle(p.x, p.y, 4.0, color);
        }
    }

    fn draw_towers(&self) {
        let (mx, my) = mouse_position();
        let range_mult = if self.factory.is_sector_active("ai_vault") { 1.2 } else { 1.0 };
        for tower in &self.towers {
            let mut col = tower.color();
            if !tower.is_active {
                col = Color::new(col.r, col.g, col.b, 0.5);
            }
            let outline_alpha = if tower.is_active { 1.0 } else { 0.4 };
            let radius = self.constants.ui.tower_base_radius
                + (tower.level.saturating_sub(1) as f32) * self.constants.ui.tower_level_radius_inc;
            draw_circle(tower.position.x, tower.position.y, radius, col);
            let outline = Color::new(dark::TEXT.r, dark::TEXT.g, dark::TEXT.b, outline_alpha);
            draw_circle_lines(tower.position.x, tower.position.y, radius, 2.0, outline);

            if tower.level > 1 {
                let ring_radius = radius + self.constants.ui.tower_ring_offset;
                draw_circle_lines(tower.position.x, tower.position.y, ring_radius, 1.5, col);
            }

            let dist = ((mx - tower.position.x).powi(2) + (my - tower.position.y).powi(2)).sqrt();
            if dist < self.constants.ui.tower_hover_dist {
                let range_alpha = if tower.is_active { 0.3 } else { 0.15 };
                draw_circle_lines(
                    tower.position.x,
                    tower.position.y,
                    tower.range * range_mult,
                    1.0,
                    Color::new(col.r, col.g, col.b, range_alpha),
                );
                self.draw_tower_tooltip(tower, range_mult, mx, my);
            }
        }
    }

    fn draw_tower_tooltip(&self, tower: &crate::engine::tower::Tower, range_mult: f32, mx: f32, my: f32) {
        let tooltip_x = mx + 12.0;
        let tooltip_y = my + 12.0;
        let lines = [
            format!("{} (Lv {})", tower_label(&tower.tower_type), tower.level),
            format!("Dmg: {:.1}", tower.damage),
            format!("Rng: {:.0}", tower.range * range_mult),
            format!("Rate: {:.2}/s", tower.fire_rate),
        ];
        let width = self.constants.ui.tooltip_w;
        let height = self.constants.ui.tooltip_h;
        draw_rectangle(tooltip_x, tooltip_y, width, height, Color::new(0.05, 0.05, 0.08, 0.9));
        draw_rectangle_lines(tooltip_x, tooltip_y, width, height, 1.0, dark::TEXT_DIM);
        for (i, line) in lines.iter().enumerate() {
            draw_text(line, tooltip_x + 6.0, tooltip_y + 14.0 + i as f32 * 12.0, 11.0, dark::TEXT);
        }
    }

    fn draw_shot_effects(&self) {
        for effect in &self.shot_effects {
            let alpha = effect.alpha();
            match effect {
                crate::engine::tower::ShotEffect::Line { from, to, color, .. } => {
                    let col = Color::new(color.r, color.g, color.b, alpha);
                    draw_line(from.x, from.y, to.x, to.y, 2.0, col);
                }
                crate::engine::tower::ShotEffect::Pulse { center, radius, color, .. } => {
                    let col = Color::new(color.r, color.g, color.b, alpha);
                    draw_circle_lines(center.x, center.y, *radius, 2.0, col);
                }
            }
        }
    }

    fn draw_particles(&self) {
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

    fn draw_enemies(&self) {
        for enemy in &self.wave_manager.enemies {
            if !enemy.is_alive {
                continue;
            }

            let (radius, color) = match enemy.enemy_type {
                EnemyType::Scout => (5.0, Color::new(0.4, 0.9, 0.4, 1.0)),
                EnemyType::Drone => (7.0, Color::new(0.9, 0.5, 0.2, 1.0)),
                EnemyType::HeavyUnit => (10.0, Color::new(0.8, 0.2, 0.2, 1.0)),
                EnemyType::Saboteur => (6.0, Color::new(0.7, 0.3, 0.9, 1.0)),
                EnemyType::Commander => (14.0, Color::new(1.0, 0.8, 0.0, 1.0)),
            };

            let mut col = color;
            if enemy.hit_flash_timer > 0.0 {
                col = Color::new(1.0, 1.0, 1.0, 1.0);
            } else if enemy.dodge_timer > 0.0 {
                col = Color::new(color.r, color.g, color.b, 0.4);
            }
            draw_circle(enemy.position.x, enemy.position.y, radius, col);

            let bar_w = radius * 3.0;
            let bar_h = 3.0;
            let bar_x = enemy.position.x - bar_w / 2.0;
            let bar_y = enemy.position.y - radius - 6.0;
            let fill = enemy.health / enemy.max_health;
            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.3, 0.0, 0.0, 0.8));
            draw_rectangle(bar_x, bar_y, bar_w * fill, bar_h, dark::POSITIVE);
        }
    }

    fn draw_hud(&self) {
        draw_rectangle(0.0, 0.0, screen_width(), self.constants.ui.hud_height, Color::new(0.08, 0.08, 0.1, 0.95));

        let y = 8.0;
        let font_size = 16.0;
        let mut x = self.constants.ui.build_panel_w + 10.0;
        let spacing = 140.0;

        let gen = self.factory.power_generation();
        let tower_drain: f32 = self.towers.iter().filter(|t| t.is_active).map(|t| t.power_drain).sum();
        let net_rate = gen - self.factory.power_consumption() - tower_drain;
        let power_text = if net_rate >= 0.0 {
            format!("Power: {:.0} (+{:.0}/s)", self.resources.power, net_rate)
        } else {
            format!("Power: {:.0} ({:.0}/s)", self.resources.power, net_rate)
        };
        let power_color = if net_rate >= 0.0 { dark::POSITIVE } else { dark::NEGATIVE };

        let threat_text = format!(
            "Threat: {} ({:.0})",
            self.threat.reaction_tier().label(),
            self.threat.awareness_level()
        );
        let mut threat_col = super::helpers::threat_color(&self.threat);
        if self.threat_flash_timer > 0.0 {
            threat_col = Color::new(1.0, 0.85, 0.25, 1.0);
        }

        let beacon_text = format!("Beacon: {} ({:.0})", self.beacon_phase.label(), self.beacon_strength);
        let pulse = 0.6 + 0.4 * (f32::sin(get_time() as f32 * 3.0).abs());
        let base_beacon_col = beacon_color(&self.beacon_phase);
        let beacon_col = Color::new(base_beacon_col.r, base_beacon_col.g, base_beacon_col.b, pulse);

        let scavenger_text = format!("Scavengers: {} out", self.scavengers_out);

        let morale_color = if self.population.morale < 20.0 {
            dark::NEGATIVE
        } else if self.population.morale < 50.0 {
            dark::WARNING
        } else {
            dark::POSITIVE
        };
        let income_rate = self.population.productivity(&self.constants) * self.constants.economy.productivity_scrap_rate;

        let items = [
            (format!("Wave: {}", self.current_wave), dark::TEXT),
            (power_text, power_color),
            (format!("Scrap: {:.0}", self.resources.scrap), dark::WARNING),
            (format!("Pop: {}", self.population.count), dark::POSITIVE),
            (format!("Food: {:.0}", self.population.food_supply), dark::TEXT_DIM),
            (format!("Morale: {:.0}", self.population.morale), morale_color),
            (format!("Income: {:.1}/s", income_rate), dark::TEXT_DIM),
            (format!("{}", self.factory.phase.label()), dark::TEXT_BRIGHT),
            (beacon_text, beacon_col),
            (scavenger_text, dark::TEXT_DIM),
            (threat_text, threat_col),
        ];

        for (text, color) in &items {
            draw_text(text, x, y + font_size, font_size, *color);
            x += spacing;
        }

        let max_food = (self.population.count as f32 * self.constants.ui.food_bar_max_per_pop)
            .max(self.constants.ui.food_bar_min_max);
        let bar_w = self.constants.ui.food_bar_w;
        let bar_h = self.constants.ui.food_bar_h;
        let bar_x = screen_width() - bar_w - 10.0;
        let bar_y = 22.0;
        crate::ui::draw_resource_bar(bar_x, bar_y, bar_w, bar_h, self.population.food_supply, max_food, dark::POSITIVE);
    }

    fn draw_wave_status(&self) {
        let text = if self.between_waves {
            format!("Next wave in {:.0}s...", self.wave_timer.max(0.0))
        } else {
            let alive = self.wave_manager.alive_count();
            let queued = self.wave_manager.spawn_queue.len();
            format!("Wave {} - {} enemies ({} spawning)", self.current_wave, alive, queued)
        };

        let x = self.constants.ui.build_panel_w + 10.0;
        let y = self.constants.ui.hud_height + 5.0;
        draw_text(&text, x, y + 14.0, 14.0, dark::TEXT_DIM);

        if self.time_scale > 1.0 {
            let speed_text = "Speed: 2x";
            let dims = measure_text(speed_text, None, 12, 1.0);
            draw_text(speed_text, screen_width() - dims.width - self.constants.ui.sector_panel_w - 10.0, y + 14.0, 12.0, dark::WARNING);
        }

        if self.between_waves {
            let preview = self.build_wave_preview();
            if !preview.is_empty() {
                let list = preview
                    .iter()
                    .map(|(t, c)| format!("{}x{}", enemy_label(t), c))
                    .collect::<Vec<_>>()
                    .join("  ");
                draw_text(&format!("Incoming: {}", list), x, y + 28.0, 12.0, dark::TEXT_DIM);
            }
        }
    }

    fn draw_wave_start_flash(&self) {
        if self.wave_flash_timer <= 0.0 {
            return;
        }
        let alpha = (self.wave_flash_timer / self.constants.ui.wave_flash_duration).clamp(0.0, 1.0);
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(1.0, 1.0, 1.0, self.constants.ui.wave_flash_alpha * alpha),
        );
        let text = if self.shutdown_triggered {
            "BEACON SHUTDOWN".to_string()
        } else {
            format!("WAVE {}", self.last_wave_started)
        };
        let dims = measure_text(&text, None, self.constants.ui.wave_flash_text_size as u16, 1.0);
        draw_text(
            &text,
            screen_width() / 2.0 - dims.width / 2.0,
            screen_height() * self.constants.ui.wave_flash_text_y_frac,
            self.constants.ui.wave_flash_text_size,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    fn draw_notifications(&self) {
        let mut y = self.constants.ui.hud_height + 28.0;
        for note in &self.notifications {
            let alpha = (note.ttl / self.constants.ui.notification_ttl).clamp(0.0, 1.0);
            let col = Color::new(dark::TEXT_BRIGHT.r, dark::TEXT_BRIGHT.g, dark::TEXT_BRIGHT.b, alpha);
            draw_text(&note.text, self.constants.ui.build_panel_w + 10.0, y, 16.0, col);
            y += 18.0;
        }
    }
}
