//! Shared modal settings overlay.

use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::draw_ui_text;

/// Result of a settings-overlay frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsAction {
    None,
    Close,
}

/// Modal settings overlay, reused by the main menu and the in-game pause menu.
/// Mutates and persists `settings` in place; returns `Close` when the player
/// dismisses it.
pub fn draw_settings_overlay(settings: &mut crate::save::Settings) -> SettingsAction {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.7));

    let pw = 480.0_f32.min(sw - 40.0);
    let ph = 430.0_f32.min(sh - 40.0);
    let px = (sw - pw) * 0.5;
    let py = (sh - ph) * 0.5;
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.98))
        .with_border(1.6, Color::new(0.24, 0.58, 0.62, 0.9));
    macroquad_toolkit::ui::draw_surface(Rect::new(px, py, pw, ph), &surface);

    draw_ui_text("SETTINGS", px + 24.0, py + 42.0, 26.0, dark::ACCENT);

    let row_x = px + 24.0;
    let row_w = pw - 48.0;
    let row_h = 34.0;
    let gap = 14.0;
    let mut y = py + 74.0;
    let mut changed = false;

    if macroquad_toolkit::ui::toggle_row(
        Rect::new(row_x, y, row_w, row_h),
        "Autosave",
        &mut settings.autosave,
    ) {
        changed = true;
    }
    y += row_h + gap;
    if macroquad_toolkit::ui::toggle_row(
        Rect::new(row_x, y, row_w, row_h),
        "Start runs at 2x speed",
        &mut settings.default_fast_speed,
    ) {
        changed = true;
    }
    y += row_h + gap;
    if macroquad_toolkit::ui::slider_row(
        Rect::new(row_x, y, row_w, row_h),
        "Master volume",
        &mut settings.game.master_volume,
        0.0,
        1.0,
    ) {
        changed = true;
    }
    y += row_h + gap;
    if macroquad_toolkit::ui::slider_row(
        Rect::new(row_x, y, row_w, row_h),
        "SFX volume",
        &mut settings.game.sfx_volume,
        0.0,
        1.0,
    ) {
        changed = true;
    }
    y += row_h + gap;
    let tut_label = if settings.tutorial_seen {
        "Replay tutorial on next new run"
    } else {
        "Tutorial will show on next new run"
    };
    if macroquad_toolkit::ui::button(row_x, y, row_w, row_h, tut_label) && settings.tutorial_seen {
        settings.tutorial_seen = false;
        changed = true;
    }

    if changed {
        let _ = settings.save();
    }

    let close_w = 150.0;
    if macroquad_toolkit::ui::button(
        px + (pw - close_w) * 0.5,
        py + ph - 48.0,
        close_w,
        34.0,
        "Close",
    ) {
        return SettingsAction::Close;
    }
    SettingsAction::None
}
