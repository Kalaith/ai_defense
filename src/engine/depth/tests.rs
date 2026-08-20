use super::*;

#[test]
fn every_depth_directive_round_trips_through_save_text() {
    for directive in DepthDirective::ALL {
        assert_eq!(
            DepthDirective::from_str(directive.as_str()),
            Some(directive)
        );
    }
    assert_eq!(DepthDirective::from_str("unknown"), None);
}
