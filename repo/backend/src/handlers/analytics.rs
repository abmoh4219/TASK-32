//! Analytics HTTP handlers — dashboard metrics, CSV/PDF export, scheduled reports.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use shared::AuditAction;

use crate::error::{AppError, AppResult};
use crate::middleware::require_role::{AuthenticatedUser, RequireFinance};
use crate::models::analytics::ScheduledReport;
use crate::services::analytics_service::{
    AnalyticsService, ApprovalStats, ChurnRate, EventSummary, FundSummary, MemberMetrics,
};
use crate::services::audit_service::AuditService;
use crate::AppState;

fn analytics(state: &AppState) -> AnalyticsService {
    AnalyticsService::new(state.db.clone(), (*state.reports_dir).clone())
}

pub async fn members(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
) -> AppResult<Json<MemberMetrics>> {
    Ok(Json(analytics(&state).get_member_metrics().await?))
}

pub async fn churn(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
) -> AppResult<Json<ChurnRate>> {
    Ok(Json(analytics(&state).get_churn_rate().await?))
}

pub async fn events(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
) -> AppResult<Json<EventSummary>> {
    Ok(Json(analytics(&state).get_event_participation().await?))
}

#[derive(Deserialize, Default)]
pub struct PeriodQuery {
    pub period: Option<String>,
}

pub async fn fund_summary(
    State(state): State<AppState>,
    RequireFinance(_): RequireFinance,
    Query(q): Query<PeriodQuery>,
) -> AppResult<Json<FundSummary>> {
    Ok(Json(analytics(&state).get_fund_summary(q.period.as_deref()).await?))
}

pub async fn approval_cycles(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
) -> AppResult<Json<ApprovalStats>> {
    Ok(Json(analytics(&state).get_approval_cycle_stats().await?))
}

#[derive(Deserialize)]
pub struct ExportRequest {
    pub report_type: String,
    pub period: Option<String>,
}

pub async fn export_csv(
    State(state): State<AppState>,
    RequireFinance(user): RequireFinance,
    Json(req): Json<ExportRequest>,
) -> AppResult<Response> {
    let bytes = analytics(&state)
        .generate_csv(&req.report_type, req.period.as_deref())
        .await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::ExportReport,
            "csv",
            Some(&req.report_type),
            None,
            Some(AuditService::compute_hash(&format!("{} bytes", bytes.len()))),
            None,
        )
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}.csv\"", req.report_type))
            .unwrap_or(HeaderValue::from_static("attachment")),
    );
    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

pub async fn export_pdf(
    State(state): State<AppState>,
    RequireFinance(user): RequireFinance,
    Json(req): Json<ExportRequest>,
) -> AppResult<Response> {
    let bytes = analytics(&state)
        .generate_pdf(&req.report_type, req.period.as_deref())
        .await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::ExportReport,
            "pdf",
            Some(&req.report_type),
            None,
            Some(AuditService::compute_hash(&format!("{} bytes", bytes.len()))),
            None,
        )
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}.pdf\"", req.report_type))
            .unwrap_or(HeaderValue::from_static("attachment")),
    );
    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

#[derive(Deserialize)]
pub struct ScheduleReportRequest {
    pub report_type: String,
    pub format: String,
    pub period: Option<String>,
}

pub async fn schedule_report(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<ScheduleReportRequest>,
) -> AppResult<Json<ScheduledReport>> {
    let row = analytics(&state)
        .schedule_report(&req.report_type, &req.format, req.period.as_deref(), &user.id)
        .await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::ExportReport,
            "scheduled_report",
            Some(&row.id),
            None,
            Some(AuditService::compute_hash(&row.id)),
            None,
        )
        .await?;
    Ok(Json(row))
}

pub async fn list_reports(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> AppResult<Json<Vec<ScheduledReport>>> {
    Ok(Json(analytics(&state).list_reports(&user.id).await?))
}

pub async fn download_report(
    State(state): State<AppState>,
    Path((id, token)): Path<(String, String)>,
) -> AppResult<Response> {
    let (mime, bytes) = analytics(&state).download_report(&id, &token).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment"),
    );
    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

#[allow(dead_code)]
fn _force(_: AppError) {}
