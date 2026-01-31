//! Main menu state.

use crate::save::SaveData;
use crate::state::StateTransition;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::button;

pub struct MenuState {
    start_clicked: bool,
    continue_clicked: bool,
    has_save: bool,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            start_clicked: false,
            continue_clicked: false,
            has_save: SaveData::exists(),
        }
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        if self.start_clicked {
            self.start_clicked = false;
            return Some(StateTransition::ToGameplay);
        }
        if self.continue_clicked {
            self.continue_clicked = false;
            if let Ok(save) = SaveData::load() {
                return Some(StateTransition::ToGameplayLoaded { save });
            }
        }
        None
    }

    pub fn draw(&mut self) {
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;

        // Title
        let title = "LAST ASSEMBLY";
        let font_size = 48.0;
        let dims = measure_text(title, None, font_size as u16, 1.0);
        draw_text(
            title,
            center_x - dims.width / 2.0,
            center_y - 80.0,
            font_size,
            dark::TEXT_BRIGHT,
        );

        // Subtitle
        let subtitle = "Tower Defense  |  Factory Management  |  AI Uprising";
        let sub_dims = measure_text(subtitle, None, 20, 1.0);
        draw_text(
            subtitle,
            center_x - sub_dims.width / 2.0,
            center_y - 40.0,
            20.0,
            dark::TEXT_DIM,
        );

        // Start button
        let btn_w = 220.0;
        let btn_h = 50.0;
        let btn_x = center_x - btn_w / 2.0;
        let mut btn_y = center_y + 10.0;

        if button(btn_x, btn_y, btn_w, btn_h, "New Campaign") {
            self.start_clicked = true;
        }

        if self.has_save {
            btn_y += 60.0;
            if button(btn_x, btn_y, btn_w, btn_h, "Continue") {
                self.continue_clicked = true;
            }
        }
    }
}
