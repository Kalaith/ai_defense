//! Full-screen modals and banners: the premise card, pause menu, onboarding
//! coach, and end-of-cycle salvage report. Each handles its own buttons.

use crate::engine::map::BuildingState;
use crate::state::gameplay::{GameplayState, COACH_STEPS};
use crate::ui;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::{button, draw_text_centered, draw_ui_text, TextStyle};

/// Draw `text` centred on `center_x`, using the default UI font.
fn centered(text: &str, center_x: f32, baseline_y: f32, font_size: f32, color: Color) {
    draw_text_centered(text, center_x, baseline_y, TextStyle::new(font_size, color));
}

impl GameplayState {
    /// Opening premise card for a fresh campaign. Handles its own Begin button.
    pub fn draw_intro(&mut self) {
        if !self.show_intro {
            return;
        }
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.82));

        let pw = 620.0_f32.min(sw - 40.0);
        let ph = 400.0_f32.min(sh - 40.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;
        let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.98))
            .with_border(1.6, Color::new(0.78, 0.16, 0.12, 0.9));
        macroquad_toolkit::ui::draw_surface(Rect::new(px, py, pw, ph), &surface);

        let cx = px + pw * 0.5;
        centered("LAST ASSEMBLY", cx, py + 46.0, 28.0, dark::NEGATIVE);
        centered(
            "You woke a dead megafactory. Its beacon is bait.",
            cx,
            py + 74.0,
            14.0,
            dark::TEXT_DIM,
        );

        let body = [
            "The factory screams into the dark so the machines swarm you",
            "instead of the last human settlements hiding elsewhere.",
            "",
            "Restore its systems to help those settlers — but every system",
            "you power makes the beacon louder and the attacks harder.",
            "",
            "While it burns, send scavengers out for salvage. Recall them",
            "in time, and choose when to go dark. Then rebuild and do it",
            "again. There is no clean victory — only how long you hold.",
        ];
        let mut y = py + 108.0;
        for line in body {
            draw_ui_text(line, px + 40.0, y, 15.0, dark::TEXT);
            y += 24.0;
        }

        let bw = 220.0;
        let bh = 42.0;
        if button(
            cx - bw * 0.5,
            py + ph - bh - 22.0,
            bw,
            bh,
            "Hold the Beacon",
        ) {
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
        centered("PAUSED", px + pw * 0.5, py + 44.0, 26.0, dark::ACCENT);

        let bw = pw - 48.0;
        let bh = 40.0;
        let bx = px + 24.0;
        let gap = 12.0;
        let mut y = py + 74.0;

        if button(bx, y, bw, bh, "Resume") {
            self.paused = false;
        }
        y += bh + gap;
        if button(bx, y, bw, bh, "Settings") {
            self.show_settings = true;
        }
        y += bh + gap;
        if button(bx, y, bw, bh, "Abandon Run (see report)") {
            self.end_campaign_requested = true;
        }
        y += bh + gap;
        if button(bx, y, bw, bh, "Quit to Desktop") {
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
            2 => !self.towers.is_empty(),
            _ => self.beacon_active,
        };
        if done {
            self.coach.step += 1;
            if self.coach.step >= COACH_STEPS.len() {
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
        let Some((title, body)) = COACH_STEPS.get(self.coach.step).copied() else {
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
            &format!(
                "GETTING STARTED  ·  Step {} of {}",
                self.coach.step + 1,
                COACH_STEPS.len()
            ),
            bx + 12.0,
            by + 18.0,
            11.0,
            dark::ACCENT,
        );
        draw_ui_text(title, bx + 12.0, by + 39.0, 16.0, dark::TEXT_BRIGHT);
        ui::draw_bounded_text(body, bx + 12.0, by + 59.0, bw - 24.0, 11.0, dark::TEXT_DIM);

        let skip_w = 86.0;
        let skip_h = 22.0;
        let skip_x = bx + bw - skip_w - 10.0;
        let skip_y = by + 8.0;
        if button(skip_x, skip_y, skip_w, skip_h, "Skip") {
            self.finish_coach();
        }
    }

    /// Inline end-of-cycle salvage report shown after the beacon is shut down
    /// and the field clears. Offers a return to base-build (raise the beacon
    /// again) or ending the campaign for a final tally. Handles its own clicks.
    pub fn draw_salvage_report(&mut self) {
        let Some(report) = self.salvage_report.clone() else {
            return;
        };

        let sw = screen_width();
        let sh = screen_height();
        // Dim backdrop.
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.62));

        let panel_w = 540.0_f32.min(sw - 40.0);
        let panel_h = 452.0_f32.min(sh - 40.0);
        let panel_x = (sw - panel_w) * 0.5;
        let panel_y = (sh - panel_h) * 0.5;

        let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.04, 0.06, 0.07, 0.98))
            .with_border(1.6, Color::new(0.20, 0.52, 0.52, 0.9));
        macroquad_toolkit::ui::draw_surface(
            Rect::new(panel_x, panel_y, panel_w, panel_h),
            &surface,
        );

        let cx = panel_x + panel_w * 0.5;
        let title = format!("SALVAGE REPORT — Cycle {}", report.cycle);
        centered(&title, cx, panel_y + 40.0, 24.0, dark::POSITIVE);
        centered(
            "Beacon dark. The field is clear.",
            cx,
            panel_y + 64.0,
            13.0,
            dark::TEXT_DIM,
        );

        // Headline: the sacrifice ledger. This is the campaign's real score —
        // how many people the beacon's scream let escape elsewhere this cycle.
        centered(
            &format!("+{} SURVIVORS EVACUATED", report.survivors_evacuated_cycle),
            cx,
            panel_y + 100.0,
            22.0,
            dark::POSITIVE,
        );
        centered(
            &format!(
                "{} have now reached safe territory",
                report.survivors_evacuated_total
            ),
            cx,
            panel_y + 122.0,
            13.0,
            dark::TEXT_DIM,
        );

        let lines = [
            format!("Waves held this cycle: {}", report.waves),
            format!("Peak beacon: {}", report.beacon_phase.label()),
            format!(
                "Scavengers  sent {} / returned {} / lost {}",
                report.scavengers_sent, report.scavengers_returned, report.scavengers_lost
            ),
            format!("Salvage banked: +{:.0} scrap", report.scrap),
            format!("Rations banked: +{:.0} food", report.food),
            format!("Survivors found: +{}", report.population),
        ];
        let mut y = panel_y + 154.0;
        for line in &lines {
            draw_ui_text(line, panel_x + 40.0, y, 16.0, dark::TEXT);
            y += 26.0;
        }

        // The machines are wiser: warn that the next cycle starts harder.
        draw_ui_text(
            &format!(
                "Machine assault escalation: +{:.0}% — the next cycle starts harder.",
                report.escalation_pct
            ),
            panel_x + 40.0,
            y + 4.0,
            13.0,
            dark::WARNING,
        );

        let btn_w = (panel_w - 60.0) * 0.5;
        let btn_h = 40.0;
        let btn_y = panel_y + panel_h - btn_h - 24.0;
        if button(
            panel_x + 20.0,
            btn_y,
            btn_w,
            btn_h,
            "Rebuild & Raise Beacon",
        ) {
            self.dismiss_salvage_report();
        }
        if button(panel_x + 40.0 + btn_w, btn_y, btn_w, btn_h, "End Campaign") {
            self.end_campaign_requested = true;
        }
    }
}
