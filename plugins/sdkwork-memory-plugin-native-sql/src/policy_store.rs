//! Policy and policy assignment store methods for commercial memory management.

use sdkwork_utils_rust::MAX_LIST_PAGE_SIZE;
use sqlx::Row;

use crate::store::{now_text, NativeSqlMemoryStore, NativeSqlStoreError};

#[derive(Debug, Clone)]
pub struct NativeSqlPolicyRow {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: i64,
    pub policy_type: String,
    pub scope: String,
    pub scope_ref: Option<String>,
    pub status: String,
    pub policy_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone)]
pub struct NativeSqlPolicyAssignmentRow {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: i64,
    pub policy_id: i64,
    pub policy_uuid: String,
    pub target_type: String,
    pub target_id: i64,
    pub priority: i32,
    pub inheritance_mode: String,
    pub status: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

pub struct InsertPolicyCommand<'a> {
    pub id: i64,
    pub uuid: &'a str,
    pub tenant_id: i64,
    pub policy_type: &'a str,
    pub scope: &'a str,
    pub scope_ref: Option<&'a str>,
    pub policy_json: &'a str,
}

pub struct UpdatePolicyCommand<'a> {
    pub policy_type: Option<&'a str>,
    pub scope: Option<&'a str>,
    pub scope_ref: Option<&'a str>,
    pub policy_json: Option<&'a str>,
    pub status: Option<&'a str>,
}

pub struct InsertPolicyAssignmentCommand<'a> {
    pub id: i64,
    pub uuid: &'a str,
    pub tenant_id: i64,
    pub policy_id: i64,
    pub target_type: &'a str,
    pub target_id: i64,
    pub priority: i32,
    pub inheritance_mode: &'a str,
    pub valid_from: Option<&'a str>,
    pub valid_to: Option<&'a str>,
}

pub struct UpdatePolicyAssignmentCommand<'a> {
    pub priority: Option<i32>,
    pub inheritance_mode: Option<&'a str>,
    pub status: Option<&'a str>,
    pub valid_from: Option<&'a str>,
    pub valid_to: Option<&'a str>,
}

impl NativeSqlMemoryStore {
    pub async fn resolve_policy_internal_id(
        &self,
        tenant_id: i64,
        policy_uuid: &str,
    ) -> Result<i64, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT id FROM ai_policy
            WHERE tenant_id = ? AND uuid = ? AND status <> 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(policy_uuid)
        .fetch_optional(self.pool())
        .await?;
        row.map(|value| value.get("id"))
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: format!("policy {policy_uuid} not found"),
            })
    }

    pub async fn insert_policy(
        &self,
        cmd: InsertPolicyCommand<'_>,
    ) -> Result<(), NativeSqlStoreError> {
        let now = now_text();
        sqlx::query(
            r#"
            INSERT INTO ai_policy (
              id, uuid, tenant_id, policy_type, scope, scope_ref, status,
              policy_json, created_at, updated_at, version
            )
            VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?, ?, 1)
            "#,
        )
        .bind(cmd.id)
        .bind(cmd.uuid)
        .bind(cmd.tenant_id)
        .bind(cmd.policy_type)
        .bind(cmd.scope)
        .bind(cmd.scope_ref)
        .bind(cmd.policy_json)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn retrieve_policy(
        &self,
        tenant_id: i64,
        policy_uuid: &str,
    ) -> Result<Option<NativeSqlPolicyRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT id, uuid, tenant_id, policy_type, scope, scope_ref, status,
                   policy_json, created_at, updated_at, version
            FROM ai_policy
            WHERE tenant_id = ? AND uuid = ? AND status <> 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(policy_uuid)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(map_policy_row))
    }

    pub async fn list_policies(
        &self,
        tenant_id: i64,
        policy_type: Option<&str>,
        scope: Option<&str>,
        cursor: Option<&str>,
        page_size: i32,
    ) -> Result<Vec<NativeSqlPolicyRow>, NativeSqlStoreError> {
        let limit = page_size.clamp(1, MAX_LIST_PAGE_SIZE) + 1;
        let cursor = cursor.unwrap_or("");
        let rows = sqlx::query(
            r#"
            SELECT id, uuid, tenant_id, policy_type, scope, scope_ref, status,
                   policy_json, created_at, updated_at, version
            FROM ai_policy
            WHERE tenant_id = ?
              AND status <> 'deleted'
              AND (? IS NULL OR policy_type = ?)
              AND (? IS NULL OR scope = ?)
              AND uuid > ?
            ORDER BY uuid ASC
            LIMIT ?
            "#,
        )
        .bind(tenant_id)
        .bind(policy_type)
        .bind(policy_type)
        .bind(scope)
        .bind(scope)
        .bind(cursor)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;
        Ok(rows.into_iter().map(map_policy_row).collect())
    }

    pub async fn update_policy(
        &self,
        tenant_id: i64,
        policy_uuid: &str,
        cmd: UpdatePolicyCommand<'_>,
    ) -> Result<bool, NativeSqlStoreError> {
        let now = now_text();
        let result = sqlx::query(
            r#"
            UPDATE ai_policy
            SET policy_type = COALESCE(?, policy_type),
                scope = COALESCE(?, scope),
                scope_ref = COALESCE(?, scope_ref),
                policy_json = COALESCE(?, policy_json),
                status = COALESCE(?, status),
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND uuid = ? AND status <> 'deleted'
            "#,
        )
        .bind(cmd.policy_type)
        .bind(cmd.scope)
        .bind(cmd.scope_ref)
        .bind(cmd.policy_json)
        .bind(cmd.status)
        .bind(&now)
        .bind(tenant_id)
        .bind(policy_uuid)
        .execute(self.pool())
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_policy(
        &self,
        tenant_id: i64,
        policy_uuid: &str,
    ) -> Result<bool, NativeSqlStoreError> {
        let now = now_text();
        let result = sqlx::query(
            r#"
            UPDATE ai_policy
            SET status = 'deleted', updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND uuid = ? AND status <> 'deleted'
            "#,
        )
        .bind(&now)
        .bind(tenant_id)
        .bind(policy_uuid)
        .execute(self.pool())
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_policy_assignment(
        &self,
        cmd: InsertPolicyAssignmentCommand<'_>,
    ) -> Result<(), NativeSqlStoreError> {
        let now = now_text();
        sqlx::query(
            r#"
            INSERT INTO ai_policy_assignment (
              id, uuid, tenant_id, policy_id, target_type, target_id,
              priority, inheritance_mode, status, valid_from, valid_to,
              created_at, updated_at, version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active', ?, ?, ?, ?, 1)
            "#,
        )
        .bind(cmd.id)
        .bind(cmd.uuid)
        .bind(cmd.tenant_id)
        .bind(cmd.policy_id)
        .bind(cmd.target_type)
        .bind(cmd.target_id)
        .bind(cmd.priority)
        .bind(cmd.inheritance_mode)
        .bind(cmd.valid_from)
        .bind(cmd.valid_to)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn retrieve_policy_assignment(
        &self,
        tenant_id: i64,
        assignment_uuid: &str,
    ) -> Result<Option<NativeSqlPolicyAssignmentRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
              assignment.id,
              assignment.uuid,
              assignment.tenant_id,
              assignment.policy_id,
              policy.uuid AS policy_uuid,
              assignment.target_type,
              assignment.target_id,
              assignment.priority,
              assignment.inheritance_mode,
              assignment.status,
              assignment.valid_from,
              assignment.valid_to,
              assignment.created_at,
              assignment.updated_at,
              assignment.version
            FROM ai_policy_assignment assignment
            JOIN ai_policy policy
              ON policy.id = assignment.policy_id
             AND policy.tenant_id = assignment.tenant_id
            WHERE assignment.tenant_id = ?
              AND assignment.uuid = ?
              AND assignment.deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(assignment_uuid)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(map_policy_assignment_row))
    }

    pub async fn list_policy_assignments(
        &self,
        tenant_id: i64,
        target_type: Option<&str>,
        target_id: Option<i64>,
        policy_uuid: Option<&str>,
        cursor: Option<&str>,
        page_size: i32,
    ) -> Result<Vec<NativeSqlPolicyAssignmentRow>, NativeSqlStoreError> {
        let limit = page_size.clamp(1, MAX_LIST_PAGE_SIZE) + 1;
        let cursor = cursor.unwrap_or("");
        let rows = sqlx::query(
            r#"
            SELECT
              assignment.id,
              assignment.uuid,
              assignment.tenant_id,
              assignment.policy_id,
              policy.uuid AS policy_uuid,
              assignment.target_type,
              assignment.target_id,
              assignment.priority,
              assignment.inheritance_mode,
              assignment.status,
              assignment.valid_from,
              assignment.valid_to,
              assignment.created_at,
              assignment.updated_at,
              assignment.version
            FROM ai_policy_assignment assignment
            JOIN ai_policy policy
              ON policy.id = assignment.policy_id
             AND policy.tenant_id = assignment.tenant_id
            WHERE assignment.tenant_id = ?
              AND assignment.deleted_at IS NULL
              AND (? IS NULL OR assignment.target_type = ?)
              AND (? IS NULL OR assignment.target_id = ?)
              AND (? IS NULL OR policy.uuid = ?)
              AND assignment.uuid > ?
            ORDER BY assignment.uuid ASC
            LIMIT ?
            "#,
        )
        .bind(tenant_id)
        .bind(target_type)
        .bind(target_type)
        .bind(target_id)
        .bind(target_id)
        .bind(policy_uuid)
        .bind(policy_uuid)
        .bind(cursor)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;
        Ok(rows.into_iter().map(map_policy_assignment_row).collect())
    }

    pub async fn update_policy_assignment(
        &self,
        tenant_id: i64,
        assignment_uuid: &str,
        cmd: UpdatePolicyAssignmentCommand<'_>,
    ) -> Result<bool, NativeSqlStoreError> {
        let now = now_text();
        let result = sqlx::query(
            r#"
            UPDATE ai_policy_assignment
            SET priority = COALESCE(?, priority),
                inheritance_mode = COALESCE(?, inheritance_mode),
                status = COALESCE(?, status),
                valid_from = COALESCE(?, valid_from),
                valid_to = COALESCE(?, valid_to),
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND uuid = ? AND deleted_at IS NULL
            "#,
        )
        .bind(cmd.priority)
        .bind(cmd.inheritance_mode)
        .bind(cmd.status)
        .bind(cmd.valid_from)
        .bind(cmd.valid_to)
        .bind(&now)
        .bind(tenant_id)
        .bind(assignment_uuid)
        .execute(self.pool())
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_policy_assignment(
        &self,
        tenant_id: i64,
        assignment_uuid: &str,
    ) -> Result<bool, NativeSqlStoreError> {
        let now = now_text();
        let result = sqlx::query(
            r#"
            UPDATE ai_policy_assignment
            SET status = 'deleted', deleted_at = ?, updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND uuid = ? AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&now)
        .bind(tenant_id)
        .bind(assignment_uuid)
        .execute(self.pool())
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_policies_for_tenant(
        &self,
        tenant_id: i64,
    ) -> Result<i64, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM ai_policy
            WHERE tenant_id = ? AND status <> 'deleted'
            "#,
        )
        .bind(tenant_id)
        .fetch_one(self.pool())
        .await?;
        Ok(row.get("total"))
    }

    pub async fn count_policy_assignments_for_tenant(
        &self,
        tenant_id: i64,
    ) -> Result<i64, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM ai_policy_assignment
            WHERE tenant_id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .fetch_one(self.pool())
        .await?;
        Ok(row.get("total"))
    }
}

fn map_policy_row(row: sqlx::any::AnyRow) -> NativeSqlPolicyRow {
    NativeSqlPolicyRow {
        id: row.get("id"),
        uuid: row.get("uuid"),
        tenant_id: row.get("tenant_id"),
        policy_type: row.get("policy_type"),
        scope: row.get("scope"),
        scope_ref: row.get("scope_ref"),
        status: row.get("status"),
        policy_json: row.get("policy_json"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: row.get("version"),
    }
}

fn map_policy_assignment_row(row: sqlx::any::AnyRow) -> NativeSqlPolicyAssignmentRow {
    NativeSqlPolicyAssignmentRow {
        id: row.get("id"),
        uuid: row.get("uuid"),
        tenant_id: row.get("tenant_id"),
        policy_id: row.get("policy_id"),
        policy_uuid: row.get("policy_uuid"),
        target_type: row.get("target_type"),
        target_id: row.get("target_id"),
        priority: row.get("priority"),
        inheritance_mode: row.get("inheritance_mode"),
        status: row.get("status"),
        valid_from: row.get("valid_from"),
        valid_to: row.get("valid_to"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        version: row.get("version"),
    }
}
