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
        let has_research = self.factory.is_sector_active("research_lab");
        if tower.level >= max_level || tower.specialization_id.is_some() {
            let tower_id = tower.tower_id.clone();
            let specialization_id = tower.specialization_id.clone();
            self.draw_specialization_choices(
                rect,
                idx,
                &tower_id,
                specialization_id.as_deref(),
                has_research,
                data,
            );
            return;
        }
        let upgrade_cost = tower.base_scrap_cost * tower.level as f32;
        let can_upgrade =
            has_research && tower.level < max_level && self.resources.scrap >= upgrade_cost;

        let label = if !has_research {
            text().panels.requires_research.clone()
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

    fn draw_specialization_choices(
        &mut self,
        rect: Rect,
        tower_idx: usize,
        tower_id: &str,
        selected_id: Option<&str>,
        has_research: bool,
        data: &GameData,
    ) {
        let Some(def) = data.tower_def_by_id(tower_id) else {
            return;
        };
        if let Some(selected_id) = selected_id {
            let name = def
                .specializations
                .iter()
                .find(|branch| branch.id == selected_id)
                .map(|branch| branch.name.as_str())
                .unwrap_or(selected_id);
            ui::draw_console_button(
                rect.x + rect.w - 260.0,
                rect.y + 58.0,
                244.0,
                44.0,
                &fill(&text().panels.specialized, &[("name", name)]),
                ConsoleButtonState::Disabled,
            );
            return;
        }

        ui::draw_bounded_text(
            &text().panels.choose_specialization,
            rect.x + rect.w - 344.0,
            rect.y + 28.0,
            328.0,
            12.0,
            dark::WARNING,
        );
        let mut clicked = None;
        for (column, branch) in def.specializations.iter().take(2).enumerate() {
            let x = rect.x + rect.w - 344.0 + column as f32 * 168.0;
            let can = has_research && self.resources.scrap >= branch.cost_scrap;
            let label = if has_research {
                fill(
                    &text().panels.specialize,
                    &[
                        ("name", &branch.name.to_uppercase()),
                        ("n", &format!("{:.0}", branch.cost_scrap)),
                    ],
                )
            } else {
                text().panels.requires_research.clone()
            };
            if ui::draw_console_button(
                x,
                rect.y + 40.0,
                160.0,
                40.0,
                &label,
                if can {
                    ConsoleButtonState::Recommended
                } else {
                    ConsoleButtonState::Disabled
                },
            ) {
                clicked = Some(branch.id.clone());
            }
            ui::draw_bounded_text(
                &branch.description,
                x + 4.0,
                rect.y + 101.0,
                152.0,
                10.0,
                dark::TEXT_DIM,
            );
        }
        if let Some(id) = clicked {
            self.specialize_tower(tower_idx, &id, data);
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
