//! Bottom-panel context for a selected tower: live stats and the upgrade path.

use crate::data::strings::{fill, text};
use crate::data::GameData;
use crate::ui::{self, ConsoleButtonState, ConsoleIcon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use super::super::GameplayState;

impl GameplayState {
    pub(super) fn draw_tower_context(&mut self, rect: Rect, idx: usize, data: &GameData) {
        self.draw_tower_context_summary(
            Rect::new(rect.x + 14.0, rect.y + 14.0, rect.w * 0.56, rect.h - 28.0),
            idx,
            data,
        );
        let Some(tower) = self.towers.get(idx) else {
            return;
        };
        let max_level_bonus =
            self.factory
                .upgrade_effect("max_tower_level_bonus", &self.upgrade_defs) as u32;
        let max_level = self.constants.tower.upgrade_max_level + max_level_bonus;
        let upgrade_cost = tower.base_scrap_cost * tower.level as f32;
        let has_research = self.factory.is_sector_active("research_lab");
        let can_upgrade =
            has_research && tower.level < max_level && self.resources.scrap >= upgrade_cost;

        let label = if !has_research {
            text().panels.requires_research.clone()
        } else if tower.level >= max_level {
            text().panels.max_level.clone()
        } else if can_upgrade {
            fill(
                &text().panels.upgrade,
                &[("n", &format!("{upgrade_cost:.0}"))],
            )
        } else {
            fill(
                &text().status.need_scrap,
                &[("n", &format!("{:.0}", upgrade_cost - self.resources.scrap))],
            )
        };
        if ui::draw_console_button(
            rect.x + rect.w - 182.0,
            rect.y + 58.0,
            166.0,
            44.0,
            &label,
            if can_upgrade {
                ConsoleButtonState::Recommended
            } else {
                ConsoleButtonState::Disabled
            },
        ) {
            self.upgrade_tower(idx);
        }
    }

    /// Read-only tower readout, also embedded in the pad context when a pad is
    /// occupied.
    pub(super) fn draw_tower_context_summary(&self, rect: Rect, idx: usize, data: &GameData) {
        let Some(tower) = self.towers.get(idx) else {
            return;
        };
        let name = data
            .tower_def_by_id(&tower.tower_id)
            .map(|d| d.name.as_str())
            .unwrap_or(&text().panels.tower_fallback_name);
        ui::draw_icon(
            ConsoleIcon::Tower,
            rect.x,
            rect.y + 4.0,
            38.0,
            tower.color(),
        );
        ui::draw_console_header(
            rect.x + 48.0,
            rect.y + 27.0,
            &fill(
                &text().panels.tower_title,
                &[
                    ("name", &name.to_uppercase()),
                    ("level", &tower.level.to_string()),
                ],
            ),
            &text().panels.tower_subtitle,
            tower.color(),
        );
        let stats = self.tower_stats.get(idx).cloned().unwrap_or_default();
        ui::draw_bounded_text(
            &fill(
                &text().panels.tower_stats,
                &[
                    ("damage", &format!("{:.1}", tower.damage)),
                    ("range", &format!("{:.0}", tower.range)),
                    ("fire", &format!("{:.1}", tower.fire_rate)),
                    ("power", &format!("{:.0}", tower.power_drain)),
                ],
            ),
            rect.x + 48.0,
            rect.y + 62.0,
            rect.w - 52.0,
            13.0,
            dark::TEXT_BRIGHT,
        );
        ui::draw_bounded_text(
            &fill(
                &text().panels.tower_wave_stats,
                &[
                    ("shots", &stats.shots_this_wave.to_string()),
                    ("hits", &stats.hits_this_wave.to_string()),
                    ("kills", &stats.kills_this_wave.to_string()),
                ],
            ),
            rect.x + 48.0,
            rect.y + 84.0,
            rect.w - 52.0,
            12.0,
            dark::TEXT_DIM,
        );
        let covered =
            self.covered_paths_for_range(tower.position, self.effective_tower_range(tower.range));
        let coverage = if covered.is_empty() {
            text().panels.coverage_none.clone()
        } else {
            fill(
                &text().panels.coverage,
                &[("paths", &self.join_path_names(&covered))],
            )
        };
        ui::draw_bounded_text(
            &coverage,
            rect.x + 48.0,
            rect.y + 104.0,
            rect.w - 52.0,
            12.0,
            dark::ACCENT,
        );
    }
}
