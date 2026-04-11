//! Backend unit tests for the knowledge service: cycle detection, bulk-edit
//! limits, merge cycle prevention, reference counting.

use std::str::FromStr;

use backend::services::knowledge_service::{
    BulkUpdate, CreateCategoryInput, CreateKnowledgePointInput, KnowledgeService, MAX_BULK_EDIT,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

async fn fresh_db() -> SqlitePool {
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .unwrap();
    let dir = format!("{}/src/db/migrations", env!("CARGO_MANIFEST_DIR"));
    backend::db::run_migrations(&pool, &dir).await.unwrap();
    pool
}

#[tokio::test]
async fn test_cycle_detection_no_cycle() {
    let pool = fresh_db().await;
    let svc = KnowledgeService::new(pool);
    // Seeded categories form: cat-root → (cat-mathematics → cat-algebra/cat-calculus, cat-physics)
    // cat-physics is not in the descendants of cat-root in a way that would loop back.
    // Adding edge cat-physics → cat-root would create a cycle (cat-root is ancestor of cat-physics).
    // Conversely, parent_id=cat-physics, child_id=cat-algebra is fine (no path algebra→physics).
    let cycle = svc
        .check_would_create_cycle("cat-physics", "cat-algebra")
        .await
        .unwrap();
    assert!(!cycle);
}

#[tokio::test]
async fn test_cycle_detection_direct_cycle() {
    let pool = fresh_db().await;
    let svc = KnowledgeService::new(pool);
    // Same node on both ends — must be flagged as a cycle.
    assert!(svc
        .check_would_create_cycle("cat-mathematics", "cat-mathematics")
        .await
        .unwrap());
}

#[tokio::test]
async fn test_cycle_detection_indirect_cycle() {
    let pool = fresh_db().await;
    let svc = KnowledgeService::new(pool);
    // cat-algebra is a descendant of cat-mathematics. Trying to add edge
    // cat-algebra→cat-mathematics (i.e. make cat-algebra the parent of math)
    // would create an indirect cycle.
    assert!(svc
        .check_would_create_cycle("cat-algebra", "cat-mathematics")
        .await
        .unwrap());
}

#[tokio::test]
async fn test_merge_blocks_when_target_in_source_subtree() {
    let pool = fresh_db().await;
    let svc = KnowledgeService::new(pool);
    let err = svc
        .merge_nodes("cat-mathematics", "cat-algebra")
        .await
        .unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("cycle"), "got: {}", msg);
}

#[tokio::test]
async fn test_reference_count_includes_all_types() {
    let pool = fresh_db().await;
    let svc = KnowledgeService::new(pool);
    let rc = svc.get_reference_count("cat-mathematics").await.unwrap();
    // Two child categories (algebra + calculus) — no direct kps under math itself.
    assert_eq!(rc.child_category_count, 2);
    assert!(rc.total >= 2);
}

#[tokio::test]
async fn test_bulk_update_1001_returns_error() {
    let pool = fresh_db().await;
    let svc = KnowledgeService::new(pool);
    let ids: Vec<String> = (0..(MAX_BULK_EDIT + 1)).map(|i| format!("kp-{i}")).collect();
    let err = svc
        .bulk_update(
            &ids,
            &BulkUpdate {
                difficulty: Some(3),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("1000") || msg.contains("limited"),
        "got: {}",
        msg
    );
}

#[tokio::test]
async fn test_bulk_update_exactly_1000_succeeds() {
    let pool = fresh_db().await;
    let svc = KnowledgeService::new(pool);
    let ids: Vec<String> = (0..MAX_BULK_EDIT).map(|i| format!("kp-{i}")).collect();
    let result = svc
        .bulk_update(
            &ids,
            &BulkUpdate {
                difficulty: Some(3),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(result, 0); // no rows match the synthetic ids
}

#[tokio::test]
async fn test_create_category_then_kp_then_merge_flow() {
    let pool = fresh_db().await;
    let svc = KnowledgeService::new(pool);
    let a = svc
        .create_category(
            CreateCategoryInput {
                name: "Temp A".into(),
                parent_id: Some("cat-root".into()),
                description: None,
            },
            "u-curator",
        )
        .await
        .unwrap();
    let b = svc
        .create_category(
            CreateCategoryInput {
                name: "Temp B".into(),
                parent_id: Some("cat-root".into()),
                description: None,
            },
            "u-curator",
        )
        .await
        .unwrap();
    let _kp = svc
        .create_knowledge_point(
            CreateKnowledgePointInput {
                category_id: a.id.clone(),
                title: "Test KP".into(),
                content: String::new(),
                difficulty: 2,
                discrimination: 0.4,
                tags: vec!["alpha".into()],
            },
            "u-curator",
        )
        .await
        .unwrap();
    svc.merge_nodes(&a.id, &b.id).await.unwrap();
    let refs_b = svc.get_reference_count(&b.id).await.unwrap();
    assert_eq!(refs_b.direct_kp_count, 1);
}
