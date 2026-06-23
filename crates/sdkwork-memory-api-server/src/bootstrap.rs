use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use sdkwork_intelligence_memory_repository_sqlx::bootstrap_memory_runtime_from_env;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_router_memory_app_api::{
    build_router_with_shared_app_api, wrap_router_with_web_framework_from_env as wrap_app_router,
};
use sdkwork_router_memory_backend_api::{
    build_router_with_shared_backend_api,
    wrap_router_with_web_framework_from_env as wrap_backend_router,
};
use sdkwork_router_memory_common::{
    memory_http_metrics, refresh_memory_http_metric_dimensions,
};
use sdkwork_router_memory_open_api::{
    build_router_with_shared_open_api, wrap_router_with_web_framework_from_env as wrap_open_router,
};
use std::sync::Arc;
use tracing::info;

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(Extension(product): Extension<Arc<OpenMemoryService>>) -> Result<&'static str, StatusCode> {
    product
        .ready_check()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok("ok")
}

async fn metrics() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        memory_http_metrics().render_prometheus(),
    )
}

pub async fn build_router() -> Result<Router, String> {
    refresh_memory_http_metric_dimensions();
    let runtime = bootstrap_memory_runtime_from_env().await?;
    info!(
        profile_id = %runtime.profile_id,
        primary_plugin_id = %runtime.primary_plugin_id,
        dialect = ?runtime.data_plane.store().dialect(),
        postgres_host_pool = runtime.data_plane.host_pool.is_some(),
        "memory runtime ready"
    );
    let product = Arc::new(OpenMemoryService::from_phase1_runtime(
        runtime.data_plane.phase1,
        runtime.profile_id,
        runtime.primary_plugin_id,
    ));
    OpenMemoryService::spawn_background_workers(&product);

    let open_business_router = build_router_with_shared_open_api(product.clone());
    let app_business_router = build_router_with_shared_app_api(product.clone());
    let backend_business_router = build_router_with_shared_backend_api(product.clone());

    let open_router = wrap_open_router(open_business_router).await;
    let app_router = wrap_app_router(app_business_router).await;
    let backend_router = wrap_backend_router(backend_business_router).await;

    Ok(Router::new()
        .merge(open_router)
        .merge(app_router)
        .merge(backend_router)
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .layer(Extension(product)))
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_memory_database_host::bootstrap_memory_database_from_env().await?;
    info!("memory database migration completed");
    Ok(())
}
