use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookEvent {
    pub id: i64,
    pub repository_id: Option<i64>,
    pub event_type: String,
    pub event_action: Option<String>,
    pub delivery_id: Uuid,
    pub payload: JsonValue,
    pub signature: String,
    pub received_at: DateTime<Utc>,
    pub processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookEvent {
    pub repository_id: Option<i64>,
    pub event_type: String,
    pub event_action: Option<String>,
    pub delivery_id: Uuid,
    pub payload: JsonValue,
    pub signature: String,
}

impl WebhookEvent {
    pub async fn create(
        pool: &sqlx::PgPool,
        data: CreateWebhookEvent,
    ) -> Result<Self, sqlx::Error> {
        let event = sqlx::query_as::<_, WebhookEvent>(
            r#"
            INSERT INTO webhook_events (repository_id, event_type, event_action, delivery_id, payload, signature)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(data.repository_id)
        .bind(data.event_type)
        .bind(data.event_action)
        .bind(data.delivery_id)
        .bind(data.payload)
        .bind(data.signature)
        .fetch_one(pool)
        .await?;

        Ok(event)
    }

    pub async fn mark_processed(pool: &sqlx::PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE webhook_events SET processed = true, processed_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        let event = sqlx::query_as::<_, WebhookEvent>("SELECT * FROM webhook_events WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(event)
    }

    pub async fn list_by_repository(
        pool: &sqlx::PgPool,
        repository_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let events = sqlx::query_as::<_, WebhookEvent>(
            "SELECT * FROM webhook_events WHERE repository_id = $1 ORDER BY received_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repository_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    pub async fn list_all(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let events = sqlx::query_as::<_, WebhookEvent>(
            "SELECT * FROM webhook_events ORDER BY received_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    pub async fn list_by_type(
        pool: &sqlx::PgPool,
        event_type: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let events = sqlx::query_as::<_, WebhookEvent>(
            "SELECT * FROM webhook_events WHERE event_type = $1 ORDER BY received_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(event_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    pub async fn count(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM webhook_events")
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }
}
