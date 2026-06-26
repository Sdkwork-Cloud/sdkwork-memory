//! Backend API route boundary for SDKWork Memory.

pub mod auth;
pub mod error;
pub mod http_route_manifest;
pub mod manifest;
pub mod paths;
pub mod routes;
pub mod web_bootstrap;

pub use error::{BackendApiError, BackendApiProblem, BackendApiResult};
pub use http_route_manifest::backend_route_manifest;
pub use routes::{build_router_with_backend_api, build_router_with_shared_backend_api};
pub use sdkwork_memory_contract::{MemoryBackendApi, MemoryBackendRequestContext, ProblemDetails};
pub use web_bootstrap::{
    memory_backend_api_prefixes, memory_backend_api_public_path_prefixes,
    wrap_router_with_iam_database_web_framework, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};

pub fn gateway_route_manifest() -> HttpRouteManifest {
    backend_route_manifest()
}

pub fn gateway_mount(api: Arc<dyn MemoryBackendApi>) -> Router {
    build_router_with_shared_backend_api(api)
}
