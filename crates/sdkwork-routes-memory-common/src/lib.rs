//! Shared Memory router auth wiring for sdkwork-web-framework integration.

pub mod correlation;
pub mod metrics;
pub mod problem;
pub mod readiness;

use async_trait::async_trait;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_memory_contract::{
    memory_is_production_like_environment, memory_use_dev_inline_auth_resolver,
};
use sdkwork_web_core::{WebFrameworkError, WebRequestContextResolver, WebRequestPrincipal};

pub use correlation::{with_problem_correlation, MemoryProblemCorrelation};
pub use metrics::{memory_http_metrics, memory_metric_environment_label, refresh_memory_http_metric_dimensions};
pub use problem::{MemoryApiError, MemoryApiProblem, MemoryApiResult};
pub use readiness::memory_dependency_ready_check;

const PRODUCTION_AUTH_UNAVAILABLE: &str =
    "production memory auth requires IAM PostgreSQL database";

/// How HTTP routers should resolve request context from environment.
pub enum MemoryWebAuthMode {
    DevInline,
    IamDatabase(IamWebRequestContextResolver),
    ProductionFailClosed,
}

/// Resolve the Memory web auth mode from runtime environment variables.
pub async fn memory_web_auth_mode_from_env() -> MemoryWebAuthMode {
    if memory_use_dev_inline_auth_resolver() {
        return MemoryWebAuthMode::DevInline;
    }

    let iam_database_explicitly_configured = std::env::var("SDKWORK_IAM_DATABASE_URL")
        .or_else(|_| std::env::var("SDKWORK_IAM_DATABASE_ENGINE"))
        .is_ok();

    if memory_is_production_like_environment() && !iam_database_explicitly_configured {
        return MemoryWebAuthMode::ProductionFailClosed;
    }

    MemoryWebAuthMode::IamDatabase(
        sdkwork_iam_web_adapter::IamWebRequestContextResolver::new(
            readiness::shared_iam_postgres_pool().await,
        ),
    )
}

#[derive(Clone, Default)]
pub struct ProductionFailClosedResolver;

#[async_trait]
impl WebRequestContextResolver for ProductionFailClosedResolver {
    async fn resolve_api_key(
        &self,
        _raw_api_key: &str,
    ) -> Result<WebRequestPrincipal, WebFrameworkError> {
        Err(WebFrameworkError::invalid_credentials(PRODUCTION_AUTH_UNAVAILABLE))
    }

    async fn resolve_dual_token(
        &self,
        _raw_auth_token: &str,
        _raw_access_token: &str,
    ) -> Result<WebRequestPrincipal, WebFrameworkError> {
        Err(WebFrameworkError::invalid_credentials(PRODUCTION_AUTH_UNAVAILABLE))
    }

    async fn resolve_access_token(
        &self,
        _raw_access_token: &str,
    ) -> Result<WebRequestPrincipal, WebFrameworkError> {
        Err(WebFrameworkError::invalid_credentials(PRODUCTION_AUTH_UNAVAILABLE))
    }

    async fn resolve_oauth_bearer(
        &self,
        _raw_bearer_token: &str,
    ) -> Result<WebRequestPrincipal, WebFrameworkError> {
        Err(WebFrameworkError::invalid_credentials(PRODUCTION_AUTH_UNAVAILABLE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn production_without_iam_database_uses_fail_closed_mode() {
        let _guard = sdkwork_memory_contract::runtime_env::env_test_lock();
        let previous_environment = std::env::var("SDKWORK_MEMORY_ENVIRONMENT").ok();
        let previous_bypass = std::env::var("SDKWORK_MEMORY_DEV_AUTH_BYPASS").ok();
        std::env::set_var("SDKWORK_MEMORY_ENVIRONMENT", "production");
        std::env::remove_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS");
        std::env::remove_var("SDKWORK_IAM_DATABASE_URL");

        let mode = memory_web_auth_mode_from_env().await;
        assert!(matches!(mode, MemoryWebAuthMode::ProductionFailClosed));

        if let Some(value) = previous_environment {
            std::env::set_var("SDKWORK_MEMORY_ENVIRONMENT", value);
        } else {
            std::env::remove_var("SDKWORK_MEMORY_ENVIRONMENT");
        }
        if let Some(value) = previous_bypass {
            std::env::set_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS", value);
        } else {
            std::env::remove_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS");
        }
    }

    #[tokio::test]
    async fn production_without_iam_database_fails_dependency_ready_check() {
        let _guard = sdkwork_memory_contract::runtime_env::env_test_lock();
        let previous_environment = std::env::var("SDKWORK_MEMORY_ENVIRONMENT").ok();
        let previous_bypass = std::env::var("SDKWORK_MEMORY_DEV_AUTH_BYPASS").ok();
        std::env::set_var("SDKWORK_MEMORY_ENVIRONMENT", "production");
        std::env::remove_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS");
        std::env::remove_var("SDKWORK_IAM_DATABASE_URL");
        std::env::remove_var("SDKWORK_IAM_DATABASE_ENGINE");

        assert!(!super::readiness::memory_dependency_ready_check().await);

        if let Some(value) = previous_environment {
            std::env::set_var("SDKWORK_MEMORY_ENVIRONMENT", value);
        } else {
            std::env::remove_var("SDKWORK_MEMORY_ENVIRONMENT");
        }
        if let Some(value) = previous_bypass {
            std::env::set_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS", value);
        } else {
            std::env::remove_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS");
        }
    }
}
