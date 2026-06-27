-- Tenant-level and user-level preference store (schema-registry 005-memory-governance.yaml).

CREATE TABLE IF NOT EXISTS ai_tenant_preference (
  id INTEGER PRIMARY KEY,
  tenant_id INTEGER NOT NULL,
  user_id INTEGER,
  preference_key TEXT NOT NULL,
  preference_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0
);

-- SQLite does not support NULLS NOT DISTINCT; use partial indexes to enforce
-- uniqueness for both user-level and tenant-level (user_id IS NULL) preferences.
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_tenant_preference_scope_user
  ON ai_tenant_preference (tenant_id, user_id, preference_key)
  WHERE user_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_tenant_preference_scope_tenant
  ON ai_tenant_preference (tenant_id, preference_key)
  WHERE user_id IS NULL;
