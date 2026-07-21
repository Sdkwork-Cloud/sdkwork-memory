-- Tenant-level and user-level preference store (schema-registry 005-memory-governance.yaml).

CREATE TABLE IF NOT EXISTS ai_tenant_preference (
  id BIGINT NOT NULL PRIMARY KEY,
  tenant_id BIGINT NOT NULL,
  user_id BIGINT,
  preference_key TEXT NOT NULL,
  preference_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

-- NULLS NOT DISTINCT ensures tenant-level preferences (user_id IS NULL) remain unique
-- per (tenant_id, preference_key). Requires PostgreSQL 15+.
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_tenant_preference_scope
  ON ai_tenant_preference (tenant_id, user_id, preference_key)
  NULLS NOT DISTINCT;
