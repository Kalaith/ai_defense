//! The gameplay screen's view layer, split by surface: [`panels`] draws the
//! docked sidebars, [`placement`] the world-space build ghost, [`overlays`] the
//! modals and banners, and [`actions`] applies the intents they raise.
//!
//! Panels never mutate state directly — they resolve a click to an intent and
//! dispatch it to an `actions` method.

mod actions;
mod overlays;
mod panels;
mod placement;

use super::GameplayState;

/// Exactly one thing is selected at a time, and each selection clears the rest.
/// These keep that invariant in one place instead of repeating five assignments
/// at every call site.
impl GameplayState {
    fn select_slot_only(&mut self, idx: usize) {
        self.selected_slot = Some(idx);
        self.selected_building = None;
        self.selected_tower = self.map_state.slots[idx].tower_index;
        self.selected_core = false;
        self.selected_upgrade = None;
    }

    fn select_building_only(&mut self, idx: usize) {
        self.selected_building = Some(idx);
        self.selected_slot = None;
        self.selected_tower = None;
        self.selected_core = false;
        self.selected_upgrade = None;
    }

    /// Selecting the core opens the factory console, so pre-arm it with the next
    /// unpurchased upgrade rather than opening on an empty detail pane.
    fn select_core_only(&mut self) {
        self.selected_core = true;
        self.selected_slot = None;
        self.selected_building = None;
        self.selected_tower = None;
        if self.selected_upgrade.is_none() {
            if let Some(upg) = self
                .available_upgrades()
                .into_iter()
                .find(|u| !self.factory.has_upgrade(&u.id))
            {
                self.selected_upgrade = Some(upg.id.clone());
            }
        }
    }

    fn clear_selection(&mut self) {
        self.selected_slot = None;
        self.selected_building = None;
        self.selected_tower = None;
        self.selected_core = false;
        self.selected_upgrade = None;
    }
}
