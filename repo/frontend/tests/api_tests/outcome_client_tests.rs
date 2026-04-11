//! Outcome API client serialization tests.

use frontend::api::outcomes::{
    AddContributorInput, CreateOutcomeInput, CreateOutcomeResult, DuplicateCandidate, Outcome,
};

#[test]
fn test_create_outcome_request_serializes_correctly() {
    let input = CreateOutcomeInput {
        r#type: "paper".into(),
        title: "Quantum".into(),
        abstract_snippet: "summary".into(),
        certificate_number: Some("CERT-1".into()),
    };
    let v = serde_json::to_value(&input).unwrap();
    assert_eq!(v["type"], "paper", "should serialize to JSON key `type`");
    assert_eq!(v["title"], "Quantum");
    assert_eq!(v["certificate_number"], "CERT-1");
}

#[test]
fn test_outcome_dto_uses_type_alias_for_type_field() {
    let body = r#"{
        "id":"o-1",
        "type":"patent",
        "title":"Title",
        "abstract_snippet":"a",
        "certificate_number":null,
        "status":"draft",
        "submitted_at":null,
        "approved_at":null,
        "rejected_at":null,
        "rejection_reason":null,
        "approver_id":null,
        "created_by":"u-reviewer",
        "created_at":"2026-04-01T00:00:00Z",
        "updated_at":"2026-04-01T00:00:00Z"
    }"#;
    let o: Outcome = serde_json::from_str(body).unwrap();
    assert_eq!(o.r#type, "patent");
    assert_eq!(o.status, "draft");
}

#[test]
fn test_duplicate_candidate_response_deserializes() {
    let body = r#"{
        "id":"o-9",
        "title":"Existing",
        "similarity_score":0.92,
        "reason":"title similarity 0.92"
    }"#;
    let cand: DuplicateCandidate = serde_json::from_str(body).unwrap();
    assert_eq!(cand.id, "o-9");
    assert!((cand.similarity_score - 0.92).abs() < 1e-9);
}

#[test]
fn test_create_outcome_result_carries_outcome_and_candidates() {
    let body = r#"{
        "outcome": {
            "id":"o-1","type":"paper","title":"T","abstract_snippet":"a","certificate_number":null,
            "status":"draft","submitted_at":null,"approved_at":null,"rejected_at":null,
            "rejection_reason":null,"approver_id":null,"created_by":"u","created_at":"x","updated_at":"x"
        },
        "duplicate_candidates": [
            {"id":"o-2","title":"Other","similarity_score":0.88,"reason":"title similarity 0.88"}
        ]
    }"#;
    let res: CreateOutcomeResult = serde_json::from_str(body).unwrap();
    assert_eq!(res.outcome.id, "o-1");
    assert_eq!(res.duplicate_candidates.len(), 1);
}

#[test]
fn test_add_contributor_input_round_trip() {
    let input = AddContributorInput {
        user_id: "u-1".into(),
        share_percentage: 60,
        role_in_work: Some("author".into()),
    };
    let v = serde_json::to_value(&input).unwrap();
    assert_eq!(v["user_id"], "u-1");
    assert_eq!(v["share_percentage"], 60);
    assert_eq!(v["role_in_work"], "author");
}
