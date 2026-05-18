use crate::data::GameData;
use crate::ui::{self, ConsoleButtonState, ConsoleIcon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::notifications::{
    draw_notification, Notification, NotificationRenderConfig, NotificationType,
};

use super::helpers::{beacon_color, threat_color};
use super::ui_advice::{AdviceTarget, AlertBanner, AlertSeverity, PowerGridSnapshot, UiAdvice};
use super::GameplayState;

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
                usable_top_w * 0.34,
                usable_top_w * 0.38,
                usable_top_w * 0.28,
            )
        } else {
            let survival_w = (usable_top_w * 0.34).clamp(280.0, 390.0);
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

    fn draw_survival_zone(&self, rect: Rect) {
        ui::draw_console_panel(rect, Color::new(0.18, 0.38, 0.36, 0.82));
        ui::draw_console_header(rect.x + 12.0, rect.y + 18.0, "SURVIVAL", "", dark::POSITIVE);
        draw_bounded_text(
            self.factory.phase.label(),
            rect.x + rect.w - 112.0,
            rect.y + 18.0,
            100.0,
            10.0,
            dark::TEXT_DIM,
        );

        let y = rect.y + 49.0;
        let col_w = rect.w / 4.0;
        draw_metric_icon(
            ConsoleIcon::People,
            rect.x + 12.0,
            y,
            "POP",
            &format!(
                "{}/{}",
                self.population.count,
                self.constants.starting.population + 12
            ),
            dark::TEXT_BRIGHT,
        );
        draw_metric_icon(
            ConsoleIcon::Food,
            rect.x + col_w,
            y,
            "FOOD",
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
            ConsoleIcon::Morale,
            rect.x + col_w * 2.0,
            y,
            "MORALE",
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
            rect.x + col_w * 3.0,
            y,
            "HEALTH",
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
        ui::draw_console_header(rect.x + 12.0, rect.y + 18.0, "POWER GRID", "", accent);

        let number_y = rect.y + 45.0;
        let col_w = rect.w / 4.0;
        draw_small_metric(
            rect.x + 12.0,
            number_y,
            "GEN",
            &format!("{:.0}", power.generated),
            dark::POSITIVE,
        );
        draw_small_metric(
            rect.x + col_w,
            number_y,
            "USED",
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
            "NET",
            &format!("{:+.0}/s", power.net),
            net_color,
        );
        draw_small_metric(
            rect.x + col_w * 3.0,
            number_y,
            "BATTERY",
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
            draw_text(
                &format!("{} OFFLINE", power.offline_towers),
                bar_x,
                bar_y - 8.0,
                9.0,
                dark::NEGATIVE,
            );
        }
    }

    fn draw_threat_zone(&self, rect: Rect, advice: &UiAdvice) {
        ui::draw_console_panel(rect, Color::new(0.48, 0.2, 0.16, 0.85));
        ui::draw_console_header(rect.x + 12.0, rect.y + 18.0, "THREAT", "", dark::WARNING);

        let status = if self.between_waves {
            format!(
                "Wave {} in {:.0}s",
                self.current_wave + 1,
                self.wave_timer.max(0.0)
            )
        } else {
            format!(
                "Wave {}: {} alive",
                self.current_wave,
                self.wave_manager.alive_count()
            )
        };
        draw_bounded_text(
            &status,
            rect.x + 12.0,
            rect.y + 45.0,
            rect.w - 24.0,
            15.0,
            dark::TEXT_BRIGHT,
        );
        draw_bounded_text(
            &format!(
                "{} ({:.0}%)",
                self.threat.reaction_tier().label(),
                self.threat.awareness_level()
            ),
            rect.x + 12.0,
            rect.y + 64.0,
            rect.w - 24.0,
            12.0,
            threat_color(&self.threat),
        );
        draw_bounded_text(
            &advice.wave_preview.recommendation,
            rect.x + 12.0,
            rect.y + rect.h - 7.0,
            rect.w - 24.0,
            11.0,
            dark::TEXT_DIM,
        );
    }

    fn draw_objective_strip(&mut self, rect: Rect, advice: &UiAdvice, data: &GameData) {
        ui::draw_console_panel(rect, Color::new(0.2, 0.42, 0.64, 0.9));
        draw_text(
            "NEXT STEP",
            rect.x + 12.0,
            rect.y + 16.0,
            11.0,
            Color::new(0.56, 0.82, 1.0, 1.0),
        );
        draw_bounded_text(
            &advice.suggested_action.label,
            rect.x + 12.0,
            rect.y + 35.0,
            rect.w - 124.0,
            17.0,
            dark::TEXT_BRIGHT,
        );
        draw_bounded_text(
            &format!("{}    {}", advice.suggested_action.cost, advice.risk),
            rect.x + 12.0,
            rect.y + 50.0,
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
            "FOCUS",
            btn_state,
        ) {
            let target = advice.suggested_action.target.clone();
            self.apply_advice_focus(&target, data);
        }
    }

    fn draw_alert_banners(&self, x: f32, y: f32, w: f32, h: f32, alerts: &[AlertBanner]) {
        let max_visible = self.constants.ui.alert_max_visible.min(alerts.len());
        if max_visible == 0 {
            let rect = Rect::new(x, y, w, h);
            ui::draw_console_panel(rect, Color::new(0.2, 0.32, 0.34, 0.5));
            draw_text("ALL CLEAR", x + 12.0, y + 22.0, 13.0, dark::TEXT_DIM);
            draw_text(
                "Hold resources or start a controlled draw",
                x + 12.0,
                y + 41.0,
                11.0,
                dark::TEXT_DIM,
            );
            return;
        }

        let gap = 6.0;
        let each_w = (w - gap * (max_visible.saturating_sub(1) as f32)) / max_visible as f32;
        for (i, alert) in alerts.iter().take(max_visible).enumerate() {
            let ax = x + i as f32 * (each_w + gap);
            let color = match alert.severity {
                AlertSeverity::Critical => dark::NEGATIVE,
                AlertSeverity::Warning => dark::WARNING,
                AlertSeverity::Info => dark::ACCENT,
            };
            let pulse = if alert.severity == AlertSeverity::Critical {
                0.72 + 0.2 * (get_time() as f32 * 7.0).sin().abs()
            } else {
                0.82
            };
            ui::draw_console_panel(
                Rect::new(ax, y, each_w, h),
                Color::new(color.r, color.g, color.b, pulse),
            );
            draw_bounded_text(
                &alert.label,
                ax + 10.0,
                y + 21.0,
                each_w - 20.0,
                13.0,
                color,
            );
            draw_bounded_text(
                &alert.detail,
                ax + 10.0,
                y + 40.0,
                each_w - 20.0,
                11.0,
                dark::TEXT_BRIGHT,
            );
        }
    }

    fn draw_beacon_panel(&mut self, rect: Rect) {
        let phase_color = beacon_color(&self.beacon_phase);
        let danger = match self.beacon_phase {
            crate::engine::beacon::BeaconPhase::WarmSignal => 0.24,
            crate::engine::beacon::BeaconPhase::SustainedCall => 0.48,
            crate::engine::beacon::BeaconPhase::ScreamingBeacon => 0.72,
            crate::engine::beacon::BeaconPhase::TerminalHowl => 1.0,
        };
        let pulse = 0.68 + 0.18 * (get_time() as f32 * (4.0 + danger * 4.0)).sin().abs();
        ui::draw_console_panel(rect, Color::new(0.78, 0.16, 0.12, pulse));
        draw_text(
            "BEACON CONTROL",
            rect.x + 14.0,
            rect.y + 18.0,
            11.0,
            dark::NEGATIVE,
        );
        draw_bounded_text(
            &self.beacon_phase.label().to_uppercase(),
            rect.x + 14.0,
            rect.y + 43.0,
            rect.w - 28.0,
            24.0,
            phase_color,
        );

        draw_segmented_meter(
            rect.x + 14.0,
            rect.y + 55.0,
            rect.w - 28.0,
            11.0,
            danger,
            phase_color,
        );
        draw_bounded_text(
            &format!("Next phase: +{:.0} draw", self.beacon_draw_to_next_phase()),
            rect.x + 14.0,
            rect.y + 78.0,
            rect.w * 0.48,
            11.0,
            dark::TEXT_DIM,
        );
        draw_bounded_text(
            &format!(
                "{} {:.0}",
                self.threat.reaction_tier().label(),
                self.threat.awareness_level()
            ),
            rect.x + rect.w * 0.5,
            rect.y + 78.0,
            rect.w * 0.45,
            11.0,
            threat_color(&self.threat),
        );

        if self.beacon_active {
            self.draw_scavenger_status_cards(rect);
        } else {
            draw_bounded_text(
                &format!(
                    "Teams ready: {}",
                    self.constants.scavenger.initial_scavengers
                ),
                rect.x + 14.0,
                rect.y + 100.0,
                rect.w - 28.0,
                12.0,
                dark::TEXT_BRIGHT,
            );
        }

        let btn_y = rect.y + rect.h - 29.0;
        if !self.beacon_active {
            if ui::draw_console_button(
                rect.x + 14.0,
                btn_y,
                rect.w - 28.0,
                24.0,
                "START BEACON",
                ConsoleButtonState::Dangerous,
            ) {
                self.start_beacon();
            }
            return;
        }

        let left_w = (rect.w - 34.0) * 0.5;
        if self.scavengers_out > 0 && !self.scavenger_recall_active {
            if ui::draw_console_button(
                rect.x + 14.0,
                btn_y,
                left_w,
                24.0,
                "RECALL",
                ConsoleButtonState::Recommended,
            ) {
                self.scavenger_recall_active = true;
                self.scavenger_recall_timer = 0.0;
            }
        } else {
            ui::draw_console_button(
                rect.x + 14.0,
                btn_y,
                left_w,
                24.0,
                "RECALL",
                ConsoleButtonState::Disabled,
            );
        }

        let shutdown_state = if self.shutdown_triggered || self.current_wave < 1 {
            ConsoleButtonState::Disabled
        } else {
            ConsoleButtonState::Dangerous
        };
        let shutdown_label = if self.current_wave < 1 {
            "SHUTDOWN LOCKED"
        } else {
            "SHUTDOWN"
        };
        if ui::draw_console_button(
            rect.x + 20.0 + left_w,
            btn_y,
            left_w,
            24.0,
            shutdown_label,
            shutdown_state,
        ) {
            self.trigger_shutdown();
        }
    }

    fn draw_scavenger_status_cards(&self, rect: Rect) {
        let card_y = rect.y + 90.0;
        let card_h = 20.0;
        let gap = 5.0;
        let card_w = (rect.w - 28.0 - gap * 2.0) / 3.0;
        for i in 0..3 {
            let x = rect.x + 14.0 + i as f32 * (card_w + gap);
            draw_rectangle(
                x,
                card_y,
                card_w,
                card_h,
                Color::new(0.08, 0.07, 0.07, 0.78),
            );
            draw_rectangle_lines(
                x,
                card_y,
                card_w,
                card_h,
                1.0,
                Color::new(0.5, 0.18, 0.14, 0.5),
            );
            let text = if !self.beacon_active {
                if i == 0 {
                    format!("Ready {}", self.constants.scavenger.initial_scavengers)
                } else {
                    "Standby".to_string()
                }
            } else if self.scavenger_recall_active {
                if i == 0 {
                    let remaining = (self.constants.scavenger.recall_interval
                        - self.scavenger_recall_timer)
                        .max(0.0);
                    format!("Return {:.0}s", remaining)
                } else {
                    "Awaiting".to_string()
                }
            } else if i < self.scavengers_out as usize {
                if self.beacon_phase.rank() >= 2 {
                    "At Risk".to_string()
                } else {
                    "Out".to_string()
                }
            } else {
                "Home".to_string()
            };
            draw_bounded_text(
                &text,
                x + 6.0,
                card_y + 14.0,
                card_w - 12.0,
                10.0,
                dark::TEXT_BRIGHT,
            );
        }
    }

    pub(super) fn draw_wave_status(&self) {
        if self.time_scale <= 1.0 {
            return;
        }

        let text = "2x";
        let tag_w = 34.0;
        let tag_h = 20.0;
        let x = screen_width() - self.constants.ui.sector_panel_w - tag_w - 12.0;
        let y = self.constants.ui.hud_height + 8.0;
        draw_rectangle(x, y, tag_w, tag_h, Color::new(0.12, 0.09, 0.03, 0.86));
        draw_rectangle_lines(x, y, tag_w, tag_h, 1.0, dark::WARNING);
        draw_centered_text(text, x + tag_w * 0.5, y + 14.0, 12.0, dark::WARNING);
    }

    pub(super) fn draw_wave_start_flash(&self) {
        if self.wave_flash_timer <= 0.0 {
            return;
        }
        let alpha = (self.wave_flash_timer / self.constants.ui.wave_flash_duration).clamp(0.0, 1.0);
        let text = if self.shutdown_triggered {
            "BEACON SHUTDOWN".to_string()
        } else {
            format!("WAVE {}", self.last_wave_started)
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
        draw_text(
            "INCOMING",
            x + 18.0,
            y + 17.0,
            11.0,
            Color::new(1.0, 0.72, 0.34, alpha),
        );
        let dims = measure_text(&text, None, 24, 1.0);
        draw_text(
            &text,
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
    draw_text(label, x + 25.0, y - 10.0, 10.0, dark::TEXT_DIM);
    draw_text(value, x + 25.0, y + 11.0, 19.0, color);
}

fn draw_small_metric(x: f32, y: f32, label: &str, value: &str, color: Color) {
    draw_text(label, x, y - 10.0, 10.0, dark::TEXT_DIM);
    draw_text(value, x, y + 12.0, 18.0, color);
}

fn draw_segmented_meter(x: f32, y: f32, w: f32, h: f32, value: f32, color: Color) {
    let segments = 14;
    let gap = 3.0;
    let seg_w = (w - gap * (segments as f32 - 1.0)) / segments as f32;
    let filled = (value * segments as f32).ceil() as i32;
    for i in 0..segments {
        let sx = x + i as f32 * (seg_w + gap);
        let active = i < filled;
        let fill = if active {
            color
        } else {
            Color::new(0.18, 0.09, 0.08, 0.78)
        };
        draw_rectangle(sx, y, seg_w, h, fill);
    }
}

fn draw_bounded_text(text: &str, x: f32, y: f32, max_w: f32, size: f32, color: Color) {
    let bounded = truncate_to_width(text, max_w, size as u16);
    draw_text(&bounded, x, y, size, color);
}

fn truncate_to_width(text: &str, max_w: f32, font_size: u16) -> String {
    if measure_text(text, None, font_size, 1.0).width <= max_w {
        return text.to_string();
    }

    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}...", out, ch);
        if measure_text(&candidate, None, font_size, 1.0).width > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{}...", out)
}

fn draw_centered_text(text: &str, center_x: f32, baseline_y: f32, font_size: f32, color: Color) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(
        text,
        center_x - dims.width * 0.5,
        baseline_y,
        font_size,
        color,
    );
}
