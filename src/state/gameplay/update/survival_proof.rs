//! Headless balance harness: drives a scripted defense at a fixed timestep so a
//! test can assert the game is still winnable. Not reachable in normal play.

use crate::data::GameData;
use crate::engine::map::SlotState;
use crate::state::StateTransition;

use super::super::GameplayState;

impl GameplayState {
    /// Put the state into deterministic proof mode: no autosave, no dodge/skip
    /// rolls, and the whole map revealed.
    pub(crate) fn enable_survival_proof(&mut self) {
        self.autosave_enabled = false;
        self.survival_proof_active = true;
        self.wave_manager.enemy_tuning.scout_dodge_chance = 0.0;
        self.wave_manager.enemy_tuning.saboteur_skip_chance = 0.0;
        // The proof exercises combat math over the full route; progressive
        // disclosure would truncate enemy paths at the revealed frontier.
        self.map_state.reveal_all_sections();
    }

    /// Advance the scripted run by a batch of fixed steps, returning a result
    /// transition once it reaches wave 10 or dies trying.
    pub(crate) fn update_survival_proof(&mut self, data: &GameData) -> Option<StateTransition> {
        if !self.survival_proof_active {
            return self.update();
        }

        self.paused = false;
        self.wave_manager.enemy_tuning.scout_dodge_chance = 0.0;
        self.wave_manager.enemy_tuning.saboteur_skip_chance = 0.0;

        for _ in 0..80 {
            self.keep_building_survival_proof_defense(data);

            if !self.beacon_active && !self.shutdown_triggered {
                self.start_beacon();
            }

            if self.current_wave >= 10
                && !self.wave_manager.wave_active
                && self.wave_manager.spawn_queue.is_empty()
            {
                self.shutdown_triggered = true;
                self.beacon_active = false;
                self.between_waves = false;
                return Some(StateTransition::ToResults {
                    summary: self.build_run_summary(true, false),
                });
            }

            let dt = 0.1;
            self.update_wave_timers(dt);
            self.start_wave_if_ready();
            self.tick_wave(dt);
            self.update_combat(dt);
            self.update_power(dt);
            self.update_building_boons(dt);
            self.update_population(dt);
            self.update_threat(dt);
            self.update_factory();
            self.update_notifications(dt);
            self.update_wave_flash(dt);

            if self.is_game_over() {
                return Some(StateTransition::ToResults {
                    summary: self.build_run_summary(false, false),
                });
            }
        }

        None
    }

    fn keep_building_survival_proof_defense(&mut self, data: &GameData) {
        const BUILD_PLAN: &[(&str, &str)] = &[
            ("ballistic_turret", "slot_01"),
            ("ballistic_turret", "slot_02"),
            ("ballistic_turret", "slot_03"),
            ("ballistic_turret", "slot_04"),
            ("ballistic_turret", "slot_05"),
            ("ballistic_turret", "slot_07"),
            ("ballistic_turret", "slot_09"),
            ("ballistic_turret", "slot_11"),
            ("ballistic_turret", "slot_13"),
            ("ballistic_turret", "slot_15"),
            ("ballistic_turret", "slot_17"),
        ];

        for (tower_id, slot_id) in BUILD_PLAN {
            self.try_build_survival_proof_tower(data, tower_id, slot_id);
        }
    }

    /// Build only when the grid can carry the tower outright — the proof must
    /// never brown itself out.
    fn try_build_survival_proof_tower(&mut self, data: &GameData, tower_id: &str, slot_id: &str) {
        let Some(slot_idx) = self
            .map_state
            .slots
            .iter()
            .position(|slot| slot.id == slot_id)
        else {
            return;
        };
        let slot = &self.map_state.slots[slot_idx];
        if slot.state != SlotState::Powered || slot.tower_index.is_some() {
            return;
        }
        let Some(def) = data.tower_def_by_id(tower_id) else {
            return;
        };

        let grid_generation = self.factory.power_generation(&self.constants.economy)
            + self.unlocked_building_boon().power_per_sec;
        let current_drain = self.factory.power_consumption()
            + self
                .towers
                .iter()
                .filter(|tower| tower.is_active)
                .map(|tower| tower.power_drain)
                .sum::<f32>();
        if current_drain + def.cost_power > grid_generation {
            return;
        }

        let before = self.towers.len();
        self.try_place_tower_on_slot(slot_idx, tower_id, data);
        if self.towers.len() > before {
            self.selected_tower = None;
        }
    }
}

#[cfg(test)]
mod tests;
