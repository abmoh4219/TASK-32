//! Analytics API client wrappers.

use serde::{Deserialize, Serialize};

use crate::api::client::{get_json, post_json, ApiError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberSnapshot {
    pub id: String,
    pub snapshot_date: String,
    pub total_members: i64,
    pub new_members: i64,
    pub churned_members: i64,
    pub created_at: String,
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
pub struct FundTransaction {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub amount: f64,
    pub category: String,
    pub description: String,
    pub budget_period: String,
    pub recorded_by: String,
    pub created_at: String,
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
pub struct EventParticipation {
    pub id: String,
    pub event_name: String,
    pub event_date: String,
    pub participant_count: i64,
    pub category: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub total_events: i64,
    pub total_participants: i64,
    pub events: Vec<EventParticipation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalCycleRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub submitted_at: String,
    pub approved_at: Option<String>,
    pub approver_id: Option<String>,
    pub cycle_time_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStats {
    pub count: i64,
    pub avg_minutes: f64,
    pub median_minutes: f64,
    pub slowest: Vec<ApprovalCycleRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduledReport {
    pub id: String,
    pub report_type: String,
    pub filters: String,
    pub status: String,
    pub file_path: Option<String>,
    pub download_token: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScheduleReportRequest {
    pub report_type: String,
    pub format: String,
    pub period: Option<String>,
    /// Optional ISO date range start (e.g. "2026-01-01").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from: Option<String>,
    /// Optional ISO date range end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to: Option<String>,
    /// Optional fund category filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

pub async fn members() -> Result<MemberMetrics, ApiError> {
    get_json("/api/analytics/members").await
}

pub async fn churn() -> Result<ChurnRate, ApiError> {
    get_json("/api/analytics/churn").await
}

pub async fn events() -> Result<EventSummary, ApiError> {
    get_json("/api/analytics/events").await
}

pub async fn fund_summary() -> Result<FundSummary, ApiError> {
    get_json("/api/analytics/funds").await
}

pub async fn approval_cycles() -> Result<ApprovalStats, ApiError> {
    get_json("/api/analytics/approval-cycles").await
}

pub async fn schedule_report(req: ScheduleReportRequest) -> Result<ScheduledReport, ApiError> {
    post_json("/api/analytics/reports/schedule", &req).await
}

pub async fn list_reports() -> Result<Vec<ScheduledReport>, ApiError> {
    get_json("/api/analytics/reports").await
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportRequest {
    pub report_type: String,
    pub period: Option<String>,
}

/// Direct CSV/PDF export paths are form-POSTs that stream file bytes. The
/// simplest way to trigger a browser download is to open a POST-enabled URL
/// inside a form submission, but for the API client we just expose the paths
/// and the request shape so the Leptos page can either open them in a new
/// tab or POST via fetch and convert the blob. We keep the helper simple and
/// focused on request-shape correctness.
pub fn csv_export_path() -> &'static str {
    "/api/analytics/export/csv"
}

pub fn pdf_export_path() -> &'static str {
    "/api/analytics/export/pdf"
}
