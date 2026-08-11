//! The beacon cycle: raise it, accrue evacuees while it screams, then bank the
//! ledger and escalate the machines when it goes dark.

use crate::data::strings::{fill, text};
use crate::engine::beacon::{phase_from_strength, BeaconPhase};
use macroquad_toolkit::rng;

use super::super::{GameplayState, SalvageReport};

impl GameplayState {
    /// Survivors/sec the beacon is currently evacuating, scaling with phase: a
    /// louder beacon draws more machines away, so more people escape elsewhere.
    fn evacuation_rate(&self) -> f32 {
        let evac = &self.constants.evacuation;
        match self.beacon_phase {
            BeaconPhase::WarmSignal => evac.warm_rate,
            BeaconPhase::SustainedCall => evac.sustained_rate,
            BeaconPhase::ScreamingBeacon => evac.screaming_rate,
            BeaconPhase::TerminalHowl => evac.terminal_rate,
        }
    }

    /// Accrue evacuees into the pending pool while the beacon is up. Nothing is
    /// banked until a clean shutdown — hold too long and lose it all to defeat.
    pub(super) fn update_evacuation(&mut self, dt: f32) {
        self.pending_evacuees += self.evacuation_rate() * dt;
    }

    /// Beacon shut down and the field is clear: bank any remaining scavengers,
    /// commit this cycle's evacuees to the persistent ledger, ratchet up the
    /// permanent assault escalation, produce the salvage report, and drop back
    /// to base-build mode so the player can rebuild and raise the beacon again.
    pub(super) fn finish_beacon_cycle(&mut self) {
        self.resolve_remaining_scavengers();

        // Bank pending evacuees; carry the fractional remainder to the next
        // cycle so slow low-phase accrual is never silently lost to rounding.
        let banked = self.pending_evacuees.floor();
        let banked_u = banked as u32;
        self.survivors_evacuated += banked_u;
        self.pending_evacuees -= banked;
        self.announce_evac_milestones();

        // The machines learn the beacon is bait: every completed cycle makes the
        // next assault permanently heavier, decaying the safe low-phase farm.
        self.machine_escalation += self.constants.waves.escalation_per_cycle;

        self.beacon_cycles_completed += 1;
        self.salvage_report = Some(self.build_salvage_report(banked_u));
        self.beacon_start_difficulty_bonus = 0.0;
        self.autosave();
    }

    /// Push a milestone line each time the banked total crosses a new interval,
    /// giving the open-ended campaign a felt sense of a rising goal.
    fn announce_evac_milestones(&mut self) {
        let interval = self.constants.evacuation.milestone_interval.max(1);
        while self.survivors_evacuated >= self.next_evac_milestone {
            let reached = self.next_evac_milestone;
            self.push_notification(fill(
                &text().notifications.evac_milestone,
                &[("n", &reached.to_string())],
            ));
            self.next_evac_milestone += interval;
        }
    }

    /// Force any field teams still out to return home when the beacon goes dark,
    /// each risking loss at the shutdown loss rate.
    fn resolve_remaining_scavengers(&mut self) {
        let still_out = self.scavengers_out;
        if still_out == 0 {
            return;
        }
        let mut lost = 0u32;
        for _ in 0..still_out {
            if rng::chance(self.constants.scavenger.shutdown_loss_chance) {
                lost += 1;
            }
        }
        let returned = still_out - lost;
        self.scavengers_out = 0;
        self.scavengers_lost += lost;
        self.scavengers_returned += returned;
        self.scavenger_recall_active = false;
    }

    /// Subtract the cycle baseline so the report shows only this window's gains.
    fn build_salvage_report(&self, evacuated_cycle: u32) -> SalvageReport {
        let b = &self.cycle_baseline;
        SalvageReport {
            cycle: self.beacon_cycles_completed,
            waves: self.current_wave.saturating_sub(b.wave),
            scavengers_sent: self.scavengers_sent.saturating_sub(b.sent),
            scavengers_returned: self.scavengers_returned.saturating_sub(b.returned),
            scavengers_lost: self.scavengers_lost.saturating_sub(b.lost),
            scrap: (self.scavenger_scrap_gained - b.scrap).max(0.0),
            food: (self.scavenger_food_gained - b.food).max(0.0),
            population: self
                .scavenger_population_gained
                .saturating_sub(b.population),
            beacon_phase: self.beacon_phase.clone(),
            survivors_evacuated_cycle: evacuated_cycle,
            survivors_evacuated_total: self.survivors_evacuated,
            escalation_pct: self.machine_escalation * 100.0,
        }
    }

    /// Dismiss the salvage report and return to base-build mode for the next
    /// beacon cycle.
    pub(crate) fn dismiss_salvage_report(&mut self) {
        self.salvage_report = None;
        self.shutdown_triggered = false;
        self.beacon_active = false;
        self.between_waves = true;
        self.wave_timer = self.constants.ui.wave_start_delay;
        self.push_notification(text().notifications.field_secured.clone());
    }

    /// Beacon draw is a function of how much factory is lit up; crossing a
    /// threshold escalates the phase permanently for this cycle.
    pub(crate) fn update_beacon(&mut self) {
        let unlocked = self.factory.unlocked_count() as f32;
        let power_throughput = self.factory.power_generation(&self.constants.economy);
        let ai_vault_tier = if self.factory.is_sector_active("ai_vault") {
            1.0
        } else {
            0.0
        };
        let population = self.population.count as f32;
        let evac = &self.constants.evacuation;

        self.beacon_strength = (unlocked * evac.strength_per_unlocked_sector)
            + (power_throughput * evac.strength_per_power_throughput)
            + (ai_vault_tier * evac.strength_per_ai_vault)
            + (population * evac.strength_per_population);

        let next_phase = phase_from_strength(self.beacon_strength, evac);
        if next_phase.rank() > self.beacon_phase.rank() {
            if let Some(flavor) = beacon_phase_flavor(&next_phase) {
                self.push_notification(flavor.to_string());
            }
            self.beacon_phase = next_phase;
        }
    }

    /// Teams trickle home one per recall interval, each rolling that phase's
    /// payout — or, at terminal howl, its chance of never coming back.
    pub(super) fn update_scavenger_recall(&mut self, dt: f32) {
        if !self.scavenger_recall_active {
            return;
        }
        if self.scavengers_out == 0 {
            self.scavenger_recall_active = false;
            return;
        }

        self.scavenger_recall_timer += dt;
        if self.scavenger_recall_timer < self.constants.scavenger.recall_interval {
            return;
        }
        self.scavenger_recall_timer = 0.0;

        let (scrap, food, pop_gain) = match self.beacon_phase {
            BeaconPhase::WarmSignal => (
                self.constants.scavenger.warm_scrap,
                self.constants.scavenger.warm_food,
                0,
            ),
            BeaconPhase::SustainedCall => {
                let pop = if rng::chance(self.constants.scavenger.sustained_pop_chance) {
                    1
                } else {
                    0
                };
                (
                    self.constants.scavenger.sustained_scrap,
                    self.constants.scavenger.sustained_food,
                    pop,
                )
            }
            BeaconPhase::ScreamingBeacon => (
                self.constants.scavenger.screaming_scrap,
                self.constants.scavenger.screaming_food,
                self.constants.scavenger.screaming_pop_gain,
            ),
            BeaconPhase::TerminalHowl => {
                if rng::chance(self.constants.scavenger.terminal_loss_chance) {
                    self.scavengers_out = self.scavengers_out.saturating_sub(1);
                    self.scavengers_lost += 1;
                    self.push_notification(text().notifications.team_lost.clone());
                    return;
                }
                (
                    self.constants.scavenger.terminal_scrap,
                    self.constants.scavenger.terminal_food,
                    self.constants.scavenger.terminal_pop_gain,
                )
            }
        };

        self.scavengers_out = self.scavengers_out.saturating_sub(1);
        self.scavengers_returned += 1;
        self.resources.scrap += scrap;
        self.population.food_supply += food;
        self.population.count += pop_gain;

        self.scavenger_scrap_gained += scrap;
        self.scavenger_food_gained += food;
        self.scavenger_population_gained += pop_gain;

        self.push_notification(fill(
            &text().notifications.team_returned,
            &[
                ("scrap", &format!("{scrap:.0}")),
                ("food", &format!("{food:.0}")),
                ("pop", &pop_gain.to_string()),
            ],
        ));
    }
}

/// Short narrative line pushed as a notification when the beacon escalates to a
/// new phase. `WarmSignal` is the opening state, so it has no line.
fn beacon_phase_flavor(phase: &BeaconPhase) -> Option<&'static str> {
    let flavor = &text().beacon.flavor;
    match phase {
        BeaconPhase::WarmSignal => None,
        BeaconPhase::SustainedCall => Some(flavor.sustained_call.as_str()),
        BeaconPhase::ScreamingBeacon => Some(flavor.screaming_beacon.as_str()),
        BeaconPhase::TerminalHowl => Some(flavor.terminal_howl.as_str()),
    }
}

#[cfg(test)]
mod tests;
