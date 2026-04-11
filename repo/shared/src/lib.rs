//! Shared DTOs used by both backend and frontend.
//! These types form the contract between the Axum API and the Leptos frontend.
//! All types are serializable so they can flow over the JSON HTTP boundary.

use serde::{Deserialize, Serialize};

/// Roles enforced across the entire ScholarVault system.
/// Stored as lowercase strings in SQLite (`role` column).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Administrator,
    ContentCurator,
    Reviewer,
    FinanceManager,
    StoreManager,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Administrator => "administrator",
            UserRole::ContentCurator => "content_curator",
            UserRole::Reviewer => "reviewer",
            UserRole::FinanceManager => "finance_manager",
            UserRole::StoreManager => "store_manager",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "administrator" => Some(UserRole::Administrator),
            "content_curator" => Some(UserRole::ContentCurator),
            "reviewer" => Some(UserRole::Reviewer),
            "finance_manager" => Some(UserRole::FinanceManager),
            "store_manager" => Some(UserRole::StoreManager),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            UserRole::Administrator => "Administrator",
            UserRole::ContentCurator => "Content Curator",
            UserRole::Reviewer => "Reviewer",
            UserRole::FinanceManager => "Finance Manager",
            UserRole::StoreManager => "Store Manager",
        }
    }
}

/// Audit log action enumeration. Stored as a string in `audit_logs.action`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Login,
    Logout,
    LoginFailed,
    Create,
    Update,
    Delete,
    Submit,
    Approve,
    Reject,
    UploadEvidence,
    BulkUpdate,
    MergeNodes,
    RunBackup,
    RestoreSandbox,
    ActivateRestore,
    LifecycleCleanup,
    ExportReport,
    Checkout,
    RoleChange,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Login => "login",
            AuditAction::Logout => "logout",
            AuditAction::LoginFailed => "login_failed",
            AuditAction::Create => "create",
            AuditAction::Update => "update",
            AuditAction::Delete => "delete",
            AuditAction::Submit => "submit",
            AuditAction::Approve => "approve",
            AuditAction::Reject => "reject",
            AuditAction::UploadEvidence => "upload_evidence",
            AuditAction::BulkUpdate => "bulk_update",
            AuditAction::MergeNodes => "merge_nodes",
            AuditAction::RunBackup => "run_backup",
            AuditAction::RestoreSandbox => "restore_sandbox",
            AuditAction::ActivateRestore => "activate_restore",
            AuditAction::LifecycleCleanup => "lifecycle_cleanup",
            AuditAction::ExportReport => "export_report",
            AuditAction::Checkout => "checkout",
            AuditAction::RoleChange => "role_change",
        }
    }
}

/// Outcome categories from the business prompt.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeType {
    Paper,
    Patent,
    CompetitionResult,
    SoftwareCopyright,
}

impl OutcomeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutcomeType::Paper => "paper",
            OutcomeType::Patent => "patent",
            OutcomeType::CompetitionResult => "competition_result",
            OutcomeType::SoftwareCopyright => "software_copyright",
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            OutcomeType::Paper => "Research Paper",
            OutcomeType::Patent => "Patent",
            OutcomeType::CompetitionResult => "Competition Result",
            OutcomeType::SoftwareCopyright => "Software Copyright",
        }
    }
}

/// Promotion discount types — percentage of price or a fixed amount off.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PromotionType {
    Percent,
    Fixed,
}

impl PromotionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PromotionType::Percent => "percent",
            PromotionType::Fixed => "fixed",
        }
    }
}

/// Difficulty rating constrained to 1..=5 by the SQLite CHECK constraint.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DifficultyLevel(pub u8);

impl DifficultyLevel {
    pub fn new(value: u8) -> Option<Self> {
        if (1..=5).contains(&value) { Some(Self(value)) } else { None }
    }
}

/// Standard success envelope returned by API endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self { Self { success: true, data } }
}

/// Standard error envelope returned with non-2xx HTTP responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub timestamp: String,
}

/// Discrimination band classification used by analytics filters and the UI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscriminationBand {
    Poor,
    Acceptable,
    Good,
    Excellent,
}

impl DiscriminationBand {
    /// Range thresholds defined in CLAUDE.md Open Questions:
    /// Poor < 0.1, Acceptable 0.1..0.3, Good 0.3..0.5, Excellent > 0.5
    pub fn range(&self) -> (f64, f64) {
        match self {
            DiscriminationBand::Poor => (-1.0, 0.1),
            DiscriminationBand::Acceptable => (0.1, 0.3),
            DiscriminationBand::Good => (0.3, 0.5),
            DiscriminationBand::Excellent => (0.5, 1.0001),
        }
    }
}
