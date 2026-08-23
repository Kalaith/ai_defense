//! Full-screen modals and banners: the premise card, pause menu, onboarding
//! coach, and end-of-cycle salvage report. Each handles its own buttons.

use crate::data::strings::{fill, text};
use crate::engine::map::BuildingState;
use crate::engine::population::WorkforcePolicy;
use crate::state::gameplay::GameplayState;
use crate::ui::{self, ConsoleButtonState};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::{button, draw_text_centered, draw_ui_text, TextStyle};

/// Draw `text` centred on `center_x`, using the default UI font.
fn centered(text: &str, center_x: f32, baseline_y: f32, font_size: f32, color: Color) {
    draw_text_centered(text, center_x, baseline_y, TextStyle::new(font_size, color));
}

impl GameplayState {
    pub fn draw_workforce_overlay(&mut self) {
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.72));

        let panel_w = 700.0_f32.min(sw - 32.0);
        let panel_h = 470.0_f32.min(sh - 32.0);
        let panel_x = (sw - panel_w) * 0.5;
        let panel_y = (sh - panel_h) * 0.5;
        let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.99))
            .with_border(1.8, Color::new(0.20, 0.62, 0.58, 0.95));
        macroquad_toolkit::ui::draw_surface(
            Rect::new(panel_x, panel_y, panel_w, panel_h),
            &surface,
        );

        let t = &text().workforce;
        let cx = panel_x + panel_w * 0.5;
        centered(&t.title, cx, panel_y + 42.0, 25.0, dark::POSITIVE);
        centered(
            &fill(
                &t.subtitle,
                &[
                    ("count", &self.population.count.to_string()),
                    ("capacity", &self.shelter_capacity().to_string()),
                ],
            ),
            cx,
            panel_y + 68.0,
            13.0,
            dark::TEXT_DIM,
        );

        let options = [
            (
                WorkforcePolicy::Sustain,
                t.sustain.as_str(),
                t.sustain_detail.as_str(),
            ),
            (
                WorkforcePolicy::Balanced,
                t.balanced.as_str(),
                t.balanced_detail.as_str(),
            ),
            (
                WorkforcePolicy::Salvage,
                t.salvage.as_str(),
                t.salvage_detail.as_str(),
            ),
            (
                WorkforcePolicy::Defense,
                t.defense.as_str(),
                t.defense_detail.as_str(),
            ),
        ];
        let mut chosen = None;
        let row_x = panel_x + 28.0;
        let row_w = panel_w - 56.0;
        let mut row_y = panel_y + 88.0;
        for (policy, name, detail) in options {
            let selected = self.population.workforce_policy == policy;
            ui::draw_console_panel(
                Rect::new(row_x, row_y, row_w, 68.0),
                if selected {
                    Color::new(0.20, 0.62, 0.58, 0.78)
                } else {
                    Color::new(0.16, 0.30, 0.31, 0.72)
                },
            );
            draw_ui_text(name, row_x + 14.0, row_y + 24.0, 16.0, dark::TEXT_BRIGHT);
            ui::draw_bounded_text(
                detail,
                row_x + 14.0,
                row_y + 47.0,
                row_w - 214.0,
                12.0,
                dark::TEXT_DIM,
            );
            let label = if selected {
                fill(&t.active, &[("mode", name)])
            } else {
                name.to_string()
            };
            if ui::draw_console_button(
                row_x + row_w - 184.0,
                row_y + 15.0,
                166.0,
                38.0,
                &label,
                if selected {
                    ConsoleButtonState::Disabled
                } else {
                    ConsoleButtonState::Recommended
                },
            ) && !selected
            {
                chosen = Some(policy);
            }
            row_y += 75.0;
        }

        if let Some(policy) = chosen {
            self.population.workforce_policy = policy;
            self.show_workforce = false;
            self.autosave();
            return;
        }

        if button(cx - 130.0, panel_y + panel_h - 50.0, 260.0, 34.0, &t.close) {
            self.show_workforce = false;
        }
    }

    /// Opening premise card for a fresh campaign. Handles its own Begin button.
    pub fn draw_intro(&mut self) {
        if !self.show_intro {
            return;
        }
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.82));

        let pw = 660.0_f32.min(sw - 40.0);
        let ph = 430.0_f32.min(sh - 40.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;
        let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.98))
            .with_border(1.6, Color::new(0.78, 0.16, 0.12, 0.9));
        macroquad_toolkit::ui::draw_surface(Rect::new(px, py, pw, ph), &surface);

        let t = &text().intro;
        let cx = px + pw * 0.5;
        centered(&t.title, cx, py + 46.0, 28.0, dark::NEGATIVE);
        centered(&t.subtitle, cx, py + 74.0, 14.0, dark::TEXT_DIM);

        let sections = [
            (t.mission_label.as_str(), &t.mission, dark::ACCENT),
            (t.trade_label.as_str(), &t.trade, dark::WARNING),
            (t.endgame_label.as_str(), &t.endgame, dark::POSITIVE),
        ];
        let mut y = py + 108.0;
        for (label, lines, color) in sections {
            draw_ui_text(label, px + 42.0, y, 11.0, color);
            y += 22.0;
            for line in lines {
                draw_ui_text(line, px + 42.0, y, 14.0, dark::TEXT);
                y += 20.0;
            }
            y += 10.0;
        }

        let bw = 220.0;
        let bh = 42.0;
        if button(cx - bw * 0.5, py + ph - bh - 22.0, bw, bh, &t.button) {
            self.show_intro = false;
        }
    }

    /// In-game pause menu (Esc). Handles its own buttons; also hosts the shared
    /// settings overlay.
    pub fn draw_pause_menu(&mut self) {
        if self.show_settings {
            if ui::draw_settings_overlay(&mut self.settings) == ui::SettingsAction::Close {
                self.show_settings = false;
                // Apply the autosave preference live.
                self.autosave_enabled = self.settings.autosave;
            }
            return;
        }

        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.64));

        let pw = 320.0_f32.min(sw - 40.0);
        let ph = 300.0_f32.min(sh - 40.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;
        let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.98))
            .with_border(1.6, Color::new(0.24, 0.58, 0.62, 0.9));
        macroquad_toolkit::ui::draw_surface(Rect::new(px, py, pw, ph), &surface);
        let t = &text().pause;
        centered(&t.title, px + pw * 0.5, py + 44.0, 26.0, dark::ACCENT);

        let bw = pw - 48.0;
        let bh = 40.0;
        let bx = px + 24.0;
        let gap = 12.0;
        let mut y = py + 74.0;

        if button(bx, y, bw, bh, &t.resume) {
            self.paused = false;
        }
        y += bh + gap;
        if button(bx, y, bw, bh, &t.settings) {
            self.show_settings = true;
        }
        y += bh + gap;
        if button(bx, y, bw, bh, &t.abandon) {
            self.end_campaign_requested = true;
        }
        y += bh + gap;
        if button(bx, y, bw, bh, &t.quit) {
            crate::state::menu::request_exit();
        }
    }

    /// Advance the onboarding coach as each step is actually performed.
    pub(crate) fn update_coach(&mut self) {
        if !self.coach.active {
            return;
        }
        let done = match self.coach.step {
            0 => self
                .map_state
                .buildings
                .iter()
                .any(|b| b.state != BuildingState::Broken),
            1 => self
                .map_state
                .buildings
                .iter()
                .any(|b| b.state == BuildingState::Powered),
            2 => self.towers.len() >= 3,
            _ => self.beacon_active,
        };
        if done {
            self.coach.step += 1;
            if self.coach.step >= text().coach.steps.len() {
                self.finish_coach();
            }
        }
    }

    pub(crate) fn finish_coach(&mut self) {
        self.coach.active = false;
        let mut settings = crate::save::Settings::load();
        if !settings.tutorial_seen {
            settings.tutorial_seen = true;
            let _ = settings.save();
        }
    }

    /// Onboarding banner across the top of the play area. Handles its own Skip
    /// click.
    pub fn draw_coach(&mut self) {
        if !self.coach.active {
            return;
        }
        let coach = &text().coach;
        let Some(step) = coach.steps.get(self.coach.step) else {
            return;
        };

        let bx = self.constants.ui.build_panel_w + 20.0;
        let bw = (screen_width()
            - self.constants.ui.build_panel_w
            - self.constants.ui.sector_panel_w
            - 40.0)
            .max(320.0);
        let by = self.constants.ui.hud_height + 10.0;
        let bh = 74.0;

        let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.05, 0.08, 0.10, 0.96))
            .with_border(1.4, Color::new(0.24, 0.62, 0.72, 0.9));
        macroquad_toolkit::ui::draw_surface(Rect::new(bx, by, bw, bh), &surface);

        draw_ui_text(
            &fill(
                &coach.header,
                &[
                    ("step", &(self.coach.step + 1).to_string()),
                    ("total", &coach.steps.len().to_string()),
                ],
            ),
            bx + 12.0,
            by + 18.0,
            11.0,
            dark::ACCENT,
        );
        draw_ui_text(&step.title, bx + 12.0, by + 39.0, 16.0, dark::TEXT_BRIGHT);
        ui::draw_bounded_text(
            &step.body,
            bx + 12.0,
            by + 59.0,
            bw - 24.0,
            11.0,
            dark::TEXT_DIM,
        );

        let skip_w = 86.0;
        let skip_h = 22.0;
        let skip_x = bx + bw - skip_w - 10.0;
        let skip_y = by + 8.0;
        if button(skip_x, skip_y, skip_w, skip_h, &coach.skip) {
            self.finish_coach();
        }
    }

    /// Inline end-of-cycle salvage report shown after the beacon is shut down
    /// and the field clears. Its only exit is back to base-build for the next
    /// cycle — stopping the run is the pause menu's business. Handles its own
    /// clicks.
    pub fn draw_salvage_report(&mut self) {
        let Some(report) = self.salvage_report.clone() else {
            return;
        };

        let sw = screen_width();
        let sh = screen_height();
        // Dim backdrop.
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.62));

        let panel_w = 540.0_f32.min(sw - 40.0);
        let panel_h = 400.0_f32.min(sh - 40.0);
        let panel_x = (sw - panel_w) * 0.5;
        let panel_y = (sh - panel_h) * 0.5;

        let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.98))
            .with_border(1.6, Color::new(0.20, 0.52, 0.52, 0.9));
        macroquad_toolkit::ui::draw_surface(
            Rect::new(panel_x, panel_y, panel_w, panel_h),
            &surface,
        );

        let t = &text().report;
        let cx = panel_x + panel_w * 0.5;
        let title = fill(&t.title, &[("cycle", &report.cycle.to_string())]);
        centered(&title, cx, panel_y + 40.0, 24.0, dark::POSITIVE);
        centered(&t.subtitle, cx, panel_y + 64.0, 13.0, dark::TEXT_DIM);

        // Headline: the sacrifice ledger. This is the campaign's real score —
        // how many people the beacon's scream let escape elsewhere this cycle.
        centered(
            &fill(
                &t.headline,
                &[("n", &report.survivors_evacuated_cycle.to_string())],
            ),
            cx,
            panel_y + 100.0,
            22.0,
            dark::POSITIVE,
        );
        centered(
            &fill(
                &t.total,
                &[("total", &report.survivors_evacuated_total.to_string())],
            ),
            cx,
            panel_y + 122.0,
            13.0,
            dark::TEXT_DIM,
        );

        let lines = [
            fill(&t.waves, &[("n", &report.waves.to_string())]),
            fill(&t.peak, &[("phase", report.beacon_phase.label())]),
            fill(
                &t.teams,
                &[
                    ("out", &report.scavengers_sent.to_string()),
                    ("home", &report.scavengers_returned.to_string()),
                    ("lost", &report.scavengers_lost.to_string()),
                ],
            ),
            fill(&t.scrap, &[("n", &format!("{:.0}", report.scrap))]),
            fill(&t.food, &[("n", &format!("{:.0}", report.food))]),
            fill(&t.population, &[("n", &report.population.to_string())]),
        ];
        let mut y = panel_y + 154.0;
        for line in &lines {
            draw_ui_text(line, panel_x + 40.0, y, 16.0, dark::TEXT);
            y += 26.0;
        }

        // The machines are wiser: warn that the next cycle starts harder.
        draw_ui_text(
            &fill(
                &t.escalation,
                &[("pct", &format!("{:.0}", report.escalation_pct))],
            ),
            panel_x + 40.0,
            y + 4.0,
            13.0,
            dark::WARNING,
        );

        // One way out of the report: back to the work. Ending the run is the
        // player's own decision from the pause menu — the cycle boundary should
        // never offer to stop as though it were the natural thing to do.
        let btn_w = 240.0_f32.min(panel_w - 40.0);
        let btn_h = 40.0;
        let btn_y = panel_y + panel_h - btn_h - 24.0;
        if button(cx - btn_w * 0.5, btn_y, btn_w, btn_h, &t.button) {
            self.dismiss_salvage_report();
        }
    }
}
