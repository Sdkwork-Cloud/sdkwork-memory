use async_trait::async_trait;

use crate::MemorySpiResult;

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
            user_id: None,
        }
    }
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
    async fn create(&self, command: CreateMemoryRecordCommand) -> MemorySpiResult<MemoryRecord>;

    async fn retrieve(
        &self,
        query: RetrieveMemoryRecordQuery,
    ) -> MemorySpiResult<Option<MemoryRecord>>;

    async fn mark_deleted(
        &self,
        command: DeleteMemoryRecordCommand,
    ) -> MemorySpiResult<MemoryDeletionReceipt>;
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
pub trait MemoryPolicyStorePort: Send + Sync {
    async fn resolve_policy(&self, policy_code: String) -> MemorySpiResult<MemoryPolicy>;
}

#[async_trait]
pub trait MemoryRetrieverPort: Send + Sync {
    fn retriever_code(&self) -> &str;

    async fn retrieve(
        &self,
        command: RetrieveMemoryCandidatesCommand,
    ) -> MemorySpiResult<MemoryRetrieverResult>;
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
}

#[async_trait]
pub trait MemoryEvaluationPort: Send + Sync {
    async fn run(&self, command: RunMemoryEvalCommand) -> MemorySpiResult<MemoryEvalRunResult>;
}
