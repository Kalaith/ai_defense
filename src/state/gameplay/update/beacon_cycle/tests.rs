use crate::data::GameData;
use crate::state::gameplay::GameplayState;

#[test]
fn beacon_shutdown_yields_report_then_allows_another_cycle() {
    let data = GameData::load();
    let mut state = GameplayState::new(&data);
    state.autosave_enabled = false;

    // Raise the beacon: a fresh field team goes out and this cycle's
    // baseline is snapshotted.
    state.start_beacon();
    assert!(state.beacon_active);
    let teams = data.constants.scavenger.initial_scavengers;
    assert_eq!(state.scavengers_out, teams);

    // Shut it down; with no enemies in the field the cycle finishes and
    // produces a salvage report instead of ending the run.
    state.trigger_shutdown();
    assert!(state.enemies_cleared());
    state.finish_beacon_cycle();

    let report = state
        .salvage_report
        .as_ref()
        .expect("shutdown with a clear field should produce a salvage report");
    assert_eq!(report.cycle, 1);
    assert_eq!(state.beacon_cycles_completed, 1);
    assert_eq!(state.scavengers_out, 0, "field teams resolve on shutdown");

    // Dismiss the report: back to base-build mode, beacon down, restartable.
    state.dismiss_salvage_report();
    assert!(state.salvage_report.is_none());
    assert!(!state.beacon_active);
    assert!(!state.shutdown_triggered);

    // A second beacon cycle sends another fresh team out.
    state.start_beacon();
    assert!(state.beacon_active);
    assert_eq!(state.scavengers_out, teams);
    assert_eq!(state.scavengers_sent, teams * 2);
}
