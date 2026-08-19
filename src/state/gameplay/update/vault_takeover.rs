//! Final vault operation: a three-stage upload under a locked terminal beacon.

use crate::data::strings::text;
use crate::engine::beacon::BeaconPhase;

use super::super::GameplayState;

impl GameplayState {
    pub fn vault_takeover_ready(&self) -> bool {
        !self.vault_takeover.active
            && !self.vault_takeover.upload_complete
            && !self.beacon_active
            && self.wave_manager.spawn_queue.is_empty()
            && self.wave_manager.alive_count() == 0
            && self
                .factory
                .sectors
                .iter()
                .all(|sector| sector.unlocked && sector.integrity > 0.0)
            && self.factory.is_sector_active("ai_vault")
    }

    pub(crate) fn start_vault_takeover(&mut self) {
        if !self.vault_takeover_ready() {
            return;
        }
        self.vault_takeover.start();
        self.beacon_active = true;
        self.shutdown_triggered = false;
        self.beacon_phase = BeaconPhase::TerminalHowl;
        self.between_waves = true;
        self.wave_timer = self.constants.gameplay.beacon_start_delay;
        self.beacon_start_difficulty_bonus = self.compute_beacon_start_difficulty_bonus()
            + self.machine_escalation
            + self.constants.vault.wave_budget_bonus;
        self.scavenger_recall_active = false;
        self.scavengers_out = 0;
        self.push_notification(text().notifications.vault_takeover_started.clone());
        self.autosave();
    }

    pub(super) fn update_vault_takeover(&mut self, dt: f32) {
        if !self.vault_takeover.active || !self.factory.is_sector_active("ai_vault") {
            return;
        }
        self.beacon_phase = BeaconPhase::TerminalHowl;
        self.threat
            .add_corruption(self.constants.vault.corruption_per_sec * dt);
        let total = self.constants.vault.stage_seconds * 3.0;
        if self.vault_takeover.tick(dt, total) {
            self.beacon_active = false;
            self.between_waves = false;
            self.push_notification(text().notifications.vault_upload_complete.clone());
            self.autosave();
        }
    }
}
