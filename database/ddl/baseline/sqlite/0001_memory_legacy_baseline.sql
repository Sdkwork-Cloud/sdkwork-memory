-- Consolidated legacy baseline for sdkwork-memory database module.
-- source: plugins/sdkwork-memory-plugin-native-sql/migrations/postgres/V202606100001__memory_phase1.sql

CREATE TABLE IF NOT EXISTS mem_space (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  organization_id BIGINT,
  owner_subject_type VARCHAR(32) NOT NULL,
  owner_subject_id VARCHAR(128) NOT NULL,
  space_type VARCHAR(32) NOT NULL,
  display_name VARCHAR(200) NOT NULL,
  default_scope VARCHAR(32) NOT NULL,
  lifecycle_status VARCHAR(32) NOT NULL,
  metadata_json JSONB,
  policy_json JSONB,
  created_by BIGINT,
  updated_by BIGINT,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_space_uuid
  ON mem_space (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_space_owner_type
  ON mem_space (tenant_id, owner_subject_type, owner_subject_id, space_type);

CREATE TABLE IF NOT EXISTS mem_event (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  space_id BIGINT NOT NULL REFERENCES mem_space(id),
  user_id BIGINT,
  actor_type VARCHAR(32) NOT NULL,
  actor_id VARCHAR(128),
  session_id VARCHAR(128),
  trace_id VARCHAR(128),
  request_id VARCHAR(64),
  idempotency_key VARCHAR(128),
  event_type VARCHAR(64) NOT NULL,
  source_type VARCHAR(64) NOT NULL,
  source_ref VARCHAR(256),
  event_time TIMESTAMPTZ NOT NULL,
  payload_json JSONB NOT NULL,
  payload_hash VARCHAR(128) NOT NULL,
  sensitivity_level VARCHAR(32) NOT NULL,
  ingestion_status VARCHAR(32) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_event_uuid
  ON mem_event (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_event_idempotency
  ON mem_event (tenant_id, idempotency_key)
  WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS mem_record (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  space_id BIGINT NOT NULL REFERENCES mem_space(id),
  user_id BIGINT,
  scope VARCHAR(32) NOT NULL,
  memory_type VARCHAR(32) NOT NULL,
  subject VARCHAR(256),
  predicate VARCHAR(128),
  object_text TEXT NOT NULL,
  canonical_text TEXT NOT NULL,
  summary_text TEXT,
  language VARCHAR(16),
  confidence DECIMAL(5,4) NOT NULL,
  evidence_count INTEGER NOT NULL DEFAULT 0,
  contradiction_count INTEGER NOT NULL DEFAULT 0,
  importance_score DECIMAL(5,4) NOT NULL,
  recency_score DECIMAL(5,4) NOT NULL,
  habit_strength DECIMAL(5,4),
  valid_from TIMESTAMPTZ,
  valid_to TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  status VARCHAR(32) NOT NULL,
  sensitivity_level VARCHAR(32) NOT NULL,
  metadata_json JSONB,
  tags_json JSONB,
  supersedes_memory_id BIGINT REFERENCES mem_record(id),
  superseded_by_memory_id BIGINT REFERENCES mem_record(id),
  created_by BIGINT,
  updated_by BIGINT,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_record_uuid
  ON mem_record (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_mem_record_scope_type_status
  ON mem_record (tenant_id, space_id, scope, memory_type, status, updated_at);

CREATE TABLE IF NOT EXISTS mem_record_source (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  memory_id BIGINT NOT NULL REFERENCES mem_record(id),
  event_id BIGINT NOT NULL REFERENCES mem_event(id),
  source_role VARCHAR(32) NOT NULL,
  confidence_delta DECIMAL(5,4),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_record_source_pair
  ON mem_record_source (tenant_id, memory_id, event_id, source_role);

CREATE TABLE IF NOT EXISTS mem_candidate (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  space_id BIGINT NOT NULL REFERENCES mem_space(id),
  user_id BIGINT,
  candidate_type VARCHAR(32) NOT NULL,
  memory_type VARCHAR(32) NOT NULL,
  proposed_text TEXT NOT NULL,
  proposed_payload_json JSONB,
  target_memory_id BIGINT REFERENCES mem_record(id),
  evidence_json JSONB,
  confidence DECIMAL(5,4) NOT NULL,
  novelty_score DECIMAL(5,4),
  risk_score DECIMAL(5,4),
  decision_state VARCHAR(32) NOT NULL,
  decision_reason VARCHAR(256),
  decided_by BIGINT,
  decided_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_candidate_uuid
  ON mem_candidate (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_habit (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  space_id BIGINT NOT NULL REFERENCES mem_space(id),
  user_id BIGINT NOT NULL,
  habit_key VARCHAR(160) NOT NULL,
  habit_type VARCHAR(64) NOT NULL,
  description TEXT NOT NULL,
  stage VARCHAR(32) NOT NULL,
  strength DECIMAL(5,4) NOT NULL,
  confidence DECIMAL(5,4) NOT NULL,
  support_count INTEGER NOT NULL DEFAULT 0,
  last_signal_at TIMESTAMPTZ,
  promoted_memory_id BIGINT REFERENCES mem_record(id),
  decay_after TIMESTAMPTZ,
  metadata_json JSONB,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_habit_key
  ON mem_habit (tenant_id, space_id, user_id, habit_key);

CREATE TABLE IF NOT EXISTS mem_retrieval_trace (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  space_id BIGINT,
  retrieval_profile_id BIGINT,
  actor_id VARCHAR(128),
  query_text TEXT,
  query_hash VARCHAR(128) NOT NULL,
  retrievers_json JSONB,
  latency_ms INTEGER,
  result_count INTEGER NOT NULL DEFAULT 0,
  degraded BOOLEAN NOT NULL DEFAULT FALSE,
  metadata_json JSONB,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_retrieval_trace_uuid
  ON mem_retrieval_trace (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_retrieval_hit (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  retrieval_trace_id BIGINT NOT NULL REFERENCES mem_retrieval_trace(id),
  memory_id BIGINT REFERENCES mem_record(id),
  retriever_name VARCHAR(64) NOT NULL,
  result_rank INTEGER NOT NULL,
  raw_score DECIMAL(10,6),
  fused_score DECIMAL(10,6),
  explanation_json JSONB,
  status VARCHAR(32) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_mem_retrieval_hit_trace_rank
  ON mem_retrieval_hit (tenant_id, retrieval_trace_id, result_rank);

CREATE TABLE IF NOT EXISTS mem_context_pack (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  retrieval_trace_id BIGINT REFERENCES mem_retrieval_trace(id),
  actor_id VARCHAR(128),
  query_text TEXT,
  pack_json JSONB NOT NULL,
  estimated_tokens INTEGER NOT NULL,
  truncated BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_context_pack_uuid
  ON mem_context_pack (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_index (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  space_id BIGINT,
  index_kind VARCHAR(32) NOT NULL,
  implementation_profile_id BIGINT,
  provider_binding_id BIGINT,
  schema_version VARCHAR(32) NOT NULL,
  status VARCHAR(32) NOT NULL,
  rebuild_cursor VARCHAR(256),
  config_json JSONB,
  last_rebuilt_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_index_uuid
  ON mem_index (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_retrieval_profile (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  space_id BIGINT,
  name VARCHAR(160) NOT NULL,
  strategy VARCHAR(64) NOT NULL,
  retrievers_json JSONB NOT NULL,
  fusion_policy_json JSONB,
  rerank_policy_json JSONB,
  top_k INTEGER NOT NULL,
  context_budget_tokens INTEGER NOT NULL,
  status VARCHAR(32) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_retrieval_profile_uuid
  ON mem_retrieval_profile (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_implementation_profile (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  name VARCHAR(160) NOT NULL,
  implementation_kind VARCHAR(64) NOT NULL,
  role VARCHAR(32) NOT NULL,
  status VARCHAR(32) NOT NULL,
  capability_json JSONB NOT NULL,
  config_json JSONB,
  rollout_json JSONB,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_implementation_profile_uuid
  ON mem_implementation_profile (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_provider_binding (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  provider_kind VARCHAR(32) NOT NULL,
  provider_code VARCHAR(128) NOT NULL,
  display_name VARCHAR(160) NOT NULL,
  endpoint_ref VARCHAR(256),
  secret_ref VARCHAR(256),
  model_ref VARCHAR(256),
  capabilities_json JSONB NOT NULL,
  config_json JSONB,
  health_state VARCHAR(32) NOT NULL,
  last_health_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_provider_binding_uuid
  ON mem_provider_binding (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_eval_run (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  eval_type VARCHAR(64) NOT NULL,
  state VARCHAR(32) NOT NULL,
  metrics_json JSONB,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_eval_run_uuid
  ON mem_eval_run (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_audit_log (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  actor_type VARCHAR(32) NOT NULL,
  actor_id VARCHAR(128),
  action VARCHAR(128) NOT NULL,
  resource_type VARCHAR(64) NOT NULL,
  resource_id VARCHAR(128),
  request_id VARCHAR(64),
  trace_id VARCHAR(128),
  result VARCHAR(32) NOT NULL,
  reason VARCHAR(256),
  metadata_json JSONB,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_audit_log_uuid
  ON mem_audit_log (tenant_id, uuid);

CREATE TABLE IF NOT EXISTS mem_outbox_event (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  aggregate_type VARCHAR(64) NOT NULL,
  aggregate_id VARCHAR(128) NOT NULL,
  event_type VARCHAR(128) NOT NULL,
  event_version VARCHAR(32) NOT NULL,
  payload_json JSONB NOT NULL,
  publish_state VARCHAR(32) NOT NULL,
  published_at TIMESTAMPTZ,
  retry_count INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_outbox_event_uuid
  ON mem_outbox_event (tenant_id, uuid);
