use crate::data::GameData;
use crate::engine::tower::TowerType;
use crate::ui::{self, SectorPanelAction};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::button;

use super::helpers::tower_type_color;
use super::GameplayState;

impl GameplayState {
    pub fn draw_build_panel(&mut self, data: &GameData) {
        let build_clicked = ui::draw_build_panel(
            0.0,
            self.constants.ui.hud_height,
            self.constants.ui.build_panel_w,
            &data.tower_defs,
            self.resources.scrap,
            self.resources.power,
        );
        if let Some(tower_id) = build_clicked {
            self.placing_tower = Some(tower_id);
        }
    }

    pub fn draw_sector_panel(&mut self) {
        let sector_x = screen_width() - self.constants.ui.sector_panel_w;
        let power_gen = self.factory.power_generation();
        let panel_action = ui::draw_sector_panel(
            sector_x,
            self.constants.ui.hud_height,
            self.constants.ui.sector_panel_w,
            &self.factory.sectors,
            self.resources.scrap,
            power_gen,
        );
        if let Some(action) = panel_action {
            match action {
                SectorPanelAction::Unlock(sector_id) => self.unlock_sector(&sector_id),
                SectorPanelAction::Repair(sector_id) => self.repair_sector(&sector_id),
            }
        }
    }

    pub fn draw_placement_ghost(&self, data: &GameData) {
        if let Some(ref tower_id) = self.placing_tower {
            if let Some(def) = data.tower_def_by_id(tower_id) {
                let (mx, my) = mouse_position();
                let pos = self.snap_to_grid(mx, my);

                let on_path = self.is_on_path(pos);
                let occupied = self
                    .towers
                    .iter()
                    .any(|t| (t.position - pos).length() < self.constants.placement.tower_min_spacing);
                let blocked = on_path || occupied;

                let ghost_color = if blocked {
                    Color::new(0.9, 0.2, 0.2, 0.5)
                } else {
                    let c = tower_type_color(&def.tower_type);
                    Color::new(c.r, c.g, c.b, 0.5)
                };

                let range_color = if blocked {
                    Color::new(0.9, 0.2, 0.2, 0.2)
                } else {
                    let c = tower_type_color(&def.tower_type);
                    Color::new(c.r, c.g, c.b, 0.3)
                };

                let range_mult = if self.factory.is_sector_active("ai_vault") { 1.2 } else { 1.0 };
                let radius = self.constants.ui.tower_base_radius;
                draw_circle(pos.x, pos.y, radius, ghost_color);
                draw_circle_lines(pos.x, pos.y, def.base_range * range_mult, 1.0, range_color);

                let hint = format!("Place {} - Click to confirm, Right-click to cancel", def.name);
                draw_text(&hint, self.constants.ui.build_panel_w + 10.0, screen_height() - 15.0, 14.0, dark::TEXT_DIM);
            }
        }
    }

    pub fn draw_shutdown_ui(&mut self) {
        let btn_w = 160.0;
        let btn_h = 24.0;
        let btn_x = screen_width() - self.constants.ui.sector_panel_w - btn_w - 10.0;
        let btn_y = 6.0;

        if !self.beacon_active {
            if button(btn_x, btn_y, btn_w, btn_h, "Start Beacon") {
                self.start_beacon();
            }
            return;
        }

        if !self.shutdown_triggered && self.current_wave >= 1 {
            // Recall button: pull scavengers back while beacon stays on
            if !self.scavenger_recall_active && self.scavengers_out > 0 {
                let recall_y = btn_y + 28.0;
                if button(btn_x, recall_y, btn_w, btn_h, "Recall Scavengers") {
                    self.scavenger_recall_active = true;
                    self.scavenger_recall_timer = 0.0;
                    self.push_notification("Scavenger recall issued. Hold the line.".to_string());
                }
            }

            // Shutdown button
            if button(btn_x, btn_y, btn_w, btn_h, "Shutdown Beacon") {
                self.trigger_shutdown();
            }
        }
    }

    pub fn draw_selected_tower_panel(&mut self, data: &GameData) {
        let Some(idx) = self.selected_tower else { return; };
        let Some(tower) = self.towers.get(idx) else { return; };

        let panel_x = 0.0;
        let panel_y = self.constants.ui.hud_height;
        let panel_w = self.constants.ui.build_panel_w;
        let panel_h = screen_height() - panel_y;

        let name = data
            .tower_def_by_id(&tower.tower_id)
            .map(|d| d.name.as_str())
            .unwrap_or("Tower");

        let info_x = panel_x + 10.0;
        let mut info_y = panel_y + panel_h - 110.0;

        draw_text("SELECTED", info_x, info_y, 14.0, dark::TEXT_DIM);
        info_y += 16.0;
        draw_text(&format!("{} (Lv {})", name, tower.level), info_x, info_y, 14.0, dark::TEXT_BRIGHT);
        info_y += 16.0;
        draw_text(&format!("Dmg: {:.1}", tower.damage), info_x, info_y, 12.0, dark::TEXT);
        info_y += 14.0;
        draw_text(&format!("Rng: {:.0}", tower.range), info_x, info_y, 12.0, dark::TEXT);
        info_y += 14.0;
        draw_text(&format!("Rate: {:.2}/s", tower.fire_rate), info_x, info_y, 12.0, dark::TEXT);

        let btn_w = panel_w - 20.0;
        let btn_h = 24.0;
        let btn_x = panel_x + 10.0;
        let btn_y = panel_y + panel_h - 30.0;

        if tower.level >= self.constants.tower.upgrade_max_level {
            draw_text("Max Level", btn_x, btn_y + 16.0, 12.0, dark::TEXT_DIM);
            return;
        }

        if !self.factory.is_sector_active("research_lab") {
            draw_text("Requires Research Lab", btn_x, btn_y + 16.0, 12.0, dark::TEXT_DIM);
            return;
        }

        let upgrade_cost = tower.base_scrap_cost * tower.level as f32;
        let label = format!("Upgrade ({:.0})", upgrade_cost);
        if self.resources.scrap >= upgrade_cost {
            if button(btn_x, btn_y, btn_w, btn_h, &label) {
                self.upgrade_tower(idx);
            }
        } else {
            draw_rectangle(btn_x, btn_y, btn_w, btn_h, Color::new(0.2, 0.2, 0.2, 0.5));
            let dims = measure_text(&label, None, 12, 1.0);
            draw_text(&label, btn_x + (btn_w - dims.width) / 2.0, btn_y + 16.0, 12.0, dark::TEXT_DIM);
        }
    }

    pub fn handle_placement_click(&mut self, data: &GameData) {
        if self.placing_tower.is_none() {
            return;
        }

        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        let (mx, my) = mouse_position();
        if mx < self.constants.ui.build_panel_w
            || mx > screen_width() - self.constants.ui.sector_panel_w
            || my < self.constants.ui.hud_height
        {
            return;
        }

        let tower_id = self.placing_tower.as_ref().unwrap().clone();
        if let Some(def) = data.tower_def_by_id(&tower_id) {
            if self.resources.scrap < def.cost_scrap {
                return;
            }

            let gen = self.factory.power_generation();
            let current_drain: f32 = self.factory.power_consumption()
                + self.towers.iter().filter(|t| t.is_active).map(|t| t.power_drain).sum::<f32>();
            let net_after = gen - current_drain - def.cost_power;
            if net_after < 0.0 && self.resources.power < self.constants.economy.power_buffer_min_for_build {
                return;
            }

            let pos = self.snap_to_grid(mx, my);
            let occupied = self.towers.iter().any(|t| (t.position - pos).length() < 20.0);
            let on_path = self.is_on_path(pos);
            if !occupied && !on_path {
                self.resources.scrap -= def.cost_scrap;

                let tt = match def.tower_type.as_str() {
                    "Ballistic" => TowerType::Ballistic,
                    "Laser" => TowerType::Laser,
                    "Emp" => TowerType::Emp,
                    "AreaDenial" => TowerType::AreaDenial,
                    "Subversion" => TowerType::Subversion,
                    _ => TowerType::Ballistic,
                };

                self.towers.push(crate::engine::tower::Tower::new(
                    tt,
                    def.id.clone(),
                    pos,
                    def.base_range,
                    def.base_damage,
                    def.fire_rate,
                    def.cost_power,
                    def.cost_scrap,
                ));
                self.threat.add_noise(0.5);
                self.placing_tower = None;
            }
        }
    }

    pub fn handle_selection_click(&mut self) {
        if self.placing_tower.is_some() {
            return;
        }

        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        let (mx, my) = mouse_position();
        if mx < self.constants.ui.build_panel_w
            || mx > screen_width() - self.constants.ui.sector_panel_w
            || my < self.constants.ui.hud_height
        {
            return;
        }

        let mut selected = None;
        let mut best_dist = self.constants.ui.tower_select_radius;
        for (i, tower) in self.towers.iter().enumerate() {
            let dist = (tower.position - vec2(mx, my)).length();
            if dist < best_dist {
                best_dist = dist;
                selected = Some(i);
            }
        }

        self.selected_tower = selected;
    }

    fn unlock_sector(&mut self, sector_id: &str) {
        if let Some(sector) = self.factory.sectors.iter_mut().find(|s| s.id == sector_id) {
            if !sector.unlocked && self.resources.scrap >= sector.unlock_cost {
                self.resources.scrap -= sector.unlock_cost;
                sector.unlocked = true;
                self.threat.add_from_sector(sector_id);
                if sector_id == "ai_vault" {
                    self.threat.add_territory(5.0);
                }
                self.factory.check_awakening();
            }
        }
    }

    fn repair_sector(&mut self, sector_id: &str) {
        let repair_cost = self.constants.sector.repair_cost;
        let repair_amount = self.constants.sector.repair_amount;
        if self.resources.scrap < repair_cost {
            return;
        }
        if let Some(sector) = self.factory.sectors.iter_mut().find(|s| s.id == sector_id && s.unlocked) {
            if sector.integrity >= sector.max_integrity {
                return;
            }
            self.resources.scrap -= repair_cost;
            sector.integrity = (sector.integrity + repair_amount).min(sector.max_integrity);
        }
    }

    fn start_beacon(&mut self) {
        self.beacon_active = true;
        self.between_waves = true;
        self.wave_timer = self.constants.gameplay.beacon_start_delay;
        if self.scavengers_sent == 0 {
            self.scavengers_out = self.constants.scavenger.initial_scavengers;
            self.scavengers_sent = self.constants.scavenger.initial_scavengers;
        }
        self.push_notification("Beacon activated. Machines inbound.".to_string());
    }

    fn trigger_shutdown(&mut self) {
        self.shutdown_triggered = true;
        self.scavenger_recall_active = false;
        // Current wave (if any) plays out — don't clear spawn_queue
        // No new waves will start (update_wave_timers and start_wave_if_ready check shutdown_triggered)

        // Resolve scavengers still out: flat 20% loss chance per team
        let still_out = self.scavengers_out;
        if still_out > 0 {
            let mut lost = 0u32;
            for _ in 0..still_out {
                if gen_range(0.0f32, 1.0) < self.constants.scavenger.shutdown_loss_chance {
                    lost += 1;
                }
            }
            let returned = still_out - lost;
            self.scavengers_out = 0;
            self.scavengers_lost += lost;
            self.scavengers_returned += returned;
            if lost > 0 {
                self.push_notification(format!("{} scavenger team(s) lost during shutdown", lost));
            }
            if returned > 0 {
                self.push_notification(format!("{} scavenger team(s) made it back", returned));
            }
        }

        if self.wave_manager.alive_count() > 0 || !self.wave_manager.spawn_queue.is_empty() {
            self.push_notification("Beacon offline. Survive the remaining wave.".to_string());
        } else {
            self.push_notification("Beacon offline. No hostiles remaining.".to_string());
        }
    }

    fn snap_to_grid(&self, x: f32, y: f32) -> Vec2 {
        let grid = self.constants.placement.grid;
        vec2((x / grid).round() * grid, (y / grid).round() * grid)
    }

    fn is_on_path(&self, pos: Vec2) -> bool {
        let min_dist = self.constants.placement.path_min_dist;
        for i in 0..self.map_path.len().saturating_sub(1) {
            let a = self.map_path[i];
            let b = self.map_path[i + 1];
            if super::helpers::point_to_segment_dist(pos, a, b) < min_dist {
                return true;
            }
        }
        false
    }
}
