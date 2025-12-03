use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use maud::{html, PreEscaped, DOCTYPE};
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::Event;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct EventFilters {
    #[serde(deserialize_with = "deserialize_optional_i64")]
    pub page: Option<i64>,
    pub source: Option<String>,
    pub event_type: Option<String>,
    pub action: Option<String>,
    pub actor_name: Option<String>,
    pub processed: Option<bool>,
    pub search: Option<String>,
}

fn deserialize_optional_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => s.parse::<i64>().map(Some).map_err(serde::de::Error::custom),
    }
}

pub async fn list_events(
    pool: web::Data<PgPool>,
    query: web::Query<EventFilters>,
) -> Result<HttpResponse> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = 300;
    let offset = (page - 1) * per_page;

    // Get filtered events
    let events = Event::search_and_filter(
        pool.get_ref(),
        query.source.as_deref(),
        query.event_type.as_deref(),
        query.action.as_deref(),
        query.actor_name.as_deref(),
        query.processed,
        query.search.as_deref(),
        per_page,
        offset,
    )
    .await
    .unwrap_or_default();

    let total_count = Event::count_filtered(
        pool.get_ref(),
        query.source.as_deref(),
        query.event_type.as_deref(),
        query.action.as_deref(),
        query.actor_name.as_deref(),
        query.processed,
        query.search.as_deref(),
    )
    .await
    .unwrap_or(0);

    // Get unique event types, sources, actions, and actor names for filter dropdowns
    let event_types = Event::get_event_types(pool.get_ref())
        .await
        .unwrap_or_default();
    let sources = Event::get_sources(pool.get_ref()).await.unwrap_or_default();
    let actions = Event::get_actions(pool.get_ref()).await.unwrap_or_default();
    let actor_names = Event::get_actor_names(pool.get_ref())
        .await
        .unwrap_or_default();

    let total_pages = (total_count as f64 / per_page as f64).ceil() as i64;

    let markup = html! {
        (DOCTYPE)
        html lang="en" data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Events - Cross Bow" }
                link rel="stylesheet" href="/assets/daisy.css";
                link rel="stylesheet" href="/assets/themes.css";
                script src="/assets/htmx.js" {}
                script src="/assets/tw.js" {}
                script src="/assets/theme-switcher.js" {}
            }
            body {
                (render_navbar())

                div class="container mx-auto px-4 py-8" {
                    h1 class="text-4xl font-bold mb-8" { "Webhook Events" }

                    // Filters section
                    div class="card bg-base-100 shadow-xl mb-6" {
                        div class="card-body" {
                            h2 class="card-title mb-4" { "Filters" }
                            form
                                id="filter-form"
                                method="get"
                                action="/events"
                                hx-get="/events"
                                hx-target="body"
                                hx-push-url="true"
                                class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-6 gap-4"
                            {
                                // Search input
                                div class="form-control" {
                                    label class="label" {
                                        span class="label-text" { "Search" }
                                    }
                                    input
                                        type="text"
                                        name="search"
                                        placeholder="Search in payload..."
                                        class="input input-bordered"
                                        value=(query.search.as_deref().unwrap_or(""))
                                        hx-get="/events"
                                        hx-target="body"
                                        hx-push-url="true"
                                        hx-trigger="input changed delay:500ms"
                                        hx-include="[name='source'], [name='event_type'], [name='action'], [name='actor_name'], [name='processed']";
                                }

                                // Source filter
                                div class="form-control" {
                                    label class="label" {
                                        span class="label-text" { "Source" }
                                    }
                                    select
                                        name="source"
                                        class="select select-bordered"
                                        hx-get="/events"
                                        hx-target="body"
                                        hx-push-url="true"
                                        hx-trigger="change"
                                        hx-include="[name='search'], [name='event_type'], [name='action'], [name='actor_name'], [name='processed']"
                                    {
                                        option value="" selected[query.source.is_none()] { "All Sources" }
                                        @for source in &sources {
                                            option
                                                value=(source)
                                                selected[query.source.as_deref() == Some(source.as_str())]
                                            { (source) }
                                        }
                                    }
                                }

                                // Event type filter
                                div class="form-control" {
                                    label class="label" {
                                        span class="label-text" { "Event Type" }
                                    }
                                    select
                                        name="event_type"
                                        class="select select-bordered"
                                        hx-get="/events"
                                        hx-target="body"
                                        hx-push-url="true"
                                        hx-trigger="change"
                                        hx-include="[name='search'], [name='source'], [name='action'], [name='actor_name'], [name='processed']"
                                    {
                                        option value="" selected[query.event_type.is_none()] { "All Types" }
                                        @for event_type in &event_types {
                                            option
                                                value=(event_type)
                                                selected[query.event_type.as_deref() == Some(event_type.as_str())]
                                            { (event_type) }
                                        }
                                    }
                                }

                                // Action filter
                                div class="form-control" {
                                    label class="label" {
                                        span class="label-text" { "Action" }
                                    }
                                    select
                                        name="action"
                                        class="select select-bordered"
                                        hx-get="/events"
                                        hx-target="body"
                                        hx-push-url="true"
                                        hx-trigger="change"
                                        hx-include="[name='search'], [name='source'], [name='event_type'], [name='actor_name'], [name='processed']"
                                    {
                                        option value="" selected[query.action.is_none()] { "All Actions" }
                                        @for action in &actions {
                                            option
                                                value=(action)
                                                selected[query.action.as_deref() == Some(action.as_str())]
                                            { (action) }
                                        }
                                    }
                                }

                                // Actor name filter
                                div class="form-control" {
                                    label class="label" {
                                        span class="label-text" { "Actor" }
                                    }
                                    select
                                        name="actor_name"
                                        class="select select-bordered"
                                        hx-get="/events"
                                        hx-target="body"
                                        hx-push-url="true"
                                        hx-trigger="change"
                                        hx-include="[name='search'], [name='source'], [name='event_type'], [name='action'], [name='processed']"
                                    {
                                        option value="" selected[query.actor_name.is_none()] { "All Actors" }
                                        @for actor_name in &actor_names {
                                            option
                                                value=(actor_name)
                                                selected[query.actor_name.as_deref() == Some(actor_name.as_str())]
                                            { (actor_name) }
                                        }
                                    }
                                }

                                // Processed status filter
                                div class="form-control" {
                                    label class="label" {
                                        span class="label-text" { "Status" }
                                    }
                                    select
                                        name="processed"
                                        class="select select-bordered"
                                        hx-get="/events"
                                        hx-target="body"
                                        hx-push-url="true"
                                        hx-trigger="change"
                                        hx-include="[name='search'], [name='source'], [name='event_type'], [name='action'], [name='actor_name']"
                                    {
                                        option value="" selected[query.processed.is_none()] { "All Status" }
                                        option value="true" selected[query.processed == Some(true)] { "Processed" }
                                        option value="false" selected[query.processed == Some(false)] { "Pending" }
                                    }
                                }

                                // Clear filters button
                                div class="form-control flex items-end" {
                                    a href="/events" class="btn btn-ghost" { "Clear Filters" }
                                }
                            }
                        }
                    }

                    // Results summary
                    div class="alert alert-info mb-6" {
                        span { "Showing " (events.len()) " of " (total_count) " events" }
                    }

                    // Events table
                    div class="card bg-base-100 shadow-xl mb-6" {
                        div class="card-body p-0" {
                            div class="overflow-x-auto max-h-[600px] overflow-y-auto" {
                                table class="table table-zebra" {
                                    thead {
                                        tr {
                                            th { "ID" }
                                            th { "Source" }
                                            th { "Event Type" }
                                            th { "Action" }
                                            th { "Actor" }
                                            th { "Received" }
                                            th { "Status" }
                                            th { "Actions" }
                                        }
                                    }
                                    tbody {
                                        @if events.is_empty() {
                                            tr {
                                                td colspan="8" class="text-center text-base-content/60 py-8" {
                                                    "No events found matching the filters"
                                                }
                                            }
                                        } @else {
                                            @for event in &events {
                                                tr {
                                                    td { (event.id) }
                                                    td {
                                                        span class="badge badge-secondary" { (event.source) }
                                                    }
                                                    td {
                                                        span class="badge badge-primary" { (event.event_type) }
                                                    }
                                                    td {
                                                        @if let Some(action) = &event.action {
                                                            span class="badge badge-ghost" { (action) }
                                                        } @else {
                                                            span class="text-base-content/60" { "-" }
                                                        }
                                                    }
                                                    td {
                                                        @if let Some(actor_name) = &event.actor_name {
                                                            div class="text-sm" {
                                                                div { (actor_name) }
                                                                @if let Some(actor_email) = &event.actor_email {
                                                                    div class="text-xs text-base-content/60" { (actor_email) }
                                                                }
                                                            }
                                                        } @else {
                                                            span class="text-base-content/60" { "-" }
                                                        }
                                                    }
                                                    td class="text-sm" {
                                                        (format_datetime(&event.received_at))
                                                    }
                                                    td {
                                                        @if event.processed {
                                                            span class="badge badge-success" { "Processed" }
                                                        } @else {
                                                            span class="badge badge-warning" { "Pending" }
                                                        }
                                                    }
                                                    td {
                                                        button
                                                            class="btn btn-xs btn-ghost"
                                                            onclick=(format!("document.getElementById('event-modal-{}').showModal()", event.id))
                                                        {
                                                            "View"
                                                        }
                                                    }
                                                }

                                                // Modal for event details
                                                dialog id=(format!("event-modal-{}", event.id)) class="modal" {
                                                    div class="modal-box max-w-4xl" {
                                                        h3 class="font-bold text-lg mb-4" {
                                                            "Event #" (event.id) " - " (event.source) " - " (event.event_type)
                                                        }
                                                        div class="space-y-4" {
                                                            div {
                                                                h4 class="font-semibold" { "Details" }
                                                                div class="grid grid-cols-2 gap-2 text-sm mt-2" {
                                                                    div { span class="font-medium" { "Source: " } (event.source) }
                                                                    div { span class="font-medium" { "Delivery ID: " } (event.delivery_id) }
                                                                    div { span class="font-medium" { "Received: " } (format_datetime(&event.received_at)) }
                                                                    div { span class="font-medium" { "Event Type: " } (event.event_type) }
                                                                    @if let Some(action) = &event.action {
                                                                        div { span class="font-medium" { "Action: " } (action) }
                                                                    }
                                                                    @if let Some(actor_name) = &event.actor_name {
                                                                        div { span class="font-medium" { "Actor: " } (actor_name) }
                                                                    }
                                                                    @if let Some(actor_email) = &event.actor_email {
                                                                        div { span class="font-medium" { "Actor Email: " } (actor_email) }
                                                                    }
                                                                    @if let Some(actor_id) = &event.actor_id {
                                                                        div { span class="font-medium" { "Actor ID: " } (actor_id) }
                                                                    }
                                                                    div { span class="font-medium" { "Status: " }
                                                                        @if event.processed {
                                                                            span class="badge badge-success" { "Processed" }
                                                                        } @else {
                                                                            span class="badge badge-warning" { "Pending" }
                                                                        }
                                                                    }
                                                                    @if let Some(processed_at) = event.processed_at {
                                                                        div { span class="font-medium" { "Processed At: " } (format_datetime(&processed_at)) }
                                                                    }
                                                                }
                                                            }
                                                            div {
                                                                h4 class="font-semibold mb-2" { "Raw Event Payload" }
                                                                pre class="bg-base-200 p-4 rounded-lg overflow-x-auto text-xs" {
                                                                    code {
                                                                        (PreEscaped(serde_json::to_string_pretty(&event.raw_event).unwrap_or_else(|_| "{}".to_string())))
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        div class="modal-action" {
                                                            form method="dialog" {
                                                                button class="btn" { "Close" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Pagination
                    @if total_pages > 1 {
                        div class="flex justify-center" {
                            div class="join" {
                                @for p in 1..=total_pages {
                                    a
                                        href=(build_page_url(p, &query))
                                        class=(format!("join-item btn {}", if p == page { "btn-active" } else { "" }))
                                    {
                                        (p)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(markup.into_string()))
}

fn render_navbar() -> maud::Markup {
    html! {
        div class="navbar bg-base-100 shadow-lg" {
            div class="flex-1" {
                a class="btn btn-ghost text-xl" href="/" { "Cross Bow" }
            }
            div class="flex-none gap-2" {
                ul class="menu menu-horizontal px-1" {
                    li { a href="/" { "Dashboard" } }
                    li { a href="/events" class="active" { "Events" } }
                }
                button
                    class="btn btn-ghost btn-circle"
                    onclick="toggleTheme()"
                    title="Toggle theme"
                {
                    // Sun icon for light mode
                    svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-5 w-5"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    {
                        path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z";
                    }
                }
            }
        }
    }
}

fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn build_page_url(page: i64, query: &web::Query<EventFilters>) -> String {
    let mut params = vec![format!("page={}", page)];

    if let Some(source) = &query.source {
        params.push(format!("source={source}"));
    }
    if let Some(event_type) = &query.event_type {
        params.push(format!("event_type={event_type}"));
    }
    if let Some(action) = &query.action {
        params.push(format!("action={action}"));
    }
    if let Some(actor_name) = &query.actor_name {
        params.push(format!("actor_name={actor_name}"));
    }
    if let Some(processed) = query.processed {
        params.push(format!("processed={processed}"));
    }
    if let Some(search) = &query.search {
        params.push(format!("search={search}"));
    }

    format!("/events?{}", params.join("&"))
}
