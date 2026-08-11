use super::*;

#[test]
fn scripted_defense_can_survive_ten_waves() {
    let data = GameData::load();
    let mut state = GameplayState::new(&data);
    state.enable_survival_proof();

    for _ in 0..180 {
        if let Some(StateTransition::ToResults { summary }) = state.update_survival_proof(&data) {
            let (dom, dom_val) = state.threat.dominant();
            assert!(
                summary.shutdown_triggered,
                "defeat instead of shutdown: wave {}, pop {}, integrity {:.1}, food {:.1}, awareness {:.1}, loudest {} {:.1}",
                summary.waves_survived,
                summary.population_surviving,
                state.factory_integrity,
                state.population.food_supply,
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
            "game over before ten waves: wave {}, population {}, integrity {:.1}, towers {}, scrap {:.1}, power {:.1}",
            state.current_wave,
            state.population.count,
            state.factory_integrity,
            state.towers.len(),
            state.resources.scrap,
            state.resources.power
        );
    }

    panic!("survival proof did not produce a wave-10 result");
}
