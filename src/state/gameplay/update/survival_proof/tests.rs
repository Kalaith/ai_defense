use super::*;
use crate::state::gameplay::DefenseReplay;

fn assert_replay_survives_ten_waves(replay: DefenseReplay) {
    let data = GameData::load();
    let mut state = GameplayState::new(&data);
    state.enable_defense_replay(replay);

    for _ in 0..180 {
        if let Some(StateTransition::ToResults { summary }) = state.update_survival_proof(&data) {
            let (dom, dom_val) = state.threat.dominant();
            assert!(
                summary.shutdown_triggered,
                "{replay:?} was defeated instead of shutting down: wave {}, pop {}, integrity {:.1}, food {:.1}, water {:.1}, towers {}, scrap {:.1}, awareness {:.1}, loudest {} {:.1}",
                summary.waves_survived,
                summary.population_surviving,
                state.factory_integrity,
                state.population.food_supply,
                state.resources.water,
                state.towers.len(),
                state.resources.scrap,
                state.threat.awareness_level(),
                dom.label(),
                dom_val,
            );
            assert_eq!(summary.waves_survived, 10);
            assert!(summary.population_surviving > 0);
            return;
        }
        assert!(
            !state.is_game_over(),
            "{replay:?} ended before ten waves: wave {}, population {}, integrity {:.1}, towers {}, scrap {:.1}, power {:.1}",
            state.current_wave,
            state.population.count,
            state.factory_integrity,
            state.towers.len(),
            state.resources.scrap,
            state.resources.power
        );
    }

    panic!("{replay:?} did not produce a wave-10 result");
}

#[test]
fn scripted_common_defense_replays_survive_ten_waves() {
    for replay in DefenseReplay::ALL {
        assert_replay_survives_ten_waves(replay);
    }
}
