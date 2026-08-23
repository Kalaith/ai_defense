use super::*;

#[test]
fn embedded_strings_load_and_are_populated() {
    // Parse the embedded copy directly: the disk copy is what ships, but a
    // broken embedded fallback would only surface on WASM otherwise.
    let t: Strings =
        serde_json::from_str(STRINGS_JSON).expect("embedded strings.json must parse into Strings");

    assert!(!t.intro.mission.is_empty(), "intro card has no mission");
    assert!(!t.intro.trade.is_empty(), "intro card has no trade-off");
    assert!(!t.intro.endgame.is_empty(), "intro card has no endgame");
    assert!(
        !t.hud.start_beacon_context.is_empty(),
        "beacon start has no visible trade-off"
    );
    assert_eq!(t.coach.steps.len(), 4, "coach expects four steps");
    assert!(!t.entrances.is_empty(), "entrance labels are empty");
    assert!(!t.map.node_labels.is_empty(), "map node labels are empty");
    assert!(
        !t.beacon.phases.terminal_howl.is_empty(),
        "beacon phase labels must be non-empty"
    );
    assert!(
        t.report.title.contains("{cycle}"),
        "report title must keep its {{cycle}} placeholder"
    );
}

#[test]
fn fill_substitutes_named_placeholders() {
    assert_eq!(
        fill("{n} of {total}", &[("n", "2"), ("total", "5")]),
        "2 of 5"
    );
    // An absent key leaves the placeholder alone rather than panicking.
    assert_eq!(fill("{a}/{b}", &[("a", "1")]), "1/{b}");
}
