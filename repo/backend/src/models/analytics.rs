//! Analytics + scheduled report row mappings.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemberSnapshot {
    pub id: String,
    pub snapshot_date: String,
    pub total_members: i64,
    pub new_members: i64,
    pub churned_members: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventParticipation {
    pub id: String,
    pub event_name: String,
    pub event_date: String,
    pub participant_count: i64,
    pub category: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FundTransaction {
    pub id: String,
    pub r#type: String,
    pub amount: f64,
    pub category: String,
    pub description: String,
    pub budget_period: String,
    pub recorded_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApprovalCycleRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub submitted_at: String,
    pub approved_at: Option<String>,
    pub approver_id: Option<String>,
    pub cycle_time_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
