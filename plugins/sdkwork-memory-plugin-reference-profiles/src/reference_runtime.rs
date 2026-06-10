use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use sdkwork_memory_spi::{
    AppendMemoryAuditCommand, AppendMemoryEventCommand, AppendMemoryOutboxCommand,
    AppendMemoryRetrievalTraceCommand, ApproveMemoryCandidateCommand, AssembleMemoryContextCommand,
    CreateMemoryCandidateCommand, CreateMemoryRecordCommand, DecayMemoryHabitCommand,
    DeleteMemoryRecordCommand, ExternalMemoryBridgePort, ExternalMemoryDeleteCommand,
    ExternalMemoryDeleteReceipt, ExternalMemoryExportCommand, ExternalMemoryExportResult,
    ExternalMemoryImportCommand, ExternalMemoryImportResult, ExternalMemoryShadowReadCommand,
    ExternalMemoryShadowReadResult, ListMemoryRetrievalTracesQuery, ListPendingMemoryOutboxQuery,
    MarkMemoryOutboxFailedCommand, MarkMemoryOutboxPublishedCommand, MemoryAuditRecord,
    MemoryAuditStorePort, MemoryCandidate, MemoryCandidateStorePort, MemoryContextAssemblerPort,
    MemoryContextPackDraft, MemoryDeletionReceipt, MemoryEvalRunResult, MemoryEvaluationPort,
    MemoryEvent, MemoryEventStorePort, MemoryHabit, MemoryHabitStorePort, MemoryIndexPort,
    MemoryIndexReceipt, MemoryOutboxEvent, MemoryOutboxStorePort, MemoryRecord,
    MemoryRecordStorePort, MemoryRetrievalTrace, MemoryRetrievalTraceStorePort,
    MemoryRetrieverPort, MemoryRetrieverResult, MemoryScopeContext, MemorySpiError,
    MemorySpiResult, PromoteMemoryHabitCommand, RejectMemoryCandidateCommand,
    RetrieveMemoryAuditQuery, RetrieveMemoryCandidateQuery, RetrieveMemoryCandidatesCommand,
    RetrieveMemoryEventQuery, RetrieveMemoryHabitQuery, RetrieveMemoryOutboxQuery,
    RetrieveMemoryRecordQuery, RetrieveMemoryRetrievalTraceQuery, RunMemoryEvalCommand,
    UpsertMemoryHabitCommand,
};

#[derive(Debug, Default)]
pub struct ReferenceMemoryRuntime {
    records: Mutex<HashMap<ScopedId, MemoryRecordState>>,
    events: Mutex<HashMap<ScopedId, MemoryEvent>>,
    audits: Mutex<HashMap<ScopedId, MemoryAuditRecord>>,
    outbox: Mutex<HashMap<ScopedId, MemoryOutboxEvent>>,
    candidates: Mutex<HashMap<ScopedId, MemoryCandidate>>,
    habits: Mutex<HashMap<ScopedHabitKey, MemoryHabit>>,
    retrieval_traces: Mutex<HashMap<ScopedId, MemoryRetrievalTrace>>,
}

impl ReferenceMemoryRuntime {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MemoryRecordStorePort for ReferenceMemoryRuntime {
    async fn create(&self, command: CreateMemoryRecordCommand) -> MemorySpiResult<MemoryRecord> {
        let record = MemoryRecord {
            memory_id: command.memory_id.clone(),
            content: command.content,
        };
        let key = ScopedId::new(&command.scope, command.memory_id);
        self.records
            .lock()
            .map_err(lock_error)?
            .insert(key, MemoryRecordState::active(record.clone()));

        Ok(record)
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryRecordQuery,
    ) -> MemorySpiResult<Option<MemoryRecord>> {
        let key = ScopedId::new(&query.scope, query.memory_id);
        let records = self.records.lock().map_err(lock_error)?;
        Ok(records
            .get(&key)
            .and_then(MemoryRecordState::visible_record))
    }

    async fn mark_deleted(
        &self,
        command: DeleteMemoryRecordCommand,
    ) -> MemorySpiResult<MemoryDeletionReceipt> {
        let key = ScopedId::new(&command.scope, command.memory_id.clone());
        let mut records = self.records.lock().map_err(lock_error)?;

        let Some(record) = records.get_mut(&key) else {
            return Ok(MemoryDeletionReceipt {
                memory_id: command.memory_id,
                deleted: false,
                already_deleted: false,
            });
        };

        let already_deleted = record.deleted;
        record.deleted = true;

        Ok(MemoryDeletionReceipt {
            memory_id: command.memory_id,
            deleted: true,
            already_deleted,
        })
    }
}

#[async_trait]
impl MemoryEventStorePort for ReferenceMemoryRuntime {
    async fn append(&self, command: AppendMemoryEventCommand) -> MemorySpiResult<MemoryEvent> {
        let event = MemoryEvent {
            event_id: command.event_id.clone(),
            content: command.content,
        };
        let key = ScopedId::new(&command.scope, command.event_id);
        self.events
            .lock()
            .map_err(lock_error)?
            .insert(key, event.clone());

        Ok(event)
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryEventQuery,
    ) -> MemorySpiResult<Option<MemoryEvent>> {
        let key = ScopedId::new(&query.scope, query.event_id);
        Ok(self.events.lock().map_err(lock_error)?.get(&key).cloned())
    }
}

#[async_trait]
impl MemoryAuditStorePort for ReferenceMemoryRuntime {
    async fn append(
        &self,
        command: AppendMemoryAuditCommand,
    ) -> MemorySpiResult<MemoryAuditRecord> {
        let audit = MemoryAuditRecord {
            audit_id: command.audit_id.clone(),
            action: command.action,
            resource_type: command.resource_type,
            resource_id: command.resource_id,
            result: command.result,
        };
        let key = ScopedId::new(&command.scope, command.audit_id);
        self.audits
            .lock()
            .map_err(lock_error)?
            .insert(key, audit.clone());

        Ok(audit)
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryAuditQuery,
    ) -> MemorySpiResult<Option<MemoryAuditRecord>> {
        let key = ScopedId::new(&query.scope, query.audit_id);
        Ok(self.audits.lock().map_err(lock_error)?.get(&key).cloned())
    }
}

#[async_trait]
impl MemoryOutboxStorePort for ReferenceMemoryRuntime {
    async fn append(
        &self,
        command: AppendMemoryOutboxCommand,
    ) -> MemorySpiResult<MemoryOutboxEvent> {
        let outbox = MemoryOutboxEvent {
            outbox_id: command.outbox_id.clone(),
            aggregate_type: command.aggregate_type,
            aggregate_id: command.aggregate_id,
            event_type: command.event_type,
            event_version: command.event_version,
            payload_json: command.payload_json,
            publish_state: "pending".to_string(),
            published_at: None,
            retry_count: 0,
        };
        let key = ScopedId::new(&command.scope, command.outbox_id);
        self.outbox
            .lock()
            .map_err(lock_error)?
            .insert(key, outbox.clone());

        Ok(outbox)
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryOutboxQuery,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>> {
        let key = ScopedId::new(&query.scope, query.outbox_id);
        Ok(self.outbox.lock().map_err(lock_error)?.get(&key).cloned())
    }

    async fn list_pending(
        &self,
        query: ListPendingMemoryOutboxQuery,
    ) -> MemorySpiResult<Vec<MemoryOutboxEvent>> {
        let outbox = self.outbox.lock().map_err(lock_error)?;
        let mut pending = outbox
            .iter()
            .filter(|(key, event)| {
                key.matches_scope(&query.scope) && event.publish_state == "pending"
            })
            .map(|(_, event)| event.clone())
            .collect::<Vec<_>>();
        pending.sort_by(|left, right| left.outbox_id.cmp(&right.outbox_id));
        pending.truncate(query.limit as usize);
        Ok(pending)
    }

    async fn mark_published(
        &self,
        command: MarkMemoryOutboxPublishedCommand,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>> {
        self.update_outbox_state(
            &command.scope,
            command.outbox_id,
            "published",
            Some(now_text()),
        )
    }

    async fn mark_failed(
        &self,
        command: MarkMemoryOutboxFailedCommand,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>> {
        let key = ScopedId::new(&command.scope, command.outbox_id);
        let mut outbox = self.outbox.lock().map_err(lock_error)?;
        let Some(event) = outbox.get_mut(&key) else {
            return Ok(None);
        };
        event.publish_state = "failed".to_string();
        event.retry_count += 1;
        Ok(Some(event.clone()))
    }
}

impl ReferenceMemoryRuntime {
    fn update_outbox_state(
        &self,
        scope: &MemoryScopeContext,
        outbox_id: String,
        publish_state: &str,
        published_at: Option<String>,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>> {
        let key = ScopedId::new(scope, outbox_id);
        let mut outbox = self.outbox.lock().map_err(lock_error)?;
        let Some(event) = outbox.get_mut(&key) else {
            return Ok(None);
        };
        event.publish_state = publish_state.to_string();
        event.published_at = published_at;
        Ok(Some(event.clone()))
    }
}

#[async_trait]
impl MemoryCandidateStorePort for ReferenceMemoryRuntime {
    async fn create(
        &self,
        command: CreateMemoryCandidateCommand,
    ) -> MemorySpiResult<MemoryCandidate> {
        let candidate = MemoryCandidate {
            candidate_id: command.candidate_id.clone(),
            candidate_type: command.candidate_type,
            memory_type: command.memory_type,
            proposed_text: command.proposed_text,
            proposed_payload_json: command.proposed_payload_json,
            evidence_json: command.evidence_json,
            confidence: command.confidence,
            decision_state: "pending".to_string(),
            decision_reason: None,
            decided_by: None,
            decided_at: None,
        };
        let key = ScopedId::new(&command.scope, command.candidate_id);
        self.candidates
            .lock()
            .map_err(lock_error)?
            .insert(key, candidate.clone());

        Ok(candidate)
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryCandidateQuery,
    ) -> MemorySpiResult<Option<MemoryCandidate>> {
        let key = ScopedId::new(&query.scope, query.candidate_id);
        Ok(self
            .candidates
            .lock()
            .map_err(lock_error)?
            .get(&key)
            .cloned())
    }

    async fn approve(
        &self,
        command: ApproveMemoryCandidateCommand,
    ) -> MemorySpiResult<Option<MemoryCandidate>> {
        self.decide_candidate(
            &command.scope,
            command.candidate_id,
            "approved",
            command.decision_reason,
            command.decided_by,
        )
    }

    async fn reject(
        &self,
        command: RejectMemoryCandidateCommand,
    ) -> MemorySpiResult<Option<MemoryCandidate>> {
        self.decide_candidate(
            &command.scope,
            command.candidate_id,
            "rejected",
            command.decision_reason,
            command.decided_by,
        )
    }
}

impl ReferenceMemoryRuntime {
    fn decide_candidate(
        &self,
        scope: &MemoryScopeContext,
        candidate_id: String,
        decision_state: &str,
        decision_reason: Option<String>,
        decided_by: Option<i64>,
    ) -> MemorySpiResult<Option<MemoryCandidate>> {
        let key = ScopedId::new(scope, candidate_id);
        let mut candidates = self.candidates.lock().map_err(lock_error)?;
        let Some(candidate) = candidates.get_mut(&key) else {
            return Ok(None);
        };

        candidate.decision_state = decision_state.to_string();
        candidate.decision_reason = decision_reason;
        candidate.decided_by = decided_by;
        candidate.decided_at = Some(now_text());

        Ok(Some(candidate.clone()))
    }
}

#[async_trait]
impl MemoryHabitStorePort for ReferenceMemoryRuntime {
    async fn upsert(&self, command: UpsertMemoryHabitCommand) -> MemorySpiResult<MemoryHabit> {
        let key = ScopedHabitKey::new(&command.scope, command.user_id, command.habit_key.clone());
        let habit = MemoryHabit {
            habit_id: command.habit_id,
            user_id: command.user_id,
            habit_key: command.habit_key,
            habit_type: command.habit_type,
            description: command.description,
            stage: command.stage,
            strength: command.strength,
            confidence: command.confidence,
            support_count: command.support_count,
            last_signal_at: Some(now_text()),
            promoted_memory_id: None,
            decay_after: None,
            metadata_json: command.metadata_json,
        };

        self.habits
            .lock()
            .map_err(lock_error)?
            .insert(key, habit.clone());

        Ok(habit)
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryHabitQuery,
    ) -> MemorySpiResult<Option<MemoryHabit>> {
        let key = ScopedHabitKey::new(&query.scope, query.user_id, query.habit_key);
        Ok(self.habits.lock().map_err(lock_error)?.get(&key).cloned())
    }

    async fn promote(
        &self,
        command: PromoteMemoryHabitCommand,
    ) -> MemorySpiResult<Option<MemoryHabit>> {
        let key = ScopedHabitKey::new(&command.scope, command.user_id, command.habit_key);
        let mut habits = self.habits.lock().map_err(lock_error)?;
        let Some(habit) = habits.get_mut(&key) else {
            return Ok(None);
        };

        habit.stage = "promoted".to_string();
        habit.promoted_memory_id = command.promoted_memory_id;

        Ok(Some(habit.clone()))
    }

    async fn decay(
        &self,
        command: DecayMemoryHabitCommand,
    ) -> MemorySpiResult<Option<MemoryHabit>> {
        let key = ScopedHabitKey::new(&command.scope, command.user_id, command.habit_key);
        let mut habits = self.habits.lock().map_err(lock_error)?;
        let Some(habit) = habits.get_mut(&key) else {
            return Ok(None);
        };

        habit.stage = "decayed".to_string();
        habit.strength = (habit.strength - command.strength_delta).max(0.0);

        Ok(Some(habit.clone()))
    }
}

#[async_trait]
impl MemoryRetrievalTraceStorePort for ReferenceMemoryRuntime {
    async fn append(
        &self,
        command: AppendMemoryRetrievalTraceCommand,
    ) -> MemorySpiResult<MemoryRetrievalTrace> {
        let trace = MemoryRetrievalTrace {
            trace_id: command.trace_id.clone(),
            actor_id: command.actor_id,
            query_text: command.query_text,
            query_hash: command.query_hash,
            retrievers_json: command.retrievers_json,
            latency_ms: command.latency_ms,
            result_count: command.hits.len() as i64,
            degraded: command.degraded,
            metadata_json: command.metadata_json,
            hits: command.hits,
            context_pack: command.context_pack,
        };
        let key = ScopedId::new(&command.scope, command.trace_id);
        self.retrieval_traces
            .lock()
            .map_err(lock_error)?
            .insert(key, trace.clone());

        Ok(trace)
    }

    async fn retrieve(
        &self,
        query: RetrieveMemoryRetrievalTraceQuery,
    ) -> MemorySpiResult<Option<MemoryRetrievalTrace>> {
        let key = ScopedId::new(&query.scope, query.trace_id);
        Ok(self
            .retrieval_traces
            .lock()
            .map_err(lock_error)?
            .get(&key)
            .cloned())
    }

    async fn list_recent(
        &self,
        query: ListMemoryRetrievalTracesQuery,
    ) -> MemorySpiResult<Vec<MemoryRetrievalTrace>> {
        let traces = self.retrieval_traces.lock().map_err(lock_error)?;
        let mut recent = traces
            .iter()
            .filter(|(key, _trace)| key.matches_scope(&query.scope))
            .map(|(_, trace)| trace.clone())
            .collect::<Vec<_>>();
        recent.sort_by(|left, right| right.trace_id.cmp(&left.trace_id));
        recent.truncate(query.limit as usize);
        Ok(recent)
    }
}

#[async_trait]
impl MemoryRetrieverPort for ReferenceMemoryRuntime {
    fn retriever_code(&self) -> &str {
        "reference_keyword"
    }

    async fn retrieve(
        &self,
        command: RetrieveMemoryCandidatesCommand,
    ) -> MemorySpiResult<MemoryRetrieverResult> {
        let query = command.query.to_ascii_lowercase();
        let records = self.records.lock().map_err(lock_error)?;
        let mut memory_ids = records
            .values()
            .filter_map(|state| {
                state
                    .visible_record()
                    .filter(|record| record.content.to_ascii_lowercase().contains(&query))
                    .map(|record| record.memory_id)
            })
            .collect::<Vec<_>>();
        memory_ids.sort();
        Ok(MemoryRetrieverResult { memory_ids })
    }
}

#[async_trait]
impl MemoryIndexPort for ReferenceMemoryRuntime {
    fn index_kind(&self) -> &str {
        "reference_sql_keyword"
    }

    async fn index(&self, memory_id: String) -> MemorySpiResult<MemoryIndexReceipt> {
        Ok(MemoryIndexReceipt { memory_id })
    }
}

#[async_trait]
impl ExternalMemoryBridgePort for ReferenceMemoryRuntime {
    fn provider_code(&self) -> &str {
        "reference_external_bridge_unconfigured"
    }

    async fn import(
        &self,
        _command: ExternalMemoryImportCommand,
    ) -> MemorySpiResult<ExternalMemoryImportResult> {
        Err(external_bridge_unconfigured())
    }

    async fn export(
        &self,
        _command: ExternalMemoryExportCommand,
    ) -> MemorySpiResult<ExternalMemoryExportResult> {
        Err(external_bridge_unconfigured())
    }

    async fn delete(
        &self,
        _command: ExternalMemoryDeleteCommand,
    ) -> MemorySpiResult<ExternalMemoryDeleteReceipt> {
        Err(external_bridge_unconfigured())
    }

    async fn shadow_read(
        &self,
        _command: ExternalMemoryShadowReadCommand,
    ) -> MemorySpiResult<ExternalMemoryShadowReadResult> {
        Err(external_bridge_unconfigured())
    }
}

#[async_trait]
impl MemoryContextAssemblerPort for ReferenceMemoryRuntime {
    async fn assemble(
        &self,
        command: AssembleMemoryContextCommand,
    ) -> MemorySpiResult<MemoryContextPackDraft> {
        let records = self.records.lock().map_err(lock_error)?;
        let context_lines = command
            .memory_ids
            .iter()
            .filter_map(|memory_id| {
                records.values().find_map(|state| {
                    state
                        .visible_record()
                        .filter(|record| &record.memory_id == memory_id)
                })
            })
            .map(|record| record.content)
            .collect::<Vec<_>>();

        Ok(MemoryContextPackDraft {
            memory_ids: command.memory_ids,
            context_text: context_lines.join("\n"),
        })
    }
}

#[async_trait]
impl MemoryEvaluationPort for ReferenceMemoryRuntime {
    async fn run(&self, command: RunMemoryEvalCommand) -> MemorySpiResult<MemoryEvalRunResult> {
        Ok(MemoryEvalRunResult {
            eval_type: command.eval_type,
        })
    }
}

#[derive(Debug, Clone)]
struct MemoryRecordState {
    record: MemoryRecord,
    deleted: bool,
}

impl MemoryRecordState {
    fn active(record: MemoryRecord) -> Self {
        Self {
            record,
            deleted: false,
        }
    }

    fn visible_record(&self) -> Option<MemoryRecord> {
        if self.deleted {
            None
        } else {
            Some(self.record.clone())
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ScopedId {
    tenant_id: i64,
    space_id: i64,
    id: String,
}

impl ScopedId {
    fn new(scope: &MemoryScopeContext, id: String) -> Self {
        Self {
            tenant_id: scope.tenant_id,
            space_id: scope.space_id,
            id,
        }
    }

    fn matches_scope(&self, scope: &MemoryScopeContext) -> bool {
        self.tenant_id == scope.tenant_id && self.space_id == scope.space_id
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ScopedHabitKey {
    tenant_id: i64,
    space_id: i64,
    user_id: i64,
    habit_key: String,
}

impl ScopedHabitKey {
    fn new(scope: &MemoryScopeContext, user_id: i64, habit_key: String) -> Self {
        Self {
            tenant_id: scope.tenant_id,
            space_id: scope.space_id,
            user_id,
            habit_key,
        }
    }
}

fn external_bridge_unconfigured() -> MemorySpiError {
    MemorySpiError::PortOperationFailed {
        port: "ExternalMemoryBridgePort".to_string(),
        message: "reference external memory bridge is fail-closed until a reviewed provider adapter is configured".to_string(),
    }
}

fn lock_error<T>(_error: std::sync::PoisonError<T>) -> MemorySpiError {
    MemorySpiError::PortOperationFailed {
        port: "ReferenceMemoryRuntime".to_string(),
        message: "reference runtime lock is poisoned".to_string(),
    }
}

fn now_text() -> String {
    "2026-06-10T00:00:00Z".to_string()
}
