//! The gameplay frame loop.
//!
//! [`GameplayState::update`] is the single ordered entry point; the per-system
//! work lives in [`systems`], the beacon/evacuation economy in [`beacon_cycle`],
//! and the headless balance harness in [`survival_proof`].

mod beacon_cycle;
mod survival_proof;
mod systems;
mod vault_takeover;

use crate::state::{RunSummary, StateTransition};
use macroquad::prelude::*;

use super::GameplayState;

impl GameplayState {
    /// Advance one frame. Returns a transition only when the run ends.
    pub fn update(&mut self) -> Option<StateTransition> {
        if self.end_campaign_requested {
            return Some(StateTransition::ToResults {
                summary: self.build_run_summary(true, false),
            });
        }

        self.handle_camera_input();

        // The intro card and salvage report freeze the sim; each handles its own
        // buttons during draw.
        if self.show_intro
            || self.show_workforce
            || self.salvage_report.is_some()
            || self.pending_depth_directive.is_some()
        {
            return None;
        }

        self.handle_input();
        if self.paused {
            return None;
        }

        let dt = get_frame_time() * self.time_scale;
        self.validate_selection();
        self.update_presentation(dt);

        if self.beacon_active {
            self.update_beacon();
            self.update_evacuation(dt);
        }

        self.update_wave_timers(dt);
        self.start_wave_if_ready();
        self.update_scavenger_recall(dt);
        self.tick_wave(dt);

        self.update_combat(dt);
        self.update_power(dt);
        self.update_building_boons(dt);
        self.update_population(dt);
        self.update_threat(dt);
        self.update_vault_takeover(dt);
        self.update_factory();
        self.update_notifications(dt);
        self.update_wave_flash(dt);
        self.update_coach();

        if self.vault_takeover.upload_complete && self.enemies_cleared() {
            return Some(StateTransition::ToResults {
                summary: self.build_run_summary(true, true),
            });
        }

        // Shutdown only resolves once the field is actually clear, so the player
        // still has to survive whatever the beacon already drew in.
        if self.shutdown_triggered && self.enemies_cleared() {
            self.finish_beacon_cycle();
            return None;
        }

        if self.is_game_over() {
            return Some(StateTransition::ToResults {
                summary: self.build_run_summary(false, false),
            });
        }

        None
    }

    fn handle_camera_input(&mut self) {
        let dt = get_frame_time();
        self.camera.update(dt, false);
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.camera.screen_to_world(screen_pos)
    }

    fn handle_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            if self.show_workforce {
                self.show_workforce = false;
            } else if self.show_settings {
                self.show_settings = false;
            } else if self.placing_tower.is_some() {
                self.placing_tower = None;
            } else {
                self.paused = !self.paused;
            }
        }

        if is_key_pressed(KeyCode::Space) {
            self.time_scale = if self.time_scale > 1.0 {
                1.0
            } else {
                self.constants.gameplay.speed_multiplier
            };
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            self.placing_tower = None;
            self.selected_tower = None;
            self.selected_slot = None;
            self.selected_building = None;
            self.selected_core = false;
            self.selected_upgrade = None;
        }
    }

    /// Drop selections that no longer point at anything real — towers can be
    /// destroyed and buildings can re-lock underneath the cursor.
    fn validate_selection(&mut self) {
        if let Some(idx) = self.selected_tower {
            if idx >= self.towers.len() {
                self.selected_tower = None;
            }
        }
        if let Some(idx) = self.selected_building {
            let unlocked = self
                .map_state
                .buildings
                .get(idx)
                .map(|b| self.is_building_unlocked(b))
                .unwrap_or(false);
            if !unlocked {
                self.selected_building = None;
            }
        }
    }

    fn enemies_cleared(&self) -> bool {
        self.wave_manager.spawn_queue.is_empty() && self.wave_manager.alive_count() == 0
    }

    fn is_game_over(&self) -> bool {
        self.population.count == 0
            || self.factory_integrity <= 0.0
            || (self.vault_takeover.active && !self.factory.is_sector_active("ai_vault"))
    }

    fn build_run_summary(&self, survived: bool, campaign_won: bool) -> RunSummary {
        RunSummary {
            waves_survived: self.current_wave,
            beacon_phase: self.beacon_phase.clone(),
            scavengers_sent: self.scavengers_sent,
            scavengers_returned: self.scavengers_returned,
            scavengers_lost: self.scavengers_lost,
            scavenger_scrap: self.scavenger_scrap_gained,
            scavenger_food: self.scavenger_food_gained,
            scavenger_population: self.scavenger_population_gained,
            factory_online: self.factory_online_count(),
            population_surviving: self.population.count,
            shutdown_triggered: survived,
            campaign_won,
            survivors_evacuated: self.survivors_evacuated
                + if campaign_won {
                    self.pending_evacuees.floor() as u32
                } else {
                    0
                },
            // A defeat forfeits the current beacon window's un-banked evacuees.
            evacuees_lost: if survived {
                0
            } else {
                self.pending_evacuees.floor() as u32
            },
        }
    }

    pub(crate) fn push_notification(&mut self, text: String) {
        self.notifications.push(super::Notification {
            text,
            ttl: self.constants.ui.notification_ttl,
        });
    }
}
