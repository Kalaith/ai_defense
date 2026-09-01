//! Compact telemetry strip, edge controls, and short-lived screen feedback.
//!
//! The factory floor is the primary surface. Larger consoles are opened from
//! the edge controls only when the player needs them.

mod beacon_panel;

use crate::data::strings::{fill, text};
use crate::data::GameData;
use crate::ui::{self, ConsoleButtonState, ConsoleIcon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::notifications::{
    draw_notification, Notification, NotificationRenderConfig, NotificationType,
};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::helpers::{beacon_color, threat_color};
use super::ui::EdgeSide;
use super::ui_advice::{AdviceTarget, AlertSeverity, PowerGridSnapshot, UiAdvice};
use super::GameplayState;

impl GameplayState {
    pub(super) fn draw_hud(&mut self, data: &GameData) {
        let advice = self.build_ui_advice(data);
        let hud_h = self.constants.ui.hud_height;
        let sw = screen_width();
        draw_rectangle(0.0, 0.0, sw, hud_h, Color::new(0.018, 0.026, 0.031, 0.94));
        draw_rectangle(
            0.0,
            hud_h - 3.0,
            sw,
            3.0,
            Color::new(0.16, 0.42, 0.42, 0.82),
        );

        let gap = 8.0;
        let margin = 10.0;
        let status_w = (sw * 0.14).clamp(142.0, 178.0);
        let threat_w = (sw * 0.16).clamp(148.0, 194.0);
        let beacon_w = (sw * 0.17).clamp(158.0, 216.0);
        let wide = sw >= 1180.0;
        let next_w = if wide {
            (sw * 0.2).clamp(190.0, 260.0)
        } else {
            0.0
        };
        let fixed_w = status_w + threat_w + beacon_w + next_w + gap * if wide { 4.0 } else { 3.0 };
        let metrics_w = (sw - margin * 2.0 - fixed_w).max(260.0);

        let status_rect = Rect::new(margin, 9.0, status_w, hud_h - 18.0);
        let metrics_rect = Rect::new(status_rect.x + status_w + gap, 9.0, metrics_w, hud_h - 18.0);
        let threat_rect = Rect::new(
            metrics_rect.x + metrics_w + gap,
            9.0,
            threat_w,
            hud_h - 18.0,
        );
        let mut next_x = threat_rect.x + threat_w + gap;

        self.draw_compact_status(status_rect);
        self.draw_compact_metrics(metrics_rect, &advice.power);
        self.draw_compact_threat(threat_rect, &advice);

        if wide {
            let next_rect = Rect::new(next_x, 9.0, next_w, hud_h - 18.0);
            self.draw_compact_next(next_rect, &advice, data);
            next_x += next_w + gap;
        }
        self.draw_compact_beacon(Rect::new(next_x, 9.0, beacon_w, hud_h - 18.0));
    }

    fn draw_compact_status(&self, rect: Rect) {
        let t = &text().hud;
        ui::draw_console_panel(rect, Color::new(0.16, 0.42, 0.38, 0.66));
        draw_ui_text(
            &t.survival,
            rect.x + 10.0,
            rect.y + 17.0,
            11.0,
            dark::POSITIVE,
        );
        draw_ui_text(
            &self.depth_readout(),
            rect.x + 10.0,
            rect.y + 34.0,
            10.0,
            dark::TEXT_DIM,
        );
        draw_ui_text(
            &self.beacon_phase.label().to_uppercase(),
            rect.x + 10.0,
            rect.y + 52.0,
            11.0,
            beacon_color(&self.beacon_phase),
        );
        ui::draw_icon(
            ConsoleIcon::Health,
            rect.x + rect.w - 27.0,
            rect.y + 34.0,
            14.0,
            Color::new(0.35, 0.95, 0.72, 0.78),
        );
    }

    fn draw_compact_metrics(&self, rect: Rect, power: &PowerGridSnapshot) {
        let t = &text().hud;
        ui::draw_console_panel(rect, Color::new(0.12, 0.24, 0.25, 0.58));
        let entries = [
            (
                ConsoleIcon::People,
                t.pop.as_str(),
                format!("{}/{}", self.population.count, self.shelter_capacity()),
                dark::TEXT_BRIGHT,
            ),
            (
                ConsoleIcon::Scrap,
                t.scrap.as_str(),
                format!("{:.0}", self.resources.scrap),
                dark::TEXT_BRIGHT,
            ),
            (
                ConsoleIcon::Power,
                t.power.as_str(),
                format!("{:.0} {:+.0}", power.battery, power.net),
                dark::POSITIVE,
            ),
            (
                ConsoleIcon::Food,
                t.food.as_str(),
                format!("{:.0}", self.population.food_supply),
                dark::TEXT_BRIGHT,
            ),
            (
                ConsoleIcon::Water,
                t.water.as_str(),
                format!("{:.0}", self.resources.water),
                dark::TEXT_BRIGHT,
            ),
            (
                ConsoleIcon::Morale,
                t.condition.as_str(),
                format!(
                    "{:.0}/{:.0}",
                    self.population.morale, self.population.health
                ),
                if self.population.morale < 25.0 {
                    dark::NEGATIVE
                } else {
                    dark::POSITIVE
                },
            ),
        ];
        let col_w = rect.w / entries.len() as f32;
        for (idx, (icon, label, value, color)) in entries.iter().enumerate() {
            let x = rect.x + idx as f32 * col_w;
            ui::draw_icon(
                *icon,
                x + 7.0,
                rect.y + 9.0,
                18.0,
                Color::new(color.r, color.g, color.b, 0.78),
            );
            draw_ui_text(label, x + 29.0, rect.y + 20.0, 8.0, dark::TEXT_DIM);
            draw_ui_text(value, x + 29.0, rect.y + 42.0, 14.0, *color);
        }
        draw_button_hint(
            Rect::new(rect.x + col_w * 2.0, rect.y, col_w, rect.h),
            &t.power_grid,
            &format!(
                "{} {:.0} · {} {:.0} · {} {:+.0}/s",
                t.gen, power.generated, t.used, power.used, t.net, power.net
            ),
        );
    }

    fn draw_compact_threat(&self, rect: Rect, advice: &UiAdvice) {
        let t = &text().hud;
        let color = threat_color(&self.threat, &self.constants.threat);
        ui::draw_console_panel(rect, Color::new(color.r, color.g, color.b, 0.52));
        draw_ui_text(&t.threat, rect.x + 10.0, rect.y + 17.0, 11.0, color);
        draw_ui_text(
            &format!(
                "{:.0}% · {}",
                self.threat.awareness_level(),
                self.threat.reaction_tier(&self.constants.threat).label()
            ),
            rect.x + 10.0,
            rect.y + 34.0,
            10.0,
            dark::TEXT_BRIGHT,
        );
        let wave = if !self.beacon_active && !self.shutdown_triggered {
            t.beacon_offline.clone()
        } else if self.between_waves {
            fill(
                &t.wave_incoming,
                &[
                    ("wave", &(self.current_wave + 1).to_string()),
                    ("secs", &format!("{:.0}", self.wave_timer.max(0.0))),
                ],
            )
        } else {
            fill(
                &t.wave_active,
                &[
                    ("wave", &self.current_wave.to_string()),
                    ("alive", &self.wave_manager.alive_count().to_string()),
                ],
            )
        };
        ui::draw_bounded_text(
            &wave,
            rect.x + 10.0,
            rect.y + 51.0,
            rect.w - 20.0,
            9.0,
            dark::TEXT_DIM,
        );
        if let Some(alert) = advice.alerts.first() {
            let alert_color = if alert.severity == AlertSeverity::Critical {
                dark::NEGATIVE
            } else {
                dark::WARNING
            };
            draw_rectangle(rect.x + rect.w - 13.0, rect.y + 8.0, 5.0, 5.0, alert_color);
            draw_button_hint(rect, &alert.label, &alert.detail);
        } else {
            let composition = super::ui_advice::format_enemy_counts(&advice.wave_preview.counts);
            draw_button_hint(rect, &t.threat, &composition);
        }
    }

    fn draw_compact_next(&mut self, rect: Rect, advice: &UiAdvice, data: &GameData) {
        let t = &text().hud;
        ui::draw_console_panel(rect, Color::new(0.16, 0.36, 0.54, 0.58));
        draw_ui_text(
            &t.next_step,
            rect.x + 10.0,
            rect.y + 16.0,
            9.0,
            dark::ACCENT,
        );
        ui::draw_bounded_text(
            &advice.suggested_action.label,
            rect.x + 10.0,
            rect.y + 34.0,
            rect.w - 78.0,
            14.0,
            dark::TEXT_BRIGHT,
        );
        ui::draw_bounded_text(
            &fill(
                &t.cost_risk,
                &[
                    ("cost", &advice.suggested_action.cost),
                    ("risk", &advice.risk),
                ],
            ),
            rect.x + 10.0,
            rect.y + 50.0,
            rect.w - 78.0,
            9.0,
            dark::TEXT_DIM,
        );
        let interactive = !self.show_intro
            && !self.show_workforce
            && self.salvage_report.is_none()
            && self.pending_depth_directive.is_none()
            && !self.paused;
        let state = if !interactive || matches!(advice.suggested_action.target, AdviceTarget::None)
        {
            ConsoleButtonState::Disabled
        } else {
            ConsoleButtonState::Recommended
        };
        if interactive
            && ui::draw_console_button(
                rect.x + rect.w - 64.0,
                rect.y + 16.0,
                54.0,
                30.0,
                &t.focus,
                state,
            )
        {
            let target = advice.suggested_action.target.clone();
            self.apply_advice_focus(&target, data);
        }
        draw_button_hint(
            Rect::new(rect.x, rect.y, rect.w - 72.0, rect.h),
            &advice.suggested_action.label,
            &advice.suggested_action.detail,
        );
    }

    fn draw_compact_beacon(&self, rect: Rect) {
        let t = &text().hud;
        let color = beacon_color(&self.beacon_phase);
        ui::draw_console_panel(rect, Color::new(color.r, color.g, color.b, 0.5));
        draw_ui_text(&t.edge_beacon, rect.x + 10.0, rect.y + 17.0, 10.0, color);
        ui::draw_bounded_text(
            &self.beacon_phase.label().to_uppercase(),
            rect.x + 10.0,
            rect.y + 36.0,
            rect.w - 20.0,
            13.0,
            dark::TEXT_BRIGHT,
        );
        let evac = if self.beacon_active {
            fill(
                &t.evac_pending,
                &[
                    ("n", &self.survivors_evacuated.to_string()),
                    ("pending", &format!("{:.0}", self.pending_evacuees)),
                ],
            )
        } else {
            fill(&t.evac, &[("n", &self.survivors_evacuated.to_string())])
        };
        ui::draw_bounded_text(
            &evac,
            rect.x + 10.0,
            rect.y + 52.0,
            rect.w - 20.0,
            9.0,
            dark::POSITIVE,
        );
    }

    pub(super) fn draw_edge_controls(&mut self) {
        let t = &text().hud;
        if ui::draw_edge_button(
            self.edge_button_rect(EdgeSide::Left, 0),
            ConsoleIcon::Tower,
            &t.edge_build,
            self.show_build_panel,
            dark::ACCENT,
        ) {
            self.open_only_build_panel();
        }
        if ui::draw_edge_button(
            self.edge_button_rect(EdgeSide::Left, 1),
            ConsoleIcon::Power,
            &t.edge_systems,
            self.show_sector_panel,
            dark::POSITIVE,
        ) {
            self.open_only_sector_panel();
        }
        if ui::draw_edge_button(
            self.edge_button_rect(EdgeSide::Left, 2),
            ConsoleIcon::People,
            &t.edge_shift,
            self.show_workforce,
            dark::POSITIVE,
        ) {
            self.close_dock_panels();
            self.show_workforce = true;
        }
        if ui::draw_edge_button(
            self.edge_button_rect(EdgeSide::Right, 0),
            ConsoleIcon::Risk,
            &t.edge_beacon,
            self.show_beacon_panel,
            dark::NEGATIVE,
        ) {
            self.open_only_beacon_panel();
        }
        if ui::draw_edge_button(
            self.edge_button_rect(EdgeSide::Right, 1),
            ConsoleIcon::Pause,
            &t.edge_pause,
            false,
            dark::WARNING,
        ) {
            self.close_dock_panels();
            self.paused = true;
        }
    }

    pub(super) fn draw_wave_status(&self) {
        if self.time_scale <= 1.0 {
            return;
        }
        let tag = &text().hud.speed_tag;
        let tag_w = 34.0;
        let tag_h = 20.0;
        let x = screen_width() - super::ui::EDGE_RAIL_MARGIN - super::ui::EDGE_RAIL_W - tag_w - 8.0;
        let y = self.constants.ui.hud_height + 8.0;
        draw_rectangle(x, y, tag_w, tag_h, Color::new(0.12, 0.09, 0.03, 0.86));
        draw_rectangle_lines(x, y, tag_w, tag_h, 1.0, dark::WARNING);
        draw_centered_text(tag, x + tag_w * 0.5, y + 14.0, 12.0, dark::WARNING);
    }

    pub(super) fn draw_wave_start_flash(&self) {
        if self.wave_flash_timer <= 0.0 {
            return;
        }
        let alpha = (self.wave_flash_timer / self.constants.ui.wave_flash_duration).clamp(0.0, 1.0);
        let t = &text().hud;
        let flash = if self.shutdown_triggered {
            t.beacon_shutdown_flash.clone()
        } else {
            fill(&t.wave_flash, &[("n", &self.last_wave_started.to_string())])
        };
        let w = 300.0;
        let h = 46.0;
        let x = (screen_width() - w) * 0.5;
        let y = self.constants.ui.hud_height + 14.0;
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.07, 0.06, 0.84 * alpha));
        draw_rectangle_lines(x, y, w, h, 2.0, Color::new(1.0, 0.58, 0.18, 0.9 * alpha));
        draw_ui_text(
            &t.incoming,
            x + 18.0,
            y + 17.0,
            11.0,
            Color::new(1.0, 0.72, 0.34, alpha),
        );
        let dims = measure_ui_text(&flash, None, 24, 1.0);
        draw_ui_text(
            &flash,
            x + (w - dims.width) * 0.5,
            y + 34.0,
            24.0,
            Color::new(1.0, 0.9, 0.7, alpha),
        );
    }

    pub(super) fn draw_notifications(&self) {
        let config = NotificationRenderConfig {
            width: 340.0,
            row_height: 30.0,
            spacing: 4.0,
            padding: 8.0,
            font_size: 16.0,
            background: Color::new(0.08, 0.08, 0.1, 0.9),
            text_color: dark::TEXT_BRIGHT,
            ..Default::default()
        };
        let mut y = self.constants.ui.hud_height + 28.0;
        for note in &self.notifications {
            let notification = Notification {
                message: note.text.clone(),
                notification_type: NotificationType::Info,
                time_remaining: note.ttl,
                total_duration: self.constants.ui.notification_ttl,
            };
            draw_notification(
                &notification,
                super::ui::EDGE_RAIL_MARGIN + super::ui::EDGE_RAIL_W + 14.0,
                y,
                &config,
            );
            y += config.row_height + config.spacing;
        }
    }
}

/// If the mouse is over `anchor`, explain the action without keeping a help
/// card on screen permanently.
pub(super) fn draw_button_hint(anchor: Rect, title: &str, body: &str) {
    let (mx, my) = mouse_position();
    let hovered =
        mx >= anchor.x && mx <= anchor.x + anchor.w && my >= anchor.y && my <= anchor.y + anchor.h;
    if !hovered {
        return;
    }
    let w = 232.0;
    let h = 48.0;
    let x = (anchor.x + anchor.w - w).clamp(6.0, screen_width() - w - 6.0);
    let y = anchor.y + anchor.h + 6.0;
    draw_rectangle(x, y, w, h, Color::new(0.03, 0.05, 0.06, 0.98));
    draw_rectangle_lines(x, y, w, h, 1.2, Color::new(0.22, 0.52, 0.52, 0.85));
    draw_ui_text(title, x + 9.0, y + 17.0, 12.0, dark::TEXT_BRIGHT);
    ui::draw_bounded_text(body, x + 9.0, y + 34.0, w - 18.0, 10.0, dark::TEXT_DIM);
}

fn draw_centered_text(text: &str, center_x: f32, baseline_y: f32, font_size: f32, color: Color) {
    let dims = measure_ui_text(text, None, font_size as u16, 1.0);
    draw_ui_text(
        text,
        center_x - dims.width * 0.5,
        baseline_y,
        font_size,
        color,
    );
}
