//! Top-level game struct: owns state, handles transitions, delegates update/draw.

use crate::data::GameData;
use crate::engine::beacon::BeaconPhase;
use crate::state::gameplay::{GameplayState, SalvageReport};
use crate::state::menu::MenuState;
use crate::state::results::ResultsState;
use crate::state::{GameState, RunSummary, StateTransition};

pub struct Game {
    pub state: GameState,
    pub data: GameData,
    survival_proof_mode: bool,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load();
        let survival_proof_mode = survival_proof_requested();
        let state = if survival_proof_mode {
            let mut gameplay = GameplayState::new(&data);
            gameplay.enable_survival_proof();
            GameState::Gameplay(gameplay)
        } else {
            GameState::Menu(MenuState::new())
        };

        Self {
            state,
            data,
            survival_proof_mode,
        }
    }

    pub fn update(&mut self) {
        let transition = match &mut self.state {
            GameState::Menu(s) => s.update(),
            GameState::Gameplay(s) => {
                if self.survival_proof_mode {
                    s.update_survival_proof(&self.data)
                } else {
                    s.update()
                }
            }
            GameState::Results(s) => s.update(),
        };

        if let Some(t) = transition {
            self.transition(t);
        }
    }

    pub fn draw(&mut self) {
        match &mut self.state {
            GameState::Menu(s) => s.draw(),
            GameState::Gameplay(s) => s.draw(&self.data),
            GameState::Results(s) => s.draw(),
        }
    }

    /// Seed a specific scene for the screenshot harness (see
    /// `docs/screenshot_capture_harness_guide.md`). Unrecognized scene names
    /// fall through to whatever the normal boot flow lands on.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "menu" => self.state = GameState::Menu(MenuState::new()),
            "gameplay" => {
                let mut gameplay = GameplayState::new(&self.data);
                // Skip the first-run premise card so the capture shows the
                // actual factory floor instead of a modal.
                gameplay.show_intro = false;
                self.state = GameState::Gameplay(gameplay);
            }
            "sector" => {
                let mut gameplay = GameplayState::new(&self.data);
                gameplay.show_intro = false;
                gameplay.coach.active = false;
                for core in ["building_01", "building_02", "building_09"] {
                    if let Some(building) = gameplay
                        .map_state
                        .buildings
                        .iter_mut()
                        .find(|building| building.id == core)
                    {
                        building.state = crate::engine::map::BuildingState::Powered;
                    }
                }
                gameplay.map_state.update_section_visibility();
                if let Some(idx) = gameplay
                    .map_state
                    .buildings
                    .iter()
                    .position(|building| building.id == "building_10")
                {
                    gameplay.map_state.buildings[idx].state =
                        crate::engine::map::BuildingState::Repaired;
                    gameplay.selected_building = Some(idx);
                    gameplay.resources.scrap = 180.0;
                }
                self.state = GameState::Gameplay(gameplay);
            }
            "specialization" => {
                let mut gameplay = GameplayState::new(&self.data);
                gameplay.show_intro = false;
                gameplay.coach.active = false;
                gameplay.factory.unlock_from_core("building_13");
                gameplay.resources.scrap = 500.0;
                if let (Some(def), Some(slot)) = (
                    self.data.tower_def_by_id("ballistic_turret"),
                    gameplay.map_state.slots.first(),
                ) {
                    let mut tower = crate::engine::tower::Tower::new(
                        def.tower_type.clone(),
                        def.id.clone(),
                        slot.position,
                        def.base_range,
                        def.base_damage,
                        def.fire_rate,
                        def.cost_power,
                        def.cost_scrap,
                        def.color(),
                    );
                    for _ in 1..gameplay.constants.tower.upgrade_max_level {
                        tower.level += 1;
                        tower.damage *= gameplay.constants.tower.upgrade_damage_mult;
                        tower.range *= gameplay.constants.tower.upgrade_range_mult;
                    }
                    tower.target_priority = crate::engine::tower::TargetPriority::Strongest;
                    gameplay.towers.push(tower);
                    gameplay.tower_stats.push(Default::default());
                    gameplay.map_state.slots[0].tower_index = Some(0);
                    gameplay.selected_tower = Some(0);
                }
                self.state = GameState::Gameplay(gameplay);
            }
            "intro" => {
                let mut gameplay = GameplayState::new(&self.data);
                gameplay.show_intro = true;
                self.state = GameState::Gameplay(gameplay);
            }
            // The coach only arms itself for players who have not seen the
            // tutorial, so force it on rather than depending on local settings.
            "coach" => {
                let mut gameplay = GameplayState::new(&self.data);
                gameplay.show_intro = false;
                gameplay.coach.active = true;
                gameplay.coach.step = 0;
                self.state = GameState::Gameplay(gameplay);
            }
            "report" => {
                let mut gameplay = GameplayState::new(&self.data);
                gameplay.show_intro = false;
                gameplay.salvage_report = Some(sample_salvage_report());
                self.state = GameState::Gameplay(gameplay);
            }
            "vault" => {
                let mut gameplay = GameplayState::new(&self.data);
                gameplay.show_intro = false;
                gameplay.coach.active = false;
                for sector in &mut gameplay.factory.sectors {
                    sector.unlocked = true;
                    sector.integrity = sector.max_integrity;
                }
                gameplay.factory.check_awakening();
                gameplay.selected_core = true;
                self.state = GameState::Gameplay(gameplay);
            }
            "results" => self.state = GameState::Results(ResultsState::new(sample_run_summary())),
            "victory" => {
                self.state = GameState::Results(ResultsState::new(sample_victory_summary()))
            }
            "workforce" => {
                let mut gameplay = GameplayState::new(&self.data);
                gameplay.show_intro = false;
                gameplay.coach.active = false;
                gameplay.population.count = gameplay.shelter_capacity() + 4;
                gameplay.show_workforce = true;
                self.state = GameState::Gameplay(gameplay);
            }
            _ => {}
        }
    }

    fn transition(&mut self, transition: StateTransition) {
        self.state = match transition {
            StateTransition::ToMenu => GameState::Menu(MenuState::new()),
            StateTransition::ToGameplay => GameState::Gameplay(GameplayState::new(&self.data)),
            StateTransition::ToGameplayLoaded { save } => {
                GameState::Gameplay(GameplayState::from_save(&self.data, save))
            }
            StateTransition::ToResults { summary } => {
                GameState::Results(ResultsState::new(summary))
            }
        };
    }
}

/// Representative end-of-cycle numbers for the `report` capture scene, so the
/// modal's copy and layout can be checked without playing a cycle out.
fn sample_salvage_report() -> SalvageReport {
    SalvageReport {
        cycle: 3,
        waves: 7,
        scavengers_sent: 4,
        scavengers_returned: 3,
        scavengers_lost: 1,
        scrap: 240.0,
        food: 95.0,
        population: 2,
        beacon_phase: BeaconPhase::ScreamingBeacon,
        survivors_evacuated_cycle: 14,
        survivors_evacuated_total: 41,
        escalation_pct: 18.0,
    }
}

/// Same idea for the `results` capture scene.
fn sample_run_summary() -> RunSummary {
    RunSummary {
        waves_survived: 22,
        beacon_phase: BeaconPhase::TerminalHowl,
        scavengers_sent: 11,
        scavengers_returned: 8,
        scavengers_lost: 3,
        scavenger_scrap: 760.0,
        scavenger_food: 310.0,
        scavenger_population: 5,
        factory_online: 9,
        population_surviving: 12,
        shutdown_triggered: true,
        campaign_won: false,
        survivors_evacuated: 41,
        evacuees_lost: 0,
    }
}

fn sample_victory_summary() -> RunSummary {
    RunSummary {
        waves_survived: 31,
        beacon_phase: BeaconPhase::TerminalHowl,
        scavengers_sent: 18,
        scavengers_returned: 14,
        scavengers_lost: 4,
        scavenger_scrap: 1_240.0,
        scavenger_food: 560.0,
        scavenger_population: 9,
        factory_online: 12,
        population_surviving: 17,
        shutdown_triggered: true,
        campaign_won: true,
        survivors_evacuated: 73,
        evacuees_lost: 0,
    }
}

#[cfg(target_arch = "wasm32")]
fn survival_proof_requested() -> bool {
    macroquad_toolkit::wasm_storage::storage_get("last_assembly_proof_mode").as_deref()
        == Some("survive10")
}

#[cfg(not(target_arch = "wasm32"))]
fn survival_proof_requested() -> bool {
    std::env::var("LAST_ASSEMBLY_PROOF_MODE").as_deref() == Ok("survive10")
}
