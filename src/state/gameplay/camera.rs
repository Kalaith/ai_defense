//! Camera bounds derived from the currently revealed factory sections.

use macroquad::prelude::vec2;
use macroquad_toolkit::camera::CameraBounds;

use super::GameplayState;

impl GameplayState {
    /// Grow the camera's pannable area to match the currently-revealed sections
    /// so newly-powered wings of the factory become reachable, while keeping the
    /// unrevealed expanse out of reach.
    pub(crate) fn sync_camera_bounds(&mut self) {
        let (vmin, vmax) = self.map_state.visible_bounds();
        let pad = 240.0;
        let bmin = vec2((vmin.x - pad).max(0.0), (vmin.y - pad).max(0.0));
        let bmax = vec2(
            (vmax.x + pad).min(self.map_state.map_size.x),
            (vmax.y + pad).min(self.map_state.map_size.y),
        );
        self.camera.set_bounds(Some(CameraBounds::new(bmin, bmax)));
    }
}
