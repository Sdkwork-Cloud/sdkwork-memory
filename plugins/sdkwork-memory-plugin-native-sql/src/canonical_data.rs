//! Coarse-grained canonical memory mutations with durable journal side effects.

use sdkwork_memory_spi::{
    CreateCanonicalMemoryCommand, DeleteCanonicalMemoryCommand, MemoryCanonicalRecord,
    MemoryDeletionReceipt, MemoryMutationJournal, UpdateCanonicalMemoryCommand,
};
use sqlx::Row;

use crate::pool_backend::MemorySqlDialect;
use crate::store::{NativeSqlMemoryRecordDetail, NativeSqlMemoryStore, NativeSqlStoreError};

impl NativeSqlMemoryStore {
    pub async fn create_canonical_memory_atomic(
        &self,
        command: &CreateCanonicalMemoryCommand,
    ) -> Result<MemoryCanonicalRecord, NativeSqlStoreError> {
        validate_journal(&command.memory_id, &command.journal)?;
        self.ensure_space(&command.scope).await?;
        let mut tx = self.begin_tx().await?;
        Self::create_record_on_tx(
            &mut tx,
            &command.scope,
            &command.memory_id,
            &command.scope_label,
            &command.memory_type,
            command.subject.as_deref(),
            command.predicate.as_deref(),
            &command.object_text,
            &command.canonical_text,
            &command.sensitivity_level,
        )
        .await?;
        append_journal_on_tx(&mut tx, &command.scope, &command.journal).await?;
        sync_record_fts_on_tx(
            self.dialect(),
            &mut tx,
            &command.scope,
            &command.memory_id,
            &command.canonical_text,
            &command.object_text,
            command.subject.as_deref(),
            command.predicate.as_deref(),
        )
        .await?;
        tx.commit().await.map_err(NativeSqlStoreError::from)?;
        self.load_canonical_record(&command.scope, &command.memory_id)
            .await
    }

    pub async fn update_canonical_memory_atomic(
        &self,
        command: &UpdateCanonicalMemoryCommand,
    ) -> Result<Option<MemoryCanonicalRecord>, NativeSqlStoreError> {
        validate_journal(&command.memory_id, &command.journal)?;
        let mut tx = self.begin_tx().await?;
        let updated = Self::update_record_on_tx(
            &mut tx,
            &command.scope,
            &command.memory_id,
            command.canonical_text.as_deref(),
            command.subject.as_deref(),
        )
        .await?;
        if !updated {
            tx.rollback().await.map_err(NativeSqlStoreError::from)?;
            return Ok(None);
        }
        append_journal_on_tx(&mut tx, &command.scope, &command.journal).await?;
        if matches!(self.dialect(), MemorySqlDialect::Sqlite) {
            let row = sqlx::query(
                r#"
                SELECT canonical_text, object_text, subject, predicate
                FROM ai_record
                WHERE tenant_id = ? AND space_id = ? AND uuid = ? AND status <> 'deleted'
                "#,
            )
            .bind(command.scope.tenant_id)
            .bind(command.scope.space_id)
            .bind(&command.memory_id)
            .fetch_one(&mut *tx)
            .await?;
            let canonical_text: String = row.get("canonical_text");
            let object_text: String = row.get("object_text");
            let subject: Option<String> = row.get("subject");
            let predicate: Option<String> = row.get("predicate");
            sync_record_fts_on_tx(
                self.dialect(),
                &mut tx,
                &command.scope,
                &command.memory_id,
                &canonical_text,
                &object_text,
                subject.as_deref(),
                predicate.as_deref(),
            )
            .await?;
        }
        tx.commit().await.map_err(NativeSqlStoreError::from)?;

        let record = self
            .load_canonical_record(&command.scope, &command.memory_id)
            .await?;
        Ok(Some(record))
    }

    pub async fn delete_canonical_memory_atomic(
        &self,
        command: &DeleteCanonicalMemoryCommand,
    ) -> Result<MemoryDeletionReceipt, NativeSqlStoreError> {
        validate_journal(&command.memory_id, &command.journal)?;
        let mut tx = self.begin_tx().await?;
        let deleted = Self::mark_record_deleted_on_tx(
            &mut tx,
            &command.scope,
            &command.memory_id,
        )
        .await?;
        if !deleted {
            tx.rollback().await.map_err(NativeSqlStoreError::from)?;
            return Ok(MemoryDeletionReceipt {
                memory_id: command.memory_id.clone(),
                deleted: false,
                already_deleted: false,
            });
        }
        append_journal_on_tx(&mut tx, &command.scope, &command.journal).await?;
        remove_record_fts_on_tx(
            self.dialect(),
            &mut tx,
            &command.scope,
            &command.memory_id,
        )
        .await?;
        tx.commit().await.map_err(NativeSqlStoreError::from)?;
        Ok(MemoryDeletionReceipt {
            memory_id: command.memory_id.clone(),
            deleted: true,
            already_deleted: false,
        })
    }

    pub async fn retrieve_canonical_memory(
        &self,
        scope: &sdkwork_memory_spi::MemoryScopeContext,
        memory_id: &str,
    ) -> Result<Option<MemoryCanonicalRecord>, NativeSqlStoreError> {
        self.retrieve_record_detail(scope, memory_id)
            .await
            .map(|record| record.map(into_canonical_record))
    }

    async fn load_canonical_record(
        &self,
        scope: &sdkwork_memory_spi::MemoryScopeContext,
        memory_id: &str,
    ) -> Result<MemoryCanonicalRecord, NativeSqlStoreError> {
        self.retrieve_canonical_memory(scope, memory_id)
            .await?
            .ok_or_else(|| NativeSqlStoreError::InvariantViolation {
                message: format!("canonical memory {memory_id} was not readable after mutation"),
            })
    }
}

async fn sync_record_fts_on_tx(
    dialect: MemorySqlDialect,
    tx: &mut sqlx::Transaction<'_, sqlx::Any>,
    scope: &sdkwork_memory_spi::MemoryScopeContext,
    memory_uuid: &str,
    canonical_text: &str,
    object_text: &str,
    subject: Option<&str>,
    predicate: Option<&str>,
) -> Result<(), NativeSqlStoreError> {
    if !matches!(dialect, MemorySqlDialect::Sqlite) {
        return Ok(());
    }
    let row_id: i64 = sqlx::query(
        "SELECT id FROM ai_record WHERE tenant_id = ? AND space_id = ? AND uuid = ?",
    )
    .bind(scope.tenant_id)
    .bind(scope.space_id)
    .bind(memory_uuid)
    .fetch_one(&mut **tx)
    .await?
    .get("id");
    sqlx::query("DELETE FROM ai_record_fts WHERE rowid = ?")
        .bind(row_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        r#"
        INSERT INTO ai_record_fts(
          rowid, memory_uuid, tenant_id, space_id, canonical_text, object_text, subject, predicate
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(row_id)
    .bind(memory_uuid)
    .bind(scope.tenant_id)
    .bind(scope.space_id)
    .bind(canonical_text)
    .bind(object_text)
    .bind(subject.unwrap_or(""))
    .bind(predicate.unwrap_or(""))
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn remove_record_fts_on_tx(
    dialect: MemorySqlDialect,
    tx: &mut sqlx::Transaction<'_, sqlx::Any>,
    scope: &sdkwork_memory_spi::MemoryScopeContext,
    memory_uuid: &str,
) -> Result<(), NativeSqlStoreError> {
    if !matches!(dialect, MemorySqlDialect::Sqlite) {
        return Ok(());
    }
    sqlx::query(
        "DELETE FROM ai_record_fts WHERE tenant_id = ? AND space_id = ? AND memory_uuid = ?",
    )
    .bind(scope.tenant_id)
    .bind(scope.space_id)
    .bind(memory_uuid)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn append_journal_on_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Any>,
    scope: &sdkwork_memory_spi::MemoryScopeContext,
    journal: &MemoryMutationJournal,
) -> Result<(), NativeSqlStoreError> {
    NativeSqlMemoryStore::append_outbox_on_tx(
        tx,
        scope,
        &journal.outbox_id,
        &journal.aggregate_type,
        &journal.aggregate_id,
        &journal.event_type,
        &journal.event_version,
        &journal.payload_json,
    )
    .await?;
    NativeSqlMemoryStore::append_audit_on_tx(
        tx,
        scope,
        &journal.audit_id,
        &journal.audit_action,
        &journal.audit_resource_type,
        &journal.audit_resource_id,
        &journal.audit_result,
    )
    .await
}

fn validate_journal(
    memory_id: &str,
    journal: &MemoryMutationJournal,
) -> Result<(), NativeSqlStoreError> {
    if journal.aggregate_id != memory_id || journal.audit_resource_id != memory_id {
        return Err(NativeSqlStoreError::InvariantViolation {
            message: "memory mutation journal resource ids must match the canonical memory id"
                .to_string(),
        });
    }
    Ok(())
}

pub(crate) fn into_canonical_record(row: NativeSqlMemoryRecordDetail) -> MemoryCanonicalRecord {
    MemoryCanonicalRecord {
        memory_id: row.memory_id,
        space_id: row.space_id,
        user_id: row.user_id,
        scope_label: row.scope,
        memory_type: row.memory_type,
        subject: row.subject,
        predicate: row.predicate,
        object_text: row.object_text,
        canonical_text: row.canonical_text,
        confidence: row.confidence,
        evidence_count: row.evidence_count.unwrap_or(0),
        contradiction_count: row.contradiction_count.unwrap_or(0),
        status: row.status,
        sensitivity_level: row.sensitivity_level,
        supersedes_memory_id: row.supersedes_memory_id,
        superseded_by_memory_id: row.superseded_by_memory_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        version: row.version,
    }
}
