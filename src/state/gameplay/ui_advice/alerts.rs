//! Threshold watchers that turn raw state into ranked HUD alert banners.

use crate::data::strings::{fill, text};
use crate::engine::beacon::BeaconPhase;
use crate::engine::enemy::EnemyType;

use super::super::GameplayState;
use super::{AlertBanner, AlertSeverity, PowerGridSnapshot};

impl GameplayState {
    pub(super) fn power_grid_snapshot(&self) -> PowerGridSnapshot {
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

    pub(super) fn build_alerts(&self, power: &PowerGridSnapshot) -> Vec<AlertBanner> {
        let t = &text().alerts;
        let mut alerts = Vec::new();
        if power.offline_towers > 0 || (power.net < 0.0 && power.battery <= 0.0) {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: t.power_failure.clone(),
                detail: fill(
                    &t.power_failure_detail,
                    &[("n", &power.offline_towers.to_string())],
                ),
                priority: 100,
            });
        } else if power.net < 0.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: t.power_deficit.clone(),
                detail: fill(
                    &t.power_deficit_detail,
                    &[(
                        "secs",
                        &format!("{:.0}", power.seconds_to_empty.unwrap_or(0.0)),
                    )],
                ),
                priority: 90,
            });
        }

        let food_seconds = self.food_seconds_remaining();
        if food_seconds <= 40.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: t.food_shortage.clone(),
                detail: fill(&t.food_detail, &[("secs", &format!("{food_seconds:.0}"))]),
                priority: 88,
            });
        } else if food_seconds <= 90.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: t.food_low.clone(),
                detail: fill(&t.food_detail, &[("secs", &format!("{food_seconds:.0}"))]),
                priority: 72,
            });
        }

        if self.saboteur_inbound() {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: t.infiltrator.clone(),
                detail: t.infiltrator_detail.clone(),
                priority: 82,
            });
        }

        if self.scavengers_out > 0
            && self.beacon_phase.rank() >= BeaconPhase::ScreamingBeacon.rank()
        {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: t.team_at_risk.clone(),
                detail: fill(
                    &t.team_at_risk_detail,
                    &[("n", &self.scavengers_out.to_string())],
                ),
                priority: 76,
            });
        }

        let phase_remaining = self.beacon_draw_to_next_phase();
        if self.beacon_active && phase_remaining <= 8.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: t.beacon_escalating.clone(),
                detail: fill(
                    &t.beacon_escalating_detail,
                    &[("draw", &format!("{phase_remaining:.0}"))],
                ),
                priority: 70,
            });
        }

        let awareness = self.threat.awareness_level();
        if awareness >= 60.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: t.machine_aware.clone(),
                detail: fill(&t.awareness_detail, &[("pct", &format!("{awareness:.0}"))]),
                priority: 68,
            });
        } else if awareness >= 25.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Warning,
                label: t.machine_watching.clone(),
                detail: fill(&t.awareness_detail, &[("pct", &format!("{awareness:.0}"))]),
                priority: 58,
            });
        }

        if self.factory_integrity < 35.0 {
            alerts.push(AlertBanner {
                severity: AlertSeverity::Critical,
                label: t.factory_damage.clone(),
                detail: fill(
                    &t.factory_damage_detail,
                    &[("pct", &format!("{:.0}", self.factory_integrity))],
                ),
                priority: 64,
            });
        }

        alerts
    }

    /// A saboteur is alive on the field or still queued to spawn.
    fn saboteur_inbound(&self) -> bool {
        self.wave_manager
            .enemies
            .iter()
            .any(|e| e.is_alive && e.enemy_type == EnemyType::Saboteur)
            || self
                .wave_manager
                .spawn_queue
                .iter()
                .any(|e| e.enemy_type == EnemyType::Saboteur)
    }

    /// Seconds of food left at the current net burn, or infinity when
    /// production covers the holdout.
    pub(super) fn food_seconds_remaining(&self) -> f32 {
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
}

pub(super) fn severity_rank(severity: AlertSeverity) -> u32 {
    match severity {
        AlertSeverity::Warning => 0,
        AlertSeverity::Critical => 1,
    }
}
