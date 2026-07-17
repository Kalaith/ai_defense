//! Derived, non-serialized UI advice for the command deck.
//!
//! Nothing here mutates gameplay state (except [`GameplayState::apply_advice_focus`],
//! which only moves the selection): each frame the HUD asks for a fresh
//! [`UiAdvice`] and renders it. Split by question — [`alerts`] "what's wrong?",
//! [`suggest`] "what next?", [`selectors`] "which one?", [`wave_preview`]
//! "what's coming?", [`coverage`] "what does this pad cover?", and [`format`]
//! "what do we call it?".

mod alerts;
mod coverage;
mod format;
mod selectors;
mod suggest;
mod wave_preview;

pub use wave_preview::format_enemy_counts;

use crate::data::GameData;
use crate::engine::enemy::EnemyType;
use macroquad::prelude::*;

use super::GameplayState;

#[derive(Clone, Debug)]
pub struct UiAdvice {
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

/// What the NEXT STEP strip's FOCUS button should select.
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
pub enum AlertSeverity {
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

/// Composition of the wave the player is about to face — the live one if a wave
/// is running, otherwise the one the machines would send next.
#[derive(Clone, Debug)]
pub struct WavePreviewCard {
    pub counts: Vec<(EnemyType, usize)>,
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
        let t = &crate::data::strings::text().invalid_pad;
        match self {
            InvalidPadReason::NeedsPower => t.needs_power.as_str(),
            InvalidPadReason::Occupied => t.occupied.as_str(),
            InvalidPadReason::TooExpensive => t.too_expensive.as_str(),
        }
    }
}

impl GameplayState {
    /// Recompute the whole command deck for this frame.
    pub fn build_ui_advice(&self, data: &GameData) -> UiAdvice {
        let power = self.power_grid_snapshot();
        let wave_preview = self.wave_preview_card();
        let mut alerts = self.build_alerts(&power);
        alerts.sort_by(|a, b| {
            alerts::severity_rank(b.severity)
                .cmp(&alerts::severity_rank(a.severity))
                .then_with(|| b.priority.cmp(&a.priority))
        });

        let suggested_action = self.suggest_next_action(data, &power);
        let risk = suggested_action.risk.clone();

        UiAdvice {
            suggested_action,
            risk,
            alerts,
            power,
            wave_preview,
        }
    }

    /// Move the selection to whatever the advisor is pointing at, so FOCUS lands
    /// the player on the thing it just recommended.
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

    /// Classify every visible pad for the tower currently being placed, and work
    /// out what the pad under the cursor would cover.
    pub fn placement_preview(
        &self,
        data: &GameData,
        world_mouse: Vec2,
    ) -> Option<PlacementPreview> {
        let tower_id = self.placing_tower.as_ref()?;
        let def = data.tower_def_by_id(tower_id)?;
        let range = self.effective_tower_range(def.base_range);
        let affordable = self.resources.scrap >= def.cost_scrap && self.has_power_buffer_for(def);

        let mut valid_slots = Vec::new();
        let mut invalid_slots = Vec::new();
        let mut hovered_slot = None;
        let mut hovered_dist = self.map_state.slot_interact_radius;

        for (idx, slot) in self.map_state.slots.iter().enumerate() {
            if !self.map_state.is_slot_visible(slot) {
                continue;
            }
            match self.invalid_pad_reason(slot, affordable) {
                Some(reason) => invalid_slots.push((idx, reason)),
                None => valid_slots.push(idx),
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

    pub fn effective_tower_range(&self, base_range: f32) -> f32 {
        let range_mult = if self.factory.is_sector_active("ai_vault") {
            self.constants.tower.upgrade_range_mult
        } else {
            1.0
        };
        base_range * range_mult
    }
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
