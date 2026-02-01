use crate::data::GameData;
use crate::engine::map::{BuildingState, SlotState};
use crate::engine::tower::TowerType;
use crate::ui::{self, SectorPanelAction};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::button;

use super::helpers::tower_type_color;
use super::GameplayState;

impl GameplayState {
    pub fn draw_build_panel(&mut self, data: &GameData) {
        let unlocked_towers: Vec<_> = data
            .tower_defs
            .iter()
            .filter(|t| self.is_tower_unlocked(&t.id))
            .cloned()
            .collect();
        let build_clicked = ui::draw_build_panel(
            0.0,
            self.constants.ui.hud_height,
            self.constants.ui.build_panel_w,
            &unlocked_towers,
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
            &self.factory,
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
                let world_mouse = self.screen_to_world(vec2(mouse_position().0, mouse_position().1));

                // Find nearest powered empty slot
                let mut best_slot = None;
                let mut best_dist = self.map_state.slot_interact_radius;
                for (idx, slot) in self.map_state.slots.iter().enumerate() {
                    if slot.state != SlotState::Powered || slot.tower_index.is_some() {
                        continue;
                    }
                    let dist = (slot.position - world_mouse).length();
                    if dist < best_dist {
                        best_dist = dist;
                        best_slot = Some(idx);
                    }
                }

                if let Some(idx) = best_slot {
                    let pos = self.map_state.slots[idx].position;
                    let c = tower_type_color(&def.tower_type);
                    let ghost_color = Color::new(c.r, c.g, c.b, 0.5);
                    let range_color = Color::new(c.r, c.g, c.b, 0.3);

                    let range_mult = if self.factory.is_sector_active("ai_vault") { 1.2 } else { 1.0 };
                    let radius = self.constants.ui.tower_base_radius;
                    draw_circle(pos.x, pos.y, radius, ghost_color);
                    draw_circle_lines(pos.x, pos.y, def.base_range * range_mult, 1.0, range_color);
                }

                // Hint text is drawn in screen space - need to return to default camera for this
                // (handled by caller since this is called in world-space section)
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

        if self.beacon_active && !self.shutdown_triggered && self.current_wave >= 1 {
            if button(btn_x, btn_y, btn_w, btn_h, "Shutdown Beacon") {
                self.trigger_shutdown();
            }
        }

        if self.beacon_active && !self.scavenger_recall_active && self.scavengers_out > 0 {
            let recall_y = btn_y + 28.0;
            if button(btn_x, recall_y, btn_w, btn_h, "Recall Scavengers") {
                self.scavenger_recall_active = true;
                self.scavenger_recall_timer = 0.0;
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

    pub fn handle_map_click(&mut self, data: &GameData) {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        let (mx, my) = mouse_position();

        // Skip if click is in UI panels
        if mx < self.constants.ui.build_panel_w
            || mx > screen_width() - self.constants.ui.sector_panel_w
            || my < self.constants.ui.hud_height
        {
            return;
        }

        // Skip if in bottom slot panel area
        let panel_w = 300.0;
        let panel_h = 80.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = screen_height() - panel_h - 10.0;
        if mx >= panel_x && mx <= panel_x + panel_w && my >= panel_y && my <= panel_y + panel_h {
            return;
        }
        if self.selected_core {
            let core_panel_x = panel_x - 10.0;
            let core_panel_w = panel_w + 20.0;
            let core_panel_top = panel_y - 160.0;
            if mx >= core_panel_x && mx <= core_panel_x + core_panel_w && my >= core_panel_top && my <= panel_y + panel_h {
                return;
            }
        }

        let world_pos = self.screen_to_world(vec2(mx, my));

        let core_dist = (world_pos - self.map_state.factory_core).length();
        if core_dist <= 26.0 {
            self.selected_core = true;
            self.selected_slot = None;
            self.selected_building = None;
            self.selected_tower = None;
            if self.selected_upgrade.is_none() {
                if let Some(upg) = self.available_upgrades().into_iter().find(|u| !self.factory.has_upgrade(&u.id)) {
                    self.selected_upgrade = Some(upg.id.clone());
                }
            }
            return;
        }

        // If placing tower and clicked a powered empty slot -> place tower
        if let Some(ref tower_id) = self.placing_tower.clone() {
            if let Some((slot_idx, _)) = self.map_state.nearest_slot(world_pos) {
                let slot = &self.map_state.slots[slot_idx];
                if slot.state == SlotState::Powered && slot.tower_index.is_none() {
                    self.try_place_tower_on_slot(slot_idx, tower_id, data);
                    return;
                }
            }
        }

        // Try to find nearest slot or building for selection
        let slot_result = self.map_state.nearest_slot(world_pos);
        let building_result = self.nearest_unlocked_building(world_pos);

        // Pick the closer of slot vs building
        match (slot_result, building_result) {
            (Some((si, sd)), Some((bi, bd))) => {
                if sd <= bd {
                    self.selected_slot = Some(si);
                    self.selected_building = None;
                    self.selected_tower = self.map_state.slots[si].tower_index;
                    self.selected_core = false;
                    self.selected_upgrade = None;
                } else {
                    self.selected_building = Some(bi);
                    self.selected_slot = None;
                    self.selected_tower = None;
                    self.selected_core = false;
                    self.selected_upgrade = None;
                }
            }
            (Some((si, _)), None) => {
                self.selected_slot = Some(si);
                self.selected_building = None;
                self.selected_tower = self.map_state.slots[si].tower_index;
                self.selected_core = false;
                self.selected_upgrade = None;
            }
            (None, Some((bi, _))) => {
                self.selected_building = Some(bi);
                self.selected_slot = None;
                self.selected_tower = None;
                self.selected_core = false;
                self.selected_upgrade = None;
            }
            (None, None) => {
                self.selected_slot = None;
                self.selected_building = None;
                self.selected_tower = None;
                self.selected_core = false;
                self.selected_upgrade = None;
            }
        }
    }

    pub(crate) fn clear_slot(&mut self, idx: usize) {
        let cost = self.map_state.slots[idx].clear_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let newly_active = self.map_state.set_slot_state(idx, SlotState::Cleared);
        for path_id in &newly_active {
            self.push_notification(format!("New path opened: {}", path_id));
        }
    }

    pub(crate) fn power_slot(&mut self, idx: usize) {
        let cost = self.map_state.slots[idx].power_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let newly_active = self.map_state.set_slot_state(idx, SlotState::Powered);
        for path_id in &newly_active {
            self.push_notification(format!("New path opened: {}", path_id));
        }
    }

    fn try_place_tower_on_slot(&mut self, slot_idx: usize, tower_id: &str, data: &GameData) {
        if !self.is_tower_unlocked(tower_id) {
            return;
        }
        let slot = &self.map_state.slots[slot_idx];
        if slot.state != SlotState::Powered || slot.tower_index.is_some() {
            return;
        }

        let Some(def) = data.tower_def_by_id(tower_id) else { return; };
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

        self.resources.scrap -= def.cost_scrap;
        let pos = slot.position;

        let tt = match def.tower_type.as_str() {
            "Ballistic" => TowerType::Ballistic,
            "Laser" => TowerType::Laser,
            "Emp" => TowerType::Emp,
            "AreaDenial" => TowerType::AreaDenial,
            "Subversion" => TowerType::Subversion,
            _ => TowerType::Ballistic,
        };

        let tower_idx = self.towers.len();
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

        self.map_state.slots[slot_idx].tower_index = Some(tower_idx);
        self.threat.add_noise(0.5);
        self.placing_tower = None;
        self.selected_tower = Some(tower_idx);
    }

    pub(crate) fn repair_building(&mut self, idx: usize) {
        if self
            .map_state
            .buildings
            .get(idx)
            .map(|b| !self.is_building_unlocked(b))
            .unwrap_or(true)
        {
            return;
        }
        let cost = self.map_state.buildings[idx].repair_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        self.map_state.set_building_state(idx, BuildingState::Repaired);
        self.push_notification(format!("{} repaired", self.map_state.buildings[idx].id));
    }

    pub(crate) fn power_building(&mut self, idx: usize) {
        if self
            .map_state
            .buildings
            .get(idx)
            .map(|b| !self.is_building_unlocked(b))
            .unwrap_or(true)
        {
            return;
        }
        let cost = self.map_state.buildings[idx].power_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let newly_active = self.map_state.set_building_state(idx, BuildingState::Powered);
        self.push_notification(format!("{} powered on", self.map_state.buildings[idx].id));
        for path_id in &newly_active {
            self.push_notification(format!("New entrance opened: {}", path_id));
        }
    }

    pub(crate) fn purchase_upgrade(&mut self, upgrade_id: &str) {
        let def = self.upgrade_defs.iter().find(|u| u.id == upgrade_id).cloned();
        if let Some(def) = def {
            if self.factory.can_purchase(&def, self.resources.scrap, self.resources.power) {
                self.resources.scrap -= def.cost_scrap;
                self.factory.purchase_upgrade(&def);
            }
        }
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
        self.beacon_start_difficulty_bonus = self.compute_beacon_start_difficulty_bonus();
        if self.scavengers_sent == 0 {
            self.scavengers_out = self.constants.scavenger.initial_scavengers;
            self.scavengers_sent = self.constants.scavenger.initial_scavengers;
        }
        self.push_notification("Beacon activated. Machines inbound.".to_string());
    }

    fn trigger_shutdown(&mut self) {
        self.shutdown_triggered = true;
        self.beacon_active = false;
        self.beacon_start_difficulty_bonus = 0.0;
        self.scavenger_recall_active = true;
        self.scavenger_recall_timer = 0.0;
        self.wave_manager.spawn_queue.clear();
        self.between_waves = false;
        self.push_notification("Beacon shut down. No new waves incoming.".to_string());
    }
}
