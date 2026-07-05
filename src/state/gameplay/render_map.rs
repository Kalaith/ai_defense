use crate::engine::enemy::EnemyType;
use crate::engine::map::{BuildingState, MapPath, SlotState, TraceNode};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use super::helpers::entrance_label;
use super::GameplayState;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl GameplayState {
    pub(super) fn draw_circuit_board(&self) {
        let map_w = self.map_state.map_size.x;
        let map_h = self.map_state.map_size.y;
        let world_mouse = self.screen_to_world(vec2(mouse_position().0, mouse_position().1));
        let hovered_slot = self.map_state.nearest_slot(world_mouse).map(|(idx, _)| idx);
        let hovered_building = self
            .map_state
            .nearest_building(world_mouse)
            .map(|(idx, _)| idx);

        // 1. PCB background
        draw_rectangle(0.0, 0.0, map_w, map_h, Color::new(0.015, 0.045, 0.04, 1.0));
        for x in (0..map_w as i32).step_by(160) {
            draw_line(
                x as f32,
                0.0,
                x as f32,
                map_h,
                1.0,
                Color::new(0.04, 0.12, 0.1, 0.16),
            );
        }
        for y in (0..map_h as i32).step_by(160) {
            draw_line(
                0.0,
                y as f32,
                map_w,
                y as f32,
                1.0,
                Color::new(0.04, 0.12, 0.1, 0.16),
            );
        }

        // 1b. Section backplates + corridor connectors
        //
        // Hidden sections used to draw their name + "UNPOWERED" tag, spoiling
        // the layout and cluttering the map with wings the player can't touch
        // yet. Now only the *next* locked section renders — as an anonymous
        // teaser — and everything further out is simply absent until revealed.
        let sections = self.map_state.section_render_info();
        let next_hidden = sections
            .iter()
            .filter(|s| !s.visible)
            .map(|s| s.index)
            .min();
        // Section name tags are orientation aids for the zoomed-out view; when
        // the player zooms in to work on pads they become oversized text lying
        // across the play area, so fade them out with zoom.
        let label_fade = ((1.35 - self.camera.zoom) / 0.5).clamp(0.0, 1.0);
        let mut section_centers: Vec<(Vec2, bool)> = Vec::new();
        for section in &sections {
            if !section.visible && Some(section.index) != next_hidden {
                continue;
            }
            let pad_x = 80.0 + (section.index as f32 * 6.0);
            let pad_y = 70.0 + (section.index as f32 * 4.0);
            let min = vec2(section.min.x - pad_x, section.min.y - pad_y);
            let max = vec2(section.max.x + pad_x, section.max.y + pad_y);
            let w = (max.x - min.x).max(120.0);
            let h = (max.y - min.y).max(80.0);
            let base = 0.05 + (section.index as f32 * 0.02).min(0.28);
            let fill = if section.visible {
                Color::new(0.025 + base, 0.08 + base * 0.5, 0.06 + base * 0.25, 0.68)
            } else {
                // The locked teaser must sit *below* revealed content in
                // visual weight — barely-there outline, not a bright slab.
                Color::new(0.02, 0.035, 0.045, 0.14)
            };
            let border = if section.visible {
                Color::new(0.12 + base, 0.32 + base, 0.22 + base * 0.45, 0.82)
            } else {
                Color::new(0.17, 0.28, 0.32, 0.22)
            };

            // Uniform, quiet backplate for every section. The old per-theme
            // silhouettes (a giant circle for water, bands, nested boxes…)
            // read as gameplay elements — the water circle in particular
            // looked exactly like a tower range ring.
            draw_rectangle(min.x, min.y, w, h, fill);
            draw_rectangle_lines(min.x, min.y, w, h, 2.0, border);

            if label_fade > 0.01 {
                if section.visible {
                    draw_ui_text(
                        &section.label,
                        min.x + 14.0,
                        min.y + 20.0,
                        13.0,
                        Color::new(0.48, 0.68, 0.62, 0.36 * label_fade),
                    );
                    if let Some(ref entrance) = section.unlock_entrance {
                        draw_ui_text(
                            &format!("Can open {}", entrance_label(entrance)),
                            min.x + 14.0,
                            min.y + 38.0,
                            11.0,
                            Color::new(0.95, 0.7, 0.24, 0.72 * label_fade),
                        );
                    }
                } else {
                    // The single next-locked teaser: no name spoiler, just a
                    // hint that powering the frontier reveals it.
                    draw_ui_text(
                        "LOCKED SECTION",
                        min.x + 14.0,
                        min.y + 20.0,
                        12.0,
                        Color::new(0.30, 0.42, 0.46, 0.34 * label_fade),
                    );
                    draw_ui_text(
                        "Power the frontier to reveal",
                        min.x + 14.0,
                        min.y + 38.0,
                        10.0,
                        Color::new(0.35, 0.52, 0.58, 0.4 * label_fade),
                    );
                }
            }

            let center = vec2((min.x + max.x) * 0.5, (min.y + max.y) * 0.5);
            section_centers.push((center, section.visible));
        }

        for i in 0..section_centers.len().saturating_sub(1) {
            let (a, a_visible) = section_centers[i];
            let (b, b_visible) = section_centers[i + 1];
            // Only link sections the player has actually revealed. A corridor
            // stroke to the locked teaser cut a meaningless diagonal across
            // the play area.
            if !a_visible || !b_visible {
                continue;
            }
            draw_line(a.x, a.y, b.x, b.y, 10.0, Color::new(0.02, 0.28, 0.18, 0.44));
            draw_line(a.x, a.y, b.x, b.y, 4.0, Color::new(0.18, 0.58, 0.36, 0.72));
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
                    .is_some_and(|s| self.map_state.is_slot_visible(s)),
                TraceNode::Building(idx) => self
                    .map_state
                    .buildings
                    .get(idx)
                    .is_some_and(|b| self.map_state.is_building_visible(b)),
                TraceNode::FactoryCore => true,
            };
            let to_visible = match trace.to {
                TraceNode::Slot(idx) => self
                    .map_state
                    .slots
                    .get(idx)
                    .is_some_and(|s| self.map_state.is_slot_visible(s)),
                TraceNode::Building(idx) => self
                    .map_state
                    .buildings
                    .get(idx)
                    .is_some_and(|b| self.map_state.is_building_visible(b)),
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

            // Route point-to-point traces as L-shaped circuit runs (horizontal
            // then vertical) instead of bent diagonals — the old synthetic
            // curve produced bright stray strokes across the play area that
            // looked like they meant something.
            let mut render_points = points.clone();
            if render_points.len() == 2 {
                let a = render_points[0];
                let b = render_points[1];
                if (a.x - b.x).abs() > 4.0 && (a.y - b.y).abs() > 4.0 {
                    render_points.insert(1, vec2(b.x, a.y));
                }
            }

            for i in 0..render_points.len().saturating_sub(1) {
                let a = render_points[i];
                let b = render_points[i + 1];
                if powered {
                    draw_line(a.x, a.y, b.x, b.y, 4.5, Color::new(0.0, 0.6, 0.2, 0.10));
                    draw_line(a.x, a.y, b.x, b.y, 1.8, Color::new(0.2, 1.0, 0.4, 0.42));
                } else {
                    draw_line(a.x, a.y, b.x, b.y, 1.6, Color::new(0.08, 0.2, 0.1, 0.28));
                }
            }
        }

        // 3. Enemy paths
        //
        // Locked routes are hidden by default — a second dim path on screen
        // just confused new players. They appear only as a warning preview
        // while the player has selected the pad/machine that would open them.
        let previewed_entrance: Option<String> = self
            .selected_slot
            .and_then(|i| self.map_state.slots.get(i))
            .and_then(|s| s.opens_entrance.clone())
            .or_else(|| {
                self.selected_building
                    .and_then(|i| self.map_state.buildings.get(i))
                    .and_then(|b| b.opens_entrance.clone())
            });
        let previews_path = |path: &crate::engine::map::MapPath| {
            previewed_entrance
                .as_deref()
                .is_some_and(|entrance| path.requires_entrance.as_deref() == Some(entrance))
        };
        // Animate route flow only while machines are actually in the field;
        // idle routes with moving dots read as enemies that aren't there.
        let route_live =
            self.wave_manager.alive_count() > 0 || !self.wave_manager.spawn_queue.is_empty();

        let max_x = self.map_state.max_visible_x();
        for path in &self.map_state.paths {
            if !path.active && !previews_path(path) {
                continue;
            }
            let reveal_limit = if path.active {
                max_x + 80.0
            } else {
                max_x + 220.0
            };
            if !path.active
                && !path
                    .points
                    .iter()
                    .any(|p| p.x <= reveal_limit && p.x >= -120.0)
            {
                continue;
            }
            draw_enemy_route(path, reveal_limit, route_live);
        }

        // 4. Entrance markers
        for path in &self.map_state.paths {
            if !path.active && !previews_path(path) {
                continue;
            }
            let reveal_limit = if path.active {
                max_x + 80.0
            } else {
                max_x + 220.0
            };
            if path.entrance.x > reveal_limit {
                continue;
            }

            let e = path.entrance;
            if path.active {
                let pulse = 0.55 + 0.3 * (get_time() as f32 * 4.0).sin().abs();
                draw_circle(e.x, e.y, 17.0, Color::new(0.92, 0.18, 0.08, 0.88));
                draw_circle_lines(e.x, e.y, 24.0, 3.0, Color::new(1.0, 0.46, 0.12, pulse));
                draw_ui_text(
                    entrance_label(&path.id),
                    e.x + 24.0,
                    e.y + 5.0,
                    13.0,
                    Color::new(1.0, 0.62, 0.24, 0.82),
                );
            } else {
                // Only drawn while the player is inspecting the pad/machine
                // that would open this route — a warning preview.
                draw_circle(e.x, e.y, 10.0, Color::new(0.45, 0.08, 0.08, 0.42));
                draw_circle_lines(e.x, e.y, 16.0, 2.0, Color::new(0.85, 0.24, 0.14, 0.44));
                // Entrances can sit on the map's edge (e.g. the northwest
                // breach at the top border); clamp the tag inward so it isn't
                // sliced off at the world boundary.
                let label_y = (e.y + 4.0).max(26.0);
                draw_ui_text(
                    "OPENS THIS ROUTE",
                    e.x + 18.0,
                    label_y,
                    10.0,
                    Color::new(0.95, 0.42, 0.26, 0.6),
                );
            }
        }

        // 5. Tower slots
        for (idx, slot) in self.map_state.slots.iter().enumerate() {
            // Slots outside any section (map.json gaps) pass is_slot_visible
            // unconditionally, so also gate on the revealed frontier — same
            // rule the buildings below already use.
            if !self.map_state.is_slot_visible(slot) || slot.position.x > max_x + 120.0 {
                continue;
            }
            let selected = self.selected_slot == Some(idx);
            let hovered = hovered_slot == Some(idx);
            let pad = slot.position;
            match slot.state {
                SlotState::Debris => {
                    draw_circle(pad.x, pad.y, 13.0, Color::new(0.28, 0.17, 0.08, 0.86));
                    draw_circle_lines(pad.x, pad.y, 15.0, 2.4, Color::new(0.85, 0.52, 0.16, 0.78));
                    let s = 6.0;
                    draw_line(
                        pad.x - s,
                        pad.y - s,
                        pad.x + s,
                        pad.y + s,
                        2.0,
                        Color::new(1.0, 0.66, 0.2, 0.68),
                    );
                    draw_line(
                        pad.x + s,
                        pad.y - s,
                        pad.x - s,
                        pad.y + s,
                        2.0,
                        Color::new(1.0, 0.66, 0.2, 0.68),
                    );
                    if slot.opens_entrance.is_some() {
                        draw_circle_lines(
                            pad.x,
                            pad.y,
                            18.0,
                            2.0,
                            Color::new(1.0, 0.76, 0.18, 0.62),
                        );
                    }
                }
                SlotState::Cleared => {
                    draw_circle(pad.x, pad.y, 13.0, Color::new(0.05, 0.12, 0.2, 0.75));
                    draw_circle_lines(pad.x, pad.y, 16.0, 2.4, Color::new(0.32, 0.68, 1.0, 0.82));
                    draw_circle(pad.x, pad.y, 4.0, Color::new(0.45, 0.82, 1.0, 0.7));
                }
                SlotState::Powered => {
                    if slot.tower_index.is_some() {
                        draw_circle(pad.x, pad.y, 14.0, Color::new(0.04, 0.18, 0.09, 0.62));
                        draw_circle_lines(
                            pad.x,
                            pad.y,
                            17.0,
                            2.0,
                            Color::new(0.24, 0.92, 0.45, 0.55),
                        );
                    } else {
                        let pulse = 0.5 + 0.3 * (get_time() as f32 * 2.0).sin().abs();
                        draw_circle(pad.x, pad.y, 14.0, Color::new(0.04, 0.2, 0.08, 0.9));
                        draw_circle_lines(
                            pad.x,
                            pad.y,
                            18.0,
                            2.8,
                            Color::new(0.25, 1.0, 0.45, pulse),
                        );
                        draw_line(
                            pad.x - 5.0,
                            pad.y,
                            pad.x + 5.0,
                            pad.y,
                            2.0,
                            Color::new(0.5, 1.0, 0.6, 0.8),
                        );
                        draw_line(
                            pad.x,
                            pad.y - 5.0,
                            pad.x,
                            pad.y + 5.0,
                            2.0,
                            Color::new(0.5, 1.0, 0.6, 0.8),
                        );
                    }
                }
            }

            if hovered && !selected {
                draw_circle_lines(pad.x, pad.y, 23.0, 2.2, Color::new(0.35, 0.85, 1.0, 0.82));
            }
            if selected {
                draw_circle_lines(pad.x, pad.y, 26.0, 3.2, Color::new(0.45, 0.9, 1.0, 1.0));
                draw_label_tag(
                    &slot.id.to_uppercase(),
                    pad + vec2(20.0, -22.0),
                    Color::new(0.42, 0.82, 1.0, 0.96),
                );
            }
        }

        // 6. Buildings
        for (idx, building) in self.map_state.buildings.iter().enumerate() {
            if !self.map_state.is_building_visible(building) || building.position.x > max_x + 120.0
            {
                continue;
            }
            let unlocked = self.is_building_unlocked(building);
            let selected = self.selected_building == Some(idx);
            let hovered = hovered_building == Some(idx);
            let (bg_color, border_color, state_text, state_color) = if !unlocked {
                (
                    Color::new(0.06, 0.07, 0.08, 0.7),
                    Color::new(0.24, 0.28, 0.3, 0.55),
                    "LOCKED",
                    Color::new(0.45, 0.5, 0.52, 0.72),
                )
            } else {
                match building.state {
                    BuildingState::Broken => (
                        Color::new(0.28, 0.05, 0.05, 0.9),
                        Color::new(0.78, 0.18, 0.12, 0.86),
                        "REPAIR",
                        Color::new(1.0, 0.42, 0.28, 0.88),
                    ),
                    BuildingState::Repaired => (
                        Color::new(0.25, 0.22, 0.04, 0.9),
                        Color::new(0.86, 0.74, 0.16, 0.9),
                        "POWER",
                        Color::new(1.0, 0.78, 0.18, 0.9),
                    ),
                    BuildingState::Powered => (
                        Color::new(0.03, 0.22, 0.2, 0.94),
                        Color::new(0.22, 0.92, 0.78, 0.95),
                        "ONLINE",
                        Color::new(0.32, 1.0, 0.72, 0.95),
                    ),
                    BuildingState::Disabled => (
                        Color::new(0.15, 0.15, 0.15, 0.6),
                        Color::new(0.3, 0.3, 0.3, 0.6),
                        "OFFLINE",
                        dark::TEXT_DIM,
                    ),
                }
            };

            let is_core = self.map_state.is_core_building(&building.id);
            let w = if is_core { 84.0 } else { 62.0 };
            let h = if is_core { 50.0 } else { 38.0 };
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

            let text_col = if selected || hovered {
                dark::TEXT_BRIGHT
            } else {
                Color::new(0.72, 0.78, 0.76, 0.54)
            };
            draw_centered_text(
                building_node_label(&building.building_type),
                building.position.x,
                building.position.y + 4.0,
                if is_core { 14.0 } else { 12.0 },
                text_col,
            );
            draw_centered_text(
                state_text,
                building.position.x,
                building.position.y + h / 2.0 + 13.0,
                10.0,
                state_color,
            );
            if building.opens_entrance.is_some() && building.state != BuildingState::Powered {
                draw_rectangle_lines(
                    building.position.x - w / 2.0 - 5.0,
                    building.position.y - h / 2.0 - 5.0,
                    w + 10.0,
                    h + 10.0,
                    2.0,
                    Color::new(1.0, 0.58, 0.16, 0.58),
                );
            }

            if hovered && !selected && unlocked {
                draw_rectangle_lines(
                    building.position.x - w / 2.0 - 5.0,
                    building.position.y - h / 2.0 - 5.0,
                    w + 10.0,
                    h + 10.0,
                    2.0,
                    Color::new(0.35, 0.85, 1.0, 0.74),
                );
            }

            if selected && unlocked {
                draw_rectangle_lines(
                    building.position.x - w / 2.0 - 7.0,
                    building.position.y - h / 2.0 - 7.0,
                    w + 14.0,
                    h + 14.0,
                    3.0,
                    Color::new(0.45, 0.9, 1.0, 1.0),
                );
                draw_label_tag(
                    building_node_label(&building.building_type),
                    building.position + vec2(w * 0.5 + 8.0, -h * 0.5 - 4.0),
                    Color::new(0.42, 0.82, 1.0, 0.96),
                );
            }
        }

        // 7. Factory core
        let core = self.map_state.factory_core;
        let pulse = 0.6 + 0.4 * (get_time() as f32 * 3.0).sin().abs();
        draw_circle(core.x, core.y, 20.0, Color::new(0.1, 0.5, 0.2, pulse));
        draw_circle_lines(core.x, core.y, 22.0, 2.0, Color::new(0.3, 0.9, 0.4, 0.8));
        draw_ui_text(
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

    pub(super) fn draw_towers(&self) {
        for (idx, tower) in self.towers.iter().enumerate() {
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

            if self.selected_tower == Some(idx) {
                let range_alpha = if tower.is_active { 0.3 } else { 0.15 };
                draw_circle_lines(
                    tower.position.x,
                    tower.position.y,
                    radius + 8.0,
                    3.0,
                    Color::new(0.45, 0.9, 1.0, 1.0),
                );
                draw_circle_lines(
                    tower.position.x,
                    tower.position.y,
                    self.effective_tower_range(tower.range),
                    2.0,
                    Color::new(col.r, col.g, col.b, range_alpha),
                );
                draw_label_tag(
                    "SELECTED TOWER",
                    tower.position + vec2(20.0, -24.0),
                    Color::new(0.45, 0.9, 1.0, 1.0),
                );
            }
        }
    }

    pub(super) fn draw_shot_effects(&self) {
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

    pub(super) fn draw_particles(&self) {
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

    pub(super) fn draw_enemies(&self) {
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
}

fn draw_enemy_route(path: &MapPath, reveal_limit: f32, live: bool) {
    let active = path.active;
    let outer = if active {
        Color::new(0.52, 0.08, 0.03, 0.42)
    } else {
        Color::new(0.26, 0.07, 0.05, 0.18)
    };
    let inner = if active {
        Color::new(1.0, 0.56, 0.12, 0.96)
    } else {
        Color::new(0.78, 0.22, 0.14, 0.28)
    };
    let outer_width = if active { 16.0 } else { 8.0 };
    let inner_width = if active { 6.5 } else { 3.0 };

    for i in 0..path.points.len().saturating_sub(1) {
        let a = path.points[i];
        let b = path.points[i + 1];
        if a.x > reveal_limit && b.x > reveal_limit {
            break;
        }
        draw_line(a.x, a.y, b.x, b.y, outer_width, outer);
        draw_line(a.x, a.y, b.x, b.y, inner_width, inner);

        if active {
            // Moving flow dots only while a wave is actually in the field —
            // on an idle route they read as enemies that don't exist. Static
            // chevrons always show the direction of attack.
            if live {
                draw_route_flow(a, b, reveal_limit);
            }
            draw_route_arrows(a, b, reveal_limit);
        } else {
            let mid = (a + b) * 0.5;
            if mid.x <= reveal_limit {
                draw_circle(mid.x, mid.y, 3.0, Color::new(0.9, 0.25, 0.12, 0.28));
            }
        }
    }
}

fn draw_route_flow(a: Vec2, b: Vec2, reveal_limit: f32) {
    let segment = b - a;
    let len = segment.length();
    if len <= 1.0 {
        return;
    }

    let dir = segment / len;
    let steps = (len / 70.0).ceil().max(1.0) as i32;
    for step in 0..steps {
        let phase = (get_time() as f32 * 0.45 + step as f32 * 0.31).fract();
        let pos = a + dir * (len * phase);
        if pos.x > reveal_limit {
            continue;
        }
        draw_circle(pos.x, pos.y, 3.6, Color::new(1.0, 0.78, 0.24, 0.84));
    }
}

fn draw_route_arrows(a: Vec2, b: Vec2, reveal_limit: f32) {
    let segment = b - a;
    let len = segment.length();
    if len <= 40.0 {
        return;
    }

    let dir = segment / len;
    let perp = vec2(-dir.y, dir.x);
    let steps = (len / 130.0).ceil().max(1.0) as i32;
    for step in 0..steps {
        let t = ((step as f32 + 0.5) / steps as f32).clamp(0.15, 0.9);
        let p = a + segment * t;
        if p.x > reveal_limit {
            continue;
        }
        let tip = p + dir * 13.0;
        let left = p - dir * 7.0 + perp * 7.0;
        let right = p - dir * 7.0 - perp * 7.0;
        draw_triangle(tip, left, right, Color::new(1.0, 0.78, 0.22, 0.88));
    }
}

fn building_node_label(kind: &str) -> &str {
    match kind {
        "scrap_converter" => "SCRAP",
        "water_reclaimer" => "WATER",
        "nutrient_vats" => "FOOD",
        "power_turbine" => "POWER",
        "power_distribution_spine" => "SPINE",
        "assembly_hall_core" => "ASSY",
        "logistics_hub_core" => "LOGI",
        "robotics_bay_core" => "ROBOT",
        "research_core" => "TECH",
        _ => "SYSTEM",
    }
}

fn draw_centered_text(text: &str, center_x: f32, baseline_y: f32, font_size: f32, color: Color) {
    let dims = measure_ui_text(text, None, font_size as u16, 1.0);
    draw_ui_text(
        text,
        center_x - dims.width * 0.5,
        baseline_y,
        font_size,
        color,
    );
}

fn draw_label_tag(text: &str, pos: Vec2, color: Color) {
    let font_size = 12.0;
    let dims = measure_ui_text(text, None, font_size as u16, 1.0);
    let w = dims.width + 16.0;
    let h = 20.0;
    draw_rectangle(pos.x, pos.y, w, h, Color::new(0.03, 0.05, 0.07, 0.88));
    draw_rectangle_lines(pos.x, pos.y, w, h, 1.5, color);
    draw_ui_text(text, pos.x + 8.0, pos.y + 14.0, font_size, color);
}
