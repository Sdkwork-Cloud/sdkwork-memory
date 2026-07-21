-- Extend ai_eval_run to align with schema-registry 005-memory-governance.yaml.
-- Adds dataset_ref, profile_ref, result_json, started_at, finished_at columns
-- declared in the design contract but absent from 0001_memory_phase1.

ALTER TABLE ai_eval_run ADD COLUMN IF NOT EXISTS dataset_ref VARCHAR(256);
ALTER TABLE ai_eval_run ADD COLUMN IF NOT EXISTS profile_ref VARCHAR(256);
ALTER TABLE ai_eval_run ADD COLUMN IF NOT EXISTS result_json TEXT;
ALTER TABLE ai_eval_run ADD COLUMN IF NOT EXISTS started_at TEXT;
ALTER TABLE ai_eval_run ADD COLUMN IF NOT EXISTS finished_at TEXT;
