//! Route-coverage geometry: which attack paths a given position and range
//! actually threatens.

use macroquad::prelude::*;

use super::super::GameplayState;

impl GameplayState {
    /// Ids of the active attack routes that pass within `range` of `center`.
    pub fn covered_paths_for_range(&self, center: Vec2, range: f32) -> Vec<String> {
        let mut result = Vec::new();
        for path in self.map_state.active_paths_limited() {
            let covered = path
                .points
                .windows(2)
                .any(|pair| distance_to_segment(center, pair[0], pair[1]) <= range);
            if covered {
                result.push(path.id);
            }
        }
        result
    }

    /// Distance from `pos` to the nearest active route, falling back to the core
    /// when no route is open yet.
    pub(super) fn distance_to_active_route(&self, pos: Vec2) -> f32 {
        let mut best = f32::MAX;
        for path in self.map_state.active_paths_limited() {
            for pair in path.points.windows(2) {
                best = best.min(distance_to_segment(pos, pair[0], pair[1]));
            }
        }
        if best == f32::MAX {
            (pos - self.map_state.factory_core).length()
        } else {
            best
        }
    }
}

fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq <= f32::EPSILON {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (p - closest).length()
}
