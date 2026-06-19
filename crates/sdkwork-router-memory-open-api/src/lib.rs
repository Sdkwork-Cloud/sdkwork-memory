//! Open API route boundary for SDKWork Memory.

pub mod http_route_manifest;
pub mod paths;
pub mod web_bootstrap;

pub use http_route_manifest::open_route_manifest;
pub use web_bootstrap::{
    memory_open_api_prefixes, memory_open_api_public_path_prefixes, wrap_router_with_web_framework,
    wrap_router_with_iam_database_web_framework, wrap_router_with_web_framework_from_env,
};
