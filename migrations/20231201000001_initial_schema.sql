-- Initial schema for cross_bow
-- Consolidated migration containing all tables and indexes

-- Create repositories table
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

-- Create webhook_events table
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

-- Create commits table
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

-- Create pull_requests table
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

-- Create issues table
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

-- Create generic events table
CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    source VARCHAR(50) NOT NULL, -- github, gitlab, bitbucket, auth0, etc.
    event_type VARCHAR(100) NOT NULL,
    action VARCHAR(100),
    actor_name VARCHAR(255),
    actor_email VARCHAR(255),
    actor_id VARCHAR(255), -- can be username, id, or any identifier from the source
    raw_event JSONB NOT NULL,
    delivery_id UUID NOT NULL UNIQUE,
    signature VARCHAR(255), -- optional, not all webhooks have signatures
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ,
    repository_id BIGINT REFERENCES repositories(id) ON DELETE CASCADE
);

CREATE INDEX idx_events_source ON events(source);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_received ON events(received_at DESC);
CREATE INDEX idx_events_delivery ON events(delivery_id);
CREATE INDEX idx_events_actor ON events(actor_id);
CREATE INDEX idx_events_repository ON events(repository_id);
CREATE INDEX idx_events_raw ON events USING gin(raw_event);
