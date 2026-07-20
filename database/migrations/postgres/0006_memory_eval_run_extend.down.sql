-- Reverse 0006_ai_eval_run_extend: drop the five governance columns added
-- to align ai_eval_run with schema-registry 005-memory-governance.yaml.

ALTER TABLE ai_eval_run DROP COLUMN IF EXISTS finished_at;
ALTER TABLE ai_eval_run DROP COLUMN IF EXISTS started_at;
ALTER TABLE ai_eval_run DROP COLUMN IF EXISTS result_json;
ALTER TABLE ai_eval_run DROP COLUMN IF EXISTS profile_ref;
ALTER TABLE ai_eval_run DROP COLUMN IF EXISTS dataset_ref;
