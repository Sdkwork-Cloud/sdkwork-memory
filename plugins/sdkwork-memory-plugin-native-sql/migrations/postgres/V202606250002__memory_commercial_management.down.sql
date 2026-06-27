-- Reverse 0007_memory_commercial_management for postgres.
-- Drops commercial management tables and activated planned tables in reverse
-- dependency order to satisfy foreign key constraints.

DROP TABLE IF EXISTS ai_commercial_readiness_snapshot;
DROP TABLE IF EXISTS ai_relation_rebuild_job;
DROP TABLE IF EXISTS ai_policy_assignment;
DROP TABLE IF EXISTS ai_capability_binding;
DROP TABLE IF EXISTS ai_memory_binding;
DROP TABLE IF EXISTS ai_subject;
DROP TABLE IF EXISTS ai_policy;
DROP TABLE IF EXISTS ai_edge;
DROP TABLE IF EXISTS ai_entity;
