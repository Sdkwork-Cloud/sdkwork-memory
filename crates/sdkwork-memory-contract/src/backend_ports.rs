use async_trait::async_trait;

use crate::admin_dto::{
    ListAdminResourcesQuery, MemoryAuditLogList, MemoryEvalRun, MemoryEvalRunList,
    MemoryEvalRunRequest, MemoryImplementationProfile, MemoryImplementationProfileList,
    MemoryImplementationProfileRequest, MemoryIndex, MemoryIndexList, MemoryIndexRequest,
    MemoryMigrationJobRequest, MemoryProviderBindingList, MemoryProviderBindingRequest,
    MemoryRetentionJobRequest, MemoryRetrievalProfile, MemoryRetrievalProfileList,
    MemoryRetrievalProfileRequest,
};
use crate::dto::{
    ListAuditLogsQuery, ListCandidatesQuery, ListEventsQuery, ListMemoriesQuery,
    ListRetrievalTracesQuery, MemoryCandidate, MemoryCandidateList, MemoryEvent, MemoryEventList,
    MemoryExtractionRequest, MemoryLearningJob, MemoryProviderHealth, MemoryRecord,
    MemoryRecordList, MemoryRecordPatch, MemoryRecordRequest, MemoryRetrievalTrace,
    MemoryRetrievalTraceList, MemoryReviewRequest,
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
        _space_id: u64,
    ) -> MemoryServiceResult<MemoryRecord> {
        backend_not_implemented!("memories.retrieve", MemoryRecord)
    }

    async fn update_memory(
        &self,
        _context: MemoryBackendRequestContext,
        _memory_id: u64,
        _space_id: u64,
        _patch: MemoryRecordPatch,
    ) -> MemoryServiceResult<MemoryRecord> {
        backend_not_implemented!("memories.update", MemoryRecord)
    }

    async fn supersede_memory(
        &self,
        _context: MemoryBackendRequestContext,
        _memory_id: u64,
        _request: MemoryRecordRequest,
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
        _space_id: u64,
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
        _request: MemoryReviewRequest,
    ) -> MemoryServiceResult<MemoryCandidate> {
        backend_not_implemented!("candidates.approve", MemoryCandidate)
    }

    async fn reject_candidate(
        &self,
        _context: MemoryBackendRequestContext,
        _candidate_id: u64,
        _request: MemoryReviewRequest,
    ) -> MemoryServiceResult<MemoryCandidate> {
        backend_not_implemented!("candidates.reject", MemoryCandidate)
    }

    async fn create_extraction_job(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryExtractionRequest,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        backend_not_implemented!("extractionJobs.create", MemoryLearningJob)
    }

    async fn retrieve_extraction_job(
        &self,
        _context: MemoryBackendRequestContext,
        _job_id: u64,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        backend_not_implemented!("extractionJobs.retrieve", MemoryLearningJob)
    }

    async fn create_consolidation_job(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryExtractionRequest,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        backend_not_implemented!("consolidationJobs.create", MemoryLearningJob)
    }

    async fn list_indexes(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListAdminResourcesQuery,
    ) -> MemoryServiceResult<MemoryIndexList> {
        backend_not_implemented!("indexes.list", MemoryIndexList)
    }

    async fn create_index(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryIndexRequest,
    ) -> MemoryServiceResult<MemoryIndex> {
        backend_not_implemented!("indexes.create", MemoryIndex)
    }

    async fn retrieve_index(
        &self,
        _context: MemoryBackendRequestContext,
        _index_id: u64,
    ) -> MemoryServiceResult<MemoryIndex> {
        backend_not_implemented!("indexes.retrieve", MemoryIndex)
    }

    async fn update_index(
        &self,
        _context: MemoryBackendRequestContext,
        _index_id: u64,
        _request: MemoryIndexRequest,
    ) -> MemoryServiceResult<MemoryIndex> {
        backend_not_implemented!("indexes.update", MemoryIndex)
    }

    async fn rebuild_index(
        &self,
        _context: MemoryBackendRequestContext,
        _index_id: u64,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        backend_not_implemented!("indexes.rebuild", MemoryLearningJob)
    }

    async fn list_retrieval_profiles(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListAdminResourcesQuery,
    ) -> MemoryServiceResult<MemoryRetrievalProfileList> {
        backend_not_implemented!("retrievalProfiles.list", MemoryRetrievalProfileList)
    }

    async fn create_retrieval_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryRetrievalProfileRequest,
    ) -> MemoryServiceResult<MemoryRetrievalProfile> {
        backend_not_implemented!("retrievalProfiles.create", MemoryRetrievalProfile)
    }

    async fn retrieve_retrieval_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _profile_id: u64,
    ) -> MemoryServiceResult<MemoryRetrievalProfile> {
        backend_not_implemented!("retrievalProfiles.retrieve", MemoryRetrievalProfile)
    }

    async fn update_retrieval_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _profile_id: u64,
        _request: MemoryRetrievalProfileRequest,
    ) -> MemoryServiceResult<MemoryRetrievalProfile> {
        backend_not_implemented!("retrievalProfiles.update", MemoryRetrievalProfile)
    }

    async fn list_implementation_profiles(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListAdminResourcesQuery,
    ) -> MemoryServiceResult<MemoryImplementationProfileList> {
        backend_not_implemented!("implementationProfiles.list", MemoryImplementationProfileList)
    }

    async fn create_implementation_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryImplementationProfileRequest,
    ) -> MemoryServiceResult<MemoryImplementationProfile> {
        backend_not_implemented!("implementationProfiles.create", MemoryImplementationProfile)
    }

    async fn retrieve_implementation_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _profile_id: u64,
    ) -> MemoryServiceResult<MemoryImplementationProfile> {
        backend_not_implemented!("implementationProfiles.retrieve", MemoryImplementationProfile)
    }

    async fn update_implementation_profile(
        &self,
        _context: MemoryBackendRequestContext,
        _profile_id: u64,
        _request: MemoryImplementationProfileRequest,
    ) -> MemoryServiceResult<MemoryImplementationProfile> {
        backend_not_implemented!("implementationProfiles.update", MemoryImplementationProfile)
    }

    async fn list_provider_bindings(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListAdminResourcesQuery,
    ) -> MemoryServiceResult<MemoryProviderBindingList> {
        backend_not_implemented!("providerBindings.list", MemoryProviderBindingList)
    }

    async fn create_provider_binding(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryProviderBindingRequest,
    ) -> MemoryServiceResult<crate::dto::MemoryProviderBinding> {
        backend_not_implemented!("providerBindings.create", crate::dto::MemoryProviderBinding)
    }

    async fn update_provider_binding(
        &self,
        _context: MemoryBackendRequestContext,
        _provider_binding_id: u64,
        _request: MemoryProviderBindingRequest,
    ) -> MemoryServiceResult<crate::dto::MemoryProviderBinding> {
        backend_not_implemented!("providerBindings.update", crate::dto::MemoryProviderBinding)
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
        _query: ListAdminResourcesQuery,
    ) -> MemoryServiceResult<MemoryEvalRunList> {
        backend_not_implemented!("evalRuns.list", MemoryEvalRunList)
    }

    async fn create_eval_run(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryEvalRunRequest,
    ) -> MemoryServiceResult<MemoryEvalRun> {
        backend_not_implemented!("evalRuns.create", MemoryEvalRun)
    }

    async fn retrieve_eval_run(
        &self,
        _context: MemoryBackendRequestContext,
        _eval_run_id: u64,
    ) -> MemoryServiceResult<MemoryEvalRun> {
        backend_not_implemented!("evalRuns.retrieve", MemoryEvalRun)
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
    ) -> MemoryServiceResult<MemoryRetrievalTrace> {
        backend_not_implemented!("retrievalTraces.retrieve", MemoryRetrievalTrace)
    }

    async fn list_audit_logs(
        &self,
        _context: MemoryBackendRequestContext,
        _query: ListAuditLogsQuery,
    ) -> MemoryServiceResult<MemoryAuditLogList> {
        backend_not_implemented!("auditLogs.list", MemoryAuditLogList)
    }

    async fn create_retention_job(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryRetentionJobRequest,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        backend_not_implemented!("retentionJobs.create", MemoryLearningJob)
    }

    async fn create_migration_job(
        &self,
        _context: MemoryBackendRequestContext,
        _request: MemoryMigrationJobRequest,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        backend_not_implemented!("migrationJobs.create", MemoryLearningJob)
    }

    async fn retrieve_migration_job(
        &self,
        _context: MemoryBackendRequestContext,
        _migration_job_id: u64,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        backend_not_implemented!("migrationJobs.retrieve", MemoryLearningJob)
    }
}
