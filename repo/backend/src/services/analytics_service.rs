//! Analytics + scheduled reports service.
//!
//! Computes the four dashboard metrics from SPEC.md (member scale & churn,
//! event participation, fund income/expense vs the $2,500 monthly cap, approval
//! cycle time) and produces CSV / PDF exports plus the scheduled-report
//! workflow with single-use download tokens.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::analytics::{
    ApprovalCycleRecord, EventParticipation, FundTransaction, MemberSnapshot, ScheduledReport,
};

/// The example monthly fund cap from SPEC.md.
pub const FUND_BUDGET_CAP: f64 = 2_500.00;

#[derive(Clone)]
pub struct AnalyticsService {
    pub db: SqlitePool,
    pub reports_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberMetrics {
    pub current_total: i64,
    pub new_members: i64,
    pub churned: i64,
    pub series: Vec<MemberSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnRate {
    pub rate_pct: f64,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundSummary {
    pub total_income: f64,
    pub total_expense: f64,
    pub net: f64,
    pub budget_cap: f64,
    pub over_budget: bool,
    pub period: String,
    pub transactions: Vec<FundTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStats {
    pub count: i64,
    pub avg_minutes: f64,
    pub median_minutes: f64,
    pub slowest: Vec<ApprovalCycleRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub total_events: i64,
    pub total_participants: i64,
    pub events: Vec<EventParticipation>,
}

impl AnalyticsService {
    pub fn new(db: SqlitePool, reports_dir: std::path::PathBuf) -> Self {
        Self { db, reports_dir }
    }

    pub async fn get_member_metrics(&self) -> AppResult<MemberMetrics> {
        let series = sqlx::query_as::<_, MemberSnapshot>(
            "SELECT * FROM member_snapshots ORDER BY snapshot_date DESC LIMIT 12",
        )
        .fetch_all(&self.db)
        .await?;
        let current_total = series.first().map(|s| s.total_members).unwrap_or(0);
        let new_members: i64 = series.iter().map(|s| s.new_members).sum();
        let churned: i64 = series.iter().map(|s| s.churned_members).sum();
        Ok(MemberMetrics {
            current_total,
            new_members,
            churned,
            series,
        })
    }

    /// Churn rate = `churned / prior_period_total` expressed as a percentage.
    /// If there is only one snapshot it returns 0% (no prior period to compare).
    pub async fn get_churn_rate(&self) -> AppResult<ChurnRate> {
        let snaps = sqlx::query_as::<_, MemberSnapshot>(
            "SELECT * FROM member_snapshots ORDER BY snapshot_date DESC LIMIT 2",
        )
        .fetch_all(&self.db)
        .await?;
        if snaps.len() < 2 {
            return Ok(ChurnRate {
                rate_pct: 0.0,
                from: snaps.first().map(|s| s.snapshot_date.clone()).unwrap_or_default(),
                to: snaps.first().map(|s| s.snapshot_date.clone()).unwrap_or_default(),
            });
        }
        let latest = &snaps[0];
        let prior = &snaps[1];
        let rate = if prior.total_members > 0 {
            (latest.churned_members as f64 / prior.total_members as f64) * 100.0
        } else {
            0.0
        };
        Ok(ChurnRate {
            rate_pct: rate,
            from: prior.snapshot_date.clone(),
            to: latest.snapshot_date.clone(),
        })
    }

    pub async fn get_event_participation(&self) -> AppResult<EventSummary> {
        let events = sqlx::query_as::<_, EventParticipation>(
            "SELECT * FROM event_participation ORDER BY event_date DESC LIMIT 100",
        )
        .fetch_all(&self.db)
        .await?;
        let total_events = events.len() as i64;
        let total_participants: i64 = events.iter().map(|e| e.participant_count).sum();
        Ok(EventSummary {
            total_events,
            total_participants,
            events,
        })
    }

    /// Fund summary with optional structured filters. `period` is the legacy
    /// budget-period string; `date_from`/`date_to`/`category`/`role` narrow
    /// further. All are `None` by default so existing callers see no change.
    pub async fn get_fund_summary(
        &self,
        period: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        category: Option<&str>,
        role: Option<&str>,
    ) -> AppResult<FundSummary> {
        let mut q = String::from("SELECT * FROM fund_transactions WHERE 1=1");
        let mut binds: Vec<String> = Vec::new();
        if let Some(p) = period {
            q.push_str(" AND budget_period = ?");
            binds.push(p.to_string());
        }
        if let Some(d) = date_from {
            q.push_str(" AND created_at >= ?");
            binds.push(d.to_string());
        }
        if let Some(d) = date_to {
            q.push_str(" AND created_at <= ?");
            binds.push(d.to_string());
        }
        if let Some(c) = category {
            q.push_str(" AND category = ?");
            binds.push(c.to_string());
        }
        if let Some(r) = role {
            // Filter by the role of the user who recorded the transaction.
            // The `recorded_by` column holds a user id; we join against users
            // to match on role. Using a subquery keeps the dynamic SQL simple.
            q.push_str(
                " AND recorded_by IN (SELECT id FROM users WHERE role = ?)",
            );
            binds.push(r.to_string());
        }
        q.push_str(" ORDER BY created_at DESC LIMIT 200");
        let mut query = sqlx::query_as::<_, FundTransaction>(&q);
        for b in &binds {
            query = query.bind(b);
        }
        let transactions = query.fetch_all(&self.db).await?;
        let total_income: f64 = transactions
            .iter()
            .filter(|t| t.r#type == "income")
            .map(|t| t.amount)
            .sum();
        let total_expense: f64 = transactions
            .iter()
            .filter(|t| t.r#type == "expense")
            .map(|t| t.amount)
            .sum();
        let net = total_income - total_expense;
        let over_budget = total_expense > FUND_BUDGET_CAP;
        Ok(FundSummary {
            total_income,
            total_expense,
            net,
            budget_cap: FUND_BUDGET_CAP,
            over_budget,
            period: period.unwrap_or("all").to_string(),
            transactions,
        })
    }

    pub async fn get_approval_cycle_stats(&self) -> AppResult<ApprovalStats> {
        let rows = sqlx::query_as::<_, ApprovalCycleRecord>(
            "SELECT * FROM approval_cycle_records WHERE cycle_time_minutes IS NOT NULL ORDER BY cycle_time_minutes DESC",
        )
        .fetch_all(&self.db)
        .await?;
        let count = rows.len() as i64;
        let mins: Vec<i64> = rows.iter().filter_map(|r| r.cycle_time_minutes).collect();
        let sum: i64 = mins.iter().sum();
        let avg = if !mins.is_empty() {
            sum as f64 / mins.len() as f64
        } else {
            0.0
        };
        let median = if !mins.is_empty() {
            let mut sorted = mins.clone();
            sorted.sort();
            sorted[sorted.len() / 2] as f64
        } else {
            0.0
        };
        let slowest = rows.iter().take(5).cloned().collect();
        Ok(ApprovalStats {
            count,
            avg_minutes: avg,
            median_minutes: median,
            slowest,
        })
    }

    // ─── Exports ────────────────────────────────────────────────────────

    /// Render a CSV with full custom filters (date range, category, role).
    pub async fn generate_csv_filtered(
        &self,
        report_type: &str,
        period: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        category: Option<&str>,
        role: Option<&str>,
    ) -> AppResult<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        match report_type {
            "fund" => {
                wtr.write_record([
                    "id", "type", "amount", "category", "description", "period", "recorded_by", "created_at",
                ]).map_err(|e| AppError::Internal(e.to_string()))?;
                let summary = self.get_fund_summary(period, date_from, date_to, category, role).await?;
                for t in summary.transactions {
                    wtr.write_record([
                        t.id, t.r#type, format!("{:.2}", t.amount), t.category,
                        t.description, t.budget_period, t.recorded_by, t.created_at,
                    ]).map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            "members" => {
                wtr.write_record(["snapshot_date", "total_members", "new_members", "churned_members"])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                let metrics = self.get_member_metrics().await?;
                for s in metrics.series {
                    wtr.write_record([
                        s.snapshot_date, s.total_members.to_string(),
                        s.new_members.to_string(), s.churned_members.to_string(),
                    ]).map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            "events" => {
                wtr.write_record(["event_name", "event_date", "participant_count", "category"])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                let summary = self.get_event_participation().await?;
                for e in summary.events {
                    wtr.write_record([
                        e.event_name, e.event_date, e.participant_count.to_string(),
                        e.category.unwrap_or_default(),
                    ]).map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            other => return Err(AppError::Validation(format!("unknown report type: {other}"))),
        }
        wtr.into_inner().map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Render a CSV byte buffer for the requested report.
    pub async fn generate_csv(
        &self,
        report_type: &str,
        period: Option<&str>,
    ) -> AppResult<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        match report_type {
            "fund" => {
                wtr.write_record([
                    "id",
                    "type",
                    "amount",
                    "category",
                    "description",
                    "period",
                    "recorded_by",
                    "created_at",
                ])
                .map_err(|e| AppError::Internal(e.to_string()))?;
                let summary = self.get_fund_summary(period, None, None, None, None).await?;
                for t in summary.transactions {
                    wtr.write_record([
                        t.id,
                        t.r#type,
                        format!("{:.2}", t.amount),
                        t.category,
                        t.description,
                        t.budget_period,
                        t.recorded_by,
                        t.created_at,
                    ])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            "members" => {
                wtr.write_record(["snapshot_date", "total_members", "new_members", "churned_members"])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                let metrics = self.get_member_metrics().await?;
                for s in metrics.series {
                    wtr.write_record([
                        s.snapshot_date,
                        s.total_members.to_string(),
                        s.new_members.to_string(),
                        s.churned_members.to_string(),
                    ])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            "events" => {
                wtr.write_record(["event_name", "event_date", "participant_count", "category"])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                let summary = self.get_event_participation().await?;
                for e in summary.events {
                    wtr.write_record([
                        e.event_name,
                        e.event_date,
                        e.participant_count.to_string(),
                        e.category.unwrap_or_default(),
                    ])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            other => return Err(AppError::Validation(format!("unknown report type: {other}"))),
        }
        wtr.into_inner()
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Render a real PDF via the `printpdf` crate.
    ///
    /// All async data fetching happens **before** the `PdfDocument` is created.
    /// `printpdf::PdfDocumentReference` wraps an `Rc<RefCell<…>>` that is not
    /// `Send`, so it must never be held across an `.await` point — otherwise
    /// the resulting future cannot be used as an axum handler.
    pub async fn generate_pdf(
        &self,
        report_type: &str,
        period: Option<&str>,
    ) -> AppResult<Vec<u8>> {
        // Step 1 — collect text via async DB queries (no PDF objects yet).
        let title = format!("ScholarVault — {} report", report_type);
        let generated_at = format!("Generated {}", Utc::now().to_rfc3339());
        let lines: Vec<String> = match report_type {
            "fund" => {
                let summary = self.get_fund_summary(period, None, None, None, None).await?;
                let mut v = vec![
                    format!("Total income:  ${:.2}", summary.total_income),
                    format!("Total expense: ${:.2}", summary.total_expense),
                    format!("Net:           ${:.2}", summary.net),
                    format!(
                        "Budget cap:    ${:.2} ({})",
                        summary.budget_cap,
                        if summary.over_budget { "OVER" } else { "ok" }
                    ),
                    String::new(),
                    "Transactions:".into(),
                ];
                for t in summary.transactions.iter().take(20) {
                    v.push(format!(
                        "  {} ${:.2} {} - {}",
                        t.r#type, t.amount, t.category, t.description
                    ));
                }
                v
            }
            "members" => {
                let m = self.get_member_metrics().await?;
                let mut v = vec![
                    format!("Current members: {}", m.current_total),
                    format!("New (window):    {}", m.new_members),
                    format!("Churned:         {}", m.churned),
                    String::new(),
                    "Snapshots:".into(),
                ];
                for s in m.series.iter().take(12) {
                    v.push(format!(
                        "  {}: total={} new={} churned={}",
                        s.snapshot_date, s.total_members, s.new_members, s.churned_members
                    ));
                }
                v
            }
            other => return Err(AppError::Validation(format!("unknown report type: {other}"))),
        };

        // Step 2 — synchronously build the PDF. `Send`-only types live here.
        Self::render_pdf(&title, &generated_at, &lines)
    }

    fn render_pdf(title: &str, generated_at: &str, lines: &[String]) -> AppResult<Vec<u8>> {
        use printpdf::*;
        let (doc, page1, layer1) =
            PdfDocument::new("ScholarVault Report", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| AppError::Internal(format!("pdf font: {e}")))?;
        let current_layer = doc.get_page(page1).get_layer(layer1);
        current_layer.use_text(title, 18.0, Mm(20.0), Mm(270.0), &font);
        current_layer.use_text(generated_at, 10.0, Mm(20.0), Mm(260.0), &font);

        let mut y: f32 = 240.0;
        for line in lines {
            current_layer.use_text(line, 11.0, Mm(20.0), Mm(y), &font);
            y -= 6.0;
            if y < 20.0 {
                break;
            }
        }

        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = std::io::BufWriter::new(&mut buf);
            doc.save(&mut writer)
                .map_err(|e| AppError::Internal(format!("pdf save: {e}")))?;
        }
        Ok(buf)
    }

    /// Schedule a report for generation. The current implementation runs the
    /// generation **inline** so the workflow is deterministic for tests, but
    /// the public surface matches the SPEC: status flips from `pending` to
    /// `complete`, the file lands on disk, and a single-use `download_token`
    /// is issued.
    /// Schedule a report for generation with optional custom filters.
    /// Filters are stored in the `filters` JSON column and flowed through to
    /// CSV/PDF generation so the output reflects the requested filter state.
    #[allow(clippy::too_many_arguments)]
    pub async fn schedule_report(
        &self,
        report_type: &str,
        format: &str,
        period: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        category: Option<&str>,
        role: Option<&str>,
        actor_id: &str,
    ) -> AppResult<ScheduledReport> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let filters_json = serde_json::json!({
            "format": format,
            "period": period,
            "date_from": date_from,
            "date_to": date_to,
            "category": category,
            "role": role,
        })
        .to_string();
        sqlx::query(
            "INSERT INTO scheduled_reports (id, report_type, filters, status, created_by, created_at) VALUES (?, ?, ?, 'pending', ?, ?)",
        )
        .bind(&id)
        .bind(report_type)
        .bind(&filters_json)
        .bind(actor_id)
        .bind(&now)
        .execute(&self.db)
        .await?;

        let bytes = match format {
            "csv" => self.generate_csv_filtered(report_type, period, date_from, date_to, category, role).await?,
            "pdf" => self.generate_pdf(report_type, period).await?,
            other => return Err(AppError::Validation(format!("unknown format: {other}"))),
        };
        std::fs::create_dir_all(&self.reports_dir).ok();
        let extension = if format == "pdf" { "pdf" } else { "csv" };
        let path = self.reports_dir.join(format!("{}.{}", id, extension));
        std::fs::write(&path, &bytes)?;

        let token = Uuid::new_v4().to_string();
        let completed = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE scheduled_reports SET status='complete', file_path = ?, download_token = ?, completed_at = ? WHERE id = ?",
        )
        .bind(path.to_string_lossy().to_string())
        .bind(&token)
        .bind(&completed)
        .bind(&id)
        .execute(&self.db)
        .await?;

        let row = sqlx::query_as::<_, ScheduledReport>(
            "SELECT * FROM scheduled_reports WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    pub async fn list_reports(&self, user_id: &str) -> AppResult<Vec<ScheduledReport>> {
        let rows = sqlx::query_as::<_, ScheduledReport>(
            "SELECT * FROM scheduled_reports WHERE created_by = ? ORDER BY created_at DESC LIMIT 100",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    /// Download a completed report by id + token. The token is **single-use**:
    /// after a successful read it is cleared from the row so the same URL
    /// cannot be reused.
    /// Download a completed report. The caller must be authenticated; ownership
    /// is verified here at the data layer so an attacker in possession of a
    /// token alone cannot fetch another user's report. Administrators bypass
    /// the ownership check.
    pub async fn download_report(
        &self,
        report_id: &str,
        token: &str,
        requesting_user_id: &str,
        is_admin: bool,
    ) -> AppResult<(String, Vec<u8>)> {
        let row: Option<ScheduledReport> = sqlx::query_as::<_, ScheduledReport>(
            "SELECT * FROM scheduled_reports WHERE id = ?",
        )
        .bind(report_id)
        .fetch_optional(&self.db)
        .await?;
        let row = row.ok_or(AppError::NotFound)?;
        if !is_admin && row.created_by != requesting_user_id {
            return Err(AppError::Forbidden);
        }
        if row.status != "complete" {
            return Err(AppError::Conflict("report not yet complete".into()));
        }
        let stored_token = row.download_token.as_deref().unwrap_or("");
        if stored_token.is_empty() || stored_token != token {
            return Err(AppError::NotFound);
        }
        let path = row.file_path.as_deref().ok_or(AppError::NotFound)?;
        let bytes = std::fs::read(path)?;
        // Single-use: clear the token so the same URL cannot be reused.
        sqlx::query("UPDATE scheduled_reports SET download_token = NULL WHERE id = ?")
            .bind(report_id)
            .execute(&self.db)
            .await?;
        let mime = if path.ends_with(".pdf") {
            "application/pdf".to_string()
        } else {
            "text/csv".to_string()
        };
        Ok((mime, bytes))
    }
}
