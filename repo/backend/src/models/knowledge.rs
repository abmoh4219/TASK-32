//! Knowledge graph row mappings: categories form a DAG, knowledge_points hang
//! off categories, questions are many-to-many with knowledge_points.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KnowledgePoint {
    pub id: String,
    pub category_id: String,
    pub title: String,
    pub content: String,
    pub difficulty: i64,
    pub discrimination: f64,
    pub tags: String, // JSON-encoded array
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Question {
    pub id: String,
    pub knowledge_point_id: Option<String>,
    pub question_text: String,
    pub question_type: String,
    pub options: String, // JSON-encoded array
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub chapter: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KnowledgeQuestionLink {
    pub knowledge_point_id: String,
    pub question_id: String,
    pub linked_at: String,
}
