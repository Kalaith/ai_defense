//! Bottom-panel context for a selected factory machine: repair, then power.

use crate::data::strings::{fill, text};
use crate::engine::map::BuildingState;
use crate::ui::{self, ConsoleButtonState};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use super::super::GameplayState;

impl GameplayState {
    pub(super) fn draw_building_context(&mut self, rect: Rect, idx: usize) {
        let Some(building) = self.map_state.buildings.get(idx) else {
            return;
        };
        let name = self.building_display_name(building);
        let state = building.state;
        let repair_cost = building.repair_cost;
        let power_cost = building.power_cost;
        let threat = building.threat_per_sec;
        let benefit = self.boon_text(&building.boon);
        let icon = ui::icon_for_boon(&building.boon, threat);

        ui::draw_icon(icon, rect.x + 14.0, rect.y + 18.0, 38.0, dark::ACCENT);
        ui::draw_console_header(
            rect.x + 62.0,
            rect.y + 27.0,
            &name.to_uppercase(),
            &text().panels.building_subtitle,
            dark::ACCENT,
        );
        let (status_label, status_color) = match state {
            BuildingState::Broken => (text().status.damaged.as_str(), dark::NEGATIVE),
            BuildingState::Repaired => (text().status.repaired.as_str(), dark::WARNING),
            BuildingState::Powered => (text().status.online.as_str(), dark::POSITIVE),
            BuildingState::Disabled => (text().status.locked.as_str(), dark::TEXT_DIM),
        };
        ui::draw_status_pill(rect.x + 62.0, rect.y + 43.0, status_label, status_color);
        ui::draw_bounded_text(
            &fill(&text().panels.benefit, &[("text", &benefit)]),
            rect.x + 62.0,
            rect.y + 76.0,
            rect.w * 0.42,
            14.0,
            dark::TEXT_BRIGHT,
        );
        ui::draw_bounded_text(
            &fill(&text().panels.risk_noise, &[("n", &format!("{threat:.2}"))]),
            rect.x + 62.0,
            rect.y + 98.0,
            rect.w * 0.42,
            12.0,
            dark::WARNING,
        );

        // Broken and Repaired each offer their next transition; the other two
        // states just restate the status on a dead button.
        let cost = match state {
            BuildingState::Broken => Some((repair_cost, text().status.repair.as_str())),
            BuildingState::Repaired => Some((power_cost, text().status.power.as_str())),
            BuildingState::Powered | BuildingState::Disabled => None,
        };

        let Some((cost, verb)) = cost else {
            ui::draw_console_button(
                rect.x + rect.w - 182.0,
                rect.y + 58.0,
                166.0,
                44.0,
                status_label,
                ConsoleButtonState::Disabled,
            );
            return;
        };

        let can = self.resources.scrap >= cost;
        let label = if can {
            fill(
                &text().panels.cost_action,
                &[("verb", verb), ("n", &format!("{cost:.0}"))],
            )
        } else {
            fill(
                &text().status.need_scrap,
                &[("n", &format!("{:.0}", cost - self.resources.scrap))],
            )
        };
        if ui::draw_console_button(
            rect.x + rect.w - 182.0,
            rect.y + 58.0,
            166.0,
            44.0,
            &label,
            if can {
                ConsoleButtonState::Recommended
            } else {
                ConsoleButtonState::Disabled
            },
        ) {
            match state {
                BuildingState::Broken => self.repair_building(idx),
                BuildingState::Repaired => self.power_building(idx),
                _ => {}
            }
        }
    }
}
