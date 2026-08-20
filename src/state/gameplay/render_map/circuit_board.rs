//! The main circuit-board map render: PCB background, section backplates,
//! traces, enemy routes, tower slots, buildings, and the factory core.

use crate::data::strings::{fill, text};
use crate::engine::map::{BuildingState, MapPath, SlotState, TraceNode};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::super::assets::{draw_frame, machine_index, tile_rect, FLOOR_VARIANTS};
use super::super::GameplayState;
use super::draw_label_tag;

impl GameplayState {
    pub(super) fn draw_pcb_background(&self) {
        let map_w = self.map_state.map_size.x;
        let map_h = self.map_state.map_size.y;

        for y in (0..map_h as i32).step_by(64) {
            for x in (0..map_w as i32).step_by(64) {
                let variant =
                    FLOOR_VARIANTS[((x / 64 + y / 64 * 3) as usize) % FLOOR_VARIANTS.len()];
                draw_texture_ex(
                    &self
                        .assets
                        .as_ref()
                        .expect("rendering requires gameplay assets")
                        .tiles,
                    x as f32,
                    y as f32,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(64.0, 64.0)),
                        source: Some(tile_rect(variant)),
                        ..Default::default()
                    },
                );
            }
        }
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
    }

    /// Section backplates + corridor connectors.
    ///
    /// Hidden sections used to draw their name + "UNPOWERED" tag, spoiling
    /// the layout and cluttering the map with wings the player can't touch
    /// yet. Now only the *next* locked section renders — as an anonymous
    /// teaser — and everything further out is simply absent until revealed.
    pub(super) fn draw_sections_and_corridors(&self) {
        let sections = self.map_state.section_render_info();
        let mut visible = sections.iter().filter(|section| section.visible);
        let mut previous: Option<Rect> = None;
        for section in visible.by_ref() {
            let pad = 42.0;
            let rect = Rect::new(
                (section.min.x - pad).max(0.0),
                (section.min.y - pad).max(0.0),
                (section.max.x - section.min.x + pad * 2.0).max(80.0),
                (section.max.y - section.min.y + pad * 2.0).max(80.0),
            );
            let accent = depth_accent(section.depth);
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(accent.r, accent.g, accent.b, 0.045),
            );
            draw_rectangle_lines(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                1.5,
                Color::new(accent.r, accent.g, accent.b, 0.22),
            );
            draw_line(
                rect.x,
                rect.y,
                rect.x + rect.w.min(250.0),
                rect.y,
                3.0,
                Color::new(accent.r, accent.g, accent.b, 0.5),
            );

            let depth_name = text()
                .map
                .depth_names
                .get(section.depth.saturating_sub(1) as usize)
                .map(String::as_str)
                .unwrap_or("DEEP FACTORY");
            let label = fill(
                &text().map.depth_label,
                &[
                    ("n", &format!("{:02}", section.depth)),
                    ("name", depth_name),
                ],
            );
            draw_ui_text(
                &label,
                rect.x + 12.0,
                rect.y + 18.0,
                10.0,
                Color::new(accent.r, accent.g, accent.b, 0.78),
            );
            draw_ui_text(
                &section.label,
                rect.x + 12.0,
                rect.y + 34.0,
                14.0,
                Color::new(0.78, 0.86, 0.82, 0.48),
            );
            draw_depth_motif(rect, section.depth, accent);

            if let Some(previous) = previous {
                let a = vec2(
                    previous.x + previous.w + 34.0,
                    previous.y + previous.h * 0.5,
                );
                let b = vec2(
                    rect.x - 34.0,
                    (rect.y + rect.h * 0.5).clamp(0.0, self.map_state.map_size.y),
                );
                draw_line(
                    a.x,
                    a.y,
                    b.x,
                    b.y,
                    2.0,
                    Color::new(accent.r, accent.g, accent.b, 0.16),
                );
            }
            previous = Some(rect);
        }
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

    pub(super) fn draw_traces(&self) {
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
    }

    /// Enemy paths and their entrance markers. Locked routes are hidden by
    /// default — a second dim path on screen just confused new players. They
    /// appear only as a warning preview while the player has selected the
    /// pad/machine that would open them.
    pub(super) fn draw_paths_and_entrances(&self, max_x: f32) {
        let previewed_entrance: Option<String> = self
            .selected_slot
            .and_then(|i| self.map_state.slots.get(i))
            .and_then(|s| s.opens_entrance.clone())
            .or_else(|| {
                self.selected_building
                    .and_then(|i| self.map_state.buildings.get(i))
                    .and_then(|b| b.opens_entrance.clone())
            });
        let previews_path = |path: &MapPath| {
            previewed_entrance
                .as_deref()
                .is_some_and(|entrance| path.requires_entrance.as_deref() == Some(entrance))
        };
        // Animate route flow only while machines are actually in the field;
        // idle routes with moving dots read as enemies that aren't there.
        let route_live =
            self.wave_manager.alive_count() > 0 || !self.wave_manager.spawn_queue.is_empty();

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
    }

    pub(super) fn draw_tower_slots(&self, hovered_slot: Option<usize>, max_x: f32) {
        for (idx, slot) in self.map_state.slots.iter().enumerate() {
            // Slots outside any section (map.json gaps) pass is_slot_visible
            // unconditionally, so also gate on the revealed frontier — same
            // rule the buildings below already use.
            if !self.map_state.is_slot_visible(slot) || slot.position.x > max_x + 120.0 {
                continue;
            }
            let selected = self.selected_slot == Some(idx);
            let _hovered = hovered_slot == Some(idx);
            let pad = slot.position;
            draw_slot_pad(
                &self
                    .assets
                    .as_ref()
                    .expect("rendering requires gameplay assets")
                    .pads,
                pad,
                slot.state,
                slot.opens_entrance.is_some(),
                slot.tower_index.is_some(),
            );

            if selected {
                draw_label_tag(
                    &slot.id.to_uppercase(),
                    pad + vec2(20.0, -22.0),
                    Color::new(0.42, 0.82, 1.0, 0.96),
                );
            }
        }
    }

    pub(super) fn draw_map_buildings(&self, hovered_building: Option<usize>, max_x: f32) {
        for (idx, building) in self.map_state.buildings.iter().enumerate() {
            if !self.map_state.is_building_visible(building) || building.position.x > max_x + 120.0
            {
                continue;
            }
            let unlocked = self.is_building_unlocked(building);
            let selected = self.selected_building == Some(idx);
            let hovered = hovered_building == Some(idx);
            let (_bg_color, _border_color, state_text, state_color) =
                building_visual(unlocked, building.state);

            let is_core = self.map_state.is_core_building(&building.id);
            let w = if is_core { 84.0 } else { 62.0 };
            let h = if is_core { 50.0 } else { 38.0 };
            let frame = match building.state {
                BuildingState::Broken | BuildingState::Disabled => 0,
                BuildingState::Repaired => 1,
                BuildingState::Powered => 2,
            };
            let sprite_size = if is_core {
                vec2(108.0, 76.0)
            } else {
                vec2(76.0, 58.0)
            };
            draw_frame(
                &self
                    .assets
                    .as_ref()
                    .expect("rendering requires gameplay assets")
                    .machines[machine_index(&building.building_type)],
                frame,
                vec2(128.0, 96.0),
                building.position,
                sprite_size,
                if unlocked {
                    WHITE
                } else {
                    Color::new(0.45, 0.5, 0.52, 0.55)
                },
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
            if selected && unlocked {
                draw_label_tag(
                    building_node_label(&building.building_type),
                    building.position + vec2(w * 0.5 + 8.0, -h * 0.5 - 4.0),
                    Color::new(0.42, 0.82, 1.0, 0.96),
                );
            }
        }
    }

    pub(super) fn draw_factory_core(&self) {
        let core = self.map_state.factory_core;
        draw_frame(
            &self
                .assets
                .as_ref()
                .expect("rendering requires gameplay assets")
                .core,
            if self
                .map_state
                .buildings
                .iter()
                .any(|b| b.state == BuildingState::Powered)
            {
                2
            } else {
                0
            },
            vec2(128.0, 128.0),
            core,
            vec2(132.0, 104.0),
            WHITE,
        );
        draw_ui_text(
            &text().map.factory,
            core.x - 25.0,
            core.y + 30.0,
            12.0,
            Color::new(0.3, 0.9, 0.4, 0.8),
        );
    }
}

/// (background, border, status label, status colour) for a building node.
fn building_visual(unlocked: bool, state: BuildingState) -> (Color, Color, &'static str, Color) {
    if !unlocked {
        return (
            Color::new(0.06, 0.07, 0.08, 0.7),
            Color::new(0.24, 0.28, 0.3, 0.55),
            text().status.locked.as_str(),
            Color::new(0.45, 0.5, 0.52, 0.72),
        );
    }
    match state {
        BuildingState::Broken => (
            Color::new(0.28, 0.05, 0.05, 0.9),
            Color::new(0.78, 0.18, 0.12, 0.86),
            text().status.repair.as_str(),
            Color::new(1.0, 0.42, 0.28, 0.88),
        ),
        BuildingState::Repaired => (
            Color::new(0.25, 0.22, 0.04, 0.9),
            Color::new(0.86, 0.74, 0.16, 0.9),
            text().status.power.as_str(),
            Color::new(1.0, 0.78, 0.18, 0.9),
        ),
        BuildingState::Powered => (
            Color::new(0.03, 0.22, 0.2, 0.94),
            Color::new(0.22, 0.92, 0.78, 0.95),
            text().status.online.as_str(),
            Color::new(0.32, 1.0, 0.72, 0.95),
        ),
        BuildingState::Disabled => (
            Color::new(0.15, 0.15, 0.15, 0.6),
            Color::new(0.3, 0.3, 0.3, 0.6),
            text().status.offline.as_str(),
            dark::TEXT_DIM,
        ),
    }
}

fn draw_slot_pad(
    texture: &Texture2D,
    pad: Vec2,
    state: SlotState,
    opens_entrance: bool,
    has_tower: bool,
) {
    let frame = match state {
        SlotState::Debris => 0,
        SlotState::Cleared => 1,
        SlotState::Powered if has_tower => 3,
        SlotState::Powered => 2,
    };
    draw_frame(
        texture,
        frame,
        vec2(64.0, 64.0),
        pad,
        vec2(38.0, 38.0),
        WHITE,
    );
    match state {
        SlotState::Debris => {
            let _ = opens_entrance;
        }
        SlotState::Cleared => {}
        SlotState::Powered => {
            let _ = has_tower;
        }
    }
}

fn depth_accent(depth: u32) -> Color {
    match depth {
        1 => Color::new(0.24, 0.72, 0.62, 1.0),
        2 => Color::new(0.26, 0.64, 0.9, 1.0),
        3 => Color::new(0.72, 0.5, 0.92, 1.0),
        _ => Color::new(0.96, 0.42, 0.28, 1.0),
    }
}

/// Small schematic marks make the four depth bands feel like different
/// industrial environments while leaving the existing tiles and routes legible.
fn draw_depth_motif(rect: Rect, depth: u32, color: Color) {
    let alpha = 0.16;
    match depth {
        1 => {
            let mut x = rect.x + 26.0;
            while x < rect.x + rect.w - 20.0 {
                draw_line(
                    x,
                    rect.y + rect.h - 24.0,
                    x + 18.0,
                    rect.y + rect.h - 24.0,
                    2.0,
                    Color::new(color.r, color.g, color.b, alpha),
                );
                draw_circle(
                    x + 22.0,
                    rect.y + rect.h - 24.0,
                    3.0,
                    Color::new(color.r, color.g, color.b, alpha + 0.1),
                );
                x += 54.0;
            }
        }
        2 => {
            let mut y = rect.y + 54.0;
            while y < rect.y + rect.h - 18.0 {
                draw_line(
                    rect.x + rect.w - 54.0,
                    y,
                    rect.x + rect.w - 24.0,
                    y,
                    3.0,
                    Color::new(color.r, color.g, color.b, alpha),
                );
                draw_line(
                    rect.x + rect.w - 42.0,
                    y - 8.0,
                    rect.x + rect.w - 42.0,
                    y + 8.0,
                    1.0,
                    Color::new(color.r, color.g, color.b, alpha),
                );
                y += 42.0;
            }
        }
        3 => {
            let center = vec2(rect.x + rect.w - 52.0, rect.y + 54.0);
            for spoke in 0..6 {
                let angle = spoke as f32 * std::f32::consts::TAU / 6.0;
                let end = center + vec2(angle.cos(), angle.sin()) * 24.0;
                draw_line(
                    center.x,
                    center.y,
                    end.x,
                    end.y,
                    2.0,
                    Color::new(color.r, color.g, color.b, alpha),
                );
            }
            draw_circle_lines(
                center.x,
                center.y,
                12.0,
                1.5,
                Color::new(color.r, color.g, color.b, alpha + 0.1),
            );
        }
        _ => {
            let center = vec2(rect.x + rect.w - 56.0, rect.y + rect.h - 50.0);
            draw_circle_lines(
                center.x,
                center.y,
                28.0,
                2.0,
                Color::new(color.r, color.g, color.b, alpha + 0.1),
            );
            draw_circle_lines(
                center.x,
                center.y,
                11.0,
                2.0,
                Color::new(color.r, color.g, color.b, alpha + 0.2),
            );
            draw_line(
                center.x - 36.0,
                center.y,
                center.x + 36.0,
                center.y,
                1.0,
                Color::new(color.r, color.g, color.b, alpha),
            );
            draw_line(
                center.x,
                center.y - 36.0,
                center.x,
                center.y + 36.0,
                1.0,
                Color::new(color.r, color.g, color.b, alpha),
            );
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
        draw_triangle(
            pos + dir * 6.0,
            pos - dir * 4.0 + vec2(-dir.y, dir.x) * 4.0,
            pos - dir * 4.0 - vec2(-dir.y, dir.x) * 4.0,
            Color::new(1.0, 0.78, 0.24, 0.84),
        );
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

/// Short glyph text for a machine node. Unknown building types fall back to the
/// generic label rather than going blank on the map.
fn building_node_label(kind: &str) -> &str {
    let map = &text().map;
    map.node_labels
        .get(kind)
        .map(|label| label.as_str())
        .unwrap_or(map.node_fallback.as_str())
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
