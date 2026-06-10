use async_trait::async_trait;
use sdkwork_memory_spi::{
    AppendMemoryAuditCommand, AppendMemoryEventCommand, AppendMemoryOutboxCommand,
    AppendMemoryRetrievalTraceCommand, ApproveMemoryCandidateCommand, CreateMemoryCandidateCommand,
    CreateMemoryRecordCommand, DecayMemoryHabitCommand, DeleteMemoryRecordCommand,
    ListMemoryRetrievalTracesQuery, ListPendingMemoryOutboxQuery, MarkMemoryOutboxFailedCommand,
    MarkMemoryOutboxPublishedCommand, MemoryAuditRecord, MemoryAuditStorePort, MemoryCandidate,
    MemoryCandidateStorePort, MemoryContextPackSnapshot, MemoryDeletionReceipt, MemoryEvent,
    MemoryEventStorePort, MemoryHabit, MemoryHabitStorePort, MemoryOutboxEvent,
    MemoryOutboxStorePort, MemoryRecord, MemoryRecordStorePort, MemoryRetrievalHitDraft,
    MemoryRetrievalTrace, MemoryRetrievalTraceStorePort, MemoryScopeContext, MemorySpiError,
    MemorySpiResult, PromoteMemoryHabitCommand, RejectMemoryCandidateCommand,
    RetrieveMemoryAuditQuery, RetrieveMemoryCandidateQuery, RetrieveMemoryEventQuery,
    RetrieveMemoryHabitQuery, RetrieveMemoryOutboxQuery, RetrieveMemoryRecordQuery,
    RetrieveMemoryRetrievalTraceQuery, UpsertMemoryHabitCommand,
};
use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct NativeSqlMemoryStore {
    pool: SqlitePool,
}

impl NativeSqlMemoryStore {
    pub async fn new_in_memory_sqlite() -> Result<Self, NativeSqlStoreError> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        let store = Self { pool };
        store.apply_sqlite_phase1_migration().await?;
        Ok(store)
    }

    pub async fn append_event(
        &self,
        scope: &MemoryScopeContext,
        event_id: &str,
        content: &str,
    ) -> Result<(), NativeSqlStoreError> {
        self.ensure_space(scope).await?;
        let payload_json = serde_json::json!({ "content": content }).to_string();
        let payload_hash = stable_hash(content);

        if let Some(existing) = self
            .retrieve_event_idempotency_state(scope, event_id)
            .await?
        {
            if existing.space_id == scope.space_id
                && existing.payload_json == payload_json
                && existing.payload_hash == payload_hash
            {
                return Ok(());
            }

            return Err(NativeSqlStoreError::EventConflict {
                tenant_id: scope.tenant_id,
                event_id: event_id.to_string(),
            });
        }

        sqlx::query(
            r#"
            INSERT INTO mem_event (
              uuid,
              tenant_id,
              space_id,
              actor_type,
              event_type,
              source_type,
              event_time,
              payload_json,
              payload_hash,
              sensitivity_level,
              ingestion_status,
              created_at
            )
            VALUES (?, ?, ?, 'system', 'memory.event.appended', 'api', ?, ?, ?, 'internal', 'received', ?)
            "#,
        )
        .bind(event_id)
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(now_text())
        .bind(payload_json)
        .bind(payload_hash)
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn retrieve_event(
        &self,
        scope: &MemoryScopeContext,
        event_id: &str,
    ) -> Result<Option<NativeSqlMemoryEvent>, NativeSqlStoreError> {
        let row = sqlx::query(
            "SELECT uuid, payload_json FROM mem_event WHERE tenant_id = ? AND space_id = ? AND uuid = ?",
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let payload: String = row.get("payload_json");
            let payload = parse_event_payload(&payload).unwrap_or(Value::Null);
            NativeSqlMemoryEvent {
                event_id: row.get("uuid"),
                content: payload
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            }
        }))
    }

    pub async fn retrieve_event_payload(
        &self,
        scope: &MemoryScopeContext,
        event_id: &str,
    ) -> Result<Option<Value>, NativeSqlStoreError> {
        let row = sqlx::query(
            "SELECT payload_json FROM mem_event WHERE tenant_id = ? AND space_id = ? AND uuid = ?",
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .map(|row| {
                let payload: String = row.get("payload_json");
                parse_event_payload(&payload)
            })
            .transpose()?)
    }

    pub async fn create_record(
        &self,
        scope: &MemoryScopeContext,
        memory_id: &str,
        subject: &str,
        content: &str,
    ) -> Result<(), NativeSqlStoreError> {
        self.ensure_space(scope).await?;
        sqlx::query(
            r#"
            INSERT INTO mem_record (
              uuid,
              tenant_id,
              space_id,
              scope,
              memory_type,
              subject,
              predicate,
              object_text,
              canonical_text,
              confidence,
              evidence_count,
              contradiction_count,
              importance_score,
              recency_score,
              status,
              sensitivity_level,
              created_at,
              updated_at
            )
            VALUES (?, ?, ?, 'user', 'semantic', ?, 'is', ?, ?, 1.0, 1, 0, 0.5, 0.5, 'active', 'internal', ?, ?)
            "#,
        )
        .bind(memory_id)
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(subject)
        .bind(content)
        .bind(content)
        .bind(now_text())
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn retrieve_record(
        &self,
        scope: &MemoryScopeContext,
        memory_id: &str,
    ) -> Result<Option<NativeSqlMemoryRecord>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, object_text
            FROM mem_record
            WHERE tenant_id = ?
              AND space_id = ?
              AND uuid = ?
              AND status <> 'deleted'
            "#,
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(memory_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| NativeSqlMemoryRecord {
            memory_id: row.get("uuid"),
            content: row.get("object_text"),
        }))
    }

    pub async fn mark_record_deleted(
        &self,
        scope: &MemoryScopeContext,
        memory_id: &str,
    ) -> Result<MemoryDeletionReceipt, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT status, deleted_at
            FROM mem_record
            WHERE tenant_id = ? AND space_id = ? AND uuid = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(memory_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(MemoryDeletionReceipt {
                memory_id: memory_id.to_string(),
                deleted: false,
                already_deleted: false,
            });
        };

        let status: String = row.get("status");
        let deleted_at: Option<String> = row.get("deleted_at");

        if status == "deleted" || deleted_at.is_some() {
            return Ok(MemoryDeletionReceipt {
                memory_id: memory_id.to_string(),
                deleted: true,
                already_deleted: true,
            });
        }

        sqlx::query(
            r#"
            UPDATE mem_record
            SET status = 'deleted',
                deleted_at = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND space_id = ? AND uuid = ?
            "#,
        )
        .bind(now_text())
        .bind(now_text())
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(memory_id)
        .execute(&self.pool)
        .await?;

        Ok(MemoryDeletionReceipt {
            memory_id: memory_id.to_string(),
            deleted: true,
            already_deleted: false,
        })
    }

    pub async fn retrieve_record_lifecycle(
        &self,
        scope: &MemoryScopeContext,
        memory_id: &str,
    ) -> Result<Option<NativeSqlMemoryRecordLifecycle>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, status, deleted_at
            FROM mem_record
            WHERE tenant_id = ? AND space_id = ? AND uuid = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(memory_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| NativeSqlMemoryRecordLifecycle {
            memory_id: row.get("uuid"),
            status: row.get("status"),
            deleted_at: row.get("deleted_at"),
        }))
    }

    pub async fn append_audit(
        &self,
        scope: &MemoryScopeContext,
        audit_id: &str,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        result: &str,
    ) -> Result<NativeSqlMemoryAuditRecord, NativeSqlStoreError> {
        sqlx::query(
            r#"
            INSERT INTO mem_audit_log (
              uuid,
              tenant_id,
              actor_type,
              action,
              resource_type,
              resource_id,
              result,
              created_at
            )
            VALUES (?, ?, 'system', ?, ?, ?, ?, ?)
            "#,
        )
        .bind(audit_id)
        .bind(scope.tenant_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(result)
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        Ok(NativeSqlMemoryAuditRecord {
            audit_id: audit_id.to_string(),
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            result: result.to_string(),
        })
    }

    pub async fn retrieve_audit(
        &self,
        scope: &MemoryScopeContext,
        audit_id: &str,
    ) -> Result<Option<NativeSqlMemoryAuditRecord>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, action, resource_type, resource_id, result
            FROM mem_audit_log
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(audit_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| NativeSqlMemoryAuditRecord {
            audit_id: row.get("uuid"),
            action: row.get("action"),
            resource_type: row.get("resource_type"),
            resource_id: row.get("resource_id"),
            result: row.get("result"),
        }))
    }

    pub async fn append_outbox_event(
        &self,
        command: NativeSqlAppendOutboxEventCommand<'_>,
    ) -> Result<NativeSqlMemoryOutboxEvent, NativeSqlStoreError> {
        let _payload: Value = serde_json::from_str(command.payload_json)?;

        if let Some(existing) = self
            .retrieve_outbox_idempotency_state(command.scope, command.outbox_id)
            .await?
        {
            if existing.aggregate_type == command.aggregate_type
                && existing.aggregate_id == command.aggregate_id
                && existing.event_type == command.event_type
                && existing.event_version == command.event_version
                && existing.payload_json == command.payload_json
            {
                return Ok(existing.into_outbox_event(command.outbox_id));
            }

            return Err(NativeSqlStoreError::OutboxConflict {
                tenant_id: command.scope.tenant_id,
                outbox_id: command.outbox_id.to_string(),
            });
        }

        sqlx::query(
            r#"
            INSERT INTO mem_outbox_event (
              uuid,
              tenant_id,
              aggregate_type,
              aggregate_id,
              event_type,
              event_version,
              payload_json,
              publish_state,
              created_at,
              updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?)
            "#,
        )
        .bind(command.outbox_id)
        .bind(command.scope.tenant_id)
        .bind(command.aggregate_type)
        .bind(command.aggregate_id)
        .bind(command.event_type)
        .bind(command.event_version)
        .bind(command.payload_json)
        .bind(now_text())
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        Ok(NativeSqlMemoryOutboxEvent {
            outbox_id: command.outbox_id.to_string(),
            aggregate_type: command.aggregate_type.to_string(),
            aggregate_id: command.aggregate_id.to_string(),
            event_type: command.event_type.to_string(),
            event_version: command.event_version.to_string(),
            payload_json: command.payload_json.to_string(),
            publish_state: "pending".to_string(),
            published_at: None,
            retry_count: 0,
        })
    }

    pub async fn retrieve_outbox_event(
        &self,
        scope: &MemoryScopeContext,
        outbox_id: &str,
    ) -> Result<Option<NativeSqlMemoryOutboxEvent>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
              uuid,
              aggregate_type,
              aggregate_id,
              event_type,
              event_version,
              payload_json,
              publish_state,
              published_at,
              retry_count
            FROM mem_outbox_event
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(outbox_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| NativeSqlMemoryOutboxEvent {
            outbox_id: row.get("uuid"),
            aggregate_type: row.get("aggregate_type"),
            aggregate_id: row.get("aggregate_id"),
            event_type: row.get("event_type"),
            event_version: row.get("event_version"),
            payload_json: row.get("payload_json"),
            publish_state: row.get("publish_state"),
            published_at: row.get("published_at"),
            retry_count: row.get("retry_count"),
        }))
    }

    pub async fn list_pending_outbox_events(
        &self,
        scope: &MemoryScopeContext,
        limit: u32,
    ) -> Result<Vec<NativeSqlMemoryOutboxEvent>, NativeSqlStoreError> {
        let row_limit = i64::from(limit.max(1));
        let rows = sqlx::query(
            r#"
            SELECT
              uuid,
              aggregate_type,
              aggregate_id,
              event_type,
              event_version,
              payload_json,
              publish_state,
              published_at,
              retry_count
            FROM mem_outbox_event
            WHERE tenant_id = ? AND publish_state = 'pending'
            ORDER BY created_at ASC, id ASC
            LIMIT ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(row_limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| NativeSqlMemoryOutboxEvent {
                outbox_id: row.get("uuid"),
                aggregate_type: row.get("aggregate_type"),
                aggregate_id: row.get("aggregate_id"),
                event_type: row.get("event_type"),
                event_version: row.get("event_version"),
                payload_json: row.get("payload_json"),
                publish_state: row.get("publish_state"),
                published_at: row.get("published_at"),
                retry_count: row.get("retry_count"),
            })
            .collect())
    }

    pub async fn mark_outbox_published(
        &self,
        scope: &MemoryScopeContext,
        outbox_id: &str,
    ) -> Result<Option<NativeSqlMemoryOutboxEvent>, NativeSqlStoreError> {
        sqlx::query(
            r#"
            UPDATE mem_outbox_event
            SET publish_state = 'published',
                published_at = ?,
                updated_at = ?
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(now_text())
        .bind(now_text())
        .bind(scope.tenant_id)
        .bind(outbox_id)
        .execute(&self.pool)
        .await?;

        self.retrieve_outbox_event(scope, outbox_id).await
    }

    pub async fn mark_outbox_failed(
        &self,
        scope: &MemoryScopeContext,
        outbox_id: &str,
    ) -> Result<Option<NativeSqlMemoryOutboxEvent>, NativeSqlStoreError> {
        sqlx::query(
            r#"
            UPDATE mem_outbox_event
            SET publish_state = 'failed',
                retry_count = retry_count + 1,
                updated_at = ?
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(now_text())
        .bind(scope.tenant_id)
        .bind(outbox_id)
        .execute(&self.pool)
        .await?;

        self.retrieve_outbox_event(scope, outbox_id).await
    }

    pub async fn create_candidate(
        &self,
        command: &CreateMemoryCandidateCommand,
    ) -> Result<MemoryCandidate, NativeSqlStoreError> {
        self.ensure_space(&command.scope).await?;
        sqlx::query(
            r#"
            INSERT INTO mem_candidate (
              uuid,
              tenant_id,
              space_id,
              user_id,
              candidate_type,
              memory_type,
              proposed_text,
              proposed_payload_json,
              evidence_json,
              confidence,
              decision_state,
              created_at,
              updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?)
            "#,
        )
        .bind(&command.candidate_id)
        .bind(command.scope.tenant_id)
        .bind(command.scope.space_id)
        .bind(command.scope.user_id)
        .bind(&command.candidate_type)
        .bind(&command.memory_type)
        .bind(&command.proposed_text)
        .bind(&command.proposed_payload_json)
        .bind(&command.evidence_json)
        .bind(command.confidence)
        .bind(now_text())
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        Ok(MemoryCandidate {
            candidate_id: command.candidate_id.clone(),
            candidate_type: command.candidate_type.clone(),
            memory_type: command.memory_type.clone(),
            proposed_text: command.proposed_text.clone(),
            proposed_payload_json: command.proposed_payload_json.clone(),
            evidence_json: command.evidence_json.clone(),
            confidence: command.confidence,
            decision_state: "pending".to_string(),
            decision_reason: None,
            decided_by: None,
            decided_at: None,
        })
    }

    pub async fn retrieve_candidate(
        &self,
        scope: &MemoryScopeContext,
        candidate_id: &str,
    ) -> Result<Option<MemoryCandidate>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
              uuid,
              candidate_type,
              memory_type,
              proposed_text,
              proposed_payload_json,
              evidence_json,
              confidence,
              decision_state,
              decision_reason,
              decided_by,
              decided_at
            FROM mem_candidate
            WHERE tenant_id = ? AND space_id = ? AND uuid = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(candidate_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(candidate_from_row))
    }

    pub async fn approve_candidate(
        &self,
        command: &ApproveMemoryCandidateCommand,
    ) -> Result<Option<MemoryCandidate>, NativeSqlStoreError> {
        self.decide_candidate(
            &command.scope,
            &command.candidate_id,
            "approved",
            command.decision_reason.as_deref(),
            command.decided_by,
        )
        .await
    }

    pub async fn reject_candidate(
        &self,
        command: &RejectMemoryCandidateCommand,
    ) -> Result<Option<MemoryCandidate>, NativeSqlStoreError> {
        self.decide_candidate(
            &command.scope,
            &command.candidate_id,
            "rejected",
            command.decision_reason.as_deref(),
            command.decided_by,
        )
        .await
    }

    pub async fn upsert_habit(
        &self,
        command: &UpsertMemoryHabitCommand,
    ) -> Result<MemoryHabit, NativeSqlStoreError> {
        self.ensure_space(&command.scope).await?;
        sqlx::query(
            r#"
            INSERT INTO mem_habit (
              uuid,
              tenant_id,
              space_id,
              user_id,
              habit_key,
              habit_type,
              description,
              stage,
              strength,
              confidence,
              support_count,
              last_signal_at,
              metadata_json,
              created_at,
              updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (tenant_id, space_id, user_id, habit_key)
            DO UPDATE SET
              uuid = excluded.uuid,
              habit_type = excluded.habit_type,
              description = excluded.description,
              stage = excluded.stage,
              strength = excluded.strength,
              confidence = excluded.confidence,
              support_count = excluded.support_count,
              last_signal_at = excluded.last_signal_at,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at,
              version = mem_habit.version + 1
            "#,
        )
        .bind(&command.habit_id)
        .bind(command.scope.tenant_id)
        .bind(command.scope.space_id)
        .bind(command.user_id)
        .bind(&command.habit_key)
        .bind(&command.habit_type)
        .bind(&command.description)
        .bind(&command.stage)
        .bind(command.strength)
        .bind(command.confidence)
        .bind(command.support_count)
        .bind(now_text())
        .bind(&command.metadata_json)
        .bind(now_text())
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        self.retrieve_habit(&command.scope, command.user_id, &command.habit_key)
            .await?
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: "habit upsert did not return a stored row".to_string(),
            })
    }

    pub async fn retrieve_habit(
        &self,
        scope: &MemoryScopeContext,
        user_id: i64,
        habit_key: &str,
    ) -> Result<Option<MemoryHabit>, NativeSqlStoreError> {
        let row = self.fetch_habit(scope, user_id, habit_key).await?;
        Ok(row.map(habit_from_row))
    }

    pub async fn promote_habit(
        &self,
        command: &PromoteMemoryHabitCommand,
    ) -> Result<Option<MemoryHabit>, NativeSqlStoreError> {
        let promoted_memory_row_id = match command.promoted_memory_id.as_deref() {
            Some(memory_id) => self.lookup_record_row_id(&command.scope, memory_id).await?,
            None => None,
        };

        sqlx::query(
            r#"
            UPDATE mem_habit
            SET stage = 'promoted',
                promoted_memory_id = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND space_id = ? AND user_id = ? AND habit_key = ?
            "#,
        )
        .bind(promoted_memory_row_id)
        .bind(now_text())
        .bind(command.scope.tenant_id)
        .bind(command.scope.space_id)
        .bind(command.user_id)
        .bind(&command.habit_key)
        .execute(&self.pool)
        .await?;

        self.retrieve_habit(&command.scope, command.user_id, &command.habit_key)
            .await
    }

    pub async fn decay_habit(
        &self,
        command: &DecayMemoryHabitCommand,
    ) -> Result<Option<MemoryHabit>, NativeSqlStoreError> {
        sqlx::query(
            r#"
            UPDATE mem_habit
            SET stage = 'decayed',
                strength = CASE
                  WHEN strength - ? < 0 THEN 0
                  ELSE strength - ?
                END,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND space_id = ? AND user_id = ? AND habit_key = ?
            "#,
        )
        .bind(command.strength_delta)
        .bind(command.strength_delta)
        .bind(now_text())
        .bind(command.scope.tenant_id)
        .bind(command.scope.space_id)
        .bind(command.user_id)
        .bind(&command.habit_key)
        .execute(&self.pool)
        .await?;

        self.retrieve_habit(&command.scope, command.user_id, &command.habit_key)
            .await
    }

    pub async fn append_retrieval_trace(
        &self,
        command: &AppendMemoryRetrievalTraceCommand,
    ) -> Result<MemoryRetrievalTrace, NativeSqlStoreError> {
        self.ensure_space(&command.scope).await?;
        sqlx::query(
            r#"
            INSERT INTO mem_retrieval_trace (
              uuid,
              tenant_id,
              space_id,
              actor_id,
              query_text,
              query_hash,
              retrievers_json,
              latency_ms,
              result_count,
              degraded,
              metadata_json,
              created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&command.trace_id)
        .bind(command.scope.tenant_id)
        .bind(command.scope.space_id)
        .bind(&command.actor_id)
        .bind(&command.query_text)
        .bind(&command.query_hash)
        .bind(&command.retrievers_json)
        .bind(command.latency_ms)
        .bind(command.hits.len() as i64)
        .bind(bool_to_sqlite_int(command.degraded))
        .bind(&command.metadata_json)
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        let trace_row_id = self
            .lookup_retrieval_trace_row_id(&command.scope, &command.trace_id)
            .await?
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: "retrieval trace append did not return a stored row".to_string(),
            })?;

        for hit in &command.hits {
            let memory_row_id = match hit.memory_id.as_deref() {
                Some(memory_id) => self.lookup_record_row_id(&command.scope, memory_id).await?,
                None => None,
            };
            sqlx::query(
                r#"
                INSERT INTO mem_retrieval_hit (
                  uuid,
                  tenant_id,
                  retrieval_trace_id,
                  memory_id,
                  retriever_name,
                  result_rank,
                  raw_score,
                  fused_score,
                  explanation_json,
                  status,
                  created_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&hit.hit_id)
            .bind(command.scope.tenant_id)
            .bind(trace_row_id)
            .bind(memory_row_id)
            .bind(&hit.retriever_name)
            .bind(hit.result_rank)
            .bind(hit.raw_score)
            .bind(hit.fused_score)
            .bind(&hit.explanation_json)
            .bind(&hit.status)
            .bind(now_text())
            .execute(&self.pool)
            .await?;
        }

        if let Some(context_pack) = &command.context_pack {
            sqlx::query(
                r#"
                INSERT INTO mem_context_pack (
                  uuid,
                  tenant_id,
                  retrieval_trace_id,
                  actor_id,
                  query_text,
                  pack_json,
                  estimated_tokens,
                  truncated,
                  created_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&context_pack.context_pack_id)
            .bind(command.scope.tenant_id)
            .bind(trace_row_id)
            .bind(&command.actor_id)
            .bind(&command.query_text)
            .bind(&context_pack.pack_json)
            .bind(context_pack.estimated_tokens)
            .bind(bool_to_sqlite_int(context_pack.truncated))
            .bind(now_text())
            .execute(&self.pool)
            .await?;
        }

        self.retrieve_retrieval_trace(&command.scope, &command.trace_id)
            .await?
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: "retrieval trace append could not retrieve stored row".to_string(),
            })
    }

    pub async fn retrieve_retrieval_trace(
        &self,
        scope: &MemoryScopeContext,
        trace_id: &str,
    ) -> Result<Option<MemoryRetrievalTrace>, NativeSqlStoreError> {
        let row = sqlx::query(retrieval_trace_select_sql())
            .bind(scope.tenant_id)
            .bind(scope.space_id)
            .bind(trace_id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        self.retrieval_trace_from_row(scope, row).await.map(Some)
    }

    pub async fn list_recent_retrieval_traces(
        &self,
        scope: &MemoryScopeContext,
        limit: u32,
    ) -> Result<Vec<MemoryRetrievalTrace>, NativeSqlStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
              id,
              uuid,
              actor_id,
              query_text,
              query_hash,
              retrievers_json,
              latency_ms,
              result_count,
              degraded,
              metadata_json
            FROM mem_retrieval_trace
            WHERE tenant_id = ? AND space_id = ?
            ORDER BY created_at DESC, id DESC
            LIMIT ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(i64::from(limit.max(1)))
        .fetch_all(&self.pool)
        .await?;

        let mut traces = Vec::with_capacity(rows.len());
        for row in rows {
            traces.push(self.retrieval_trace_from_row(scope, row).await?);
        }

        Ok(traces)
    }

    async fn apply_sqlite_phase1_migration(&self) -> Result<(), NativeSqlStoreError> {
        let migration = include_str!("../migrations/sqlite/V202606100001__memory_phase1.sql");
        for statement in migration.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                sqlx::query(statement).execute(&self.pool).await?;
            }
        }
        Ok(())
    }

    async fn retrieve_event_idempotency_state(
        &self,
        scope: &MemoryScopeContext,
        event_id: &str,
    ) -> Result<Option<NativeSqlEventIdempotencyState>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT space_id, payload_json, payload_hash
            FROM mem_event
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| NativeSqlEventIdempotencyState {
            space_id: row.get("space_id"),
            payload_json: row.get("payload_json"),
            payload_hash: row.get("payload_hash"),
        }))
    }

    async fn retrieve_outbox_idempotency_state(
        &self,
        scope: &MemoryScopeContext,
        outbox_id: &str,
    ) -> Result<Option<NativeSqlOutboxIdempotencyState>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
              aggregate_type,
              aggregate_id,
              event_type,
              event_version,
              payload_json,
              publish_state,
              published_at,
              retry_count
            FROM mem_outbox_event
            WHERE tenant_id = ? AND uuid = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(outbox_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| NativeSqlOutboxIdempotencyState {
            aggregate_type: row.get("aggregate_type"),
            aggregate_id: row.get("aggregate_id"),
            event_type: row.get("event_type"),
            event_version: row.get("event_version"),
            payload_json: row.get("payload_json"),
            publish_state: row.get("publish_state"),
            published_at: row.get("published_at"),
            retry_count: row.get("retry_count"),
        }))
    }

    async fn decide_candidate(
        &self,
        scope: &MemoryScopeContext,
        candidate_id: &str,
        decision_state: &str,
        decision_reason: Option<&str>,
        decided_by: Option<i64>,
    ) -> Result<Option<MemoryCandidate>, NativeSqlStoreError> {
        sqlx::query(
            r#"
            UPDATE mem_candidate
            SET decision_state = ?,
                decision_reason = ?,
                decided_by = ?,
                decided_at = ?,
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND space_id = ? AND uuid = ?
            "#,
        )
        .bind(decision_state)
        .bind(decision_reason)
        .bind(decided_by)
        .bind(now_text())
        .bind(now_text())
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(candidate_id)
        .execute(&self.pool)
        .await?;

        self.retrieve_candidate(scope, candidate_id).await
    }

    async fn fetch_habit(
        &self,
        scope: &MemoryScopeContext,
        user_id: i64,
        habit_key: &str,
    ) -> Result<Option<SqliteRow>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
              habit.uuid,
              habit.user_id,
              habit.habit_key,
              habit.habit_type,
              habit.description,
              habit.stage,
              habit.strength,
              habit.confidence,
              habit.support_count,
              habit.last_signal_at,
              promoted.uuid AS promoted_memory_uuid,
              habit.decay_after,
              habit.metadata_json
            FROM mem_habit habit
            LEFT JOIN mem_record promoted
              ON promoted.id = habit.promoted_memory_id
             AND promoted.tenant_id = habit.tenant_id
             AND promoted.space_id = habit.space_id
            WHERE habit.tenant_id = ?
              AND habit.space_id = ?
              AND habit.user_id = ?
              AND habit.habit_key = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(user_id)
        .bind(habit_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn lookup_record_row_id(
        &self,
        scope: &MemoryScopeContext,
        memory_id: &str,
    ) -> Result<Option<i64>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT id
            FROM mem_record
            WHERE tenant_id = ?
              AND space_id = ?
              AND uuid = ?
              AND status <> 'deleted'
            "#,
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(memory_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| row.get("id")))
    }

    async fn lookup_retrieval_trace_row_id(
        &self,
        scope: &MemoryScopeContext,
        trace_id: &str,
    ) -> Result<Option<i64>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT id
            FROM mem_retrieval_trace
            WHERE tenant_id = ? AND space_id = ? AND uuid = ?
            "#,
        )
        .bind(scope.tenant_id)
        .bind(scope.space_id)
        .bind(trace_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| row.get("id")))
    }

    async fn retrieval_trace_from_row(
        &self,
        scope: &MemoryScopeContext,
        row: SqliteRow,
    ) -> Result<MemoryRetrievalTrace, NativeSqlStoreError> {
        let trace_row_id: i64 = row.get("id");
        let hits = self.fetch_retrieval_hits(scope, trace_row_id).await?;
        let context_pack = self.fetch_context_pack(scope, trace_row_id).await?;

        Ok(MemoryRetrievalTrace {
            trace_id: row.get("uuid"),
            actor_id: row.get("actor_id"),
            query_text: row.get("query_text"),
            query_hash: row.get("query_hash"),
            retrievers_json: row.get("retrievers_json"),
            latency_ms: row.get("latency_ms"),
            result_count: row.get("result_count"),
            degraded: sqlite_int_to_bool(row.get("degraded")),
            metadata_json: row.get("metadata_json"),
            hits,
            context_pack,
        })
    }

    async fn fetch_retrieval_hits(
        &self,
        scope: &MemoryScopeContext,
        trace_row_id: i64,
    ) -> Result<Vec<MemoryRetrievalHitDraft>, NativeSqlStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
              hit.uuid,
              record.uuid AS memory_uuid,
              hit.retriever_name,
              hit.result_rank,
              hit.raw_score,
              hit.fused_score,
              hit.explanation_json,
              hit.status
            FROM mem_retrieval_hit hit
            LEFT JOIN mem_record record
              ON record.id = hit.memory_id
             AND record.tenant_id = hit.tenant_id
             AND record.space_id = ?
            WHERE hit.tenant_id = ?
              AND hit.retrieval_trace_id = ?
            ORDER BY hit.result_rank ASC, hit.id ASC
            "#,
        )
        .bind(scope.space_id)
        .bind(scope.tenant_id)
        .bind(trace_row_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| MemoryRetrievalHitDraft {
                hit_id: row.get("uuid"),
                memory_id: row.get("memory_uuid"),
                retriever_name: row.get("retriever_name"),
                result_rank: row.get("result_rank"),
                raw_score: row.get("raw_score"),
                fused_score: row.get("fused_score"),
                explanation_json: row.get("explanation_json"),
                status: row.get("status"),
            })
            .collect())
    }

    async fn fetch_context_pack(
        &self,
        scope: &MemoryScopeContext,
        trace_row_id: i64,
    ) -> Result<Option<MemoryContextPackSnapshot>, NativeSqlStoreError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, pack_json, estimated_tokens, truncated
            FROM mem_context_pack
            WHERE tenant_id = ? AND retrieval_trace_id = ?
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
        .bind(scope.tenant_id)
        .bind(trace_row_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| MemoryContextPackSnapshot {
            context_pack_id: row.get("uuid"),
            pack_json: row.get("pack_json"),
            estimated_tokens: row.get("estimated_tokens"),
            truncated: sqlite_int_to_bool(row.get("truncated")),
        }))
    }

    async fn ensure_space(&self, scope: &MemoryScopeContext) -> Result<(), NativeSqlStoreError> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO mem_space (
              id,
              uuid,
              tenant_id,
              owner_subject_type,
              owner_subject_id,
              space_type,
              display_name,
              default_scope,
              lifecycle_status,
              created_at,
              updated_at,
              version
            )
            VALUES (?, ?, ?, 'user', ?, 'personal', 'Default Memory Space', 'user', 'active', ?, ?, 0)
            "#,
        )
        .bind(scope.space_id)
        .bind(format!("space-{}", scope.space_id))
        .bind(scope.tenant_id)
        .bind(format!("tenant-{}-space-{}", scope.tenant_id, scope.space_id))
        .bind(now_text())
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl MemoryEventStorePort for NativeSqlMemoryStore {
    async fn append(&self, command: AppendMemoryEventCommand) -> MemorySpiResult<MemoryEvent> {
        self.append_event(&command.scope, &command.event_id, &command.content)
            .await
            .map_err(|err| port_error("MemoryEventStorePort", err))?;

        Ok(MemoryEvent {
            event_id: command.event_id,
            content: command.content,
        })
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryEventQuery,
    ) -> MemorySpiResult<Option<MemoryEvent>> {
        let event = self
            .retrieve_event(&query.scope, &query.event_id)
            .await
            .map_err(|err| port_error("MemoryEventStorePort", err))?;

        Ok(event.map(|event| MemoryEvent {
            event_id: event.event_id,
            content: event.content,
        }))
    }
}

#[async_trait]
impl MemoryRecordStorePort for NativeSqlMemoryStore {
    async fn create(&self, command: CreateMemoryRecordCommand) -> MemorySpiResult<MemoryRecord> {
        self.create_record(&command.scope, &command.memory_id, "spi", &command.content)
            .await
            .map_err(|err| port_error("MemoryRecordStorePort", err))?;

        Ok(MemoryRecord {
            memory_id: command.memory_id,
            content: command.content,
        })
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryRecordQuery,
    ) -> MemorySpiResult<Option<MemoryRecord>> {
        let record = self
            .retrieve_record(&query.scope, &query.memory_id)
            .await
            .map_err(|err| port_error("MemoryRecordStorePort", err))?;

        Ok(record.map(|record| MemoryRecord {
            memory_id: record.memory_id,
            content: record.content,
        }))
    }

    async fn mark_deleted(
        &self,
        command: DeleteMemoryRecordCommand,
    ) -> MemorySpiResult<MemoryDeletionReceipt> {
        self.mark_record_deleted(&command.scope, &command.memory_id)
            .await
            .map_err(|err| port_error("MemoryRecordStorePort", err))
    }
}

#[async_trait]
impl MemoryAuditStorePort for NativeSqlMemoryStore {
    async fn append(
        &self,
        command: AppendMemoryAuditCommand,
    ) -> MemorySpiResult<MemoryAuditRecord> {
        let audit = self
            .append_audit(
                &command.scope,
                &command.audit_id,
                &command.action,
                &command.resource_type,
                &command.resource_id,
                &command.result,
            )
            .await
            .map_err(|err| port_error("MemoryAuditStorePort", err))?;

        Ok(MemoryAuditRecord {
            audit_id: audit.audit_id,
            action: audit.action,
            resource_type: audit.resource_type,
            resource_id: audit.resource_id,
            result: audit.result,
        })
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryAuditQuery,
    ) -> MemorySpiResult<Option<MemoryAuditRecord>> {
        let audit = self
            .retrieve_audit(&query.scope, &query.audit_id)
            .await
            .map_err(|err| port_error("MemoryAuditStorePort", err))?;

        Ok(audit.map(|audit| MemoryAuditRecord {
            audit_id: audit.audit_id,
            action: audit.action,
            resource_type: audit.resource_type,
            resource_id: audit.resource_id,
            result: audit.result,
        }))
    }
}

#[async_trait]
impl MemoryOutboxStorePort for NativeSqlMemoryStore {
    async fn append(
        &self,
        command: AppendMemoryOutboxCommand,
    ) -> MemorySpiResult<MemoryOutboxEvent> {
        let outbox = self
            .append_outbox_event(NativeSqlAppendOutboxEventCommand {
                scope: &command.scope,
                outbox_id: &command.outbox_id,
                aggregate_type: &command.aggregate_type,
                aggregate_id: &command.aggregate_id,
                event_type: &command.event_type,
                event_version: &command.event_version,
                payload_json: &command.payload_json,
            })
            .await
            .map_err(|err| port_error("MemoryOutboxStorePort", err))?;

        Ok(MemoryOutboxEvent {
            outbox_id: outbox.outbox_id,
            aggregate_type: outbox.aggregate_type,
            aggregate_id: outbox.aggregate_id,
            event_type: outbox.event_type,
            event_version: outbox.event_version,
            payload_json: outbox.payload_json,
            publish_state: outbox.publish_state,
            published_at: outbox.published_at,
            retry_count: outbox.retry_count,
        })
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryOutboxQuery,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>> {
        let outbox = self
            .retrieve_outbox_event(&query.scope, &query.outbox_id)
            .await
            .map_err(|err| port_error("MemoryOutboxStorePort", err))?;

        Ok(outbox.map(into_spi_outbox_event))
    }

    async fn list_pending(
        &self,
        query: ListPendingMemoryOutboxQuery,
    ) -> MemorySpiResult<Vec<MemoryOutboxEvent>> {
        let outbox_events = self
            .list_pending_outbox_events(&query.scope, query.limit)
            .await
            .map_err(|err| port_error("MemoryOutboxStorePort", err))?;

        Ok(outbox_events
            .into_iter()
            .map(into_spi_outbox_event)
            .collect())
    }

    async fn mark_published(
        &self,
        command: MarkMemoryOutboxPublishedCommand,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>> {
        let outbox = self
            .mark_outbox_published(&command.scope, &command.outbox_id)
            .await
            .map_err(|err| port_error("MemoryOutboxStorePort", err))?;

        Ok(outbox.map(into_spi_outbox_event))
    }

    async fn mark_failed(
        &self,
        command: MarkMemoryOutboxFailedCommand,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>> {
        let outbox = self
            .mark_outbox_failed(&command.scope, &command.outbox_id)
            .await
            .map_err(|err| port_error("MemoryOutboxStorePort", err))?;

        Ok(outbox.map(into_spi_outbox_event))
    }
}

#[async_trait]
impl MemoryCandidateStorePort for NativeSqlMemoryStore {
    async fn create(
        &self,
        command: CreateMemoryCandidateCommand,
    ) -> MemorySpiResult<MemoryCandidate> {
        self.create_candidate(&command)
            .await
            .map_err(|err| port_error("MemoryCandidateStorePort", err))
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryCandidateQuery,
    ) -> MemorySpiResult<Option<MemoryCandidate>> {
        self.retrieve_candidate(&query.scope, &query.candidate_id)
            .await
            .map_err(|err| port_error("MemoryCandidateStorePort", err))
    }

    async fn approve(
        &self,
        command: ApproveMemoryCandidateCommand,
    ) -> MemorySpiResult<Option<MemoryCandidate>> {
        self.approve_candidate(&command)
            .await
            .map_err(|err| port_error("MemoryCandidateStorePort", err))
    }

    async fn reject(
        &self,
        command: RejectMemoryCandidateCommand,
    ) -> MemorySpiResult<Option<MemoryCandidate>> {
        self.reject_candidate(&command)
            .await
            .map_err(|err| port_error("MemoryCandidateStorePort", err))
    }
}

#[async_trait]
impl MemoryHabitStorePort for NativeSqlMemoryStore {
    async fn upsert(&self, command: UpsertMemoryHabitCommand) -> MemorySpiResult<MemoryHabit> {
        self.upsert_habit(&command)
            .await
            .map_err(|err| port_error("MemoryHabitStorePort", err))
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryHabitQuery,
    ) -> MemorySpiResult<Option<MemoryHabit>> {
        self.retrieve_habit(&query.scope, query.user_id, &query.habit_key)
            .await
            .map_err(|err| port_error("MemoryHabitStorePort", err))
    }

    async fn promote(
        &self,
        command: PromoteMemoryHabitCommand,
    ) -> MemorySpiResult<Option<MemoryHabit>> {
        self.promote_habit(&command)
            .await
            .map_err(|err| port_error("MemoryHabitStorePort", err))
    }

    async fn decay(
        &self,
        command: DecayMemoryHabitCommand,
    ) -> MemorySpiResult<Option<MemoryHabit>> {
        self.decay_habit(&command)
            .await
            .map_err(|err| port_error("MemoryHabitStorePort", err))
    }
}

#[async_trait]
impl MemoryRetrievalTraceStorePort for NativeSqlMemoryStore {
    async fn append(
        &self,
        command: AppendMemoryRetrievalTraceCommand,
    ) -> MemorySpiResult<MemoryRetrievalTrace> {
        self.append_retrieval_trace(&command)
            .await
            .map_err(|err| port_error("MemoryRetrievalTraceStorePort", err))
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryRetrievalTraceQuery,
    ) -> MemorySpiResult<Option<MemoryRetrievalTrace>> {
        self.retrieve_retrieval_trace(&query.scope, &query.trace_id)
            .await
            .map_err(|err| port_error("MemoryRetrievalTraceStorePort", err))
    }

    async fn list_recent(
        &self,
        query: ListMemoryRetrievalTracesQuery,
    ) -> MemorySpiResult<Vec<MemoryRetrievalTrace>> {
        self.list_recent_retrieval_traces(&query.scope, query.limit)
            .await
            .map_err(|err| port_error("MemoryRetrievalTraceStorePort", err))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSqlMemoryEvent {
    pub event_id: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSqlMemoryRecord {
    pub memory_id: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSqlMemoryRecordLifecycle {
    pub memory_id: String,
    pub status: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSqlMemoryAuditRecord {
    pub audit_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSqlMemoryOutboxEvent {
    pub outbox_id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub event_version: String,
    pub payload_json: String,
    pub publish_state: String,
    pub published_at: Option<String>,
    pub retry_count: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct NativeSqlAppendOutboxEventCommand<'a> {
    pub scope: &'a MemoryScopeContext,
    pub outbox_id: &'a str,
    pub aggregate_type: &'a str,
    pub aggregate_id: &'a str,
    pub event_type: &'a str,
    pub event_version: &'a str,
    pub payload_json: &'a str,
}

#[derive(Debug, Error)]
pub enum NativeSqlStoreError {
    #[error("native SQL store database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("native SQL store JSON payload error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("native SQL event append conflict for tenant {tenant_id} event {event_id}")]
    EventConflict { tenant_id: i64, event_id: String },
    #[error("native SQL outbox append conflict for tenant {tenant_id} outbox event {outbox_id}")]
    OutboxConflict { tenant_id: i64, outbox_id: String },
    #[error("native SQL store invariant violation: {message}")]
    InvariantViolation { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeSqlEventIdempotencyState {
    space_id: i64,
    payload_json: String,
    payload_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeSqlOutboxIdempotencyState {
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    event_version: String,
    payload_json: String,
    publish_state: String,
    published_at: Option<String>,
    retry_count: i64,
}

impl NativeSqlOutboxIdempotencyState {
    fn into_outbox_event(self, outbox_id: &str) -> NativeSqlMemoryOutboxEvent {
        NativeSqlMemoryOutboxEvent {
            outbox_id: outbox_id.to_string(),
            aggregate_type: self.aggregate_type,
            aggregate_id: self.aggregate_id,
            event_type: self.event_type,
            event_version: self.event_version,
            payload_json: self.payload_json,
            publish_state: self.publish_state,
            published_at: self.published_at,
            retry_count: self.retry_count,
        }
    }
}

fn into_spi_outbox_event(outbox: NativeSqlMemoryOutboxEvent) -> MemoryOutboxEvent {
    MemoryOutboxEvent {
        outbox_id: outbox.outbox_id,
        aggregate_type: outbox.aggregate_type,
        aggregate_id: outbox.aggregate_id,
        event_type: outbox.event_type,
        event_version: outbox.event_version,
        payload_json: outbox.payload_json,
        publish_state: outbox.publish_state,
        published_at: outbox.published_at,
        retry_count: outbox.retry_count,
    }
}

fn candidate_from_row(row: SqliteRow) -> MemoryCandidate {
    MemoryCandidate {
        candidate_id: row.get("uuid"),
        candidate_type: row.get("candidate_type"),
        memory_type: row.get("memory_type"),
        proposed_text: row.get("proposed_text"),
        proposed_payload_json: row.get("proposed_payload_json"),
        evidence_json: row.get("evidence_json"),
        confidence: row.get("confidence"),
        decision_state: row.get("decision_state"),
        decision_reason: row.get("decision_reason"),
        decided_by: row.get("decided_by"),
        decided_at: row.get("decided_at"),
    }
}

fn habit_from_row(row: SqliteRow) -> MemoryHabit {
    MemoryHabit {
        habit_id: row.get("uuid"),
        user_id: row.get("user_id"),
        habit_key: row.get("habit_key"),
        habit_type: row.get("habit_type"),
        description: row.get("description"),
        stage: row.get("stage"),
        strength: row.get("strength"),
        confidence: row.get("confidence"),
        support_count: row.get("support_count"),
        last_signal_at: row.get("last_signal_at"),
        promoted_memory_id: row.get("promoted_memory_uuid"),
        decay_after: row.get("decay_after"),
        metadata_json: row.get("metadata_json"),
    }
}

fn retrieval_trace_select_sql() -> &'static str {
    r#"
    SELECT
      id,
      uuid,
      actor_id,
      query_text,
      query_hash,
      retrievers_json,
      latency_ms,
      result_count,
      degraded,
      metadata_json
    FROM mem_retrieval_trace
    WHERE tenant_id = ? AND space_id = ? AND uuid = ?
    "#
}

fn bool_to_sqlite_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn sqlite_int_to_bool(value: i64) -> bool {
    value != 0
}

fn now_text() -> &'static str {
    "2026-06-10T00:00:00Z"
}

fn stable_hash(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn parse_event_payload(payload: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(payload)
}

fn port_error(port: &str, error: NativeSqlStoreError) -> MemorySpiError {
    if let NativeSqlStoreError::EventConflict { event_id, .. } = error {
        return MemorySpiError::IdempotencyConflict {
            idempotency_key: event_id,
        };
    }
    if let NativeSqlStoreError::OutboxConflict { outbox_id, .. } = error {
        return MemorySpiError::IdempotencyConflict {
            idempotency_key: outbox_id,
        };
    }

    MemorySpiError::PortOperationFailed {
        port: port.to_string(),
        message: error.to_string(),
    }
}
