//! UI helpers and reusable components for Last Assembly.
//!
//! Split by surface: [`widgets`] draws the console chrome, [`icons`] the glyph
//! vocabulary, [`build_panel`] the left sidebar, [`settings`] the shared modal,
//! and [`text`] the width-bounded text primitives the rest build on.

mod build_panel;
mod icons;
mod settings;
mod text;
mod widgets;

pub use build_panel::{draw_build_panel, TowerButtonInfo};
pub use icons::{draw_icon, icon_for_boon, ConsoleIcon};
pub use settings::{draw_settings_overlay, SettingsAction};
pub use text::draw_bounded_text;
pub use widgets::{
    draw_console_button, draw_console_header, draw_console_panel, draw_edge_button,
    draw_resource_bar, draw_status_pill, ConsoleButtonState,
};
