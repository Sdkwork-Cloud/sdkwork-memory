use axum::{routing::get, Router};
use sdkwork_intelligence_memory_repository_sqlx::{
    connect_memory_pool_from_env, install_sqlite_schema,
};
use sdkwork_intelligence_memory_service::MemoryService;
use sdkwork_router_memory_app_api::wrap_router_with_web_framework_from_env as wrap_app_router;
use sdkwork_router_memory_backend_api::wrap_router_with_web_framework_from_env as wrap_backend_router;
use sdkwork_router_memory_open_api::wrap_router_with_web_framework_from_env as wrap_open_router;

async fn healthz() -> &'static str {
    "ok"
}

pub async fn build_router() -> Result<Router, String> {
    let pool = connect_memory_pool_from_env()
        .await
        .map_err(|error| format!("connect memory database failed: {error}"))?;
    install_sqlite_schema(&pool)
        .await
        .map_err(|error| format!("install memory schema failed: {error}"))?;

    let _service = MemoryService::new();

    let open_router = wrap_open_router(Router::new().route("/healthz", get(healthz))).await;
    let app_router = wrap_app_router(Router::new().route("/healthz", get(healthz))).await;
    let backend_router =
        wrap_backend_router(Router::new().route("/healthz", get(healthz))).await;

    Ok(Router::new()
        .merge(open_router)
        .merge(app_router)
        .merge(backend_router))
}
