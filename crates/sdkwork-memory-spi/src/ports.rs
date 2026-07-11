use async_trait::async_trait;

use crate::{MemoryRetrieverKind, MemorySpiError, MemorySpiResult};

pub trait MemoryRuntimePlugin: Send + Sync {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryScopeContext {
    pub tenant_id: i64,
    pub space_id: i64,
    pub organization_id: Option<i64>,
    pub user_id: Option<i64>,
}

impl MemoryScopeContext {
    pub fn for_test(tenant_id: i64, space_id: i64) -> Self {
        Self {
            tenant_id,
            space_id,
            organization_id: None,
            user_id: Some(space_id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySensitivityReadScope {
    Public,
    Elevated,
    Owner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateMemoryRecordCommand {
    pub scope: MemoryScopeContext,
    pub memory_id: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRecord {
    pub memory_id: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCanonicalRecord {
    pub memory_id: String,
    pub space_id: i64,
    pub user_id: Option<i64>,
    pub scope_label: String,
    pub memory_type: String,
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object_text: String,
    pub canonical_text: String,
    pub confidence: f64,
    pub evidence_count: i32,
    pub contradiction_count: i32,
    pub status: String,
    pub sensitivity_level: String,
    pub supersedes_memory_id: Option<String>,
    pub superseded_by_memory_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryMutationJournal {
    pub outbox_id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub event_version: String,
    pub payload_json: String,
    pub audit_id: String,
    pub audit_action: String,
    pub audit_resource_type: String,
    pub audit_resource_id: String,
    pub audit_result: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCanonicalMemoryCommand {
    pub scope: MemoryScopeContext,
    pub memory_id: String,
    pub scope_label: String,
    pub memory_type: String,
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object_text: String,
    pub canonical_text: String,
    pub sensitivity_level: String,
    pub journal: MemoryMutationJournal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveCanonicalMemoryQuery {
    pub scope: MemoryScopeContext,
    pub memory_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCanonicalMemoryCommand {
    pub scope: MemoryScopeContext,
    pub memory_id: String,
    pub canonical_text: Option<String>,
    pub subject: Option<String>,
    pub journal: MemoryMutationJournal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteCanonicalMemoryCommand {
    pub scope: MemoryScopeContext,
    pub memory_id: String,
    pub journal: MemoryMutationJournal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveMemoryRecordQuery {
    pub scope: MemoryScopeContext,
    pub memory_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteMemoryRecordCommand {
    pub scope: MemoryScopeContext,
    pub memory_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryDeletionReceipt {
    pub memory_id: String,
    pub deleted: bool,
    pub already_deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendMemoryEventCommand {
    pub scope: MemoryScopeContext,
    pub event_id: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEvent {
    pub event_id: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveMemoryEventQuery {
    pub scope: MemoryScopeContext,
    pub event_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAuditRecord {
    pub audit_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendMemoryAuditCommand {
    pub scope: MemoryScopeContext,
    pub audit_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveMemoryAuditQuery {
    pub scope: MemoryScopeContext,
    pub audit_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryOutboxEvent {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendMemoryOutboxCommand {
    pub scope: MemoryScopeContext,
    pub outbox_id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub event_version: String,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveMemoryOutboxQuery {
    pub scope: MemoryScopeContext,
    pub outbox_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPendingMemoryOutboxQuery {
    pub scope: MemoryScopeContext,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkMemoryOutboxPublishedCommand {
    pub scope: MemoryScopeContext,
    pub outbox_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkMemoryOutboxFailedCommand {
    pub scope: MemoryScopeContext,
    pub outbox_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateMemoryCandidateCommand {
    pub scope: MemoryScopeContext,
    pub candidate_id: String,
    pub candidate_type: String,
    pub memory_type: String,
    pub proposed_text: String,
    pub proposed_payload_json: Option<String>,
    pub evidence_json: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveMemoryCandidateQuery {
    pub scope: MemoryScopeContext,
    pub candidate_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproveMemoryCandidateCommand {
    pub scope: MemoryScopeContext,
    pub candidate_id: String,
    pub decision_reason: Option<String>,
    pub decided_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectMemoryCandidateCommand {
    pub scope: MemoryScopeContext,
    pub candidate_id: String,
    pub decision_reason: Option<String>,
    pub decided_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCandidate {
    pub candidate_id: String,
    pub candidate_type: String,
    pub memory_type: String,
    pub proposed_text: String,
    pub proposed_payload_json: Option<String>,
    pub evidence_json: Option<String>,
    pub confidence: f64,
    pub decision_state: String,
    pub decision_reason: Option<String>,
    pub decided_by: Option<i64>,
    pub decided_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpsertMemoryHabitCommand {
    pub scope: MemoryScopeContext,
    pub habit_id: String,
    pub user_id: i64,
    pub habit_key: String,
    pub habit_type: String,
    pub description: String,
    pub stage: String,
    pub strength: f64,
    pub confidence: f64,
    pub support_count: i64,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveMemoryHabitQuery {
    pub scope: MemoryScopeContext,
    pub user_id: i64,
    pub habit_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromoteMemoryHabitCommand {
    pub scope: MemoryScopeContext,
    pub user_id: i64,
    pub habit_key: String,
    pub promoted_memory_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecayMemoryHabitCommand {
    pub scope: MemoryScopeContext,
    pub user_id: i64,
    pub habit_key: String,
    pub strength_delta: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHabit {
    pub habit_id: String,
    pub user_id: i64,
    pub habit_key: String,
    pub habit_type: String,
    pub description: String,
    pub stage: String,
    pub strength: f64,
    pub confidence: f64,
    pub support_count: i64,
    pub last_signal_at: Option<String>,
    pub promoted_memory_id: Option<String>,
    pub decay_after: Option<String>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetrievalHitDraft {
    pub hit_id: String,
    pub memory_id: Option<String>,
    pub space_id: Option<i64>,
    pub retriever_name: String,
    pub result_rank: i64,
    pub raw_score: Option<f64>,
    pub fused_score: Option<f64>,
    pub explanation_json: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryContextPackSnapshot {
    pub context_pack_id: String,
    pub pack_json: String,
    pub estimated_tokens: i64,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppendMemoryRetrievalTraceCommand {
    pub scope: MemoryScopeContext,
    pub trace_id: String,
    pub actor_id: Option<String>,
    pub query_text: Option<String>,
    pub query_hash: String,
    pub retrievers_json: Option<String>,
    pub latency_ms: Option<i64>,
    pub degraded: bool,
    pub metadata_json: Option<String>,
    pub hits: Vec<MemoryRetrievalHitDraft>,
    pub context_pack: Option<MemoryContextPackSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveMemoryRetrievalTraceQuery {
    pub scope: MemoryScopeContext,
    pub trace_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListMemoryRetrievalTracesQuery {
    pub scope: MemoryScopeContext,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetrievalTrace {
    pub trace_id: String,
    pub actor_id: Option<String>,
    pub query_text: Option<String>,
    pub query_hash: String,
    pub retrievers_json: Option<String>,
    pub latency_ms: Option<i64>,
    pub result_count: i64,
    pub degraded: bool,
    pub metadata_json: Option<String>,
    pub hits: Vec<MemoryRetrievalHitDraft>,
    pub context_pack: Option<MemoryContextPackSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryPolicy {
    pub policy_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveMemoryCandidatesCommand {
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRetrieverResult {
    pub memory_ids: Vec<String>,
}

pub const MAX_MEMORY_RETRIEVAL_CANDIDATES: u32 = 200;

/// Bounded, scope-aware retrieval input used by the application retrieval pipeline.
///
/// The plugin receives only the search scope and the enabled retriever kinds. It must never
/// infer tenant or space context from ambient state, and it must honor the limit at the
/// authoritative index/store boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMemoryCandidatesQuery {
    pub scope: MemoryScopeContext,
    pub query: String,
    pub limit: u32,
    pub retriever_kinds: Vec<MemoryRetrieverKind>,
    pub memory_types: Vec<String>,
    pub read_scope: MemorySensitivityReadScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRetrievalRecordCandidate {
    pub memory_id: String,
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object_text: String,
    pub canonical_text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRetrievalEventCandidate {
    pub memory_id: String,
    pub event_id: String,
    pub payload_text: String,
    pub created_at: String,
}

/// Candidate projections are not canonical truth. The service must rehydrate every candidate
/// through `MemoryRecordStorePort` before returning it or assembling context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRetrieverSearchResult {
    pub records: Vec<MemoryRetrievalRecordCandidate>,
    pub events: Vec<MemoryRetrievalEventCandidate>,
    pub degraded: bool,
    pub unavailable_retriever_kinds: Vec<MemoryRetrieverKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryIndexReceipt {
    pub memory_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageModelCommand {
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingCommand {
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RerankMemoryHitsCommand {
    pub memory_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RerankMemoryHitsResult {
    pub memory_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemoryImportCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemoryImportResult {
    pub imported_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemoryExportCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemoryExportResult {
    pub exported_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemoryDeleteCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemoryDeleteReceipt {
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemoryShadowReadCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemoryShadowReadResult {
    pub comparable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembleMemoryContextCommand {
    pub memory_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryContextPackDraft {
    pub memory_ids: Vec<String>,
    pub context_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunMemoryEvalCommand {
    pub eval_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEvalRunResult {
    pub eval_type: String,
}

#[async_trait]
pub trait MemoryRecordStorePort: Send + Sync {
    fn supports_canonical_atomic(&self) -> bool {
        false
    }

    async fn create(&self, command: CreateMemoryRecordCommand) -> MemorySpiResult<MemoryRecord>;

    async fn retrieve(
        &self,
        query: RetrieveMemoryRecordQuery,
    ) -> MemorySpiResult<Option<MemoryRecord>>;

    async fn mark_deleted(
        &self,
        command: DeleteMemoryRecordCommand,
    ) -> MemorySpiResult<MemoryDeletionReceipt>;

    async fn create_canonical_atomic(
        &self,
        _command: CreateCanonicalMemoryCommand,
    ) -> MemorySpiResult<MemoryCanonicalRecord> {
        Err(atomic_record_operation_required("create_canonical_atomic"))
    }

    async fn retrieve_canonical(
        &self,
        _query: RetrieveCanonicalMemoryQuery,
    ) -> MemorySpiResult<Option<MemoryCanonicalRecord>> {
        Err(atomic_record_operation_required("retrieve_canonical"))
    }

    async fn update_canonical_atomic(
        &self,
        _command: UpdateCanonicalMemoryCommand,
    ) -> MemorySpiResult<Option<MemoryCanonicalRecord>> {
        Err(atomic_record_operation_required("update_canonical_atomic"))
    }

    async fn delete_canonical_atomic(
        &self,
        _command: DeleteCanonicalMemoryCommand,
    ) -> MemorySpiResult<MemoryDeletionReceipt> {
        Err(atomic_record_operation_required("delete_canonical_atomic"))
    }
}

#[async_trait]
pub trait MemoryEventStorePort: Send + Sync {
    async fn append(&self, command: AppendMemoryEventCommand) -> MemorySpiResult<MemoryEvent>;

    async fn retrieve(
        &self,
        query: RetrieveMemoryEventQuery,
    ) -> MemorySpiResult<Option<MemoryEvent>>;
}

#[async_trait]
pub trait MemoryAuditStorePort: Send + Sync {
    async fn append(&self, command: AppendMemoryAuditCommand)
        -> MemorySpiResult<MemoryAuditRecord>;

    async fn retrieve(
        &self,
        query: RetrieveMemoryAuditQuery,
    ) -> MemorySpiResult<Option<MemoryAuditRecord>>;
}

#[async_trait]
pub trait MemoryOutboxStorePort: Send + Sync {
    async fn append(
        &self,
        command: AppendMemoryOutboxCommand,
    ) -> MemorySpiResult<MemoryOutboxEvent>;

    async fn retrieve(
        &self,
        query: RetrieveMemoryOutboxQuery,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>>;

    async fn list_pending(
        &self,
        query: ListPendingMemoryOutboxQuery,
    ) -> MemorySpiResult<Vec<MemoryOutboxEvent>>;

    async fn mark_published(
        &self,
        command: MarkMemoryOutboxPublishedCommand,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>>;

    async fn mark_failed(
        &self,
        command: MarkMemoryOutboxFailedCommand,
    ) -> MemorySpiResult<Option<MemoryOutboxEvent>>;
}

#[async_trait]
pub trait MemoryCandidateStorePort: Send + Sync {
    async fn create(
        &self,
        command: CreateMemoryCandidateCommand,
    ) -> MemorySpiResult<MemoryCandidate>;

    async fn retrieve(
        &self,
        query: RetrieveMemoryCandidateQuery,
    ) -> MemorySpiResult<Option<MemoryCandidate>>;

    async fn approve(
        &self,
        command: ApproveMemoryCandidateCommand,
    ) -> MemorySpiResult<Option<MemoryCandidate>>;

    async fn reject(
        &self,
        command: RejectMemoryCandidateCommand,
    ) -> MemorySpiResult<Option<MemoryCandidate>>;
}

#[async_trait]
pub trait MemoryHabitStorePort: Send + Sync {
    async fn upsert(&self, command: UpsertMemoryHabitCommand) -> MemorySpiResult<MemoryHabit>;

    async fn retrieve(
        &self,
        query: RetrieveMemoryHabitQuery,
    ) -> MemorySpiResult<Option<MemoryHabit>>;

    async fn promote(
        &self,
        command: PromoteMemoryHabitCommand,
    ) -> MemorySpiResult<Option<MemoryHabit>>;

    async fn decay(&self, command: DecayMemoryHabitCommand)
        -> MemorySpiResult<Option<MemoryHabit>>;
}

#[async_trait]
pub trait MemoryRetrievalTraceStorePort: Send + Sync {
    async fn append(
        &self,
        command: AppendMemoryRetrievalTraceCommand,
    ) -> MemorySpiResult<MemoryRetrievalTrace>;

    async fn retrieve(
        &self,
        query: RetrieveMemoryRetrievalTraceQuery,
    ) -> MemorySpiResult<Option<MemoryRetrievalTrace>>;

    async fn list_recent(
        &self,
        query: ListMemoryRetrievalTracesQuery,
    ) -> MemorySpiResult<Vec<MemoryRetrievalTrace>>;
}

#[async_trait]
pub trait MemoryPolicyStorePort: Send + Sync {
    async fn resolve_policy(&self, policy_code: String) -> MemorySpiResult<MemoryPolicy>;
}

#[async_trait]
pub trait MemoryRetrieverPort: Send + Sync {
    fn retriever_code(&self) -> &str;

    /// Whether the implementation supports the bounded, scope-aware search contract.
    /// Production HTTP composition fails closed when this capability is absent.
    fn supports_bounded_scoped_search(&self) -> bool {
        false
    }

    async fn retrieve(
        &self,
        command: RetrieveMemoryCandidatesCommand,
    ) -> MemorySpiResult<MemoryRetrieverResult>;

    async fn retrieve_scoped(
        &self,
        _scope: MemoryScopeContext,
        _command: RetrieveMemoryCandidatesCommand,
    ) -> MemorySpiResult<MemoryRetrieverResult> {
        Err(MemorySpiError::PortOperationFailed {
            port: "MemoryRetrieverPort".to_string(),
            message: "scope-aware retrieval is not implemented; refusing an unscoped fallback"
                .to_string(),
        })
    }

    async fn search_scoped(
        &self,
        _query: SearchMemoryCandidatesQuery,
    ) -> MemorySpiResult<MemoryRetrieverSearchResult> {
        Err(MemorySpiError::PortOperationFailed {
            port: "MemoryRetrieverPort".to_string(),
            message:
                "bounded scope-aware retrieval is not implemented; refusing an unbounded fallback"
                    .to_string(),
        })
    }
}

#[async_trait]
pub trait MemoryIndexPort: Send + Sync {
    fn index_kind(&self) -> &str;

    async fn index(&self, memory_id: String) -> MemorySpiResult<MemoryIndexReceipt>;
}

#[async_trait]
pub trait LanguageModelPort: Send + Sync {
    fn provider_code(&self) -> &str;

    async fn generate(&self, command: LanguageModelCommand) -> MemorySpiResult<String>;
}

#[async_trait]
pub trait EmbeddingModelPort: Send + Sync {
    fn provider_code(&self) -> &str;

    fn dimensions(&self) -> usize;

    async fn embed(&self, command: EmbeddingCommand) -> MemorySpiResult<Vec<f32>>;
}

#[async_trait]
pub trait RerankModelPort: Send + Sync {
    fn provider_code(&self) -> &str;

    async fn rerank(
        &self,
        command: RerankMemoryHitsCommand,
    ) -> MemorySpiResult<RerankMemoryHitsResult>;
}

#[async_trait]
pub trait ExternalMemoryBridgePort: Send + Sync {
    fn provider_code(&self) -> &str;

    async fn import(
        &self,
        command: ExternalMemoryImportCommand,
    ) -> MemorySpiResult<ExternalMemoryImportResult>;

    async fn export(
        &self,
        command: ExternalMemoryExportCommand,
    ) -> MemorySpiResult<ExternalMemoryExportResult>;

    async fn delete(
        &self,
        command: ExternalMemoryDeleteCommand,
    ) -> MemorySpiResult<ExternalMemoryDeleteReceipt>;

    async fn shadow_read(
        &self,
        command: ExternalMemoryShadowReadCommand,
    ) -> MemorySpiResult<ExternalMemoryShadowReadResult>;
}

#[async_trait]
pub trait MemoryContextAssemblerPort: Send + Sync {
    async fn assemble(
        &self,
        command: AssembleMemoryContextCommand,
    ) -> MemorySpiResult<MemoryContextPackDraft>;

    async fn assemble_scoped(
        &self,
        _scope: MemoryScopeContext,
        _command: AssembleMemoryContextCommand,
    ) -> MemorySpiResult<MemoryContextPackDraft> {
        Err(MemorySpiError::PortOperationFailed {
            port: "MemoryContextAssemblerPort".to_string(),
            message:
                "scope-aware context assembly is not implemented; refusing an unscoped fallback"
                    .to_string(),
        })
    }
}

#[async_trait]
pub trait MemoryEvaluationPort: Send + Sync {
    async fn run(&self, command: RunMemoryEvalCommand) -> MemorySpiResult<MemoryEvalRunResult>;
}

fn atomic_record_operation_required(operation: &str) -> MemorySpiError {
    MemorySpiError::PortOperationFailed {
        port: "MemoryRecordStorePort".to_string(),
        message: format!(
            "atomic canonical memory operation {operation} is not implemented; refusing non-atomic fallback"
        ),
    }
}
