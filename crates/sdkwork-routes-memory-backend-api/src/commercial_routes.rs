//! Commercial management route handlers for the backend API.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_contract::{
    CapabilityTargetType, CreateBindingCommand, CreateCapabilityBindingCommand,
    CreateSubjectCommand, ListBindingsQuery, ListCapabilityBindingsQuery, ListSubjectsQuery,
    MemoryBackendRequestContext, UpdateSubjectCommand,
};
use sdkwork_routes_memory_support::{
    success_created_resource_response, success_no_content_response, success_page_response,
    success_resource_response,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::require_backend_context, paths, BackendApiProblem};

pub fn commercial_routes() -> Router {
    Router::new()
        .route(paths::SUBJECTS, get(list_subjects).post(create_subject))
        .route(
            paths::SUBJECT,
            get(retrieve_subject)
                .patch(update_subject)
                .delete(delete_subject),
        )
        .route(paths::BINDINGS, get(list_bindings).post(create_binding))
        .route(paths::BINDING, get(retrieve_binding).patch(stub_not_implemented).delete(delete_binding))
        .route(
            paths::CAPABILITY_BINDINGS,
            get(list_capability_bindings).post(create_capability_binding),
        )
        .route(
            paths::CAPABILITY_BINDING,
            get(retrieve_capability_binding).patch(stub_not_implemented).delete(delete_capability_binding),
        )
        .route(paths::CAPABILITIES_RESOLVE, post(resolve_capabilities))
        .route(paths::COMMERCIAL_READINESS, get(retrieve_commercial_readiness))
        .route(paths::COMMERCIAL_READINESS_REBUILD, post(rebuild_commercial_readiness))
        .route(paths::BINDINGS_BULK_DELETE, post(stub_not_implemented))
        .route(paths::BINDINGS_BULK_UPSERT, post(stub_not_implemented))
        .route(paths::BINDINGS_RESOLVE, post(stub_not_implemented))
        .route(
            paths::SUBJECT_EFFECTIVE_CAPABILITIES,
            get(stub_not_implemented),
        )
        .route(paths::SUBJECT_EFFECTIVE_POLICIES, get(stub_not_implemented))
        .route(paths::ENTITIES, get(stub_not_implemented).post(stub_not_implemented))
        .route(paths::ENTITY, get(stub_not_implemented).patch(stub_not_implemented))
        .route(paths::ENTITY_MERGE, post(stub_not_implemented))
        .route(paths::EDGES, get(stub_not_implemented).post(stub_not_implemented))
        .route(paths::EDGE, get(stub_not_implemented).patch(stub_not_implemented).delete(stub_not_implemented))
        .route(paths::POLICY_ASSIGNMENTS, get(stub_not_implemented).post(stub_not_implemented))
        .route(paths::POLICY_ASSIGNMENT, get(stub_not_implemented).patch(stub_not_implemented).delete(stub_not_implemented))
        .route(paths::RELATION_REBUILD_JOBS, post(stub_not_implemented))
        .route(paths::RELATION_REBUILD_JOB, get(stub_not_implemented))
}

fn forbidden(detail: &str) -> Response {
    BackendApiProblem::new(StatusCode::FORBIDDEN, "forbidden", detail).into_response()
}

fn bad_request(detail: &str) -> Response {
    BackendApiProblem::new(StatusCode::BAD_REQUEST, "validation_error", detail).into_response()
}

#[allow(clippy::result_large_err)]
fn parse_tenant_id(query_tenant_id: &str, context_tenant_id: u64) -> Result<u64, Response> {
    match query_tenant_id.parse::<u64>() {
        Ok(id) if id == context_tenant_id => Ok(id),
        _ => Err(bad_request("invalid or mismatched tenantId")),
    }
}

// --- Subject handlers ---

async fn create_subject(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreateSubjectCommand>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != cmd.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.create_subject(cmd).await {
        Ok(subject) => success_created_resource_response(subject),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn retrieve_subject(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(subject_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.retrieve_subject(tenant_id, &subject_id).await {
        Ok(subject) => success_resource_response(subject),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn list_subjects(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListSubjectsQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != query.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.list_subjects(query).await {
        Ok(list) => success_page_response(list),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn update_subject(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(subject_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdateSubjectCommand>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.update_subject(tenant_id, &subject_id, cmd).await {
        Ok(subject) => success_resource_response(subject),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn delete_subject(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(subject_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.delete_subject(tenant_id, &subject_id).await {
        Ok(()) => success_no_content_response(),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

// --- Binding handlers ---

async fn create_binding(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreateBindingCommand>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != cmd.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.create_binding(cmd).await {
        Ok(binding) => success_created_resource_response(binding),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn retrieve_binding(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(binding_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.retrieve_binding(tenant_id, &binding_id).await {
        Ok(binding) => success_resource_response(binding),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn list_bindings(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListBindingsQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != query.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.list_bindings(query).await {
        Ok(list) => success_page_response(list),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn delete_binding(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(binding_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.delete_binding(tenant_id, &binding_id).await {
        Ok(()) => success_no_content_response(),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

// --- Capability binding handlers ---

async fn create_capability_binding(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreateCapabilityBindingCommand>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != cmd.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.create_capability_binding(cmd).await {
        Ok(cap) => success_created_resource_response(cap),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn retrieve_capability_binding(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(cap_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .retrieve_capability_binding(tenant_id, &cap_id)
        .await
    {
        Ok(cap) => success_resource_response(cap),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn list_capability_bindings(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListCapabilityBindingsQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != query.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.list_capability_bindings(query).await {
        Ok(list) => success_page_response(list),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn delete_capability_binding(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(cap_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .delete_capability_binding(tenant_id, &cap_id)
        .await
    {
        Ok(()) => success_no_content_response(),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

// --- Capability resolution ---

#[derive(Deserialize)]
pub struct ResolveCapabilitiesRequest {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "targetType")]
    pub target_type: String,
    #[serde(rename = "targetId")]
    pub target_id: String,
}

async fn resolve_capabilities(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(req): Json<ResolveCapabilitiesRequest>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&req.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let target_type = match req.target_type.as_str() {
        "subject" => CapabilityTargetType::Subject,
        "space" => CapabilityTargetType::Space,
        "binding" => CapabilityTargetType::Binding,
        "memory" => CapabilityTargetType::Memory,
        _ => return bad_request("invalid targetType"),
    };
    let target_id = match req.target_id.parse::<u64>() {
        Ok(id) => id,
        _ => return bad_request("invalid targetId"),
    };
    match product
        .resolve_capabilities(tenant_id, target_type, target_id)
        .await
    {
        Ok(caps) => success_resource_response(caps),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

#[derive(Deserialize)]
pub struct TenantIdQuery {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
}

// --- Commercial readiness (stub endpoints for API contract compliance) ---

async fn retrieve_commercial_readiness(
    Extension(_product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let _ = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    // Commercial readiness snapshot retrieval is a planned feature.
    // Return 501 Not Implemented to distinguish from 404 (route not mounted).
    BackendApiProblem::new(
        StatusCode::NOT_IMPLEMENTED,
        "not_implemented",
        "commercial readiness retrieval is not yet implemented",
    )
    .into_response()
}

async fn rebuild_commercial_readiness(
    Extension(_product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let _ = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    // Commercial readiness rebuild is a planned feature.
    // Return 501 Not Implemented to distinguish from 404 (route not mounted).
    BackendApiProblem::new(
        StatusCode::NOT_IMPLEMENTED,
        "not_implemented",
        "commercial readiness rebuild is not yet implemented",
    )
    .into_response()
}

/// Generic stub handler for planned but not yet implemented endpoints.
/// Returns 501 Not Implemented to distinguish from 404 (route not mounted).
async fn stub_not_implemented() -> Response {
    BackendApiProblem::new(
        StatusCode::NOT_IMPLEMENTED,
        "not_implemented",
        "this endpoint is defined in the API contract but not yet implemented",
    )
    .into_response()
}
