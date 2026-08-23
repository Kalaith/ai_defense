use super::*;

#[test]
fn corruption_distortion_starts_only_after_the_high_corruption_threshold() {
    assert_eq!(corruption_distortion_strength(0.0), 0.0);
    assert_eq!(
        corruption_distortion_strength(CORRUPTION_DISTORTION_START),
        0.0
    );
    assert!(corruption_distortion_strength(CORRUPTION_DISTORTION_START + 14.0) > 0.0);
    assert_eq!(
        corruption_distortion_strength(CORRUPTION_DISTORTION_START + CORRUPTION_DISTORTION_RANGE),
        1.0
    );
}
