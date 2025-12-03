use actix_web::{web, HttpResponse, Result};
use maud::{html, DOCTYPE};
use sqlx::PgPool;

use crate::utils::PaginationParams;

pub async fn list_repositories(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let params = query.into_inner();
    let limit = params.limit();
    let offset = params.offset();

    let repositories = crate::models::Repository::list_all(pool.get_ref(), limit, offset)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let total = crate::models::Repository::count(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let markup = html! {
        (DOCTYPE)
        html lang="en" data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Repositories - Cross Bow" }
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
                    h1 class="text-4xl font-bold mb-8" { "Repositories" }
                    p class="mb-4" { "Total: " (total) " repositories" }

                    @if repositories.is_empty() {
                        div class="alert alert-info" {
                            span { "No repositories found. Webhook events will automatically create repository records." }
                        }
                    } @else {
                        div class="overflow-x-auto" {
                            table class="table table-zebra w-full" {
                                thead {
                                    tr {
                                        th { "Name" }
                                        th { "Owner" }
                                        th { "Description" }
                                        th { "Private" }
                                        th { "Actions" }
                                    }
                                }
                                tbody {
                                    @for repo in repositories {
                                        tr {
                                            td {
                                                a class="link link-primary" href=(format!("/repositories/{}", repo.id)) {
                                                    (repo.full_name)
                                                }
                                            }
                                            td { (repo.owner) }
                                            td {
                                                @if let Some(desc) = &repo.description {
                                                    (desc)
                                                } @else {
                                                    span class="text-gray-500" { "No description" }
                                                }
                                            }
                                            td {
                                                @if repo.is_private {
                                                    span class="badge badge-warning" { "Private" }
                                                } @else {
                                                    span class="badge badge-success" { "Public" }
                                                }
                                            }
                                            td {
                                                a class="btn btn-sm btn-primary" href=(repo.url) target="_blank" {
                                                    "View on GitHub"
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
    };

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(markup.into_string()))
}

pub async fn repository_detail(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let repo_id = path.into_inner();

    let repository = crate::models::Repository::find_by_id(pool.get_ref(), repo_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Repository not found"))?;

    let commits = crate::models::Commit::list_by_repository(pool.get_ref(), repo_id, 10, 0)
        .await
        .unwrap_or_default();

    let prs = crate::models::PullRequest::list_by_repository(pool.get_ref(), repo_id, 10, 0)
        .await
        .unwrap_or_default();

    let issues = crate::models::Issue::list_by_repository(pool.get_ref(), repo_id, 10, 0)
        .await
        .unwrap_or_default();

    let commit_count = crate::models::Commit::count_by_repository(pool.get_ref(), repo_id)
        .await
        .unwrap_or(0);

    let markup = html! {
        (DOCTYPE)
        html lang="en" data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (repository.full_name) " - Cross Bow" }
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
                    div class="breadcrumbs text-sm mb-4" {
                        ul {
                            li { a href="/repositories" { "Repositories" } }
                            li { (repository.full_name) }
                        }
                    }

                    div class="card bg-base-100 shadow-xl mb-8" {
                        div class="card-body" {
                            h1 class="card-title text-3xl" { (repository.full_name) }
                            @if let Some(desc) = &repository.description {
                                p class="text-gray-600" { (desc) }
                            }
                            div class="flex gap-2 mt-4" {
                                @if repository.is_private {
                                    span class="badge badge-warning" { "Private" }
                                } @else {
                                    span class="badge badge-success" { "Public" }
                                }
                                span class="badge badge-outline" { "Owner: " (repository.owner) }
                            }
                            div class="card-actions justify-end mt-4" {
                                a class="btn btn-primary" href=(repository.url) target="_blank" {
                                    "View on GitHub"
                                }
                            }
                        }
                    }

                    div class="stats shadow mb-8 w-full" {
                        div class="stat" {
                            div class="stat-title" { "Commits" }
                            div class="stat-value" { (commit_count) }
                        }
                        div class="stat" {
                            div class="stat-title" { "Pull Requests" }
                            div class="stat-value" { (prs.len()) }
                        }
                        div class="stat" {
                            div class="stat-title" { "Issues" }
                            div class="stat-value" { (issues.len()) }
                        }
                    }

                    h2 class="text-2xl font-bold mb-4" { "Recent Commits" }
                    @if commits.is_empty() {
                        div class="alert alert-info mb-8" {
                            span { "No commits tracked yet." }
                        }
                    } @else {
                        div class="space-y-4 mb-8" {
                            @for commit in commits {
                                div class="card bg-base-200 shadow" {
                                    div class="card-body" {
                                        div class="flex justify-between items-start" {
                                            div {
                                                p class="font-mono text-sm text-primary" {
                                                    (commit.sha[..7].to_string())
                                                }
                                                p class="mt-2" { (commit.message) }
                                                p class="text-sm text-gray-500 mt-1" {
                                                    "by " (commit.author_name) " at " (commit.committed_at.format("%Y-%m-%d %H:%M"))
                                                }
                                            }
                                            a class="btn btn-sm btn-ghost" href=(commit.url) target="_blank" {
                                                "View"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    h2 class="text-2xl font-bold mb-4" { "Recent Pull Requests" }
                    @if prs.is_empty() {
                        div class="alert alert-info mb-8" {
                            span { "No pull requests tracked yet." }
                        }
                    } @else {
                        div class="space-y-4 mb-8" {
                            @for pr in prs {
                                div class="card bg-base-200 shadow" {
                                    div class="card-body" {
                                        div class="flex justify-between items-start" {
                                            div {
                                                p class="font-bold" { "#" (pr.number) " " (pr.title) }
                                                p class="text-sm text-gray-500 mt-1" {
                                                    "by " (pr.author) " - " (pr.head_branch) " → " (pr.base_branch)
                                                }
                                                div class="mt-2" {
                                                    @if pr.state == "open" {
                                                        span class="badge badge-success" { "Open" }
                                                    } @else if pr.merged_at.is_some() {
                                                        span class="badge badge-primary" { "Merged" }
                                                    } @else {
                                                        span class="badge badge-error" { "Closed" }
                                                    }
                                                }
                                            }
                                            a class="btn btn-sm btn-ghost" href=(pr.url) target="_blank" {
                                                "View"
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
    };

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(markup.into_string()))
}
