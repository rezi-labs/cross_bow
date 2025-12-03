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
