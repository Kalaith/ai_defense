//! Circuit-board map runtime state: slots, paths, traces, and unlocks.

use crate::data::{BuildingBoon, BuildingSlotDef, MapDef, PathDef, TraceDef};
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

        let traces = Self::build_traces(&slots, def.traces);

        Self {
            map_size: Vec2::new(def.map_size[0], def.map_size[1]),
            factory_core: Vec2::new(def.factory_core[0], def.factory_core[1]),
            slot_interact_radius: def.slot_interact_radius,
            building_interact_radius: def.building_interact_radius,
            slots,
            buildings,
            paths,
            traces,
            unlocked_entrances,
        }
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

    fn build_traces(slots: &[MapSlot], traces: Vec<TraceDef>) -> Vec<MapTrace> {
        let mut slot_map: HashMap<&str, usize> = HashMap::new();
        for (idx, slot) in slots.iter().enumerate() {
            slot_map.insert(slot.id.as_str(), idx);
        }

        let mut result = Vec::new();
        for trace in traces {
            let from = Self::resolve_trace_node(&trace.from, &slot_map);
            let to = Self::resolve_trace_node(&trace.to, &slot_map);
            let Some(from) = from else { continue; };
            let Some(to) = to else { continue; };
            let via = trace
                .via
                .iter()
                .map(|p| Vec2::new(p[0], p[1]))
                .collect::<Vec<_>>();
            result.push(MapTrace { from, to, via });
        }
        result
    }

    fn resolve_trace_node(id: &str, slot_map: &HashMap<&str, usize>) -> Option<TraceNode> {
        if id == "factory_core" {
            return Some(TraceNode::FactoryCore);
        }
        slot_map.get(id).copied().map(TraceNode::Slot)
    }

    pub fn active_paths(&self) -> Vec<&MapPath> {
        self.paths.iter().filter(|p| p.active).collect()
    }

    pub fn nearest_slot(&self, pos: Vec2) -> Option<(usize, f32)> {
        let mut best = None;
        let mut best_dist = self.slot_interact_radius;
        for (idx, slot) in self.slots.iter().enumerate() {
            let dist = (slot.position - pos).length();
            if dist <= best_dist {
                best_dist = dist;
                best = Some(idx);
            }
        }
        best.map(|idx| (idx, best_dist))
    }


    pub fn trace_powered(&self, trace: &MapTrace) -> bool {
        let from_powered = match trace.from {
            TraceNode::FactoryCore => true,
            TraceNode::Slot(idx) => self.slots.get(idx).map_or(false, |s| s.state == SlotState::Powered),
        };
        let to_powered = match trace.to {
            TraceNode::FactoryCore => true,
            TraceNode::Slot(idx) => self.slots.get(idx).map_or(false, |s| s.state == SlotState::Powered),
        };
        from_powered && to_powered
    }

    pub fn set_slot_state(&mut self, idx: usize, state: SlotState) -> Vec<String> {
        let mut entrance_to_unlock = None;
        {
            let Some(slot) = self.slots.get_mut(idx) else { return Vec::new(); };
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
            let Some(building) = self.buildings.get_mut(idx) else { return Vec::new(); };
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
}
