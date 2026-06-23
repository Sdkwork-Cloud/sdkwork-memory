use async_trait::async_trait;

use crate::dto::{
    ListCandidatesQuery, ListMemoriesQuery, MemoryCandidate, MemoryCandidateList,
    MemoryCapabilities, MemoryContextPack, MemoryContextPackRequest, MemoryEvent,
    MemoryEventRequest, MemoryExtractionRequest, MemoryFeedback, MemoryFeedbackRequest,
    MemoryLearningJob, MemoryProviderHealth, MemoryRecord, MemoryRecordList, MemoryRecordPatch,
    MemoryRecordRequest, MemoryRetrievalRequest, MemoryRetrievalResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryOpenApiRequestContext {
    pub api_key_id: String,
    pub tenant_id: u64,
    pub actor_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryServiceErrorKind {
    NotFound,
    Conflict,
    Validation,
    Forbidden,
    Storage,
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryServiceError {
    pub kind: MemoryServiceErrorKind,
    pub code: String,
    pub detail: String,
}

impl MemoryServiceError {
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self {
            kind: MemoryServiceErrorKind::NotFound,
            code: "not_found".to_string(),
            detail: detail.into(),
        }
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self {
            kind: MemoryServiceErrorKind::Conflict,
            code: "conflict".to_string(),
            detail: detail.into(),
        }
    }

    pub fn validation(detail: impl Into<String>) -> Self {
        Self {
            kind: MemoryServiceErrorKind::Validation,
            code: "validation_error".to_string(),
            detail: detail.into(),
        }
    }

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self {
            kind: MemoryServiceErrorKind::Forbidden,
            code: "forbidden".to_string(),
            detail: detail.into(),
        }
    }

    pub fn storage(_detail: impl Into<String>) -> Self {
        Self {
            kind: MemoryServiceErrorKind::Storage,
            code: "storage_error".to_string(),
            detail: "internal storage error".to_string(),
        }
    }

    pub fn storage_internal(detail: impl Into<String>) -> Self {
        Self {
            kind: MemoryServiceErrorKind::Storage,
            code: "storage_error".to_string(),
            detail: detail.into(),
        }
    }

    pub fn not_implemented(operation_id: &'static str) -> Self {
        Self {
            kind: MemoryServiceErrorKind::NotImplemented,
            code: "operation_not_implemented".to_string(),
            detail: format!("operation is not implemented: {operation_id}"),
        }
    }
}

pub type MemoryServiceResult<T> = Result<T, MemoryServiceError>;

#[async_trait]
pub trait MemoryOpenApi: Send + Sync + 'static {
    async fn retrieve_capabilities(
        &self,
        _context: MemoryOpenApiRequestContext,
    ) -> MemoryServiceResult<MemoryCapabilities> {
        Err(MemoryServiceError::not_implemented("capabilities.retrieve"))
    }

    async fn create_event(
        &self,
        _context: MemoryOpenApiRequestContext,
        _request: MemoryEventRequest,
    ) -> MemoryServiceResult<MemoryEvent> {
        Err(MemoryServiceError::not_implemented("events.create"))
    }

    async fn retrieve_event(
        &self,
        _context: MemoryOpenApiRequestContext,
        _event_id: u64,
        _space_id: u64,
    ) -> MemoryServiceResult<MemoryEvent> {
        Err(MemoryServiceError::not_implemented("events.retrieve"))
    }

    async fn list_memories(
        &self,
        _context: MemoryOpenApiRequestContext,
        _query: ListMemoriesQuery,
    ) -> MemoryServiceResult<MemoryRecordList> {
        Err(MemoryServiceError::not_implemented("memories.list"))
    }

    async fn create_memory(
        &self,
        _context: MemoryOpenApiRequestContext,
        _request: MemoryRecordRequest,
    ) -> MemoryServiceResult<MemoryRecord> {
        Err(MemoryServiceError::not_implemented("memories.create"))
    }

    async fn retrieve_memory(
        &self,
        _context: MemoryOpenApiRequestContext,
        _memory_id: u64,
        _space_id: u64,
    ) -> MemoryServiceResult<MemoryRecord> {
        Err(MemoryServiceError::not_implemented("memories.retrieve"))
    }

    async fn update_memory(
        &self,
        _context: MemoryOpenApiRequestContext,
        _memory_id: u64,
        _space_id: u64,
        _patch: MemoryRecordPatch,
    ) -> MemoryServiceResult<MemoryRecord> {
        Err(MemoryServiceError::not_implemented("memories.update"))
    }

    async fn delete_memory(
        &self,
        _context: MemoryOpenApiRequestContext,
        _memory_id: u64,
        _space_id: u64,
    ) -> MemoryServiceResult<()> {
        Err(MemoryServiceError::not_implemented("memories.delete"))
    }

    async fn create_retrieval(
        &self,
        _context: MemoryOpenApiRequestContext,
        _request: MemoryRetrievalRequest,
    ) -> MemoryServiceResult<MemoryRetrievalResult> {
        Err(MemoryServiceError::not_implemented("retrievals.create"))
    }

    async fn retrieve_retrieval(
        &self,
        _context: MemoryOpenApiRequestContext,
        _retrieval_id: u64,
    ) -> MemoryServiceResult<MemoryRetrievalResult> {
        Err(MemoryServiceError::not_implemented("retrievals.retrieve"))
    }

    async fn create_context_pack(
        &self,
        _context: MemoryOpenApiRequestContext,
        _request: MemoryContextPackRequest,
    ) -> MemoryServiceResult<MemoryContextPack> {
        Err(MemoryServiceError::not_implemented("contextPacks.create"))
    }

    async fn retrieve_context_pack(
        &self,
        _context: MemoryOpenApiRequestContext,
        _context_pack_id: u64,
    ) -> MemoryServiceResult<MemoryContextPack> {
        Err(MemoryServiceError::not_implemented("contextPacks.retrieve"))
    }

    async fn create_feedback(
        &self,
        _context: MemoryOpenApiRequestContext,
        _request: MemoryFeedbackRequest,
    ) -> MemoryServiceResult<MemoryFeedback> {
        Err(MemoryServiceError::not_implemented("feedback.create"))
    }

    async fn create_extraction(
        &self,
        _context: MemoryOpenApiRequestContext,
        _request: MemoryExtractionRequest,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        Err(MemoryServiceError::not_implemented("extractions.create"))
    }

    async fn list_candidates(
        &self,
        _context: MemoryOpenApiRequestContext,
        _query: ListCandidatesQuery,
    ) -> MemoryServiceResult<MemoryCandidateList> {
        Err(MemoryServiceError::not_implemented("candidates.list"))
    }

    async fn retrieve_candidate(
        &self,
        _context: MemoryOpenApiRequestContext,
        _candidate_id: u64,
    ) -> MemoryServiceResult<MemoryCandidate> {
        Err(MemoryServiceError::not_implemented("candidates.retrieve"))
    }

    async fn retrieve_provider_health(
        &self,
        _context: MemoryOpenApiRequestContext,
    ) -> MemoryServiceResult<MemoryProviderHealth> {
        Err(MemoryServiceError::not_implemented(
            "providerHealth.retrieve",
        ))
    }
}
