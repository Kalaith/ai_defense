//! Bottom-panel context for the factory core: the research/upgrade console.

use crate::data::strings::{fill, text};
use crate::data::{GameData, UpgradeDef};
use crate::engine::vault::VaultStage;
use crate::ui::{self, ConsoleButtonState, ConsoleIcon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

use super::super::GameplayState;

/// How many upgrade rows the console lists at once.
const VISIBLE_UPGRADES: usize = 3;

impl GameplayState {
    pub(super) fn draw_factory_context(&mut self, rect: Rect, _data: &GameData) {
        ui::draw_icon(
            ConsoleIcon::Power,
            rect.x + 14.0,
            rect.y + 18.0,
            34.0,
            dark::ACCENT,
        );
        ui::draw_console_header(
            rect.x + 56.0,
            rect.y + 24.0,
            &text().panels.factory_console,
            &text().panels.factory_console_subtitle,
            dark::ACCENT,
        );

        if self.draw_vault_takeover_control(rect) {
            return;
        }

        let upgrades = self.available_upgrades();
        let clicked_upgrade = self.draw_upgrade_list(rect, &upgrades);

        // Open on the next thing worth buying rather than an empty detail pane.
        if self.selected_upgrade.is_none() {
            if let Some(upg) = upgrades.iter().find(|u| !self.factory.has_upgrade(&u.id)) {
                self.selected_upgrade = Some(upg.id.clone());
            }
        }

        let unlock_upgrade = self.draw_upgrade_detail(rect);

        if let Some(id) = clicked_upgrade {
            self.selected_upgrade = Some(id);
        }
        if let Some(id) = unlock_upgrade {
            self.purchase_upgrade(&id);
        }
    }

    fn draw_vault_takeover_control(&mut self, rect: Rect) -> bool {
        let ready = self.vault_takeover_ready();
        if !ready && !self.vault_takeover.active && !self.vault_takeover.upload_complete {
            return false;
        }

        let t = &text().vault;
        let total = self.constants.vault.stage_seconds * 3.0;
        let fraction = self.vault_takeover.fraction(total);
        let stage = match self
            .vault_takeover
            .stage(self.constants.vault.stage_seconds)
        {
            VaultStage::Handshake => &t.handshake,
            VaultStage::Severance => &t.severance,
            VaultStage::Override => &t.override_stage,
            VaultStage::Complete => &t.complete,
        };
        ui::draw_bounded_text(
            &t.title,
            rect.x + 56.0,
            rect.y + 62.0,
            rect.w * 0.32,
            15.0,
            dark::WARNING,
        );
        let detail = if ready { &t.ready_detail } else { stage };
        ui::draw_bounded_text(
            detail,
            rect.x + 56.0,
            rect.y + 84.0,
            rect.w * 0.35,
            11.0,
            dark::TEXT_DIM,
        );

        let meter_x = rect.x + rect.w * 0.43;
        let meter_w = rect.w * 0.30;
        ui::draw_bounded_text(
            &fill(
                &t.progress,
                &[
                    ("stage", stage),
                    ("pct", &format!("{:.0}", fraction * 100.0)),
                ],
            ),
            meter_x,
            rect.y + 62.0,
            meter_w,
            12.0,
            dark::TEXT_BRIGHT,
        );
        ui::draw_resource_bar(
            meter_x,
            rect.y + 76.0,
            meter_w,
            10.0,
            fraction,
            1.0,
            dark::WARNING,
        );

        let (label, state) = if ready {
            (&t.begin, ConsoleButtonState::Dangerous)
        } else if self.vault_takeover.upload_complete {
            (&t.clear_line, ConsoleButtonState::Disabled)
        } else {
            (&t.locked, ConsoleButtonState::Disabled)
        };
        if ui::draw_console_button(
            rect.x + rect.w - 210.0,
            rect.y + 58.0,
            194.0,
            44.0,
            label,
            state,
        ) && ready
        {
            self.start_vault_takeover();
        }
        true
    }

    /// Selectable rows for the first few available upgrades. Returns the id of a
    /// row clicked this frame.
    fn draw_upgrade_list(&self, rect: Rect, upgrades: &[&UpgradeDef]) -> Option<String> {
        let list_x = rect.x + rect.w * 0.42;
        let list_w = rect.w * 0.34;
        let row_h = 30.0;
        let mut y = rect.y + 16.0;
        let mut clicked = None;

        for upg in upgrades.iter().take(VISIBLE_UPGRADES) {
            let selected = self.selected_upgrade.as_deref() == Some(&upg.id);
            let purchased = self.factory.has_upgrade(&upg.id);
            let accent = if selected {
                dark::TEXT_BRIGHT
            } else if purchased {
                dark::POSITIVE
            } else {
                dark::ACCENT
            };
            ui::draw_console_panel(
                Rect::new(list_x, y, list_w, row_h),
                Color::new(accent.r, accent.g, accent.b, 0.55),
            );
            ui::draw_bounded_text(
                &upg.name,
                list_x + 8.0,
                y + 13.0,
                list_w - 16.0,
                11.0,
                accent,
            );
            ui::draw_bounded_text(
                &upg.description,
                list_x + 8.0,
                y + 26.0,
                list_w - 16.0,
                9.0,
                dark::TEXT_DIM,
            );
            let (mx, my) = mouse_position();
            if mx >= list_x
                && mx <= list_x + list_w
                && my >= y
                && my <= y + row_h
                && is_mouse_button_pressed(MouseButton::Left)
            {
                clicked = Some(upg.id.clone());
            }
            y += row_h + 7.0;
        }

        clicked
    }

    /// Detail pane plus UNLOCK button for the selected upgrade. Returns the id
    /// to purchase when the button is pressed.
    fn draw_upgrade_detail(&self, rect: Rect) -> Option<String> {
        let selected_id = self.selected_upgrade.clone()?;
        let selected = self.upgrade_defs.iter().find(|u| u.id == selected_id)?;

        ui::draw_bounded_text(
            &selected.name,
            rect.x + 56.0,
            rect.y + 62.0,
            rect.w * 0.34,
            14.0,
            dark::TEXT_BRIGHT,
        );
        ui::draw_bounded_text(
            &selected.description,
            rect.x + 56.0,
            rect.y + 82.0,
            rect.w * 0.34,
            11.0,
            dark::TEXT_DIM,
        );

        let purchased = self.factory.has_upgrade(&selected.id);
        let can = self
            .factory
            .can_purchase(selected, self.resources.scrap, self.resources.power);
        let label = if purchased {
            text().panels.unlocked.clone()
        } else if can {
            fill(
                &text().panels.unlock,
                &[("n", &format!("{:.0}", selected.cost_scrap))],
            )
        } else {
            fill(
                &text().status.need_scrap,
                &[(
                    "n",
                    &format!(
                        "{:.0}",
                        (selected.cost_scrap - self.resources.scrap).max(0.0)
                    ),
                )],
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
            return Some(selected.id.clone());
        }
        None
    }
}
