//! Player-facing names and blurbs derived from raw ids and data definitions.

use crate::data::BuildingBoon;
use crate::engine::map::MapBuilding;

use super::super::helpers::entrance_label;
use super::super::GameplayState;

impl GameplayState {
    pub fn path_display_name(&self, path_id: &str) -> String {
        let label = entrance_label(path_id);
        if label == path_id {
            titleize(path_id)
        } else {
            label.to_string()
        }
    }

    /// Comma-joined display names for a set of path ids, for coverage read-outs.
    pub(crate) fn join_path_names(&self, path_ids: &[String]) -> String {
        path_ids
            .iter()
            .map(|id| self.path_display_name(id))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn building_display_name(&self, building: &MapBuilding) -> String {
        let raw = if building.building_type.is_empty() {
            building.id.as_str()
        } else {
            building.building_type.as_str()
        };
        titleize(raw)
    }

    /// Same as [`GameplayState::building_display_name`] but keyed by id, for
    /// callers that only hold an unlock token or a power-dependency reference.
    /// Falls back to titleizing the id when no such building exists.
    pub(crate) fn building_display_name_by_id(&self, id: &str) -> String {
        self.map_state
            .buildings
            .iter()
            .find(|b| b.id == id)
            .map(|b| self.building_display_name(b))
            .unwrap_or_else(|| titleize(id))
    }

    pub fn boon_text(&self, boon: &BuildingBoon) -> String {
        let mut parts = Vec::new();
        if boon.scrap_per_sec > 0.0 {
            parts.push(format!("+{:.1} scrap/s", boon.scrap_per_sec));
        }
        if boon.food_per_sec > 0.0 {
            parts.push(format!("+{:.1} food/s", boon.food_per_sec));
        }
        if boon.water_per_sec > 0.0 {
            parts.push(format!("+{:.1} water/s", boon.water_per_sec));
        }
        if boon.power_per_sec > 0.0 {
            parts.push(format!("+{:.1} power/s", boon.power_per_sec));
        }
        if parts.is_empty() {
            "Factory stability".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// `scrap_converter` -> `Scrap Converter`.
fn titleize(id: &str) -> String {
    id.split('_')
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
