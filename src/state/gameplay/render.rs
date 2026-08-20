//! Top-level draw dispatch: world-space map/entities under the camera, then
//! screen-space HUD/panels/overlays.

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

        // Opening premise card takes precedence over everything else.
        if self.show_intro {
            self.draw_intro();
            return;
        }

        if self.show_workforce {
            self.draw_workforce_overlay();
            return;
        }

        if self.pending_depth_directive.is_some() {
            self.draw_depth_directive_overlay();
            return;
        }

        // When the salvage report modal is up, freeze the build UI behind it and
        // let only the modal take input.
        if self.salvage_report.is_some() {
            self.draw_salvage_report();
            return;
        }

        // Pause menu is a modal too: skip the interactive build UI behind it.
        if self.paused {
            self.draw_pause_menu();
            return;
        }

        self.draw_build_panel(data);
        self.draw_sector_panel();
        self.draw_slot_panel(data);
        self.draw_hover_tooltip(data);
        self.draw_coach();
        self.handle_map_click(data);
    }
}
