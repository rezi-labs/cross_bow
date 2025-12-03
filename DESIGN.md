# GitHub Observer Server - Design Documentation

## Overview

A webhook-based GitHub event monitoring system built with Rust, Actix-web, HTMX, Maud, and PostgreSQL. The server receives GitHub webhook events, stores them in a database, and provides a web interface for viewing and analyzing repository activity.

## Architecture

### Technology Stack

- **Backend Framework**: Actix-web (Rust)
- **Frontend**: HTMX + Maud (server-side HTML templating)
- **Styling**: Tailwind CSS + DaisyUI
- **Database**: PostgreSQL
- **Additional Libraries**:
  - `sqlx` - Async PostgreSQL driver with compile-time query checking
  - `serde` - JSON serialization/deserialization
  - `tokio` - Async runtime
  - `chrono` - Date/time handling
  - `hmac` + `sha2` - Webhook signature verification

### System Components

```
┌─────────────┐
│   GitHub    │
│  Webhooks   │
└──────┬──────┘
       │
       │ HTTP POST
       ▼
┌─────────────────────────────────────────┐
│         Actix-web Server                │
│                                         │
│  ┌─────────────────────────────────┐  │
│  │   Webhook Handler                │  │
│  │  - Signature verification        │  │
│  │  - Event parsing                 │  │
│  │  - Data validation               │  │
│  └────────┬────────────────────────┘  │
│           │                             │
│           ▼                             │
│  ┌─────────────────────────────────┐  │
│  │   Business Logic Layer          │  │
│  │  - Event processing              │  │
│  │  - Data transformation           │  │
│  └────────┬────────────────────────┘  │
│           │                             │
│           ▼                             │
│  ┌─────────────────────────────────┐  │
│  │   Database Layer (sqlx)         │  │
│  │  - Connection pooling            │  │
│  │  - Query execution               │  │
│  └────────┬────────────────────────┘  │
└───────────┼─────────────────────────────┘
            │
            ▼
    ┌──────────────┐
    │  PostgreSQL  │
    │   Database   │
    └──────────────┘
            ▲
            │
┌───────────┼─────────────────────────────┐
│           │                             │
│  ┌────────┴────────────────────────┐  │
│  │   Web Interface Handler         │  │
│  │  - Route handling                │  │
│  │  - Template rendering (Maud)     │  │
│  └────────┬────────────────────────┘  │
│           │                             │
│           ▼                             │
│  ┌─────────────────────────────────┐  │
│  │   HTMX Frontend                  │  │
│  │  - Dynamic content updates       │  │
│  │  - Event filtering/search        │  │
│  │  - Real-time notifications       │  │
│  └─────────────────────────────────┘  │
│                                         │
│         Actix-web Server                │
└─────────────────────────────────────────┘
            │
            ▼
    ┌──────────────┐
    │   Browser    │
    └──────────────┘
```

## Database Schema

### Tables

#### `repositories`
Stores information about tracked GitHub repositories.

```sql
CREATE TABLE repositories (
    id BIGSERIAL PRIMARY KEY,
    github_id BIGINT NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL UNIQUE,
    owner VARCHAR(255) NOT NULL,
    description TEXT,
    url VARCHAR(500) NOT NULL,
    is_private BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_repositories_full_name ON repositories(full_name);
CREATE INDEX idx_repositories_owner ON repositories(owner);
```

#### `webhook_events`
Stores raw webhook events from GitHub.

```sql
CREATE TABLE webhook_events (
    id BIGSERIAL PRIMARY KEY,
    repository_id BIGINT REFERENCES repositories(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    event_action VARCHAR(100),
    delivery_id UUID NOT NULL UNIQUE,
    payload JSONB NOT NULL,
    signature VARCHAR(255) NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_webhook_events_repo ON webhook_events(repository_id);
CREATE INDEX idx_webhook_events_type ON webhook_events(event_type);
CREATE INDEX idx_webhook_events_received ON webhook_events(received_at DESC);
CREATE INDEX idx_webhook_events_delivery ON webhook_events(delivery_id);
CREATE INDEX idx_webhook_payload ON webhook_events USING gin(payload);
```

#### `commits`
Extracted commit information from push events.

```sql
CREATE TABLE commits (
    id BIGSERIAL PRIMARY KEY,
    repository_id BIGINT REFERENCES repositories(id) ON DELETE CASCADE,
    webhook_event_id BIGINT REFERENCES webhook_events(id) ON DELETE CASCADE,
    sha VARCHAR(40) NOT NULL,
    message TEXT NOT NULL,
    author_name VARCHAR(255) NOT NULL,
    author_email VARCHAR(255) NOT NULL,
    committer_name VARCHAR(255) NOT NULL,
    committer_email VARCHAR(255) NOT NULL,
    committed_at TIMESTAMPTZ NOT NULL,
    url VARCHAR(500) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_commits_sha_repo ON commits(sha, repository_id);
CREATE INDEX idx_commits_repo ON commits(repository_id);
CREATE INDEX idx_commits_author ON commits(author_email);
CREATE INDEX idx_commits_date ON commits(committed_at DESC);
```

#### `pull_requests`
Stores pull request information.

```sql
CREATE TABLE pull_requests (
    id BIGSERIAL PRIMARY KEY,
    repository_id BIGINT REFERENCES repositories(id) ON DELETE CASCADE,
    webhook_event_id BIGINT REFERENCES webhook_events(id) ON DELETE CASCADE,
    github_id BIGINT NOT NULL,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    state VARCHAR(50) NOT NULL,
    author VARCHAR(255) NOT NULL,
    base_branch VARCHAR(255) NOT NULL,
    head_branch VARCHAR(255) NOT NULL,
    url VARCHAR(500) NOT NULL,
    opened_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    merged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_pr_github_id ON pull_requests(github_id);
CREATE INDEX idx_pr_repo ON pull_requests(repository_id);
CREATE INDEX idx_pr_state ON pull_requests(state);
CREATE INDEX idx_pr_author ON pull_requests(author);
```

#### `issues`
Stores issue information.

```sql
CREATE TABLE issues (
    id BIGSERIAL PRIMARY KEY,
    repository_id BIGINT REFERENCES repositories(id) ON DELETE CASCADE,
    webhook_event_id BIGINT REFERENCES webhook_events(id) ON DELETE CASCADE,
    github_id BIGINT NOT NULL,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    state VARCHAR(50) NOT NULL,
    author VARCHAR(255) NOT NULL,
    labels TEXT[],
    url VARCHAR(500) NOT NULL,
    opened_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_issues_github_id ON issues(github_id);
CREATE INDEX idx_issues_repo ON issues(repository_id);
CREATE INDEX idx_issues_state ON issues(state);
CREATE INDEX idx_issues_author ON issues(author);
CREATE INDEX idx_issues_labels ON issues USING gin(labels);
```

## API Endpoints

### Webhook Endpoints

#### `POST /webhooks/github`
Receives GitHub webhook events.

**Headers:**
- `X-GitHub-Event`: Event type (push, pull_request, issues, etc.)
- `X-GitHub-Delivery`: Unique delivery ID
- `X-Hub-Signature-256`: HMAC SHA256 signature of the payload

**Request Body:** GitHub event payload (JSON)

**Response:**
- `200 OK`: Event received and processed
- `400 Bad Request`: Invalid payload or signature
- `401 Unauthorized`: Invalid signature
- `500 Internal Server Error`: Processing error

### Web Interface Endpoints

#### `GET /`
Dashboard homepage showing overview statistics.

**Response:** HTML page with:
- Total repositories tracked
- Recent events summary
- Activity timeline
- Quick stats (commits, PRs, issues)

#### `GET /repositories`
List all tracked repositories.

**Query Parameters:**
- `page` (optional): Page number for pagination
- `per_page` (optional): Items per page (default: 20)

**Response:** HTML page with repository list

#### `GET /repositories/{id}`
Repository detail page.

**Response:** HTML page with:
- Repository information
- Recent activity
- Commit history
- PR list
- Issue list

#### `GET /events`
List all webhook events.

**Query Parameters:**
- `event_type` (optional): Filter by event type
- `repository_id` (optional): Filter by repository
- `page` (optional): Page number
- `per_page` (optional): Items per page

**Response:** HTML page with event list (HTMX-enabled for infinite scroll)

#### `GET /events/{id}`
Event detail page showing full payload.

**Response:** HTML page with formatted JSON payload

#### `GET /commits`
List all commits across repositories.

**Query Parameters:**
- `repository_id` (optional): Filter by repository
- `author` (optional): Filter by author email
- `from` (optional): Start date (ISO 8601)
- `to` (optional): End date (ISO 8601)
- `page` (optional): Page number

**Response:** HTML page with commit list

#### `GET /pull-requests`
List all pull requests.

**Query Parameters:**
- `repository_id` (optional): Filter by repository
- `state` (optional): Filter by state (open, closed, merged)
- `author` (optional): Filter by author
- `page` (optional): Page number

**Response:** HTML page with PR list

#### `GET /issues`
List all issues.

**Query Parameters:**
- `repository_id` (optional): Filter by repository
- `state` (optional): Filter by state (open, closed)
- `author` (optional): Filter by author
- `label` (optional): Filter by label
- `page` (optional): Page number

**Response:** HTML page with issue list

### HTMX Partial Endpoints

These endpoints return HTML fragments for dynamic updates:

#### `GET /htmx/events/recent`
Returns latest events HTML fragment.

#### `GET /htmx/stats/refresh`
Returns updated statistics HTML fragment.

#### `GET /htmx/search`
Search across commits, PRs, and issues.

**Query Parameters:**
- `q`: Search query
- `type`: Search type (all, commits, prs, issues)

## Application Structure

### Directory Layout

```
cross_bow/
├── Cargo.toml
├── .env.example
├── DESIGN.md
├── README.md
├── migrations/
│   ├── 001_create_repositories.sql
│   ├── 002_create_webhook_events.sql
│   ├── 003_create_commits.sql
│   ├── 004_create_pull_requests.sql
│   └── 005_create_issues.sql
├── src/
│   ├── main.rs
│   ├── config.rs              # Configuration management
│   ├── models/
│   │   ├── mod.rs
│   │   ├── repository.rs
│   │   ├── webhook_event.rs
│   │   ├── commit.rs
│   │   ├── pull_request.rs
│   │   └── issue.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── webhook.rs         # Webhook handlers
│   │   ├── dashboard.rs       # Dashboard routes
│   │   ├── repository.rs      # Repository routes
│   │   ├── event.rs           # Event listing routes
│   │   ├── commit.rs          # Commit routes
│   │   ├── pull_request.rs    # PR routes
│   │   ├── issue.rs           # Issue routes
│   │   └── htmx.rs            # HTMX partial handlers
│   ├── services/
│   │   ├── mod.rs
│   │   ├── github.rs          # GitHub webhook processing
│   │   ├── repository.rs      # Repository service
│   │   ├── event.rs           # Event service
│   │   └── stats.rs           # Statistics service
│   ├── db/
│   │   ├── mod.rs
│   │   └── pool.rs            # Database connection pool
│   ├── templates/
│   │   ├── mod.rs
│   │   ├── layout.rs          # Base layout template
│   │   ├── dashboard.rs       # Dashboard templates
│   │   ├── repository.rs      # Repository templates
│   │   ├── event.rs           # Event templates
│   │   ├── commit.rs          # Commit templates
│   │   ├── pull_request.rs    # PR templates
│   │   ├── issue.rs           # Issue templates
│   │   └── components.rs      # Reusable components
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs            # Authentication middleware
│   └── utils/
│       ├── mod.rs
│       ├── signature.rs       # Webhook signature verification
│       └── pagination.rs      # Pagination helpers
└── static/
    ├── css/
    │   ├── style.css          # Custom styles
    │   └── tailwind.css       # Tailwind output (if building separately)
    └── js/
        └── app.js             # Additional JS if needed
```

## Key Implementation Details

### Webhook Signature Verification

GitHub signs webhook payloads with HMAC SHA256. Verification process:

1. Extract `X-Hub-Signature-256` header
2. Compute HMAC SHA256 of raw request body using configured secret
3. Compare computed signature with header value (constant-time comparison)
4. Reject request if signatures don't match

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_signature(secret: &[u8], payload: &[u8], signature: &str) -> bool {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .expect("HMAC can take key of any size");
    mac.update(payload);
    
    let expected = mac.finalize().into_bytes();
    let signature_hex = signature.strip_prefix("sha256=").unwrap_or(signature);
    
    // Constant-time comparison
    constant_time_eq(&expected, &hex::decode(signature_hex).unwrap_or_default())
}
```

### Event Processing Pipeline

1. **Reception**: Webhook handler receives POST request
2. **Verification**: Validate signature and headers
3. **Storage**: Store raw event in `webhook_events` table
4. **Parsing**: Parse event payload based on event type
5. **Extraction**: Extract relevant data (commits, PRs, issues)
6. **Persistence**: Store extracted data in respective tables
7. **Response**: Return 200 OK to GitHub

### HTMX Integration

HTMX enables dynamic updates without full page reloads:

- **Event list**: Infinite scroll using `hx-get` on scroll trigger
- **Search**: Live search with `hx-get` and debounce
- **Stats refresh**: Auto-refresh statistics every 30 seconds
- **Filters**: Dynamic filtering without page reload

Example Maud template with HTMX:

```rust
html! {
    div #events 
        hx-get="/htmx/events/recent" 
        hx-trigger="every 30s"
        hx-swap="innerHTML" {
        @for event in events {
            (render_event(event))
        }
    }
}
```

### Styling with Tailwind CSS and DaisyUI

The application uses Tailwind CSS for utility-first styling and DaisyUI for pre-built component classes:

- **Tailwind CSS**: Provides low-level utility classes for custom designs
- **DaisyUI**: Component library built on Tailwind, offering semantic class names for common UI patterns
  - Buttons, cards, badges, alerts
  - Form controls and inputs
  - Navigation components (navbar, tabs, breadcrumbs)
  - Data display (tables, stats, timelines)
  - Theme support with built-in color schemes

Example Maud template with DaisyUI components:

```rust
html! {
    div .card .bg-base-100 .shadow-xl {
        div .card-body {
            h2 .card-title { "Repository Activity" }
            div .stats .shadow {
                div .stat {
                    div .stat-title { "Total Commits" }
                    div .stat-value { (commit_count) }
                }
                div .stat {
                    div .stat-title { "Open PRs" }
                    div .stat-value .text-primary { (pr_count) }
                }
            }
            div .card-actions .justify-end {
                button .btn .btn-primary { "View Details" }
            }
        }
    }
}
```

**Benefits:**
- Rapid UI development with pre-styled components
- Consistent design system across the application
- Easy theming and dark mode support
- Minimal custom CSS required
- Responsive design out of the box

### Database Connection Pooling

Use `sqlx::PgPool` for efficient connection management:

```rust
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

### Error Handling

Implement custom error types for different scenarios:

- `WebhookError`: Signature verification, parsing errors
- `DatabaseError`: Query failures, connection issues
- `ValidationError`: Invalid data formats

## Configuration

Environment variables (`.env` file):

```env
# Server
HOST=0.0.0.0
PORT=8080

# Database
DATABASE_URL=postgres://user:password@localhost/github_observer

# GitHub
GITHUB_WEBHOOK_SECRET=your_webhook_secret_here

# Optional
RUST_LOG=info
MAX_CONNECTIONS=5
```

## Security Considerations

1. **Webhook Authentication**: Always verify GitHub signatures
2. **SQL Injection**: Use parameterized queries (sqlx handles this)
3. **XSS Protection**: Maud auto-escapes HTML by default
4. **Rate Limiting**: Consider implementing rate limits on webhook endpoint
5. **HTTPS**: Use reverse proxy (nginx/caddy) for TLS termination
6. **Secret Management**: Never commit webhook secrets to version control

## Deployment

### Development

```bash
# Setup database
createdb github_observer
sqlx migrate run

# Run server
cargo run
```

### Production

```bash
# Build optimized binary
cargo build --release

# Run with systemd or docker
./target/release/cross_bow
```

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/cross_bow /usr/local/bin/
CMD ["cross_bow"]
```

## Future Enhancements

- [ ] Real-time updates using Server-Sent Events (SSE)
- [ ] Email/Slack notifications for specific events
- [ ] Advanced analytics and visualizations
- [ ] Custom webhook filters and rules
- [ ] Export functionality (CSV, JSON)
- [ ] Multi-tenant support for multiple GitHub accounts/orgs
- [ ] GraphQL API for flexible data queries
- [ ] Webhook delivery retry mechanism
- [ ] Event replay functionality
- [ ] Custom event processors/plugins

## Performance Considerations

- **Indexing**: Proper database indexes for common queries
- **Connection pooling**: Reuse database connections
- **Async processing**: Non-blocking I/O with Tokio
- **Payload size**: Consider storing large payloads in object storage
- **Caching**: Cache frequently accessed data (repository info, stats)
- **Pagination**: Limit query results to prevent memory issues

## Testing Strategy

- **Unit tests**: Test individual functions (signature verification, parsing)
- **Integration tests**: Test database operations with test database
- **Handler tests**: Test HTTP handlers with mock requests
- **End-to-end tests**: Simulate GitHub webhook delivery
- **Load testing**: Verify performance under high webhook volume
