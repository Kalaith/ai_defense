use crate::data::{GameData, TowerDef};
use crate::engine::map::{BuildingState, SlotState};
use crate::ui::{self, ConsoleButtonState, ConsoleIcon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use super::helpers::entrance_label;
use super::ui_advice::{format_enemy_counts, AdviceTarget, WavePreviewCard};
use super::GameplayState;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl GameplayState {
    pub(super) fn draw_slot_panel(&mut self, data: &GameData) {
        let play_x = self.constants.ui.build_panel_w + 8.0;
        let play_w =
            (screen_width() - self.constants.ui.build_panel_w - self.constants.ui.sector_panel_w)
                .max(360.0)
                - 16.0;
        let panel_h = self.constants.ui.bottom_context_h;
        let panel_x = play_x;
        let panel_y = screen_height() - panel_h - 8.0;
        let panel_w = play_w;

        let rect = Rect::new(panel_x, panel_y, panel_w, panel_h);
        ui::draw_console_panel(rect, Color::new(0.22, 0.42, 0.45, 0.86));

        if self.selected_core {
            self.draw_factory_context(rect, data);
        } else if let Some(idx) = self.selected_tower {
            self.draw_tower_context(rect, idx, data);
        } else if let Some(idx) = self.selected_slot {
            self.draw_slot_context(rect, idx, data);
        } else if let Some(idx) = self.selected_building {
            self.draw_building_context(rect, idx);
        } else {
            self.draw_default_context(rect, data);
        }
    }

    fn draw_default_context(&mut self, rect: Rect, data: &GameData) {
        let advice = self.build_ui_advice(data);
        let action_w = rect.w - 210.0;
        ui::draw_console_header(
            rect.x + 16.0,
            rect.y + 27.0,
            "COMMAND CONTEXT",
            &advice.objective,
            dark::ACCENT,
        );
        draw_bounded_text(
            &advice.suggested_action.label,
            rect.x + 16.0,
            rect.y + 61.0,
            action_w,
            20.0,
            dark::TEXT_BRIGHT,
        );
        draw_bounded_text(
            &advice.suggested_action.detail,
            rect.x + 16.0,
            rect.y + 83.0,
            action_w,
            13.0,
            dark::TEXT_DIM,
        );

        let chip_y = rect.y + rect.h - 34.0;
        let chip_w = ((action_w - 18.0) / 3.0).max(112.0);
        draw_info_chip(
            "COST",
            &advice.suggested_action.cost.replace("Cost: ", ""),
            rect.x + 16.0,
            chip_y,
            chip_w,
            dark::TEXT_BRIGHT,
        );
        draw_info_chip(
            "RISK",
            &advice.risk.replace("Risk: ", ""),
            rect.x + 16.0 + chip_w + 9.0,
            chip_y,
            chip_w,
            dark::WARNING,
        );
        draw_info_chip(
            "THREAT READ",
            &format_enemy_counts(&advice.wave_preview.counts),
            rect.x + 16.0 + (chip_w + 9.0) * 2.0,
            chip_y,
            chip_w,
            dark::ACCENT,
        );

        let btn_state = if matches!(advice.suggested_action.target, AdviceTarget::None) {
            ConsoleButtonState::Disabled
        } else {
            ConsoleButtonState::Recommended
        };
        draw_bounded_text(
            &format!(
                "Scrap {:.0} | Battery {:.0}",
                self.resources.scrap, self.resources.power
            ),
            rect.x + rect.w - 188.0,
            rect.y + 33.0,
            174.0,
            12.0,
            dark::TEXT_DIM,
        );
        if ui::draw_console_button(
            rect.x + rect.w - 188.0,
            rect.y + 45.0,
            172.0,
            50.0,
            "FOCUS ACTION",
            btn_state,
        ) {
            let target = advice.suggested_action.target.clone();
            self.apply_advice_focus(&target, data);
        }
    }

    fn draw_slot_context(&mut self, rect: Rect, idx: usize, data: &GameData) {
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
            &format!("PAD {}", slot_id.to_uppercase()),
            "tower pad command surface",
            dark::ACCENT,
        );

        let detail_x = rect.x + 56.0;
        let detail_y = rect.y + 58.0;
        let coverage = self.coverage_for_best_available_tower(position, data);
        match slot_state {
            SlotState::Debris => {
                let warning = opens
                    .as_deref()
                    .map(|id| format!("Clearing can open {}", self.path_display_name(id)))
                    .unwrap_or_else(|| "Clears a build position".to_string());
                draw_bounded_text(
                    "Status: Debris",
                    detail_x,
                    detail_y,
                    210.0,
                    13.0,
                    dark::WARNING,
                );
                draw_bounded_text(
                    &warning,
                    detail_x,
                    detail_y + 21.0,
                    300.0,
                    12.0,
                    dark::TEXT_DIM,
                );
                let state = if self.resources.scrap >= clear_cost {
                    ConsoleButtonState::Recommended
                } else {
                    ConsoleButtonState::Disabled
                };
                let label = if self.resources.scrap >= clear_cost {
                    format!("CLEAR {:.0}", clear_cost)
                } else {
                    format!("NEED {:.0} SCRAP", clear_cost - self.resources.scrap)
                };
                if ui::draw_console_button(
                    rect.x + rect.w - 182.0,
                    rect.y + 58.0,
                    166.0,
                    44.0,
                    &label,
                    state,
                ) {
                    self.clear_slot(idx);
                }
            }
            SlotState::Cleared => {
                draw_bounded_text(
                    "Status: Cleared",
                    detail_x,
                    detail_y,
                    210.0,
                    13.0,
                    dark::ACCENT,
                );
                draw_bounded_text(
                    &coverage,
                    detail_x,
                    detail_y + 21.0,
                    300.0,
                    12.0,
                    dark::TEXT_DIM,
                );
                let state = if self.resources.scrap >= power_cost {
                    ConsoleButtonState::Recommended
                } else {
                    ConsoleButtonState::Disabled
                };
                let label = if self.resources.scrap >= power_cost {
                    format!("POWER PAD {:.0}", power_cost)
                } else {
                    format!("NEED {:.0} SCRAP", power_cost - self.resources.scrap)
                };
                if ui::draw_console_button(
                    rect.x + rect.w - 182.0,
                    rect.y + 58.0,
                    166.0,
                    44.0,
                    &label,
                    state,
                ) {
                    self.power_slot(idx);
                }
            }
            SlotState::Powered => {
                if let Some(tower_idx) = tower_index {
                    draw_bounded_text(
                        "Status: Occupied",
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
                        166.0,
                        44.0,
                        "SELECT TOWER",
                        ConsoleButtonState::Affordable,
                    ) {
                        self.selected_tower = Some(tower_idx);
                    }
                } else {
                    draw_bounded_text(
                        "Status: Powered",
                        detail_x,
                        detail_y,
                        210.0,
                        13.0,
                        dark::POSITIVE,
                    );
                    draw_bounded_text(
                        &coverage,
                        detail_x,
                        detail_y + 21.0,
                        300.0,
                        12.0,
                        dark::TEXT_DIM,
                    );
                    let selected = self
                        .placing_tower
                        .as_ref()
                        .and_then(|id| data.tower_def_by_id(id));
                    let recommended =
                        selected.or_else(|| self.recommended_tower_for_position(position, data));
                    if let Some(def) = recommended {
                        draw_bounded_text(
                            &format!("Recommended: {}", def.name),
                            detail_x,
                            detail_y + 42.0,
                            300.0,
                            12.0,
                            dark::TEXT_BRIGHT,
                        );
                        let can_build = self.resources.scrap >= def.cost_scrap;
                        let label = if self.placing_tower.as_deref() == Some(def.id.as_str()) {
                            if can_build {
                                format!("BUILD {:.0}", def.cost_scrap)
                            } else {
                                format!("NEED {:.0} SCRAP", def.cost_scrap - self.resources.scrap)
                            }
                        } else {
                            "SELECT TOWER".to_string()
                        };
                        let state = if can_build {
                            ConsoleButtonState::Recommended
                        } else {
                            ConsoleButtonState::Disabled
                        };
                        if ui::draw_console_button(
                            rect.x + rect.w - 182.0,
                            rect.y + 58.0,
                            166.0,
                            44.0,
                            &label,
                            state,
                        ) {
                            if self.placing_tower.as_deref() == Some(def.id.as_str()) {
                                self.try_place_tower_on_slot(idx, &def.id, data);
                            } else {
                                self.placing_tower = Some(def.id.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_building_context(&mut self, rect: Rect, idx: usize) {
        let Some(building) = self.map_state.buildings.get(idx) else {
            return;
        };
        let name = self.building_display_name(building);
        let state = building.state;
        let repair_cost = building.repair_cost;
        let power_cost = building.power_cost;
        let threat = building.threat_per_sec;
        let benefit = self.boon_text(&building.boon);
        let icon = building_icon(&building.boon, threat);

        ui::draw_icon(icon, rect.x + 14.0, rect.y + 18.0, 38.0, dark::ACCENT);
        ui::draw_console_header(
            rect.x + 62.0,
            rect.y + 27.0,
            &name.to_uppercase(),
            "factory system",
            dark::ACCENT,
        );
        let status = match state {
            BuildingState::Broken => ("DAMAGED", dark::NEGATIVE),
            BuildingState::Repaired => ("REPAIRED", dark::WARNING),
            BuildingState::Powered => ("ONLINE", dark::POSITIVE),
            BuildingState::Disabled => ("LOCKED", dark::TEXT_DIM),
        };
        ui::draw_status_pill(rect.x + 62.0, rect.y + 43.0, status.0, status.1);
        draw_bounded_text(
            &format!("Benefit: {}", benefit),
            rect.x + 62.0,
            rect.y + 76.0,
            rect.w * 0.42,
            14.0,
            dark::TEXT_BRIGHT,
        );
        draw_bounded_text(
            &format!("Risk: +{:.2} noise/s", threat),
            rect.x + 62.0,
            rect.y + 98.0,
            rect.w * 0.42,
            12.0,
            dark::WARNING,
        );

        match state {
            BuildingState::Broken => {
                let can = self.resources.scrap >= repair_cost;
                let label = if can {
                    format!("REPAIR {:.0}", repair_cost)
                } else {
                    format!("NEED {:.0} SCRAP", repair_cost - self.resources.scrap)
                };
                if ui::draw_console_button(
                    rect.x + rect.w - 182.0,
                    rect.y + 58.0,
                    166.0,
                    44.0,
                    &label,
                    if can {
                        ConsoleButtonState::Recommended
                    } else {
                        ConsoleButtonState::Disabled
                    },
                ) {
                    self.repair_building(idx);
                }
            }
            BuildingState::Repaired => {
                let can = self.resources.scrap >= power_cost;
                let label = if can {
                    format!("POWER {:.0}", power_cost)
                } else {
                    format!("NEED {:.0} SCRAP", power_cost - self.resources.scrap)
                };
                if ui::draw_console_button(
                    rect.x + rect.w - 182.0,
                    rect.y + 58.0,
                    166.0,
                    44.0,
                    &label,
                    if can {
                        ConsoleButtonState::Recommended
                    } else {
                        ConsoleButtonState::Disabled
                    },
                ) {
                    self.power_building(idx);
                }
            }
            BuildingState::Powered | BuildingState::Disabled => {
                ui::draw_console_button(
                    rect.x + rect.w - 182.0,
                    rect.y + 58.0,
                    166.0,
                    44.0,
                    status.0,
                    ConsoleButtonState::Disabled,
                );
            }
        }
    }

    fn draw_tower_context(&mut self, rect: Rect, idx: usize, data: &GameData) {
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
        let can_upgrade = self.factory.is_sector_active("research_lab")
            && tower.level < max_level
            && self.resources.scrap >= upgrade_cost;
        let label = if !self.factory.is_sector_active("research_lab") {
            "REQUIRES RESEARCH".to_string()
        } else if tower.level >= max_level {
            "MAX LEVEL".to_string()
        } else if can_upgrade {
            format!("UPGRADE {:.0}", upgrade_cost)
        } else {
            format!("NEED {:.0} SCRAP", upgrade_cost - self.resources.scrap)
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

    fn draw_tower_context_summary(&self, rect: Rect, idx: usize, data: &GameData) {
        let Some(tower) = self.towers.get(idx) else {
            return;
        };
        let name = data
            .tower_def_by_id(&tower.tower_id)
            .map(|d| d.name.as_str())
            .unwrap_or("Tower");
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
            &format!("{} LVL {}", name.to_uppercase(), tower.level),
            "tower command",
            tower.color(),
        );
        let stats = self.tower_stats.get(idx).cloned().unwrap_or_default();
        draw_bounded_text(
            &format!(
                "Damage {:.1} | Range {:.0} | Fire {:.1}/s | Power {:.0}",
                tower.damage, tower.range, tower.fire_rate, tower.power_drain
            ),
            rect.x + 48.0,
            rect.y + 62.0,
            rect.w - 52.0,
            13.0,
            dark::TEXT_BRIGHT,
        );
        draw_bounded_text(
            &format!(
                "This wave: {} shots, {} hits, {} kills",
                stats.shots_this_wave, stats.hits_this_wave, stats.kills_this_wave
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
            "Coverage: none".to_string()
        } else {
            format!(
                "Coverage: {}",
                covered
                    .iter()
                    .map(|id| self.path_display_name(id))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        draw_bounded_text(
            &coverage,
            rect.x + 48.0,
            rect.y + 104.0,
            rect.w - 52.0,
            12.0,
            dark::ACCENT,
        );
    }

    fn draw_factory_context(&mut self, rect: Rect, _data: &GameData) {
        ui::draw_icon(
            ConsoleIcon::Power,
            rect.x + 14.0,
            rect.y + 18.0,
            34.0,
            dark::ACCENT,
        );
        ui::draw_console_header(
            rect.x + 56.0,
            rect.y + 24.0,
            "FACTORY CONSOLE",
            "research and machine unlocks",
            dark::ACCENT,
        );

        let upgrades = self.available_upgrades();
        let mut clicked_upgrade: Option<String> = None;
        let mut unlock_upgrade: Option<String> = None;
        let list_x = rect.x + rect.w * 0.42;
        let list_w = rect.w * 0.34;
        let mut y = rect.y + 16.0;
        for upg in upgrades.iter().take(3) {
            let row_h = 30.0;
            let selected = self.selected_upgrade.as_deref() == Some(&upg.id);
            let purchased = self.factory.has_upgrade(&upg.id);
            let accent = if selected {
                dark::TEXT_BRIGHT
            } else if purchased {
                dark::POSITIVE
            } else {
                dark::ACCENT
            };
            ui::draw_console_panel(
                Rect::new(list_x, y, list_w, row_h),
                Color::new(accent.r, accent.g, accent.b, 0.55),
            );
            draw_bounded_text(
                &upg.name,
                list_x + 8.0,
                y + 13.0,
                list_w - 16.0,
                11.0,
                accent,
            );
            draw_bounded_text(
                &upg.description,
                list_x + 8.0,
                y + 26.0,
                list_w - 16.0,
                9.0,
                dark::TEXT_DIM,
            );
            let (mx, my) = mouse_position();
            if mx >= list_x
                && mx <= list_x + list_w
                && my >= y
                && my <= y + row_h
                && is_mouse_button_pressed(MouseButton::Left)
            {
                clicked_upgrade = Some(upg.id.clone());
            }
            y += row_h + 7.0;
        }

        if self.selected_upgrade.is_none() {
            if let Some(upg) = upgrades.iter().find(|u| !self.factory.has_upgrade(&u.id)) {
                self.selected_upgrade = Some(upg.id.clone());
            }
        }

        if let Some(selected_id) = self.selected_upgrade.clone() {
            let selected = self.upgrade_defs.iter().find(|u| u.id == selected_id);
            if let Some(selected) = selected {
                draw_bounded_text(
                    &selected.name,
                    rect.x + 56.0,
                    rect.y + 62.0,
                    rect.w * 0.34,
                    14.0,
                    dark::TEXT_BRIGHT,
                );
                draw_bounded_text(
                    &selected.description,
                    rect.x + 56.0,
                    rect.y + 82.0,
                    rect.w * 0.34,
                    11.0,
                    dark::TEXT_DIM,
                );
                let purchased = self.factory.has_upgrade(&selected.id);
                let can =
                    self.factory
                        .can_purchase(selected, self.resources.scrap, self.resources.power);
                let label = if purchased {
                    "UNLOCKED".to_string()
                } else if can {
                    format!("UNLOCK {:.0}", selected.cost_scrap)
                } else {
                    format!(
                        "NEED {:.0} SCRAP",
                        (selected.cost_scrap - self.resources.scrap).max(0.0)
                    )
                };
                if ui::draw_console_button(
                    rect.x + rect.w - 182.0,
                    rect.y + 58.0,
                    166.0,
                    44.0,
                    &label,
                    if can {
                        ConsoleButtonState::Recommended
                    } else {
                        ConsoleButtonState::Disabled
                    },
                ) {
                    unlock_upgrade = Some(selected.id.clone());
                }
            }
        }

        if let Some(id) = clicked_upgrade {
            self.selected_upgrade = Some(id);
        }
        if let Some(id) = unlock_upgrade {
            self.purchase_upgrade(&id);
        }
    }

    #[allow(dead_code)]
    fn draw_wave_preview_card(&self, rect: Rect, card: &WavePreviewCard) {
        ui::draw_console_panel(rect, Color::new(0.4, 0.28, 0.16, 0.64));
        draw_ui_text(
            &card.title,
            rect.x + 10.0,
            rect.y + 16.0,
            11.0,
            dark::WARNING,
        );
        let counts = format_enemy_counts(&card.counts);
        draw_bounded_text(
            &counts,
            rect.x + 10.0,
            rect.y + 38.0,
            rect.w - 20.0,
            12.0,
            dark::TEXT_BRIGHT,
        );
        if card.live {
            draw_bounded_text(
                &format!("{} alive / {} queued", card.alive_count, card.queued_count),
                rect.x + 10.0,
                rect.y + 56.0,
                rect.w - 20.0,
                10.0,
                dark::TEXT_DIM,
            );
        } else {
            let split = if card.path_split.is_empty() {
                "Routes: none".to_string()
            } else {
                card.path_split
                    .iter()
                    .map(|(id, frac)| {
                        format!("{} {:.0}%", self.path_display_name(id), frac * 100.0)
                    })
                    .collect::<Vec<_>>()
                    .join(" / ")
            };
            draw_bounded_text(
                &split,
                rect.x + 10.0,
                rect.y + 56.0,
                rect.w - 20.0,
                10.0,
                dark::TEXT_DIM,
            );
        }
        draw_bounded_text(
            &card.recommendation,
            rect.x + 10.0,
            rect.y + rect.h - 12.0,
            rect.w - 20.0,
            10.0,
            dark::ACCENT,
        );
    }

    #[allow(dead_code)]
    fn draw_resource_affordance(&self, rect: Rect) {
        ui::draw_console_panel(rect, Color::new(0.18, 0.34, 0.28, 0.56));
        draw_ui_text(
            "CAN AFFORD",
            rect.x + 10.0,
            rect.y + 16.0,
            11.0,
            dark::POSITIVE,
        );
        draw_bounded_text(
            &format!("Scrap {:.0}", self.resources.scrap),
            rect.x + 10.0,
            rect.y + 40.0,
            rect.w - 20.0,
            17.0,
            dark::TEXT_BRIGHT,
        );
        draw_bounded_text(
            &format!(
                "Battery {:.0} / Power draw visible in top deck",
                self.resources.power
            ),
            rect.x + 10.0,
            rect.y + 62.0,
            rect.w - 20.0,
            10.0,
            dark::TEXT_DIM,
        );
    }

    fn recommended_tower_for_position<'a>(
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
            return "Coverage potential: no tower unlocked".to_string();
        };
        let covered = self.covered_paths_for_range(pos, self.effective_tower_range(def.base_range));
        if covered.is_empty() {
            format!("{} covers no active route", def.name)
        } else {
            format!(
                "{} covers {}",
                def.name,
                covered
                    .iter()
                    .map(|id| self.path_display_name(id))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    pub(super) fn draw_hover_tooltip(&self, data: &GameData) {
        let (mx, my) = mouse_position();
        let world_mouse = self.screen_to_world(vec2(mx, my));

        if let Some(ref tower_id) = self.placing_tower {
            if let Some(def) = data.tower_def_by_id(tower_id) {
                let preview = self.placement_preview(data, world_mouse);
                let covers = preview
                    .as_ref()
                    .map(|p| {
                        if p.covered_paths.is_empty() {
                            "none".to_string()
                        } else {
                            p.covered_paths
                                .iter()
                                .map(|id| self.path_display_name(id))
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    })
                    .unwrap_or_else(|| "none".to_string());
                let expected = preview
                    .as_ref()
                    .map(|p| format_enemy_counts(&p.expected_targets))
                    .unwrap_or_else(|| "No preview".to_string());
                let afford = if self.resources.scrap < def.cost_scrap {
                    format!("Need {:.0} scrap", def.cost_scrap - self.resources.scrap)
                } else {
                    format!("{:.0} scrap / {:.0} power", def.cost_scrap, def.cost_power)
                };
                let lines = vec![
                    def.name.clone(),
                    format!("Covers: {}", covers),
                    format!("Expected: {}", expected),
                    format!("Cost: {}", afford),
                ];
                draw_tooltip_box(mx, my, 300.0, &lines);
                return;
            }
        }

        let mut tooltip = None;

        let core_dist = (world_mouse - self.map_state.factory_core).length();
        if core_dist <= 26.0 {
            let next_upg = self
                .available_upgrades()
                .into_iter()
                .find(|u| !self.factory.has_upgrade(&u.id));
            if let Some(upg) = next_upg {
                tooltip = Some(format!("{} - {} scrap", upg.name, upg.cost_scrap as i32));
            } else {
                tooltip = Some("Factory Console".to_string());
            }
        } else if let Some((slot_idx, _)) = self.map_state.nearest_slot(world_mouse) {
            let slot = &self.map_state.slots[slot_idx];
            if let Some(ref entrance) = slot.opens_entrance {
                tooltip = Some(format!("Unlocks {}", entrance_label(entrance)));
            }
        } else if let Some((b_idx, _)) = self.nearest_unlocked_building(world_mouse) {
            let building = &self.map_state.buildings[b_idx];
            if let Some(ref entrance) = building.opens_entrance {
                tooltip = Some(format!("Unlocks {}", entrance_label(entrance)));
            }
        }

        let Some(text) = tooltip else {
            return;
        };
        draw_tooltip_box(mx, my, self.constants.ui.tooltip_w, &[text]);
    }
}

fn building_icon(boon: &crate::data::BuildingBoon, risk: f32) -> ConsoleIcon {
    if boon.scrap_per_sec > 0.0 {
        ConsoleIcon::Scrap
    } else if boon.food_per_sec > 0.0 {
        ConsoleIcon::Food
    } else if boon.water_per_sec > 0.0 {
        ConsoleIcon::Water
    } else if boon.power_per_sec > 0.0 {
        ConsoleIcon::Power
    } else if risk > 0.0 {
        ConsoleIcon::Risk
    } else {
        ConsoleIcon::Locked
    }
}

fn draw_info_chip(label: &str, value: &str, x: f32, y: f32, w: f32, color: Color) {
    let h = 28.0;
    draw_rectangle(x, y, w, h, Color::new(0.055, 0.07, 0.078, 0.92));
    draw_rectangle_lines(x, y, w, h, 1.5, Color::new(color.r, color.g, color.b, 0.62));
    draw_bounded_text(label, x + 8.0, y + 10.0, w - 16.0, 9.0, dark::TEXT_DIM);
    draw_bounded_text(value, x + 8.0, y + 23.0, w - 16.0, 12.0, color);
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
        draw_bounded_text(
            line,
            x + 8.0,
            y + 16.0 + idx as f32 * 15.0,
            w - 16.0,
            12.0,
            color,
        );
    }
}

fn draw_bounded_text(text: &str, x: f32, y: f32, max_w: f32, font_size: f32, color: Color) {
    let bounded = truncate_text(text, max_w, font_size as u16);
    draw_ui_text(&bounded, x, y, font_size, color);
}

fn truncate_text(text: &str, max_w: f32, font_size: u16) -> String {
    if measure_ui_text(text, None, font_size, 1.0).width <= max_w {
        return text.to_string();
    }

    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}...", out, ch);
        if measure_ui_text(&candidate, None, font_size, 1.0).width > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{}...", out)
}
