//! The top HUD strip: survival/power/threat zones, the objective banner,
//! alert cards, and (via [`beacon_panel`]) the beacon control panel.

mod beacon_panel;

use crate::data::strings::{fill, text};
use crate::data::GameData;
use crate::engine::population::WorkforcePolicy;
use crate::ui::{self, ConsoleButtonState, ConsoleIcon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::math::pulse_range;
use macroquad_toolkit::notifications::{
    draw_notification, Notification, NotificationRenderConfig, NotificationType,
};

use super::helpers::threat_color;
use super::ui_advice::{AdviceTarget, AlertBanner, AlertSeverity, PowerGridSnapshot, UiAdvice};
use super::GameplayState;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl GameplayState {
    pub(super) fn draw_hud(&mut self, data: &GameData) {
        let advice = self.build_ui_advice(data);
        let top_h = self.constants.ui.top_bar_h;
        let strip_h = self.constants.ui.command_strip_h;
        let hud_h = self.constants.ui.hud_height;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            hud_h,
            Color::new(0.025, 0.032, 0.037, 0.97),
        );
        draw_rectangle(
            0.0,
            hud_h - 3.0,
            screen_width(),
            3.0,
            Color::new(0.16, 0.34, 0.34, 0.85),
        );

        let margin = 10.0;
        let beacon_w = (screen_width() * 0.27).clamp(330.0, 410.0);
        let beacon_x = screen_width() - beacon_w - margin;
        let left_w = beacon_x - margin * 2.0;
        let zone_gap = 8.0;
        let usable_top_w = (left_w - zone_gap * 2.0).max(1.0);
        let (survival_w, power_w, threat_w) = if usable_top_w < 740.0 {
            (
                usable_top_w * 0.40,
                usable_top_w * 0.34,
                usable_top_w * 0.26,
            )
        } else {
            let survival_w = (usable_top_w * 0.40).clamp(320.0, 430.0);
            let threat_w = (usable_top_w * 0.24).clamp(220.0, 285.0);
            let power_w = (usable_top_w - survival_w - threat_w).max(260.0);
            (survival_w, power_w, threat_w)
        };

        let survival_rect = Rect::new(margin, margin, survival_w, top_h - margin * 1.4);
        let power_rect = Rect::new(
            survival_rect.x + survival_rect.w + zone_gap,
            margin,
            power_w,
            top_h - margin * 1.4,
        );
        let threat_rect = Rect::new(
            power_rect.x + power_rect.w + zone_gap,
            margin,
            threat_w,
            top_h - margin * 1.4,
        );

        self.draw_survival_zone(survival_rect);
        self.draw_power_zone(power_rect, &advice.power);
        self.draw_threat_zone(threat_rect, &advice);

        let strip_y = top_h + 4.0;
        let objective_max = (left_w - 230.0).max(260.0);
        let objective_w = (left_w * 0.58).clamp(260.0, objective_max);
        let objective_rect = Rect::new(margin, strip_y, objective_w, strip_h - 8.0);
        self.draw_objective_strip(objective_rect, &advice, data);

        let alert_x = objective_rect.x + objective_rect.w + zone_gap;
        let alert_w = (beacon_x - alert_x - margin).max(160.0);
        self.draw_alert_banners(alert_x, strip_y, alert_w, strip_h - 8.0, &advice.alerts);

        self.draw_beacon_panel(Rect::new(beacon_x, margin, beacon_w, hud_h - margin * 1.4));
    }

    fn draw_survival_zone(&mut self, rect: Rect) {
        ui::draw_console_panel(rect, Color::new(0.18, 0.38, 0.36, 0.82));
        let t = &text().hud;
        ui::draw_console_header(
            rect.x + 12.0,
            rect.y + 18.0,
            &t.survival,
            self.factory.phase.label(),
            dark::POSITIVE,
        );
        let workforce = &text().workforce;
        let mode = match self.population.workforce_policy {
            WorkforcePolicy::Sustain => &workforce.sustain,
            WorkforcePolicy::Balanced => &workforce.balanced,
            WorkforcePolicy::Salvage => &workforce.salvage,
            WorkforcePolicy::Defense => &workforce.defense,
        };
        let workforce_interactive = !self.show_intro
            && !self.show_workforce
            && self.salvage_report.is_none()
            && !self.paused;
        if ui::draw_console_button(
            rect.x + rect.w - 142.0,
            rect.y + 8.0,
            130.0,
            24.0,
            &fill(&workforce.hud_button, &[("mode", mode)]),
            if workforce_interactive {
                ConsoleButtonState::Affordable
            } else {
                ConsoleButtonState::Disabled
            },
        ) && workforce_interactive
        {
            self.show_workforce = true;
        }

        let y = rect.y + 49.0;
        let col_w = rect.w / 5.0;
        draw_metric_icon(
            ConsoleIcon::People,
            rect.x + 12.0,
            y,
            &t.pop,
            &fill(
                &t.pop_value,
                &[
                    ("count", &self.population.count.to_string()),
                    ("max", &self.shelter_capacity().to_string()),
                ],
            ),
            dark::TEXT_BRIGHT,
        );
        draw_metric_icon(
            ConsoleIcon::Food,
            rect.x + col_w,
            y,
            &t.food,
            &format!("{:.0}", self.population.food_supply),
            dark::TEXT_BRIGHT,
        );
        let morale_color = if self.population.morale < 25.0 {
            dark::NEGATIVE
        } else if self.population.morale < 50.0 {
            dark::WARNING
        } else {
            dark::POSITIVE
        };
        draw_metric_icon(
            ConsoleIcon::Water,
            rect.x + col_w * 2.0,
            y,
            &t.water,
            &format!("{:.0}", self.resources.water),
            dark::TEXT_BRIGHT,
        );
        draw_metric_icon(
            ConsoleIcon::Morale,
            rect.x + col_w * 3.0,
            y,
            &t.morale,
            &format!("{:.0}%", self.population.morale),
            morale_color,
        );
        let health_color = if self.population.health < 35.0 {
            dark::NEGATIVE
        } else {
            dark::POSITIVE
        };
        draw_metric_icon(
            ConsoleIcon::Health,
            rect.x + col_w * 4.0,
            y,
            &t.health,
            &format!("{:.0}%", self.population.health),
            health_color,
        );
    }

    fn draw_power_zone(&self, rect: Rect, power: &PowerGridSnapshot) {
        let accent = if power.offline_towers > 0 || power.net < 0.0 {
            dark::WARNING
        } else {
            dark::POSITIVE
        };
        ui::draw_console_panel(rect, Color::new(accent.r, accent.g, accent.b, 0.82));
        let t = &text().hud;
        ui::draw_console_header(rect.x + 12.0, rect.y + 18.0, &t.power_grid, "", accent);

        let number_y = rect.y + 45.0;
        let col_w = rect.w / 4.0;
        draw_small_metric(
            rect.x + 12.0,
            number_y,
            &t.gen,
            &format!("{:.0}", power.generated),
            dark::POSITIVE,
        );
        draw_small_metric(
            rect.x + col_w,
            number_y,
            &t.used,
            &format!("{:.0}", power.used),
            dark::WARNING,
        );
        let net_color = if power.net >= 0.0 {
            dark::POSITIVE
        } else {
            dark::NEGATIVE
        };
        draw_small_metric(
            rect.x + col_w * 2.0,
            number_y,
            &t.net,
            &format!("{:+.0}/s", power.net),
            net_color,
        );
        draw_small_metric(
            rect.x + col_w * 3.0,
            number_y,
            &t.battery,
            &format!("{:.0}", power.battery),
            dark::TEXT_BRIGHT,
        );

        let bar_x = rect.x + 12.0;
        let bar_y = rect.y + 60.0;
        ui::draw_resource_bar(
            bar_x,
            bar_y,
            rect.w - 24.0,
            8.0,
            power.battery,
            self.constants.economy.power_cap,
            net_color,
        );
        if power.offline_towers > 0 {
            draw_ui_text(
                &fill(&t.offline, &[("n", &power.offline_towers.to_string())]),
                bar_x,
                bar_y - 8.0,
                9.0,
                dark::NEGATIVE,
            );
        }
    }

    fn draw_threat_zone(&self, rect: Rect, advice: &UiAdvice) {
        ui::draw_console_panel(rect, Color::new(0.48, 0.2, 0.16, 0.85));
        let t = &text().hud;
        ui::draw_console_header(rect.x + 12.0, rect.y + 18.0, &t.threat, "", dark::WARNING);

        // What is drawing the most machine attention right now — the signal the
        // player can act on to go quieter. Right-aligned in the header row.
        if let Some(loudest) = self.loudest_threat_label() {
            let line = fill(&t.loudest, &[("label", loudest)]);
            let w = measure_ui_text(&line, None, 10, 1.0).width;
            draw_ui_text(
                &line,
                rect.x + rect.w - 12.0 - w,
                rect.y + 16.0,
                10.0,
                threat_color(&self.threat, &self.constants.threat),
            );
        }

        // Two lines only: wave status, then what the next/current wave is made
        // of. The awareness tier already lives in the beacon panel — repeating
        // it here was clutter (and the third line overlapped the second).
        let (status, status_color) = if !self.beacon_active && !self.shutdown_triggered {
            (t.beacon_offline.clone(), dark::TEXT_DIM)
        } else if self.between_waves {
            (
                fill(
                    &t.wave_incoming,
                    &[
                        ("wave", &(self.current_wave + 1).to_string()),
                        ("secs", &format!("{:.0}", self.wave_timer.max(0.0))),
                    ],
                ),
                dark::TEXT_BRIGHT,
            )
        } else {
            (
                fill(
                    &t.wave_active,
                    &[
                        ("wave", &self.current_wave.to_string()),
                        ("alive", &self.wave_manager.alive_count().to_string()),
                    ],
                ),
                dark::TEXT_BRIGHT,
            )
        };
        ui::draw_bounded_text(
            &status,
            rect.x + 12.0,
            rect.y + 45.0,
            rect.w - 24.0,
            15.0,
            status_color,
        );

        // Lead the composition with the machines' current adaptation so the
        // player reads the qualitative shift ("saboteurs incoming") alongside
        // the raw counts.
        let composition = super::ui_advice::format_enemy_counts(&advice.wave_preview.counts);
        let (line, line_color) = match self.adaptation_incoming_label() {
            Some(note) => (
                fill(
                    &t.adaptation_composition,
                    &[("note", note), ("composition", &composition)],
                ),
                dark::WARNING,
            ),
            None => (composition, dark::TEXT_DIM),
        };
        ui::draw_bounded_text(
            &line,
            rect.x + 12.0,
            rect.y + 64.0,
            rect.w - 24.0,
            11.0,
            line_color,
        );
    }

    fn draw_objective_strip(&mut self, rect: Rect, advice: &UiAdvice, data: &GameData) {
        ui::draw_console_panel(rect, Color::new(0.2, 0.42, 0.64, 0.9));
        let t = &text().hud;
        draw_ui_text(
            &t.next_step,
            rect.x + 12.0,
            rect.y + 16.0,
            11.0,
            Color::new(0.56, 0.82, 1.0, 1.0),
        );
        ui::draw_bounded_text(
            &advice.suggested_action.label,
            rect.x + 12.0,
            rect.y + 35.0,
            rect.w - 124.0,
            17.0,
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
            rect.x + 12.0,
            rect.y + 46.0,
            rect.w - 124.0,
            11.0,
            dark::TEXT_DIM,
        );

        let btn_state = if matches!(advice.suggested_action.target, AdviceTarget::None) {
            ConsoleButtonState::Disabled
        } else {
            ConsoleButtonState::Recommended
        };
        if ui::draw_console_button(
            rect.x + rect.w - 106.0,
            rect.y + 12.0,
            92.0,
            31.0,
            &t.focus,
            btn_state,
        ) {
            let target = advice.suggested_action.target.clone();
            self.apply_advice_focus(&target, data);
        }

        // Hovering the strip explains the suggestion (the old bottom panel's
        // detail line, now on demand instead of always on screen).
        draw_button_hint(
            Rect::new(rect.x, rect.y, rect.w - 110.0, rect.h),
            &advice.suggested_action.label,
            &advice.suggested_action.detail,
        );
    }

    fn draw_alert_banners(&self, x: f32, y: f32, w: f32, h: f32, alerts: &[AlertBanner]) {
        let max_visible = self.constants.ui.alert_max_visible.min(alerts.len());
        if max_visible == 0 {
            // Nothing to report — draw nothing. An "ALL CLEAR" box was just
            // one more panel competing for attention.
            return;
        }

        let gap = 6.0;
        let each_w = (w - gap * (max_visible.saturating_sub(1) as f32)) / max_visible as f32;
        for (i, alert) in alerts.iter().take(max_visible).enumerate() {
            let ax = x + i as f32 * (each_w + gap);
            let color = match alert.severity {
                AlertSeverity::Critical => dark::NEGATIVE,
                AlertSeverity::Warning => dark::WARNING,
            };
            let pulse = if alert.severity == AlertSeverity::Critical {
                pulse_range(7.0, 0.72, 0.92)
            } else {
                0.82
            };
            ui::draw_console_panel(
                Rect::new(ax, y, each_w, h),
                Color::new(color.r, color.g, color.b, pulse),
            );
            ui::draw_bounded_text(
                &alert.label,
                ax + 10.0,
                y + 21.0,
                each_w - 20.0,
                13.0,
                color,
            );
            ui::draw_bounded_text(
                &alert.detail,
                ax + 10.0,
                y + 40.0,
                each_w - 20.0,
                11.0,
                dark::TEXT_BRIGHT,
            );
        }
    }

    pub(super) fn draw_wave_status(&self) {
        if self.time_scale <= 1.0 {
            return;
        }

        let tag = &text().hud.speed_tag;
        let tag_w = 34.0;
        let tag_h = 20.0;
        let x = screen_width() - self.constants.ui.sector_panel_w - tag_w - 12.0;
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
        let x = self.constants.ui.build_panel_w
            + (screen_width()
                - self.constants.ui.build_panel_w
                - self.constants.ui.sector_panel_w
                - w)
                * 0.5;
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
                self.constants.ui.build_panel_w + 10.0,
                y,
                &config,
            );
            y += config.row_height + config.spacing;
        }
    }
}

fn draw_metric_icon(icon: ConsoleIcon, x: f32, y: f32, label: &str, value: &str, color: Color) {
    ui::draw_icon(
        icon,
        x,
        y - 20.0,
        20.0,
        Color::new(color.r, color.g, color.b, 0.86),
    );
    draw_ui_text(label, x + 25.0, y - 10.0, 10.0, dark::TEXT_DIM);
    draw_ui_text(value, x + 25.0, y + 11.0, 19.0, color);
}

fn draw_small_metric(x: f32, y: f32, label: &str, value: &str, color: Color) {
    draw_ui_text(label, x, y - 10.0, 10.0, dark::TEXT_DIM);
    draw_ui_text(value, x, y + 12.0, 18.0, color);
}

/// If the mouse is over `anchor`, draw a small explanatory tooltip just below
/// it. Used to spell out what the beacon-control buttons actually do.
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
