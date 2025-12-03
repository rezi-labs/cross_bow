use crate::config::Config;
use crate::models::{CreateEvent, CreateWebhookEvent, Event, WebhookEvent};
use crate::services::{convert_github_webhook_to_event, process_github_event};
use crate::utils::verify_github_signature;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

/// Generic webhook handler that accepts webhooks from any source
pub async fn generic_webhook(
    req: HttpRequest,
    body: web::Bytes,
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let source = path.into_inner();

    log::info!("Received webhook from source: {source}");

    // Generate a delivery ID if not provided
    let delivery_id = extract_delivery_id(&req, &source).unwrap_or_else(Uuid::new_v4);

    // Parse payload
    let payload: JsonValue = serde_json::from_slice(&body).map_err(|e| {
        log::error!("Failed to parse webhook payload from {source}: {e}");
        actix_web::error::ErrorBadRequest("Invalid JSON payload")
    })?;

    // Extract basic event information
    let event_type = extract_event_type(&source, &payload, &req);
    let action = extract_action(&source, &payload);
    let signature = extract_signature(&source, &req);

    // For GitHub, verify signature if present
    if source == "github" {
        if let Some(sig) = &signature {
            if !verify_github_signature(&config.github_webhook_secret, &body, sig) {
                log::warn!("Invalid GitHub webhook signature for delivery {delivery_id}");
                return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid signature"
                })));
            }
        } else {
            log::warn!("Missing GitHub signature for delivery {delivery_id}");
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Missing signature"
            })));
        }
    }

    // Extract actor information (source-specific)
    let (actor_name, actor_email, actor_id) = extract_actor_info(&source, &payload);

    // Create generic event
    let create_event = CreateEvent {
        source: source.clone(),
        event_type: event_type.clone(),
        action: action.clone(),
        actor_name,
        actor_email,
        actor_id,
        raw_event: payload.clone(),
        delivery_id,
        signature: signature.clone(),
        repository_id: None, // Will be set by source-specific processors
    };

    let event = Event::create(pool.get_ref(), create_event)
        .await
        .map_err(|e| {
            log::error!("Failed to store generic event from {source}: {e}");
            actix_web::error::ErrorInternalServerError("Failed to store event")
        })?;

    log::info!(
        "Stored event #{} from source: {} (type: {}, delivery: {})",
        event.id,
        source,
        event_type,
        delivery_id
    );

    // Process event asynchronously based on source
    let pool_clone = pool.get_ref().clone();
    let event_clone = event.clone();
    let source_clone = source.clone();

    tokio::spawn(async move {
        if let Err(e) = process_event_by_source(&pool_clone, &event_clone, &source_clone).await {
            log::error!(
                "Failed to process {} event {}: {}",
                source_clone,
                event_clone.id,
                e
            );
        } else {
            log::info!(
                "Successfully processed {} event {}",
                source_clone,
                event_clone.id
            );
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "received",
        "source": source,
        "event_id": event.id,
        "event_type": event_type
    })))
}

/// Backward compatibility: GitHub-specific webhook endpoint
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

    // Store legacy webhook event for backward compatibility
    let webhook_event = CreateWebhookEvent {
        repository_id,
        event_type: event_type.clone(),
        event_action: event_action.clone(),
        delivery_id,
        payload: payload.clone(),
        signature: signature.to_string(),
    };

    let _legacy_event = WebhookEvent::create(pool.get_ref(), webhook_event)
        .await
        .map_err(|e| {
            log::error!("Failed to store legacy webhook event: {e}");
            actix_web::error::ErrorInternalServerError("Failed to store event")
        })?;

    // Convert to generic event
    let create_event = convert_github_webhook_to_event(
        event_type.clone(),
        event_action,
        payload,
        delivery_id,
        Some(signature.to_string()),
        repository_id,
    );

    let event = Event::create(pool.get_ref(), create_event)
        .await
        .map_err(|e| {
            log::error!("Failed to store generic event: {e}");
            actix_web::error::ErrorInternalServerError("Failed to store event")
        })?;

    log::info!("Received GitHub webhook event: {event_type} (delivery: {delivery_id})");

    // Process event asynchronously
    let pool_clone = pool.get_ref().clone();
    let event_clone = event.clone();
    tokio::spawn(async move {
        if let Err(e) = process_github_event(&pool_clone, &event_clone).await {
            log::error!("Failed to process GitHub event {}: {}", event_clone.id, e);
        } else {
            log::info!("Successfully processed GitHub event {}", event_clone.id);
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "received",
        "event_id": event.id
    })))
}

/// Extract delivery ID from headers based on source
fn extract_delivery_id(req: &HttpRequest, source: &str) -> Option<Uuid> {
    match source {
        "github" => req
            .headers()
            .get("X-GitHub-Delivery")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok()),
        "gitlab" => req
            .headers()
            .get("X-Gitlab-Event-UUID")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok()),
        _ => None,
    }
}

/// Extract event type from payload or headers based on source
fn extract_event_type(source: &str, payload: &JsonValue, req: &HttpRequest) -> String {
    match source {
        "github" => req
            .headers()
            .get("X-GitHub-Event")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        "gitlab" => req
            .headers()
            .get("X-Gitlab-Event")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                payload["object_kind"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string()
            }),
        "auth0" => payload["type"]
            .as_str()
            .or_else(|| payload["event"].as_str())
            .unwrap_or("unknown")
            .to_string(),
        _ => payload["type"]
            .as_str()
            .or_else(|| payload["event"].as_str())
            .or_else(|| payload["event_type"].as_str())
            .unwrap_or("webhook")
            .to_string(),
    }
}

/// Extract action from payload
fn extract_action(_source: &str, payload: &JsonValue) -> Option<String> {
    payload["action"]
        .as_str()
        .or_else(|| payload["event_action"].as_str())
        .map(|s| s.to_string())
}

/// Extract signature from headers based on source
fn extract_signature(source: &str, req: &HttpRequest) -> Option<String> {
    match source {
        "github" => req
            .headers()
            .get("X-Hub-Signature-256")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        "gitlab" => req
            .headers()
            .get("X-Gitlab-Token")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        _ => None,
    }
}

/// Extract actor information based on source
fn extract_actor_info(
    source: &str,
    payload: &JsonValue,
) -> (Option<String>, Option<String>, Option<String>) {
    match source {
        "github" => {
            let name = payload["sender"]["login"]
                .as_str()
                .or_else(|| payload["pusher"]["name"].as_str())
                .map(|s| s.to_string());

            let email = payload["sender"]["email"]
                .as_str()
                .or_else(|| payload["pusher"]["email"].as_str())
                .map(|s| s.to_string());

            let id = payload["sender"]["login"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| payload["sender"]["id"].as_i64().map(|i| i.to_string()));

            (name, email, id)
        }
        "gitlab" => {
            let name = payload["user_username"]
                .as_str()
                .or_else(|| payload["user"]["username"].as_str())
                .map(|s| s.to_string());

            let email = payload["user_email"]
                .as_str()
                .or_else(|| payload["user"]["email"].as_str())
                .map(|s| s.to_string());

            let id = payload["user_id"]
                .as_i64()
                .map(|i| i.to_string())
                .or_else(|| payload["user"]["id"].as_i64().map(|i| i.to_string()));

            (name, email, id)
        }
        "auth0" => {
            let name = payload["user"]["name"]
                .as_str()
                .or_else(|| payload["user"]["username"].as_str())
                .map(|s| s.to_string());

            let email = payload["user"]["email"].as_str().map(|s| s.to_string());

            let id = payload["user"]["user_id"]
                .as_str()
                .or_else(|| payload["user"]["id"].as_str())
                .map(|s| s.to_string());

            (name, email, id)
        }
        _ => {
            // Generic extraction
            let name = payload["actor"]
                .as_str()
                .or_else(|| payload["user"].as_str())
                .or_else(|| payload["username"].as_str())
                .map(|s| s.to_string());

            let email = payload["email"].as_str().map(|s| s.to_string());

            let id = payload["actor_id"]
                .as_str()
                .or_else(|| payload["user_id"].as_str())
                .map(|s| s.to_string());

            (name, email, id)
        }
    }
}

/// Route event to source-specific processor
async fn process_event_by_source(
    pool: &PgPool,
    event: &Event,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match source {
        "github" => {
            process_github_event(pool, event).await?;
        }
        "gitlab" => {
            log::info!(
                "GitLab event processing not yet implemented for event {}",
                event.id
            );
            Event::mark_processed(pool, event.id).await?;
        }
        "auth0" => {
            log::info!(
                "Auth0 event processing not yet implemented for event {}",
                event.id
            );
            Event::mark_processed(pool, event.id).await?;
        }
        _ => {
            log::info!(
                "No specific processor for source '{}', marking event {} as processed",
                source,
                event.id
            );
            Event::mark_processed(pool, event.id).await?;
        }
    }

    Ok(())
}
