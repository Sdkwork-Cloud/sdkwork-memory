use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sdkwork_memory_spi::{
    AppendMemoryAuditCommand, AppendMemoryEventCommand, AppendMemoryOutboxCommand,
    AppendMemoryRetrievalTraceCommand, ApproveMemoryCandidateCommand, AssembleMemoryContextCommand,
    CreateCanonicalMemoryCommand, CreateMemoryCandidateCommand, CreateMemoryRecordCommand,
    DecayMemoryHabitCommand, DeleteCanonicalMemoryCommand, DeleteMemoryRecordCommand,
    ExternalMemoryBridgePort, ExternalMemoryDeleteCommand,
    ExternalMemoryDeleteReceipt, ExternalMemoryExportCommand, ExternalMemoryExportResult,
    ExternalMemoryImportCommand, ExternalMemoryImportResult, ExternalMemoryShadowReadCommand,
    ExternalMemoryShadowReadResult, ListMemoryRetrievalTracesQuery, ListPendingMemoryOutboxQuery,
    MarkMemoryOutboxFailedCommand, MarkMemoryOutboxPublishedCommand, MemoryAuditRecord,
    MemoryAuditStorePort, MemoryCandidate, MemoryCandidateStorePort, MemoryContextAssemblerPort,
    MemoryContextPackDraft, MemoryDeletionReceipt, MemoryEvalRunResult, MemoryEvaluationPort,
    MemoryEvent, MemoryEventStorePort, MemoryHabit, MemoryHabitStorePort, MemoryIndexPort,
    MemoryCanonicalRecord, MemoryIndexReceipt, MemoryMutationJournal, MemoryOutboxEvent,
    MemoryOutboxStorePort, MemoryPluginPorts, MemoryRecord, MemoryRecordStorePort,
    MemoryRetrievalEventCandidate, MemoryRetrievalRecordCandidate, MemoryRetrievalTrace,
    MemoryRetrievalTraceStorePort, MemoryRetrieverKind, MemoryRetrieverPort, MemoryRetrieverResult,
    MemoryRetrieverSearchResult, MemoryScopeContext, MemorySensitivityReadScope, MemorySpiError,
    MemorySpiResult, PromoteMemoryHabitCommand, RejectMemoryCandidateCommand,
    RetrieveCanonicalMemoryQuery, RetrieveMemoryAuditQuery, RetrieveMemoryCandidateQuery,
    RetrieveMemoryCandidatesCommand, RetrieveMemoryEventQuery, RetrieveMemoryHabitQuery,
    RetrieveMemoryOutboxQuery, RetrieveMemoryRecordQuery, RetrieveMemoryRetrievalTraceQuery,
    RunMemoryEvalCommand, SearchMemoryCandidatesQuery, UpdateCanonicalMemoryCommand,
    UpsertMemoryHabitCommand, MAX_MEMORY_RETRIEVAL_CANDIDATES,
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

pub fn build_reference_executable_runtime(
    runtime: Arc<ReferenceMemoryRuntime>,
) -> sdkwork_memory_spi::MemoryExecutablePluginRuntime {
    sdkwork_memory_spi::MemoryExecutablePluginRuntime::new(
        MemoryPluginPorts::new()
            .with_record_store(runtime.clone())
            .with_event_store(runtime.clone())
            .with_audit_store(runtime.clone())
            .with_outbox_store(runtime.clone())
            .with_candidate_store(runtime.clone())
            .with_habit_store(runtime.clone())
            .with_retrieval_trace_store(runtime.clone())
            .with_retriever(runtime.clone())
            .with_index(runtime.clone())
            .with_external_memory_bridge(runtime.clone())
            .with_context_assembler(runtime.clone())
            .with_evaluation(runtime),
    )
}

#[async_trait]
impl MemoryRecordStorePort for ReferenceMemoryRuntime {
    fn supports_canonical_atomic(&self) -> bool {
        true
    }

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

    async fn create_canonical_atomic(
        &self,
        command: CreateCanonicalMemoryCommand,
    ) -> MemorySpiResult<MemoryCanonicalRecord> {
        validate_memory_journal(&command.memory_id, &command.journal)?;
        let timestamp = now_text();
        let canonical = MemoryCanonicalRecord {
            memory_id: command.memory_id.clone(),
            space_id: command.scope.space_id,
            user_id: command.scope.user_id,
            scope_label: command.scope_label,
            memory_type: command.memory_type,
            subject: command.subject,
            predicate: command.predicate.or_else(|| Some("is".to_string())),
            object_text: command.object_text.clone(),
            canonical_text: command.canonical_text,
            confidence: 1.0,
            evidence_count: 1,
            contradiction_count: 0,
            status: "active".to_string(),
            sensitivity_level: command.sensitivity_level,
            supersedes_memory_id: None,
            superseded_by_memory_id: None,
            created_at: timestamp.clone(),
            updated_at: timestamp,
            version: 1,
        };
        let key = ScopedId::new(&command.scope, command.memory_id);
        let (outbox_key, outbox, audit_key, audit) =
            reference_journal_entries(&command.scope, command.journal);

        let mut records = self.records.lock().map_err(lock_error)?;
        let mut outbox_store = self.outbox.lock().map_err(lock_error)?;
        let mut audit_store = self.audits.lock().map_err(lock_error)?;
        records.insert(key, MemoryRecordState::active_canonical(canonical.clone()));
        outbox_store.insert(outbox_key, outbox);
        audit_store.insert(audit_key, audit);
        Ok(canonical)
    }

    async fn retrieve_canonical(
        &self,
        query: RetrieveCanonicalMemoryQuery,
    ) -> MemorySpiResult<Option<MemoryCanonicalRecord>> {
        let key = ScopedId::new(&query.scope, query.memory_id);
        Ok(self
            .records
            .lock()
            .map_err(lock_error)?
            .get(&key)
            .and_then(MemoryRecordState::visible_canonical))
    }

    async fn update_canonical_atomic(
        &self,
        command: UpdateCanonicalMemoryCommand,
    ) -> MemorySpiResult<Option<MemoryCanonicalRecord>> {
        validate_memory_journal(&command.memory_id, &command.journal)?;
        let key = ScopedId::new(&command.scope, command.memory_id);
        let (outbox_key, outbox, audit_key, audit) =
            reference_journal_entries(&command.scope, command.journal);

        let mut records = self.records.lock().map_err(lock_error)?;
        let Some(state) = records.get_mut(&key) else {
            return Ok(None);
        };
        if state.deleted {
            return Ok(None);
        }
        let Some(canonical) = state.canonical.as_mut() else {
            return Err(atomic_record_state_missing());
        };
        if let Some(text) = command.canonical_text {
            canonical.canonical_text = text.clone();
            canonical.object_text = text.clone();
            state.record.content = text;
        }
        if let Some(subject) = command.subject {
            canonical.subject = Some(subject);
        }
        canonical.updated_at = now_text();
        canonical.version += 1;
        let updated = canonical.clone();

        self.outbox
            .lock()
            .map_err(lock_error)?
            .insert(outbox_key, outbox);
        self.audits
            .lock()
            .map_err(lock_error)?
            .insert(audit_key, audit);
        Ok(Some(updated))
    }

    async fn delete_canonical_atomic(
        &self,
        command: DeleteCanonicalMemoryCommand,
    ) -> MemorySpiResult<MemoryDeletionReceipt> {
        validate_memory_journal(&command.memory_id, &command.journal)?;
        let key = ScopedId::new(&command.scope, command.memory_id.clone());
        let (outbox_key, outbox, audit_key, audit) =
            reference_journal_entries(&command.scope, command.journal);

        let mut records = self.records.lock().map_err(lock_error)?;
        let Some(state) = records.get_mut(&key) else {
            return Ok(MemoryDeletionReceipt {
                memory_id: command.memory_id,
                deleted: false,
                already_deleted: false,
            });
        };
        if state.deleted {
            return Ok(MemoryDeletionReceipt {
                memory_id: command.memory_id,
                deleted: true,
                already_deleted: true,
            });
        }
        state.deleted = true;
        if let Some(canonical) = state.canonical.as_mut() {
            canonical.status = "deleted".to_string();
            canonical.updated_at = now_text();
            canonical.version += 1;
        }
        self.outbox
            .lock()
            .map_err(lock_error)?
            .insert(outbox_key, outbox);
        self.audits
            .lock()
            .map_err(lock_error)?
            .insert(audit_key, audit);
        Ok(MemoryDeletionReceipt {
            memory_id: command.memory_id,
            deleted: true,
            already_deleted: false,
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

    fn supports_bounded_scoped_search(&self) -> bool {
        true
    }

    async fn retrieve(
        &self,
        _command: RetrieveMemoryCandidatesCommand,
    ) -> MemorySpiResult<MemoryRetrieverResult> {
        Err(scope_required_error(
            "MemoryRetrieverPort",
            "retrieve_scoped",
        ))
    }

    async fn retrieve_scoped(
        &self,
        scope: MemoryScopeContext,
        command: RetrieveMemoryCandidatesCommand,
    ) -> MemorySpiResult<MemoryRetrieverResult> {
        let query = command.query.to_ascii_lowercase();
        let records = self.records.lock().map_err(lock_error)?;
        let mut memory_ids = records
            .iter()
            .filter(|(key, _state)| key.matches_scope(&scope))
            .filter_map(|(_key, state)| {
                state
                    .visible_record()
                    .filter(|record| record.content.to_ascii_lowercase().contains(&query))
                    .map(|record| record.memory_id)
            })
            .collect::<Vec<_>>();
        memory_ids.sort();
        memory_ids.truncate(MAX_MEMORY_RETRIEVAL_CANDIDATES as usize);
        Ok(MemoryRetrieverResult { memory_ids })
    }

    async fn search_scoped(
        &self,
        query: SearchMemoryCandidatesQuery,
    ) -> MemorySpiResult<MemoryRetrieverSearchResult> {
        let normalized_query = query.query.trim().to_lowercase();
        if normalized_query.is_empty() {
            return Err(MemorySpiError::PortOperationFailed {
                port: "MemoryRetrieverPort".to_string(),
                message: "memory retrieval query must not be blank".to_string(),
            });
        }
        if query.retriever_kinds.is_empty() {
            return Err(MemorySpiError::PortOperationFailed {
                port: "MemoryRetrieverPort".to_string(),
                message: "memory retrieval must select at least one retriever".to_string(),
            });
        }
        if query.limit == 0 || query.limit > MAX_MEMORY_RETRIEVAL_CANDIDATES {
            return Err(MemorySpiError::PortOperationFailed {
                port: "MemoryRetrieverPort".to_string(),
                message: format!(
                    "memory retrieval candidate limit must be between 1 and {MAX_MEMORY_RETRIEVAL_CANDIDATES}"
                ),
            });
        }

        let supported_record_search = query.retriever_kinds.iter().any(|kind| {
            matches!(
                kind,
                MemoryRetrieverKind::Keyword
                    | MemoryRetrieverKind::Dictionary
                    | MemoryRetrieverKind::Time
            )
        });
        let unavailable_retriever_kinds = query
            .retriever_kinds
            .iter()
            .filter(|kind| {
                !matches!(
                    kind,
                    MemoryRetrieverKind::Keyword
                        | MemoryRetrieverKind::Dictionary
                        | MemoryRetrieverKind::Time
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        if !supported_record_search {
            return Err(MemorySpiError::PortOperationFailed {
                port: "MemoryRetrieverPort".to_string(),
                message: "selected memory retrievers are not supported by the reference runtime"
                    .to_string(),
            });
        }

        let records = self.records.lock().map_err(lock_error)?;
        let mut candidates = records
            .iter()
            .filter(|(key, _state)| key.matches_scope(&query.scope))
            .filter_map(|(_key, state)| state.visible_canonical())
            .filter(|record| {
                query.memory_types.is_empty()
                    || query
                        .memory_types
                        .iter()
                        .any(|memory_type| memory_type == &record.memory_type)
            })
            .filter(|record| match query.read_scope {
                MemorySensitivityReadScope::Owner => true,
                MemorySensitivityReadScope::Elevated => record.sensitivity_level != "restricted",
                MemorySensitivityReadScope::Public => {
                    matches!(record.sensitivity_level.as_str(), "public" | "internal")
                }
            })
            .filter(|record| {
                let searchable = format!(
                    "{} {} {} {}",
                    record.subject.as_deref().unwrap_or_default(),
                    record.predicate.as_deref().unwrap_or_default(),
                    record.object_text,
                    record.canonical_text
                )
                .to_lowercase();
                searchable.contains(&normalized_query)
            })
            .map(|record| MemoryRetrievalRecordCandidate {
                memory_id: record.memory_id,
                subject: record.subject,
                predicate: record.predicate,
                object_text: record.object_text,
                canonical_text: record.canonical_text,
                created_at: record.created_at,
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| left.memory_id.cmp(&right.memory_id))
        });
        candidates.truncate(query.limit as usize);

        Ok(MemoryRetrieverSearchResult {
            records: candidates,
            events: Vec::<MemoryRetrievalEventCandidate>::new(),
            degraded: !unavailable_retriever_kinds.is_empty(),
            unavailable_retriever_kinds,
        })
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
        _command: AssembleMemoryContextCommand,
    ) -> MemorySpiResult<MemoryContextPackDraft> {
        Err(scope_required_error(
            "MemoryContextAssemblerPort",
            "assemble_scoped",
        ))
    }

    async fn assemble_scoped(
        &self,
        scope: MemoryScopeContext,
        command: AssembleMemoryContextCommand,
    ) -> MemorySpiResult<MemoryContextPackDraft> {
        let records = self.records.lock().map_err(lock_error)?;
        let mut memory_ids = Vec::new();
        let mut context_lines = Vec::new();
        for memory_id in command.memory_ids {
            let key = ScopedId::new(&scope, memory_id.clone());
            if let Some(record) = records
                .get(&key)
                .and_then(MemoryRecordState::visible_record)
            {
                memory_ids.push(memory_id);
                context_lines.push(record.content);
            }
        }

        Ok(MemoryContextPackDraft {
            memory_ids,
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
    canonical: Option<MemoryCanonicalRecord>,
    deleted: bool,
}

impl MemoryRecordState {
    fn active(record: MemoryRecord) -> Self {
        Self {
            record,
            canonical: None,
            deleted: false,
        }
    }

    fn active_canonical(canonical: MemoryCanonicalRecord) -> Self {
        Self {
            record: MemoryRecord {
                memory_id: canonical.memory_id.clone(),
                content: canonical.object_text.clone(),
            },
            canonical: Some(canonical),
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

    fn visible_canonical(&self) -> Option<MemoryCanonicalRecord> {
        if self.deleted {
            None
        } else {
            self.canonical.clone()
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

fn validate_memory_journal(
    memory_id: &str,
    journal: &MemoryMutationJournal,
) -> MemorySpiResult<()> {
    if journal.aggregate_id != memory_id || journal.audit_resource_id != memory_id {
        return Err(MemorySpiError::PortOperationFailed {
            port: "MemoryRecordStorePort".to_string(),
            message: "memory mutation journal resource ids must match the canonical memory id"
                .to_string(),
        });
    }
    Ok(())
}

fn reference_journal_entries(
    scope: &MemoryScopeContext,
    journal: MemoryMutationJournal,
) -> (ScopedId, MemoryOutboxEvent, ScopedId, MemoryAuditRecord) {
    let outbox_key = ScopedId::new(scope, journal.outbox_id.clone());
    let audit_key = ScopedId::new(scope, journal.audit_id.clone());
    let outbox = MemoryOutboxEvent {
        outbox_id: journal.outbox_id,
        aggregate_type: journal.aggregate_type,
        aggregate_id: journal.aggregate_id,
        event_type: journal.event_type,
        event_version: journal.event_version,
        payload_json: journal.payload_json,
        publish_state: "pending".to_string(),
        published_at: None,
        retry_count: 0,
    };
    let audit = MemoryAuditRecord {
        audit_id: journal.audit_id,
        action: journal.audit_action,
        resource_type: journal.audit_resource_type,
        resource_id: journal.audit_resource_id,
        result: journal.audit_result,
    };
    (outbox_key, outbox, audit_key, audit)
}

fn atomic_record_state_missing() -> MemorySpiError {
    MemorySpiError::PortOperationFailed {
        port: "MemoryRecordStorePort".to_string(),
        message: "reference record was not created through the canonical atomic path".to_string(),
    }
}

fn scope_required_error(port: &str, scoped_method: &str) -> MemorySpiError {
    MemorySpiError::PortOperationFailed {
        port: port.to_string(),
        message: format!(
            "reference runtime requires explicit tenant and space scope; use {scoped_method}"
        ),
    }
}

fn lock_error<T>(_error: std::sync::PoisonError<T>) -> MemorySpiError {
    MemorySpiError::PortOperationFailed {
        port: "ReferenceMemoryRuntime".to_string(),
        message: "reference runtime lock is poisoned".to_string(),
    }
}

fn now_text() -> String {
    sdkwork_utils_rust::format_datetime(sdkwork_utils_rust::now(), None)
}
