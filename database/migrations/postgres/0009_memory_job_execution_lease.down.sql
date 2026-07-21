DROP INDEX IF EXISTS idx_ai_eval_run_execution_lease;
DROP INDEX IF EXISTS idx_ai_learning_job_execution_lease;

ALTER TABLE ai_eval_run
  DROP COLUMN IF EXISTS lease_expires_at,
  DROP COLUMN IF EXISTS lease_token,
  DROP COLUMN IF EXISTS lease_owner;
ALTER TABLE ai_learning_job
  DROP COLUMN IF EXISTS lease_expires_at,
  DROP COLUMN IF EXISTS lease_token,
  DROP COLUMN IF EXISTS lease_owner;
