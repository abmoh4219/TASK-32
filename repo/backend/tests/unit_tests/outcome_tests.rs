//! Backend unit tests for the outcome service: contributor share validation,
//! file magic-number gating, fingerprint dedup, duplicate detection thresholds.

use std::str::FromStr;

use backend::services::file_service::FileService;
use backend::services::outcome_service::{
    AddContributorInput, CreateOutcomeInput, OutcomeService,
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

async fn create_draft(svc: &OutcomeService) -> String {
    let result = svc
        .create_outcome(
            CreateOutcomeInput {
                r#type: "paper".into(),
                title: "Quantum Topology Foundations".into(),
                abstract_snippet: "An exploration of topological invariants in low-dimensional manifolds.".into(),
                certificate_number: None,
            },
            "u-reviewer",
        )
        .await
        .unwrap();
    result.outcome.id
}

#[tokio::test]
async fn test_share_validation_exactly_100_passes() {
    let pool = fresh_db().await;
    let svc = OutcomeService::new(pool);
    let id = create_draft(&svc).await;
    svc.add_contributor(
        &id,
        AddContributorInput {
            user_id: "u-reviewer".into(),
            share_percentage: 60,
            role_in_work: None,
        },
    )
    .await
    .unwrap();
    svc.add_contributor(
        &id,
        AddContributorInput {
            user_id: "u-curator".into(),
            share_percentage: 40,
            role_in_work: None,
        },
    )
    .await
    .unwrap();
    let updated = svc.submit_outcome(&id).await.unwrap();
    assert_eq!(updated.status, "submitted");
}

#[tokio::test]
async fn test_share_validation_99_fails() {
    let pool = fresh_db().await;
    let svc = OutcomeService::new(pool);
    let id = create_draft(&svc).await;
    svc.add_contributor(
        &id,
        AddContributorInput {
            user_id: "u-reviewer".into(),
            share_percentage: 99,
            role_in_work: None,
        },
    )
    .await
    .unwrap();
    let err = svc.submit_outcome(&id).await.unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("100"), "got: {}", msg);
}

#[tokio::test]
async fn test_share_validation_101_fails_at_add() {
    let pool = fresh_db().await;
    let svc = OutcomeService::new(pool);
    let id = create_draft(&svc).await;
    svc.add_contributor(
        &id,
        AddContributorInput {
            user_id: "u-reviewer".into(),
            share_percentage: 60,
            role_in_work: None,
        },
    )
    .await
    .unwrap();
    let err = svc
        .add_contributor(
            &id,
            AddContributorInput {
                user_id: "u-curator".into(),
                share_percentage: 50,
                role_in_work: None,
            },
        )
        .await
        .unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("100"), "got: {}", msg);
}

// PDF, JPEG, PNG magic-number test fixtures.
const PDF_BYTES: &[u8] = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\nrest";
const JPEG_BYTES: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00];
const PNG_BYTES: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H', b'D', b'R',
];
const EXE_BYTES: &[u8] = &[0x4D, 0x5A, 0x90, 0x00];

#[test]
fn test_file_pdf_magic_number_accepted() {
    FileService::validate_file(PDF_BYTES, "application/pdf").unwrap();
}

#[test]
fn test_file_jpeg_magic_number_accepted() {
    FileService::validate_file(JPEG_BYTES, "image/jpeg").unwrap();
}

#[test]
fn test_file_png_magic_number_accepted() {
    FileService::validate_file(PNG_BYTES, "image/png").unwrap();
}

#[test]
fn test_file_wrong_magic_number_rejected() {
    let err = FileService::validate_file(EXE_BYTES, "application/pdf").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("invalid") || msg.contains("MIME") || msg.contains("unknown"),
        "got: {}",
        msg
    );
}

#[test]
fn test_file_exceeds_25mb_rejected() {
    let huge = vec![0u8; 26 * 1024 * 1024];
    let err = FileService::validate_file(&huge, "application/pdf").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("file too large"), "got: {}", msg);
}

#[test]
fn test_file_mime_mismatch_rejected() {
    // Real PDF bytes but the upload claims it is a JPEG.
    let err = FileService::validate_file(PDF_BYTES, "image/jpeg").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("MIME"), "got: {}", msg);
}

#[test]
fn test_fingerprint_consistent() {
    assert_eq!(
        FileService::fingerprint(b"hello world"),
        FileService::fingerprint(b"hello world")
    );
    assert_ne!(
        FileService::fingerprint(b"hello"),
        FileService::fingerprint(b"world")
    );
}

#[tokio::test]
async fn test_duplicate_detection_above_title_threshold() {
    let pool = fresh_db().await;
    let svc = OutcomeService::new(pool);
    let _ = svc
        .create_outcome(
            CreateOutcomeInput {
                r#type: "paper".into(),
                title: "Advances in Quantum Computing".into(),
                abstract_snippet: "Survey of recent quantum computing breakthroughs".into(),
                certificate_number: None,
            },
            "u-reviewer",
        )
        .await
        .unwrap();
    let res = svc
        .create_outcome(
            CreateOutcomeInput {
                r#type: "paper".into(),
                title: "Advances in Quantum Computing.".into(),
                abstract_snippet: "Different abstract entirely".into(),
                certificate_number: None,
            },
            "u-reviewer",
        )
        .await
        .unwrap();
    assert!(
        !res.duplicate_candidates.is_empty(),
        "high-similarity title should produce a candidate"
    );
}

#[tokio::test]
async fn test_duplicate_detection_below_threshold() {
    let pool = fresh_db().await;
    let svc = OutcomeService::new(pool);
    let _ = svc
        .create_outcome(
            CreateOutcomeInput {
                r#type: "paper".into(),
                title: "Advances in Quantum Computing".into(),
                abstract_snippet: "Survey".into(),
                certificate_number: None,
            },
            "u-reviewer",
        )
        .await
        .unwrap();
    let res = svc
        .create_outcome(
            CreateOutcomeInput {
                r#type: "paper".into(),
                title: "Cellular Automata in Biology".into(),
                abstract_snippet: "Wholly unrelated topic".into(),
                certificate_number: None,
            },
            "u-reviewer",
        )
        .await
        .unwrap();
    assert!(res.duplicate_candidates.is_empty());
}
