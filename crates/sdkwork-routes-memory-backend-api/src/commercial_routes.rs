//! Commercial management route handlers for the backend API.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use sdkwork_memory_contract::{
    CreateBindingCommand, CreateCapabilityBindingCommand,
    CreateEdgeCommand, CreateEntityCommand, CreatePolicyAssignmentCommand, CreatePolicyCommand,
    CreateSubjectCommand, ListBindingsQuery, ListCapabilityBindingsQuery, ListEdgesQuery,
    ListEntitiesQuery, ListPoliciesQuery, ListPolicyAssignmentsQuery, ListSubjectsQuery,
    MemoryBackendRequestContext, RebuildCommercialReadinessCommand, ResolveCapabilitiesQuery,
    UpdateEdgeCommand,
    UpdateEntityCommand, UpdatePolicyAssignmentCommand, UpdatePolicyCommand, UpdateSubjectCommand,
};
use sdkwork_routes_memory_support::{
    parse_principal_u64, success_created_page_response, success_created_resource_response,
    success_no_content_response, success_page_response, success_resource_response,
};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use serde::Deserialize;

use crate::{auth::require_backend_context, paths, routes::BackendState, BackendApiProblem};

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
        .route(
            paths::BINDING,
            get(retrieve_binding).delete(delete_binding),
        )
        .route(
            paths::CAPABILITY_BINDINGS,
            get(list_capability_bindings).post(create_capability_binding),
        )
        .route(
            paths::CAPABILITY_BINDING,
            get(retrieve_capability_binding).delete(delete_capability_binding),
        )
        .route(paths::CAPABILITIES_RESOLVE, post(resolve_capabilities))
        .route(paths::ENTITIES, get(list_entities).post(create_entity))
        .route(
            paths::ENTITY,
            get(retrieve_entity).patch(update_entity),
        )
        .route(paths::EDGES, get(list_edges).post(create_edge))
        .route(
            paths::EDGE,
            get(retrieve_edge)
                .patch(update_edge)
                .delete(delete_edge),
        )
        .route(paths::POLICIES, get(list_policies).post(create_policy))
        .route(
            paths::POLICY,
            get(retrieve_policy)
                .patch(update_policy)
                .delete(delete_policy),
        )
        .route(
            paths::POLICY_ASSIGNMENTS,
            get(list_policy_assignments).post(create_policy_assignment),
        )
        .route(
            paths::POLICY_ASSIGNMENT,
            get(retrieve_policy_assignment)
                .patch(update_policy_assignment)
                .delete(delete_policy_assignment),
        )
        .route(
            paths::COMMERCIAL_READINESS,
            get(retrieve_commercial_readiness),
        )
        .route(
            paths::COMMERCIAL_READINESS_REBUILD,
            post(rebuild_commercial_readiness),
        )
}

fn forbidden(detail: &str) -> Response {
    BackendApiProblem::new(StatusCode::FORBIDDEN, "forbidden", detail).into_response()
}

fn bad_request(detail: &str) -> Response {
    BackendApiProblem::new(StatusCode::BAD_REQUEST, "validation_error", detail).into_response()
}

#[allow(clippy::result_large_err)]
fn parse_tenant_id(query_tenant_id: &str, context_tenant_id: u64) -> Result<u64, Response> {
    match parse_principal_u64(query_tenant_id) {
        Some(id) if id == context_tenant_id => Ok(id),
        _ => Err(bad_request("invalid or mismatched tenantId")),
    }
}

// --- Subject handlers ---

async fn create_subject(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreateSubjectCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(subject_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListSubjectsQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(subject_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdateSubjectCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(subject_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreateBindingCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(binding_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListBindingsQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(binding_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreateCapabilityBindingCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(cap_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListCapabilityBindingsQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(cap_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
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

async fn resolve_capabilities(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(req): Json<ResolveCapabilitiesQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != req.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.resolve_capabilities(req).await {
        Ok(page) => success_created_page_response(page),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

// --- Entity handlers ---

async fn create_entity(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreateEntityCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != cmd.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product
        .create_entity(OpenMemoryService::to_open_context_backend(&context), cmd)
        .await {
        Ok(entity) => success_created_resource_response(entity),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn retrieve_entity(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(entity_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .retrieve_entity(
            OpenMemoryService::to_open_context_backend(&context),
            tenant_id,
            &entity_id,
        )
        .await
    {
        Ok(entity) => success_resource_response(entity),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn list_entities(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListEntitiesQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != query.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product
        .list_entities(OpenMemoryService::to_open_context_backend(&context), query)
        .await
    {
        Ok(list) => success_page_response(list),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn update_entity(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(entity_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdateEntityCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .update_entity(
            OpenMemoryService::to_open_context_backend(&context),
            tenant_id,
            &entity_id,
            cmd,
        )
        .await {
        Ok(entity) => success_resource_response(entity),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

// --- Edge handlers ---

async fn create_edge(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreateEdgeCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != cmd.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product
        .create_edge(OpenMemoryService::to_open_context_backend(&context), cmd)
        .await {
        Ok(edge) => success_created_resource_response(edge),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn retrieve_edge(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(edge_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .retrieve_edge(
            OpenMemoryService::to_open_context_backend(&context),
            tenant_id,
            &edge_id,
        )
        .await
    {
        Ok(edge) => success_resource_response(edge),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn list_edges(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListEdgesQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != query.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product
        .list_edges(OpenMemoryService::to_open_context_backend(&context), query)
        .await
    {
        Ok(list) => success_page_response(list),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn update_edge(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(edge_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdateEdgeCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .update_edge(
            OpenMemoryService::to_open_context_backend(&context),
            tenant_id,
            &edge_id,
            cmd,
        )
        .await {
        Ok(edge) => success_resource_response(edge),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn delete_edge(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(edge_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .delete_edge(
            OpenMemoryService::to_open_context_backend(&context),
            tenant_id,
            &edge_id,
        )
        .await
    {
        Ok(()) => success_no_content_response(),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

// --- Policy handlers ---

async fn create_policy(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreatePolicyCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != cmd.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.create_policy(cmd).await {
        Ok(policy) => success_created_resource_response(policy),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn retrieve_policy(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(policy_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.retrieve_policy(tenant_id, &policy_id).await {
        Ok(policy) => success_resource_response(policy),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn list_policies(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListPoliciesQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != query.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.list_policies(query).await {
        Ok(list) => success_page_response(list),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn update_policy(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(policy_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdatePolicyCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.update_policy(tenant_id, &policy_id, cmd).await {
        Ok(policy) => success_resource_response(policy),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn delete_policy(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(policy_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.delete_policy(tenant_id, &policy_id).await {
        Ok(()) => success_no_content_response(),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

// --- Policy assignment handlers ---

async fn create_policy_assignment(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<CreatePolicyAssignmentCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != cmd.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.create_policy_assignment(cmd).await {
        Ok(assignment) => success_created_resource_response(assignment),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn retrieve_policy_assignment(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(assignment_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .retrieve_policy_assignment(tenant_id, &assignment_id)
        .await
    {
        Ok(assignment) => success_resource_response(assignment),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn list_policy_assignments(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<ListPolicyAssignmentsQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != query.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.list_policy_assignments(query).await {
        Ok(list) => success_page_response(list),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn update_policy_assignment(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(assignment_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdatePolicyAssignmentCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .update_policy_assignment(tenant_id, &assignment_id, cmd)
        .await
    {
        Ok(assignment) => success_resource_response(assignment),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn delete_policy_assignment(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Path(assignment_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product
        .delete_policy_assignment(tenant_id, &assignment_id)
        .await
    {
        Ok(()) => success_no_content_response(),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

// --- Commercial readiness handlers ---

async fn retrieve_commercial_readiness(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Query(query): Query<TenantIdQuery>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    let tenant_id = match parse_tenant_id(&query.tenant_id, context.tenant_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match product.retrieve_commercial_readiness(tenant_id).await {
        Ok(readiness) => success_resource_response(readiness),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

async fn rebuild_commercial_readiness(
    Extension(state): Extension<BackendState>,
    context: Option<Extension<MemoryBackendRequestContext>>,
    Json(cmd): Json<RebuildCommercialReadinessCommand>,
) -> Response {
    let product = match state.require_product() {
        Ok(product) => product,
        Err(resp) => return resp,
    };
    let context = match require_backend_context(context) {
        Ok(ctx) => ctx,
        Err(problem) => return problem.into_response(),
    };
    if context.tenant_id != cmd.tenant_id {
        return forbidden("tenantId mismatch");
    }
    match product.rebuild_commercial_readiness(cmd).await {
        Ok(readiness) => success_created_resource_response(readiness),
        Err(error) => BackendApiProblem::from(error).into_response(),
    }
}

#[derive(Deserialize)]
pub struct TenantIdQuery {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
}
