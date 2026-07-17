//! The beacon control panel: phase read-out, the evacuation ledger, scavenger
//! status cards, and the raise / recall / shutdown buttons.

use crate::data::strings::{fill, text};
use crate::engine::beacon::BeaconPhase;
use crate::ui::{self, ConsoleButtonState};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::math::pulse_range;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::super::helpers::{beacon_color, threat_color};
use super::super::GameplayState;
use super::draw_button_hint;

impl GameplayState {
    pub(super) fn draw_beacon_panel(&mut self, rect: Rect) {
        let t = &text().hud;
        let phase_color = beacon_color(&self.beacon_phase);
        let danger = match self.beacon_phase {
            BeaconPhase::WarmSignal => 0.24,
            BeaconPhase::SustainedCall => 0.48,
            BeaconPhase::ScreamingBeacon => 0.72,
            BeaconPhase::TerminalHowl => 1.0,
        };
        let pulse = pulse_range(4.0 + danger * 4.0, 0.68, 0.86);
        ui::draw_console_panel(rect, Color::new(0.78, 0.16, 0.12, pulse));
        draw_ui_text(
            &t.beacon_control,
            rect.x + 14.0,
            rect.y + 18.0,
            11.0,
            dark::NEGATIVE,
        );

        // Sacrifice ledger, always visible: total survivors evacuated so far,
        // plus the un-banked count the current beacon window is earning (lost if
        // the factory falls before shutdown).
        let evac_text = if self.beacon_active {
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
        let evac_w = measure_ui_text(&evac_text, None, 11, 1.0).width;
        draw_ui_text(
            &evac_text,
            rect.x + rect.w - 14.0 - evac_w,
            rect.y + 18.0,
            11.0,
            dark::POSITIVE,
        );
        ui::draw_bounded_text(
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
        ui::draw_bounded_text(
            &fill(
                &t.next_phase,
                &[("draw", &format!("{:.0}", self.beacon_draw_to_next_phase()))],
            ),
            rect.x + 14.0,
            rect.y + 78.0,
            rect.w * 0.48,
            11.0,
            dark::TEXT_DIM,
        );
        ui::draw_bounded_text(
            &fill(
                &t.threat_readout,
                &[
                    (
                        "tier",
                        self.threat.reaction_tier(&self.constants.threat).label(),
                    ),
                    ("pct", &format!("{:.0}", self.threat.awareness_level())),
                ],
            ),
            rect.x + rect.w * 0.5,
            rect.y + 78.0,
            rect.w * 0.45,
            11.0,
            threat_color(&self.threat, &self.constants.threat),
        );

        if self.beacon_active {
            self.draw_scavenger_status_cards(rect);
        } else {
            ui::draw_bounded_text(
                &fill(
                    &t.teams_ready,
                    &[(
                        "n",
                        &self.constants.scavenger.initial_scavengers.to_string(),
                    )],
                ),
                rect.x + 14.0,
                rect.y + 100.0,
                rect.w - 28.0,
                12.0,
                dark::TEXT_BRIGHT,
            );
        }

        // Buttons behind a modal (salvage report) or the pause menu must not
        // register clicks.
        let interactive = self.salvage_report.is_none() && !self.paused;

        let btn_y = rect.y + rect.h - 29.0;
        if !self.beacon_active {
            // While the salvage report modal is up the beacon can't be re-raised
            // from the panel behind it; the player rebuilds first.
            let start_state = if interactive {
                ConsoleButtonState::Dangerous
            } else {
                ConsoleButtonState::Disabled
            };
            let start_rect = Rect::new(rect.x + 14.0, btn_y, rect.w - 28.0, 24.0);
            if ui::draw_console_button(
                start_rect.x,
                start_rect.y,
                start_rect.w,
                start_rect.h,
                &t.start_beacon,
                start_state,
            ) && interactive
            {
                self.start_beacon();
            }
            draw_button_hint(start_rect, &t.start_beacon_tip, &t.start_beacon_tip_detail);
            return;
        }

        let left_w = (rect.w - 34.0) * 0.5;
        let recall_rect = Rect::new(rect.x + 14.0, btn_y, left_w, 24.0);
        if interactive && self.scavengers_out > 0 && !self.scavenger_recall_active {
            if ui::draw_console_button(
                recall_rect.x,
                recall_rect.y,
                recall_rect.w,
                recall_rect.h,
                &t.recall,
                ConsoleButtonState::Recommended,
            ) {
                self.scavenger_recall_active = true;
                self.scavenger_recall_timer = 0.0;
            }
        } else {
            ui::draw_console_button(
                recall_rect.x,
                recall_rect.y,
                recall_rect.w,
                recall_rect.h,
                &t.recall,
                ConsoleButtonState::Disabled,
            );
        }
        draw_button_hint(recall_rect, &t.recall_tip, &t.recall_tip_detail);

        let shutdown_state = if self.shutdown_triggered || self.current_wave < 1 || !interactive {
            ConsoleButtonState::Disabled
        } else {
            ConsoleButtonState::Dangerous
        };
        let shutdown_label = if self.current_wave < 1 {
            &t.shutdown_locked
        } else {
            &t.shutdown
        };
        let shutdown_rect = Rect::new(rect.x + 20.0 + left_w, btn_y, left_w, 24.0);
        if ui::draw_console_button(
            shutdown_rect.x,
            shutdown_rect.y,
            shutdown_rect.w,
            shutdown_rect.h,
            shutdown_label,
            shutdown_state,
        ) && interactive
        {
            self.trigger_shutdown();
        }
        draw_button_hint(shutdown_rect, &t.shutdown_tip, &t.shutdown_tip_detail);
    }

    fn draw_scavenger_status_cards(&self, rect: Rect) {
        let t = &text().hud;
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
            let card = if !self.beacon_active {
                if i == 0 {
                    fill(
                        &t.card_ready,
                        &[(
                            "n",
                            &self.constants.scavenger.initial_scavengers.to_string(),
                        )],
                    )
                } else {
                    t.card_standby.clone()
                }
            } else if self.scavenger_recall_active {
                if i == 0 {
                    let remaining = (self.constants.scavenger.recall_interval
                        - self.scavenger_recall_timer)
                        .max(0.0);
                    fill(&t.card_return, &[("secs", &format!("{remaining:.0}"))])
                } else {
                    t.card_awaiting.clone()
                }
            } else if i < self.scavengers_out as usize {
                if self.beacon_phase.rank() >= 2 {
                    t.card_at_risk.clone()
                } else {
                    t.card_out.clone()
                }
            } else {
                t.card_home.clone()
            };
            ui::draw_bounded_text(
                &card,
                x + 6.0,
                card_y + 14.0,
                card_w - 12.0,
                10.0,
                dark::TEXT_BRIGHT,
            );
        }
    }
}

/// The beacon's danger meter: discrete segments so the phase reads as a hard
/// step rather than a smooth bar.
fn draw_segmented_meter(x: f32, y: f32, w: f32, h: f32, value: f32, color: Color) {
    let segments = 14;
    let gap = 3.0;
    let seg_w = (w - gap * (segments as f32 - 1.0)) / segments as f32;
    let filled = (value * segments as f32).ceil() as i32;
    for i in 0..segments {
        let sx = x + i as f32 * (seg_w + gap);
        let active = i < filled;
        let fill_color = if active {
            color
        } else {
            Color::new(0.18, 0.09, 0.08, 0.78)
        };
        draw_rectangle(sx, y, seg_w, h, fill_color);
    }
}
