use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use sdkwork_memory_contract::{
    ListCandidatesQuery, ListMemoriesQuery, MemoryContextPackRequest, MemoryEventRequest,
    MemoryExtractionRequest, MemoryFeedbackRequest, MemoryOpenApi, MemoryOpenApiRequestContext,
    MemoryRecordPatch, MemoryRecordRequest, MemoryRetrievalRequest, MemorySpaceScopeQuery,
};
use sdkwork_routes_memory_support::{
    created_resource_json, no_content_json, ok_page_json, ok_resource_json,
};
use std::sync::Arc;

use crate::{auth::require_context, paths, ApiProblem};

#[derive(Clone)]
struct OpenState {
    api: Arc<dyn MemoryOpenApi>,
}

pub fn build_router_with_open_api<A>(api: A) -> Router
where
    A: MemoryOpenApi,
{
    build_router_with_shared_open_api(Arc::new(api))
}

pub fn build_router_with_shared_open_api(api: Arc<dyn MemoryOpenApi>) -> Router {
    Router::new()
        .route(paths::CAPABILITIES, get(retrieve_capabilities))
        .route(paths::EVENTS, post(create_event))
        .route(paths::EVENT, get(retrieve_event))
        .route(paths::MEMORIES, get(list_memories).post(create_memory))
        .route(
            paths::MEMORY,
            get(retrieve_memory)
                .patch(update_memory)
                .delete(delete_memory),
        )
        .route(paths::RETRIEVALS, post(create_retrieval))
        .route(paths::RETRIEVAL, get(retrieve_retrieval))
        .route(paths::CONTEXT_PACKS, post(create_context_pack))
        .route(paths::CONTEXT_PACK, get(retrieve_context_pack))
        .route(paths::FEEDBACK, post(create_feedback))
        .route(paths::EXTRACTIONS, post(create_extraction))
        .route(paths::CANDIDATES, get(list_candidates))
        .route(paths::CANDIDATE, get(retrieve_candidate))
        .route(paths::PROVIDER_HEALTH, get(retrieve_provider_health))
        .with_state(OpenState { api })
}

async fn retrieve_capabilities(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_resource_json(state.api.retrieve_capabilities(context).await)
}

async fn create_event(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Json(request): Json<MemoryEventRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    created_resource_json(state.api.create_event(context, request).await)
}

async fn retrieve_event(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(event_id): Path<u64>,
    Query(scope): Query<MemorySpaceScopeQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_resource_json(
        state
            .api
            .retrieve_event(context, event_id, scope.space_id)
            .await,
    )
}

async fn list_memories(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Query(query): Query<ListMemoriesQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_page_json(state.api.list_memories(context, query).await)
}

async fn create_memory(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Json(request): Json<MemoryRecordRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    created_resource_json(state.api.create_memory(context, request).await)
}

async fn retrieve_memory(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(memory_id): Path<u64>,
    Query(scope): Query<MemorySpaceScopeQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_resource_json(
        state
            .api
            .retrieve_memory(context, memory_id, scope.space_id)
            .await,
    )
}

async fn update_memory(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(memory_id): Path<u64>,
    Query(scope): Query<MemorySpaceScopeQuery>,
    Json(patch): Json<MemoryRecordPatch>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_resource_json(
        state
            .api
            .update_memory(context, memory_id, scope.space_id, patch)
            .await,
    )
}

async fn delete_memory(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(memory_id): Path<u64>,
    Query(scope): Query<MemorySpaceScopeQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    no_content_json(
        state
            .api
            .delete_memory(context, memory_id, scope.space_id)
            .await,
    )
}

async fn create_retrieval(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Json(request): Json<MemoryRetrievalRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    created_resource_json(state.api.create_retrieval(context, request).await)
}

async fn retrieve_retrieval(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(retrieval_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_resource_json(state.api.retrieve_retrieval(context, retrieval_id).await)
}

async fn create_context_pack(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Json(request): Json<MemoryContextPackRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    created_resource_json(state.api.create_context_pack(context, request).await)
}

async fn retrieve_context_pack(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(context_pack_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_resource_json(
        state
            .api
            .retrieve_context_pack(context, context_pack_id)
            .await,
    )
}

async fn create_feedback(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Json(request): Json<MemoryFeedbackRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    created_resource_json(state.api.create_feedback(context, request).await)
}

async fn create_extraction(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Json(request): Json<MemoryExtractionRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    created_resource_json(state.api.create_extraction(context, request).await)
}

async fn list_candidates(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Query(query): Query<ListCandidatesQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_page_json(state.api.list_candidates(context, query).await)
}

async fn retrieve_candidate(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(candidate_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_resource_json(state.api.retrieve_candidate(context, candidate_id).await)
}

async fn retrieve_provider_health(
    State(state): State<OpenState>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    ok_resource_json(state.api.retrieve_provider_health(context).await)
}
