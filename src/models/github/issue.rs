use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Issue {
    pub id: i64,
    pub repository_id: i64,
    pub webhook_event_id: i64,
    pub github_id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub author: String,
    pub labels: Vec<String>,
    pub url: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssue {
    pub repository_id: i64,
    pub webhook_event_id: i64,
    pub github_id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub author: String,
    pub labels: Vec<String>,
    pub url: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl Issue {
    pub async fn create(pool: &sqlx::PgPool, data: CreateIssue) -> Result<Self, sqlx::Error> {
        let issue = sqlx::query_as::<_, Issue>(
            r#"
            INSERT INTO issues (repository_id, webhook_event_id, github_id, number, title, state, author, labels, url, opened_at, closed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (github_id) DO UPDATE
            SET title = EXCLUDED.title,
                state = EXCLUDED.state,
                author = EXCLUDED.author,
                labels = EXCLUDED.labels,
                url = EXCLUDED.url,
                closed_at = EXCLUDED.closed_at,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(data.repository_id)
        .bind(data.webhook_event_id)
        .bind(data.github_id)
        .bind(data.number)
        .bind(data.title)
        .bind(data.state)
        .bind(data.author)
        .bind(data.labels)
        .bind(data.url)
        .bind(data.opened_at)
        .bind(data.closed_at)
        .fetch_one(pool)
        .await?;

        Ok(issue)
    }

    pub async fn list_by_repository(
        pool: &sqlx::PgPool,
        repository_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let issues = sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE repository_id = $1 ORDER BY opened_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repository_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(issues)
    }

    #[allow(dead_code)]
    pub async fn list_all(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let issues = sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues ORDER BY opened_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(issues)
    }

    #[allow(dead_code)]
    pub async fn list_by_state(
        pool: &sqlx::PgPool,
        state: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let issues = sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE state = $1 ORDER BY opened_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(state)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(issues)
    }

    pub async fn count(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM issues")
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }

    pub async fn count_by_state(pool: &sqlx::PgPool, state: &str) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM issues WHERE state = $1")
            .bind(state)
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }
}
