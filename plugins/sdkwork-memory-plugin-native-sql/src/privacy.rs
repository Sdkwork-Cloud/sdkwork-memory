//! Privacy-oriented forget, export, and LIKE helpers for native SQL storage.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;

use sdkwork_memory_spi::MemoryScopeContext;

use crate::store::{now_text, NativeSqlMemoryStore, NativeSqlOpenApiEventRow, NativeSqlStoreError};

/// Escape `%`, `_`, and `\` for SQL `LIKE` patterns.
pub fn escape_like_pattern(query: &str) -> String {
    let mut escaped = String::with_capacity(query.len());
    for ch in query.chars() {
        match ch {
            '%' | '_' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            other => escaped.push(other),
        }
    }
    escaped
}

pub fn like_pattern(query: &str) -> String {
    format!("%{}%", escape_like_pattern(query.trim()))
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgetScopeStats {
    pub deleted_records: u32,
    pub purged_events: u32,
    pub rejected_candidates: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportCollectedPayload {
    pub records: Vec<Value>,
    pub events: Vec<Value>,
}

impl NativeSqlMemoryStore {
    pub async fn ping(&self) -> Result<(), NativeSqlStoreError> {
        sqlx::query("SELECT 1").execute(self.pool()).await?;
        Ok(())
    }

    pub async fn purge_derivatives_for_memory(
        &self,
        tenant_id: i64,
        memory_uuid: &str,
    ) -> Result<(u32, u32), NativeSqlStoreError> {
        let rejected = sqlx::query(
            r#"
            UPDATE ai_candidate
            SET decision_state = 'rejected',
                decision_reason = 'privacy_forget',
                decided_at = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ?
              AND target_memory_id = (
                SELECT id FROM ai_record WHERE tenant_id = ? AND uuid = ? LIMIT 1
              )
              AND decision_state = 'pending'
            "#,
        )
        .bind(now_text())
        .bind(now_text())
        .bind(tenant_id)
        .bind(tenant_id)
        .bind(memory_uuid)
        .execute(self.pool())
        .await?
        .rows_affected();

        let sources = sqlx::query(
            r#"
            DELETE FROM ai_record_source
            WHERE tenant_id = ?
              AND memory_id = (
                SELECT id FROM ai_record WHERE tenant_id = ? AND uuid = ? LIMIT 1
              )
            "#,
        )
        .bind(tenant_id)
        .bind(tenant_id)
        .bind(memory_uuid)
        .execute(self.pool())
        .await?
        .rows_affected();

        Ok((rejected as u32, sources as u32))
    }

    pub async fn forget_all_records_in_space(
        &self,
        scope: &MemoryScopeContext,
    ) -> Result<ForgetScopeStats, NativeSqlStoreError> {
        let mut stats = ForgetScopeStats::default();
        let mut cursor = String::new();

        loop {
            let rows = self
                .list_record_details(scope, None, sdkwork_utils_rust::MAX_LIST_PAGE_SIZE, Some(&cursor), crate::store::SENSITIVITY_READ_OWNER)
                .await?;
            if rows.is_empty() {
                break;
            }
            let page_limit = i32::try_from(sdkwork_utils_rust::MAX_LIST_PAGE_SIZE)
                .unwrap_or(200) as usize;
            let has_more = rows.len() > page_limit;
            let batch = if has_more {
                &rows[..page_limit]
            } else {
                &rows[..]
            };

            for row in batch {
                if self.hard_delete_record(scope, &row.memory_id).await? {
                    stats.deleted_records += 1;
                }
                let (rejected, _) = self
                    .purge_derivatives_for_memory(scope.tenant_id, &row.memory_id)
                    .await?;
                stats.rejected_candidates += rejected;
            }

            if !has_more {
                break;
            }
            cursor.clone_from(&batch.last().expect("batch non-empty").memory_id);
        }

        stats.purged_events += self
            .delete_events_in_space(scope.tenant_id, scope.space_id)
            .await?;

        Ok(stats)
    }

    pub async fn forget_records_for_user(
        &self,
        tenant_id: i64,
        user_id: i64,
        space_id: Option<i64>,
    ) -> Result<ForgetScopeStats, NativeSqlStoreError> {
        let mut stats = ForgetScopeStats::default();
        let now = now_text();

        let deleted = if let Some(space_id) = space_id {
            sqlx::query(
                r#"
                DELETE FROM ai_record
                WHERE tenant_id = ? AND space_id = ? AND user_id = ?
                "#,
            )
            .bind(tenant_id)
            .bind(space_id)
            .bind(user_id)
            .execute(self.pool())
            .await?
            .rows_affected()
        } else {
            sqlx::query(
                r#"
                DELETE FROM ai_record
                WHERE tenant_id = ? AND user_id = ?
                "#,
            )
            .bind(tenant_id)
            .bind(user_id)
            .execute(self.pool())
            .await?
            .rows_affected()
        };
        stats.deleted_records = deleted as u32;

        let rejected = if let Some(space_id) = space_id {
            sqlx::query(
                r#"
                UPDATE ai_candidate
                SET decision_state = 'rejected',
                    decision_reason = 'privacy_forget',
                    decided_at = ?,
                    updated_at = ?,
                    version = version + 1
                WHERE tenant_id = ? AND space_id = ? AND user_id = ? AND decision_state = 'pending'
                "#,
            )
            .bind(&now)
            .bind(&now)
            .bind(tenant_id)
            .bind(space_id)
            .bind(user_id)
            .execute(self.pool())
            .await?
            .rows_affected()
        } else {
            sqlx::query(
                r#"
                UPDATE ai_candidate
                SET decision_state = 'rejected',
                    decision_reason = 'privacy_forget',
                    decided_at = ?,
                    updated_at = ?,
                    version = version + 1
                WHERE tenant_id = ? AND user_id = ? AND decision_state = 'pending'
                "#,
            )
            .bind(&now)
            .bind(&now)
            .bind(tenant_id)
            .bind(user_id)
            .execute(self.pool())
            .await?
            .rows_affected()
        };
        stats.rejected_candidates = rejected as u32;

        stats.purged_events = if let Some(space_id) = space_id {
            self.delete_events_in_space(tenant_id, space_id).await?
        } else {
            self.delete_events_for_user_all_spaces(tenant_id, user_id)
                .await?
        };

        Ok(stats)
    }

    pub async fn forget_records_matching_query(
        &self,
        scope: &MemoryScopeContext,
        query: &str,
    ) -> Result<ForgetScopeStats, NativeSqlStoreError> {
        let pattern = like_pattern(query);
        let batch_size = i64::from(sdkwork_utils_rust::MAX_LIST_PAGE_SIZE);
        let mut stats = ForgetScopeStats::default();

        loop {
            let rows = sqlx::query(
                r#"
                SELECT uuid
                FROM ai_record
                WHERE tenant_id = ?
                  AND space_id = ?
                  AND status <> 'deleted'
                  AND (
                    canonical_text LIKE ? ESCAPE '\'
                    OR object_text LIKE ? ESCAPE '\'
                    OR COALESCE(subject, '') LIKE ? ESCAPE '\'
                  )
                LIMIT ?
                "#,
            )
            .bind(scope.tenant_id)
            .bind(scope.space_id)
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .bind(batch_size)
            .fetch_all(self.pool())
            .await?;

            if rows.is_empty() {
                break;
            }

            for row in rows {
                let memory_id: String = row.get("uuid");
                if self.hard_delete_record(scope, &memory_id).await? {
                    stats.deleted_records += 1;
                }
                let (rejected, _) = self
                    .purge_derivatives_for_memory(scope.tenant_id, &memory_id)
                    .await?;
                stats.rejected_candidates += rejected;
            }
        }

        Ok(stats)
    }

    pub async fn collect_export_payload_for_spaces(
        &self,
        tenant_id: i64,
        space_ids: &[i64],
        include_events: bool,
        sensitivity_scope: i32,
    ) -> Result<ExportCollectedPayload, NativeSqlStoreError> {
        let mut records = Vec::new();
        let mut events = Vec::new();

        let max_export_records = std::env::var("SDKWORK_MEMORY_EXPORT_MAX_RECORDS")
            .ok()
            .and_then(|value| sdkwork_utils_rust::parse_int(&value))
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(100_000);
        let max_export_events = std::env::var("SDKWORK_MEMORY_EXPORT_MAX_EVENTS")
            .ok()
            .and_then(|value| sdkwork_utils_rust::parse_int(&value))
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(100_000);

        for space_id in space_ids {
            let scope = MemoryScopeContext {
                tenant_id,
                space_id: *space_id,
                organization_id: None,
                user_id: None,
            };
            let mut cursor = String::new();
            loop {
                let rows = self
                    .list_record_details(&scope, None, sdkwork_utils_rust::MAX_LIST_PAGE_SIZE, Some(&cursor), sensitivity_scope)
                    .await?;
                if rows.is_empty() {
                    break;
                }
                let page_limit = i32::try_from(sdkwork_utils_rust::MAX_LIST_PAGE_SIZE)
                    .unwrap_or(200) as usize;
                let has_more = rows.len() > page_limit;
                let batch = if has_more {
                    &rows[..page_limit]
                } else {
                    &rows[..]
                };
                for row in batch {
                    if records.len() >= max_export_records {
                        return Err(NativeSqlStoreError::InvariantViolation {
                            message: format!(
                                "export record limit exceeded (max {max_export_records} records per job)"
                            ),
                        });
                    }
                    records.push(json!({
                        "memoryId": row.memory_id,
                        "spaceId": row.space_id,
                        "scope": row.scope,
                        "memoryType": row.memory_type,
                        "canonicalText": row.canonical_text,
                        "sensitivityLevel": row.sensitivity_level,
                        "createdAt": row.created_at,
                    }));
                }
                if !has_more {
                    break;
                }
                cursor.clone_from(&batch.last().expect("batch non-empty").memory_id);
            }

            if include_events {
                let mut event_cursor = String::new();
                loop {
                    let cursor = if event_cursor.is_empty() {
                        None
                    } else {
                        Some(event_cursor.as_str())
                    };
                    let event_rows = self
                        .list_open_api_events_for_tenant(
                            tenant_id,
                            Some(*space_id),
                            i32::try_from(sdkwork_utils_rust::MAX_LIST_PAGE_SIZE).unwrap_or(200),
                            cursor,
                        )
                        .await?;
                    if event_rows.is_empty() {
                        break;
                    }
                    let page_limit = i32::try_from(sdkwork_utils_rust::MAX_LIST_PAGE_SIZE)
                        .unwrap_or(200) as usize;
                    let has_more = event_rows.len() > page_limit;
                    let batch = if has_more {
                        &event_rows[..page_limit]
                    } else {
                        &event_rows[..]
                    };
                    for row in batch {
                        if events.len() >= max_export_events {
                            return Err(NativeSqlStoreError::InvariantViolation {
                                message: format!(
                                    "export event limit exceeded (max {max_export_events} events per job)"
                                ),
                            });
                        }
                        events.push(Self::map_export_event(row));
                    }
                    if !has_more {
                        break;
                    }
                    event_cursor.clone_from(&batch.last().expect("batch non-empty").event_id);
                }
            }
        }

        Ok(ExportCollectedPayload { records, events })
    }

    fn map_export_event(row: &NativeSqlOpenApiEventRow) -> Value {
        json!({
            "eventId": row.event_id,
            "spaceId": row.space_id,
            "eventType": row.event_type,
            "payload": row.payload,
            "createdAt": row.created_at,
        })
    }

    async fn delete_events_in_space(
        &self,
        tenant_id: i64,
        space_id: i64,
    ) -> Result<u32, NativeSqlStoreError> {
        let purged = sqlx::query(
            r#"
            DELETE FROM ai_event
            WHERE tenant_id = ? AND space_id = ?
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .execute(self.pool())
        .await?
        .rows_affected();
        Ok(purged as u32)
    }

    async fn delete_events_for_user_all_spaces(
        &self,
        tenant_id: i64,
        user_id: i64,
    ) -> Result<u32, NativeSqlStoreError> {
        let purged = sqlx::query(
            r#"
            DELETE FROM ai_event
            WHERE tenant_id = ? AND user_id = ?
            "#,
        )
        .bind(tenant_id)
        .bind(user_id)
        .execute(self.pool())
        .await?
        .rows_affected();
        Ok(purged as u32)
    }
}
