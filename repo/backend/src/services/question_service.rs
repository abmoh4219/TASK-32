//! Question bank service. Owns CRUD on `questions` plus the
//! `knowledge_question_links` join table.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::knowledge::Question;

#[derive(Clone)]
pub struct QuestionService {
    pub db: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionInput {
    pub knowledge_point_id: Option<String>,
    pub question_text: String,
    pub question_type: String,
    pub options: Vec<String>,
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub chapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateQuestionInput {
    pub knowledge_point_id: Option<String>,
    pub question_text: Option<String>,
    pub question_type: Option<String>,
    pub options: Option<Vec<String>>,
    pub correct_answer: Option<String>,
    pub explanation: Option<String>,
    pub chapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuestionFilter {
    pub knowledge_point_id: Option<String>,
    pub chapter: Option<String>,
}

impl QuestionService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create(&self, input: CreateQuestionInput, actor_id: &str) -> AppResult<Question> {
        if input.question_text.trim().is_empty() {
            return Err(AppError::Validation("question text is required".into()));
        }
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let opts = serde_json::to_string(&input.options).unwrap_or_else(|_| "[]".to_string());
        sqlx::query(
            "INSERT INTO questions (id, knowledge_point_id, question_text, question_type, options, correct_answer, explanation, chapter, created_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&input.knowledge_point_id)
        .bind(&input.question_text)
        .bind(&input.question_type)
        .bind(&opts)
        .bind(&input.correct_answer)
        .bind(&input.explanation)
        .bind(&input.chapter)
        .bind(actor_id)
        .bind(&now)
        .execute(&self.db)
        .await?;
        self.get(&id).await
    }

    pub async fn get(&self, id: &str) -> AppResult<Question> {
        let row = sqlx::query_as::<_, Question>("SELECT * FROM questions WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db)
            .await?;
        row.ok_or(AppError::NotFound)
    }

    pub async fn update(&self, id: &str, input: UpdateQuestionInput) -> AppResult<Question> {
        let current = self.get(id).await?;
        let kp = input.knowledge_point_id.or(current.knowledge_point_id);
        let text = input.question_text.unwrap_or(current.question_text);
        let qtype = input.question_type.unwrap_or(current.question_type);
        let opts = match input.options {
            Some(o) => serde_json::to_string(&o).unwrap_or_else(|_| "[]".to_string()),
            None => current.options,
        };
        let answer = input.correct_answer.unwrap_or(current.correct_answer);
        let explanation = input.explanation.or(current.explanation);
        let chapter = input.chapter.or(current.chapter);
        sqlx::query(
            "UPDATE questions SET knowledge_point_id = ?, question_text = ?, question_type = ?, options = ?, correct_answer = ?, explanation = ?, chapter = ? WHERE id = ?",
        )
        .bind(&kp)
        .bind(&text)
        .bind(&qtype)
        .bind(&opts)
        .bind(&answer)
        .bind(&explanation)
        .bind(&chapter)
        .bind(id)
        .execute(&self.db)
        .await?;
        self.get(id).await
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        let mut tx = self.db.begin().await?;
        sqlx::query("DELETE FROM knowledge_question_links WHERE question_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        let res = sqlx::query("DELETE FROM questions WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn link_to_kp(&self, question_id: &str, kp_id: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT OR IGNORE INTO knowledge_question_links (knowledge_point_id, question_id, linked_at) VALUES (?, ?, ?)",
        )
        .bind(kp_id)
        .bind(question_id)
        .bind(&now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn unlink_from_kp(&self, question_id: &str, kp_id: &str) -> AppResult<()> {
        sqlx::query(
            "DELETE FROM knowledge_question_links WHERE knowledge_point_id = ? AND question_id = ?",
        )
        .bind(kp_id)
        .bind(question_id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn filter(&self, filter: &QuestionFilter) -> AppResult<Vec<Question>> {
        let mut q = String::from("SELECT * FROM questions WHERE 1=1");
        let mut binds: Vec<String> = Vec::new();
        if let Some(kp) = &filter.knowledge_point_id {
            q.push_str(" AND knowledge_point_id = ?");
            binds.push(kp.clone());
        }
        if let Some(ch) = &filter.chapter {
            q.push_str(" AND chapter = ?");
            binds.push(ch.clone());
        }
        q.push_str(" ORDER BY created_at DESC LIMIT 500");
        let mut query = sqlx::query_as::<_, Question>(&q);
        for b in &binds {
            query = query.bind(b);
        }
        Ok(query.fetch_all(&self.db).await?)
    }
}
