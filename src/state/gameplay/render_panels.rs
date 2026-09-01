//! The on-selection bottom context panel and the hover tooltip.
//!
//! The panel shows exactly one context, keyed off the current selection; each
//! `context_*` module owns one of them.

mod context_building;
mod context_factory;
mod context_slot;
mod context_tower;

use crate::data::strings::{fill, text};
use crate::data::GameData;
use crate::ui;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use super::helpers::entrance_label;
use super::ui_advice::format_enemy_counts;
use super::GameplayState;

impl GameplayState {
    pub(super) fn draw_slot_panel(&mut self, data: &GameData) {
        if self.any_dock_panel_open() {
            return;
        }

        // Only show the context panel when something is actually selected.
        // With nothing selected it used to repeat the NEXT STEP strip's advice
        // (with a second FOCUS button) — pure duplication that also hid ~130px
        // of map.
        let has_selection = self.selected_core
            || self.selected_tower.is_some()
            || self.selected_slot.is_some()
            || self.selected_building.is_some();
        if !has_selection {
            return;
        }

        let rect = self.context_panel_rect();
        ui::draw_console_panel(rect, Color::new(0.22, 0.42, 0.45, 0.86));

        // Selection is mutually exclusive; core wins if somehow both are set.
        if self.selected_core {
            self.draw_factory_context(rect, data);
        } else if let Some(idx) = self.selected_tower {
            self.draw_tower_context(rect, idx, data);
        } else if let Some(idx) = self.selected_slot {
            self.draw_slot_context(rect, idx, data);
        } else if let Some(idx) = self.selected_building {
            self.draw_building_context(rect, idx);
        }
    }

    pub(super) fn draw_hover_tooltip(&self, data: &GameData) {
        let (mx, my) = mouse_position();
        let world_mouse = self.screen_to_world(vec2(mx, my));

        // While placing, the tooltip is a build read-out and takes precedence.
        if let Some(ref tower_id) = self.placing_tower {
            if let Some(def) = data.tower_def_by_id(tower_id) {
                let preview = self.placement_preview(data, world_mouse);
                let t = &text().panels;
                let covers = preview
                    .as_ref()
                    .filter(|p| !p.covered_paths.is_empty())
                    .map(|p| self.join_path_names(&p.covered_paths))
                    .unwrap_or_else(|| t.none.clone());
                let expected = preview
                    .as_ref()
                    .map(|p| format_enemy_counts(&p.expected_targets))
                    .unwrap_or_else(|| text().wave_preview.no_preview.clone());
                let afford = if self.resources.scrap < def.cost_scrap {
                    fill(
                        &t.need_scrap_line,
                        &[(
                            "n",
                            &format!("{:.0}", def.cost_scrap - self.resources.scrap),
                        )],
                    )
                } else {
                    fill(
                        &t.cost_line,
                        &[
                            ("scrap", &format!("{:.0}", def.cost_scrap)),
                            ("power", &format!("{:.0}", def.cost_power)),
                        ],
                    )
                };
                draw_tooltip_box(
                    mx,
                    my,
                    300.0,
                    &[
                        def.name.clone(),
                        fill(&t.covers, &[("text", &covers)]),
                        fill(&t.expected, &[("text", &expected)]),
                        fill(&t.cost, &[("text", &afford)]),
                    ],
                );
                return;
            }
        }

        let Some(line) = self.hover_tooltip_text(world_mouse) else {
            return;
        };
        draw_tooltip_box(mx, my, self.constants.ui.tooltip_w, &[line]);
    }

    /// One-liner for whatever the cursor is over: the core's next upgrade, or a
    /// warning that a pad/machine would open a new enemy entrance.
    fn hover_tooltip_text(&self, world_mouse: Vec2) -> Option<String> {
        let core_dist = (world_mouse - self.map_state.factory_core).length();
        if core_dist <= 26.0 {
            let next_upg = self
                .available_upgrades()
                .into_iter()
                .find(|u| !self.factory.has_upgrade(&u.id));
            let t = &text().panels;
            return Some(match next_upg {
                Some(upg) => fill(
                    &t.upgrade_row,
                    &[
                        ("name", &upg.name),
                        ("n", &(upg.cost_scrap as i32).to_string()),
                    ],
                ),
                None => t.factory_console_fallback.clone(),
            });
        }

        if let Some((slot_idx, _)) = self.map_state.nearest_slot(world_mouse) {
            let entrance = self.map_state.slots[slot_idx].opens_entrance.as_ref()?;
            return Some(fill(
                &text().panels.unlocks_entrance,
                &[("path", entrance_label(entrance))],
            ));
        }

        let (b_idx, _) = self.nearest_unlocked_building(world_mouse)?;
        let entrance = self.map_state.buildings[b_idx].opens_entrance.as_ref()?;
        Some(fill(
            &text().panels.unlocks_entrance,
            &[("path", entrance_label(entrance))],
        ))
    }
}

fn draw_tooltip_box(mx: f32, my: f32, w: f32, lines: &[String]) {
    let h = (lines.len() as f32 * 15.0 + 12.0).max(30.0);
    let x = (mx + 12.0).clamp(10.0, screen_width() - w - 10.0);
    let y = (my + 12.0).clamp(10.0, screen_height() - h - 10.0);
    draw_rectangle(x, y, w, h, Color::new(0.08, 0.08, 0.1, 0.92));
    draw_rectangle_lines(x, y, w, h, 1.0, dark::TEXT_DIM);
    for (idx, line) in lines.iter().enumerate() {
        let color = if idx == 0 {
            dark::TEXT_BRIGHT
        } else {
            dark::TEXT_DIM
        };
        ui::draw_bounded_text(
            line,
            x + 8.0,
            y + 16.0 + idx as f32 * 15.0,
            w - 16.0,
            12.0,
            color,
        );
    }
}
