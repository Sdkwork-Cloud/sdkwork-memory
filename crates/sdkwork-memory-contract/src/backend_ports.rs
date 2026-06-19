use async_trait::async_trait;

use crate::dto::{
    ListAuditLogsQuery, ListCandidatesQuery, ListEventsQuery, ListMemoriesQuery,
    ListRetrievalTracesQuery, MemoryCandidate, MemoryCandidateList, MemoryEvent, MemoryEventList,
    MemoryProviderHealth, MemoryRecord, MemoryRecordList, MemoryRecordPatch,
    MemoryRetrievalTraceList,
};
use crate::ports::{MemoryServiceError, MemoryServiceResult};
use crate::space::{ListSpacesQuery, MemorySpace, MemorySpaceList, MemorySpaceRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryBackendRequestContext {
    pub tenant_id: u64,
    pub operator_id: Option<u64>,
}

macro_rules! backend_not_implemented {
    ($name:literal, $ret:ty) => {
        Err(MemoryServiceError::not_implemented($name)) as MemoryServiceResult<$ret>
    };
}

#[async_trait]
pub trait MemoryBackendApi: Send + Sync + 'static {
    async fn list_spaces(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListSpacesQuery,
    ) -> MemoryServiceResult<MemorySpaceList> {
        backend_not_implemented!("spaces.list", MemorySpaceList)
    }

    async fn retrieve_space(
        &self,
        _context: MemoryBackendRequestContext,
        _space_id: u64,
    ) -> MemoryServiceResult<MemorySpace> {
        backend_not_implemented!("spaces.retrieve", MemorySpace)
    }

    async fn update_space(
        &self,
        _context: MemoryBackendRequestContext,
        _space_id: u64,
        _request: MemorySpaceRequest,
    ) -> MemoryServiceResult<MemorySpace> {
        backend_not_implemented!("spaces.update", MemorySpace)
    }

    async fn list_memories(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListMemoriesQuery,
    ) -> MemoryServiceResult<MemoryRecordList> {
        backend_not_implemented!("memories.list", MemoryRecordList)
    }

    async fn retrieve_memory(
        &self,
        _context: MemoryBackendRequestContext,
        _memory_id: u64,
    ) -> MemoryServiceResult<MemoryRecord> {
        backend_not_implemented!("memories.retrieve", MemoryRecord)
    }

    async fn update_memory(
        &self,
        _context: MemoryBackendRequestContext,
        _memory_id: u64,
        _patch: MemoryRecordPatch,
    ) -> MemoryServiceResult<MemoryRecord> {
        backend_not_implemented!("memories.update", MemoryRecord)
    }

    async fn supersede_memory(
        &self,
        _context: MemoryBackendRequestContext,
        _memory_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<MemoryRecord> {
        backend_not_implemented!("memories.supersede", MemoryRecord)
    }

    async fn list_events(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListEventsQuery,
    ) -> MemoryServiceResult<MemoryEventList> {
        backend_not_implemented!("events.list", MemoryEventList)
    }

    async fn retrieve_event(
        &self,
        _context: MemoryBackendRequestContext,
        _event_id: u64,
    ) -> MemoryServiceResult<MemoryEvent> {
        backend_not_implemented!("events.retrieve", MemoryEvent)
    }

    async fn list_candidates(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListCandidatesQuery,
    ) -> MemoryServiceResult<MemoryCandidateList> {
        backend_not_implemented!("candidates.list", MemoryCandidateList)
    }

    async fn approve_candidate(
        &self,
        _context: MemoryBackendRequestContext,
        _candidate_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<MemoryCandidate> {
        backend_not_implemented!("candidates.approve", MemoryCandidate)
    }

    async fn reject_candidate(
        &self,
        _context: MemoryBackendRequestContext,
        _candidate_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<MemoryCandidate> {
        backend_not_implemented!("candidates.reject", MemoryCandidate)
    }

    async fn create_extraction_job(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("extractionJobs.create", serde_json::Value)
    }

    async fn retrieve_extraction_job(
        &self,
        _context: MemoryBackendRequestContext,
        _job_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("extractionJobs.retrieve", serde_json::Value)
    }

    async fn create_consolidation_job(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("consolidationJobs.create", serde_json::Value)
    }

    async fn list_indexes(
        &self,
        _context: MemoryBackendRequestContext,
        _query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("indexes.list", serde_json::Value)
    }

    async fn create_index(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("indexes.create", serde_json::Value)
    }

    async fn retrieve_index(
        &self,
        _context: MemoryBackendRequestContext,
        _index_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("indexes.retrieve", serde_json::Value)
    }

    async fn update_index(
        &self,
        _context: MemoryBackendRequestContext,
        _index_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("indexes.update", serde_json::Value)
    }

    async fn rebuild_index(
        &self,
        _context: MemoryBackendRequestContext,
        _index_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("indexes.rebuild", serde_json::Value)
    }

    async fn list_retrieval_profiles(
        &self,
        _context: MemoryBackendRequestContext,
        _query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("retrievalProfiles.list", serde_json::Value)
    }

    async fn create_retrieval_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("retrievalProfiles.create", serde_json::Value)
    }

    async fn retrieve_retrieval_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _profile_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("retrievalProfiles.retrieve", serde_json::Value)
    }

    async fn update_retrieval_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _profile_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("retrievalProfiles.update", serde_json::Value)
    }

    async fn list_implementation_profiles(
        &self,
        _context: MemoryBackendRequestContext,
        _query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("implementationProfiles.list", serde_json::Value)
    }

    async fn create_implementation_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("implementationProfiles.create", serde_json::Value)
    }

    async fn retrieve_implementation_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _profile_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("implementationProfiles.retrieve", serde_json::Value)
    }

    async fn update_implementation_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _profile_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("implementationProfiles.update", serde_json::Value)
    }

    async fn list_provider_bindings(
        &self,
        _context: MemoryBackendRequestContext,
        _query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("providerBindings.list", serde_json::Value)
    }

    async fn create_provider_binding(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("providerBindings.create", serde_json::Value)
    }

    async fn update_provider_binding(
        &self,
        _context: MemoryBackendRequestContext,
        _provider_binding_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("providerBindings.update", serde_json::Value)
    }

    async fn retrieve_provider_health(
        &self,
        _context: MemoryBackendRequestContext,
    ) -> MemoryServiceResult<MemoryProviderHealth> {
        backend_not_implemented!("providerHealth.retrieve", MemoryProviderHealth)
    }

    async fn list_eval_runs(
        &self,
        _context: MemoryBackendRequestContext,
        _query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("evalRuns.list", serde_json::Value)
    }

    async fn create_eval_run(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("evalRuns.create", serde_json::Value)
    }

    async fn retrieve_eval_run(
        &self,
        _context: MemoryBackendRequestContext,
        _eval_run_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("evalRuns.retrieve", serde_json::Value)
    }

    async fn list_retrieval_traces(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListRetrievalTracesQuery,
    ) -> MemoryServiceResult<MemoryRetrievalTraceList> {
        backend_not_implemented!("retrievalTraces.list", MemoryRetrievalTraceList)
    }

    async fn retrieve_retrieval_trace(
        &self,
        _context: MemoryBackendRequestContext,
        _trace_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("retrievalTraces.retrieve", serde_json::Value)
    }

    async fn list_audit_logs(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListAuditLogsQuery,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("auditLogs.list", serde_json::Value)
    }

    async fn create_retention_job(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("retentionJobs.create", serde_json::Value)
    }

    async fn create_migration_job(
        &self,
        _context: MemoryBackendRequestContext,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("migrationJobs.create", serde_json::Value)
    }

    async fn retrieve_migration_job(
        &self,
        _context: MemoryBackendRequestContext,
        _migration_job_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        backend_not_implemented!("migrationJobs.retrieve", serde_json::Value)
    }
}
