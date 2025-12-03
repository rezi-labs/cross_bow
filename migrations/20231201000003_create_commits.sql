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
