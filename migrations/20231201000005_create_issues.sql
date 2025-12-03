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
