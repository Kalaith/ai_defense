//! Touch-first overlay for the operating choice at a newly reached depth.

use crate::data::strings::{fill, text};
use crate::engine::depth::DepthDirective;
use crate::ui::{self, ConsoleButtonState};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::draw_ui_text;

use super::GameplayState;

impl GameplayState {
    pub(super) fn draw_depth_directive_overlay(&mut self) {
        let Some(depth) = self.pending_depth_directive else {
            return;
        };
        let t = text().depth_directive.clone();
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.7));

        let panel_w = 820.0_f32.min(sw - 36.0);
        let panel_h = 360.0_f32.min(sh - 36.0);
        let panel_x = (sw - panel_w) * 0.5;
        let panel_y = (sh - panel_h) * 0.5;
        let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.99))
            .with_border(1.8, Color::new(0.3, 0.72, 0.78, 0.95));
        macroquad_toolkit::ui::draw_surface(
            Rect::new(panel_x, panel_y, panel_w, panel_h),
            &surface,
        );

        let title = fill(&t.title, &[("n", &format!("{depth:02}"))]);
        draw_ui_text(&title, panel_x + 28.0, panel_y + 42.0, 25.0, dark::ACCENT);
        ui::draw_bounded_text(
            &t.subtitle,
            panel_x + 28.0,
            panel_y + 68.0,
            panel_w - 56.0,
            13.0,
            dark::TEXT_DIM,
        );

        let gap = 12.0;
        let card_w = (panel_w - 56.0 - gap * 2.0) / 3.0;
        let card_y = panel_y + 105.0;
        let mut chosen = None;
        for (idx, directive) in DepthDirective::ALL.into_iter().enumerate() {
            let (name, detail, color) = match directive {
                DepthDirective::ReinforcedRoutes => (
                    &t.reinforced_routes,
                    &t.reinforced_routes_detail,
                    Color::new(0.25, 0.65, 0.92, 1.0),
                ),
                DepthDirective::SalvageLifts => (
                    &t.salvage_lifts,
                    &t.salvage_lifts_detail,
                    Color::new(0.84, 0.63, 0.22, 1.0),
                ),
                DepthDirective::EvacuationRelay => (
                    &t.evacuation_relay,
                    &t.evacuation_relay_detail,
                    Color::new(0.28, 0.86, 0.65, 1.0),
                ),
            };
            let x = panel_x + 28.0 + idx as f32 * (card_w + gap);
            ui::draw_console_panel(
                Rect::new(x, card_y, card_w, 164.0),
                Color::new(color.r, color.g, color.b, 0.28),
            );
            draw_ui_text(name, x + 12.0, card_y + 25.0, 13.0, color);
            ui::draw_bounded_text(
                detail,
                x + 12.0,
                card_y + 54.0,
                card_w - 24.0,
                11.0,
                dark::TEXT,
            );
            if ui::draw_console_button(
                x + 12.0,
                card_y + 116.0,
                card_w - 24.0,
                34.0,
                &t.install,
                ConsoleButtonState::Recommended,
            ) {
                chosen = Some(directive);
            }
        }

        if let Some(directive) = chosen {
            self.choose_depth_directive(directive);
        }
    }
}
