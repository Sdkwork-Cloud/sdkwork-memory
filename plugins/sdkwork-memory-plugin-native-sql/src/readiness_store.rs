//! Commercial readiness snapshot store methods.

use sqlx::Row;

use crate::store::{now_text, NativeSqlMemoryStore, NativeSqlStoreError};

#[derive(Debug, Clone)]
pub struct NativeSqlCommercialReadinessRow {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: i64,
    pub implementation_profile_id: Option<i64>,
    pub score: f64,
    pub state: String,
    pub contract_coverage_json: Option<String>,
    pub management_coverage_json: Option<String>,
    pub runtime_conformance_json: Option<String>,
    pub privacy_coverage_json: Option<String>,
    pub audit_coverage_json: Option<String>,
    pub sdk_coverage_json: Option<String>,
    pub evaluation_coverage_json: Option<String>,
    pub observability_coverage_json: Option<String>,
    pub migration_coverage_json: Option<String>,
    pub blocking_findings_json: Option<String>,
    pub warning_findings_json: Option<String>,
    pub created_at: String,
}

pub struct InsertCommercialReadinessCommand<'a> {
    pub id: i64,
    pub uuid: &'a str,
    pub tenant_id: i64,
    pub implementation_profile_id: Option<i64>,
    pub score: f64,
    pub state: &'a str,
    pub contract_coverage_json: Option<&'a str>,
    pub management_coverage_json: Option<&'a str>,
    pub runtime_conformance_json: Option<&'a str>,
    pub privacy_coverage_json: Option<&'a str>,
    pub audit_coverage_json: Option<&'a str>,
    pub sdk_coverage_json: Option<&'a str>,
    pub evaluation_coverage_json: Option<&'a str>,
    pub observability_coverage_json: Option<&'a str>,
    pub migration_coverage_json: Option<&'a str>,
    pub blocking_findings_json: Option<&'a str>,
    pub warning_findings_json: Option<&'a str>,
}

impl NativeSqlMemoryStore {
    pub async fn replace_commercial_readiness_snapshot(
        &self,
        cmd: InsertCommercialReadinessCommand<'_>,
    ) -> Result<(), NativeSqlStoreError> {
        let mut tx = self.begin_tx().await?;
        sqlx::query(
            r#"
            DELETE FROM ai_commercial_readiness_snapshot
            WHERE tenant_id = ?
              AND (
                (? IS NULL AND implementation_profile_id IS NULL)
                OR implementation_profile_id = ?
              )
            "#,
        )
        .bind(cmd.tenant_id)
        .bind(cmd.implementation_profile_id)
        .bind(cmd.implementation_profile_id)
        .execute(&mut *tx)
        .await?;
        let now = now_text();
        sqlx::query(
            r#"
            INSERT INTO ai_commercial_readiness_snapshot (
              id, uuid, tenant_id, implementation_profile_id, score, state,
              contract_coverage_json, management_coverage_json, runtime_conformance_json,
              privacy_coverage_json, audit_coverage_json, sdk_coverage_json,
              evaluation_coverage_json, observability_coverage_json, migration_coverage_json,
              blocking_findings_json, warning_findings_json, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(cmd.id)
        .bind(cmd.uuid)
        .bind(cmd.tenant_id)
        .bind(cmd.implementation_profile_id)
        .bind(cmd.score)
        .bind(cmd.state)
        .bind(cmd.contract_coverage_json)
        .bind(cmd.management_coverage_json)
        .bind(cmd.runtime_conformance_json)
        .bind(cmd.privacy_coverage_json)
        .bind(cmd.audit_coverage_json)
        .bind(cmd.sdk_coverage_json)
        .bind(cmd.evaluation_coverage_json)
        .bind(cmd.observability_coverage_json)
        .bind(cmd.migration_coverage_json)
        .bind(cmd.blocking_findings_json)
        .bind(cmd.warning_findings_json)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
        tx.commit().await.map_err(NativeSqlStoreError::from)?;
        Ok(())
    }

    pub async fn insert_commercial_readiness_snapshot(
        &self,
        cmd: InsertCommercialReadinessCommand<'_>,
    ) -> Result<(), NativeSqlStoreError> {
        let now = now_text();
        sqlx::query(
            r#"
            INSERT INTO ai_commercial_readiness_snapshot (
              id, uuid, tenant_id, implementation_profile_id, score, state,
              contract_coverage_json, management_coverage_json, runtime_conformance_json,
              privacy_coverage_json, audit_coverage_json, sdk_coverage_json,
              evaluation_coverage_json, observability_coverage_json, migration_coverage_json,
              blocking_findings_json, warning_findings_json, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(cmd.id)
        .bind(cmd.uuid)
        .bind(cmd.tenant_id)
        .bind(cmd.implementation_profile_id)
        .bind(cmd.score)
        .bind(cmd.state)
        .bind(cmd.contract_coverage_json)
        .bind(cmd.management_coverage_json)
        .bind(cmd.runtime_conformance_json)
        .bind(cmd.privacy_coverage_json)
        .bind(cmd.audit_coverage_json)
        .bind(cmd.sdk_coverage_json)
        .bind(cmd.evaluation_coverage_json)
        .bind(cmd.observability_coverage_json)
        .bind(cmd.migration_coverage_json)
        .bind(cmd.blocking_findings_json)
        .bind(cmd.warning_findings_json)
        .bind(&now)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn retrieve_latest_commercial_readiness(
        &self,
        tenant_id: i64,
    ) -> Result<Option<NativeSqlCommercialReadinessRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
              id, uuid, tenant_id, implementation_profile_id, score, state,
              contract_coverage_json, management_coverage_json, runtime_conformance_json,
              privacy_coverage_json, audit_coverage_json, sdk_coverage_json,
              evaluation_coverage_json, observability_coverage_json, migration_coverage_json,
              blocking_findings_json, warning_findings_json, created_at
            FROM ai_commercial_readiness_snapshot
            WHERE tenant_id = ?
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(map_readiness_row))
    }

    pub async fn delete_commercial_readiness_for_profile(
        &self,
        tenant_id: i64,
        implementation_profile_id: Option<i64>,
    ) -> Result<(), NativeSqlStoreError> {
        sqlx::query(
            r#"
            DELETE FROM ai_commercial_readiness_snapshot
            WHERE tenant_id = ?
              AND (
                (? IS NULL AND implementation_profile_id IS NULL)
                OR implementation_profile_id = ?
              )
            "#,
        )
        .bind(tenant_id)
        .bind(implementation_profile_id)
        .bind(implementation_profile_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn count_subjects_for_tenant(&self, tenant_id: i64) -> Result<i64, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM ai_subject
            WHERE tenant_id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .fetch_one(self.pool())
        .await?;
        Ok(row.get("total"))
    }

    pub async fn count_bindings_for_tenant(&self, tenant_id: i64) -> Result<i64, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM ai_memory_binding
            WHERE tenant_id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .fetch_one(self.pool())
        .await?;
        Ok(row.get("total"))
    }
}

fn map_readiness_row(row: sqlx::any::AnyRow) -> NativeSqlCommercialReadinessRow {
    NativeSqlCommercialReadinessRow {
        id: row.get("id"),
        uuid: row.get("uuid"),
        tenant_id: row.get("tenant_id"),
        implementation_profile_id: row.get("implementation_profile_id"),
        score: row.get("score"),
        state: row.get("state"),
        contract_coverage_json: row.get("contract_coverage_json"),
        management_coverage_json: row.get("management_coverage_json"),
        runtime_conformance_json: row.get("runtime_conformance_json"),
        privacy_coverage_json: row.get("privacy_coverage_json"),
        audit_coverage_json: row.get("audit_coverage_json"),
        sdk_coverage_json: row.get("sdk_coverage_json"),
        evaluation_coverage_json: row.get("evaluation_coverage_json"),
        observability_coverage_json: row.get("observability_coverage_json"),
        migration_coverage_json: row.get("migration_coverage_json"),
        blocking_findings_json: row.get("blocking_findings_json"),
        warning_findings_json: row.get("warning_findings_json"),
        created_at: row.get("created_at"),
    }
}
