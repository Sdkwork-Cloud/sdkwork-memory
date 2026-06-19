use std::sync::Arc;

use axum::Router;
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, WebRequestContextProfile,
};

use crate::http_route_manifest::backend_route_manifest;
use crate::paths;

pub fn memory_backend_api_public_path_prefixes() -> Vec<String> {
    vec![paths::HEALTHZ.to_owned()]
}

pub fn memory_backend_api_prefixes() -> Vec<String> {
    vec![paths::PREFIX.to_owned()]
}

pub fn wrap_router_with_web_framework(
    resolver: DefaultWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(
        router,
        build_memory_backend_api_framework_layer(resolver),
    )
}

pub fn wrap_router_with_iam_database_web_framework(
    resolver: IamDatabaseWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(
        router,
        build_memory_backend_api_framework_layer(resolver),
    )
}

fn build_memory_backend_api_framework_layer<R>(resolver: R) -> WebFrameworkLayer<R>
where
    R: sdkwork_web_core::WebRequestContextResolver + Clone,
{
    let route_manifest = backend_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&memory_backend_api_public_path_prefixes())
        .expect("memory backend-api public prefixes must not cover protected manifest routes");

    WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            backend_api_prefix: paths::PREFIX.to_owned(),
            public_path_prefixes: memory_backend_api_public_path_prefixes(),
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(route_manifest)
        .with_domain_injector(Arc::new(MemoryBackendApiNoopInjector))
}

#[derive(Clone, Default)]
struct MemoryBackendApiNoopInjector;

impl sdkwork_web_core::DomainContextInjector for MemoryBackendApiNoopInjector {
    fn inject(
        &self,
        _request: &mut axum::extract::Request,
        _context: &sdkwork_web_core::WebRequestContext,
    ) {
    }
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    if std::env::var("SDKWORK_MEMORY_DATABASE_URL").is_ok() {
        let resolver = sdkwork_iam_web_adapter::iam_database_resolver_from_env().await;
        return wrap_router_with_iam_database_web_framework(resolver, router);
    }

    wrap_router_with_web_framework(DefaultWebRequestContextResolver::default(), router)
}
