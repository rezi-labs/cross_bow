use crate::models::{
    Commit, CreateCommit, CreateIssue, CreatePullRequest, CreateRepository, Issue, PullRequest,
    Repository, WebhookEvent,
};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;

pub async fn process_webhook_event(
    pool: &PgPool,
    event: &WebhookEvent,
) -> Result<(), ProcessingError> {
    let event_type = event.event_type.as_str();
    let payload = &event.payload;

    match event_type {
        "push" => process_push_event(pool, event, payload).await?,
        "pull_request" => process_pull_request_event(pool, event, payload).await?,
        "issues" => process_issues_event(pool, event, payload).await?,
        _ => {
            log::debug!("Unhandled event type: {event_type}");
        }
    }

    WebhookEvent::mark_processed(pool, event.id).await?;

    Ok(())
}

async fn process_push_event(
    pool: &PgPool,
    event: &WebhookEvent,
    payload: &JsonValue,
) -> Result<(), ProcessingError> {
    let repo_data = extract_repository(payload)?;
    let repository = Repository::create(pool, repo_data).await?;

    let commits = payload["commits"].as_array().ok_or_else(|| {
        ProcessingError::InvalidPayload("Missing commits array in push event".to_string())
    })?;

    for commit_data in commits {
        let sha = commit_data["id"]
            .as_str()
            .ok_or_else(|| ProcessingError::InvalidPayload("Missing commit sha".to_string()))?
            .to_string();

        let message = commit_data["message"]
            .as_str()
            .ok_or_else(|| ProcessingError::InvalidPayload("Missing commit message".to_string()))?
            .to_string();

        let author_name = commit_data["author"]["name"]
            .as_str()
            .ok_or_else(|| ProcessingError::InvalidPayload("Missing author name".to_string()))?
            .to_string();

        let author_email = commit_data["author"]["email"]
            .as_str()
            .ok_or_else(|| ProcessingError::InvalidPayload("Missing author email".to_string()))?
            .to_string();

        let committer_name = commit_data["committer"]["name"]
            .as_str()
            .ok_or_else(|| ProcessingError::InvalidPayload("Missing committer name".to_string()))?
            .to_string();

        let committer_email = commit_data["committer"]["email"]
            .as_str()
            .ok_or_else(|| ProcessingError::InvalidPayload("Missing committer email".to_string()))?
            .to_string();

        let timestamp_str = commit_data["timestamp"].as_str().ok_or_else(|| {
            ProcessingError::InvalidPayload("Missing commit timestamp".to_string())
        })?;

        let committed_at: DateTime<Utc> = timestamp_str
            .parse()
            .map_err(|_| ProcessingError::InvalidPayload("Invalid timestamp format".to_string()))?;

        let url = commit_data["url"]
            .as_str()
            .ok_or_else(|| ProcessingError::InvalidPayload("Missing commit url".to_string()))?
            .to_string();

        let commit = CreateCommit {
            repository_id: repository.id,
            webhook_event_id: event.id,
            sha,
            message,
            author_name,
            author_email,
            committer_name,
            committer_email,
            committed_at,
            url,
        };

        Commit::create(pool, commit).await?;
    }

    Ok(())
}

async fn process_pull_request_event(
    pool: &PgPool,
    event: &WebhookEvent,
    payload: &JsonValue,
) -> Result<(), ProcessingError> {
    let repo_data = extract_repository(payload)?;
    let repository = Repository::create(pool, repo_data).await?;

    let pr_data = &payload["pull_request"];

    let github_id = pr_data["id"]
        .as_i64()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing PR id".to_string()))?;

    let number = pr_data["number"]
        .as_i64()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing PR number".to_string()))?
        as i32;

    let title = pr_data["title"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing PR title".to_string()))?
        .to_string();

    let state = pr_data["state"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing PR state".to_string()))?
        .to_string();

    let author = pr_data["user"]["login"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing PR author".to_string()))?
        .to_string();

    let base_branch = pr_data["base"]["ref"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing base branch".to_string()))?
        .to_string();

    let head_branch = pr_data["head"]["ref"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing head branch".to_string()))?
        .to_string();

    let url = pr_data["html_url"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing PR url".to_string()))?
        .to_string();

    let opened_at_str = pr_data["created_at"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing PR created_at".to_string()))?;

    let opened_at: DateTime<Utc> = opened_at_str
        .parse()
        .map_err(|_| ProcessingError::InvalidPayload("Invalid timestamp format".to_string()))?;

    let closed_at = pr_data["closed_at"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok());

    let merged_at = pr_data["merged_at"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok());

    let pr = CreatePullRequest {
        repository_id: repository.id,
        webhook_event_id: event.id,
        github_id,
        number,
        title,
        state,
        author,
        base_branch,
        head_branch,
        url,
        opened_at,
        closed_at,
        merged_at,
    };

    PullRequest::create(pool, pr).await?;

    Ok(())
}

async fn process_issues_event(
    pool: &PgPool,
    event: &WebhookEvent,
    payload: &JsonValue,
) -> Result<(), ProcessingError> {
    let repo_data = extract_repository(payload)?;
    let repository = Repository::create(pool, repo_data).await?;

    let issue_data = &payload["issue"];

    let github_id = issue_data["id"]
        .as_i64()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing issue id".to_string()))?;

    let number = issue_data["number"]
        .as_i64()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing issue number".to_string()))?
        as i32;

    let title = issue_data["title"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing issue title".to_string()))?
        .to_string();

    let state = issue_data["state"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing issue state".to_string()))?
        .to_string();

    let author = issue_data["user"]["login"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing issue author".to_string()))?
        .to_string();

    let labels: Vec<String> = issue_data["labels"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let url = issue_data["html_url"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing issue url".to_string()))?
        .to_string();

    let opened_at_str = issue_data["created_at"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing issue created_at".to_string()))?;

    let opened_at: DateTime<Utc> = opened_at_str
        .parse()
        .map_err(|_| ProcessingError::InvalidPayload("Invalid timestamp format".to_string()))?;

    let closed_at = issue_data["closed_at"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok());

    let issue = CreateIssue {
        repository_id: repository.id,
        webhook_event_id: event.id,
        github_id,
        number,
        title,
        state,
        author,
        labels,
        url,
        opened_at,
        closed_at,
    };

    Issue::create(pool, issue).await?;

    Ok(())
}

fn extract_repository(payload: &JsonValue) -> Result<CreateRepository, ProcessingError> {
    let repo = &payload["repository"];

    let github_id = repo["id"]
        .as_i64()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing repository id".to_string()))?;

    let name = repo["name"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing repository name".to_string()))?
        .to_string();

    let full_name = repo["full_name"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing repository full_name".to_string()))?
        .to_string();

    let owner = repo["owner"]["login"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing repository owner".to_string()))?
        .to_string();

    let description = repo["description"].as_str().map(|s| s.to_string());

    let url = repo["html_url"]
        .as_str()
        .ok_or_else(|| ProcessingError::InvalidPayload("Missing repository url".to_string()))?
        .to_string();

    let is_private = repo["private"].as_bool().unwrap_or(false);

    Ok(CreateRepository {
        github_id,
        name,
        full_name,
        owner,
        description,
        url,
        is_private,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
