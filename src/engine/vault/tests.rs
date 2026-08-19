use super::*;

#[test]
fn takeover_advances_through_three_stages_and_completes_once() {
    let mut takeover = VaultTakeover::default();
    takeover.start();
    assert_eq!(takeover.stage(10.0), VaultStage::Handshake);

    assert!(!takeover.tick(10.0, 30.0));
    assert_eq!(takeover.stage(10.0), VaultStage::Severance);
    assert!(!takeover.tick(10.0, 30.0));
    assert_eq!(takeover.stage(10.0), VaultStage::Override);
    assert!(takeover.tick(10.0, 30.0));
    assert_eq!(takeover.stage(10.0), VaultStage::Complete);
    assert!(!takeover.tick(10.0, 30.0));
}

#[test]
fn inactive_takeover_does_not_advance() {
    let mut takeover = VaultTakeover::default();
    assert!(!takeover.tick(100.0, 30.0));
    assert_eq!(takeover.progress, 0.0);
}
