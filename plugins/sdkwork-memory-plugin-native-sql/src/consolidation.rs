//! Exact canonical duplicate consolidation with atomic evidence and journal handling.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::sqlx_compat as sqlx;
use sdkwork_memory_spi::{MemoryMutationJournal, MemoryScopeContext};
use serde_json::Value;
use sqlx::Row;

use crate::canonical_data::{append_journal_on_tx, remove_record_fts_on_tx, validate_journal};
use crate::store::{now_text, NativeSqlMemoryStore, NativeSqlStoreError};

const BATCH_LIMIT: i64 = 500;
const CONSOLIDATION_MODE: &str = "identity_bounded_supersession";
const SUPERSEDED_EVENT_TYPE: &str = "memory.record.superseded";
static COMPAT_OPERATION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryConsolidationResult {
    pub superseded_records: u32,
    pub transferred_sources: u32,
    pub deduplicated_sources: u32,
}

pub struct ConsolidateDuplicateRecordsCommand<'a> {
    pub scope: &'a MemoryScopeContext,
    /// Stable identity of one consolidation execution. Callers must reuse it on retry.
    pub operation_id: &'a str,
}

impl NativeSqlMemoryStore {
    /// Compatibility entrypoint. Production job paths should use the detailed operation API.
    pub async fn consolidate_duplicate_records_in_scope(
        &self,
        scope: &MemoryScopeContext,
    ) -> Result<u32, NativeSqlStoreError> {
        let sequence = COMPAT_OPERATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let operation_id = format!(
            "compat:{}:{}:{}:{sequence}",
            scope.tenant_id,
            scope.space_id,
            sdkwork_utils_rust::to_unix_millis(sdkwork_utils_rust::now())
        );
        Ok(self
            .consolidate_duplicate_records_in_scope_detailed(ConsolidateDuplicateRecordsCommand {
                scope,
                operation_id: &operation_id,
            })
            .await?
            .superseded_records)
    }

    pub async fn consolidate_duplicate_records_in_scope_detailed(
        &self,
        command: ConsolidateDuplicateRecordsCommand<'_>,
    ) -> Result<MemoryConsolidationResult, NativeSqlStoreError> {
        validate_operation_id(command.operation_id)?;
        let mut total = self
            .recover_consolidation_result(command.scope, command.operation_id)
            .await?;

        loop {
            let mut tx = self.begin_tx().await?;
            self.lock_space_on_tx(&mut tx, command.scope).await?;
            let rows = sqlx::query(
                r#"
                SELECT duplicate_id, duplicate_uuid, winner_id, winner_uuid
                FROM (
                  SELECT id AS duplicate_id,
                         uuid AS duplicate_uuid,
                         FIRST_VALUE(id) OVER (
                           PARTITION BY user_id, scope, memory_type,
                                        sensitivity_level, canonical_text
                           ORDER BY evidence_count DESC, confidence DESC,
                                    updated_at DESC, id DESC
                         ) AS winner_id,
                         FIRST_VALUE(uuid) OVER (
                           PARTITION BY user_id, scope, memory_type,
                                        sensitivity_level, canonical_text
                           ORDER BY evidence_count DESC, confidence DESC,
                                    updated_at DESC, id DESC
                         ) AS winner_uuid,
                         ROW_NUMBER() OVER (
                           PARTITION BY user_id, scope, memory_type,
                                        sensitivity_level, canonical_text
                           ORDER BY evidence_count DESC, confidence DESC,
                                    updated_at DESC, id DESC
                         ) AS row_num
                  FROM ai_record
                  WHERE tenant_id = ? AND space_id = ?
                    AND status NOT IN ('deleted', 'superseded')
                ) ranked
                WHERE row_num > 1
                LIMIT ?
                "#,
            )
            .bind(command.scope.tenant_id)
            .bind(command.scope.space_id)
            .bind(BATCH_LIMIT)
            .fetch_all(&mut *tx)
            .await?;

            if rows.is_empty() {
                tx.rollback().await?;
                break;
            }

            let selected_count = rows.len();
            let mut batch = MemoryConsolidationResult::default();
            for row in rows {
                let duplicate_id: i64 = row.get("duplicate_id");
                let duplicate_uuid: String = row.get("duplicate_uuid");
                let winner_id: i64 = row.get("winner_id");
                let winner_uuid: String = row.get("winner_uuid");
                let timestamp = now_text();

                let superseded = sqlx::query(
                    r#"
                    UPDATE ai_record
                    SET status = 'superseded',
                        superseded_by_memory_id = ?,
                        deleted_at = NULL,
                        updated_at = ?,
                        version = version + 1
                    WHERE tenant_id = ? AND space_id = ? AND id = ?
                      AND status NOT IN ('deleted', 'superseded')
                      AND id <> ?
                      AND EXISTS (
                        SELECT 1
                        FROM ai_record winner
                        WHERE winner.id = ?
                          AND winner.tenant_id = ai_record.tenant_id
                          AND winner.space_id = ai_record.space_id
                          AND winner.user_id IS NOT DISTINCT FROM ai_record.user_id
                          AND winner.scope = ai_record.scope
                          AND winner.memory_type = ai_record.memory_type
                          AND winner.sensitivity_level = ai_record.sensitivity_level
                          AND winner.canonical_text = ai_record.canonical_text
                          AND winner.status NOT IN ('deleted', 'superseded')
                      )
                    "#,
                )
                .bind(winner_id)
                .bind(&timestamp)
                .bind(command.scope.tenant_id)
                .bind(command.scope.space_id)
                .bind(duplicate_id)
                .bind(winner_id)
                .bind(winner_id)
                .execute(&mut *tx)
                .await?;
                if superseded.rows_affected() == 0 {
                    return Err(NativeSqlStoreError::InvariantViolation {
                        message: format!(
                            "duplicate memory {duplicate_uuid} changed during consolidation"
                        ),
                    });
                }

                let deduplicated = sqlx::query(
                    r#"
                    DELETE FROM ai_record_source
                    WHERE tenant_id = ? AND memory_id = ?
                      AND EXISTS (
                        SELECT 1
                        FROM ai_record_source winner_source
                        WHERE winner_source.tenant_id = ai_record_source.tenant_id
                          AND winner_source.memory_id = ?
                          AND winner_source.event_id = ai_record_source.event_id
                          AND winner_source.source_role = ai_record_source.source_role
                      )
                    "#,
                )
                .bind(command.scope.tenant_id)
                .bind(duplicate_id)
                .bind(winner_id)
                .execute(&mut *tx)
                .await?
                .rows_affected();

                let transferred = sqlx::query(
                    "UPDATE ai_record_source SET memory_id = ? WHERE tenant_id = ? AND memory_id = ?",
                )
                .bind(winner_id)
                .bind(command.scope.tenant_id)
                .bind(duplicate_id)
                .execute(&mut *tx)
                .await?
                .rows_affected();

                let winner_updated = sqlx::query(
                    r#"
                    UPDATE ai_record
                    SET evidence_count = (
                          SELECT COUNT(*)
                          FROM ai_record_source source
                          WHERE source.tenant_id = ai_record.tenant_id
                            AND source.memory_id = ai_record.id
                        ),
                        updated_at = ?,
                        version = version + 1
                    WHERE tenant_id = ? AND space_id = ? AND id = ?
                      AND status NOT IN ('deleted', 'superseded')
                    "#,
                )
                .bind(&timestamp)
                .bind(command.scope.tenant_id)
                .bind(command.scope.space_id)
                .bind(winner_id)
                .execute(&mut *tx)
                .await?;
                if winner_updated.rows_affected() != 1 {
                    return Err(NativeSqlStoreError::InvariantViolation {
                        message: format!("consolidation winner {winner_uuid} is no longer active"),
                    });
                }

                remove_record_fts_on_tx(self.dialect(), &mut tx, command.scope, &duplicate_uuid)
                    .await?;

                let transferred = count_to_u32(transferred, "transferred record sources")?;
                let deduplicated = count_to_u32(deduplicated, "deduplicated record sources")?;
                let journal = consolidation_journal(
                    command.scope,
                    command.operation_id,
                    &duplicate_uuid,
                    &winner_uuid,
                    transferred,
                    deduplicated,
                )?;
                validate_journal(&duplicate_uuid, &journal)?;
                append_journal_on_tx(self, &mut tx, command.scope, &journal).await?;

                batch.superseded_records = checked_add(batch.superseded_records, 1)?;
                batch.transferred_sources = checked_add(batch.transferred_sources, transferred)?;
                batch.deduplicated_sources = checked_add(batch.deduplicated_sources, deduplicated)?;
            }

            tx.commit().await?;
            if batch.superseded_records == 0 {
                return Err(NativeSqlStoreError::InvariantViolation {
                    message: "consolidation selected records but made no progress".to_string(),
                });
            }
            total.add(batch)?;
            if selected_count < BATCH_LIMIT as usize {
                break;
            }
        }

        Ok(total)
    }

    async fn recover_consolidation_result(
        &self,
        scope: &MemoryScopeContext,
        operation_id: &str,
    ) -> Result<MemoryConsolidationResult, NativeSqlStoreError> {
        let operation_hash = journal_prefix("outbox", scope, operation_id);
        let prefix = &operation_hash[..32];
        let rows = sqlx::query(
            r#"
            SELECT payload_json
            FROM ai_outbox_event
            WHERE tenant_id = ? AND event_type = ? AND uuid LIKE ?
            ORDER BY id ASC
            "#,
        )
        .bind(scope.tenant_id)
        .bind(SUPERSEDED_EVENT_TYPE)
        .bind(format!("{prefix}%"))
        .fetch_all(self.pool())
        .await?;

        let mut result = MemoryConsolidationResult::default();
        for row in rows {
            let payload_json: String = row.get("payload_json");
            let payload: Value = serde_json::from_str(&payload_json).map_err(|error| {
                NativeSqlStoreError::InvariantViolation {
                    message: format!("stored consolidation journal payload is invalid: {error}"),
                }
            })?;
            if payload.get("operationId").and_then(Value::as_str) != Some(operation_id)
                || json_i64(&payload, "spaceId") != Some(scope.space_id)
            {
                return Err(NativeSqlStoreError::IdempotencyConflict {
                    idempotency_key: operation_id.to_string(),
                });
            }
            result.superseded_records = checked_add(result.superseded_records, 1)?;
            result.transferred_sources = checked_add(
                result.transferred_sources,
                json_u32(&payload, "transferredSources")?,
            )?;
            result.deduplicated_sources = checked_add(
                result.deduplicated_sources,
                json_u32(&payload, "deduplicatedSources")?,
            )?;
        }
        Ok(result)
    }
}

impl MemoryConsolidationResult {
    fn add(&mut self, other: Self) -> Result<(), NativeSqlStoreError> {
        self.superseded_records = checked_add(self.superseded_records, other.superseded_records)?;
        self.transferred_sources =
            checked_add(self.transferred_sources, other.transferred_sources)?;
        self.deduplicated_sources =
            checked_add(self.deduplicated_sources, other.deduplicated_sources)?;
        Ok(())
    }
}

fn consolidation_journal(
    scope: &MemoryScopeContext,
    operation_id: &str,
    duplicate_uuid: &str,
    winner_uuid: &str,
    transferred_sources: u32,
    deduplicated_sources: u32,
) -> Result<MemoryMutationJournal, NativeSqlStoreError> {
    let payload_json = serde_json::to_string(&serde_json::json!({
        "operationId": operation_id,
        "tenantId": scope.tenant_id.to_string(),
        "spaceId": scope.space_id.to_string(),
        "memoryId": duplicate_uuid,
        "supersededByMemoryId": winner_uuid,
        "consolidationMode": CONSOLIDATION_MODE,
        "transferredSources": transferred_sources,
        "deduplicatedSources": deduplicated_sources,
    }))
    .map_err(|error| NativeSqlStoreError::InvariantViolation {
        message: format!("consolidation journal serialization failed: {error}"),
    })?;
    Ok(MemoryMutationJournal {
        outbox_id: journal_id("outbox", scope, operation_id, duplicate_uuid),
        aggregate_type: "memory_record".to_string(),
        aggregate_id: duplicate_uuid.to_string(),
        event_type: SUPERSEDED_EVENT_TYPE.to_string(),
        event_version: "1.0".to_string(),
        payload_json,
        audit_id: journal_id("audit", scope, operation_id, duplicate_uuid),
        audit_action: "memory.record.consolidate".to_string(),
        audit_resource_type: "memory_record".to_string(),
        audit_resource_id: duplicate_uuid.to_string(),
        audit_result: "succeeded".to_string(),
    })
}

fn journal_id(
    kind: &str,
    scope: &MemoryScopeContext,
    operation_id: &str,
    memory_uuid: &str,
) -> String {
    let prefix = journal_prefix(kind, scope, operation_id);
    let suffix = sdkwork_utils_rust::sha256_hash(memory_uuid.as_bytes());
    format!("{}{}", &prefix[..32], &suffix[..32])
}

fn journal_prefix(kind: &str, scope: &MemoryScopeContext, operation_id: &str) -> String {
    sdkwork_utils_rust::sha256_hash(
        format!(
            "memory-consolidation:{kind}:{}:{}:{operation_id}",
            scope.tenant_id, scope.space_id
        )
        .as_bytes(),
    )
}

fn validate_operation_id(operation_id: &str) -> Result<(), NativeSqlStoreError> {
    if operation_id.trim().is_empty() || operation_id.len() > 128 {
        return Err(NativeSqlStoreError::InvariantViolation {
            message: "consolidation operation_id must contain 1 to 128 characters".to_string(),
        });
    }
    Ok(())
}

fn json_i64(payload: &Value, field: &str) -> Option<i64> {
    payload
        .get(field)
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
}

fn json_u32(payload: &Value, field: &str) -> Result<u32, NativeSqlStoreError> {
    let value = payload
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
            message: format!("stored consolidation journal field {field} is invalid"),
        })?;
    Ok(value)
}

fn count_to_u32(count: u64, label: &str) -> Result<u32, NativeSqlStoreError> {
    u32::try_from(count).map_err(|_| NativeSqlStoreError::InvariantViolation {
        message: format!("{label} exceeded the supported u32 range"),
    })
}

fn checked_add(left: u32, right: u32) -> Result<u32, NativeSqlStoreError> {
    left.checked_add(right)
        .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
            message: "consolidation result exceeded the supported u32 range".to_string(),
        })
}
