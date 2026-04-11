//! Knowledge API client wrappers — categories, knowledge points, questions.

use serde::{Deserialize, Serialize};

use crate::api::client::{delete_json, get_json, post_json, put_json, ApiError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub level: i64,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryNode {
    pub category: Category,
    pub children: Vec<CategoryNode>,
    pub kp_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgePoint {
    pub id: String,
    pub category_id: String,
    pub title: String,
    pub content: String,
    pub difficulty: i64,
    pub discrimination: f64,
    pub tags: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Question {
    pub id: String,
    pub knowledge_point_id: Option<String>,
    pub question_text: String,
    pub question_type: String,
    pub options: String,
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub chapter: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceCount {
    pub direct_kp_count: i64,
    pub child_category_count: i64,
    pub indirect_question_count: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPreview {
    pub kp_id: String,
    pub field: String,
    pub current_value: String,
    pub proposed_value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCategoryInput {
    pub name: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateKnowledgePointInput {
    pub category_id: String,
    pub title: String,
    pub content: String,
    pub difficulty: i64,
    pub discrimination: f64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergeRequest {
    pub source_id: String,
    pub target_id: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BulkUpdate {
    pub category_id: Option<String>,
    pub difficulty: Option<i64>,
    pub discrimination: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkUpdateRequest {
    pub ids: Vec<String>,
    pub changes: BulkUpdate,
}

pub async fn list_categories() -> Result<Vec<Category>, ApiError> {
    get_json("/api/knowledge/categories").await
}

pub async fn get_category_tree() -> Result<Vec<CategoryNode>, ApiError> {
    get_json("/api/knowledge/categories/tree").await
}

pub async fn create_category(input: CreateCategoryInput) -> Result<Category, ApiError> {
    post_json("/api/knowledge/categories", &input).await
}

pub async fn delete_category(id: &str) -> Result<serde_json::Value, ApiError> {
    delete_json(&format!("/api/knowledge/categories/{}", id)).await
}

pub async fn get_reference_count(id: &str) -> Result<ReferenceCount, ApiError> {
    get_json(&format!("/api/knowledge/categories/{}/references", id)).await
}

pub async fn merge_categories(req: MergeRequest) -> Result<serde_json::Value, ApiError> {
    post_json("/api/knowledge/categories/merge", &req).await
}

pub async fn list_knowledge_points(
    filter: Option<KnowledgeFilter>,
) -> Result<Vec<KnowledgePoint>, ApiError> {
    let qs = filter.map(|f| f.to_query_string()).unwrap_or_default();
    let path = if qs.is_empty() {
        "/api/knowledge/points".to_string()
    } else {
        format!("/api/knowledge/points?{}", qs)
    };
    get_json(&path).await
}

pub async fn create_knowledge_point(
    input: CreateKnowledgePointInput,
) -> Result<KnowledgePoint, ApiError> {
    post_json("/api/knowledge/points", &input).await
}

pub async fn delete_knowledge_point(id: &str) -> Result<serde_json::Value, ApiError> {
    delete_json(&format!("/api/knowledge/points/{}", id)).await
}

pub async fn bulk_preview(
    req: BulkUpdateRequest,
) -> Result<Vec<ConflictPreview>, ApiError> {
    post_json("/api/knowledge/points/bulk/preview", &req).await
}

pub async fn bulk_apply(req: BulkUpdateRequest) -> Result<serde_json::Value, ApiError> {
    post_json("/api/knowledge/points/bulk/apply", &req).await
}

pub async fn list_questions() -> Result<Vec<Question>, ApiError> {
    get_json("/api/knowledge/questions").await
}

#[allow(dead_code)]
pub async fn _put_kp(id: &str, body: serde_json::Value) -> Result<KnowledgePoint, ApiError> {
    put_json(&format!("/api/knowledge/points/{}", id), &body).await
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeFilter {
    pub category_id: Option<String>,
    pub difficulty_min: Option<i64>,
    pub difficulty_max: Option<i64>,
    pub discrimination_min: Option<f64>,
    pub discrimination_max: Option<f64>,
    pub tag: Option<String>,
}

impl KnowledgeFilter {
    pub fn to_query_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(c) = &self.category_id {
            parts.push(format!("category_id={}", c));
        }
        if let Some(v) = self.difficulty_min {
            parts.push(format!("difficulty_min={}", v));
        }
        if let Some(v) = self.difficulty_max {
            parts.push(format!("difficulty_max={}", v));
        }
        if let Some(v) = self.discrimination_min {
            parts.push(format!("discrimination_min={}", v));
        }
        if let Some(v) = self.discrimination_max {
            parts.push(format!("discrimination_max={}", v));
        }
        if let Some(t) = &self.tag {
            parts.push(format!("tag={}", t));
        }
        parts.join("&")
    }
}
