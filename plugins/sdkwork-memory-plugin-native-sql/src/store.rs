use async_trait::async_trait;
use sdkwork_memory_spi::{
    AppendMemoryAuditCommand, AppendMemoryEventCommand, AppendMemoryOutboxCommand,
    CreateMemoryRecordCommand, MemoryAuditRecord, MemoryAuditStorePort, MemoryEvent,
    MemoryEventStorePort, MemoryOutboxEvent, MemoryOutboxStorePort, MemoryRecord,
    MemoryRecordStorePort, MemoryScopeContext, MemorySpiError, MemorySpiResult,
    RetrieveMemoryAuditQuery, RetrieveMemoryEventQuery, RetrieveMemoryOutboxQuery,
    RetrieveMemoryRecordQuery,
};
use serde_json::Value;
use sqlx::{Row, SqlitePool};
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
            "SELECT uuid, object_text FROM mem_record WHERE tenant_id = ? AND space_id = ? AND uuid = ?",
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
        scope: &MemoryScopeContext,
        outbox_id: &str,
        aggregate_type: &str,
        aggregate_id: &str,
        event_type: &str,
        event_version: &str,
        payload_json: &str,
    ) -> Result<NativeSqlMemoryOutboxEvent, NativeSqlStoreError> {
        let _payload: Value = serde_json::from_str(payload_json)?;

        if let Some(existing) = self
            .retrieve_outbox_idempotency_state(scope, outbox_id)
            .await?
        {
            if existing.aggregate_type == aggregate_type
                && existing.aggregate_id == aggregate_id
                && existing.event_type == event_type
                && existing.event_version == event_version
                && existing.payload_json == payload_json
            {
                return Ok(existing.into_outbox_event(outbox_id));
            }

            return Err(NativeSqlStoreError::OutboxConflict {
                tenant_id: scope.tenant_id,
                outbox_id: outbox_id.to_string(),
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
        .bind(outbox_id)
        .bind(scope.tenant_id)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(event_type)
        .bind(event_version)
        .bind(payload_json)
        .bind(now_text())
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        Ok(NativeSqlMemoryOutboxEvent {
            outbox_id: outbox_id.to_string(),
            aggregate_type: aggregate_type.to_string(),
            aggregate_id: aggregate_id.to_string(),
            event_type: event_type.to_string(),
            event_version: event_version.to_string(),
            payload_json: payload_json.to_string(),
            publish_state: "pending".to_string(),
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
            retry_count: row.get("retry_count"),
        }))
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
            retry_count: row.get("retry_count"),
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
            .append_outbox_event(
                &command.scope,
                &command.outbox_id,
                &command.aggregate_type,
                &command.aggregate_id,
                &command.event_type,
                &command.event_version,
                &command.payload_json,
            )
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

        Ok(outbox.map(|outbox| MemoryOutboxEvent {
            outbox_id: outbox.outbox_id,
            aggregate_type: outbox.aggregate_type,
            aggregate_id: outbox.aggregate_id,
            event_type: outbox.event_type,
            event_version: outbox.event_version,
            payload_json: outbox.payload_json,
            publish_state: outbox.publish_state,
            retry_count: outbox.retry_count,
        }))
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
    pub retry_count: i64,
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
            retry_count: self.retry_count,
        }
    }
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
