//! Backend unit tests for the analytics service.

use std::str::FromStr;

use backend::services::analytics_service::{AnalyticsService, FUND_BUDGET_CAP};
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

fn svc(db: SqlitePool) -> AnalyticsService {
    AnalyticsService::new(db, std::env::temp_dir())
}

#[tokio::test]
async fn test_churn_rate_calculation_formula() {
    let pool = fresh_db().await;
    let s = svc(pool);
    // Seed has snapshot 2026-04-01 (240 total, 4 churned) and 2026-03-01
    // (232 total, 6 churned). Latest is 2026-04-01 → churn = 4/232 ≈ 1.72%.
    let r = s.get_churn_rate().await.unwrap();
    assert!((r.rate_pct - (4.0 / 232.0 * 100.0)).abs() < 0.001);
}

#[tokio::test]
async fn test_fund_summary_under_budget_no_flag() {
    let pool = fresh_db().await;
    let s = svc(pool);
    // Seed expense total = 420 + 180 = 600 < 2500 cap.
    let f = s.get_fund_summary(None).await.unwrap();
    assert!(!f.over_budget);
    assert!((f.total_expense - 600.0).abs() < 1e-6);
    assert!((f.total_income - 2250.0).abs() < 1e-6);
    assert_eq!(f.budget_cap, FUND_BUDGET_CAP);
}

#[tokio::test]
async fn test_fund_summary_over_budget_flag() {
    let pool = fresh_db().await;
    // Insert one large expense to push over the cap.
    sqlx::query(
        "INSERT INTO fund_transactions (id, type, amount, category, description, budget_period, recorded_by, created_at) VALUES (?, 'expense', ?, 'capex', '', '2026-04', 'u-finance', datetime('now'))",
    )
    .bind("fund-test-big")
    .bind(3000.0)
    .execute(&pool)
    .await
    .unwrap();
    let s = svc(pool);
    let f = s.get_fund_summary(None).await.unwrap();
    assert!(f.over_budget, "expense > $2500 must trigger over_budget");
}

#[tokio::test]
async fn test_csv_output_has_correct_headers() {
    let pool = fresh_db().await;
    let s = svc(pool);
    let bytes = s.generate_csv("fund", None).await.unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let header = text.lines().next().unwrap();
    assert_eq!(
        header,
        "id,type,amount,category,description,period,recorded_by,created_at"
    );
    assert!(text.lines().count() > 1, "should have at least one data row");
}

#[tokio::test]
async fn test_approval_cycle_average_calculation() {
    let pool = fresh_db().await;
    // Insert three approval records with cycle times 30, 60, 120.
    for (id, mins) in [("a-1", 30), ("a-2", 60), ("a-3", 120)] {
        sqlx::query(
            "INSERT INTO approval_cycle_records (id, entity_type, entity_id, submitted_at, approved_at, approver_id, cycle_time_minutes) VALUES (?, 'outcome', 'x', '2026-01-01T00:00:00Z', '2026-01-01T01:00:00Z', 'u-admin', ?)",
        )
        .bind(id)
        .bind(mins)
        .execute(&pool)
        .await
        .unwrap();
    }
    let s = svc(pool);
    let stats = s.get_approval_cycle_stats().await.unwrap();
    assert_eq!(stats.count, 3);
    assert!((stats.avg_minutes - 70.0).abs() < 1e-6); // (30+60+120)/3 = 70
}

#[tokio::test]
async fn test_pdf_generation_starts_with_pdf_header() {
    let pool = fresh_db().await;
    let s = svc(pool);
    let bytes = s.generate_pdf("fund", None).await.unwrap();
    assert!(bytes.starts_with(b"%PDF"), "must be a real PDF document");
    assert!(bytes.len() > 200, "non-trivial PDF length");
}

#[tokio::test]
async fn test_schedule_report_marks_complete_with_token() {
    let pool = fresh_db().await;
    let tmp = std::env::temp_dir().join(format!("sv-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp).unwrap();
    let s = AnalyticsService::new(pool, tmp);
    let row = s
        .schedule_report("fund", "csv", None, "u-finance")
        .await
        .unwrap();
    assert_eq!(row.status, "complete");
    assert!(row.download_token.is_some());
    assert!(row.file_path.is_some());
}

#[tokio::test]
async fn test_download_token_single_use_clears() {
    let pool = fresh_db().await;
    let tmp = std::env::temp_dir().join(format!("sv-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp).unwrap();
    let s = AnalyticsService::new(pool, tmp);
    let row = s
        .schedule_report("fund", "csv", None, "u-finance")
        .await
        .unwrap();
    let token = row.download_token.unwrap();
    let (_, bytes) = s.download_report(&row.id, &token).await.unwrap();
    assert!(!bytes.is_empty());
    // Second attempt with the same token must fail.
    let err = s.download_report(&row.id, &token).await.unwrap_err();
    assert!(matches!(err, backend::AppError::NotFound));
}
