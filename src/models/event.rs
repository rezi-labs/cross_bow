use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i64,
    pub source: String,
    pub event_type: String,
    pub action: Option<String>,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
    pub actor_id: Option<String>,
    pub raw_event: JsonValue,
    pub delivery_id: Uuid,
    pub signature: Option<String>,
    pub received_at: DateTime<Utc>,
    pub processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
    pub repository_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvent {
    pub source: String,
    pub event_type: String,
    pub action: Option<String>,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
    pub actor_id: Option<String>,
    pub raw_event: JsonValue,
    pub delivery_id: Uuid,
    pub signature: Option<String>,
    pub repository_id: Option<i64>,
}

impl Event {
    pub async fn create(pool: &sqlx::PgPool, data: CreateEvent) -> Result<Self, sqlx::Error> {
        let event = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO events (source, event_type, action, actor_name, actor_email, actor_id, raw_event, delivery_id, signature, repository_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(data.source)
        .bind(data.event_type)
        .bind(data.action)
        .bind(data.actor_name)
        .bind(data.actor_email)
        .bind(data.actor_id)
        .bind(data.raw_event)
        .bind(data.delivery_id)
        .bind(data.signature)
        .bind(data.repository_id)
        .fetch_one(pool)
        .await?;

        Ok(event)
    }

    pub async fn mark_processed(pool: &sqlx::PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE events SET processed = true, processed_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn find_by_id(pool: &sqlx::PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(event)
    }

    #[allow(dead_code)]
    pub async fn list_by_repository(
        pool: &sqlx::PgPool,
        repository_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT * FROM events WHERE repository_id = $1 ORDER BY received_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repository_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    #[allow(dead_code)]
    pub async fn list_all(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT * FROM events ORDER BY received_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    #[allow(dead_code)]
    pub async fn list_by_type(
        pool: &sqlx::PgPool,
        event_type: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT * FROM events WHERE event_type = $1 ORDER BY received_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(event_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    #[allow(dead_code)]
    pub async fn list_by_source(
        pool: &sqlx::PgPool,
        source: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT * FROM events WHERE source = $1 ORDER BY received_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(source)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    pub async fn count(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn search_and_filter(
        pool: &sqlx::PgPool,
        source: Option<&str>,
        event_type: Option<&str>,
        action: Option<&str>,
        actor_name: Option<&str>,
        processed: Option<bool>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM events WHERE 1=1");
        let mut bindings = Vec::new();
        let mut param_count = 1;

        if let Some(src) = source {
            query.push_str(&format!(" AND source = ${param_count}"));
            bindings.push(src.to_string());
            param_count += 1;
        }

        if let Some(et) = event_type {
            query.push_str(&format!(" AND event_type = ${param_count}"));
            bindings.push(et.to_string());
            param_count += 1;
        }

        if let Some(act) = action {
            query.push_str(&format!(" AND action = ${param_count}"));
            bindings.push(act.to_string());
            param_count += 1;
        }

        if let Some(actor) = actor_name {
            query.push_str(&format!(" AND actor_name = ${param_count}"));
            bindings.push(actor.to_string());
            param_count += 1;
        }

        if let Some(proc) = processed {
            query.push_str(&format!(" AND processed = ${param_count}"));
            bindings.push(proc.to_string());
            param_count += 1;
        }

        if let Some(s) = search {
            if !s.is_empty() {
                query.push_str(&format!(" AND raw_event::text ILIKE ${param_count}"));
                bindings.push(format!("%{s}%"));
                param_count += 1;
            }
        }

        query.push_str(&format!(
            " ORDER BY received_at DESC LIMIT ${} OFFSET ${}",
            param_count,
            param_count + 1
        ));
        bindings.push(limit.to_string());
        bindings.push(offset.to_string());

        let mut query_builder = sqlx::query_as::<_, Event>(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let events = query_builder.fetch_all(pool).await?;

        Ok(events)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn count_filtered(
        pool: &sqlx::PgPool,
        source: Option<&str>,
        event_type: Option<&str>,
        action: Option<&str>,
        actor_name: Option<&str>,
        processed: Option<bool>,
        search: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM events WHERE 1=1");
        let mut bindings = Vec::new();
        let mut param_count = 1;

        if let Some(src) = source {
            query.push_str(&format!(" AND source = ${param_count}"));
            bindings.push(src.to_string());
            param_count += 1;
        }

        if let Some(et) = event_type {
            query.push_str(&format!(" AND event_type = ${param_count}"));
            bindings.push(et.to_string());
            param_count += 1;
        }

        if let Some(act) = action {
            query.push_str(&format!(" AND action = ${param_count}"));
            bindings.push(act.to_string());
            param_count += 1;
        }

        if let Some(actor) = actor_name {
            query.push_str(&format!(" AND actor_name = ${param_count}"));
            bindings.push(actor.to_string());
            param_count += 1;
        }

        if let Some(proc) = processed {
            query.push_str(&format!(" AND processed = ${param_count}"));
            bindings.push(proc.to_string());
            param_count += 1;
        }

        if let Some(s) = search {
            if !s.is_empty() {
                query.push_str(&format!(" AND raw_event::text ILIKE ${param_count}"));
                bindings.push(format!("%{s}%"));
            }
        }

        let mut query_builder = sqlx::query_as::<_, (i64,)>(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let count = query_builder.fetch_one(pool).await?;

        Ok(count.0)
    }

    pub async fn get_event_types(pool: &sqlx::PgPool) -> Result<Vec<String>, sqlx::Error> {
        let types: Vec<(String,)> =
            sqlx::query_as("SELECT DISTINCT event_type FROM events ORDER BY event_type")
                .fetch_all(pool)
                .await?;

        Ok(types.into_iter().map(|(t,)| t).collect())
    }

    pub async fn get_sources(pool: &sqlx::PgPool) -> Result<Vec<String>, sqlx::Error> {
        let sources: Vec<(String,)> =
            sqlx::query_as("SELECT DISTINCT source FROM events ORDER BY source")
                .fetch_all(pool)
                .await?;

        Ok(sources.into_iter().map(|(s,)| s).collect())
    }

    pub async fn get_actions(pool: &sqlx::PgPool) -> Result<Vec<String>, sqlx::Error> {
        let actions: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT action FROM events WHERE action IS NOT NULL ORDER BY action",
        )
        .fetch_all(pool)
        .await?;

        Ok(actions.into_iter().map(|(a,)| a).collect())
    }

    pub async fn get_actor_names(pool: &sqlx::PgPool) -> Result<Vec<String>, sqlx::Error> {
        let actor_names: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT actor_name FROM events WHERE actor_name IS NOT NULL ORDER BY actor_name",
        )
        .fetch_all(pool)
        .await?;

        Ok(actor_names.into_iter().map(|(a,)| a).collect())
    }
}
