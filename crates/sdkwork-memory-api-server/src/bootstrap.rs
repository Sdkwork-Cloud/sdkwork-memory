use axum::{routing::get, Router};
use sdkwork_intelligence_memory_repository_sqlx::{
    connect_memory_pool_from_env, install_sqlite_schema, open_native_sql_store_from_pool,
};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_router_memory_app_api::{
    build_router_with_shared_app_api, wrap_router_with_web_framework_from_env as wrap_app_router,
};
use sdkwork_router_memory_backend_api::{
    build_router_with_shared_backend_api,
    wrap_router_with_web_framework_from_env as wrap_backend_router,
};
use sdkwork_router_memory_open_api::{
    build_router_with_shared_open_api, wrap_router_with_web_framework_from_env as wrap_open_router,
};
use std::sync::Arc;

async fn healthz() -> &'static str {
    "ok"
}

pub async fn build_router() -> Result<Router, String> {
    let pool = connect_memory_pool_from_env()
        .await
        .map_err(|error| format!("connect memory database failed: {error}"))?;
    if pool.as_postgres().is_some() {
        sdkwork_intelligence_memory_repository_sqlx::bootstrap_memory_database(pool.clone())
            .await
            .map_err(|error| format!("bootstrap memory database failed: {error}"))?;
    } else {
        install_sqlite_schema(&pool)
            .await
            .map_err(|error| format!("install memory schema failed: {error}"))?;
    }

    let store = open_native_sql_store_from_pool(&pool)
        .await
        .map_err(|error| format!("open native sql memory store failed: {error}"))?;
    let product = Arc::new(OpenMemoryService::new(store));

    let open_business_router = build_router_with_shared_open_api(product.clone());
    let app_business_router = build_router_with_shared_app_api(product.clone());
    let backend_business_router = build_router_with_shared_backend_api(product);

    let open_router = wrap_open_router(open_business_router).await;
    let app_router = wrap_app_router(app_business_router).await;
    let backend_router = wrap_backend_router(backend_business_router).await;

    Ok(Router::new()
        .merge(open_router)
        .merge(app_router)
        .merge(backend_router)
        .route("/healthz", get(healthz)))
}
