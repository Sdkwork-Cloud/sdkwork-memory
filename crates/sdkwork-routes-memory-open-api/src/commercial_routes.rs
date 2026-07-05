//! Commercial entity and edge routes for the Open API.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Response,
    routing::{get, post, patch},
    Extension, Json, Router,
};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_contract::{
    CreateEdgeCommand, CreateEntityCommand, ListEdgesQuery, ListEntitiesQuery,
    MemoryOpenApiRequestContext, UpdateEdgeCommand, UpdateEntityCommand,
};
use sdkwork_routes_memory_support::{
    created_resource_json, no_content_json, ok_page_json, ok_resource_json,
};
use std::sync::Arc;

use crate::{auth::require_context, paths, ApiProblem};

pub fn commercial_routes() -> Router {
    Router::new()
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
}

fn forbidden(detail: &str) -> ApiProblem {
    ApiProblem::new(StatusCode::FORBIDDEN, "forbidden", detail)
}

async fn create_entity(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Json(cmd): Json<CreateEntityCommand>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    if context.tenant_id != cmd.tenant_id {
        return Err(forbidden("tenantId mismatch"));
    }
    created_resource_json(product.create_entity(cmd).await)
}

async fn retrieve_entity(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(entity_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    let tenant_id = parse_tenant_id(&query.tenant_id, context.tenant_id)?;
    ok_resource_json(product.retrieve_entity(tenant_id, &entity_id).await)
}

async fn list_entities(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Query(query): Query<ListEntitiesQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    if context.tenant_id != query.tenant_id {
        return Err(forbidden("tenantId mismatch"));
    }
    ok_page_json(product.list_entities(query).await)
}

async fn update_entity(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(entity_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdateEntityCommand>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    let tenant_id = parse_tenant_id(&query.tenant_id, context.tenant_id)?;
    ok_resource_json(product.update_entity(tenant_id, &entity_id, cmd).await)
}

async fn create_edge(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Json(cmd): Json<CreateEdgeCommand>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    if context.tenant_id != cmd.tenant_id {
        return Err(forbidden("tenantId mismatch"));
    }
    created_resource_json(product.create_edge(cmd).await)
}

async fn retrieve_edge(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(edge_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    let tenant_id = parse_tenant_id(&query.tenant_id, context.tenant_id)?;
    ok_resource_json(product.retrieve_edge(tenant_id, &edge_id).await)
}

async fn list_edges(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Query(query): Query<ListEdgesQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    if context.tenant_id != query.tenant_id {
        return Err(forbidden("tenantId mismatch"));
    }
    ok_page_json(product.list_edges(query).await)
}

async fn update_edge(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(edge_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
    Json(cmd): Json<UpdateEdgeCommand>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    let tenant_id = parse_tenant_id(&query.tenant_id, context.tenant_id)?;
    ok_resource_json(product.update_edge(tenant_id, &edge_id, cmd).await)
}

async fn delete_edge(
    Extension(product): Extension<Arc<OpenMemoryService>>,
    context: Option<Extension<MemoryOpenApiRequestContext>>,
    Path(edge_id): Path<String>,
    Query(query): Query<TenantIdQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_context(context)?;
    let tenant_id = parse_tenant_id(&query.tenant_id, context.tenant_id)?;
    no_content_json(product.delete_edge(tenant_id, &edge_id).await)
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
