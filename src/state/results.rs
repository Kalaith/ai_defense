//! End-of-run results screen.

use crate::data::strings::{fill, text};
use crate::state::{RunSummary, StateTransition};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::button;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub struct ResultsState {
    pub summary: RunSummary,
    menu_clicked: bool,
}

impl ResultsState {
    pub fn new(summary: RunSummary) -> Self {
        Self {
            summary,
            menu_clicked: false,
        }
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        if self.menu_clicked {
            self.menu_clicked = false;
            return Some(StateTransition::ToMenu);
        }
        None
    }

    pub fn draw(&mut self) {
        let t = &text().results;
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;
        let report_top = (center_y - 70.0).max(150.0);

        let title = if self.summary.campaign_won {
            &t.title_victory
        } else if self.summary.shutdown_triggered {
            &t.title_survived
        } else {
            &t.title_lost
        };
        let dims = measure_ui_text(title, None, 40, 1.0);
        draw_ui_text(
            title,
            center_x - dims.width / 2.0,
            report_top - 70.0,
            40.0,
            if self.summary.campaign_won {
                dark::ACCENT
            } else if self.summary.shutdown_triggered {
                dark::POSITIVE
            } else {
                dark::NEGATIVE
            },
        );

        let lines = build_report_lines(&self.summary);
        let mut y = report_top;
        for line in &lines {
            let dims = measure_ui_text(line, None, 20, 1.0);
            draw_ui_text(line, center_x - dims.width / 2.0, y, 20.0, dark::TEXT);
            y += 24.0;
        }

        let btn_w = 200.0;
        let btn_x = center_x - btn_w / 2.0;
        if button(btn_x, y + 20.0, btn_w, 45.0, &t.button) {
            self.menu_clicked = true;
        }
    }
}

fn build_report_lines(summary: &RunSummary) -> Vec<String> {
    let t = &text().results;
    let mut lines = Vec::new();
    // Headline: the campaign's real score — people the beacon let escape.
    lines.push(fill(
        &t.headline,
        &[("n", &summary.survivors_evacuated.to_string())],
    ));
    if summary.evacuees_lost > 0 {
        lines.push(fill(
            &t.lost_inside,
            &[("n", &summary.evacuees_lost.to_string())],
        ));
    }
    lines.push(String::new());
    lines.push(fill(
        &t.waves,
        &[("n", &summary.waves_survived.to_string())],
    ));
    lines.push(fill(&t.peak, &[("phase", summary.beacon_phase.label())]));
    lines.push(fill(
        &t.teams,
        &[
            ("out", &summary.scavengers_sent.to_string()),
            ("home", &summary.scavengers_returned.to_string()),
            ("lost", &summary.scavengers_lost.to_string()),
        ],
    ));
    lines.push(fill(
        &t.carried,
        &[
            ("scrap", &format!("{:.0}", summary.scavenger_scrap)),
            ("food", &format!("{:.0}", summary.scavenger_food)),
            ("pop", &summary.scavenger_population.to_string()),
        ],
    ));
    lines.push(fill(
        &t.machines,
        &[("n", &summary.factory_online.to_string())],
    ));
    lines.push(fill(
        &t.surviving,
        &[("n", &summary.population_surviving.to_string())],
    ));

    lines.push(if summary.campaign_won {
        t.outcome_victory.clone()
    } else if summary.shutdown_triggered {
        t.outcome_survived.clone()
    } else {
        t.outcome_lost.clone()
    });
    lines
}
