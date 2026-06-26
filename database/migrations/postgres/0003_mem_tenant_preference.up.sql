CREATE TABLE IF NOT EXISTS ai_tenant_preference (
  id BIGSERIAL PRIMARY KEY,
  tenant_id BIGINT NOT NULL,
  user_id BIGINT,
  preference_key TEXT NOT NULL,
  preference_json JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_tenant_preference_scope
  ON ai_tenant_preference (tenant_id, user_id, preference_key);
