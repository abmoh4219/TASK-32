//! Knowledge API client serialization tests.

use frontend::api::knowledge::{
    BulkUpdate, BulkUpdateRequest, CreateCategoryInput, CreateKnowledgePointInput,
    KnowledgeFilter, KnowledgePoint, MergeRequest,
};

#[test]
fn test_knowledge_point_dto_deserializes_correctly() {
    let body = r#"{
        "id":"kp-001",
        "category_id":"cat-algebra",
        "title":"Matrix Multiplication",
        "content":"Row by column",
        "difficulty":3,
        "discrimination":0.42,
        "tags":"[\"matrix\"]",
        "created_by":"u-curator",
        "created_at":"2026-04-01T00:00:00Z",
        "updated_at":"2026-04-01T00:00:00Z"
    }"#;
    let kp: KnowledgePoint = serde_json::from_str(body).unwrap();
    assert_eq!(kp.id, "kp-001");
    assert_eq!(kp.difficulty, 3);
    assert!((kp.discrimination - 0.42).abs() < 1e-9);
}

#[test]
fn test_filter_params_serialize_to_query_string() {
    let filter = KnowledgeFilter {
        category_id: Some("cat-algebra".into()),
        difficulty_min: Some(2),
        difficulty_max: Some(4),
        discrimination_min: Some(0.3),
        discrimination_max: Some(0.5),
        tags: vec!["matrix".into(), "vectors".into()],
        chapter: Some("ch-3".into()),
    };
    let qs = filter.to_query_string();
    assert!(qs.contains("category_id=cat-algebra"));
    assert!(qs.contains("difficulty_min=2"));
    assert!(qs.contains("difficulty_max=4"));
    assert!(qs.contains("discrimination_min=0.3"));
    assert!(qs.contains("tags=matrix,vectors"));
    assert!(qs.contains("chapter=ch-3"));
}

#[test]
fn test_create_category_input_serializes_optional_parent() {
    let input = CreateCategoryInput {
        name: "New".into(),
        parent_id: Some("cat-root".into()),
        description: None,
    };
    let v = serde_json::to_value(&input).unwrap();
    assert_eq!(v["name"], "New");
    assert_eq!(v["parent_id"], "cat-root");
    assert!(v["description"].is_null());
}

#[test]
fn test_merge_request_round_trip() {
    let req = MergeRequest {
        source_id: "s".into(),
        target_id: "t".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("source_id"));
    assert!(json.contains("target_id"));
}

#[test]
fn test_bulk_update_request_carries_ids_and_changes() {
    let req = BulkUpdateRequest {
        ids: vec!["kp-001".into(), "kp-002".into()],
        changes: BulkUpdate {
            difficulty: Some(4),
            ..Default::default()
        },
    };
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["ids"].as_array().unwrap().len(), 2);
    assert_eq!(v["changes"]["difficulty"], 4);
}

#[test]
fn test_create_kp_input_includes_all_required_fields() {
    let input = CreateKnowledgePointInput {
        category_id: "cat-algebra".into(),
        title: "Test".into(),
        content: String::new(),
        difficulty: 3,
        discrimination: 0.4,
        tags: vec!["t1".into()],
    };
    let v = serde_json::to_value(&input).unwrap();
    for k in ["category_id", "title", "content", "difficulty", "discrimination", "tags"] {
        assert!(v.get(k).is_some(), "missing key {}", k);
    }
}
