//! Frontend filter state unit tests — pure logic, no DOM/WASM required.

use frontend::logic::filter::{DiscriminationBandPreset, KnowledgeFilterState};

#[test]
fn test_filter_state_combines_difficulty_and_tags() {
    let mut f = KnowledgeFilterState::new();
    f.add_tag("algebra");
    f.add_tag("matrix");
    f.set_difficulty_range(Some(2), Some(4));
    assert_eq!(f.tags, vec!["algebra".to_string(), "matrix".to_string()]);
    assert_eq!(f.difficulty_min, Some(2));
    assert_eq!(f.difficulty_max, Some(4));
    assert!(!f.is_empty());
}

#[test]
fn test_filter_state_clears_correctly() {
    let mut f = KnowledgeFilterState::new();
    f.add_tag("foo");
    f.set_difficulty_range(Some(1), Some(5));
    f.apply_discrimination_band(DiscriminationBandPreset::Good);
    f.clear();
    assert!(f.is_empty());
    assert!(f.tags.is_empty());
    assert!(f.difficulty_min.is_none());
    assert!(f.discrimination_min.is_none());
}

#[test]
fn test_discrimination_band_preset_sets_correct_range() {
    let mut f = KnowledgeFilterState::new();
    f.apply_discrimination_band(DiscriminationBandPreset::Acceptable);
    assert_eq!(f.discrimination_min, Some(0.1));
    assert_eq!(f.discrimination_max, Some(0.3));

    f.apply_discrimination_band(DiscriminationBandPreset::Excellent);
    assert_eq!(f.discrimination_min, Some(0.5));
    assert!(f.discrimination_max.unwrap() > 1.0);

    f.apply_discrimination_band(DiscriminationBandPreset::Poor);
    assert_eq!(f.discrimination_min, Some(-1.0));
    assert_eq!(f.discrimination_max, Some(0.1));
}

#[test]
fn test_remove_tag_only_removes_target() {
    let mut f = KnowledgeFilterState::new();
    f.add_tag("a");
    f.add_tag("b");
    f.add_tag("c");
    f.remove_tag("b");
    assert_eq!(f.tags, vec!["a".to_string(), "c".to_string()]);
}

#[test]
fn test_add_tag_dedupes() {
    let mut f = KnowledgeFilterState::new();
    f.add_tag("repeat");
    f.add_tag("repeat");
    assert_eq!(f.tags.len(), 1);
}
