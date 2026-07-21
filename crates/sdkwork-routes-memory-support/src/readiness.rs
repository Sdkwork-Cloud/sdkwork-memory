use std::sync::Arc;

use sdkwork_memory_contract::{
    memory_is_production_like_environment, memory_use_dev_inline_auth_resolver,
};
use sqlx::PgPool;
use tokio::sync::Mutex;

static IAM_POSTGRES_POOL: Mutex<Option<Arc<PgPool>>> = Mutex::const_new(None);

pub async fn shared_iam_postgres_pool() -> Option<Arc<PgPool>> {
    let mut cached = IAM_POSTGRES_POOL.lock().await;
    if cached.as_ref().is_some_and(|pool| !pool.is_closed()) {
        return cached.clone();
    }

    let postgres = match sdkwork_database_sqlx::create_pool_from_env("IAM").await {
        Ok(Some(pool)) => pool.as_postgres().cloned().map(Arc::new),
        _ => None,
    };
    if postgres.is_some() {
        *cached = postgres.clone();
    }
    postgres
}

/// Validates runtime dependencies required before serving authenticated traffic.
pub async fn memory_dependency_ready_check() -> bool {
    if memory_use_dev_inline_auth_resolver() {
        return true;
    }

    let iam_database_configured = std::env::var("SDKWORK_IAM_DATABASE_URL")
        .or_else(|_| std::env::var("SDKWORK_IAM_DATABASE_ENGINE"))
        .is_ok();

    if memory_is_production_like_environment() && !iam_database_configured {
        tracing::warn!("memory readiness blocked: production requires SDKWORK_IAM_DATABASE_URL");
        return false;
    }

    if !iam_database_configured {
        return true;
    }

    let iam_ready = match shared_iam_postgres_pool().await {
        Some(postgres) => sqlx::query("SELECT 1")
            .execute(postgres.as_ref())
            .await
            .is_ok(),
        None => {
            tracing::warn!("memory readiness blocked: IAM database pool unavailable");
            false
        }
    };
    iam_ready && crate::web_runtime::memory_redis_ready_check().await
}
