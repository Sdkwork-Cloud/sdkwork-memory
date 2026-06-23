-- Phase-1 secondary indexes materialized from docs/schema-registry (DATABASE_SPEC alignment).

CREATE INDEX IF NOT EXISTS idx_mem_space_tenant_status
  ON mem_space (tenant_id, lifecycle_status, updated_at);

CREATE INDEX IF NOT EXISTS idx_mem_event_space_time
  ON mem_event (tenant_id, space_id, event_time, id);

CREATE INDEX IF NOT EXISTS idx_mem_event_session_time
  ON mem_event (tenant_id, session_id, event_time);

CREATE INDEX IF NOT EXISTS idx_mem_event_type_time
  ON mem_event (tenant_id, event_type, event_time);

CREATE INDEX IF NOT EXISTS idx_mem_event_hash
  ON mem_event (tenant_id, payload_hash);

CREATE INDEX IF NOT EXISTS idx_mem_record_user_type
  ON mem_record (tenant_id, user_id, memory_type, status, updated_at);

CREATE INDEX IF NOT EXISTS idx_mem_record_subject_predicate
  ON mem_record (tenant_id, space_id, subject, predicate, status);

CREATE INDEX IF NOT EXISTS idx_mem_record_validity
  ON mem_record (tenant_id, valid_from, valid_to, expires_at);

CREATE INDEX IF NOT EXISTS idx_mem_record_supersession
  ON mem_record (tenant_id, supersedes_memory_id, superseded_by_memory_id);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_record_source_uuid
  ON mem_record_source (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_mem_record_source_event
  ON mem_record_source (tenant_id, event_id);

CREATE INDEX IF NOT EXISTS idx_mem_candidate_state
  ON mem_candidate (tenant_id, space_id, decision_state, updated_at);

CREATE INDEX IF NOT EXISTS idx_mem_candidate_target
  ON mem_candidate (tenant_id, target_memory_id);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_habit_uuid
  ON mem_habit (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_mem_habit_stage
  ON mem_habit (tenant_id, space_id, stage, confidence, updated_at);

CREATE INDEX IF NOT EXISTS idx_mem_retrieval_trace_profile_created
  ON mem_retrieval_trace (tenant_id, retrieval_profile_id, created_at);

CREATE INDEX IF NOT EXISTS idx_mem_retrieval_trace_actor_created
  ON mem_retrieval_trace (tenant_id, actor_id, created_at);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_retrieval_hit_uuid
  ON mem_retrieval_hit (tenant_id, uuid);

CREATE INDEX IF NOT EXISTS idx_mem_retrieval_hit_memory
  ON mem_retrieval_hit (tenant_id, memory_id, status);

CREATE INDEX IF NOT EXISTS idx_mem_context_pack_trace
  ON mem_context_pack (tenant_id, retrieval_trace_id);

CREATE INDEX IF NOT EXISTS idx_mem_context_pack_actor_created
  ON mem_context_pack (tenant_id, actor_id, created_at);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_index_kind_space
  ON mem_index (tenant_id, space_id, index_kind, schema_version);

CREATE INDEX IF NOT EXISTS idx_mem_index_status
  ON mem_index (tenant_id, space_id, index_kind, status);

CREATE INDEX IF NOT EXISTS idx_mem_retrieval_profile_scope
  ON mem_retrieval_profile (tenant_id, space_id, status, updated_at);

CREATE INDEX IF NOT EXISTS idx_mem_implementation_profile_kind
  ON mem_implementation_profile (tenant_id, implementation_kind, status);

CREATE UNIQUE INDEX IF NOT EXISTS uk_mem_provider_binding_code
  ON mem_provider_binding (tenant_id, provider_kind, provider_code);

CREATE INDEX IF NOT EXISTS idx_mem_provider_binding_health
  ON mem_provider_binding (tenant_id, provider_kind, health_state, updated_at);

CREATE INDEX IF NOT EXISTS idx_mem_eval_run_type_state
  ON mem_eval_run (tenant_id, eval_type, state, created_at);

CREATE INDEX IF NOT EXISTS idx_mem_audit_actor_time
  ON mem_audit_log (tenant_id, actor_type, actor_id, created_at);

CREATE INDEX IF NOT EXISTS idx_mem_audit_resource_time
  ON mem_audit_log (tenant_id, resource_type, resource_id, created_at);

CREATE INDEX IF NOT EXISTS idx_mem_audit_action_time
  ON mem_audit_log (tenant_id, action, created_at);

CREATE INDEX IF NOT EXISTS idx_mem_outbox_state
  ON mem_outbox_event (tenant_id, publish_state, created_at);
