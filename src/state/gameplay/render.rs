use crate::data::GameData;
use macroquad::prelude::*;

use super::GameplayState;

impl GameplayState {
    pub fn draw(&mut self, data: &GameData) {
        // --- World-space rendering (with camera) ---
        self.camera.begin();

        self.draw_circuit_board();
        self.draw_towers();
        self.draw_enemies();
        self.draw_shot_effects();
        self.draw_particles();
        self.draw_placement_ghost(data);

        // --- Screen-space rendering (UI) ---
        set_default_camera();

        self.draw_hud(data);
        self.draw_wave_start_flash();
        self.draw_wave_status();
        self.draw_notifications();
        self.draw_build_panel(data);
        self.draw_sector_panel();
        self.draw_slot_panel(data);
        self.draw_hover_tooltip(data);
        self.handle_map_click(data);
    }
}
