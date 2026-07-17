//! What's coming next: composition, route split, and a counter-play hint for
//! either the live wave or the one being drawn.

use crate::engine::beacon::BeaconPhase;
use crate::engine::enemy::EnemyType;
use crate::engine::wave::{preview_wave_entries, PreviewSpawnEntry};
use macroquad::prelude::*;

use super::super::helpers::enemy_label;
use super::super::GameplayState;
use super::WavePreviewCard;

impl GameplayState {
    /// Live pressure once a wave is running (alive + still queued), otherwise a
    /// roll of the wave that would come next.
    pub(super) fn wave_preview_card(&self) -> WavePreviewCard {
        if !self.wave_manager.wave_active {
            return WavePreviewCard {
                counts: self.wave_preview_counts(),
            };
        }

        let mut counts = Vec::new();
        for enemy in self.wave_manager.enemies.iter().filter(|e| e.is_alive) {
            bump_enemy_count(&mut counts, enemy.enemy_type.clone());
        }
        for entry in &self.wave_manager.spawn_queue {
            bump_enemy_count(&mut counts, entry.enemy_type.clone());
        }
        WavePreviewCard { counts }
    }

    pub(super) fn wave_preview_counts(&self) -> Vec<(EnemyType, usize)> {
        let mut counts = Vec::new();
        for entry in self.next_preview_entries() {
            bump_enemy_count(&mut counts, entry.enemy_type);
        }
        counts
    }

    /// Roll the wave the machines *would* send next, without committing it.
    fn next_preview_entries(&self) -> Vec<PreviewSpawnEntry> {
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
            self.constants.threat.tier_2_awareness,
            self.constants.threat.tier_3_awareness,
            &spawn_points,
            &self.wave_adaptation(),
        )
    }
}

fn bump_enemy_count(counts: &mut Vec<(EnemyType, usize)>, enemy_type: EnemyType) {
    if let Some((_, count)) = counts.iter_mut().find(|(t, _)| *t == enemy_type) {
        *count += 1;
    } else {
        counts.push((enemy_type, 1));
    }
}

pub fn format_enemy_counts(counts: &[(EnemyType, usize)]) -> String {
    let t = &crate::data::strings::text().wave_preview;
    if counts.is_empty() {
        return t.no_preview.clone();
    }
    counts
        .iter()
        .map(|(kind, count)| {
            crate::data::strings::fill(
                &t.entry,
                &[("count", &count.to_string()), ("name", enemy_label(kind))],
            )
        })
        .collect::<Vec<_>>()
        .join(&t.separator)
}
