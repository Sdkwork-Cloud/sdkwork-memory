use async_trait::async_trait;
use sdkwork_memory_spi::{
    AppendMemoryEventCommand, CreateMemoryRecordCommand, MemoryEvent, MemoryEventStorePort,
    MemoryRecord, MemoryRecordStorePort, MemorySpiError, MemorySpiResult,
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
        store.ensure_default_space().await?;
        Ok(store)
    }

    pub async fn append_event(
        &self,
        event_id: &str,
        content: &str,
    ) -> Result<(), NativeSqlStoreError> {
        let payload_json = serde_json::json!({ "content": content }).to_string();
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
            VALUES (?, 1, 1, 'system', 'memory.event.appended', 'api', ?, ?, ?, 'internal', 'received', ?)
            "#,
        )
        .bind(event_id)
        .bind(now_text())
        .bind(payload_json)
        .bind(stable_hash(content))
        .bind(now_text())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn retrieve_event(
        &self,
        event_id: &str,
    ) -> Result<Option<NativeSqlMemoryEvent>, NativeSqlStoreError> {
        let row = sqlx::query(
            "SELECT uuid, payload_json FROM mem_event WHERE tenant_id = 1 AND uuid = ?",
        )
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
        event_id: &str,
    ) -> Result<Option<Value>, NativeSqlStoreError> {
        let row =
            sqlx::query("SELECT payload_json FROM mem_event WHERE tenant_id = 1 AND uuid = ?")
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
        memory_id: &str,
        subject: &str,
        content: &str,
    ) -> Result<(), NativeSqlStoreError> {
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
            VALUES (?, 1, 1, 'user', 'semantic', ?, 'is', ?, ?, 1.0, 1, 0, 0.5, 0.5, 'active', 'internal', ?, ?)
            "#,
        )
        .bind(memory_id)
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
        memory_id: &str,
    ) -> Result<Option<NativeSqlMemoryRecord>, NativeSqlStoreError> {
        let row = sqlx::query(
            "SELECT uuid, object_text FROM mem_record WHERE tenant_id = 1 AND uuid = ?",
        )
        .bind(memory_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| NativeSqlMemoryRecord {
            memory_id: row.get("uuid"),
            content: row.get("object_text"),
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

    async fn ensure_default_space(&self) -> Result<(), NativeSqlStoreError> {
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
            VALUES (1, 'default-space', 1, 'user', 'system', 'personal', 'Default Memory Space', 'user', 'active', ?, ?, 0)
            "#,
        )
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
        self.append_event(&command.event_id, &command.content)
            .await
            .map_err(|err| port_error("MemoryEventStorePort", err))?;

        Ok(MemoryEvent {
            event_id: command.event_id,
            content: command.content,
        })
    }
}

#[async_trait]
impl MemoryRecordStorePort for NativeSqlMemoryStore {
    async fn create(&self, command: CreateMemoryRecordCommand) -> MemorySpiResult<MemoryRecord> {
        self.create_record(&command.memory_id, "spi", &command.content)
            .await
            .map_err(|err| port_error("MemoryRecordStorePort", err))?;

        Ok(MemoryRecord {
            memory_id: command.memory_id,
            content: command.content,
        })
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

#[derive(Debug, Error)]
pub enum NativeSqlStoreError {
    #[error("native SQL store database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("native SQL store JSON payload error: {0}")]
    Json(#[from] serde_json::Error),
}

fn now_text() -> &'static str {
    "2026-06-10T00:00:00Z"
}

fn stable_hash(value: &str) -> String {
    format!("len:{}", value.len())
}

fn parse_event_payload(payload: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(payload)
}

fn port_error(port: &str, error: NativeSqlStoreError) -> MemorySpiError {
    MemorySpiError::PortOperationFailed {
        port: port.to_string(),
        message: error.to_string(),
    }
}
