//! Main menu state.

use crate::save::{SaveData, Settings};
use crate::state::StateTransition;
use crate::ui;
use macroquad::prelude::*;
use macroquad_toolkit::math::pulse_range;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub struct MenuState {
    start_clicked: bool,
    continue_clicked: bool,
    has_save: bool,
    title_texture: Option<Texture2D>,
    show_settings: bool,
    settings: Settings,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            start_clicked: false,
            continue_clicked: false,
            has_save: SaveData::exists(),
            title_texture: load_title_texture(),
            show_settings: false,
            settings: Settings::load(),
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
        self.has_save = SaveData::exists();
        draw_title_background(self.title_texture.as_ref());
        draw_title_scanlines();

        if self.show_settings {
            if ui::draw_settings_overlay(&mut self.settings) == ui::SettingsAction::Close {
                self.show_settings = false;
            }
            return;
        }

        let w = screen_width();
        let h = screen_height();
        let panel_x = (w * 0.065).clamp(28.0, 92.0);
        let btn_h = 56.0;
        let gap = 11.0;
        let panel_w = (w * 0.34).clamp(300.0, 430.0);
        let panel_y = (h * 0.52).clamp(280.0, h - (btn_h * 4.0 + gap * 3.0) - 40.0);

        let continue_label = if self.has_save {
            "CONTINUE SIGNAL"
        } else {
            "NO SAVED SIGNAL"
        };
        let continue_detail = if self.has_save {
            "Resume the last factory command state"
        } else {
            "A save will appear here after a run begins"
        };

        if draw_title_button(
            Rect::new(panel_x, panel_y, panel_w, btn_h),
            continue_label,
            continue_detail,
            self.has_save,
            true,
        ) {
            self.continue_clicked = true;
        }

        if draw_title_button(
            Rect::new(panel_x, panel_y + btn_h + gap, panel_w, btn_h),
            "NEW CAMPAIGN",
            "Wake the assembly line from cold storage",
            true,
            !self.has_save,
        ) {
            self.start_clicked = true;
        }

        if draw_title_button(
            Rect::new(panel_x, panel_y + (btn_h + gap) * 2.0, panel_w, btn_h),
            "SETTINGS",
            "Audio, autosave, speed, and tutorial options",
            true,
            false,
        ) {
            self.show_settings = true;
        }

        if draw_title_button(
            Rect::new(panel_x, panel_y + (btn_h + gap) * 3.0, panel_w, btn_h),
            "EXIT GAME",
            "Power down the console and quit",
            true,
            false,
        ) {
            request_exit();
        }
    }
}

/// Quit the process on native builds. On the web there is no process to exit,
/// so this is a no-op.
pub fn request_exit() {
    #[cfg(not(target_arch = "wasm32"))]
    std::process::exit(0);
}

fn load_title_texture() -> Option<Texture2D> {
    let image = Image::from_file_with_format(
        include_bytes!("../../assets/title_background.png"),
        Some(ImageFormat::Png),
    )
    .ok()?;
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Linear);
    Some(texture)
}

fn draw_title_background(texture: Option<&Texture2D>) {
    clear_background(BLACK);
    let w = screen_width();
    let h = screen_height();

    if let Some(texture) = texture {
        let scale = (w / texture.width()).max(h / texture.height());
        let draw_w = texture.width() * scale;
        let draw_h = texture.height() * scale;
        draw_texture_ex(
            texture,
            (w - draw_w) * 0.5,
            (h - draw_h) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.01, 0.015, 0.02, 1.0));
    }

    let vignette = Color::new(0.0, 0.0, 0.0, 0.22);
    draw_rectangle(0.0, 0.0, w, h * 0.18, vignette);
    draw_rectangle(0.0, h * 0.78, w, h * 0.22, Color::new(0.0, 0.0, 0.0, 0.32));
}

fn draw_title_scanlines() {
    let w = screen_width();
    let h = screen_height();
    let mut y = 0.0;
    while y < h {
        draw_line(0.0, y, w, y, 1.0, Color::new(1.0, 1.0, 1.0, 0.018));
        y += 5.0;
    }
}

fn draw_title_button(rect: Rect, label: &str, detail: &str, enabled: bool, primary: bool) -> bool {
    let (mx, my) = mouse_position();
    let hovered =
        enabled && mx >= rect.x && mx <= rect.x + rect.w && my >= rect.y && my <= rect.y + rect.h;
    let pulse = pulse_range(3.2, 0.55, 0.77);
    let accent = if !enabled {
        Color::new(0.28, 0.32, 0.34, 0.55)
    } else if primary {
        Color::new(0.95, 0.18, 0.15, 0.82 + pulse * 0.12)
    } else {
        Color::new(0.28, 0.74, 1.0, 0.78 + pulse * 0.1)
    };
    let fill = if hovered {
        Color::new(0.09, 0.12, 0.14, 0.82)
    } else {
        Color::new(0.02, 0.03, 0.04, 0.68)
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.8, accent);
    draw_line(
        rect.x,
        rect.y + rect.h - 2.0,
        rect.x + rect.w,
        rect.y + rect.h - 2.0,
        2.0,
        Color::new(
            accent.r,
            accent.g,
            accent.b,
            if enabled { 0.62 } else { 0.24 },
        ),
    );
    draw_rectangle(
        rect.x,
        rect.y,
        4.0,
        rect.h,
        Color::new(
            accent.r,
            accent.g,
            accent.b,
            if enabled { 0.82 } else { 0.28 },
        ),
    );

    let label_color = if enabled {
        Color::new(0.92, 0.96, 0.96, 1.0)
    } else {
        Color::new(0.42, 0.48, 0.5, 0.86)
    };
    draw_bounded_text(
        label,
        rect.x + 18.0,
        rect.y + 24.0,
        rect.w - 82.0,
        18.0,
        label_color,
    );
    draw_bounded_text(
        detail,
        rect.x + 18.0,
        rect.y + 43.0,
        rect.w - 82.0,
        11.0,
        Color::new(0.62, 0.72, 0.76, if enabled { 0.92 } else { 0.5 }),
    );

    let marker_x = rect.x + rect.w - 42.0;
    let marker_y = rect.y + rect.h * 0.5;
    draw_circle_lines(marker_x, marker_y, 15.0, 1.5, accent);
    if enabled {
        let tri = [
            vec2(marker_x - 4.0, marker_y - 7.0),
            vec2(marker_x - 4.0, marker_y + 7.0),
            vec2(marker_x + 7.0, marker_y),
        ];
        draw_triangle(tri[0], tri[1], tri[2], accent);
    } else {
        draw_line(
            marker_x - 7.0,
            marker_y,
            marker_x + 7.0,
            marker_y,
            1.5,
            accent,
        );
    }

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_bounded_text(text: &str, x: f32, y: f32, max_w: f32, font_size: f32, color: Color) {
    let bounded = truncate_text(text, max_w, font_size as u16);
    draw_ui_text(&bounded, x, y, font_size, color);
}

fn truncate_text(text: &str, max_w: f32, font_size: u16) -> String {
    if measure_ui_text(text, None, font_size, 1.0).width <= max_w {
        return text.to_string();
    }

    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}...", out, ch);
        if measure_ui_text(&candidate, None, font_size, 1.0).width > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{}...", out)
}
