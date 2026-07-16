//! Bottom-panel context for a selected tower pad: clear it, power it, or build.

use crate::data::strings::{fill, text};
use crate::data::{GameData, TowerDef};
use crate::engine::map::SlotState;
use crate::ui::{self, ConsoleButtonState, ConsoleIcon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use super::super::GameplayState;

/// Every context panel puts its action button in the same place.
const ACTION_W: f32 = 166.0;
const ACTION_H: f32 = 44.0;

impl GameplayState {
    pub(super) fn draw_slot_context(&mut self, rect: Rect, idx: usize, data: &GameData) {
        let Some(slot) = self.map_state.slots.get(idx) else {
            return;
        };
        let slot_id = slot.id.clone();
        let slot_state = slot.state;
        let clear_cost = slot.clear_cost;
        let power_cost = slot.power_cost;
        let opens = slot.opens_entrance.clone();
        let tower_index = slot.tower_index;
        let position = slot.position;

        ui::draw_icon(
            ConsoleIcon::Tower,
            rect.x + 14.0,
            rect.y + 18.0,
            34.0,
            dark::ACCENT,
        );
        ui::draw_console_header(
            rect.x + 56.0,
            rect.y + 24.0,
            &fill(&text().panels.pad, &[("id", &slot_id.to_uppercase())]),
            &text().panels.pad_subtitle,
            dark::ACCENT,
        );

        let detail_x = rect.x + 56.0;
        let detail_y = rect.y + 58.0;
        let coverage = self.coverage_for_best_available_tower(position, data);

        match slot_state {
            SlotState::Debris => {
                let warning = opens
                    .as_deref()
                    .map(|id| {
                        fill(
                            &text().panels.clearing_opens,
                            &[("path", &self.path_display_name(id))],
                        )
                    })
                    .unwrap_or_else(|| text().panels.clears_position.clone());
                ui::draw_bounded_text(
                    &text().panels.status_debris,
                    detail_x,
                    detail_y,
                    210.0,
                    13.0,
                    dark::WARNING,
                );
                ui::draw_bounded_text(
                    &warning,
                    detail_x,
                    detail_y + 21.0,
                    300.0,
                    12.0,
                    dark::TEXT_DIM,
                );
                if self.draw_cost_action(rect, clear_cost, &text().panels.clear) {
                    self.clear_slot(idx);
                }
            }
            SlotState::Cleared => {
                ui::draw_bounded_text(
                    &text().panels.status_cleared,
                    detail_x,
                    detail_y,
                    210.0,
                    13.0,
                    dark::ACCENT,
                );
                ui::draw_bounded_text(
                    &coverage,
                    detail_x,
                    detail_y + 21.0,
                    300.0,
                    12.0,
                    dark::TEXT_DIM,
                );
                if self.draw_cost_action(rect, power_cost, &text().panels.power_pad) {
                    self.power_slot(idx);
                }
            }
            SlotState::Powered => match tower_index {
                Some(tower_idx) => {
                    ui::draw_bounded_text(
                        &text().panels.status_occupied,
                        detail_x,
                        detail_y,
                        210.0,
                        13.0,
                        dark::POSITIVE,
                    );
                    self.draw_tower_context_summary(
                        Rect::new(
                            rect.x + rect.w * 0.48,
                            rect.y + 18.0,
                            rect.w * 0.3,
                            rect.h - 36.0,
                        ),
                        tower_idx,
                        data,
                    );
                    if ui::draw_console_button(
                        rect.x + rect.w - 182.0,
                        rect.y + 58.0,
                        ACTION_W,
                        ACTION_H,
                        &text().panels.select_tower,
                        ConsoleButtonState::Affordable,
                    ) {
                        self.selected_tower = Some(tower_idx);
                    }
                }
                None => {
                    ui::draw_bounded_text(
                        &text().panels.status_powered,
                        detail_x,
                        detail_y,
                        210.0,
                        13.0,
                        dark::POSITIVE,
                    );
                    ui::draw_bounded_text(
                        &coverage,
                        detail_x,
                        detail_y + 21.0,
                        300.0,
                        12.0,
                        dark::TEXT_DIM,
                    );
                    self.draw_empty_pad_build(rect, idx, position, data);
                }
            },
        }
    }

    /// An empty powered pad offers the tower already being placed, else the one
    /// that would cover the most routes from here. First click arms it, second
    /// click builds.
    fn draw_empty_pad_build(&mut self, rect: Rect, idx: usize, position: Vec2, data: &GameData) {
        let selected = self
            .placing_tower
            .as_ref()
            .and_then(|id| data.tower_def_by_id(id));
        let Some(def) = selected.or_else(|| self.recommended_tower_for_position(position, data))
        else {
            return;
        };

        ui::draw_bounded_text(
            &fill(&text().panels.recommended, &[("name", &def.name)]),
            rect.x + 56.0,
            rect.y + 100.0,
            300.0,
            12.0,
            dark::TEXT_BRIGHT,
        );

        let armed = self.placing_tower.as_deref() == Some(def.id.as_str());
        let can_build = self.resources.scrap >= def.cost_scrap;
        let label = if !armed {
            text().panels.select_tower.clone()
        } else if can_build {
            fill(
                &text().panels.build,
                &[("n", &format!("{:.0}", def.cost_scrap))],
            )
        } else {
            fill(
                &text().status.need_scrap,
                &[(
                    "n",
                    &format!("{:.0}", def.cost_scrap - self.resources.scrap),
                )],
            )
        };
        let state = if can_build {
            ConsoleButtonState::Recommended
        } else {
            ConsoleButtonState::Disabled
        };
        let def_id = def.id.clone();
        if ui::draw_console_button(
            rect.x + rect.w - 182.0,
            rect.y + 58.0,
            ACTION_W,
            ACTION_H,
            &label,
            state,
        ) {
            if armed {
                self.try_place_tower_on_slot(idx, &def_id, data);
            } else {
                self.placing_tower = Some(def_id);
            }
        }
    }

    /// Action button for a fixed scrap cost: `<VERB> <cost>` when affordable,
    /// the shortfall when not. Returns true when clicked and affordable.
    fn draw_cost_action(&self, rect: Rect, cost: f32, verb: &str) -> bool {
        let can = self.resources.scrap >= cost;
        let label = if can {
            fill(
                &text().panels.cost_action,
                &[("verb", verb), ("n", &format!("{cost:.0}"))],
            )
        } else {
            fill(
                &text().status.need_scrap,
                &[("n", &format!("{:.0}", cost - self.resources.scrap))],
            )
        };
        let state = if can {
            ConsoleButtonState::Recommended
        } else {
            ConsoleButtonState::Disabled
        };
        ui::draw_console_button(
            rect.x + rect.w - 182.0,
            rect.y + 58.0,
            ACTION_W,
            ACTION_H,
            &label,
            state,
        )
    }

    /// The unlocked tower covering the most active routes from `pos`; ties break
    /// toward the cheaper tower.
    pub(super) fn recommended_tower_for_position<'a>(
        &self,
        pos: Vec2,
        data: &'a GameData,
    ) -> Option<&'a TowerDef> {
        data.tower_defs
            .iter()
            .filter(|def| self.is_tower_unlocked(&def.id))
            .max_by(|a, b| {
                let ac = self
                    .covered_paths_for_range(pos, self.effective_tower_range(a.base_range))
                    .len();
                let bc = self
                    .covered_paths_for_range(pos, self.effective_tower_range(b.base_range))
                    .len();
                ac.cmp(&bc)
                    .then_with(|| b.cost_scrap.total_cmp(&a.cost_scrap))
            })
    }

    fn coverage_for_best_available_tower(&self, pos: Vec2, data: &GameData) -> String {
        let Some(def) = self.recommended_tower_for_position(pos, data) else {
            return text().panels.no_tower_unlocked.clone();
        };
        let covered = self.covered_paths_for_range(pos, self.effective_tower_range(def.base_range));
        if covered.is_empty() {
            fill(&text().panels.covers_no_route, &[("name", &def.name)])
        } else {
            fill(
                &text().panels.covers_routes,
                &[
                    ("name", &def.name),
                    ("paths", &self.join_path_names(&covered)),
                ],
            )
        }
    }
}
