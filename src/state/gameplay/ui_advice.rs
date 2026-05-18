//! Derived, non-serialized UI advice for the command deck.

use crate::data::{BuildingBoon, GameData, TowerDef};
use crate::engine::beacon::BeaconPhase;
use crate::engine::enemy::EnemyType;
use crate::engine::map::{BuildingState, MapBuilding, MapSlot, SlotState};
use crate::engine::wave::preview_wave_entries;
use macroquad::prelude::*;

use super::helpers::{enemy_label, entrance_label};
use super::GameplayState;

#[derive(Clone, Debug)]
pub struct UiAdvice {
    pub objective: String,
    pub suggested_action: SuggestedAction,
    pub risk: String,
    pub alerts: Vec<AlertBanner>,
    pub power: PowerGridSnapshot,
    pub wave_preview: WavePreviewCard,
}

#[derive(Clone, Debug)]
pub struct SuggestedAction {
    pub label: String,
    pub detail: String,
    pub cost: String,
    pub risk: String,
    pub target: AdviceTarget,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AdviceTarget {
    Building(usize),
    Slot(usize),
    TowerDef(String),
    BeaconStart,
    BeaconShutdown,
    ScavengerRecall,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Clone, Debug)]
pub struct AlertBanner {
    pub severity: AlertSeverity,
    pub label: String,
    pub detail: String,
    pub priority: u32,
}

#[derive(Clone, Debug)]
pub struct PowerGridSnapshot {
    pub generated: f32,
    pub used: f32,
    pub net: f32,
    pub battery: f32,
    pub seconds_to_empty: Option<f32>,
    pub offline_towers: usize,
}

#[derive(Clone, Debug)]
pub struct WavePreviewCard {
    pub title: String,
    pub counts: Vec<(EnemyType, usize)>,
    pub path_split: Vec<(String, f32)>,
    pub recommendation: String,
    pub alive_count: usize,
    pub queued_count: usize,
    pub live: bool,
}

#[derive(Clone, Debug)]
pub struct PlacementPreview {
    pub tower_id: String,
    pub valid_slots: Vec<usize>,
    pub invalid_slots: Vec<(usize, InvalidPadReason)>,
    pub hovered_slot: Option<usize>,
    pub covered_paths: Vec<String>,
    pub expected_targets: Vec<(EnemyType, usize)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidPadReason {
    NeedsPower,
    Occupied,
    TooExpensive,
}

impl InvalidPadReason {
    pub fn label(self) -> &'static str {
        match self {
            InvalidPadReason::NeedsPower => "Needs power",
            InvalidPadReason::Occupied => "Occupied",
            InvalidPadReason::TooExpensive => "Too expensive",
        }
    }
}

impl GameplayState {
    pub fn build_ui_advice(&self, data: &GameData) -> UiAdvice {
        let power = self.power_grid_snapshot();
        let wave_preview = self.wave_preview_card();
        let mut alerts = self.build_alerts(&power);
        alerts.sort_by(|a, b| {
            severity_rank(b.severity)
                .cmp(&severity_rank(a.severity))
                .then_with(|| b.priority.cmp(&a.priority))
        });

        let suggested_action = self.suggest_next_action(data, &power);
        let objective = objective_for_action(&suggested_action);
        let risk = suggested_action.risk.clone();

        UiAdvice {
            objective,
            suggested_action,
            risk,
            alerts,
            power,
            wave_preview,
        }
    }

    pub fn apply_advice_focus(&mut self, target: &AdviceTarget, data: &GameData) {
        match target {
            AdviceTarget::Building(idx) => {
                if *idx < self.map_state.buildings.len() {
                    self.selected_building = Some(*idx);
                    self.selected_slot = None;
                    self.selected_tower = None;
                    self.selected_core = false;
                    self.selected_upgrade = None;
                }
            }
            AdviceTarget::Slot(idx) => {
                if *idx < self.map_state.slots.len() {
                    self.selected_slot = Some(*idx);
                    self.selected_building = None;
                    self.selected_tower = self.map_state.slots[*idx].tower_index;
                    self.selected_core = false;
                    self.selected_upgrade = None;
                }
            }
            AdviceTarget::TowerDef(id) => {
                if data.tower_def_by_id(id).is_some() {
                    self.placing_tower = Some(id.clone());
                    self.selected_slot =
                        self.best_slot_for_tower_id(id, data).or(self.selected_slot);
                    self.selected_building = None;
                    self.selected_tower = None;
                    self.selected_core = false;
                    self.selected_upgrade = None;
                }
            }
            AdviceTarget::BeaconStart
            | AdviceTarget::BeaconShutdown
            | AdviceTarget::ScavengerRecall => {
                self.selected_core = true;
                self.selected_slot = None;
                self.selected_building = None;
                self.selected_tower = None;
            }
            AdviceTarget::None => {}
        }
    }

    pub fn placement_preview(
        &self,
        data: &GameData,
        world_mouse: Vec2,
    ) -> Option<PlacementPreview> {
        let tower_id = self.placing_tower.as_ref()?;
        let def = data.tower_def_by_id(tower_id)?;
        let range = self.effective_tower_range(def.base_range);
        let has_scrap = self.resources.scrap >= def.cost_scrap;
        let has_power = self.has_power_buffer_for(def);

        let mut valid_slots = Vec::new();
        let mut invalid_slots = Vec::new();
        let mut hovered_slot = None;
        let mut hovered_dist = self.map_state.slot_interact_radius;

        for (idx, slot) in self.map_state.slots.iter().enumerate() {
            if !self.map_state.is_slot_visible(slot) {
                continue;
            }
            let reason = self.invalid_pad_reason(slot, has_scrap && has_power);
            if let Some(reason) = reason {
                invalid_slots.push((idx, reason));
            } else {
                valid_slots.push(idx);
            }
            let dist = (slot.position - world_mouse).length();
            if dist <= hovered_dist {
                hovered_dist = dist;
                hovered_slot = Some(idx);
            }
        }

        let covered_paths = hovered_slot
            .and_then(|idx| self.map_state.slots.get(idx))
            .map(|slot| self.covered_paths_for_range(slot.position, range))
            .unwrap_or_default();

        Some(PlacementPreview {
            tower_id: tower_id.clone(),
            valid_slots,
            invalid_slots,
            hovered_slot,
            covered_paths,
            expected_targets: self.wave_preview_counts(),
        })
    }

    pub fn covered_paths_for_range(&self, center: Vec2, range: f32) -> Vec<String> {
        let mut result = Vec::new();
        for path in self.map_state.active_paths_limited() {
            let mut covered = false;
            for pair in path.points.windows(2) {
                let dist = distance_to_segment(center, pair[0], pair[1]);
                if dist <= range {
                    covered = true;
                    break;
                }
            }
            if covered {
                result.push(path.id);
            }
        }
        result
    }

    pub fn effective_tower_range(&self, base_range: f32) -> f32 {
        let range_mult = if self.factory.is_sector_active("ai_vault") {
            self.constants.tower.upgrade_range_mult
        } else {
            1.0
        };
        base_range * range_mult
    }

    pub fn path_display_name(&self, path_id: &str) -> String {
        let label = entrance_label(path_id);
        if label == path_id {
            titleize(path_id)
        } else {
            label.to_string()
        }
    }

    pub fn building_display_name(&self, building: &MapBuilding) -> String {
        let raw = if building.building_type.is_empty() {
            building.id.as_str()
        } else {
            building.building_type.as_str()
        };
        titleize(raw)
    }

    pub fn boon_text(&self, boon: &BuildingBoon) -> String {
        let mut parts = Vec::new();
        if boon.scrap_per_sec > 0.0 {
            parts.push(format!("+{:.1} scrap/s", boon.scrap_per_sec));
        }
        if boon.food_per_sec > 0.0 {
            parts.push(format!("+{:.1} food/s", boon.food_per_sec));
        }
        if boon.water_per_sec > 0.0 {
            parts.push(format!("+{:.1} water/s", boon.water_per_sec));
        }
        if boon.power_per_sec > 0.0 {
            parts.push(format!("+{:.1} power/s", boon.power_per_sec));
        }
        if parts.is_empty() {
            "Factory stability".to_string()
        } else {
            parts.join(", ")
        }
    }

    fn power_grid_snapshot(&self) -> PowerGridSnapshot {
        let generated =
            self.factory.power_generation() + self.unlocked_building_boon().power_per_sec;
        let tower_drain: f32 = self.towers.iter().map(|t| t.power_drain).sum();
        let used = self.factory.power_consumption() + tower_drain;
        let net = generated - used;
        let offline_towers = self.towers.iter().filter(|t| !t.is_active).count();
        let seconds_to_empty = if net < 0.0 && self.resources.power > 0.0 {
            Some(self.resources.power / -net)
        } else {
            None
        };
        PowerGridSnapshot {
            generated,
            used,
            net,
            battery: self.resources.power,
            seconds_to_empty,
            offline_towers,
        }
    }

    fn build_alerts(&self, power: &PowerGridSnapshot) -> Vec<AlertBanner> {
        let mut alerts = Vec::new();
        if power.offline_towers > 0 || (power.net < 0.0 && power.battery <= 0.0) {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: "POWER FAILURE".to_string(),
                detail: format!("{} towers offline", power.offline_towers),
                priority: 100,
            });
        } else if power.net < 0.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: "POWER DEFICIT".to_string(),
                detail: format!(
                    "{:.0}s battery reserve",
                    power.seconds_to_empty.unwrap_or(0.0)
                ),
                priority: 90,
            });
        }

        let food_seconds = self.food_seconds_remaining();
        if food_seconds <= 40.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: "FOOD SHORTAGE".to_string(),
                detail: format!("{:.0}s supply left", food_seconds),
                priority: 88,
            });
        } else if food_seconds <= 90.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: "FOOD LOW".to_string(),
                detail: format!("{:.0}s supply left", food_seconds),
                priority: 72,
            });
        }

        if self
            .wave_manager
            .enemies
            .iter()
            .any(|e| e.is_alive && e.enemy_type == EnemyType::Saboteur)
            || self
                .wave_manager
                .spawn_queue
                .iter()
                .any(|e| e.enemy_type == EnemyType::Saboteur)
        {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: "INFILTRATOR".to_string(),
                detail: "Saboteur on route".to_string(),
                priority: 82,
            });
        }

        if self.scavengers_out > 0
            && self.beacon_phase.rank() >= BeaconPhase::ScreamingBeacon.rank()
        {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: "TEAM AT RISK".to_string(),
                detail: format!("{} teams outside", self.scavengers_out),
                priority: 76,
            });
        }

        let phase_remaining = self.beacon_draw_to_next_phase();
        if self.beacon_active && phase_remaining <= 8.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: "BEACON ESCALATING".to_string(),
                detail: format!("+{:.0} draw to next phase", phase_remaining),
                priority: 70,
            });
        }

        let awareness = self.threat.awareness_level();
        if awareness >= 60.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: "MACHINE AWARE".to_string(),
                detail: format!("{:.0}% threat", awareness),
                priority: 68,
            });
        } else if awareness >= 25.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: "MACHINE WATCHING".to_string(),
                detail: format!("{:.0}% threat", awareness),
                priority: 58,
            });
        }

        if self.factory_integrity < 35.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: "FACTORY DAMAGE".to_string(),
                detail: format!("{:.0}% integrity", self.factory_integrity),
                priority: 64,
            });
        }

        alerts
    }

    fn suggest_next_action(&self, data: &GameData, power: &PowerGridSnapshot) -> SuggestedAction {
        if power.offline_towers > 0 || (power.net < 0.0 && power.battery <= 0.0) {
            if let Some(idx) = self.best_power_system() {
                let building = &self.map_state.buildings[idx];
                return building_action(
                    idx,
                    self.building_display_name(building),
                    building,
                    "Restore grid headroom",
                );
            }
            return SuggestedAction {
                label: "Restore power before building more towers".to_string(),
                detail: format!("{} towers are offline", power.offline_towers),
                cost: "Cost: repair power systems".to_string(),
                risk: "Risk: defenses inactive".to_string(),
                target: AdviceTarget::None,
            };
        }

        if self.food_seconds_remaining() <= 40.0 {
            if let Some(idx) = self.best_food_system() {
                let building = &self.map_state.buildings[idx];
                return building_action(
                    idx,
                    self.building_display_name(building),
                    building,
                    "Stabilize food production",
                );
            }
        }

        if let Some(idx) = self.best_income_system(true) {
            let building = &self.map_state.buildings[idx];
            return building_action(
                idx,
                self.building_display_name(building),
                building,
                "Increase resource income",
            );
        }

        if self.towers.is_empty() {
            if let Some(def) = data.tower_def_by_id("ballistic_turret") {
                if self.resources.scrap >= def.cost_scrap {
                    return SuggestedAction {
                        label: "Build Ballistic Turret".to_string(),
                        detail: "Place it on a powered pad covering the main route".to_string(),
                        cost: format!(
                            "Cost: {:.0} scrap, {:.0} power",
                            def.cost_scrap, def.cost_power
                        ),
                        risk: "Risk: +0.50 noise".to_string(),
                        target: AdviceTarget::TowerDef(def.id.clone()),
                    };
                }
            }
        }

        if let Some(idx) = self.best_repaired_system() {
            let building = &self.map_state.buildings[idx];
            return building_action(
                idx,
                self.building_display_name(building),
                building,
                "Turn repaired machinery online",
            );
        }

        if let Some(idx) = self.best_powerable_pad() {
            let slot = &self.map_state.slots[idx];
            return SuggestedAction {
                label: format!("Power {}", slot.id),
                detail: "Use this pad for stronger lane coverage".to_string(),
                cost: format!("Cost: {:.0} scrap", slot.power_cost),
                risk: slot
                    .opens_entrance
                    .as_ref()
                    .map(|p| format!("Risk: opens {}", self.path_display_name(p)))
                    .unwrap_or_else(|| "Risk: none".to_string()),
                target: AdviceTarget::Slot(idx),
            };
        }

        if !self.beacon_active && !self.towers.is_empty() {
            return SuggestedAction {
                label: "Start Beacon".to_string(),
                detail: "Draw enemies while scavengers gather resources".to_string(),
                cost: "Cost: starts Wave 1 timer".to_string(),
                risk: format!("Risk: {}", self.beacon_phase.label()),
                target: AdviceTarget::BeaconStart,
            };
        }

        if self.beacon_active && self.beacon_phase.rank() >= BeaconPhase::ScreamingBeacon.rank() {
            if self.scavengers_out > 0 {
                return SuggestedAction {
                    label: "Recall Scavengers".to_string(),
                    detail: "Bank resources before the signal gets worse".to_string(),
                    cost: "Cost: stops field gains".to_string(),
                    risk: "Risk: teams outside".to_string(),
                    target: AdviceTarget::ScavengerRecall,
                };
            }
            if self.current_wave >= 1 {
                return SuggestedAction {
                    label: "Shutdown Beacon".to_string(),
                    detail: "Stop new waves once the lane is stable".to_string(),
                    cost: "Cost: ends scavenging".to_string(),
                    risk: "Risk: active enemies remain".to_string(),
                    target: AdviceTarget::BeaconShutdown,
                };
            }
        }

        if let Some(idx) = self.best_income_system(false) {
            let building = &self.map_state.buildings[idx];
            return building_action(
                idx,
                self.building_display_name(building),
                building,
                "Repair the next damaged system",
            );
        }

        SuggestedAction {
            label: "Inspect Factory Console".to_string(),
            detail: "Review available upgrades and unlocked machinery".to_string(),
            cost: "Cost: none".to_string(),
            risk: "Risk: none".to_string(),
            target: AdviceTarget::None,
        }
    }

    fn wave_preview_card(&self) -> WavePreviewCard {
        if self.wave_manager.wave_active {
            let mut counts = Vec::new();
            for enemy in self.wave_manager.enemies.iter().filter(|e| e.is_alive) {
                bump_enemy_count(&mut counts, enemy.enemy_type.clone());
            }
            for entry in &self.wave_manager.spawn_queue {
                bump_enemy_count(&mut counts, entry.enemy_type.clone());
            }
            let dangerous = if counts.iter().any(|(t, _)| *t == EnemyType::Commander) {
                "Prepare burst damage"
            } else if counts.iter().any(|(t, _)| *t == EnemyType::Saboteur) {
                "Add slow/disruption"
            } else if counts.iter().any(|(t, _)| *t == EnemyType::HeavyUnit) {
                "Add laser damage"
            } else {
                "Hold kinetic coverage"
            };
            return WavePreviewCard {
                title: "LIVE PRESSURE".to_string(),
                counts,
                path_split: self.live_path_split(),
                recommendation: dangerous.to_string(),
                alive_count: self.wave_manager.alive_count(),
                queued_count: self.wave_manager.spawn_queue.len(),
                live: true,
            };
        }

        let counts = self.wave_preview_counts();
        WavePreviewCard {
            title: "NEXT WAVE".to_string(),
            path_split: self.preview_path_split(),
            recommendation: recommendation_for_counts(&counts),
            counts,
            alive_count: 0,
            queued_count: 0,
            live: false,
        }
    }

    fn wave_preview_counts(&self) -> Vec<(EnemyType, usize)> {
        let entries = self.next_preview_entries();
        let mut counts = Vec::new();
        for entry in entries {
            bump_enemy_count(&mut counts, entry.enemy_type);
        }
        counts
    }

    fn next_preview_entries(&self) -> Vec<crate::engine::wave::PreviewSpawnEntry> {
        let spawn_points: Vec<(String, Vec2)> = self
            .map_state
            .active_paths()
            .iter()
            .map(|p| (p.id.clone(), p.entrance))
            .collect();
        if spawn_points.is_empty() {
            return Vec::new();
        }
        let budget_bonus = if self.beacon_active {
            self.beacon_start_difficulty_bonus
        } else {
            self.compute_beacon_start_difficulty_bonus()
        };
        preview_wave_entries(
            self.current_wave + 1,
            &self.enemy_defs,
            self.base_health_scale_per_wave,
            self.threat.awareness_level(),
            self.beacon_phase.tier_floor(),
            self.constants.waves.budget_multiplier + budget_bonus,
            self.beacon_phase == BeaconPhase::TerminalHowl,
            self.constants.waves.budget_base,
            self.constants.waves.budget_per_wave,
            self.constants.waves.commander_every,
            self.constants.threat.budget_divisor,
            self.constants.threat.health_mult_per_awareness,
            &spawn_points,
        )
    }

    fn preview_path_split(&self) -> Vec<(String, f32)> {
        split_entries(
            self.next_preview_entries()
                .into_iter()
                .map(|entry| entry.path_id)
                .collect(),
        )
    }

    fn live_path_split(&self) -> Vec<(String, f32)> {
        let mut ids = Vec::new();
        for enemy in self.wave_manager.enemies.iter().filter(|e| e.is_alive) {
            ids.push(enemy.path_id.clone());
        }
        for entry in &self.wave_manager.spawn_queue {
            ids.push(entry.path_id.clone());
        }
        split_entries(ids)
    }

    pub fn best_slot_for_tower_id(&self, tower_id: &str, data: &GameData) -> Option<usize> {
        data.tower_def_by_id(tower_id)
            .and_then(|def| self.best_slot_for_tower(def))
    }

    pub fn best_slot_for_tower(&self, tower_def: &TowerDef) -> Option<usize> {
        let range = self.effective_tower_range(tower_def.base_range);
        self.map_state
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| {
                self.map_state.is_slot_visible(slot)
                    && slot.state == SlotState::Powered
                    && slot.tower_index.is_none()
            })
            .map(|(idx, slot)| {
                let covered = self.covered_paths_for_range(slot.position, range).len() as f32;
                let route_dist = self.distance_to_active_route(slot.position);
                let core_dist = (slot.position - self.map_state.factory_core).length();
                (idx, covered, route_dist, core_dist)
            })
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
                    .then_with(|| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal))
            })
            .map(|(idx, _, _, _)| idx)
    }

    fn best_power_system(&self) -> Option<usize> {
        self.map_state
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| self.map_state.is_building_visible(b) && b.boon.power_per_sec > 0.0)
            .filter(|(_, b)| {
                matches!(b.state, BuildingState::Broken | BuildingState::Repaired)
                    && self.can_progress_building(b)
            })
            .max_by(|(_, a), (_, b)| {
                a.boon
                    .power_per_sec
                    .partial_cmp(&b.boon.power_per_sec)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    fn best_food_system(&self) -> Option<usize> {
        self.map_state
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| self.map_state.is_building_visible(b))
            .filter(|(_, b)| b.boon.food_per_sec > 0.0 || b.boon.water_per_sec > 0.0)
            .filter(|(_, b)| {
                matches!(b.state, BuildingState::Broken | BuildingState::Repaired)
                    && self.can_progress_building(b)
            })
            .max_by(|(_, a), (_, b)| {
                (a.boon.food_per_sec + a.boon.water_per_sec)
                    .partial_cmp(&(b.boon.food_per_sec + b.boon.water_per_sec))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    fn best_income_system(&self, affordable_only: bool) -> Option<usize> {
        self.map_state
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| self.map_state.is_building_visible(b))
            .filter(|(_, b)| b.state == BuildingState::Broken)
            .filter(|(_, b)| !affordable_only || self.resources.scrap >= b.repair_cost)
            .max_by(|(_, a), (_, b)| {
                building_score(a)
                    .partial_cmp(&building_score(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    fn best_repaired_system(&self) -> Option<usize> {
        self.map_state
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| self.map_state.is_building_visible(b))
            .filter(|(_, b)| b.state == BuildingState::Repaired)
            .filter(|(_, b)| self.resources.scrap >= b.power_cost)
            .max_by(|(_, a), (_, b)| {
                building_score(a)
                    .partial_cmp(&building_score(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    fn best_powerable_pad(&self) -> Option<usize> {
        self.map_state
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| {
                self.map_state.is_slot_visible(slot)
                    && slot.state == SlotState::Cleared
                    && self.resources.scrap >= slot.power_cost
            })
            .map(|(idx, slot)| {
                let route_dist = self.distance_to_active_route(slot.position);
                let core_dist = (slot.position - self.map_state.factory_core).length();
                (idx, route_dist, core_dist)
            })
            .min_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            })
            .map(|(idx, _, _)| idx)
    }

    fn can_progress_building(&self, building: &MapBuilding) -> bool {
        match building.state {
            BuildingState::Broken => self.resources.scrap >= building.repair_cost,
            BuildingState::Repaired => self.resources.scrap >= building.power_cost,
            BuildingState::Powered | BuildingState::Disabled => false,
        }
    }

    fn invalid_pad_reason(&self, slot: &MapSlot, can_afford: bool) -> Option<InvalidPadReason> {
        if slot.tower_index.is_some() {
            return Some(InvalidPadReason::Occupied);
        }
        if slot.state != SlotState::Powered {
            return Some(InvalidPadReason::NeedsPower);
        }
        if !can_afford {
            return Some(InvalidPadReason::TooExpensive);
        }
        None
    }

    fn has_power_buffer_for(&self, def: &TowerDef) -> bool {
        let generated =
            self.factory.power_generation() + self.unlocked_building_boon().power_per_sec;
        let current_drain: f32 = self.factory.power_consumption()
            + self
                .towers
                .iter()
                .filter(|t| t.is_active)
                .map(|t| t.power_drain)
                .sum::<f32>();
        let net_after = generated - current_drain - def.cost_power;
        net_after >= 0.0
            || self.resources.power >= self.constants.economy.power_buffer_min_for_build
    }

    fn food_seconds_remaining(&self) -> f32 {
        let consumption =
            self.population.count as f32 * self.constants.population.food_per_person_per_sec;
        let production = self.unlocked_building_boon().food_per_sec;
        let net = consumption - production;
        if net <= 0.0 {
            f32::INFINITY
        } else {
            self.population.food_supply / net
        }
    }

    pub fn beacon_draw_to_next_phase(&self) -> f32 {
        let next = match self.beacon_phase {
            BeaconPhase::WarmSignal => 16.0,
            BeaconPhase::SustainedCall => 36.0,
            BeaconPhase::ScreamingBeacon => 61.0,
            BeaconPhase::TerminalHowl => 70.0,
        };
        (next - self.beacon_strength).max(0.0)
    }

    fn distance_to_active_route(&self, pos: Vec2) -> f32 {
        let mut best = f32::MAX;
        for path in self.map_state.active_paths_limited() {
            for pair in path.points.windows(2) {
                best = best.min(distance_to_segment(pos, pair[0], pair[1]));
            }
        }
        if best == f32::MAX {
            (pos - self.map_state.factory_core).length()
        } else {
            best
        }
    }
}

fn objective_for_action(action: &SuggestedAction) -> String {
    match action.target {
        AdviceTarget::Building(_) => "Bring the next machine online".to_string(),
        AdviceTarget::Slot(_) => "Prepare a stronger firing position".to_string(),
        AdviceTarget::TowerDef(_) => "Establish first lane coverage".to_string(),
        AdviceTarget::BeaconStart => "Start a controlled resource draw".to_string(),
        AdviceTarget::BeaconShutdown => "End the beacon before risk spikes".to_string(),
        AdviceTarget::ScavengerRecall => "Recover field teams safely".to_string(),
        AdviceTarget::None => "Stabilize the command deck".to_string(),
    }
}

fn building_action(
    idx: usize,
    name: String,
    building: &MapBuilding,
    purpose: &str,
) -> SuggestedAction {
    match building.state {
        BuildingState::Broken => SuggestedAction {
            label: format!("Repair {}", name),
            detail: purpose.to_string(),
            cost: format!("Cost: {:.0} scrap", building.repair_cost),
            risk: format!("Risk: +{:.2} noise/s", building.threat_per_sec),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Repaired => SuggestedAction {
            label: format!("Power {}", name),
            detail: purpose.to_string(),
            cost: format!("Cost: {:.0} scrap", building.power_cost),
            risk: format!("Risk: +{:.2} noise/s", building.threat_per_sec),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Powered => SuggestedAction {
            label: format!("Inspect {}", name),
            detail: purpose.to_string(),
            cost: "Cost: online".to_string(),
            risk: format!("Risk: +{:.2} noise/s", building.threat_per_sec),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Disabled => SuggestedAction {
            label: format!("Inspect {}", name),
            detail: "System disabled".to_string(),
            cost: "Cost: locked".to_string(),
            risk: "Risk: offline".to_string(),
            target: AdviceTarget::Building(idx),
        },
    }
}

fn building_score(building: &MapBuilding) -> f32 {
    let scrap = building.boon.scrap_per_sec * 100.0;
    let power = building.boon.power_per_sec * 80.0;
    let food = building.boon.food_per_sec * 70.0;
    let water = building.boon.water_per_sec * 60.0;
    let preferred = if building.building_type.contains("scrap") {
        1000.0
    } else {
        0.0
    };
    preferred + scrap + power + food + water - building.repair_cost * 0.01
}

fn severity_rank(severity: AlertSeverity) -> u32 {
    match severity {
        AlertSeverity::Info => 0,
        AlertSeverity::Warning => 1,
        AlertSeverity::Critical => 2,
    }
}

fn bump_enemy_count(counts: &mut Vec<(EnemyType, usize)>, enemy_type: EnemyType) {
    if let Some((_, count)) = counts.iter_mut().find(|(t, _)| *t == enemy_type) {
        *count += 1;
    } else {
        counts.push((enemy_type, 1));
    }
}

fn split_entries(ids: Vec<String>) -> Vec<(String, f32)> {
    if ids.is_empty() {
        return Vec::new();
    }
    let total = ids.len() as f32;
    let mut counts: Vec<(String, usize)> = Vec::new();
    for id in ids {
        if let Some((_, count)) = counts.iter_mut().find(|(pid, _)| *pid == id) {
            *count += 1;
        } else {
            counts.push((id, 1));
        }
    }
    counts
        .into_iter()
        .map(|(id, count)| (id, count as f32 / total))
        .collect()
}

fn recommendation_for_counts(counts: &[(EnemyType, usize)]) -> String {
    if counts.iter().any(|(t, _)| *t == EnemyType::Commander) {
        "Prepare burst damage".to_string()
    } else if counts.iter().any(|(t, _)| *t == EnemyType::Saboteur) {
        "Add slow/disruption".to_string()
    } else if counts.iter().any(|(t, _)| *t == EnemyType::HeavyUnit) {
        "Add laser damage".to_string()
    } else {
        "Add kinetic coverage".to_string()
    }
}

fn titleize(id: &str) -> String {
    id.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = String::new();
                    out.push(first.to_ascii_uppercase());
                    out.push_str(chars.as_str());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq <= f32::EPSILON {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (p - closest).length()
}

pub fn format_enemy_counts(counts: &[(EnemyType, usize)]) -> String {
    if counts.is_empty() {
        return "No preview".to_string();
    }
    counts
        .iter()
        .map(|(kind, count)| format!("{} {}", count, enemy_label(kind)))
        .collect::<Vec<_>>()
        .join(" / ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;

    #[test]
    fn objective_recommends_scrap_converter_repair_when_affordable() {
        let data = GameData::load();
        let state = GameplayState::new(&data);

        let advice = state.build_ui_advice(&data);

        assert!(
            advice.suggested_action.label.contains("Scrap Converter"),
            "expected scrap converter repair, got {}",
            advice.suggested_action.label
        );
    }

    #[test]
    fn objective_falls_back_to_tower_when_repairs_are_not_affordable() {
        let data = GameData::load();
        let mut state = GameplayState::new(&data);
        state.resources.scrap = 25.0;

        let advice = state.build_ui_advice(&data);

        assert!(matches!(
            advice.suggested_action.target,
            AdviceTarget::TowerDef(_)
        ));
    }

    #[test]
    fn alert_builder_emits_power_and_food_thresholds() {
        let data = GameData::load();
        let mut state = GameplayState::new(&data);
        state.resources.power = 0.0;
        state.population.food_supply = 1.0;
        state.towers.push(crate::engine::tower::Tower::new(
            crate::engine::tower::TowerType::Ballistic,
            "ballistic_turret".to_string(),
            vec2(100.0, 100.0),
            120.0,
            10.0,
            1.0,
            200.0,
            25.0,
            WHITE,
        ));
        state.towers[0].is_active = false;

        let advice = state.build_ui_advice(&data);

        assert!(advice.alerts.iter().any(|a| a.label == "POWER FAILURE"));
        assert!(advice.alerts.iter().any(|a| a.label == "FOOD SHORTAGE"));
    }

    #[test]
    fn placement_coverage_detects_active_path_intersection() {
        let data = GameData::load();
        let state = GameplayState::new(&data);
        let path = state.map_state.active_paths()[0];
        let center = path.points[0];

        let covered = state.covered_paths_for_range(center, 10.0);

        assert!(covered.iter().any(|id| id == &path.id));
    }
}
