//! Player intents: map picking and the state mutations the panels dispatch to.

use crate::data::strings::{fill, text};
use crate::data::GameData;
use crate::engine::map::{BuildingState, SlotState};
use crate::state::gameplay::{CycleBaseline, GameplayState, TowerUiStats};
use macroquad::prelude::*;

impl GameplayState {
    pub fn handle_map_click(&mut self, data: &GameData) {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        let (mx, my) = mouse_position();
        if self.click_is_over_ui(mx, my) {
            return;
        }

        let world_pos = self.screen_to_world(vec2(mx, my));

        let core_dist = (world_pos - self.map_state.factory_core).length();
        if core_dist <= 26.0 {
            self.select_core_only();
            return;
        }

        // If placing a tower and the click landed on a powered empty pad, build.
        if let Some(ref tower_id) = self.placing_tower.clone() {
            if let Some((slot_idx, _)) = self.map_state.nearest_slot(world_pos) {
                let slot = &self.map_state.slots[slot_idx];
                if slot.state == SlotState::Powered && slot.tower_index.is_none() {
                    self.try_place_tower_on_slot(slot_idx, tower_id, data);
                    return;
                }
            }
        }

        // Otherwise select whichever of the nearest pad / building is closer.
        let slot_result = self.map_state.nearest_slot(world_pos);
        let building_result = self.map_state.nearest_building(world_pos);
        match (slot_result, building_result) {
            (Some((si, sd)), Some((bi, bd))) => {
                if sd <= bd {
                    self.select_slot_only(si);
                } else {
                    self.select_building_only(bi);
                }
            }
            (Some((si, _)), None) => self.select_slot_only(si),
            (None, Some((bi, _))) => self.select_building_only(bi),
            (None, None) => self.clear_selection(),
        }
    }

    /// True when the click landed on a docked panel rather than the map, so the
    /// map picker must ignore it and let the panel's own hit-testing win.
    fn click_is_over_ui(&self, mx: f32, my: f32) -> bool {
        if mx < self.constants.ui.build_panel_w
            || mx > screen_width() - self.constants.ui.sector_panel_w
            || my < self.constants.ui.hud_height
        {
            return true;
        }

        let panel_x = self.constants.ui.build_panel_w + 8.0;
        let panel_w =
            (screen_width() - self.constants.ui.build_panel_w - self.constants.ui.sector_panel_w)
                .max(360.0)
                - 16.0;
        let panel_h = self.constants.ui.bottom_context_h;
        let panel_y = screen_height() - panel_h - 8.0;
        if my >= panel_y && my <= panel_y + panel_h && mx >= panel_x && mx <= panel_x + panel_w {
            return true;
        }

        // The factory console grows upward out of the bottom context panel, so
        // it claims a taller strip while the core is selected.
        if self.selected_core {
            let core_panel_x = panel_x - 10.0;
            let core_panel_w = panel_w + 20.0;
            let core_panel_top = panel_y - 160.0;
            if mx >= core_panel_x
                && mx <= core_panel_x + core_panel_w
                && my >= core_panel_top
                && my <= panel_y + panel_h
            {
                return true;
            }
        }

        false
    }

    pub(crate) fn clear_slot(&mut self, idx: usize) {
        if self.beacon_active {
            self.push_notification(text().notifications.repairs_locked.clone());
            return;
        }
        if !self.slot_power_requirement_met(idx) {
            return;
        }
        let cost = self.map_state.slots[idx].clear_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let newly_active = self.map_state.set_slot_state(idx, SlotState::Cleared);
        for path_id in &newly_active {
            let path = self.path_display_name(path_id);
            self.push_notification(fill(&text().notifications.path_opened, &[("path", &path)]));
        }
    }

    pub(crate) fn power_slot(&mut self, idx: usize) {
        if self.beacon_active {
            self.push_notification(text().notifications.repairs_locked.clone());
            return;
        }
        if !self.slot_power_requirement_met(idx) {
            return;
        }
        let cost = self.map_state.slots[idx].power_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let newly_active = self.map_state.set_slot_state(idx, SlotState::Powered);
        for path_id in &newly_active {
            let path = self.path_display_name(path_id);
            self.push_notification(fill(&text().notifications.path_opened, &[("path", &path)]));
        }
    }

    /// A pad may depend on a machine being powered first; notify and refuse when
    /// that machine is still dark.
    fn slot_power_requirement_met(&mut self, idx: usize) -> bool {
        let Some(req) = self.map_state.slots[idx].requires_building_power.clone() else {
            return true;
        };
        if self.map_state.is_building_powered(&req) {
            return true;
        }
        let name = self.building_display_name_by_id(&req);
        self.push_notification(fill(
            &text().notifications.needs_power_from,
            &[("name", &name)],
        ));
        false
    }

    /// Machines can also depend on an upstream machine's power.
    fn building_power_requirement_met(&mut self, idx: usize) -> bool {
        let Some(req) = self.map_state.buildings[idx].requires_power_from.clone() else {
            return true;
        };
        if self.map_state.is_building_powered(&req) {
            return true;
        }
        let name = self.building_display_name_by_id(&req);
        self.push_notification(fill(
            &text().notifications.needs_power_from,
            &[("name", &name)],
        ));
        false
    }

    pub(crate) fn try_place_tower_on_slot(
        &mut self,
        slot_idx: usize,
        tower_id: &str,
        data: &GameData,
    ) {
        if !self.is_tower_unlocked(tower_id) {
            return;
        }
        let slot = &self.map_state.slots[slot_idx];
        if slot.state != SlotState::Powered || slot.tower_index.is_some() {
            return;
        }

        let Some(def) = data.tower_def_by_id(tower_id) else {
            return;
        };
        if self.resources.scrap < def.cost_scrap {
            return;
        }

        let gen = self.factory.power_generation(&self.constants.economy);
        let current_drain: f32 = self.factory.power_consumption()
            + self
                .towers
                .iter()
                .filter(|t| t.is_active)
                .map(|t| t.power_drain)
                .sum::<f32>();
        let net_after = gen - current_drain - def.cost_power;
        if net_after < 0.0
            && self.resources.power < self.constants.economy.power_buffer_min_for_build
        {
            return;
        }

        self.resources.scrap -= def.cost_scrap;
        let pos = slot.position;

        let tower_idx = self.towers.len();
        self.towers.push(crate::engine::tower::Tower::new(
            def.tower_type.clone(),
            def.id.clone(),
            pos,
            def.base_range,
            def.base_damage,
            def.fire_rate,
            def.cost_power,
            def.cost_scrap,
            def.color(),
        ));
        self.tower_stats.push(TowerUiStats::default());

        self.map_state.slots[slot_idx].tower_index = Some(tower_idx);
        self.threat.add_noise(0.5);
        self.placing_tower = None;
        self.selected_tower = Some(tower_idx);
    }

    pub(crate) fn repair_building(&mut self, idx: usize) {
        if self.beacon_active {
            self.push_notification(text().notifications.repairs_locked.clone());
            return;
        }
        if !self.building_power_requirement_met(idx) {
            return;
        }
        let cost = self.map_state.buildings[idx].repair_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        self.map_state
            .set_building_state(idx, BuildingState::Repaired);
        let name = self.building_display_name(&self.map_state.buildings[idx]);
        self.push_notification(fill(&text().notifications.repaired, &[("name", &name)]));
    }

    pub(crate) fn power_building(&mut self, idx: usize) {
        if self.beacon_active {
            self.push_notification(text().notifications.repairs_locked.clone());
            return;
        }
        if !self.building_power_requirement_met(idx) {
            return;
        }
        let cost = self.map_state.buildings[idx].power_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let building_id = self.map_state.buildings[idx].id.clone();
        let newly_active = self
            .map_state
            .set_building_state(idx, BuildingState::Powered);
        let name = self.building_display_name(&self.map_state.buildings[idx]);
        self.push_notification(fill(&text().notifications.powered, &[("name", &name)]));
        if let Some(awakening) = self.factory.unlock_from_core(&building_id) {
            self.threat.add_kind(awakening.signature, awakening.threat);
            self.push_notification(fill(
                &text().notifications.sector_awakened,
                &[
                    ("name", &awakening.name),
                    ("role", &awakening.role),
                    ("n", &format!("{:.0}", awakening.threat)),
                    ("signature", awakening.signature.label()),
                ],
            ));
        }
        for path_id in &newly_active {
            let path = self.path_display_name(path_id);
            self.push_notification(fill(&text().notifications.path_opened, &[("path", &path)]));
        }
    }

    pub(crate) fn purchase_upgrade(&mut self, upgrade_id: &str) {
        let def = self
            .upgrade_defs
            .iter()
            .find(|u| u.id == upgrade_id)
            .cloned();
        if let Some(def) = def {
            if self
                .factory
                .can_purchase(&def, self.resources.scrap, self.resources.power)
            {
                self.resources.scrap -= def.cost_scrap;
                self.factory.purchase_upgrade(&def);
            }
        }
    }

    pub(crate) fn start_beacon(&mut self) {
        self.beacon_active = true;
        self.shutdown_triggered = false;
        self.between_waves = true;
        self.wave_timer = self.constants.gameplay.beacon_start_delay;
        // Base draw from how much factory is lit up, plus the permanent per-cycle
        // escalation the machines have banked from previous beacon windows.
        self.beacon_start_difficulty_bonus =
            self.compute_beacon_start_difficulty_bonus() + self.machine_escalation;

        // Snapshot cumulative totals so this cycle's salvage report shows only
        // the gains earned during this beacon window.
        self.cycle_baseline = CycleBaseline {
            wave: self.current_wave,
            sent: self.scavengers_sent,
            returned: self.scavengers_returned,
            lost: self.scavengers_lost,
            scrap: self.scavenger_scrap_gained,
            food: self.scavenger_food_gained,
            population: self.scavenger_population_gained,
        };

        // Send a fresh field team out for this cycle.
        let teams = self.constants.scavenger.initial_scavengers;
        self.scavengers_out += teams;
        self.scavengers_sent += teams;
        self.scavenger_recall_active = false;
        self.scavenger_recall_timer = 0.0;
        self.push_notification(text().notifications.beacon_up.clone());
    }

    pub(crate) fn trigger_shutdown(&mut self) {
        self.shutdown_triggered = true;
        self.beacon_active = false;
        self.beacon_start_difficulty_bonus = 0.0;
        self.scavenger_recall_active = true;
        self.scavenger_recall_timer = 0.0;
        self.wave_manager.spawn_queue.clear();
        self.between_waves = false;
        self.push_notification(text().notifications.beacon_down.clone());
    }
}
