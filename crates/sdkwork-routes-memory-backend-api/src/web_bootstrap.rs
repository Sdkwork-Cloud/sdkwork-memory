use std::sync::Arc;

use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_memory_contract::MemoryBackendRequestContext;
use sdkwork_routes_memory_support::{
    memory_http_metrics, memory_web_auth_mode_from_env, with_problem_correlation, MemoryWebAuthMode,
    ProductionFailClosedResolver,
};
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, DomainContextInjector, WebRequestContext,
    WebRequestContextProfile,
};

use crate::http_route_manifest::backend_route_manifest;
use crate::paths;

pub fn memory_backend_api_public_path_prefixes() -> Vec<String> {
    vec![paths::HEALTHZ.to_owned()]
}

pub fn memory_backend_api_prefixes() -> Vec<String> {
    vec![paths::PREFIX.to_owned()]
}

#[derive(Clone, Default)]
struct MemoryBackendContextInjector;

impl DomainContextInjector for MemoryBackendContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(backend_context) = memory_backend_context_from_web_request(context) {
            request.extensions_mut().insert(backend_context);
        }
    }
}

fn memory_backend_context_from_web_request(
    context: &WebRequestContext,
) -> Option<MemoryBackendRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id = principal.tenant_id().parse().ok()?;
    let operator_id = principal.user_id().parse().ok();
    Some(MemoryBackendRequestContext {
        tenant_id,
        operator_id,
    })
}

pub fn wrap_router_with_web_framework(
    resolver: DefaultWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(
        with_problem_correlation(router),
        build_memory_backend_api_framework_layer(resolver),
    )
}

pub fn wrap_router_with_iam_database_web_framework(
    resolver: IamWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(
        with_problem_correlation(router),
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
        .with_domain_injector(Arc::new(MemoryBackendContextInjector))
        .with_metrics(memory_http_metrics())
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    match memory_web_auth_mode_from_env().await {
        MemoryWebAuthMode::DevInline => {
            wrap_router_with_web_framework(DefaultWebRequestContextResolver::default(), router)
        }
        MemoryWebAuthMode::ProductionFailClosed => with_web_request_context(
            with_problem_correlation(router),
            build_memory_backend_api_framework_layer(ProductionFailClosedResolver),
        ),
        MemoryWebAuthMode::IamDatabase(resolver) => {
            wrap_router_with_iam_database_web_framework(resolver, router)
        }
    }
}
