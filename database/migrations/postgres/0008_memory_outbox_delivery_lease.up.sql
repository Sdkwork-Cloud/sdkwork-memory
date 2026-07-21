ALTER TABLE ai_outbox_event
  ADD COLUMN IF NOT EXISTS lease_owner VARCHAR(128),
  ADD COLUMN IF NOT EXISTS lease_token VARCHAR(128),
  ADD COLUMN IF NOT EXISTS lease_expires_at TEXT,
  ADD COLUMN IF NOT EXISTS next_attempt_at TEXT;

CREATE INDEX IF NOT EXISTS idx_ai_outbox_event_delivery_lease
  ON ai_outbox_event (publish_state, next_attempt_at, lease_expires_at, id);
