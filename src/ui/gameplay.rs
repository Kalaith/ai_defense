use crate::data::GameData;
use crate::engine::map::{BuildingState, SlotState};
use crate::ui;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::button;

use super::helpers::{beacon_color, threat_color};
use super::{GameplayState, TowerUiStats};

impl GameplayState {
    fn building_display_name_by_id(&self, id: &str) -> String {
        let raw = self
            .map_state
            .buildings
            .iter()
            .find(|b| b.id == id)
            .map(|b| {
                if b.building_type.is_empty() {
                    b.id.as_str()
                } else {
                    b.building_type.as_str()
                }
            })
            .unwrap_or(id);
        raw.split('_')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => {
                        let mut out = String::new();
                        out.push(first.to_ascii_uppercase());
                        out.push_str(chars.as_str());
                        out
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    pub fn draw_build_panel(&mut self, data: &GameData) {
        let unlocked_towers: Vec<_> = data
            .tower_defs
            .iter()
            .filter(|t| self.is_tower_unlocked(&t.id))
            .cloned()
            .collect();
        let build_clicked = ui::draw_build_panel(
            0.0,
            self.constants.ui.hud_height,
            self.constants.ui.build_panel_w,
            &unlocked_towers,
            self.resources.scrap,
            self.resources.power,
        );
        if let Some(tower_id) = build_clicked {
            self.placing_tower = Some(tower_id);
        }
    }

    pub fn draw_sector_panel(&mut self) {
        let sector_x = screen_width() - self.constants.ui.sector_panel_w;
        let panel_y = self.constants.ui.hud_height;
        let panel_w = self.constants.ui.sector_panel_w;
        let panel_h = screen_height() - panel_y;

        ui::draw_console_panel(
            Rect::new(sector_x, panel_y, panel_w, panel_h),
            Color::new(0.26, 0.42, 0.48, 0.82),
        );
        ui::draw_console_header(
            sector_x + 12.0,
            panel_y + 26.0,
            "SYSTEMS",
            "repair / power / upgrade",
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
                ("LOCKED", dark::TEXT_DIM)
            } else {
                match building.state {
                    BuildingState::Broken => ("DAMAGED", dark::NEGATIVE),
                    BuildingState::Repaired => ("REPAIRED", dark::WARNING),
                    BuildingState::Powered => ("ONLINE", dark::POSITIVE),
                    BuildingState::Disabled => ("OFFLINE", dark::TEXT_DIM),
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

            let icon = icon_for_boon(&building.boon, building.threat_per_sec);
            ui::draw_icon(icon, row_rect.x + 12.0, row_rect.y + 18.0, 30.0, border);

            let name = self.building_display_name(building);
            draw_panel_text(
                &name,
                row_rect.x + 50.0,
                row_y + 20.0,
                row_rect.w - 62.0,
                14.0,
                dark::TEXT,
            );
            ui::draw_status_pill(row_rect.x + 50.0, row_y + 27.0, status, status_color);

            let b = &building.boon;
            let detail = self.boon_text(b);
            let cost_line = match building.state {
                BuildingState::Broken => format!("Repair Cost: {:.0} scrap", building.repair_cost),
                BuildingState::Repaired => format!("Power Cost: {:.0} scrap", building.power_cost),
                BuildingState::Powered => "Online".to_string(),
                BuildingState::Disabled => "Locked".to_string(),
            };
            draw_panel_text(
                &cost_line,
                row_rect.x + 12.0,
                row_y + 58.0,
                row_rect.w - 126.0,
                11.0,
                dark::TEXT_DIM,
            );
            draw_panel_text(
                &format!("Benefit: {}", detail),
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
            draw_panel_text(
                &format!("Risk: +{:.2} noise/s", building.threat_per_sec),
                row_rect.x + 12.0,
                row_y + 89.0,
                row_rect.w - 126.0,
                10.0,
                dark::WARNING,
            );

            let missing_repair = (building.repair_cost - self.resources.scrap).max(0.0);
            let missing_power = (building.power_cost - self.resources.scrap).max(0.0);
            let (action_label, action_kind, action_state) = if !unlocked {
                (
                    "LOCKED".to_string(),
                    "locked",
                    ui::ConsoleButtonState::Disabled,
                )
            } else {
                match building.state {
                    BuildingState::Broken => {
                        if self.resources.scrap >= building.repair_cost {
                            (
                                "REPAIR".to_string(),
                                "repair",
                                ui::ConsoleButtonState::Affordable,
                            )
                        } else {
                            (
                                format!("NEED {:.0} SCRAP", missing_repair),
                                "repair",
                                ui::ConsoleButtonState::Disabled,
                            )
                        }
                    }
                    BuildingState::Repaired => {
                        if self.resources.scrap >= building.power_cost {
                            (
                                "POWER".to_string(),
                                "power",
                                ui::ConsoleButtonState::Recommended,
                            )
                        } else {
                            (
                                format!("NEED {:.0} SCRAP", missing_power),
                                "power",
                                ui::ConsoleButtonState::Disabled,
                            )
                        }
                    }
                    BuildingState::Powered => (
                        "ONLINE".to_string(),
                        "online",
                        ui::ConsoleButtonState::Disabled,
                    ),
                    BuildingState::Disabled => (
                        "LOCKED".to_string(),
                        "locked",
                        ui::ConsoleButtonState::Disabled,
                    ),
                }
            };
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
            self.selected_building = Some(idx);
            self.selected_slot = None;
            self.selected_tower = None;
            self.selected_core = false;
            self.selected_upgrade = None;
            match action {
                "repair" => self.repair_building(idx),
                "power" => self.power_building(idx),
                _ => {}
            }
        } else if let Some(idx) = clicked_building {
            self.selected_building = Some(idx);
            self.selected_slot = None;
            self.selected_tower = None;
            self.selected_core = false;
            self.selected_upgrade = None;
        }
    }

    pub fn draw_placement_ghost(&self, data: &GameData) {
        let world_mouse = self.screen_to_world(vec2(mouse_position().0, mouse_position().1));
        let Some(preview) = self.placement_preview(data, world_mouse) else {
            return;
        };
        let Some(def) = data.tower_def_by_id(&preview.tower_id) else {
            return;
        };

        let range = self.effective_tower_range(def.base_range);
        let c = def.color();

        for path in self.map_state.active_paths_limited() {
            for pair in path.points.windows(2) {
                draw_line(
                    pair[0].x,
                    pair[0].y,
                    pair[1].x,
                    pair[1].y,
                    9.0,
                    Color::new(1.0, 0.48, 0.08, 0.24),
                );
                draw_line(
                    pair[0].x,
                    pair[0].y,
                    pair[1].x,
                    pair[1].y,
                    4.0,
                    Color::new(1.0, 0.7, 0.22, 0.8),
                );
            }
        }

        for idx in preview.valid_slots {
            if let Some(slot) = self.map_state.slots.get(idx) {
                let pulse = 0.55 + 0.28 * (get_time() as f32 * 5.0).sin().abs();
                draw_circle(
                    slot.position.x,
                    slot.position.y,
                    19.0,
                    Color::new(0.06, 0.25, 0.22, 0.34),
                );
                draw_circle_lines(
                    slot.position.x,
                    slot.position.y,
                    23.0,
                    3.0,
                    Color::new(0.26, 0.95, 0.82, pulse),
                );
            }
        }

        for (idx, reason) in preview.invalid_slots {
            if let Some(slot) = self.map_state.slots.get(idx) {
                draw_circle(
                    slot.position.x,
                    slot.position.y,
                    17.0,
                    Color::new(0.02, 0.025, 0.03, 0.58),
                );
                draw_circle_lines(
                    slot.position.x,
                    slot.position.y,
                    18.0,
                    1.5,
                    Color::new(0.45, 0.48, 0.5, 0.32),
                );
                if preview.hovered_slot == Some(idx) {
                    draw_text(
                        reason.label(),
                        slot.position.x + 18.0,
                        slot.position.y + 4.0,
                        12.0,
                        dark::TEXT_DIM,
                    );
                }
            }
        }

        if let Some(idx) = preview.hovered_slot {
            if let Some(slot) = self.map_state.slots.get(idx) {
                let ghost_color = Color::new(c.r, c.g, c.b, 0.5);
                let range_color = Color::new(c.r, c.g, c.b, 0.32);
                draw_circle(
                    slot.position.x,
                    slot.position.y,
                    self.constants.ui.tower_base_radius,
                    ghost_color,
                );
                draw_circle_lines(slot.position.x, slot.position.y, range, 2.0, range_color);
                for path_id in &preview.covered_paths {
                    if let Some(path) = self.map_state.paths.iter().find(|p| p.id == *path_id) {
                        for pair in path.points.windows(2) {
                            draw_line(
                                pair[0].x,
                                pair[0].y,
                                pair[1].x,
                                pair[1].y,
                                6.0,
                                Color::new(c.r, c.g, c.b, 0.42),
                            );
                        }
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn draw_shutdown_ui(&mut self) {
        let panel_w = 226.0;
        let panel_h = self.constants.ui.hud_height - 10.0;
        let panel_x = screen_width() - self.constants.ui.sector_panel_w - panel_w - 8.0;
        let panel_y = 5.0;
        let panel_surface =
            macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.075, 0.065, 0.07, 0.96))
                .with_border(1.0, Color::new(0.42, 0.22, 0.18, 0.85));
        macroquad_toolkit::ui::draw_surface(
            Rect::new(panel_x, panel_y, panel_w, panel_h),
            &panel_surface,
        );

        draw_text(
            "BEACON DRAW",
            panel_x + 10.0,
            panel_y + 14.0,
            10.0,
            dark::NEGATIVE,
        );
        let phase_color = beacon_color(&self.beacon_phase);
        draw_panel_text(
            self.beacon_phase.label(),
            panel_x + 10.0,
            panel_y + 31.0,
            98.0,
            12.0,
            phase_color,
        );
        draw_signal_bar(
            panel_x + 10.0,
            panel_y + panel_h - 10.0,
            92.0,
            5.0,
            self.beacon_strength,
            phase_color,
        );

        draw_text(
            &format!("Teams {}", self.scavengers_out),
            panel_x + 108.0,
            panel_y + 31.0,
            11.0,
            dark::TEXT_DIM,
        );
        draw_text(
            &format!(
                "{} {:.0}",
                self.threat.reaction_tier().label(),
                self.threat.awareness_level()
            ),
            panel_x + 108.0,
            panel_y + 46.0,
            10.0,
            threat_color(&self.threat),
        );

        let btn_h = 20.0;
        if !self.beacon_active {
            let btn_w = 96.0;
            let btn_x = panel_x + panel_w - btn_w - 8.0;
            let btn_y = panel_y + 8.0;
            if button(btn_x, btn_y, btn_w, btn_h, "Start") {
                self.start_beacon();
            }
            return;
        }

        let shutdown_enabled = !self.shutdown_triggered && self.current_wave >= 1;
        let shutdown_x = panel_x + panel_w - 104.0;
        let shutdown_y = panel_y + 8.0;
        if shutdown_enabled {
            if button(shutdown_x, shutdown_y, 96.0, btn_h, "Shutdown") {
                self.trigger_shutdown();
            }
        } else {
            draw_rectangle(
                shutdown_x,
                shutdown_y,
                96.0,
                btn_h,
                Color::new(0.18, 0.16, 0.16, 0.78),
            );
            draw_centered_ui_text(
                "Locked",
                shutdown_x + 48.0,
                shutdown_y + 14.0,
                11.0,
                dark::TEXT_DIM,
            );
        }

        if !self.scavenger_recall_active
            && self.scavengers_out > 0
            && button(
                panel_x + panel_w - 104.0,
                panel_y + 31.0,
                96.0,
                btn_h,
                "Recall",
            )
        {
            self.scavenger_recall_active = true;
            self.scavenger_recall_timer = 0.0;
        }
    }

    #[allow(dead_code)]
    pub fn draw_selected_tower_panel(&mut self, data: &GameData) {
        let Some(idx) = self.selected_tower else {
            return;
        };
        let Some(tower) = self.towers.get(idx) else {
            return;
        };

        let panel_x = 0.0;
        let panel_y = self.constants.ui.hud_height;
        let panel_w = self.constants.ui.build_panel_w;
        let panel_h = screen_height() - panel_y;

        let name = data
            .tower_def_by_id(&tower.tower_id)
            .map(|d| d.name.as_str())
            .unwrap_or("Tower");

        let info_x = panel_x + 10.0;
        let mut info_y = panel_y + panel_h - 110.0;

        draw_text("SELECTED", info_x, info_y, 14.0, dark::TEXT_DIM);
        info_y += 16.0;
        draw_text(
            &format!("{} (Lv {})", name, tower.level),
            info_x,
            info_y,
            14.0,
            dark::TEXT_BRIGHT,
        );
        info_y += 16.0;
        draw_text(
            &format!("Dmg: {:.1}", tower.damage),
            info_x,
            info_y,
            12.0,
            dark::TEXT,
        );
        info_y += 14.0;
        draw_text(
            &format!("Rng: {:.0}", tower.range),
            info_x,
            info_y,
            12.0,
            dark::TEXT,
        );
        info_y += 14.0;
        draw_text(
            &format!("Rate: {:.2}/s", tower.fire_rate),
            info_x,
            info_y,
            12.0,
            dark::TEXT,
        );

        let btn_w = panel_w - 20.0;
        let btn_h = 24.0;
        let btn_x = panel_x + 10.0;
        let btn_y = panel_y + panel_h - 30.0;

        if tower.level >= self.constants.tower.upgrade_max_level {
            draw_text("Max Level", btn_x, btn_y + 16.0, 12.0, dark::TEXT_DIM);
            return;
        }

        if !self.factory.is_sector_active("research_lab") {
            draw_text(
                "Requires Research Lab",
                btn_x,
                btn_y + 16.0,
                12.0,
                dark::TEXT_DIM,
            );
            return;
        }

        let upgrade_cost = tower.base_scrap_cost * tower.level as f32;
        let label = format!("Upgrade ({:.0})", upgrade_cost);
        if self.resources.scrap >= upgrade_cost {
            if button(btn_x, btn_y, btn_w, btn_h, &label) {
                self.upgrade_tower(idx);
            }
        } else {
            let disabled_surface =
                macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.2, 0.2, 0.2, 0.5));
            macroquad_toolkit::ui::draw_surface(
                Rect::new(btn_x, btn_y, btn_w, btn_h),
                &disabled_surface,
            );
            let dims = measure_text(&label, None, 12, 1.0);
            draw_text(
                &label,
                btn_x + (btn_w - dims.width) / 2.0,
                btn_y + 16.0,
                12.0,
                dark::TEXT_DIM,
            );
        }
    }

    pub fn handle_map_click(&mut self, data: &GameData) {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        let (mx, my) = mouse_position();

        // Skip if click is in UI panels
        if mx < self.constants.ui.build_panel_w
            || mx > screen_width() - self.constants.ui.sector_panel_w
            || my < self.constants.ui.hud_height
        {
            return;
        }

        // Skip if in bottom slot panel area
        let play_x = self.constants.ui.build_panel_w + 8.0;
        let play_w =
            (screen_width() - self.constants.ui.build_panel_w - self.constants.ui.sector_panel_w)
                .max(360.0)
                - 16.0;
        let panel_w = play_w;
        let panel_h = self.constants.ui.bottom_context_h;
        let panel_x = play_x;
        let panel_y = screen_height() - panel_h - 8.0;
        if my >= panel_y && my <= panel_y + panel_h && mx >= panel_x && mx <= panel_x + panel_w {
            return;
        }
        if self.selected_core {
            let core_panel_x = panel_x - 10.0;
            let core_panel_w = panel_w + 20.0;
            let core_panel_top = panel_y - 160.0;
            if mx >= core_panel_x
                && mx <= core_panel_x + core_panel_w
                && my >= core_panel_top
                && my <= panel_y + panel_h
            {
                return;
            }
        }

        let world_pos = self.screen_to_world(vec2(mx, my));

        let core_dist = (world_pos - self.map_state.factory_core).length();
        if core_dist <= 26.0 {
            self.selected_core = true;
            self.selected_slot = None;
            self.selected_building = None;
            self.selected_tower = None;
            if self.selected_upgrade.is_none() {
                if let Some(upg) = self
                    .available_upgrades()
                    .into_iter()
                    .find(|u| !self.factory.has_upgrade(&u.id))
                {
                    self.selected_upgrade = Some(upg.id.clone());
                }
            }
            return;
        }

        // If placing tower and clicked a powered empty slot -> place tower
        if let Some(ref tower_id) = self.placing_tower.clone() {
            if let Some((slot_idx, _)) = self.map_state.nearest_slot(world_pos) {
                let slot = &self.map_state.slots[slot_idx];
                if slot.state == SlotState::Powered && slot.tower_index.is_none() {
                    self.try_place_tower_on_slot(slot_idx, tower_id, data);
                    return;
                }
            }
        }

        // Try to find nearest slot or building for selection
        let slot_result = self.map_state.nearest_slot(world_pos);
        let building_result = self.map_state.nearest_building(world_pos);

        // Pick the closer of slot vs building
        match (slot_result, building_result) {
            (Some((si, sd)), Some((bi, bd))) => {
                if sd <= bd {
                    self.selected_slot = Some(si);
                    self.selected_building = None;
                    self.selected_tower = self.map_state.slots[si].tower_index;
                    self.selected_core = false;
                    self.selected_upgrade = None;
                } else {
                    self.selected_building = Some(bi);
                    self.selected_slot = None;
                    self.selected_tower = None;
                    self.selected_core = false;
                    self.selected_upgrade = None;
                }
            }
            (Some((si, _)), None) => {
                self.selected_slot = Some(si);
                self.selected_building = None;
                self.selected_tower = self.map_state.slots[si].tower_index;
                self.selected_core = false;
                self.selected_upgrade = None;
            }
            (None, Some((bi, _))) => {
                self.selected_building = Some(bi);
                self.selected_slot = None;
                self.selected_tower = None;
                self.selected_core = false;
                self.selected_upgrade = None;
            }
            (None, None) => {
                self.selected_slot = None;
                self.selected_building = None;
                self.selected_tower = None;
                self.selected_core = false;
                self.selected_upgrade = None;
            }
        }
    }

    pub(crate) fn clear_slot(&mut self, idx: usize) {
        if self.beacon_active {
            self.push_notification("Repairs locked during beacon operation".to_string());
            return;
        }
        let cost = self.map_state.slots[idx].clear_cost;
        if let Some(req) = self.map_state.slots[idx].requires_building_power.as_deref() {
            if !self.map_state.is_building_powered(req) {
                let name = self.building_display_name_by_id(req);
                self.push_notification(format!("Requires power from {}", name));
                return;
            }
        }
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let newly_active = self.map_state.set_slot_state(idx, SlotState::Cleared);
        for path_id in &newly_active {
            self.push_notification(format!("New path opened: {}", path_id));
        }
    }

    pub(crate) fn power_slot(&mut self, idx: usize) {
        if self.beacon_active {
            self.push_notification("Repairs locked during beacon operation".to_string());
            return;
        }
        let cost = self.map_state.slots[idx].power_cost;
        if let Some(req) = self.map_state.slots[idx].requires_building_power.as_deref() {
            if !self.map_state.is_building_powered(req) {
                let name = self.building_display_name_by_id(req);
                self.push_notification(format!("Requires power from {}", name));
                return;
            }
        }
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let newly_active = self.map_state.set_slot_state(idx, SlotState::Powered);
        for path_id in &newly_active {
            self.push_notification(format!("New path opened: {}", path_id));
        }
    }

    pub(crate) fn try_place_tower_on_slot(
        &mut self,
        slot_idx: usize,
        tower_id: &str,
        data: &GameData,
    ) {
        if !self.is_tower_unlocked(tower_id) {
            return;
        }
        let slot = &self.map_state.slots[slot_idx];
        if slot.state != SlotState::Powered || slot.tower_index.is_some() {
            return;
        }

        let Some(def) = data.tower_def_by_id(tower_id) else {
            return;
        };
        if self.resources.scrap < def.cost_scrap {
            return;
        }

        let gen = self.factory.power_generation();
        let current_drain: f32 = self.factory.power_consumption()
            + self
                .towers
                .iter()
                .filter(|t| t.is_active)
                .map(|t| t.power_drain)
                .sum::<f32>();
        let net_after = gen - current_drain - def.cost_power;
        if net_after < 0.0
            && self.resources.power < self.constants.economy.power_buffer_min_for_build
        {
            return;
        }

        self.resources.scrap -= def.cost_scrap;
        let pos = slot.position;

        let tt = def.tower_type.clone();

        let tower_idx = self.towers.len();
        self.towers.push(crate::engine::tower::Tower::new(
            tt,
            def.id.clone(),
            pos,
            def.base_range,
            def.base_damage,
            def.fire_rate,
            def.cost_power,
            def.cost_scrap,
            def.color(),
        ));
        self.tower_stats.push(TowerUiStats::default());

        self.map_state.slots[slot_idx].tower_index = Some(tower_idx);
        self.threat.add_noise(0.5);
        self.placing_tower = None;
        self.selected_tower = Some(tower_idx);
    }

    pub(crate) fn repair_building(&mut self, idx: usize) {
        if self.beacon_active {
            self.push_notification("Repairs locked during beacon operation".to_string());
            return;
        }
        if let Some(req) = self.map_state.buildings[idx].requires_power_from.as_deref() {
            if !self.map_state.is_building_powered(req) {
                let name = self.building_display_name_by_id(req);
                self.push_notification(format!("Requires power from {}", name));
                return;
            }
        }
        let cost = self.map_state.buildings[idx].repair_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        self.map_state
            .set_building_state(idx, BuildingState::Repaired);
        self.push_notification(format!("{} repaired", self.map_state.buildings[idx].id));
    }

    pub(crate) fn power_building(&mut self, idx: usize) {
        if self.beacon_active {
            self.push_notification("Repairs locked during beacon operation".to_string());
            return;
        }
        if let Some(req) = self.map_state.buildings[idx].requires_power_from.as_deref() {
            if !self.map_state.is_building_powered(req) {
                let name = self.building_display_name_by_id(req);
                self.push_notification(format!("Requires power from {}", name));
                return;
            }
        }
        let cost = self.map_state.buildings[idx].power_cost;
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        let newly_active = self
            .map_state
            .set_building_state(idx, BuildingState::Powered);
        self.push_notification(format!("{} powered on", self.map_state.buildings[idx].id));
        for path_id in &newly_active {
            self.push_notification(format!("New entrance opened: {}", path_id));
        }
    }

    pub(crate) fn purchase_upgrade(&mut self, upgrade_id: &str) {
        let def = self
            .upgrade_defs
            .iter()
            .find(|u| u.id == upgrade_id)
            .cloned();
        if let Some(def) = def {
            if self
                .factory
                .can_purchase(&def, self.resources.scrap, self.resources.power)
            {
                self.resources.scrap -= def.cost_scrap;
                self.factory.purchase_upgrade(&def);
            }
        }
    }

    pub(crate) fn start_beacon(&mut self) {
        self.beacon_active = true;
        self.between_waves = true;
        self.wave_timer = self.constants.gameplay.beacon_start_delay;
        self.beacon_start_difficulty_bonus = self.compute_beacon_start_difficulty_bonus();
        if self.scavengers_sent == 0 {
            self.scavengers_out = self.constants.scavenger.initial_scavengers;
            self.scavengers_sent = self.constants.scavenger.initial_scavengers;
        }
        self.push_notification("Beacon activated. Machines inbound.".to_string());
    }

    pub(crate) fn trigger_shutdown(&mut self) {
        self.shutdown_triggered = true;
        self.beacon_active = false;
        self.beacon_start_difficulty_bonus = 0.0;
        self.scavenger_recall_active = true;
        self.scavenger_recall_timer = 0.0;
        self.wave_manager.spawn_queue.clear();
        self.between_waves = false;
        self.push_notification("Beacon shut down. No new waves incoming.".to_string());
    }
}

fn draw_panel_text(text: &str, x: f32, y: f32, max_w: f32, font_size: f32, color: Color) {
    let bounded = truncate_text(text, max_w, font_size as u16);
    draw_text(&bounded, x, y, font_size, color);
}

fn truncate_text(text: &str, max_w: f32, font_size: u16) -> String {
    if measure_text(text, None, font_size, 1.0).width <= max_w {
        return text.to_string();
    }

    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}...", out, ch);
        if measure_text(&candidate, None, font_size, 1.0).width > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{}...", out)
}

#[allow(dead_code)]
fn draw_signal_bar(x: f32, y: f32, w: f32, h: f32, strength: f32, color: Color) {
    draw_rectangle(x, y, w, h, Color::new(0.18, 0.09, 0.08, 0.8));
    let fill_w = w * (strength / 70.0).clamp(0.0, 1.0);
    draw_rectangle(x, y, fill_w, h, color);
}

#[allow(dead_code)]
fn draw_centered_ui_text(text: &str, center_x: f32, baseline_y: f32, font_size: f32, color: Color) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(
        text,
        center_x - dims.width * 0.5,
        baseline_y,
        font_size,
        color,
    );
}

fn icon_for_boon(boon: &crate::data::BuildingBoon, risk: f32) -> ui::ConsoleIcon {
    if boon.scrap_per_sec > 0.0 {
        ui::ConsoleIcon::Scrap
    } else if boon.food_per_sec > 0.0 {
        ui::ConsoleIcon::Food
    } else if boon.water_per_sec > 0.0 {
        ui::ConsoleIcon::Water
    } else if boon.power_per_sec > 0.0 {
        ui::ConsoleIcon::Power
    } else if risk > 0.0 {
        ui::ConsoleIcon::Risk
    } else {
        ui::ConsoleIcon::Locked
    }
}
