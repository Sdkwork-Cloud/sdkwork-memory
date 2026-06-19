use async_trait::async_trait;

use crate::dto::{
    ListCandidatesQuery, ListHabitsQuery, ListMemoriesQuery, MemoryCandidate, MemoryCandidateList,
    MemoryContextPack, MemoryContextPackRequest, MemoryEvent, MemoryEventRequest, MemoryExportJob,
    MemoryExportRequest, MemoryExtractionRequest, MemoryFeedback, MemoryFeedbackRequest,
    MemoryForgetJob, MemoryForgetRequest, MemoryHabit, MemoryHabitList, MemoryHabitRequest,
    MemoryLearningJob, MemoryLearningSettings, MemoryLearningSettingsPatch, MemoryRecord,
    MemoryRecordList, MemoryRecordPatch, MemoryRecordRequest, MemoryRecordSourceList,
    MemoryRetrievalRequest, MemoryRetrievalResult, MemoryReviewRequest,
};
use crate::ports::{MemoryServiceError, MemoryServiceResult};
use crate::space::{ListSpacesQuery, MemorySpace, MemorySpaceList, MemorySpaceRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAppRequestContext {
    pub tenant_id: u64,
    pub actor_id: Option<u64>,
    pub organization_id: Option<u64>,
    pub session_id: Option<String>,
}

macro_rules! app_not_implemented {
    ($name:literal, $ret:ty) => {
        Err(MemoryServiceError::not_implemented($name)) as MemoryServiceResult<$ret>
    };
}

#[async_trait]
pub trait MemoryAppApi: Send + Sync + 'static {
    async fn list_spaces(
        &self,
        _context: MemoryAppRequestContext,
        _query: ListSpacesQuery,
    ) -> MemoryServiceResult<MemorySpaceList> {
        app_not_implemented!("spaces.list", MemorySpaceList)
    }

    async fn create_space(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemorySpaceRequest,
    ) -> MemoryServiceResult<MemorySpace> {
        app_not_implemented!("spaces.create", MemorySpace)
    }

    async fn retrieve_space(
        &self,
        _context: MemoryAppRequestContext,
        _space_id: u64,
    ) -> MemoryServiceResult<MemorySpace> {
        app_not_implemented!("spaces.retrieve", MemorySpace)
    }

    async fn update_space(
        &self,
        _context: MemoryAppRequestContext,
        _space_id: u64,
        _request: MemorySpaceRequest,
    ) -> MemoryServiceResult<MemorySpace> {
        app_not_implemented!("spaces.update", MemorySpace)
    }

    async fn create_event(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemoryEventRequest,
    ) -> MemoryServiceResult<MemoryEvent> {
        app_not_implemented!("events.create", MemoryEvent)
    }

    async fn retrieve_event(
        &self,
        _context: MemoryAppRequestContext,
        _event_id: u64,
    ) -> MemoryServiceResult<MemoryEvent> {
        app_not_implemented!("events.retrieve", MemoryEvent)
    }

    async fn list_memories(
        &self,
        _context: MemoryAppRequestContext,
        _query: ListMemoriesQuery,
    ) -> MemoryServiceResult<MemoryRecordList> {
        app_not_implemented!("memories.list", MemoryRecordList)
    }

    async fn create_memory(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemoryRecordRequest,
    ) -> MemoryServiceResult<MemoryRecord> {
        app_not_implemented!("memories.create", MemoryRecord)
    }

    async fn retrieve_memory(
        &self,
        _context: MemoryAppRequestContext,
        _memory_id: u64,
    ) -> MemoryServiceResult<MemoryRecord> {
        app_not_implemented!("memories.retrieve", MemoryRecord)
    }

    async fn update_memory(
        &self,
        _context: MemoryAppRequestContext,
        _memory_id: u64,
        _patch: MemoryRecordPatch,
    ) -> MemoryServiceResult<MemoryRecord> {
        app_not_implemented!("memories.update", MemoryRecord)
    }

    async fn delete_memory(
        &self,
        _context: MemoryAppRequestContext,
        _memory_id: u64,
    ) -> MemoryServiceResult<()> {
        app_not_implemented!("memories.delete", ())
    }

    async fn list_memory_sources(
        &self,
        _context: MemoryAppRequestContext,
        _memory_id: u64,
    ) -> MemoryServiceResult<MemoryRecordSourceList> {
        app_not_implemented!("memories.sources.list", MemoryRecordSourceList)
    }

    async fn create_forget_request(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemoryForgetRequest,
    ) -> MemoryServiceResult<MemoryForgetJob> {
        app_not_implemented!("forgetRequests.create", MemoryForgetJob)
    }

    async fn retrieve_forget_request(
        &self,
        _context: MemoryAppRequestContext,
        _forget_request_id: u64,
    ) -> MemoryServiceResult<MemoryForgetJob> {
        app_not_implemented!("forgetRequests.retrieve", MemoryForgetJob)
    }

    async fn create_extraction(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemoryExtractionRequest,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        app_not_implemented!("extractions.create", MemoryLearningJob)
    }

    async fn list_candidates(
        &self,
        _context: MemoryAppRequestContext,
        _query: ListCandidatesQuery,
    ) -> MemoryServiceResult<MemoryCandidateList> {
        app_not_implemented!("candidates.list", MemoryCandidateList)
    }

    async fn retrieve_candidate(
        &self,
        _context: MemoryAppRequestContext,
        _candidate_id: u64,
    ) -> MemoryServiceResult<MemoryCandidate> {
        app_not_implemented!("candidates.retrieve", MemoryCandidate)
    }

    async fn approve_candidate(
        &self,
        _context: MemoryAppRequestContext,
        _candidate_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<MemoryCandidate> {
        app_not_implemented!("candidates.approve", MemoryCandidate)
    }

    async fn reject_candidate(
        &self,
        _context: MemoryAppRequestContext,
        _candidate_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<MemoryCandidate> {
        app_not_implemented!("candidates.reject", MemoryCandidate)
    }

    async fn list_habits(
        &self,
        _context: MemoryAppRequestContext,
        _query: ListHabitsQuery,
    ) -> MemoryServiceResult<MemoryHabitList> {
        app_not_implemented!("habits.list", MemoryHabitList)
    }

    async fn retrieve_habit(
        &self,
        _context: MemoryAppRequestContext,
        _habit_id: u64,
    ) -> MemoryServiceResult<MemoryHabit> {
        app_not_implemented!("habits.retrieve", MemoryHabit)
    }

    async fn update_habit(
        &self,
        _context: MemoryAppRequestContext,
        _habit_id: u64,
        _request: MemoryHabitRequest,
    ) -> MemoryServiceResult<MemoryHabit> {
        app_not_implemented!("habits.update", MemoryHabit)
    }

    async fn confirm_habit(
        &self,
        _context: MemoryAppRequestContext,
        _habit_id: u64,
        _request: MemoryReviewRequest,
    ) -> MemoryServiceResult<MemoryHabit> {
        app_not_implemented!("habits.confirm", MemoryHabit)
    }

    async fn reject_habit(
        &self,
        _context: MemoryAppRequestContext,
        _habit_id: u64,
        _request: MemoryReviewRequest,
    ) -> MemoryServiceResult<MemoryHabit> {
        app_not_implemented!("habits.reject", MemoryHabit)
    }

    async fn create_retrieval(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemoryRetrievalRequest,
    ) -> MemoryServiceResult<MemoryRetrievalResult> {
        app_not_implemented!("retrievals.create", MemoryRetrievalResult)
    }

    async fn retrieve_retrieval(
        &self,
        _context: MemoryAppRequestContext,
        _retrieval_id: u64,
    ) -> MemoryServiceResult<MemoryRetrievalResult> {
        app_not_implemented!("retrievals.retrieve", MemoryRetrievalResult)
    }

    async fn create_context_pack(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemoryContextPackRequest,
    ) -> MemoryServiceResult<MemoryContextPack> {
        app_not_implemented!("contextPacks.create", MemoryContextPack)
    }

    async fn retrieve_context_pack(
        &self,
        _context: MemoryAppRequestContext,
        _context_pack_id: u64,
    ) -> MemoryServiceResult<MemoryContextPack> {
        app_not_implemented!("contextPacks.retrieve", MemoryContextPack)
    }

    async fn create_feedback(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemoryFeedbackRequest,
    ) -> MemoryServiceResult<MemoryFeedback> {
        app_not_implemented!("feedback.create", MemoryFeedback)
    }

    async fn create_export_job(
        &self,
        _context: MemoryAppRequestContext,
        _request: MemoryExportRequest,
    ) -> MemoryServiceResult<MemoryExportJob> {
        app_not_implemented!("exportJobs.create", MemoryExportJob)
    }

    async fn retrieve_export_job(
        &self,
        _context: MemoryAppRequestContext,
        _export_job_id: u64,
    ) -> MemoryServiceResult<MemoryExportJob> {
        app_not_implemented!("exportJobs.retrieve", MemoryExportJob)
    }

    async fn retrieve_learning_settings(
        &self,
        _context: MemoryAppRequestContext,
    ) -> MemoryServiceResult<MemoryLearningSettings> {
        app_not_implemented!("learningSettings.retrieve", MemoryLearningSettings)
    }

    async fn update_learning_settings(
        &self,
        _context: MemoryAppRequestContext,
        _patch: MemoryLearningSettingsPatch,
    ) -> MemoryServiceResult<MemoryLearningSettings> {
        app_not_implemented!("learningSettings.update", MemoryLearningSettings)
    }
}
