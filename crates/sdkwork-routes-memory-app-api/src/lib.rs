//! App API route boundary for SDKWork Memory.

use std::sync::Arc;

use axum::Router;
use sdkwork_web_core::HttpRouteManifest;

pub mod auth;
pub mod commercial_routes;
pub mod error;
pub mod http_route_manifest;
pub mod manifest;
pub mod paths;
pub mod routes;
pub mod web_bootstrap;

pub use error::{ApiError, ApiProblem};
pub use http_route_manifest::app_route_manifest;
pub use routes::{
    build_router_with_app_api, build_router_with_open_memory_service,
    build_router_with_shared_app_api,
};
pub use sdkwork_memory_contract::{MemoryAppApi, MemoryAppRequestContext};
pub use web_bootstrap::{
    memory_app_api_prefixes, memory_app_api_public_path_prefixes,
    wrap_router_with_iam_database_web_framework, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};

pub fn gateway_route_manifest() -> HttpRouteManifest {
    app_route_manifest()
}

/// Mounts routes with a shared trait implementation.
///
/// **Production:** use [`build_router_with_open_memory_service`] with a concrete
/// `OpenMemoryService` so commercial routes are fully wired. Trait-only mounts return `501` for
/// commercial management endpoints.
pub fn gateway_mount(api: Arc<dyn MemoryAppApi>) -> Router {
    build_router_with_shared_app_api(api)
}
