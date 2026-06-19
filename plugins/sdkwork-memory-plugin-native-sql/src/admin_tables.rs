use sqlx::Row;

use crate::store::{now_text, NativeSqlMemoryStore, NativeSqlStoreError};

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSqlMemoryIndexRow {
    pub index_uuid: String,
    pub space_id: Option<i64>,
    pub index_kind: String,
    pub schema_version: String,
    pub status: String,
    pub config_json: Option<String>,
    pub last_rebuilt_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSqlRetrievalProfileRow {
    pub profile_uuid: String,
    pub space_id: Option<i64>,
    pub name: String,
    pub strategy: String,
    pub retrievers_json: String,
    pub top_k: i32,
    pub context_budget_tokens: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSqlImplementationProfileRow {
    pub profile_uuid: String,
    pub name: String,
    pub implementation_kind: String,
    pub role: String,
    pub status: String,
    pub capability_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSqlProviderBindingRow {
    pub binding_uuid: String,
    pub provider_kind: String,
    pub provider_code: String,
    pub display_name: String,
    pub health_state: String,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSqlEvalRunRow {
    pub eval_run_uuid: String,
    pub eval_type: String,
    pub state: String,
    pub metrics_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl NativeSqlMemoryStore {
    pub async fn ensure_default_keyword_index_for_tenant(
        &self,
        tenant_id: i64,
    ) -> Result<(), NativeSqlStoreError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mem_index WHERE tenant_id = ?")
            .bind(tenant_id)
            .fetch_one(self.pool())
            .await?;
        if count == 0 {
            self.insert_mem_index(
                tenant_id,
                "1",
                None,
                "keyword",
                "2026-06-10",
                "active",
                None,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn insert_mem_index(
        &self,
        tenant_id: i64,
        index_uuid: &str,
        space_id: Option<i64>,
        index_kind: &str,
        schema_version: &str,
        status: &str,
        config_json: Option<&str>,
    ) -> Result<(), NativeSqlStoreError> {
        sqlx::query(
            r#"
            INSERT INTO mem_index (
              uuid,
              tenant_id,
              space_id,
              index_kind,
              schema_version,
              status,
              config_json,
              created_at,
              updated_at,
              version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
            "#,
        )
        .bind(index_uuid)
        .bind(tenant_id)
        .bind(space_id)
        .bind(index_kind)
        .bind(schema_version)
        .bind(status)
        .bind(config_json)
        .bind(now_text())
        .bind(now_text())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn list_mem_indexes_for_tenant(
        &self,
        tenant_id: i64,
        page_size: i32,
    ) -> Result<Vec<NativeSqlMemoryIndexRow>, NativeSqlStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, space_id, index_kind, schema_version, status, config_json,
                   last_rebuilt_at, created_at, updated_at, version
            FROM mem_index
            WHERE tenant_id = ?
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(tenant_id)
        .bind(page_size.clamp(1, 100) as i64)
        .fetch_all(self.pool())
        .await?;

        Ok(rows.into_iter().map(map_index_row).collect())
    }

    pub async fn retrieve_mem_index_for_tenant(
        &self,
        tenant_id: i64,
        index_uuid: &str,
    ) -> Result<Option<NativeSqlMemoryIndexRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, space_id, index_kind, schema_version, status, config_json,
                   last_rebuilt_at, created_at, updated_at, version
            FROM mem_index
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(tenant_id)
        .bind(index_uuid)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(map_index_row))
    }

    pub async fn update_mem_index_for_tenant(
        &self,
        tenant_id: i64,
        index_uuid: &str,
        status: Option<&str>,
        config_json: Option<&str>,
        last_rebuilt_at: Option<&str>,
    ) -> Result<Option<NativeSqlMemoryIndexRow>, NativeSqlStoreError> {
        let existing = self
            .retrieve_mem_index_for_tenant(tenant_id, index_uuid)
            .await?
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: "memory index not found".to_string(),
            })?;

        sqlx::query(
            r#"
            UPDATE mem_index
            SET status = ?,
                config_json = ?,
                last_rebuilt_at = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(status.unwrap_or(&existing.status))
        .bind(config_json.or(existing.config_json.as_deref()))
        .bind(last_rebuilt_at.or(existing.last_rebuilt_at.as_deref()))
        .bind(now_text())
        .bind(tenant_id)
        .bind(index_uuid)
        .execute(self.pool())
        .await?;

        self.retrieve_mem_index_for_tenant(tenant_id, index_uuid)
            .await
    }

    pub async fn ensure_default_retrieval_profile_for_tenant(
        &self,
        tenant_id: i64,
    ) -> Result<(), NativeSqlStoreError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM mem_retrieval_profile WHERE tenant_id = ?")
                .bind(tenant_id)
                .fetch_one(self.pool())
                .await?;
        if count == 0 {
            self.insert_mem_retrieval_profile(
                tenant_id,
                "1",
                None,
                "keyword-default",
                "deterministic",
                r#"[{"name":"keyword","weight":1.0}]"#,
                10,
                2048,
                "active",
            )
            .await?;
        }
        Ok(())
    }

    pub async fn insert_mem_retrieval_profile(
        &self,
        tenant_id: i64,
        profile_uuid: &str,
        space_id: Option<i64>,
        name: &str,
        strategy: &str,
        retrievers_json: &str,
        top_k: i32,
        context_budget_tokens: i32,
        status: &str,
    ) -> Result<(), NativeSqlStoreError> {
        sqlx::query(
            r#"
            INSERT INTO mem_retrieval_profile (
              uuid, tenant_id, space_id, name, strategy, retrievers_json,
              top_k, context_budget_tokens, status, created_at, updated_at, version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
            "#,
        )
        .bind(profile_uuid)
        .bind(tenant_id)
        .bind(space_id)
        .bind(name)
        .bind(strategy)
        .bind(retrievers_json)
        .bind(top_k)
        .bind(context_budget_tokens)
        .bind(status)
        .bind(now_text())
        .bind(now_text())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn list_mem_retrieval_profiles_for_tenant(
        &self,
        tenant_id: i64,
        page_size: i32,
    ) -> Result<Vec<NativeSqlRetrievalProfileRow>, NativeSqlStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, space_id, name, strategy, retrievers_json, top_k,
                   context_budget_tokens, status, created_at, updated_at, version
            FROM mem_retrieval_profile
            WHERE tenant_id = ?
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(tenant_id)
        .bind(page_size.clamp(1, 100) as i64)
        .fetch_all(self.pool())
        .await?;

        Ok(rows.into_iter().map(map_retrieval_profile_row).collect())
    }

    pub async fn retrieve_mem_retrieval_profile_for_tenant(
        &self,
        tenant_id: i64,
        profile_uuid: &str,
    ) -> Result<Option<NativeSqlRetrievalProfileRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, space_id, name, strategy, retrievers_json, top_k,
                   context_budget_tokens, status, created_at, updated_at, version
            FROM mem_retrieval_profile
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(tenant_id)
        .bind(profile_uuid)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(map_retrieval_profile_row))
    }

    pub async fn update_mem_retrieval_profile_for_tenant(
        &self,
        tenant_id: i64,
        profile_uuid: &str,
        name: Option<&str>,
        strategy: Option<&str>,
        retrievers_json: Option<&str>,
        top_k: Option<i32>,
        context_budget_tokens: Option<i32>,
        status: Option<&str>,
    ) -> Result<Option<NativeSqlRetrievalProfileRow>, NativeSqlStoreError> {
        let existing = self
            .retrieve_mem_retrieval_profile_for_tenant(tenant_id, profile_uuid)
            .await?
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: "retrieval profile not found".to_string(),
            })?;

        sqlx::query(
            r#"
            UPDATE mem_retrieval_profile
            SET name = ?,
                strategy = ?,
                retrievers_json = ?,
                top_k = ?,
                context_budget_tokens = ?,
                status = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(name.unwrap_or(&existing.name))
        .bind(strategy.unwrap_or(&existing.strategy))
        .bind(retrievers_json.unwrap_or(&existing.retrievers_json))
        .bind(top_k.unwrap_or(existing.top_k))
        .bind(context_budget_tokens.unwrap_or(existing.context_budget_tokens))
        .bind(status.unwrap_or(&existing.status))
        .bind(now_text())
        .bind(tenant_id)
        .bind(profile_uuid)
        .execute(self.pool())
        .await?;

        self.retrieve_mem_retrieval_profile_for_tenant(tenant_id, profile_uuid)
            .await
    }

    pub async fn ensure_default_implementation_profile_for_tenant(
        &self,
        tenant_id: i64,
    ) -> Result<(), NativeSqlStoreError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM mem_implementation_profile WHERE tenant_id = ?",
        )
        .bind(tenant_id)
        .fetch_one(self.pool())
        .await?;
        if count == 0 {
            self.insert_mem_implementation_profile(
                tenant_id,
                "1",
                "native-sql-phase1",
                "native_sql",
                "primary",
                "active",
                r#"{"keyword":true,"embedding":false}"#,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn insert_mem_implementation_profile(
        &self,
        tenant_id: i64,
        profile_uuid: &str,
        name: &str,
        implementation_kind: &str,
        role: &str,
        status: &str,
        capability_json: &str,
    ) -> Result<(), NativeSqlStoreError> {
        sqlx::query(
            r#"
            INSERT INTO mem_implementation_profile (
              uuid, tenant_id, name, implementation_kind, role, status,
              capability_json, created_at, updated_at, version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
            "#,
        )
        .bind(profile_uuid)
        .bind(tenant_id)
        .bind(name)
        .bind(implementation_kind)
        .bind(role)
        .bind(status)
        .bind(capability_json)
        .bind(now_text())
        .bind(now_text())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn list_mem_implementation_profiles_for_tenant(
        &self,
        tenant_id: i64,
        page_size: i32,
    ) -> Result<Vec<NativeSqlImplementationProfileRow>, NativeSqlStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, name, implementation_kind, role, status, capability_json,
                   created_at, updated_at, version
            FROM mem_implementation_profile
            WHERE tenant_id = ?
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(tenant_id)
        .bind(page_size.clamp(1, 100) as i64)
        .fetch_all(self.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(map_implementation_profile_row)
            .collect())
    }

    pub async fn retrieve_mem_implementation_profile_for_tenant(
        &self,
        tenant_id: i64,
        profile_uuid: &str,
    ) -> Result<Option<NativeSqlImplementationProfileRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, name, implementation_kind, role, status, capability_json,
                   created_at, updated_at, version
            FROM mem_implementation_profile
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(tenant_id)
        .bind(profile_uuid)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(map_implementation_profile_row))
    }

    pub async fn update_mem_implementation_profile_for_tenant(
        &self,
        tenant_id: i64,
        profile_uuid: &str,
        name: Option<&str>,
        implementation_kind: Option<&str>,
        role: Option<&str>,
        status: Option<&str>,
        capability_json: Option<&str>,
    ) -> Result<Option<NativeSqlImplementationProfileRow>, NativeSqlStoreError> {
        let existing = self
            .retrieve_mem_implementation_profile_for_tenant(tenant_id, profile_uuid)
            .await?
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: "implementation profile not found".to_string(),
            })?;

        sqlx::query(
            r#"
            UPDATE mem_implementation_profile
            SET name = ?,
                implementation_kind = ?,
                role = ?,
                status = ?,
                capability_json = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(name.unwrap_or(&existing.name))
        .bind(implementation_kind.unwrap_or(&existing.implementation_kind))
        .bind(role.unwrap_or(&existing.role))
        .bind(status.unwrap_or(&existing.status))
        .bind(capability_json.unwrap_or(&existing.capability_json))
        .bind(now_text())
        .bind(tenant_id)
        .bind(profile_uuid)
        .execute(self.pool())
        .await?;

        self.retrieve_mem_implementation_profile_for_tenant(tenant_id, profile_uuid)
            .await
    }

    pub async fn insert_mem_provider_binding(
        &self,
        tenant_id: i64,
        binding_uuid: &str,
        provider_kind: &str,
        provider_code: &str,
        display_name: &str,
        health_state: &str,
    ) -> Result<(), NativeSqlStoreError> {
        sqlx::query(
            r#"
            INSERT INTO mem_provider_binding (
              uuid, tenant_id, provider_kind, provider_code, display_name,
              capabilities_json, health_state, created_at, updated_at, version
            )
            VALUES (?, ?, ?, ?, ?, '{}', ?, ?, ?, 1)
            "#,
        )
        .bind(binding_uuid)
        .bind(tenant_id)
        .bind(provider_kind)
        .bind(provider_code)
        .bind(display_name)
        .bind(health_state)
        .bind(now_text())
        .bind(now_text())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn list_mem_provider_bindings_for_tenant(
        &self,
        tenant_id: i64,
        page_size: i32,
    ) -> Result<Vec<NativeSqlProviderBindingRow>, NativeSqlStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, provider_kind, provider_code, display_name, health_state,
                   created_at, updated_at, version
            FROM mem_provider_binding
            WHERE tenant_id = ?
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(tenant_id)
        .bind(page_size.clamp(1, 100) as i64)
        .fetch_all(self.pool())
        .await?;

        Ok(rows.into_iter().map(map_provider_binding_row).collect())
    }

    pub async fn retrieve_mem_provider_binding_for_tenant(
        &self,
        tenant_id: i64,
        binding_uuid: &str,
    ) -> Result<Option<NativeSqlProviderBindingRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, provider_kind, provider_code, display_name, health_state,
                   created_at, updated_at, version
            FROM mem_provider_binding
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(tenant_id)
        .bind(binding_uuid)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(map_provider_binding_row))
    }

    pub async fn update_mem_provider_binding_for_tenant(
        &self,
        tenant_id: i64,
        binding_uuid: &str,
        display_name: Option<&str>,
        provider_code: Option<&str>,
        health_state: Option<&str>,
    ) -> Result<Option<NativeSqlProviderBindingRow>, NativeSqlStoreError> {
        let existing = self
            .retrieve_mem_provider_binding_for_tenant(tenant_id, binding_uuid)
            .await?
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: "provider binding not found".to_string(),
            })?;

        sqlx::query(
            r#"
            UPDATE mem_provider_binding
            SET display_name = ?,
                provider_code = ?,
                health_state = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(display_name.unwrap_or(&existing.display_name))
        .bind(provider_code.unwrap_or(&existing.provider_code))
        .bind(health_state.unwrap_or(&existing.health_state))
        .bind(now_text())
        .bind(tenant_id)
        .bind(binding_uuid)
        .execute(self.pool())
        .await?;

        self.retrieve_mem_provider_binding_for_tenant(tenant_id, binding_uuid)
            .await
    }

    pub async fn insert_mem_eval_run(
        &self,
        tenant_id: i64,
        eval_run_uuid: &str,
        eval_type: &str,
        state: &str,
        metrics_json: Option<&str>,
    ) -> Result<(), NativeSqlStoreError> {
        sqlx::query(
            r#"
            INSERT INTO mem_eval_run (
              uuid, tenant_id, eval_type, state, metrics_json, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(eval_run_uuid)
        .bind(tenant_id)
        .bind(eval_type)
        .bind(state)
        .bind(metrics_json)
        .bind(now_text())
        .bind(now_text())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn list_mem_eval_runs_for_tenant(
        &self,
        tenant_id: i64,
        page_size: i32,
    ) -> Result<Vec<NativeSqlEvalRunRow>, NativeSqlStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, eval_type, state, metrics_json, created_at, updated_at
            FROM mem_eval_run
            WHERE tenant_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(tenant_id)
        .bind(page_size.clamp(1, 100) as i64)
        .fetch_all(self.pool())
        .await?;

        Ok(rows.into_iter().map(map_eval_run_row).collect())
    }

    pub async fn retrieve_mem_eval_run_for_tenant(
        &self,
        tenant_id: i64,
        eval_run_uuid: &str,
    ) -> Result<Option<NativeSqlEvalRunRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, eval_type, state, metrics_json, created_at, updated_at
            FROM mem_eval_run
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(tenant_id)
        .bind(eval_run_uuid)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(map_eval_run_row))
    }
}

fn map_index_row(row: sqlx::sqlite::SqliteRow) -> NativeSqlMemoryIndexRow {
    NativeSqlMemoryIndexRow {
        index_uuid: row.get("uuid"),
        space_id: row.get("space_id"),
        index_kind: row.get("index_kind"),
        schema_version: row.get("schema_version"),
        status: row.get("status"),
        config_json: row.get("config_json"),
        last_rebuilt_at: row.get("last_rebuilt_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: row.get("version"),
    }
}

fn map_retrieval_profile_row(row: sqlx::sqlite::SqliteRow) -> NativeSqlRetrievalProfileRow {
    NativeSqlRetrievalProfileRow {
        profile_uuid: row.get("uuid"),
        space_id: row.get("space_id"),
        name: row.get("name"),
        strategy: row.get("strategy"),
        retrievers_json: row.get("retrievers_json"),
        top_k: row.get("top_k"),
        context_budget_tokens: row.get("context_budget_tokens"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: row.get("version"),
    }
}

fn map_implementation_profile_row(
    row: sqlx::sqlite::SqliteRow,
) -> NativeSqlImplementationProfileRow {
    NativeSqlImplementationProfileRow {
        profile_uuid: row.get("uuid"),
        name: row.get("name"),
        implementation_kind: row.get("implementation_kind"),
        role: row.get("role"),
        status: row.get("status"),
        capability_json: row.get("capability_json"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: row.get("version"),
    }
}

fn map_provider_binding_row(row: sqlx::sqlite::SqliteRow) -> NativeSqlProviderBindingRow {
    NativeSqlProviderBindingRow {
        binding_uuid: row.get("uuid"),
        provider_kind: row.get("provider_kind"),
        provider_code: row.get("provider_code"),
        display_name: row.get("display_name"),
        health_state: row.get("health_state"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: row.get("version"),
    }
}

fn map_eval_run_row(row: sqlx::sqlite::SqliteRow) -> NativeSqlEvalRunRow {
    NativeSqlEvalRunRow {
        eval_run_uuid: row.get("uuid"),
        eval_type: row.get("eval_type"),
        state: row.get("state"),
        metrics_json: row.get("metrics_json"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
