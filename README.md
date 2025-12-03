# Cross Bow

A webhook-based GitHub event monitoring system built with Rust, Actix-web, HTMX, Maud, and PostgreSQL. Cross Bow receives GitHub webhook events, stores them in a database, and provides a web interface for viewing and analyzing repository activity.

## Features

- **GitHub Webhook Integration**: Securely receives and processes GitHub webhook events with signature verification
- **Event Processing**: Automatically extracts and stores commits, pull requests, and issues from webhook payloads
- **Repository Tracking**: Monitors multiple repositories and their activity
- **Web Dashboard**: Clean, modern UI built with DaisyUI and Tailwind CSS
- **Database Storage**: PostgreSQL with full-text search and JSON indexing
- **Async Architecture**: Built on Tokio for high-performance, non-blocking I/O

## Technology Stack

- **Backend**: Rust with Actix-web
- **Frontend**: HTMX + Maud (server-side HTML templating)
- **Styling**: Tailwind CSS + DaisyUI
- **Database**: PostgreSQL with sqlx
- **Event Processing**: Async webhook processing with signature verification

## Quick Start

### Prerequisites

- Rust 1.75 or later
- PostgreSQL 14 or later
- A GitHub repository with webhook access

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd cross_bow
```

2. Create a `.env` file based on `.env.example`:
```bash
cp .env.example .env
```

3. Configure your environment variables in `.env`:
```env
DATABASE_URL=postgres://user:password@localhost/github_observer
GITHUB_WEBHOOK_SECRET=your_webhook_secret_here
HOST=0.0.0.0
PORT=8080
```

4. Create the database:
```bash
createdb github_observer
```

5. Build and run:
```bash
cargo run
```

Or use the justfile:
```bash
just run
```

The server will start on `http://0.0.0.0:8080`

## Database Migrations

Database migrations are run automatically when the application starts. The migrations create the following tables:

- `repositories`: Tracked GitHub repositories
- `webhook_events`: Raw webhook events from GitHub
- `commits`: Extracted commit information
- `pull_requests`: Pull request data
- `issues`: Issue tracking data

## GitHub Webhook Setup

1. Go to your GitHub repository settings
2. Navigate to Webhooks → Add webhook
3. Set the Payload URL to: `http://your-server:8080/webhooks/github`
4. Set Content type to: `application/json`
5. Set the Secret to match your `GITHUB_WEBHOOK_SECRET`
6. Select individual events or "Send me everything"
7. Save the webhook

## API Endpoints

### Webhook Endpoint
- `POST /webhooks/github` - Receives GitHub webhook events

### Web Interface
- `GET /` - Dashboard with statistics
- `GET /repositories` - List all tracked repositories
- `GET /repositories/{id}` - Repository detail page with commits, PRs, and issues

## Development

### Build Commands

```bash
# Run tests
just test

# Run linter
just lint

# Format code
just fmt

# Verify everything (lint + test)
just verify

# Build release
cargo build --release
```

### Project Structure

```
cross_bow/
├── src/
│   ├── config.rs           # Configuration management
│   ├── db/                 # Database connection pooling
│   ├── handlers/           # HTTP request handlers
│   │   ├── dashboard.rs    # Dashboard UI
│   │   ├── repositories.rs # Repository views
│   │   └── webhook.rs      # Webhook handler
│   ├── models/             # Database models
│   │   ├── commit.rs
│   │   ├── issue.rs
│   │   ├── pull_request.rs
│   │   ├── repository.rs
│   │   └── webhook_event.rs
│   ├── services/           # Business logic
│   │   └── github.rs       # GitHub event processing
│   ├── utils/              # Utilities
│   │   ├── pagination.rs   # Pagination helpers
│   │   └── signature.rs    # Webhook signature verification
│   └── main.rs             # Application entry point
├── migrations/             # SQL migrations
├── assets/                 # Static assets (CSS, JS)
└── Cargo.toml             # Dependencies
```

## Security

- **Webhook Signature Verification**: All webhook requests are verified using HMAC SHA256
- **SQL Injection Protection**: sqlx with compile-time query checking
- **XSS Protection**: Maud auto-escapes HTML by default
- **Constant-time Comparison**: Signature verification uses constant-time equality checks

## License

See LICENSE file for details.

## Future Enhancements

- Real-time updates using Server-Sent Events (SSE)
- Email/Slack notifications for specific events
- Advanced analytics and visualizations
- Custom webhook filters and rules
- Export functionality (CSV, JSON)
- Multi-tenant support for multiple GitHub accounts/orgs
