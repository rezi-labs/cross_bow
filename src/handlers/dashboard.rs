use actix_web::{web, HttpResponse, Result};
use maud::{html, DOCTYPE};
use sqlx::PgPool;

pub async fn dashboard(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let repo_count = crate::models::Repository::count(pool.get_ref())
        .await
        .unwrap_or(0);
    let event_count = crate::models::WebhookEvent::count(pool.get_ref())
        .await
        .unwrap_or(0);
    let commit_count = crate::models::Commit::count(pool.get_ref())
        .await
        .unwrap_or(0);
    let pr_count = crate::models::PullRequest::count(pool.get_ref())
        .await
        .unwrap_or(0);
    let issue_count = crate::models::Issue::count(pool.get_ref())
        .await
        .unwrap_or(0);

    let open_pr_count = crate::models::PullRequest::count_by_state(pool.get_ref(), "open")
        .await
        .unwrap_or(0);
    let open_issue_count = crate::models::Issue::count_by_state(pool.get_ref(), "open")
        .await
        .unwrap_or(0);

    let markup = html! {
        (DOCTYPE)
        html lang="en" data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Cross Bow - GitHub Observer" }
                link rel="stylesheet" href="/assets/daisy.css";
                link rel="stylesheet" href="/assets/themes.css";
                script src="/assets/htmx.js" {}
                script src="/assets/tw.js" {}
                script src="/assets/theme-switcher.js" {}
            }
            body {
                div class="navbar bg-base-100 shadow-lg" {
                    div class="flex-1" {
                        a class="btn btn-ghost text-xl" href="/" { "Cross Bow" }
                    }
                    div class="flex-none" {
                        ul class="menu menu-horizontal px-1" {
                            li { a href="/" { "Dashboard" } }
                            li { a href="/repositories" { "Repositories" } }
                        }
                    }
                }

                div class="container mx-auto px-4 py-8" {
                    h1 class="text-4xl font-bold mb-8" { "Dashboard" }

                    div class="stats stats-vertical lg:stats-horizontal shadow w-full mb-8" {
                        div class="stat" {
                            div class="stat-title" { "Repositories" }
                            div class="stat-value text-primary" { (repo_count) }
                        }
                        div class="stat" {
                            div class="stat-title" { "Total Events" }
                            div class="stat-value" { (event_count) }
                        }
                        div class="stat" {
                            div class="stat-title" { "Commits" }
                            div class="stat-value text-accent" { (commit_count) }
                        }
                    }

                    div class="stats stats-vertical lg:stats-horizontal shadow w-full mb-8" {
                        div class="stat" {
                            div class="stat-title" { "Pull Requests" }
                            div class="stat-value" { (pr_count) }
                            div class="stat-desc" { "Open: " (open_pr_count) }
                        }
                        div class="stat" {
                            div class="stat-title" { "Issues" }
                            div class="stat-value" { (issue_count) }
                            div class="stat-desc" { "Open: " (open_issue_count) }
                        }
                    }

                    div class="card bg-base-100 shadow-xl" {
                        div class="card-body" {
                            h2 class="card-title" { "Welcome to Cross Bow" }
                            p { "GitHub webhook monitoring and event tracking system." }
                            div class="card-actions justify-end" {
                                a class="btn btn-primary" href="/repositories" { "View Repositories" }
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
