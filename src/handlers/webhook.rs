use crate::config::Config;
use crate::models::{CreateWebhookEvent, WebhookEvent};
use crate::services::process_webhook_event;
use crate::utils::verify_github_signature;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn github_webhook(
    req: HttpRequest,
    body: web::Bytes,
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    // Extract headers
    let event_type = req
        .headers()
        .get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing X-GitHub-Event header"))?
        .to_string();

    let delivery_id = req
        .headers()
        .get("X-GitHub-Delivery")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid X-GitHub-Delivery header"))?;

    let signature = req
        .headers()
        .get("X-Hub-Signature-256")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing X-Hub-Signature-256 header"))?;

    // Verify signature
    if !verify_github_signature(&config.github_webhook_secret, &body, signature) {
        log::warn!("Invalid webhook signature for delivery {delivery_id}");
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid signature"
        })));
    }

    // Parse payload
    let payload: JsonValue = serde_json::from_slice(&body).map_err(|e| {
        log::error!("Failed to parse webhook payload: {e}");
        actix_web::error::ErrorBadRequest("Invalid JSON payload")
    })?;

    let event_action = payload["action"].as_str().map(|s| s.to_string());

    // Extract repository ID if present
    let repository_id = if let Some(repo) = payload["repository"].as_object() {
        if let Some(_id) = repo["id"].as_i64() {
            // Try to find or create repository
            match crate::models::Repository::find_by_full_name(
                pool.get_ref(),
                repo["full_name"].as_str().unwrap_or(""),
            )
            .await
            {
                Ok(Some(r)) => Some(r.id),
                Ok(None) => None,
                Err(e) => {
                    log::error!("Database error finding repository: {e}");
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // Store webhook event
    let webhook_event = CreateWebhookEvent {
        repository_id,
        event_type: event_type.clone(),
        event_action,
        delivery_id,
        payload: payload.clone(),
        signature: signature.to_string(),
    };

    let event = WebhookEvent::create(pool.get_ref(), webhook_event)
        .await
        .map_err(|e| {
            log::error!("Failed to store webhook event: {e}");
            actix_web::error::ErrorInternalServerError("Failed to store event")
        })?;

    log::info!("Received webhook event: {event_type} (delivery: {delivery_id})");

    // Process event asynchronously
    let pool_clone = pool.get_ref().clone();
    let event_clone = event.clone();
    tokio::spawn(async move {
        if let Err(e) = process_webhook_event(&pool_clone, &event_clone).await {
            log::error!("Failed to process webhook event {}: {}", event_clone.id, e);
        } else {
            log::info!("Successfully processed webhook event {}", event_clone.id);
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "received",
        "event_id": event.id
    })))
}
