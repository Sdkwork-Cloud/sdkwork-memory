CREATE TABLE IF NOT EXISTS mem_space (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  organization_id INTEGER,
  owner_subject_type TEXT NOT NULL,
  owner_subject_id TEXT NOT NULL,
  space_type TEXT NOT NULL,
  display_name TEXT NOT NULL,
  default_scope TEXT NOT NULL,
  lifecycle_status TEXT NOT NULL,
  metadata_json TEXT,
  policy_json TEXT,
  created_by INTEGER,
  updated_by INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_space_uuid
  ON mem_space (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_space_owner_type
  ON mem_space (tenant_id, owner_subject_type, owner_subject_id, space_type);

CREATE TABLE IF NOT EXISTS mem_event (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  space_id INTEGER NOT NULL,
  user_id INTEGER,
  actor_type TEXT NOT NULL,
  actor_id TEXT,
  session_id TEXT,
  trace_id TEXT,
  request_id TEXT,
  idempotency_key TEXT,
  event_type TEXT NOT NULL,
  source_type TEXT NOT NULL,
  source_ref TEXT,
  event_time TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  payload_hash TEXT NOT NULL,
  sensitivity_level TEXT NOT NULL,
  ingestion_status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (space_id) REFERENCES mem_space(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_event_uuid
  ON mem_event (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_event_idempotency
  ON mem_event (tenant_id, idempotency_key)
  WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS mem_record (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  space_id INTEGER NOT NULL,
  user_id INTEGER,
  scope TEXT NOT NULL,
  memory_type TEXT NOT NULL,
  subject TEXT,
  predicate TEXT,
  object_text TEXT NOT NULL,
  canonical_text TEXT NOT NULL,
  summary_text TEXT,
  language TEXT,
  confidence REAL NOT NULL,
  evidence_count INTEGER NOT NULL DEFAULT 0,
  contradiction_count INTEGER NOT NULL DEFAULT 0,
  importance_score REAL NOT NULL,
  recency_score REAL NOT NULL,
  habit_strength REAL,
  valid_from TEXT,
  valid_to TEXT,
  expires_at TEXT,
  status TEXT NOT NULL,
  sensitivity_level TEXT NOT NULL,
  metadata_json TEXT,
  tags_json TEXT,
  supersedes_memory_id INTEGER,
  superseded_by_memory_id INTEGER,
  created_by INTEGER,
  updated_by INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  version INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (space_id) REFERENCES mem_space(id),
  FOREIGN KEY (supersedes_memory_id) REFERENCES mem_record(id),
  FOREIGN KEY (superseded_by_memory_id) REFERENCES mem_record(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_record_uuid
  ON mem_record (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_mem_record_scope_type_status
  ON mem_record (tenant_id, space_id, scope, memory_type, status, updated_at);

CREATE TABLE IF NOT EXISTS mem_record_source (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  memory_id INTEGER NOT NULL,
  event_id INTEGER NOT NULL,
  source_role TEXT NOT NULL,
  confidence_delta REAL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (memory_id) REFERENCES mem_record(id),
  FOREIGN KEY (event_id) REFERENCES mem_event(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_record_source_pair
  ON mem_record_source (tenant_id, memory_id, event_id, source_role);

CREATE TABLE IF NOT EXISTS mem_candidate (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  space_id INTEGER NOT NULL,
  user_id INTEGER,
  candidate_type TEXT NOT NULL,
  memory_type TEXT NOT NULL,
  proposed_text TEXT NOT NULL,
  proposed_payload_json TEXT,
  target_memory_id INTEGER,
  evidence_json TEXT,
  confidence REAL NOT NULL,
  novelty_score REAL,
  risk_score REAL,
  decision_state TEXT NOT NULL,
  decision_reason TEXT,
  decided_by INTEGER,
  decided_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (space_id) REFERENCES mem_space(id),
  FOREIGN KEY (target_memory_id) REFERENCES mem_record(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_candidate_uuid
  ON mem_candidate (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_habit (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  space_id INTEGER NOT NULL,
  user_id INTEGER NOT NULL,
  habit_key TEXT NOT NULL,
  habit_type TEXT NOT NULL,
  description TEXT NOT NULL,
  stage TEXT NOT NULL,
  strength REAL NOT NULL,
  confidence REAL NOT NULL,
  support_count INTEGER NOT NULL DEFAULT 0,
  last_signal_at TEXT,
  promoted_memory_id INTEGER,
  decay_after TEXT,
  metadata_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (space_id) REFERENCES mem_space(id),
  FOREIGN KEY (promoted_memory_id) REFERENCES mem_record(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_habit_key
  ON mem_habit (tenant_id, space_id, user_id, habit_key);

CREATE TABLE IF NOT EXISTS mem_retrieval_trace (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  space_id INTEGER,
  retrieval_profile_id INTEGER,
  actor_id TEXT,
  query_text TEXT,
  query_hash TEXT NOT NULL,
  retrievers_json TEXT,
  latency_ms INTEGER,
  result_count INTEGER NOT NULL DEFAULT 0,
  degraded INTEGER NOT NULL DEFAULT 0,
  metadata_json TEXT,
  created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_retrieval_trace_uuid
  ON mem_retrieval_trace (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_retrieval_hit (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  retrieval_trace_id INTEGER NOT NULL,
  memory_id INTEGER,
  retriever_name TEXT NOT NULL,
  result_rank INTEGER NOT NULL,
  raw_score REAL,
  fused_score REAL,
  explanation_json TEXT,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (retrieval_trace_id) REFERENCES mem_retrieval_trace(id),
  FOREIGN KEY (memory_id) REFERENCES mem_record(id)
);

CREATE INDEX IF NOT EXISTS idx_mem_retrieval_hit_trace_rank
  ON mem_retrieval_hit (tenant_id, retrieval_trace_id, result_rank);

CREATE TABLE IF NOT EXISTS mem_context_pack (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  retrieval_trace_id INTEGER,
  actor_id TEXT,
  query_text TEXT,
  pack_json TEXT NOT NULL,
  estimated_tokens INTEGER NOT NULL,
  truncated INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  FOREIGN KEY (retrieval_trace_id) REFERENCES mem_retrieval_trace(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_context_pack_uuid
  ON mem_context_pack (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_audit_log (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  actor_type TEXT NOT NULL,
  actor_id TEXT,
  action TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id TEXT,
  request_id TEXT,
  trace_id TEXT,
  result TEXT NOT NULL,
  reason TEXT,
  metadata_json TEXT,
  created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_audit_log_uuid
  ON mem_audit_log (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_outbox_event (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL,
  tenant_id INTEGER NOT NULL,
  aggregate_type TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  event_version TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  publish_state TEXT NOT NULL,
  published_at TEXT,
  retry_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_outbox_event_uuid
  ON mem_outbox_event (tenant_id, uuid);
