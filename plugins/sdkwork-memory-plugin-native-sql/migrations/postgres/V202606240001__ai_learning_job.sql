-- Async learning/governance job queue (schema-registry 002-memory-learning.yaml).

CREATE TABLE IF NOT EXISTS ai_learning_job (
  id BIGINT PRIMARY KEY,
  uuid VARCHAR(64) NOT NULL,
  tenant_id BIGINT NOT NULL,
  space_id BIGINT REFERENCES ai_space(id),
  job_type VARCHAR(64) NOT NULL,
  state VARCHAR(32) NOT NULL,
  priority INT NOT NULL DEFAULT 0,
  idempotency_key VARCHAR(128),
  input_json JSONB,
  result_json JSONB,
  error_json JSONB,
  started_at TIMESTAMPTZ,
  finished_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  version BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_learning_job_uuid
  ON ai_learning_job (tenant_id, uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_learning_job_idempotency
  ON ai_learning_job (tenant_id, job_type, idempotency_key)
  WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_ai_learning_job_state
  ON ai_learning_job (tenant_id, job_type, state, priority DESC, created_at ASC);
