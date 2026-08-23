//! Short-lived visual feedback for factory progress and AI corruption.

use macroquad::prelude::*;
use macroquad_toolkit::math::pulse_range;

use super::{GameplayState, SectionAwakening};

const AWAKENING_DURATION: f32 = 2.8;
const CORRUPTION_DISTORTION_START: f32 = 12.0;
const CORRUPTION_DISTORTION_RANGE: f32 = 28.0;

impl GameplayState {
    /// Record the newly powered section so its first seconds visibly transition
    /// from rust-dark to reclaimed power instead of relying on a text notice.
    pub(crate) fn begin_section_awakening(&mut self, core_building: &str) {
        self.section_awakening = Some(SectionAwakening {
            core_building: core_building.to_string(),
            ttl: AWAKENING_DURATION,
        });
    }

    pub(super) fn update_presentation(&mut self, dt: f32) {
        let expired = if let Some(awakening) = &mut self.section_awakening {
            awakening.ttl = (awakening.ttl - dt).max(0.0);
            awakening.ttl <= 0.0
        } else {
            false
        };
        if expired {
            self.section_awakening = None;
        }
    }

    /// Draw a sweeping green current across the section behind its live systems.
    /// The actual sprites and routes render afterwards, keeping the effect clear
    /// without covering interactive information.
    pub(super) fn draw_section_awakening(&self) {
        let Some(awakening) = &self.section_awakening else {
            return;
        };
        let Some(section) = self
            .map_state
            .section_render_info()
            .into_iter()
            .find(|section| section.core_building == awakening.core_building)
        else {
            return;
        };

        let pad = 46.0;
        let rect = Rect::new(
            (section.min.x - pad).max(0.0),
            (section.min.y - pad).max(0.0),
            (section.max.x - section.min.x + pad * 2.0).max(80.0),
            (section.max.y - section.min.y + pad * 2.0).max(80.0),
        );
        let progress = 1.0 - awakening.ttl / AWAKENING_DURATION;
        let pulse = pulse_range(5.0, 0.75, 1.0);
        let rust_alpha = (1.0 - progress) * 0.16;
        let light_x = rect.x + rect.w * (progress * 1.12).min(1.0);
        let glow = Color::new(0.18, 0.86, 0.58, 0.16 * pulse);

        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.42, 0.15, 0.06, rust_alpha),
        );
        draw_rectangle(rect.x, rect.y, (light_x - rect.x).max(0.0), rect.h, glow);
        draw_line(
            light_x,
            rect.y + 3.0,
            light_x,
            rect.y + rect.h - 3.0,
            3.0,
            Color::new(0.48, 1.0, 0.72, 0.85 * pulse),
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            2.0,
            Color::new(0.32, 0.92, 0.64, 0.72 * (1.0 - progress * 0.45)),
        );
    }

    /// At high AI-corruption levels, lightly tear the HUD with purple-red
    /// scanlines. This leaves every target readable and does not affect input.
    pub(super) fn draw_corruption_distortion(&self) {
        let strength = corruption_distortion_strength(self.threat.corruption);
        if strength <= 0.0 {
            return;
        }

        let width = screen_width();
        let hud_h = self.constants.ui.hud_height;
        let time = get_time() as f32;
        let bands = 5 + (strength * 5.0).round() as i32;
        for band in 0..bands {
            let phase = time * (4.0 + band as f32 * 0.37) + band as f32 * 1.7;
            let y = 12.0 + (band as f32 + 0.5) * (hud_h - 24.0) / bands as f32;
            let offset = phase.sin() * strength * 9.0;
            let color = Color::new(0.88, 0.18, 0.34, 0.07 + strength * 0.1);
            draw_rectangle(offset, y, width, 1.0 + strength * 1.5, color);
            draw_rectangle(
                width * (0.08 + (phase.cos() + 1.0) * 0.18),
                y - 2.0,
                34.0 + strength * 72.0,
                4.0,
                Color::new(0.56, 0.2, 0.86, 0.09 + strength * 0.12),
            );
        }
        draw_rectangle(
            0.0,
            hud_h - 3.0,
            width,
            3.0,
            Color::new(0.86, 0.18, 0.32, 0.22 + strength * 0.38),
        );
    }
}

fn corruption_distortion_strength(corruption: f32) -> f32 {
    ((corruption - CORRUPTION_DISTORTION_START) / CORRUPTION_DISTORTION_RANGE).clamp(0.0, 1.0)
}

#[cfg(test)]
#[path = "presentation/tests.rs"]
mod tests;
