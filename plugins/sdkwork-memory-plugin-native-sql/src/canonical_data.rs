//! Coarse-grained canonical memory mutations with durable journal side effects.

use sdkwork_memory_spi::{
    CreateCanonicalMemoryCommand, DeleteCanonicalMemoryCommand, MemoryCanonicalRecord,
    MemoryDeletionReceipt, MemoryMutationJournal, UpdateCanonicalMemoryCommand,
};

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
        tx.commit().await.map_err(NativeSqlStoreError::from)?;

        self.sync_record_fts_entry(
            &command.scope,
            &command.memory_id,
            &command.canonical_text,
            &command.object_text,
            command.subject.as_deref(),
            command.predicate.as_deref(),
        )
        .await?;
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
        tx.commit().await.map_err(NativeSqlStoreError::from)?;

        let record = self
            .load_canonical_record(&command.scope, &command.memory_id)
            .await?;
        self.sync_record_fts_entry(
            &command.scope,
            &command.memory_id,
            &record.canonical_text,
            &record.object_text,
            record.subject.as_deref(),
            record.predicate.as_deref(),
        )
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
        tx.commit().await.map_err(NativeSqlStoreError::from)?;
        self.remove_record_fts_entry(&command.scope, &command.memory_id)
            .await?;
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
