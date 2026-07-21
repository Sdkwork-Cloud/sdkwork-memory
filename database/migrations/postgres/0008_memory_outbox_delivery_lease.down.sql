DROP INDEX IF EXISTS idx_ai_outbox_event_delivery_lease;

ALTER TABLE ai_outbox_event
  DROP COLUMN IF EXISTS next_attempt_at,
  DROP COLUMN IF EXISTS lease_expires_at,
  DROP COLUMN IF EXISTS lease_token,
  DROP COLUMN IF EXISTS lease_owner;
