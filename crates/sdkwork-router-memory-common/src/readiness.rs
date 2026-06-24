use std::sync::Arc;

use sdkwork_memory_contract::{
    memory_is_production_like_environment, memory_use_dev_inline_auth_resolver,
};
use sqlx::PgPool;
use tokio::sync::OnceCell;

static IAM_POSTGRES_POOL: OnceCell<Option<Arc<PgPool>>> = OnceCell::const_new();

pub async fn shared_iam_postgres_pool() -> Option<Arc<PgPool>> {
    IAM_POSTGRES_POOL
        .get_or_init(|| async {
            match sdkwork_database_sqlx::create_pool_from_env("IAM").await {
                Ok(Some(pool)) => pool.as_postgres().cloned().map(Arc::new),
                _ => None,
            }
        })
        .await
        .clone()
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

    match shared_iam_postgres_pool().await {
        Some(postgres) => sqlx::query("SELECT 1").execute(postgres.as_ref()).await.is_ok(),
        None => {
            tracing::warn!("memory readiness blocked: IAM database pool unavailable");
            false
        }
    }
}
