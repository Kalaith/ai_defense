//! End-of-run results screen.

use crate::state::{RunSummary, StateTransition};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::button;

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
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;
        let report_top = (center_y - 70.0).max(150.0);

        let title = if self.summary.shutdown_triggered {
            "MISSION COMPLETE"
        } else {
            "FACTORY LOST"
        };
        let dims = measure_text(title, None, 40, 1.0);
        draw_text(
            title,
            center_x - dims.width / 2.0,
            report_top - 70.0,
            40.0,
            if self.summary.shutdown_triggered {
                dark::POSITIVE
            } else {
                dark::NEGATIVE
            },
        );

        let lines = build_report_lines(&self.summary);
        let mut y = report_top;
        for line in &lines {
            let dims = measure_text(line, None, 20, 1.0);
            draw_text(line, center_x - dims.width / 2.0, y, 20.0, dark::TEXT);
            y += 24.0;
        }

        let btn_w = 200.0;
        let btn_x = center_x - btn_w / 2.0;
        if button(btn_x, y + 20.0, btn_w, 45.0, "Return to Menu") {
            self.menu_clicked = true;
        }
    }
}

fn build_report_lines(summary: &RunSummary) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Waves Survived: {}", summary.waves_survived));
    lines.push(format!("Beacon Phase: {}", summary.beacon_phase.label()));
    lines.push(format!(
        "Scavengers Sent/Returned/Lost: {}/{}/{}",
        summary.scavengers_sent, summary.scavengers_returned, summary.scavengers_lost
    ));
    lines.push(format!(
        "Scavenger Gains: +{:.0} scrap, +{:.0} food, +{} pop",
        summary.scavenger_scrap, summary.scavenger_food, summary.scavenger_population
    ));
    lines.push(format!(
        "Factory Systems Online: {}",
        summary.factory_online
    ));
    lines.push(format!(
        "Population Surviving: {}",
        summary.population_surviving
    ));

    let outcome = if summary.shutdown_triggered {
        "Beacon shut down - survivors evacuated"
    } else {
        "Beacon lost - survivors scattered"
    };
    lines.push(outcome.to_string());
    lines
}
