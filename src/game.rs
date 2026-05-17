//! Top-level game struct: owns state, handles transitions, delegates update/draw.

use crate::data::GameData;
use crate::state::gameplay::GameplayState;
use crate::state::menu::MenuState;
use crate::state::results::ResultsState;
use crate::state::{GameState, StateTransition};

pub struct Game {
    pub state: GameState,
    pub data: GameData,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load();

        Self {
            state: GameState::Menu(MenuState::new()),
            data,
        }
    }

    pub fn update(&mut self) {
        let transition = match &mut self.state {
            GameState::Menu(s) => s.update(),
            GameState::Gameplay(s) => s.update(),
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
