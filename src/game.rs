//! Top-level game struct: owns state, handles transitions, delegates update/draw.

use crate::data::GameData;
use crate::state::gameplay::GameplayState;
use crate::state::menu::MenuState;
use crate::state::results::ResultsState;
use crate::state::{GameState, StateTransition};

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

#[cfg(target_arch = "wasm32")]
fn survival_proof_requested() -> bool {
    macroquad_toolkit::wasm_storage::storage_get("last_assembly_proof_mode").as_deref()
        == Some("survive10")
}

#[cfg(not(target_arch = "wasm32"))]
fn survival_proof_requested() -> bool {
    std::env::var("LAST_ASSEMBLY_PROOF_MODE").as_deref() == Ok("survive10")
}
