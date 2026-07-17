//! Game state machine: one state active at a time, explicit transitions.

pub mod gameplay;
pub mod menu;
pub mod results;

use crate::engine::beacon::BeaconPhase;
use crate::save::SaveData;
use gameplay::GameplayState;
use menu::MenuState;
use results::ResultsState;

// GameplayState is intentionally much larger than Menu/Results — it owns the
// whole live simulation. Boxing it would just move the size cost to every read.
#[allow(clippy::large_enum_variant)]
pub enum GameState {
    Menu(MenuState),
    Gameplay(GameplayState),
    Results(ResultsState),
}

pub struct RunSummary {
    pub waves_survived: u32,
    pub beacon_phase: BeaconPhase,
    pub scavengers_sent: u32,
    pub scavengers_returned: u32,
    pub scavengers_lost: u32,
    pub scavenger_scrap: f32,
    pub scavenger_food: f32,
    pub scavenger_population: u32,
    pub factory_online: usize,
    pub population_surviving: u32,
    pub shutdown_triggered: bool,
    /// Total survivors the beacon evacuated across the whole campaign — the
    /// headline score. `evacuees_lost` counts those still pending when the
    /// factory fell (a defeat loses the current beacon window's progress).
    pub survivors_evacuated: u32,
    pub evacuees_lost: u32,
}

// The shared `To` prefix reads as clearer at call sites (state.transition_to(ToGameplay))
// than trimming it would.
#[allow(clippy::enum_variant_names)]
pub enum StateTransition {
    ToMenu,
    ToGameplay,
    ToGameplayLoaded { save: SaveData },
    ToResults { summary: RunSummary },
}
