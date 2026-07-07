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
    /// Backend operators authorized at the router layer may access all tenant spaces.
    pub elevated_tenant_access: bool,
}

impl MemoryOpenApiRequestContext {
    pub fn for_open_surface(
        api_key_id: impl Into<String>,
        tenant_id: u64,
        actor_id: Option<u64>,
    ) -> Self {
        Self {
            api_key_id: api_key_id.into(),
            tenant_id,
            actor_id,
            elevated_tenant_access: false,
        }
    }

    pub fn for_backend_surface(tenant_id: u64, operator_id: Option<u64>) -> Self {
        Self {
            api_key_id: format!("backend-{}", operator_id.unwrap_or(0)),
            tenant_id,
            actor_id: operator_id,
            elevated_tenant_access: true,
        }
    }

    /// Non-elevated context for background workers acting on behalf of a tenant actor.
    pub fn for_background_job(tenant_id: u64, actor_id: Option<u64>) -> Self {
        Self {
            api_key_id: "background-job".to_string(),
            tenant_id,
            actor_id,
            elevated_tenant_access: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryServiceErrorKind {
    NotFound,
    Conflict,
    Validation,
    Forbidden,
    QuotaExceeded,
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

    pub fn quota_exceeded(detail: impl Into<String>) -> Self {
        Self {
            kind: MemoryServiceErrorKind::QuotaExceeded,
            code: "quota_exceeded".to_string(),
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
        context: MemoryOpenApiRequestContext,
    ) -> MemoryServiceResult<MemoryCapabilities>;

    async fn create_event(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryEventRequest,
    ) -> MemoryServiceResult<MemoryEvent>;

    async fn retrieve_event(
        &self,
        context: MemoryOpenApiRequestContext,
        event_id: u64,
        space_id: u64,
    ) -> MemoryServiceResult<MemoryEvent>;

    async fn list_memories(
        &self,
        context: MemoryOpenApiRequestContext,
        query: ListMemoriesQuery,
    ) -> MemoryServiceResult<MemoryRecordList>;

    async fn create_memory(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryRecordRequest,
    ) -> MemoryServiceResult<MemoryRecord>;

    async fn retrieve_memory(
        &self,
        context: MemoryOpenApiRequestContext,
        memory_id: u64,
        space_id: u64,
    ) -> MemoryServiceResult<MemoryRecord>;

    async fn update_memory(
        &self,
        context: MemoryOpenApiRequestContext,
        memory_id: u64,
        space_id: u64,
        patch: MemoryRecordPatch,
    ) -> MemoryServiceResult<MemoryRecord>;

    async fn delete_memory(
        &self,
        context: MemoryOpenApiRequestContext,
        memory_id: u64,
        space_id: u64,
    ) -> MemoryServiceResult<()>;

    async fn create_retrieval(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryRetrievalRequest,
    ) -> MemoryServiceResult<MemoryRetrievalResult>;

    async fn retrieve_retrieval(
        &self,
        context: MemoryOpenApiRequestContext,
        retrieval_id: u64,
    ) -> MemoryServiceResult<MemoryRetrievalResult>;

    async fn create_context_pack(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryContextPackRequest,
    ) -> MemoryServiceResult<MemoryContextPack>;

    async fn retrieve_context_pack(
        &self,
        context: MemoryOpenApiRequestContext,
        context_pack_id: u64,
    ) -> MemoryServiceResult<MemoryContextPack>;

    async fn create_feedback(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryFeedbackRequest,
    ) -> MemoryServiceResult<MemoryFeedback>;

    async fn create_extraction(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryExtractionRequest,
    ) -> MemoryServiceResult<MemoryLearningJob>;

    async fn list_candidates(
        &self,
        context: MemoryOpenApiRequestContext,
        query: ListCandidatesQuery,
    ) -> MemoryServiceResult<MemoryCandidateList>;

    async fn retrieve_candidate(
        &self,
        context: MemoryOpenApiRequestContext,
        candidate_id: u64,
    ) -> MemoryServiceResult<MemoryCandidate>;

    async fn retrieve_provider_health(
        &self,
        context: MemoryOpenApiRequestContext,
    ) -> MemoryServiceResult<MemoryProviderHealth>;
}
