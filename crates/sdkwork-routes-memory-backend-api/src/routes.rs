use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use sdkwork_memory_contract::{
    ListAdminResourcesQuery, ListAuditLogsQuery, ListCandidatesQuery, ListEventsQuery,
    ListMemoriesQuery, ListRetrievalTracesQuery, ListSpacesQuery, MemoryBackendApi,
    MemoryBackendRequestContext, MemoryEvalRunRequest, MemoryExtractionRequest,
    MemoryImplementationProfileRequest, MemoryIndexRequest, MemoryMigrationJobRequest,
    MemoryProviderBindingRequest, MemoryRecordPatch, MemoryRecordRequest,
    MemoryRetentionJobRequest, MemoryRetrievalProfileRequest, MemoryReviewRequest,
    MemoryServiceResult, MemorySpaceRequest, MemorySpaceScopeQuery,
};
use std::sync::Arc;

use crate::{auth::require_backend_context, paths, BackendApiProblem};

#[derive(Clone)]
struct BackendState {
    api: Arc<dyn MemoryBackendApi>,
}

pub fn build_router_with_backend_api<A>(api: A) -> Router
where
    A: MemoryBackendApi,
{
    build_router_with_shared_backend_api(Arc::new(api))
}

pub fn build_router_with_shared_backend_api(api: Arc<dyn MemoryBackendApi>) -> Router {
    Router::new()
        .route(paths::SPACES, get(list_spaces))
        .route(paths::SPACE, get(retrieve_space).patch(update_space))
        .route(paths::MEMORIES, get(list_memories))
        .route(paths::MEMORY, get(retrieve_memory).patch(update_memory))
        .route(paths::MEMORY_SUPERSEDE, post(supersede_memory))
        .route(paths::EVENTS, get(list_events))
        .route(paths::EVENT, get(retrieve_event))
        .route(paths::CANDIDATES, get(list_candidates))
        .route(paths::CANDIDATE_APPROVE, post(approve_candidate))
        .route(paths::CANDIDATE_REJECT, post(reject_candidate))
        .route(paths::EXTRACTION_JOBS, post(create_extraction_job))
        .route(paths::EXTRACTION_JOB, get(retrieve_extraction_job))
        .route(paths::CONSOLIDATION_JOBS, post(create_consolidation_job))
        .route(paths::INDEXES, get(list_indexes).post(create_index))
        .route(paths::INDEX, get(retrieve_index).patch(update_index))
        .route(paths::INDEX_REBUILD, post(rebuild_index))
        .route(
            paths::RETRIEVAL_PROFILES,
            get(list_retrieval_profiles).post(create_retrieval_profile),
        )
        .route(
            paths::RETRIEVAL_PROFILE,
            get(retrieve_retrieval_profile).patch(update_retrieval_profile),
        )
        .route(
            paths::IMPLEMENTATION_PROFILES,
            get(list_implementation_profiles).post(create_implementation_profile),
        )
        .route(
            paths::IMPLEMENTATION_PROFILE,
            get(retrieve_implementation_profile).patch(update_implementation_profile),
        )
        .route(
            paths::PROVIDER_BINDINGS,
            get(list_provider_bindings).post(create_provider_binding),
        )
        .route(paths::PROVIDER_BINDING, patch(update_provider_binding))
        .route(paths::PROVIDER_HEALTH, get(retrieve_provider_health))
        .route(paths::EVAL_RUNS, get(list_eval_runs).post(create_eval_run))
        .route(paths::EVAL_RUN, get(retrieve_eval_run))
        .route(paths::RETRIEVAL_TRACES, get(list_retrieval_traces))
        .route(paths::RETRIEVAL_TRACE, get(retrieve_retrieval_trace))
        .route(paths::AUDIT_LOGS, get(list_audit_logs))
        .route(paths::RETENTION_JOBS, post(create_retention_job))
        .route(paths::MIGRATION_JOBS, post(create_migration_job))
        .route(paths::MIGRATION_JOB, get(retrieve_migration_job))
        .with_state(BackendState { api })
        .merge(crate::commercial_routes::commercial_routes())
}

async fn list_spaces(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListSpacesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_spaces(context, query).await)
}

async fn retrieve_space(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(space_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.retrieve_space(context, space_id).await)
}

async fn update_space(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(space_id): Path<u64>,
    Json(request): Json<MemorySpaceRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.update_space(context, space_id, request).await)
}

async fn list_memories(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListMemoriesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_memories(context, query).await)
}

async fn retrieve_memory(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(memory_id): Path<u64>,
    Query(scope): Query<MemorySpaceScopeQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .retrieve_memory(context, memory_id, scope.space_id)
            .await,
    )
}

async fn update_memory(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(memory_id): Path<u64>,
    Query(scope): Query<MemorySpaceScopeQuery>,
    Json(patch): Json<MemoryRecordPatch>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .update_memory(context, memory_id, scope.space_id, patch)
            .await,
    )
}

async fn supersede_memory(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(memory_id): Path<u64>,
    Json(request): Json<MemoryRecordRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .supersede_memory(context, memory_id, request)
            .await,
    )
}

async fn list_events(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_events(context, query).await)
}

async fn retrieve_event(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(event_id): Path<u64>,
    Query(scope): Query<MemorySpaceScopeQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .retrieve_event(context, event_id, scope.space_id)
            .await,
    )
}

async fn list_candidates(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListCandidatesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_candidates(context, query).await)
}

async fn approve_candidate(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<MemoryReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .approve_candidate(context, candidate_id, request)
            .await,
    )
}

async fn reject_candidate(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<MemoryReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .reject_candidate(context, candidate_id, request)
            .await,
    )
}

async fn create_extraction_job(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryExtractionRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_extraction_job(context, request).await)
}

async fn retrieve_extraction_job(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(job_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.retrieve_extraction_job(context, job_id).await)
}

async fn create_consolidation_job(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryExtractionRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_consolidation_job(context, request).await)
}

async fn list_indexes(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListAdminResourcesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_indexes(context, query).await)
}

async fn create_index(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryIndexRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_index(context, request).await)
}

async fn retrieve_index(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(index_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.retrieve_index(context, index_id).await)
}

async fn update_index(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(index_id): Path<u64>,
    Json(request): Json<MemoryIndexRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.update_index(context, index_id, request).await)
}

async fn rebuild_index(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(index_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.rebuild_index(context, index_id).await)
}

async fn list_retrieval_profiles(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListAdminResourcesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_retrieval_profiles(context, query).await)
}

async fn create_retrieval_profile(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_retrieval_profile(context, request).await)
}

async fn retrieve_retrieval_profile(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(profile_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .retrieve_retrieval_profile(context, profile_id)
            .await,
    )
}

async fn update_retrieval_profile(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<MemoryRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .update_retrieval_profile(context, profile_id, request)
            .await,
    )
}

async fn list_implementation_profiles(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListAdminResourcesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_implementation_profiles(context, query).await)
}

async fn create_implementation_profile(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryImplementationProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(
        state
            .api
            .create_implementation_profile(context, request)
            .await,
    )
}

async fn retrieve_implementation_profile(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(implementation_profile_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .retrieve_implementation_profile(context, implementation_profile_id)
            .await,
    )
}

async fn update_implementation_profile(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(implementation_profile_id): Path<u64>,
    Json(request): Json<MemoryImplementationProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .update_implementation_profile(context, implementation_profile_id, request)
            .await,
    )
}

async fn list_provider_bindings(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListAdminResourcesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_provider_bindings(context, query).await)
}

async fn create_provider_binding(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryProviderBindingRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_provider_binding(context, request).await)
}

async fn update_provider_binding(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(provider_binding_id): Path<u64>,
    Json(request): Json<MemoryProviderBindingRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .update_provider_binding(context, provider_binding_id, request)
            .await,
    )
}

async fn retrieve_provider_health(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.retrieve_provider_health(context).await)
}

async fn list_eval_runs(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListAdminResourcesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_eval_runs(context, query).await)
}

async fn create_eval_run(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryEvalRunRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_eval_run(context, request).await)
}

async fn retrieve_eval_run(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(eval_run_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.retrieve_eval_run(context, eval_run_id).await)
}

async fn list_retrieval_traces(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListRetrievalTracesQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_retrieval_traces(context, query).await)
}

async fn retrieve_retrieval_trace(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(trace_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.retrieve_retrieval_trace(context, trace_id).await)
}

async fn list_audit_logs(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(state.api.list_audit_logs(context, query).await)
}

async fn create_retention_job(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryRetentionJobRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_retention_job(context, request).await)
}

async fn create_migration_job(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(request): Json<MemoryMigrationJobRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_migration_job(context, request).await)
}

async fn retrieve_migration_job(
    State(state): State<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(migration_job_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .retrieve_migration_job(context, migration_job_id)
            .await,
    )
}

fn ok_json<T>(result: MemoryServiceResult<T>) -> Result<Response, BackendApiProblem>
where
    T: serde::Serialize,
{
    match result {
        Ok(value) => Ok((StatusCode::OK, Json(value)).into_response()),
        Err(error) => Err(error.into()),
    }
}

fn created_json<T>(result: MemoryServiceResult<T>) -> Result<Response, BackendApiProblem>
where
    T: serde::Serialize,
{
    match result {
        Ok(value) => Ok((StatusCode::CREATED, Json(value)).into_response()),
        Err(error) => Err(error.into()),
    }
}
