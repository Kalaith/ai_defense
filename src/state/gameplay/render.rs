use crate::data::GameData;
use crate::engine::enemy::EnemyType;
use crate::engine::map::{BuildingState, SlotState, TraceNode};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::notifications::{
    draw_notification, Notification, NotificationRenderConfig, NotificationType,
};
use macroquad_toolkit::ui::button;

use super::helpers::{beacon_color, enemy_label, entrance_label};
use super::GameplayState;

impl GameplayState {
    pub fn draw(&mut self, data: &GameData) {
        // --- World-space rendering (with camera) ---
        let sw = screen_width();
        let sh = screen_height();
        let cam = Camera2D {
            target: self.camera_offset,
            zoom: vec2(self.camera_zoom * 2.0 / sw, self.camera_zoom * 2.0 / sh),
            ..Default::default()
        };
        set_camera(&cam);

        self.draw_circuit_board();
        self.draw_towers();
        self.draw_enemies();
        self.draw_shot_effects();
        self.draw_particles();
        self.draw_placement_ghost(data);

        // --- Screen-space rendering (UI) ---
        set_default_camera();

        self.draw_hud();
        self.draw_wave_start_flash();
        self.draw_wave_status();
        self.draw_notifications();
        self.draw_shutdown_ui();
        self.draw_build_panel(data);
        self.draw_sector_panel();
        self.draw_slot_panel(data);
        self.draw_hover_tooltip();
        self.handle_map_click(data);
        self.draw_selected_tower_panel(data);
    }

    fn draw_circuit_board(&self) {
        let map_w = self.map_state.map_size.x;
        let map_h = self.map_state.map_size.y;

        // 1. PCB background
        draw_rectangle(0.0, 0.0, map_w, map_h, Color::new(0.02, 0.08, 0.04, 1.0));

        // 1b. Section backplates + corridor connectors
        let sections = self.map_state.section_render_info();
        let mut section_centers: Vec<Vec2> = Vec::new();
        for section in &sections {
            let pad_x = 80.0 + (section.index as f32 * 6.0);
            let pad_y = 70.0 + (section.index as f32 * 4.0);
            let min = vec2(section.min.x - pad_x, section.min.y - pad_y);
            let max = vec2(section.max.x + pad_x, section.max.y + pad_y);
            let w = (max.x - min.x).max(120.0);
            let h = (max.y - min.y).max(80.0);
            let base = 0.05 + (section.index as f32 * 0.02).min(0.28);
            let fill = Color::new(0.02 + base, 0.08 + base * 0.5, 0.04 + base * 0.2, 0.55);
            let border = Color::new(0.08 + base, 0.22 + base, 0.12 + base * 0.6, 0.65);

            let kind = section.label.to_lowercase();
            if kind.contains("intake") || kind.contains("scrap") {
                draw_rectangle(min.x, min.y + h * 0.15, w, h * 0.7, fill);
                draw_rectangle_lines(min.x, min.y + h * 0.15, w, h * 0.7, 2.0, border);
            } else if kind.contains("water") || kind.contains("hydro") {
                let cx = min.x + w * 0.5;
                let cy = min.y + h * 0.5;
                draw_circle(cx, cy, h * 0.45, fill);
                draw_circle_lines(cx, cy, h * 0.45, 2.5, border);
                draw_rectangle(
                    min.x + w * 0.3,
                    min.y + h * 0.15,
                    w * 0.4,
                    h * 0.7,
                    Color::new(fill.r, fill.g, fill.b, 0.6),
                );
            } else if kind.contains("power") {
                draw_rectangle(min.x, min.y + h * 0.35, w, h * 0.3, fill);
                draw_rectangle_lines(min.x, min.y + h * 0.35, w, h * 0.3, 2.0, border);
            } else if kind.contains("assembly") {
                draw_rectangle(min.x, min.y, w, h, fill);
                draw_rectangle_lines(min.x, min.y, w, h, 2.5, border);
                for i in 1..4 {
                    let x = min.x + w * (i as f32 / 4.0);
                    draw_line(
                        x,
                        min.y + 6.0,
                        x,
                        min.y + h - 6.0,
                        1.0,
                        Color::new(border.r, border.g, border.b, 0.4),
                    );
                }
                for i in 1..3 {
                    let y = min.y + h * (i as f32 / 3.0);
                    draw_line(
                        min.x + 6.0,
                        y,
                        min.x + w - 6.0,
                        y,
                        1.0,
                        Color::new(border.r, border.g, border.b, 0.4),
                    );
                }
            } else if kind.contains("logistics") {
                draw_rectangle(min.x, min.y, w, h, fill);
                draw_rectangle_lines(min.x, min.y, w, h, 2.0, border);
                draw_circle_lines(
                    min.x + w * 0.3,
                    min.y + h * 0.5,
                    h * 0.25,
                    2.0,
                    Color::new(border.r, border.g, border.b, 0.6),
                );
                draw_circle_lines(
                    min.x + w * 0.7,
                    min.y + h * 0.5,
                    h * 0.25,
                    2.0,
                    Color::new(border.r, border.g, border.b, 0.6),
                );
            } else if kind.contains("robotics") {
                draw_rectangle(min.x, min.y + h * 0.1, w, h * 0.8, fill);
                draw_rectangle_lines(min.x, min.y + h * 0.1, w, h * 0.8, 2.5, border);
                draw_rectangle_lines(
                    min.x + w * 0.2,
                    min.y + h * 0.2,
                    w * 0.6,
                    h * 0.6,
                    2.0,
                    Color::new(border.r, border.g, border.b, 0.6),
                );
            } else if kind.contains("research") {
                draw_rectangle(min.x, min.y, w, h, fill);
                draw_rectangle_lines(min.x, min.y, w, h, 2.0, border);
                let a = vec2(min.x + w * 0.15, min.y + h * 0.2);
                let b = vec2(min.x + w * 0.85, min.y + h * 0.5);
                let c = vec2(min.x + w * 0.25, min.y + h * 0.8);
                draw_triangle_lines(a, b, c, 2.0, Color::new(border.r, border.g, border.b, 0.7));
            } else if kind.contains("vault") || kind.contains("ai") {
                draw_rectangle(min.x + w * 0.1, min.y + h * 0.1, w * 0.8, h * 0.8, fill);
                draw_rectangle_lines(
                    min.x + w * 0.1,
                    min.y + h * 0.1,
                    w * 0.8,
                    h * 0.8,
                    3.0,
                    border,
                );
                draw_rectangle_lines(
                    min.x + w * 0.2,
                    min.y + h * 0.2,
                    w * 0.6,
                    h * 0.6,
                    2.0,
                    Color::new(border.r, border.g, border.b, 0.6),
                );
            } else if kind.contains("heart") {
                let cx = min.x + w * 0.5;
                let cy = min.y + h * 0.5;
                let r = h.min(w) * 0.45;
                draw_circle(cx, cy, r, fill);
                draw_circle_lines(cx, cy, r, 3.0, border);
                draw_circle_lines(
                    cx,
                    cy,
                    r * 0.6,
                    2.0,
                    Color::new(border.r, border.g, border.b, 0.6),
                );
            } else {
                draw_rectangle(min.x, min.y, w, h, fill);
                draw_rectangle_lines(min.x, min.y, w, h, 2.0, border);
            }

            let center = vec2((min.x + max.x) * 0.5, (min.y + max.y) * 0.5);
            section_centers.push(center);
        }

        for i in 0..section_centers.len().saturating_sub(1) {
            let a = section_centers[i];
            let b = section_centers[i + 1];
            draw_line(a.x, a.y, b.x, b.y, 10.0, Color::new(0.04, 0.14, 0.07, 0.7));
            draw_line(a.x, a.y, b.x, b.y, 4.0, Color::new(0.12, 0.35, 0.2, 0.7));
        }

        // 2. Traces
        for trace in &self.map_state.traces {
            let powered = self.map_state.trace_powered(trace);
            let from_pos = self.trace_node_pos(&trace.from);
            let to_pos = self.trace_node_pos(&trace.to);
            let from_visible = match trace.from {
                TraceNode::Slot(idx) => self
                    .map_state
                    .slots
                    .get(idx)
                    .map_or(false, |s| self.map_state.is_slot_visible(s)),
                TraceNode::Building(idx) => self
                    .map_state
                    .buildings
                    .get(idx)
                    .map_or(false, |b| self.map_state.is_building_visible(b)),
                TraceNode::FactoryCore => true,
            };
            let to_visible = match trace.to {
                TraceNode::Slot(idx) => self
                    .map_state
                    .slots
                    .get(idx)
                    .map_or(false, |s| self.map_state.is_slot_visible(s)),
                TraceNode::Building(idx) => self
                    .map_state
                    .buildings
                    .get(idx)
                    .map_or(false, |b| self.map_state.is_building_visible(b)),
                TraceNode::FactoryCore => true,
            };
            if !from_visible || !to_visible {
                continue;
            }

            let mut points = Vec::new();
            points.push(from_pos);
            for p in &trace.via {
                points.push(*p);
            }
            points.push(to_pos);

            let mut render_points = points.clone();
            if render_points.len() == 2 {
                let a = render_points[0];
                let b = render_points[1];
                let dir = (b - a).normalize_or_zero();
                let perp = vec2(-dir.y, dir.x);
                let bend = ((b - a).length() * 0.15).clamp(12.0, 40.0);
                render_points.insert(1, (a + b) * 0.5 + perp * bend);
            }

            for i in 0..render_points.len().saturating_sub(1) {
                let a = render_points[i];
                let b = render_points[i + 1];
                let mid = (a + b) * 0.5;
                let mut near_core = false;
                for building in &self.map_state.buildings {
                    if self.map_state.is_core_building(&building.id)
                        && (building.position - mid).length() < 90.0
                    {
                        near_core = true;
                        break;
                    }
                }
                if powered {
                    let wide = if near_core { 11.0 } else { 8.0 };
                    let thin = if near_core { 4.0 } else { 3.0 };
                    draw_line(a.x, a.y, b.x, b.y, wide, Color::new(0.0, 0.6, 0.2, 0.18));
                    draw_line(a.x, a.y, b.x, b.y, thin, Color::new(0.2, 1.0, 0.4, 0.85));
                } else {
                    draw_line(a.x, a.y, b.x, b.y, 3.5, Color::new(0.08, 0.2, 0.1, 0.55));
                }
            }
        }

        // 3. Enemy paths
        let max_x = self.map_state.max_visible_x();
        for path in &self.map_state.paths {
            let alpha = if path.active { 0.8 } else { 0.2 };
            let color = Color::new(0.5, 0.35, 0.15, alpha);

            for i in 0..path.points.len().saturating_sub(1) {
                let a = path.points[i];
                let b = path.points[i + 1];
                if a.x > max_x || b.x > max_x {
                    break;
                }
                draw_line(a.x, a.y, b.x, b.y, 4.0, color);
            }
        }

        // 4. Entrance markers
        for path in &self.map_state.paths {
            if path.active {
                if path.entrance.x > max_x {
                    continue;
                }
                let e = path.entrance;
                draw_circle(e.x, e.y, 14.0, Color::new(0.9, 0.2, 0.1, 0.85));
                draw_circle_lines(e.x, e.y, 18.0, 3.0, Color::new(0.9, 0.35, 0.15, 0.6));
                draw_line(
                    e.x - 10.0,
                    e.y - 4.0,
                    e.x + 12.0,
                    e.y + 6.0,
                    2.5,
                    Color::new(0.6, 0.1, 0.05, 0.6),
                );
                draw_line(
                    e.x + 8.0,
                    e.y - 8.0,
                    e.x - 6.0,
                    e.y + 10.0,
                    2.0,
                    Color::new(0.6, 0.1, 0.05, 0.6),
                );
            } else {
                if path.entrance.x > max_x {
                    continue;
                }
                draw_circle(
                    path.entrance.x,
                    path.entrance.y,
                    7.0,
                    Color::new(0.5, 0.15, 0.1, 0.35),
                );
            }
        }

        // 5. Tower slots
        for (idx, slot) in self.map_state.slots.iter().enumerate() {
            if !self.map_state.is_slot_visible(slot) {
                continue;
            }
            let selected = self.selected_slot == Some(idx);
            match slot.state {
                SlotState::Debris => {
                    draw_circle(
                        slot.position.x,
                        slot.position.y,
                        8.0,
                        Color::new(0.35, 0.25, 0.15, 0.8),
                    );
                    // X cross lines
                    let s = 5.0;
                    draw_line(
                        slot.position.x - s,
                        slot.position.y - s,
                        slot.position.x + s,
                        slot.position.y + s,
                        1.5,
                        Color::new(0.5, 0.3, 0.1, 0.6),
                    );
                    draw_line(
                        slot.position.x + s,
                        slot.position.y - s,
                        slot.position.x - s,
                        slot.position.y + s,
                        1.5,
                        Color::new(0.5, 0.3, 0.1, 0.6),
                    );
                    // Gold ring hint for slots that open entrances
                    if slot.opens_entrance.is_some() {
                        draw_circle_lines(
                            slot.position.x,
                            slot.position.y,
                            11.0,
                            1.5,
                            Color::new(0.9, 0.75, 0.2, 0.6),
                        );
                    }
                }
                SlotState::Cleared => {
                    draw_circle_lines(
                        slot.position.x,
                        slot.position.y,
                        10.0,
                        1.5,
                        Color::new(0.3, 0.4, 0.7, 0.6),
                    );
                }
                SlotState::Powered => {
                    if slot.tower_index.is_some() {
                        // Tower is placed here; tower rendering handles the visual
                        draw_circle(
                            slot.position.x,
                            slot.position.y,
                            12.0,
                            Color::new(0.05, 0.15, 0.05, 0.5),
                        );
                    } else {
                        draw_circle(
                            slot.position.x,
                            slot.position.y,
                            10.0,
                            Color::new(0.05, 0.15, 0.05, 0.8),
                        );
                        let pulse = 0.5 + 0.3 * (get_time() as f32 * 2.0).sin().abs();
                        draw_circle_lines(
                            slot.position.x,
                            slot.position.y,
                            11.0,
                            2.0,
                            Color::new(0.2, 0.8, 0.3, pulse),
                        );
                    }
                }
            }

            if selected {
                draw_circle_lines(slot.position.x, slot.position.y, 16.0, 2.0, WHITE);
            }
        }

        // 6. Buildings
        for (idx, building) in self.map_state.buildings.iter().enumerate() {
            if !self.map_state.is_building_visible(building) {
                continue;
            }
            let unlocked = self.is_building_unlocked(building);
            let selected = self.selected_building == Some(idx);
            let (bg_color, border_color) = if !unlocked {
                (
                    Color::new(0.08, 0.08, 0.08, 0.6),
                    Color::new(0.2, 0.2, 0.2, 0.6),
                )
            } else {
                match building.state {
                    BuildingState::Broken => (
                        Color::new(0.3, 0.05, 0.05, 0.8),
                        Color::new(0.5, 0.1, 0.1, 0.8),
                    ),
                    BuildingState::Repaired => (
                        Color::new(0.3, 0.3, 0.05, 0.8),
                        Color::new(0.5, 0.5, 0.1, 0.8),
                    ),
                    BuildingState::Powered => (
                        Color::new(0.05, 0.2, 0.3, 0.9),
                        Color::new(0.2, 0.7, 0.9, 0.9),
                    ),
                    BuildingState::Disabled => (
                        Color::new(0.15, 0.15, 0.15, 0.6),
                        Color::new(0.3, 0.3, 0.3, 0.6),
                    ),
                }
            };

            let is_core = self.map_state.is_core_building(&building.id);
            let w = if is_core { 62.0 } else { 40.0 };
            let h = if is_core { 44.0 } else { 30.0 };
            draw_rectangle(
                building.position.x - w / 2.0,
                building.position.y - h / 2.0,
                w,
                h,
                bg_color,
            );
            draw_rectangle_lines(
                building.position.x - w / 2.0,
                building.position.y - h / 2.0,
                w,
                h,
                2.0,
                border_color,
            );

            // Label
            let label = &building.building_type;
            let short = if label.len() > 8 { &label[..8] } else { label };
            let label_color = if unlocked {
                dark::TEXT_DIM
            } else {
                Color::new(0.35, 0.35, 0.35, 0.8)
            };
            let label_y = building.position.y + h / 2.0 + if is_core { 16.0 } else { 12.0 };
            draw_text(
                short,
                building.position.x - w / 2.0,
                label_y,
                if is_core { 12.0 } else { 10.0 },
                label_color,
            );

            if selected && unlocked {
                draw_rectangle_lines(
                    building.position.x - w / 2.0 - 2.0,
                    building.position.y - h / 2.0 - 2.0,
                    w + 4.0,
                    h + 4.0,
                    2.0,
                    WHITE,
                );
            }
        }

        // 7. Factory core
        let core = self.map_state.factory_core;
        let pulse = 0.6 + 0.4 * (get_time() as f32 * 3.0).sin().abs();
        draw_circle(core.x, core.y, 20.0, Color::new(0.1, 0.5, 0.2, pulse));
        draw_circle_lines(core.x, core.y, 22.0, 2.0, Color::new(0.3, 0.9, 0.4, 0.8));
        draw_text(
            "FACTORY",
            core.x - 25.0,
            core.y + 30.0,
            12.0,
            Color::new(0.3, 0.9, 0.4, 0.8),
        );
    }

    fn trace_node_pos(&self, node: &TraceNode) -> Vec2 {
        match node {
            TraceNode::FactoryCore => self.map_state.factory_core,
            TraceNode::Slot(idx) => self
                .map_state
                .slots
                .get(*idx)
                .map_or(Vec2::ZERO, |s| s.position),
            TraceNode::Building(idx) => self
                .map_state
                .buildings
                .get(*idx)
                .map_or(Vec2::ZERO, |b| b.position),
        }
    }

    fn draw_towers(&self) {
        let world_mouse = self.screen_to_world(vec2(mouse_position().0, mouse_position().1));
        let range_mult = if self.factory.is_sector_active("ai_vault") {
            1.2
        } else {
            1.0
        };
        for tower in &self.towers {
            let mut col = tower.color();
            if !tower.is_active {
                col = Color::new(col.r, col.g, col.b, 0.5);
            }
            let outline_alpha = if tower.is_active { 1.0 } else { 0.4 };
            let radius = self.constants.ui.tower_base_radius
                + (tower.level.saturating_sub(1) as f32) * self.constants.ui.tower_level_radius_inc;
            draw_circle(tower.position.x, tower.position.y, radius, col);
            let outline = Color::new(dark::TEXT.r, dark::TEXT.g, dark::TEXT.b, outline_alpha);
            draw_circle_lines(tower.position.x, tower.position.y, radius, 2.0, outline);

            if tower.level > 1 {
                let ring_radius = radius + self.constants.ui.tower_ring_offset;
                draw_circle_lines(tower.position.x, tower.position.y, ring_radius, 1.5, col);
            }

            let dist = (world_mouse - tower.position).length();
            if dist < self.constants.ui.tower_hover_dist {
                let range_alpha = if tower.is_active { 0.3 } else { 0.15 };
                draw_circle_lines(
                    tower.position.x,
                    tower.position.y,
                    tower.range * range_mult,
                    1.0,
                    Color::new(col.r, col.g, col.b, range_alpha),
                );
            }
        }
    }

    fn draw_shot_effects(&self) {
        for effect in &self.shot_effects {
            let alpha = effect.alpha();
            match effect {
                crate::engine::tower::ShotEffect::Line {
                    from, to, color, ..
                } => {
                    let col = Color::new(color.r, color.g, color.b, alpha);
                    draw_line(from.x, from.y, to.x, to.y, 2.0, col);
                }
                crate::engine::tower::ShotEffect::Pulse {
                    center,
                    radius,
                    color,
                    ..
                } => {
                    let col = Color::new(color.r, color.g, color.b, alpha);
                    draw_circle_lines(center.x, center.y, *radius, 2.0, col);
                }
            }
        }
    }

    fn draw_particles(&self) {
        for p in &self.particles {
            let alpha = (p.ttl / self.constants.particles.ttl).clamp(0.0, 1.0);
            let col = Color::new(1.0, 1.0, 1.0, alpha);
            draw_rectangle(
                p.position.x,
                p.position.y,
                self.constants.particles.size,
                self.constants.particles.size,
                col,
            );
        }
    }

    fn draw_enemies(&self) {
        for enemy in &self.wave_manager.enemies {
            if !enemy.is_alive {
                continue;
            }

            let (radius, color) = match enemy.enemy_type {
                EnemyType::Scout => (5.0, Color::new(0.4, 0.9, 0.4, 1.0)),
                EnemyType::Drone => (7.0, Color::new(0.9, 0.5, 0.2, 1.0)),
                EnemyType::HeavyUnit => (10.0, Color::new(0.8, 0.2, 0.2, 1.0)),
                EnemyType::Saboteur => (6.0, Color::new(0.7, 0.3, 0.9, 1.0)),
                EnemyType::Commander => (14.0, Color::new(1.0, 0.8, 0.0, 1.0)),
            };

            let mut col = color;
            if enemy.hit_flash_timer > 0.0 {
                col = Color::new(1.0, 1.0, 1.0, 1.0);
            } else if enemy.dodge_timer > 0.0 {
                col = Color::new(color.r, color.g, color.b, 0.4);
            }
            draw_circle(enemy.position.x, enemy.position.y, radius, col);

            let bar_w = radius * 3.0;
            let bar_h = 3.0;
            let bar_x = enemy.position.x - bar_w / 2.0;
            let bar_y = enemy.position.y - radius - 6.0;
            let fill = enemy.health / enemy.max_health;
            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.3, 0.0, 0.0, 0.8));
            draw_rectangle(bar_x, bar_y, bar_w * fill, bar_h, dark::POSITIVE);
        }
    }

    fn draw_hud(&self) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            self.constants.ui.hud_height,
            Color::new(0.08, 0.08, 0.1, 0.95),
        );

        let y = 8.0;
        let font_size = 16.0;
        let mut x = self.constants.ui.build_panel_w + 10.0;
        let spacing = 140.0;

        let gen = self.factory.power_generation() + self.unlocked_building_boon().power_per_sec;
        let tower_drain: f32 = self
            .towers
            .iter()
            .filter(|t| t.is_active)
            .map(|t| t.power_drain)
            .sum();
        let net_rate = gen - self.factory.power_consumption() - tower_drain;
        let power_text = if net_rate >= 0.0 {
            format!("Power: {:.0} (+{:.0}/s)", self.resources.power, net_rate)
        } else {
            format!("Power: {:.0} ({:.0}/s)", self.resources.power, net_rate)
        };
        let power_color = if net_rate >= 0.0 {
            dark::POSITIVE
        } else {
            dark::NEGATIVE
        };

        let threat_text = format!(
            "Threat: {} ({:.0})",
            self.threat.reaction_tier().label(),
            self.threat.awareness_level()
        );
        let mut threat_col = super::helpers::threat_color(&self.threat);
        if self.threat_flash_timer > 0.0 {
            threat_col = Color::new(1.0, 0.85, 0.25, 1.0);
        }

        let beacon_text = format!(
            "Beacon: {} ({:.0})",
            self.beacon_phase.label(),
            self.beacon_strength
        );
        let pulse = 0.6 + 0.4 * (f32::sin(get_time() as f32 * 3.0).abs());
        let base_beacon_col = beacon_color(&self.beacon_phase);
        let beacon_col = Color::new(
            base_beacon_col.r,
            base_beacon_col.g,
            base_beacon_col.b,
            pulse,
        );

        let scavenger_text = format!("Scavengers: {} out", self.scavengers_out);

        let morale_color = if self.population.morale < 20.0 {
            dark::NEGATIVE
        } else if self.population.morale < 50.0 {
            dark::WARNING
        } else {
            dark::POSITIVE
        };
        let income_rate = self.population.productivity(&self.constants)
            * self.constants.economy.productivity_scrap_rate;

        let items = [
            (format!("Wave: {}", self.current_wave), dark::TEXT),
            (power_text, power_color),
            (format!("Scrap: {:.0}", self.resources.scrap), dark::WARNING),
            (format!("Pop: {}", self.population.count), dark::POSITIVE),
            (
                format!("Food: {:.0}", self.population.food_supply),
                dark::TEXT_DIM,
            ),
            (
                format!("Morale: {:.0}", self.population.morale),
                morale_color,
            ),
            (format!("Income: {:.1}/s", income_rate), dark::TEXT_DIM),
            (format!("{}", self.factory.phase.label()), dark::TEXT_BRIGHT),
            (beacon_text, beacon_col),
            (scavenger_text, dark::TEXT_DIM),
            (threat_text, threat_col),
        ];

        for (text, color) in &items {
            draw_text(text, x, y + font_size, font_size, *color);
            x += spacing;
        }

        let max_food = (self.population.count as f32 * self.constants.ui.food_bar_max_per_pop)
            .max(self.constants.ui.food_bar_min_max);
        let bar_w = self.constants.ui.food_bar_w;
        let bar_h = self.constants.ui.food_bar_h;
        let bar_x = screen_width() - bar_w - 10.0;
        let bar_y = 22.0;
        crate::ui::draw_resource_bar(
            bar_x,
            bar_y,
            bar_w,
            bar_h,
            self.population.food_supply,
            max_food,
            dark::POSITIVE,
        );
    }

    fn draw_wave_status(&self) {
        let text = if self.between_waves {
            format!("Next wave in {:.0}s...", self.wave_timer.max(0.0))
        } else {
            let alive = self.wave_manager.alive_count();
            let queued = self.wave_manager.spawn_queue.len();
            format!(
                "Wave {} - {} enemies ({} spawning)",
                self.current_wave, alive, queued
            )
        };

        let x = self.constants.ui.build_panel_w + 10.0;
        let y = self.constants.ui.hud_height + 5.0;
        draw_text(&text, x, y + 14.0, 14.0, dark::TEXT_DIM);

        if self.time_scale > 1.0 {
            let speed_text = "Speed: 2x";
            let dims = measure_text(speed_text, None, 12, 1.0);
            draw_text(
                speed_text,
                screen_width() - dims.width - self.constants.ui.sector_panel_w - 10.0,
                y + 14.0,
                12.0,
                dark::WARNING,
            );
        }

        if self.between_waves {
            let preview = self.build_wave_preview();
            if !preview.is_empty() {
                let list = preview
                    .iter()
                    .map(|(t, c)| format!("{}x{}", enemy_label(t), c))
                    .collect::<Vec<_>>()
                    .join("  ");
                draw_text(
                    &format!("Incoming: {}", list),
                    x,
                    y + 28.0,
                    12.0,
                    dark::TEXT_DIM,
                );
            }
        }
    }

    fn draw_wave_start_flash(&self) {
        if self.wave_flash_timer <= 0.0 {
            return;
        }
        let alpha = (self.wave_flash_timer / self.constants.ui.wave_flash_duration).clamp(0.0, 1.0);
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(1.0, 1.0, 1.0, self.constants.ui.wave_flash_alpha * alpha),
        );
        let text = if self.shutdown_triggered {
            "BEACON SHUTDOWN".to_string()
        } else {
            format!("WAVE {}", self.last_wave_started)
        };
        let dims = measure_text(
            &text,
            None,
            self.constants.ui.wave_flash_text_size as u16,
            1.0,
        );
        draw_text(
            &text,
            screen_width() / 2.0 - dims.width / 2.0,
            screen_height() * self.constants.ui.wave_flash_text_y_frac,
            self.constants.ui.wave_flash_text_size,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    fn draw_notifications(&self) {
        let config = NotificationRenderConfig {
            width: 340.0,
            row_height: 30.0,
            spacing: 4.0,
            padding: 8.0,
            font_size: 16.0,
            background: Color::new(0.08, 0.08, 0.1, 0.9),
            text_color: dark::TEXT_BRIGHT,
            ..Default::default()
        };
        let mut y = self.constants.ui.hud_height + 28.0;
        for note in &self.notifications {
            let notification = Notification {
                message: note.text.clone(),
                notification_type: NotificationType::Info,
                time_remaining: note.ttl,
                total_duration: self.constants.ui.notification_ttl,
            };
            draw_notification(
                &notification,
                self.constants.ui.build_panel_w + 10.0,
                y,
                &config,
            );
            y += config.row_height + config.spacing;
        }
    }

    fn draw_slot_panel(&mut self, _data: &GameData) {
        // Show context info at bottom-center for selected slot or building
        let panel_w = 300.0;
        let panel_h = 80.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = screen_height() - panel_h - 10.0;

        if self.selected_core {
            draw_rectangle(
                panel_x,
                panel_y,
                panel_w,
                panel_h,
                Color::new(0.08, 0.08, 0.1, 0.9),
            );
            draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 1.0, dark::TEXT_DIM);

            let text_x = panel_x + 10.0;
            let mut text_y = panel_y + 18.0;
            draw_text("Factory Console", text_x, text_y, 14.0, dark::TEXT_BRIGHT);
            text_y += 16.0;
            draw_text("Upgrades available", text_x, text_y, 12.0, dark::TEXT_DIM);

            let upgrades: Vec<_> = self.available_upgrades().into_iter().collect();
            let mut upgrade_click: Option<String> = None;
            let mut upgrade_purchase: Option<String> = None;

            let mut row_y = panel_y - 40.0;
            for upg in &upgrades {
                row_y -= 36.0;
                if row_y < panel_y - 160.0 {
                    break;
                }
                let purchased = self.factory.has_upgrade(&upg.id);
                let selected = self.selected_upgrade.as_deref() == Some(&upg.id);
                let row_x = panel_x - 10.0;
                let row_w = panel_w + 20.0;
                let row_h = 32.0;
                let bg = if selected {
                    Color::new(0.12, 0.14, 0.2, 0.95)
                } else if purchased {
                    Color::new(0.12, 0.18, 0.12, 0.9)
                } else {
                    Color::new(0.1, 0.1, 0.12, 0.85)
                };
                draw_rectangle(row_x, row_y, row_w, row_h, bg);
                draw_rectangle_lines(row_x, row_y, row_w, row_h, 1.0, dark::TEXT_DIM);

                let name = if purchased {
                    format!("[x] {}", upg.name)
                } else {
                    upg.name.clone()
                };
                draw_text(&name, row_x + 8.0, row_y + 14.0, 12.0, dark::TEXT);
                draw_text(
                    &upg.description,
                    row_x + 8.0,
                    row_y + 26.0,
                    10.0,
                    dark::TEXT_DIM,
                );

                let (mx, my) = mouse_position();
                if mx >= row_x && mx <= row_x + row_w && my >= row_y && my <= row_y + row_h {
                    if is_mouse_button_pressed(MouseButton::Left) {
                        upgrade_click = Some(upg.id.clone());
                    }
                }
            }

            if let Some(selected_id) = self.selected_upgrade.clone() {
                let selected = self.upgrade_defs.iter().find(|u| u.id == selected_id);
                if let Some(selected) = selected {
                    let can_afford = self.factory.can_purchase(
                        selected,
                        self.resources.scrap,
                        self.resources.power,
                    );
                    let purchased = self.factory.has_upgrade(&selected.id);
                    let detail_x = panel_x + 10.0;
                    let mut detail_y = panel_y + 40.0;
                    draw_text(&selected.name, detail_x, detail_y, 14.0, dark::TEXT_BRIGHT);
                    detail_y += 16.0;
                    draw_text(
                        &selected.description,
                        detail_x,
                        detail_y,
                        12.0,
                        dark::TEXT_DIM,
                    );
                    detail_y += 14.0;
                    draw_text(
                        &format!(
                            "Cost: {}s / {}p",
                            selected.cost_scrap as i32, selected.cost_power as i32
                        ),
                        detail_x,
                        detail_y,
                        12.0,
                        dark::WARNING,
                    );
                    detail_y += 14.0;
                    draw_text(
                        &format!("Difficulty: +{:.0}", selected.difficulty_cost),
                        detail_x,
                        detail_y,
                        12.0,
                        dark::NEGATIVE,
                    );

                    if !purchased {
                        let btn_w = 70.0;
                        let btn_h = 22.0;
                        let btn_x = panel_x + panel_w - btn_w - 10.0;
                        let btn_y = panel_y + panel_h - btn_h - 8.0;
                        if can_afford {
                            if button(btn_x, btn_y, btn_w, btn_h, "Unlock") {
                                upgrade_purchase = Some(selected.id.clone());
                            }
                        } else {
                            draw_text(
                                "Need resources",
                                btn_x - 10.0,
                                btn_y - 6.0,
                                10.0,
                                dark::TEXT_DIM,
                            );
                        }
                    }
                }
            }

            if let Some(id) = upgrade_click {
                self.selected_upgrade = Some(id);
            }
            if let Some(id) = upgrade_purchase {
                self.purchase_upgrade(&id);
            }
        } else if let Some(idx) = self.selected_slot {
            if let Some(slot) = self.map_state.slots.get(idx) {
                draw_rectangle(
                    panel_x,
                    panel_y,
                    panel_w,
                    panel_h,
                    Color::new(0.08, 0.08, 0.1, 0.9),
                );
                draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 1.0, dark::TEXT_DIM);

                let text_x = panel_x + 10.0;
                let mut text_y = panel_y + 18.0;

                draw_text(
                    &format!("Slot: {}", slot.id),
                    text_x,
                    text_y,
                    14.0,
                    dark::TEXT_BRIGHT,
                );
                text_y += 16.0;

                match slot.state {
                    SlotState::Debris => {
                        let hint = if slot.opens_entrance.is_some() {
                            " [Opens path!]"
                        } else {
                            ""
                        };
                        draw_text(
                            &format!("Debris - Clear: {:.0} scrap{}", slot.clear_cost, hint),
                            text_x,
                            text_y,
                            12.0,
                            dark::WARNING,
                        );
                        text_y += 14.0;
                        let btn_label = format!("Clear ({:.0})", slot.clear_cost);
                        let btn_w = 110.0;
                        let btn_h = 22.0;
                        let btn_x = panel_x + panel_w - btn_w - 10.0;
                        let btn_y = panel_y + panel_h - btn_h - 8.0;
                        if self.resources.scrap >= slot.clear_cost {
                            if button(btn_x, btn_y, btn_w, btn_h, &btn_label) {
                                self.clear_slot(idx);
                            }
                        } else {
                            draw_text("Insufficient scrap", text_x, text_y, 11.0, dark::TEXT_DIM);
                        }
                    }
                    SlotState::Cleared => {
                        draw_text(
                            &format!("Cleared - Power: {:.0} scrap", slot.power_cost),
                            text_x,
                            text_y,
                            12.0,
                            dark::ACCENT,
                        );
                        text_y += 14.0;
                        let btn_label = format!("Power ({:.0})", slot.power_cost);
                        let btn_w = 110.0;
                        let btn_h = 22.0;
                        let btn_x = panel_x + panel_w - btn_w - 10.0;
                        let btn_y = panel_y + panel_h - btn_h - 8.0;
                        if self.resources.scrap >= slot.power_cost {
                            if button(btn_x, btn_y, btn_w, btn_h, &btn_label) {
                                self.power_slot(idx);
                            }
                        } else {
                            draw_text("Insufficient scrap", text_x, text_y, 11.0, dark::TEXT_DIM);
                        }
                    }
                    SlotState::Powered => {
                        if slot.tower_index.is_some() {
                            draw_text("Tower placed", text_x, text_y, 12.0, dark::POSITIVE);
                        } else if self.placing_tower.is_some() {
                            draw_text(
                                "Powered - Click to place tower here",
                                text_x,
                                text_y,
                                12.0,
                                dark::POSITIVE,
                            );
                        } else {
                            draw_text(
                                "Powered - Select tower from Build panel",
                                text_x,
                                text_y,
                                12.0,
                                dark::POSITIVE,
                            );
                        }
                    }
                }
            }
        } else if let Some(idx) = self.selected_building {
            if let Some(building) = self.map_state.buildings.get(idx) {
                draw_rectangle(
                    panel_x,
                    panel_y,
                    panel_w,
                    panel_h,
                    Color::new(0.08, 0.08, 0.1, 0.9),
                );
                draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 1.0, dark::TEXT_DIM);

                let text_x = panel_x + 10.0;
                let mut text_y = panel_y + 18.0;

                draw_text(
                    &format!("{} ({})", building.id, building.building_type),
                    text_x,
                    text_y,
                    14.0,
                    dark::TEXT_BRIGHT,
                );
                text_y += 16.0;

                match building.state {
                    BuildingState::Broken => {
                        draw_text(
                            &format!("Broken - Repair: {:.0} scrap", building.repair_cost),
                            text_x,
                            text_y,
                            12.0,
                            dark::NEGATIVE,
                        );
                        let btn_label = format!("Repair ({:.0})", building.repair_cost);
                        let btn_w = 110.0;
                        let btn_h = 22.0;
                        let btn_x = panel_x + panel_w - btn_w - 10.0;
                        let btn_y = panel_y + panel_h - btn_h - 8.0;
                        if self.resources.scrap >= building.repair_cost {
                            if button(btn_x, btn_y, btn_w, btn_h, &btn_label) {
                                self.repair_building(idx);
                            }
                        }
                    }
                    BuildingState::Repaired => {
                        draw_text(
                            &format!("Repaired - Power: {:.0} scrap", building.power_cost),
                            text_x,
                            text_y,
                            12.0,
                            dark::WARNING,
                        );
                        let btn_label = format!("Power ({:.0})", building.power_cost);
                        let btn_w = 110.0;
                        let btn_h = 22.0;
                        let btn_x = panel_x + panel_w - btn_w - 10.0;
                        let btn_y = panel_y + panel_h - btn_h - 8.0;
                        if self.resources.scrap >= building.power_cost {
                            if button(btn_x, btn_y, btn_w, btn_h, &btn_label) {
                                self.power_building(idx);
                            }
                        }
                    }
                    BuildingState::Powered => {
                        let b = &building.boon;
                        let mut parts = Vec::new();
                        if b.scrap_per_sec > 0.0 {
                            parts.push(format!("+{:.1} scrap/s", b.scrap_per_sec));
                        }
                        if b.food_per_sec > 0.0 {
                            parts.push(format!("+{:.1} food/s", b.food_per_sec));
                        }
                        if b.water_per_sec > 0.0 {
                            parts.push(format!("+{:.1} water/s", b.water_per_sec));
                        }
                        if b.power_per_sec > 0.0 {
                            parts.push(format!("+{:.1} power/s", b.power_per_sec));
                        }
                        draw_text(
                            &format!("Active: {}", parts.join(", ")),
                            text_x,
                            text_y,
                            12.0,
                            dark::POSITIVE,
                        );
                        text_y += 14.0;
                        draw_text(
                            &format!("Threat: +{:.2}/s", building.threat_per_sec),
                            text_x,
                            text_y,
                            11.0,
                            dark::NEGATIVE,
                        );
                    }
                    BuildingState::Disabled => {
                        draw_text("Disabled", text_x, text_y, 12.0, dark::TEXT_DIM);
                    }
                }
            }
        }
    }

    fn draw_hover_tooltip(&self) {
        let (mx, my) = mouse_position();
        let world_mouse = self.screen_to_world(vec2(mx, my));

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
        let w = self.constants.ui.tooltip_w;
        let h = self.constants.ui.tooltip_h;
        let x = (mx + 12.0).clamp(10.0, screen_width() - w - 10.0);
        let y = (my + 12.0).clamp(10.0, screen_height() - h - 10.0);
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.08, 0.1, 0.9));
        draw_rectangle_lines(x, y, w, h, 1.0, dark::TEXT_DIM);
        draw_text(&text, x + 8.0, y + 16.0, 12.0, dark::TEXT);
    }
}
