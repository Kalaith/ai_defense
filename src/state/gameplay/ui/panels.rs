//! On-demand side consoles: the tower build list and the SYSTEMS console.

use crate::data::strings::{fill, text};
use crate::data::GameData;
use crate::engine::map::BuildingState;
use crate::state::gameplay::GameplayState;
use crate::ui;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

impl GameplayState {
    pub fn draw_build_panel(&mut self, data: &GameData) {
        // Show every tower — locked ones greyed with their unlock requirement —
        // so the path to new towers is visible rather than hidden.
        let infos: Vec<ui::TowerButtonInfo> = data
            .tower_defs
            .iter()
            .map(|def| {
                let unlocked = self.is_tower_unlocked(&def.id);
                ui::TowerButtonInfo {
                    unlocked,
                    requirement: if unlocked {
                        String::new()
                    } else {
                        self.tower_unlock_requirement(&def.id)
                    },
                }
            })
            .collect();
        let rect = self.build_panel_rect();
        let build_clicked = ui::draw_build_panel(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            &data.tower_defs,
            &infos,
            self.resources.scrap,
            self.resources.power,
        );
        if let Some(tower_id) = build_clicked {
            self.placing_tower = Some(tower_id);
            // Arm placement, then get the console out of the way so the player
            // can reach pads that sit beneath the left side of the map.
            self.show_build_panel = false;
        }
    }

    /// Readable, comma-joined list of the still-unmet unlock requirements for a
    /// tower (e.g. "Research Core"), for display on locked build buttons.
    fn tower_unlock_requirement(&self, tower_id: &str) -> String {
        let Some(rule) = self.unlocks.towers.get(tower_id) else {
            return String::new();
        };
        let names: Vec<String> = rule
            .requires
            .iter()
            .filter(|req| !self.requirement_met(req))
            .map(|req| self.requirement_label(req))
            .collect();
        names.join(", ")
    }

    /// Map an unlock token (building id, sector id, or upgrade id) to a
    /// player-facing name — preferring the factory section it powers.
    fn requirement_label(&self, req: &str) -> String {
        for section in self.map_state.section_render_info() {
            if section.core_building == req {
                return section.label;
            }
        }
        self.building_display_name_by_id(req)
    }

    pub fn draw_sector_panel(&mut self) {
        let panel = self.sector_panel_rect();
        let sector_x = panel.x;
        let panel_y = panel.y;
        let panel_w = panel.w;
        let panel_h = panel.h;

        ui::draw_console_panel(
            Rect::new(sector_x, panel_y, panel_w, panel_h),
            Color::new(0.26, 0.42, 0.48, 0.82),
        );
        let t = &text().panels;
        let st = &text().status;
        ui::draw_console_header(
            sector_x + 12.0,
            panel_y + 26.0,
            &t.systems,
            &t.systems_subtitle,
            dark::ACCENT,
        );

        let mut row_y = panel_y + 56.0;
        let row_h = 94.0;
        let padding = 9.0;
        let mut clicked_building = None;
        let mut clicked_action: Option<(usize, &'static str)> = None;
        let reveal_x = self.map_state.max_visible_x() + 120.0;

        for (idx, building) in self.map_state.buildings.iter().enumerate() {
            if !self.map_state.is_building_visible(building) || building.position.x > reveal_x {
                continue;
            }
            if row_y + row_h > panel_y + panel_h - 5.0 {
                break;
            }

            let unlocked = self.is_building_unlocked(building);
            let selected = self.selected_building == Some(idx);
            let (status, status_color) = if !unlocked {
                (st.locked.as_str(), dark::TEXT_DIM)
            } else {
                match building.state {
                    BuildingState::Broken => (st.damaged.as_str(), dark::NEGATIVE),
                    BuildingState::Repaired => (st.repaired.as_str(), dark::WARNING),
                    BuildingState::Powered => (st.online.as_str(), dark::POSITIVE),
                    BuildingState::Disabled => (st.offline.as_str(), dark::TEXT_DIM),
                }
            };
            let row_rect = Rect::new(sector_x + 7.0, row_y, panel_w - 14.0, row_h);
            let border = if selected {
                dark::TEXT_BRIGHT
            } else if unlocked {
                status_color
            } else {
                dark::TEXT_DIM
            };
            ui::draw_console_panel(row_rect, Color::new(border.r, border.g, border.b, 0.72));

            let icon = ui::icon_for_boon(&building.boon, building.threat_per_sec);
            ui::draw_icon(icon, row_rect.x + 12.0, row_rect.y + 18.0, 30.0, border);

            let name = self.building_display_name(building);
            ui::draw_bounded_text(
                &name,
                row_rect.x + 50.0,
                row_y + 20.0,
                row_rect.w - 62.0,
                14.0,
                dark::TEXT,
            );
            ui::draw_status_pill(row_rect.x + 50.0, row_y + 27.0, status, status_color);

            let detail = self.boon_text(&building.boon);
            let cost_line = match building.state {
                BuildingState::Broken => fill(
                    &t.repair_cost,
                    &[("n", &format!("{:.0}", building.repair_cost))],
                ),
                BuildingState::Repaired => fill(
                    &t.power_cost,
                    &[("n", &format!("{:.0}", building.power_cost))],
                ),
                BuildingState::Powered => t.online.clone(),
                BuildingState::Disabled => t.locked.clone(),
            };
            ui::draw_bounded_text(
                &cost_line,
                row_rect.x + 12.0,
                row_y + 58.0,
                row_rect.w - 126.0,
                11.0,
                dark::TEXT_DIM,
            );
            ui::draw_bounded_text(
                &fill(&t.benefit, &[("text", &detail)]),
                row_rect.x + 12.0,
                row_y + 74.0,
                row_rect.w - 126.0,
                11.0,
                if matches!(building.state, BuildingState::Powered) {
                    dark::POSITIVE
                } else {
                    dark::TEXT_DIM
                },
            );
            ui::draw_bounded_text(
                &fill(
                    &t.risk_noise,
                    &[("n", &format!("{:.2}", building.threat_per_sec))],
                ),
                row_rect.x + 12.0,
                row_y + 89.0,
                row_rect.w - 126.0,
                10.0,
                dark::WARNING,
            );

            let (action_label, action_kind, action_state) =
                self.sector_row_action(unlocked, building);
            if ui::draw_console_button(
                row_rect.x + row_rect.w - 108.0,
                row_y + 56.0,
                96.0,
                32.0,
                &action_label,
                action_state,
            ) {
                clicked_action = Some((idx, action_kind));
            }

            let (mx, my) = mouse_position();
            if mx >= row_rect.x
                && mx <= row_rect.x + row_rect.w
                && my >= row_rect.y
                && my <= row_rect.y + row_rect.h
            {
                draw_rectangle(
                    row_rect.x,
                    row_rect.y,
                    row_rect.w,
                    row_rect.h,
                    Color::new(0.35, 0.48, 0.54, 0.1),
                );
                if is_mouse_button_pressed(MouseButton::Left) {
                    clicked_building = Some(idx);
                }
            }

            row_y += row_h + padding;
        }

        if let Some((idx, action)) = clicked_action {
            self.select_building_only(idx);
            match action {
                "repair" => self.repair_building(idx),
                "power" => self.power_building(idx),
                _ => {}
            }
        } else if let Some(idx) = clicked_building {
            self.select_building_only(idx);
        }
    }

    /// Label, intent, and button state for a SYSTEMS row's action button.
    fn sector_row_action(
        &self,
        unlocked: bool,
        building: &crate::engine::map::MapBuilding,
    ) -> (String, &'static str, ui::ConsoleButtonState) {
        let st = &text().status;
        let shortfall = |cost: f32| {
            fill(
                &st.need_scrap,
                &[(
                    "n",
                    &format!("{:.0}", (cost - self.resources.scrap).max(0.0)),
                )],
            )
        };
        if !unlocked {
            return (
                st.locked.clone(),
                "locked",
                ui::ConsoleButtonState::Disabled,
            );
        }
        match building.state {
            BuildingState::Broken => {
                if self.resources.scrap >= building.repair_cost {
                    (
                        st.repair.clone(),
                        "repair",
                        ui::ConsoleButtonState::Affordable,
                    )
                } else {
                    (
                        shortfall(building.repair_cost),
                        "repair",
                        ui::ConsoleButtonState::Disabled,
                    )
                }
            }
            BuildingState::Repaired => {
                if self.resources.scrap >= building.power_cost {
                    (
                        st.power.clone(),
                        "power",
                        ui::ConsoleButtonState::Recommended,
                    )
                } else {
                    (
                        shortfall(building.power_cost),
                        "power",
                        ui::ConsoleButtonState::Disabled,
                    )
                }
            }
            BuildingState::Powered => (
                st.online.clone(),
                "online",
                ui::ConsoleButtonState::Disabled,
            ),
            BuildingState::Disabled => (
                st.locked.clone(),
                "locked",
                ui::ConsoleButtonState::Disabled,
            ),
        }
    }
}
