//! Circuit-board map runtime state: slots, paths, traces, and unlocks.

use crate::data::{BuildingBoon, BuildingSlotDef, MapDef, PathDef, SectionDef, TraceDef};
use macroquad::prelude::Vec2;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotState {
    Debris,
    Cleared,
    Powered,
}

impl SlotState {
    pub fn from_str(state: &str) -> Self {
        match state {
            "cleared" => SlotState::Cleared,
            "powered" => SlotState::Powered,
            _ => SlotState::Debris,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SlotState::Debris => "debris",
            SlotState::Cleared => "cleared",
            SlotState::Powered => "powered",
        }
    }
}

pub struct MapSlot {
    pub id: String,
    pub position: Vec2,
    pub state: SlotState,
    pub clear_cost: f32,
    pub power_cost: f32,
    pub opens_entrance: Option<String>,
    pub requires_building_power: Option<String>,
    pub tower_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingState {
    Broken,
    Repaired,
    Powered,
    Disabled,
}

impl BuildingState {
    pub fn from_str(state: &str) -> Self {
        match state {
            "repaired" => BuildingState::Repaired,
            "powered" => BuildingState::Powered,
            "disabled" => BuildingState::Disabled,
            _ => BuildingState::Broken,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BuildingState::Broken => "broken",
            BuildingState::Repaired => "repaired",
            BuildingState::Powered => "powered",
            BuildingState::Disabled => "disabled",
        }
    }
}

pub struct MapBuilding {
    pub id: String,
    pub position: Vec2,
    pub building_type: String,
    pub state: BuildingState,
    pub repair_cost: f32,
    pub power_cost: f32,
    pub boon: BuildingBoon,
    pub threat_per_sec: f32,
    pub opens_entrance: Option<String>,
    pub requires_power_from: Option<String>,
}

impl MapBuilding {
    pub fn from_def(def: BuildingSlotDef) -> Self {
        Self {
            id: def.id,
            position: Vec2::new(def.x, def.y),
            building_type: def.building_type,
            state: BuildingState::Broken,
            repair_cost: def.repair_cost,
            power_cost: def.power_cost,
            boon: def.boon,
            threat_per_sec: def.threat_per_sec,
            opens_entrance: def.opens_entrance,
            requires_power_from: def.requires_power_from,
        }
    }

    pub fn is_active(&self) -> bool {
        self.state == BuildingState::Powered
    }
}

pub struct MapPath {
    pub id: String,
    pub entrance: Vec2,
    pub points: Vec<Vec2>,
    pub active: bool,
    pub initially_active: bool,
    pub requires_entrance: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceNode {
    Slot(usize),
    Building(usize),
    FactoryCore,
}

pub struct MapTrace {
    pub from: TraceNode,
    pub to: TraceNode,
    pub via: Vec<Vec2>,
}

pub struct MapState {
    pub map_size: Vec2,
    pub factory_core: Vec2,
    pub slot_interact_radius: f32,
    pub building_interact_radius: f32,
    pub slots: Vec<MapSlot>,
    pub buildings: Vec<MapBuilding>,
    pub paths: Vec<MapPath>,
    pub traces: Vec<MapTrace>,
    unlocked_entrances: HashSet<String>,
    sections: Vec<MapSection>,
    slot_sections: HashMap<String, usize>,
    building_sections: HashMap<String, usize>,
    /// Force every section visible (survival-proof/testing) — progressive
    /// disclosure would otherwise truncate enemy paths at the frontier.
    reveal_all: bool,
}

#[derive(Clone, Debug)]
pub struct SectionRenderInfo {
    pub label: String,
    pub depth: u32,
    pub core_building: String,
    pub min: Vec2,
    pub max: Vec2,
    pub visible: bool,
}

pub struct MapSection {
    pub label: String,
    pub depth: u32,
    pub core_building: String,
    pub buildings: Vec<String>,
    pub slots: Vec<String>,
    pub visible_at_start: bool,
    pub visible: bool,
}

/// Result of [`MapState::build_sections`] — named so the two lookup maps
/// (structurally identical `HashMap<String, usize>`) can't be swapped at the
/// call site the way an unnamed tuple return would allow.
struct BuiltSections {
    sections: Vec<MapSection>,
    slot_sections: HashMap<String, usize>,
    building_sections: HashMap<String, usize>,
}

impl MapState {
    pub fn from_def(def: MapDef) -> Self {
        let mut slots = Vec::with_capacity(def.slots.len());
        for slot in def.slots {
            slots.push(MapSlot {
                id: slot.id,
                position: Vec2::new(slot.x, slot.y),
                state: SlotState::from_str(&slot.initial_state),
                clear_cost: slot.clear_cost,
                power_cost: slot.power_cost,
                opens_entrance: slot.opens_entrance,
                requires_building_power: slot.requires_building_power,
                tower_index: None,
            });
        }

        let mut buildings = Vec::with_capacity(def.building_slots.len());
        for building in def.building_slots {
            buildings.push(MapBuilding::from_def(building));
        }

        let mut unlocked_entrances = HashSet::new();
        for slot in &slots {
            if slot.state != SlotState::Debris {
                if let Some(ref entrance) = slot.opens_entrance {
                    unlocked_entrances.insert(entrance.clone());
                }
            }
        }

        let paths = def
            .paths
            .into_iter()
            .map(|path| Self::build_path(path, &unlocked_entrances))
            .collect();

        let mut traces = Self::build_traces(&slots, &buildings, def.traces);
        traces.extend(Self::auto_building_traces(&slots, &buildings, &traces));

        let BuiltSections {
            sections,
            slot_sections,
            building_sections,
        } = Self::build_sections(&def.sections);
        let mut state = Self {
            map_size: Vec2::new(def.map_size[0], def.map_size[1]),
            factory_core: Vec2::new(def.factory_core[0], def.factory_core[1]),
            slot_interact_radius: def.slot_interact_radius,
            building_interact_radius: def.building_interact_radius,
            slots,
            buildings,
            paths,
            traces,
            unlocked_entrances,
            sections,
            slot_sections,
            building_sections,
            reveal_all: false,
        };
        state.update_section_visibility();
        state
    }

    /// Reveal every section permanently (survival-proof/testing).
    pub fn reveal_all_sections(&mut self) {
        self.reveal_all = true;
        for section in &mut self.sections {
            section.visible = true;
        }
    }

    fn build_sections(sections: &[SectionDef]) -> BuiltSections {
        let mut slot_sections: HashMap<String, usize> = HashMap::new();
        let mut building_sections: HashMap<String, usize> = HashMap::new();
        let mut result = Vec::new();
        for (idx, def) in sections.iter().enumerate() {
            for slot_id in &def.slots {
                slot_sections.insert(slot_id.clone(), idx);
            }
            for building_id in &def.buildings {
                building_sections.insert(building_id.clone(), idx);
            }
            building_sections.insert(def.core_building.clone(), idx);
            result.push(MapSection {
                label: def.label.clone(),
                depth: def.depth,
                core_building: def.core_building.clone(),
                buildings: def.buildings.clone(),
                slots: def.slots.clone(),
                visible_at_start: def.visible_at_start,
                visible: def.visible_at_start,
            });
        }
        BuiltSections {
            sections: result,
            slot_sections,
            building_sections,
        }
    }

    pub fn update_section_visibility(&mut self) {
        if self.reveal_all || self.sections.is_empty() {
            return;
        }
        let mut base_visible_max = 0usize;
        for (idx, section) in self.sections.iter().enumerate() {
            if section.visible_at_start {
                base_visible_max = base_visible_max.max(idx);
            }
        }

        let mut powered_max: Option<usize> = None;
        for (idx, section) in self.sections.iter().enumerate() {
            if section.core_building == "factory_core" {
                continue;
            }
            if self.is_building_powered(&section.core_building) {
                powered_max = Some(powered_max.map_or(idx, |m| m.max(idx)));
            }
        }

        let visible_max = powered_max
            .map(|m| (m + 1).min(self.sections.len() - 1))
            .unwrap_or(base_visible_max);

        for (idx, section) in self.sections.iter_mut().enumerate() {
            section.visible = idx <= visible_max;
        }
    }

    pub fn is_slot_visible(&self, slot: &MapSlot) -> bool {
        let Some(idx) = self.slot_sections.get(&slot.id).copied() else {
            return true;
        };
        self.sections.get(idx).is_none_or(|s| s.visible)
    }

    pub fn is_building_visible(&self, building: &MapBuilding) -> bool {
        let Some(idx) = self.building_sections.get(&building.id).copied() else {
            return true;
        };
        self.sections.get(idx).is_none_or(|s| s.visible)
    }

    pub fn is_core_building(&self, building_id: &str) -> bool {
        let Some(idx) = self.building_sections.get(building_id).copied() else {
            return false;
        };
        self.sections
            .get(idx)
            .map(|s| s.core_building == building_id)
            .unwrap_or(false)
    }

    pub fn section_render_info(&self) -> Vec<SectionRenderInfo> {
        let mut result = Vec::new();
        for section in &self.sections {
            let mut min = Vec2::new(f32::MAX, f32::MAX);
            let mut max = Vec2::new(f32::MIN, f32::MIN);

            for slot_id in &section.slots {
                if let Some(slot) = self.slots.iter().find(|s| s.id == *slot_id) {
                    min.x = min.x.min(slot.position.x);
                    min.y = min.y.min(slot.position.y);
                    max.x = max.x.max(slot.position.x);
                    max.y = max.y.max(slot.position.y);
                }
            }
            for building_id in &section.buildings {
                if let Some(building) = self.buildings.iter().find(|b| b.id == *building_id) {
                    min.x = min.x.min(building.position.x);
                    min.y = min.y.min(building.position.y);
                    max.x = max.x.max(building.position.x);
                    max.y = max.y.max(building.position.y);
                }
            }

            if min.x == f32::MAX {
                min = Vec2::ZERO;
                max = Vec2::ZERO;
            }

            result.push(SectionRenderInfo {
                label: section.label.clone(),
                depth: section.depth,
                core_building: section.core_building.clone(),
                min,
                max,
                visible: section.visible,
            });
        }
        result
    }

    /// World-space bounding box of everything currently visible (revealed
    /// sections' slots + buildings). Falls back to the full map if nothing
    /// section-scoped is visible yet. Used to frame and bound the camera so the
    /// playable area appears to grow as the factory is powered back online.
    pub fn visible_bounds(&self) -> (Vec2, Vec2) {
        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);
        for slot in &self.slots {
            if self.is_slot_visible(slot) {
                min = min.min(slot.position);
                max = max.max(slot.position);
            }
        }
        for building in &self.buildings {
            if self.building_sections.contains_key(&building.id)
                && self.is_building_visible(building)
            {
                min = min.min(building.position);
                max = max.max(building.position);
            }
        }
        if min.x == f32::MAX {
            (Vec2::ZERO, self.map_size)
        } else {
            (min, max)
        }
    }

    pub fn max_visible_x(&self) -> f32 {
        let mut max_x: f32 = 0.0;
        for slot in &self.slots {
            if self.is_slot_visible(slot) {
                max_x = max_x.max(slot.position.x);
            }
        }
        for building in &self.buildings {
            if self.building_sections.contains_key(&building.id)
                && self.is_building_visible(building)
            {
                max_x = max_x.max(building.position.x);
            }
        }
        if max_x <= 0.0 {
            self.map_size.x
        } else {
            max_x
        }
    }

    /// The deepest factory band the player can currently see. This is derived
    /// from the map's progressive disclosure, so every newly powered wing
    /// advances the campaign's depth without creating a second unlock ledger.
    pub fn deepest_visible_depth(&self) -> u32 {
        self.sections
            .iter()
            .filter(|section| section.visible)
            .map(|section| section.depth)
            .max()
            .unwrap_or(1)
    }

    fn build_path(path: PathDef, unlocked_entrances: &HashSet<String>) -> MapPath {
        let mut points = Vec::with_capacity(path.waypoints.len() + 2);
        points.push(Vec2::new(path.entrance[0], path.entrance[1]));
        for wp in path.waypoints {
            points.push(Vec2::new(wp[0], wp[1]));
        }
        points.push(Vec2::new(path.exit[0], path.exit[1]));

        let mut active = path.initially_active;
        if let Some(ref entrance) = path.requires_entrance {
            if unlocked_entrances.contains(entrance) {
                active = true;
            }
        }

        MapPath {
            id: path.id,
            entrance: Vec2::new(path.entrance[0], path.entrance[1]),
            points,
            active,
            initially_active: path.initially_active,
            requires_entrance: path.requires_entrance,
        }
    }

    fn build_traces(
        slots: &[MapSlot],
        buildings: &[MapBuilding],
        traces: Vec<TraceDef>,
    ) -> Vec<MapTrace> {
        let mut slot_map: HashMap<&str, usize> = HashMap::new();
        for (idx, slot) in slots.iter().enumerate() {
            slot_map.insert(slot.id.as_str(), idx);
        }
        let mut building_map: HashMap<&str, usize> = HashMap::new();
        for (idx, building) in buildings.iter().enumerate() {
            building_map.insert(building.id.as_str(), idx);
        }

        let mut result = Vec::new();
        for trace in traces {
            let from = Self::resolve_trace_node(&trace.from, &slot_map, &building_map);
            let to = Self::resolve_trace_node(&trace.to, &slot_map, &building_map);
            let Some(from) = from else {
                continue;
            };
            let Some(to) = to else {
                continue;
            };
            let via = trace
                .via
                .iter()
                .map(|p| Vec2::new(p[0], p[1]))
                .collect::<Vec<_>>();
            result.push(MapTrace { from, to, via });
        }
        result
    }

    fn auto_building_traces(
        slots: &[MapSlot],
        buildings: &[MapBuilding],
        traces: &[MapTrace],
    ) -> Vec<MapTrace> {
        let mut connected: HashSet<usize> = HashSet::new();
        for trace in traces {
            match trace.from {
                TraceNode::Building(idx) => {
                    connected.insert(idx);
                }
                TraceNode::Slot(_) | TraceNode::FactoryCore => {}
            }
            match trace.to {
                TraceNode::Building(idx) => {
                    connected.insert(idx);
                }
                TraceNode::Slot(_) | TraceNode::FactoryCore => {}
            }
        }

        let mut result = Vec::new();
        for (b_idx, building) in buildings.iter().enumerate() {
            if connected.contains(&b_idx) {
                continue;
            }
            let mut best_idx = None;
            let mut best_dist = f32::MAX;
            for (s_idx, slot) in slots.iter().enumerate() {
                let dist = (slot.position - building.position).length();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = Some(s_idx);
                }
            }
            if let Some(s_idx) = best_idx {
                result.push(MapTrace {
                    from: TraceNode::Building(b_idx),
                    to: TraceNode::Slot(s_idx),
                    via: Vec::new(),
                });
            }
        }
        result
    }

    fn resolve_trace_node(
        id: &str,
        slot_map: &HashMap<&str, usize>,
        building_map: &HashMap<&str, usize>,
    ) -> Option<TraceNode> {
        if id == "factory_core" {
            return Some(TraceNode::FactoryCore);
        }
        slot_map
            .get(id)
            .copied()
            .map(TraceNode::Slot)
            .or_else(|| building_map.get(id).copied().map(TraceNode::Building))
    }

    pub fn active_paths(&self) -> Vec<&MapPath> {
        self.paths.iter().filter(|p| p.active).collect()
    }

    pub fn active_paths_limited(&self) -> Vec<MapPath> {
        let max_x = self.max_visible_x();
        self.paths
            .iter()
            .filter(|p| p.active)
            .map(|p| Self::clamped_path(p, max_x))
            .filter(|p| p.points.len() >= 2)
            .collect()
    }

    pub fn nearest_slot(&self, pos: Vec2) -> Option<(usize, f32)> {
        let mut best = None;
        let mut best_dist = self.slot_interact_radius;
        for (idx, slot) in self.slots.iter().enumerate() {
            if !self.is_slot_visible(slot) {
                continue;
            }
            let dist = (slot.position - pos).length();
            if dist <= best_dist {
                best_dist = dist;
                best = Some(idx);
            }
        }
        best.map(|idx| (idx, best_dist))
    }

    pub fn nearest_building(&self, pos: Vec2) -> Option<(usize, f32)> {
        let mut best = None;
        let mut best_dist = self.building_interact_radius;
        let max_x = self.max_visible_x() + 120.0;
        for (idx, building) in self.buildings.iter().enumerate() {
            if !self.is_building_visible(building) {
                continue;
            }
            if building.position.x > max_x {
                continue;
            }
            let dist = (building.position - pos).length();
            if dist <= best_dist {
                best_dist = dist;
                best = Some(idx);
            }
        }
        best.map(|idx| (idx, best_dist))
    }

    pub fn is_building_powered(&self, id: &str) -> bool {
        self.buildings
            .iter()
            .find(|b| b.id == id)
            .map(|b| b.state == BuildingState::Powered)
            .unwrap_or(false)
    }

    pub fn trace_powered(&self, trace: &MapTrace) -> bool {
        let from_powered = match trace.from {
            TraceNode::FactoryCore => true,
            TraceNode::Slot(idx) => self
                .slots
                .get(idx)
                .is_some_and(|s| s.state == SlotState::Powered),
            TraceNode::Building(idx) => self
                .buildings
                .get(idx)
                .is_some_and(|b| b.state == BuildingState::Powered),
        };
        let to_powered = match trace.to {
            TraceNode::FactoryCore => true,
            TraceNode::Slot(idx) => self
                .slots
                .get(idx)
                .is_some_and(|s| s.state == SlotState::Powered),
            TraceNode::Building(idx) => self
                .buildings
                .get(idx)
                .is_some_and(|b| b.state == BuildingState::Powered),
        };
        from_powered && to_powered
    }

    pub fn set_slot_state(&mut self, idx: usize, state: SlotState) -> Vec<String> {
        let mut entrance_to_unlock = None;
        {
            let Some(slot) = self.slots.get_mut(idx) else {
                return Vec::new();
            };
            if slot.state == state {
                return Vec::new();
            }
            slot.state = state;
            if slot.state != SlotState::Debris {
                entrance_to_unlock = slot.opens_entrance.clone();
            }
        }

        if let Some(entrance) = entrance_to_unlock {
            return self.unlock_entrance(&entrance);
        }
        Vec::new()
    }

    pub fn set_building_state(&mut self, idx: usize, state: BuildingState) -> Vec<String> {
        let mut entrance_to_unlock = None;
        {
            let Some(building) = self.buildings.get_mut(idx) else {
                return Vec::new();
            };
            if building.state == state {
                return Vec::new();
            }
            building.state = state;
            if building.state == BuildingState::Powered {
                entrance_to_unlock = building.opens_entrance.clone();
            }
        }

        if let Some(entrance) = entrance_to_unlock {
            return self.unlock_entrance(&entrance);
        }
        Vec::new()
    }

    pub fn rebuild_unlocks(&mut self) {
        self.unlocked_entrances.clear();
        for slot in &self.slots {
            if slot.state != SlotState::Debris {
                if let Some(ref entrance) = slot.opens_entrance {
                    self.unlocked_entrances.insert(entrance.clone());
                }
            }
        }
        for building in &self.buildings {
            if building.state == BuildingState::Powered {
                if let Some(ref entrance) = building.opens_entrance {
                    self.unlocked_entrances.insert(entrance.clone());
                }
            }
        }
        self.refresh_path_activity();
    }

    fn unlock_entrance(&mut self, entrance: &str) -> Vec<String> {
        if !self.unlocked_entrances.insert(entrance.to_string()) {
            return Vec::new();
        }
        self.refresh_path_activity()
    }

    fn refresh_path_activity(&mut self) -> Vec<String> {
        let mut newly_active = Vec::new();
        for path in &mut self.paths {
            let mut should_active = path.initially_active;
            if let Some(ref entrance) = path.requires_entrance {
                if self.unlocked_entrances.contains(entrance) {
                    should_active = true;
                }
            }
            if should_active && !path.active {
                path.active = true;
                newly_active.push(path.id.clone());
            } else if !should_active {
                path.active = false;
            }
        }
        newly_active
    }

    fn clamped_path(path: &MapPath, max_x: f32) -> MapPath {
        let mut points = Vec::new();
        for p in &path.points {
            if p.x <= max_x {
                points.push(*p);
            } else {
                break;
            }
        }
        if points.len() < 2 {
            points.push(path.entrance);
            points.push(path.entrance + Vec2::new(1.0, 0.0));
        }
        MapPath {
            id: path.id.clone(),
            entrance: path.entrance,
            points,
            active: path.active,
            initially_active: path.initially_active,
            requires_entrance: path.requires_entrance.clone(),
        }
    }
}
