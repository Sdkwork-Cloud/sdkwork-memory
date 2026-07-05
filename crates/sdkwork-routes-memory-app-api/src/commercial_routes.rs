//! Commercial entity and policy assignment routes for the App API.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Response,
    routing::{get, patch},
    Extension, Json, Router,
};
use sdkwork_memory_contract::{
    CreateEntityCommand, CreatePolicyAssignmentCommand, ListEntitiesQuery,
    ListPolicyAssignmentsQuery, MemoryAppRequestContext, UpdateEntityCommand,
    UpdatePolicyAssignmentCommand,
};
use sdkwork_routes_memory_support::{
    created_resource_json, ok_page_json, ok_resource_json,
};

use crate::{auth::require_app_context, paths, routes::AppState, ApiProblem};

pub fn commercial_routes() -> Router {
    Router::new()
        .route(paths::ENTITIES, get(list_entities).post(create_entity))
        .route(
            paths::ENTITY,
            get(retrieve_entity).patch(update_entity),
        )
        .route(
            paths::POLICY_ASSIGNMENTS,
            get(list_policy_assignments).post(create_policy_assignment),
        )
        .route(
            paths::POLICY_ASSIGNMENT,
            patch(update_policy_assignment),
        )
}

fn forbidden(detail: &str) -> ApiProblem {
    ApiProblem::new(StatusCode::FORBIDDEN, "forbidden", detail)
}

async fn create_entity(
    Extension(state): Extension<AppState>,
    context: Option<Extension<MemoryAppRequestContext>>,
    Json(cmd): Json<CreateEntityCommand>,
) -> Result<Response, ApiProblem> {
    let product = state.require_product()?;
    let context = require_app_context(context)?;
    if context.tenant_id != cmd.tenant_id {
        return Err(forbidden("tenantId mismatch"));
    }
    created_resource_json(product.create_entity(cmd).await)
}

async fn retrieve_entity(
    Extension(state): Extension<AppState>,
    context: Option<Extension<MemoryAppRequestContext>>,
    Path(entity_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Result<Response, ApiProblem> {
    let product = state.require_product()?;
    let context = require_app_context(context)?;
    let tenant_id = parse_tenant_id(&query.tenant_id, context.tenant_id)?;
    ok_resource_json(product.retrieve_entity(tenant_id, &entity_id).await)
}

async fn list_entities(
    Extension(state): Extension<AppState>,
    context: Option<Extension<MemoryAppRequestContext>>,
    Query(query): Query<ListEntitiesQuery>,
) -> Result<Response, ApiProblem> {
    let product = state.require_product()?;
    let context = require_app_context(context)?;
    if context.tenant_id != query.tenant_id {
        return Err(forbidden("tenantId mismatch"));
    }
    ok_page_json(product.list_entities(query).await)
}

async fn update_entity(
    Extension(state): Extension<AppState>,
    context: Option<Extension<MemoryAppRequestContext>>,
    Path(entity_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdateEntityCommand>,
) -> Result<Response, ApiProblem> {
    let product = state.require_product()?;
    let context = require_app_context(context)?;
    let tenant_id = parse_tenant_id(&query.tenant_id, context.tenant_id)?;
    ok_resource_json(product.update_entity(tenant_id, &entity_id, cmd).await)
}

async fn create_policy_assignment(
    Extension(state): Extension<AppState>,
    context: Option<Extension<MemoryAppRequestContext>>,
    Json(cmd): Json<CreatePolicyAssignmentCommand>,
) -> Result<Response, ApiProblem> {
    let product = state.require_product()?;
    let context = require_app_context(context)?;
    if context.tenant_id != cmd.tenant_id {
        return Err(forbidden("tenantId mismatch"));
    }
    created_resource_json(product.create_policy_assignment(cmd).await)
}

async fn list_policy_assignments(
    Extension(state): Extension<AppState>,
    context: Option<Extension<MemoryAppRequestContext>>,
    Query(query): Query<ListPolicyAssignmentsQuery>,
) -> Result<Response, ApiProblem> {
    let product = state.require_product()?;
    let context = require_app_context(context)?;
    if context.tenant_id != query.tenant_id {
        return Err(forbidden("tenantId mismatch"));
    }
    ok_page_json(product.list_policy_assignments(query).await)
}

async fn update_policy_assignment(
    Extension(state): Extension<AppState>,
    context: Option<Extension<MemoryAppRequestContext>>,
    Path(assignment_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdatePolicyAssignmentCommand>,
) -> Result<Response, ApiProblem> {
    let product = state.require_product()?;
    let context = require_app_context(context)?;
    let tenant_id = parse_tenant_id(&query.tenant_id, context.tenant_id)?;
    ok_resource_json(
        product
            .update_policy_assignment(tenant_id, &assignment_id, cmd)
            .await,
    )
}

fn parse_tenant_id(query_tenant_id: &str, context_tenant_id: u64) -> Result<u64, ApiProblem> {
    match query_tenant_id.parse::<u64>() {
        Ok(id) if id == context_tenant_id => Ok(id),
        _ => Err(ApiProblem::new(
            StatusCode::BAD_REQUEST,
            "validation_error",
            "invalid or mismatched tenantId",
        )),
    }
}

#[derive(serde::Deserialize)]
pub struct TenantIdQuery {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
}
