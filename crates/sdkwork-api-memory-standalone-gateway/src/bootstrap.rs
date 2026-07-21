use axum::{
    extract::{DefaultBodyLimit, Extension},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use sdkwork_intelligence_memory_repository_sqlx::bootstrap_memory_runtime_from_env;
use sdkwork_intelligence_memory_service::{
    platform, render_memory_domain_prometheus, validate_outbox_runtime_config, OpenMemoryService,
};
use sdkwork_api_memory_assembly::assemble_api_router;
use sdkwork_routes_memory_support::{
    memory_http_metrics, memory_metric_environment_label, refresh_memory_http_metric_dimensions,
};
use sdkwork_web_bootstrap::{healthz_handler, livez_handler, readyz_handler};
use std::sync::Arc;
use tower::limit::ConcurrencyLimitLayer;
use tracing::info;

use crate::readiness::MemoryReadinessCheck;

/// Default maximum request body size: 1 MiB.
const DEFAULT_MAX_BODY_BYTES: usize = 1024 * 1024;
/// Default maximum concurrent in-flight requests.
const DEFAULT_MAX_CONCURRENCY: usize = 256;

/// Assembled application router plus background-worker shutdown handle.
///
/// The caller MUST keep `worker_shutdown_tx` alive and call `send(true)`
/// during graceful shutdown so that background workers (outbox publisher,
/// learning job worker, eval run worker, provider health probe) can drain
/// in-flight work and exit cleanly.
pub struct MemoryApplication {
    pub router: Router,
    pub worker_shutdown_tx: tokio::sync::watch::Sender<bool>,
}

async fn metrics(Extension(product): Extension<Arc<OpenMemoryService>>) -> impl IntoResponse {
    let environment = memory_metric_environment_label();
    let deployment_profile = std::env::var("SDKWORK_MEMORY_DEPLOYMENT_PROFILE")
        .unwrap_or_else(|_| "standalone".to_owned());
    let runtime_target =
        std::env::var("SDKWORK_MEMORY_RUNTIME_TARGET").unwrap_or_else(|_| "server".to_owned());
    let runtime_profile = product.runtime_profile_label();
    let body = format!(
        "{}{}",
        memory_http_metrics().render_prometheus(),
        render_memory_domain_prometheus(
            "sdkwork-api-memory-standalone-gateway",
            &environment,
            &deployment_profile,
            &runtime_target,
            runtime_profile,
        )
    );
    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
}

pub async fn build_router() -> Result<MemoryApplication, String> {
    refresh_memory_http_metric_dimensions();
    validate_outbox_runtime_config().await?;
    let runtime = bootstrap_memory_runtime_from_env().await?;
    info!(
        profile_id = %runtime.core_runtime.profile().profile_id,
        primary_plugin_id = %runtime.core_runtime.profile().primary_plugin_id,
        dialect = ?runtime.data_plane.store().dialect(),
        postgres_host_pool = runtime.data_plane.host_pool.is_some(),
        "memory runtime ready"
    );
    let mut product = OpenMemoryService::try_from_core_runtime_with_retrieval_strategy(
        runtime.data_plane.phase1,
        runtime.core_runtime,
        runtime.retrieval_strategy,
    )?;
    if let Some(uploader) =
        sdkwork_memory_drive::bootstrap_memory_drive_export_uploader_from_env().await?
    {
        product = product.with_drive_export_uploader(uploader);
    }
    let product = Arc::new(product);
    let worker_shutdown_tx = OpenMemoryService::spawn_background_workers(&product);

    let business_router = assemble_api_router(product.clone())
        .await
        .router;

    let readiness = Arc::new(MemoryReadinessCheck::new(product.clone()));

    let max_body_bytes =
        platform::read_env_usize("SDKWORK_MEMORY_MAX_BODY_BYTES", DEFAULT_MAX_BODY_BYTES);
    let max_concurrency =
        platform::read_env_usize("SDKWORK_MEMORY_MAX_CONCURRENCY", DEFAULT_MAX_CONCURRENCY);

    let router = Router::new()
        .route("/metrics", get(metrics))
        .route("/healthz", get(healthz_handler))
        .route("/livez", get(livez_handler))
        .route(
            "/readyz",
            get({
                let readiness = readiness.clone();
                move || async move { readyz_handler(Some(readiness)).await }
            }),
        )
        .merge(business_router)
        .layer(Extension(product))
        .layer(DefaultBodyLimit::max(max_body_bytes))
        .layer(ConcurrencyLimitLayer::new(max_concurrency));

    info!(
        max_body_bytes,
        max_concurrency, "memory standalone-gateway rate limits configured"
    );

    Ok(MemoryApplication {
        router,
        worker_shutdown_tx,
    })
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_memory_database_host::bootstrap_memory_database_from_env().await?;
    info!("memory database migration completed");
    Ok(())
}
