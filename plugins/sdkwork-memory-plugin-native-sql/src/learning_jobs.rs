//! Learning job queue persistence (`ai_learning_job`).

use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::store::{now_text, NativeSqlMemoryStore, NativeSqlStoreError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeSqlLearningJobRow {
    pub job_uuid: String,
    pub tenant_id: i64,
    pub space_id: Option<i64>,
    pub job_type: String,
    pub state: String,
    pub priority: i32,
    pub input_json: Option<String>,
    pub result_json: Option<String>,
    pub error_json: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

pub struct InsertLearningJobCommand<'a> {
    pub tenant_id: i64,
    pub job_uuid: &'a str,
    pub space_id: Option<i64>,
    pub job_type: &'a str,
    pub state: &'a str,
    pub priority: i32,
    pub idempotency_key: Option<&'a str>,
    pub input_json: Option<&'a str>,
}

impl NativeSqlMemoryStore {
    pub async fn insert_learning_job(
        &self,
        command: InsertLearningJobCommand<'_>,
    ) -> Result<(), NativeSqlStoreError> {
        let now = now_text();
        sqlx::query(
            r#"
            INSERT INTO ai_learning_job (
              id, uuid, tenant_id, space_id, job_type, state, priority,
              idempotency_key, input_json, created_at, updated_at, version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
            "#,
        )
        .bind(
            command
                .job_uuid
                .parse::<i64>()
                .unwrap_or(command.tenant_id),
        )
        .bind(command.job_uuid)
        .bind(command.tenant_id)
        .bind(command.space_id)
        .bind(command.job_type)
        .bind(command.state)
        .bind(command.priority)
        .bind(command.idempotency_key)
        .bind(command.input_json)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn claim_queued_learning_jobs(
        &self,
        limit: i32,
    ) -> Result<Vec<NativeSqlLearningJobRow>, NativeSqlStoreError> {
        let mut connection = self.pool().acquire().await?;
        let mut claimed = Vec::new();
        let rows = sqlx::query(
            r#"
            SELECT uuid, tenant_id, space_id, job_type, state, priority,
                   input_json, result_json, error_json,
                   started_at, finished_at, created_at, updated_at, version
            FROM ai_learning_job
            WHERE state = 'queued'
            ORDER BY priority DESC, created_at ASC
            LIMIT ?
            "#,
        )
        .bind(limit.max(1) as i64)
        .fetch_all(&mut *connection)
        .await?;

        for row in rows {
            let job_uuid: String = row.get("uuid");
            let tenant_id: i64 = row.get("tenant_id");
            let updated = sqlx::query(
                r#"
                UPDATE ai_learning_job
                SET state = 'running',
                    started_at = ?,
                    updated_at = ?,
                    version = version + 1
                WHERE tenant_id = ?
                  AND uuid = ?
                  AND state = 'queued'
                "#,
            )
            .bind(now_text())
            .bind(now_text())
            .bind(tenant_id)
            .bind(&job_uuid)
            .execute(&mut *connection)
            .await?;
            if updated.rows_affected() == 0 {
                continue;
            }
            claimed.push(map_learning_job_row(row));
        }
        Ok(claimed)
    }

    pub async fn retrieve_learning_job_for_tenant(
        &self,
        tenant_id: i64,
        job_uuid: &str,
    ) -> Result<Option<NativeSqlLearningJobRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, tenant_id, space_id, job_type, state, priority,
                   input_json, result_json, error_json,
                   started_at, finished_at, created_at, updated_at, version
            FROM ai_learning_job
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(tenant_id)
        .bind(job_uuid)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(map_learning_job_row))
    }

    pub async fn finish_learning_job(
        &self,
        tenant_id: i64,
        job_uuid: &str,
        state: &str,
        result_json: Option<&str>,
        error_json: Option<&str>,
    ) -> Result<Option<NativeSqlLearningJobRow>, NativeSqlStoreError> {
        let now = now_text();
        sqlx::query(
            r#"
            UPDATE ai_learning_job
            SET state = ?,
                result_json = COALESCE(?, result_json),
                error_json = COALESCE(?, error_json),
                finished_at = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND uuid = ? AND state = 'running'
            "#,
        )
        .bind(state)
        .bind(result_json)
        .bind(error_json)
        .bind(&now)
        .bind(&now)
        .bind(tenant_id)
        .bind(job_uuid)
        .execute(self.pool())
        .await?;
        self.retrieve_learning_job_for_tenant(tenant_id, job_uuid)
            .await
    }

    pub async fn update_eval_run_state(
        &self,
        tenant_id: i64,
        eval_run_uuid: &str,
        state: &str,
        metrics_json: Option<&str>,
        result_json: Option<&str>,
    ) -> Result<(), NativeSqlStoreError> {
        let now = now_text();
        sqlx::query(
            r#"
            UPDATE ai_eval_run
            SET state = ?,
                metrics_json = COALESCE(?, metrics_json),
                result_json = COALESCE(?, result_json),
                updated_at = ?
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(state)
        .bind(metrics_json)
        .bind(result_json)
        .bind(&now)
        .bind(tenant_id)
        .bind(eval_run_uuid)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn list_queued_eval_runs(
        &self,
        limit: i32,
    ) -> Result<Vec<(i64, String, String)>, NativeSqlStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT tenant_id, uuid, eval_type
            FROM ai_eval_run
            WHERE state IN ('accepted', 'queued')
            ORDER BY created_at ASC
            LIMIT ?
            "#,
        )
        .bind(limit.max(1) as i64)
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.get("tenant_id"),
                    row.get("uuid"),
                    row.get("eval_type"),
                )
            })
            .collect())
    }
}

fn map_learning_job_row(row: sqlx::any::AnyRow) -> NativeSqlLearningJobRow {
    NativeSqlLearningJobRow {
        job_uuid: row.get("uuid"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        job_type: row.get("job_type"),
        state: row.get("state"),
        priority: row.get("priority"),
        input_json: row.get("input_json"),
        result_json: row.get("result_json"),
        error_json: row.get("error_json"),
        started_at: row.get("started_at"),
        finished_at: row.get("finished_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: row.get("version"),
    }
}
