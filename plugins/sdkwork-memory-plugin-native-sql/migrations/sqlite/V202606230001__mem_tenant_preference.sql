CREATE TABLE IF NOT EXISTS mem_tenant_preference (
  id INTEGER PRIMARY KEY,
  tenant_id INTEGER NOT NULL,
  user_id INTEGER,
  preference_key TEXT NOT NULL,
  preference_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_tenant_preference_scope
  ON mem_tenant_preference (tenant_id, user_id, preference_key);
