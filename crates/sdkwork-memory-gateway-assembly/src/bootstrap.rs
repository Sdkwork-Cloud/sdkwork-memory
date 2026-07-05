//! Gateway bootstrap for sdkwork-memory.

use axum::Router;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_routes_memory_app_api::{
    build_router_with_open_memory_service as build_app_router_with_product,
    wrap_router_with_web_framework_from_env as wrap_app_router,
};
use sdkwork_routes_memory_backend_api::{
    build_router_with_open_memory_service as build_backend_router_with_product,
    wrap_router_with_web_framework_from_env as wrap_backend_router,
};
use sdkwork_routes_memory_open_api::{
    build_router_with_open_memory_service as build_open_router_with_product,
    wrap_router_with_web_framework_from_env as wrap_open_router,
};
use std::sync::Arc;

pub struct ApplicationAssembly {
    pub router: Router,
}

pub async fn assemble_application_business_router(
    product: Arc<OpenMemoryService>,
) -> ApplicationAssembly {
    let open_business_router = build_open_router_with_product(product.clone());
    let app_business_router = build_app_router_with_product(product.clone());
    let backend_business_router = build_backend_router_with_product(product.clone());

    let open_router = wrap_open_router(open_business_router).await;
    let app_router = wrap_app_router(app_business_router).await;
    let backend_router = wrap_backend_router(backend_business_router).await;

    let router = Router::new()
        .merge(open_router)
        .merge(app_router)
        .merge(backend_router)
        .layer(axum::Extension(product));

    ApplicationAssembly { router }
}
