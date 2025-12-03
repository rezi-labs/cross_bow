use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Commit {
    pub id: i64,
    pub repository_id: i64,
    pub webhook_event_id: i64,
    pub sha: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committed_at: DateTime<Utc>,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommit {
    pub repository_id: i64,
    pub webhook_event_id: i64,
    pub sha: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committed_at: DateTime<Utc>,
    pub url: String,
}

impl Commit {
    pub async fn create(pool: &sqlx::PgPool, data: CreateCommit) -> Result<Self, sqlx::Error> {
        let commit = sqlx::query_as::<_, Commit>(
            r#"
            INSERT INTO commits (repository_id, webhook_event_id, sha, message, author_name, author_email, committer_name, committer_email, committed_at, url)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (sha, repository_id) DO UPDATE
            SET message = EXCLUDED.message,
                author_name = EXCLUDED.author_name,
                author_email = EXCLUDED.author_email,
                committer_name = EXCLUDED.committer_name,
                committer_email = EXCLUDED.committer_email,
                committed_at = EXCLUDED.committed_at,
                url = EXCLUDED.url
            RETURNING *
            "#,
        )
        .bind(data.repository_id)
        .bind(data.webhook_event_id)
        .bind(data.sha)
        .bind(data.message)
        .bind(data.author_name)
        .bind(data.author_email)
        .bind(data.committer_name)
        .bind(data.committer_email)
        .bind(data.committed_at)
        .bind(data.url)
        .fetch_one(pool)
        .await?;

        Ok(commit)
    }

    pub async fn list_by_repository(
        pool: &sqlx::PgPool,
        repository_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let commits = sqlx::query_as::<_, Commit>(
            "SELECT * FROM commits WHERE repository_id = $1 ORDER BY committed_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repository_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(commits)
    }

    pub async fn list_all(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let commits = sqlx::query_as::<_, Commit>(
            "SELECT * FROM commits ORDER BY committed_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(commits)
    }

    pub async fn list_by_author(
        pool: &sqlx::PgPool,
        author_email: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let commits = sqlx::query_as::<_, Commit>(
            "SELECT * FROM commits WHERE author_email = $1 ORDER BY committed_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(author_email)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(commits)
    }

    pub async fn count(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM commits")
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }

    pub async fn count_by_repository(
        pool: &sqlx::PgPool,
        repository_id: i64,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM commits WHERE repository_id = $1")
            .bind(repository_id)
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }
}
