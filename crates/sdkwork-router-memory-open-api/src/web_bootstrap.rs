use std::sync::Arc;

use axum::Router;
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_memory_contract::MemoryOpenApiRequestContext;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, DomainContextInjector, WebRequestContext,
    WebRequestContextProfile,
};

use crate::http_route_manifest::open_route_manifest;
use crate::paths;

pub fn memory_open_api_public_path_prefixes() -> Vec<String> {
    vec![paths::HEALTHZ.to_owned()]
}

pub fn memory_open_api_prefixes() -> Vec<String> {
    vec![paths::PREFIX.to_owned()]
}

#[derive(Clone, Default)]
struct MemoryOpenApiContextInjector;

impl DomainContextInjector for MemoryOpenApiContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(open_context) = memory_open_api_context_from_web_request(context) {
            request.extensions_mut().insert(open_context);
        }
    }
}

fn memory_open_api_context_from_web_request(
    context: &WebRequestContext,
) -> Option<MemoryOpenApiRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id = principal.tenant_id().parse().ok()?;
    let actor_id = principal.user_id().parse().ok();
    let credential_id = principal
        .api_key_id()
        .map(str::to_owned)
        .or_else(|| principal.session_id().map(str::to_owned))
        .unwrap_or_else(|| principal.user_id().to_owned());
    Some(MemoryOpenApiRequestContext {
        api_key_id: credential_id,
        tenant_id,
        actor_id,
    })
}

pub fn wrap_router_with_web_framework(
    resolver: DefaultWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(router, build_memory_open_api_framework_layer(resolver))
}

pub fn wrap_router_with_iam_database_web_framework(
    resolver: IamDatabaseWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(router, build_memory_open_api_framework_layer(resolver))
}

fn build_memory_open_api_framework_layer<R>(resolver: R) -> WebFrameworkLayer<R>
where
    R: sdkwork_web_core::WebRequestContextResolver + Clone,
{
    let route_manifest = open_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&memory_open_api_public_path_prefixes())
        .expect("memory open-api public prefixes must not cover protected manifest routes");

    WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            open_api_prefixes: memory_open_api_prefixes(),
            public_path_prefixes: memory_open_api_public_path_prefixes(),
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(route_manifest)
        .with_domain_injector(Arc::new(MemoryOpenApiContextInjector))
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    if std::env::var("SDKWORK_MEMORY_DATABASE_URL").is_ok() {
        let resolver = sdkwork_iam_web_adapter::iam_database_resolver_from_env().await;
        return wrap_router_with_iam_database_web_framework(resolver, router);
    }

    wrap_router_with_web_framework(DefaultWebRequestContextResolver::default(), router)
}
