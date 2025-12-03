use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Repository {
    pub id: i64,
    pub github_id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub description: Option<String>,
    pub url: String,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepository {
    pub github_id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub description: Option<String>,
    pub url: String,
    pub is_private: bool,
}

impl Repository {
    pub async fn create(pool: &sqlx::PgPool, data: CreateRepository) -> Result<Self, sqlx::Error> {
        let repo = sqlx::query_as::<_, Repository>(
            r#"
            INSERT INTO repositories (github_id, name, full_name, owner, description, url, is_private)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (github_id) DO UPDATE
            SET name = EXCLUDED.name,
                full_name = EXCLUDED.full_name,
                owner = EXCLUDED.owner,
                description = EXCLUDED.description,
                url = EXCLUDED.url,
                is_private = EXCLUDED.is_private,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(data.github_id)
        .bind(data.name)
        .bind(data.full_name)
        .bind(data.owner)
        .bind(data.description)
        .bind(data.url)
        .bind(data.is_private)
        .fetch_one(pool)
        .await?;

        Ok(repo)
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        let repo = sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(repo)
    }

    pub async fn find_by_full_name(
        pool: &sqlx::PgPool,
        full_name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let repo =
            sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE full_name = $1")
                .bind(full_name)
                .fetch_optional(pool)
                .await?;

        Ok(repo)
    }

    pub async fn list_all(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let repos = sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories ORDER BY updated_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(repos)
    }

    pub async fn count(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repositories")
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }
}
