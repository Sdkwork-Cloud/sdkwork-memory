use std::sync::{Arc, OnceLock};

use sdkwork_utils_rust::is_blank;
use sdkwork_web_core::{HttpMetricsDimensions, HttpMetricsRegistry};

static MEMORY_HTTP_METRICS: OnceLock<Arc<HttpMetricsRegistry>> = OnceLock::new();

fn metric_environment() -> String {
    std::env::var("SDKWORK_MEMORY_ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_owned())
        .to_ascii_lowercase()
}

fn metric_deployment_profile() -> String {
    std::env::var("SDKWORK_MEMORY_DEPLOYMENT_PROFILE")
        .unwrap_or_else(|_| "standalone".to_owned())
}

fn metric_runtime_target() -> String {
    std::env::var("SDKWORK_MEMORY_RUNTIME_TARGET").unwrap_or_else(|_| "server".to_owned())
}

fn metric_runtime_profile() -> String {
    std::env::var("SDKWORK_MEMORY_RUNTIME_PROFILE")
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))
        .unwrap_or_else(|| {
            // Infer runtime profile from database engine when explicit override is absent.
            let engine = std::env::var("SDKWORK_MEMORY_DATABASE_ENGINE")
                .or_else(|_| std::env::var("SDKWORK_DATABASE_ENGINE"))
                .unwrap_or_else(|_| "sqlite".to_owned())
                .to_ascii_lowercase();
            match engine.as_str() {
                "postgres" | "postgresql" => "postgresql".to_owned(),
                _ => "sqlite".to_owned(),
            }
        })
}

fn memory_http_metric_dimensions() -> HttpMetricsDimensions {
    HttpMetricsDimensions {
        service: "sdkwork-memory-standalone-gateway".to_owned(),
        environment: memory_metric_environment_label(),
        deployment_profile: metric_deployment_profile(),
        runtime_target: metric_runtime_target(),
        runtime_profile: metric_runtime_profile(),
    }
}

/// Shared Prometheus registry for all Memory HTTP surfaces (`OBSERVABILITY_SPEC.md` §3).
pub fn memory_http_metrics() -> Arc<HttpMetricsRegistry> {
    MEMORY_HTTP_METRICS
        .get_or_init(|| HttpMetricsRegistry::with_dimensions(memory_http_metric_dimensions()))
        .clone()
}

pub fn refresh_memory_http_metric_dimensions() {
    memory_http_metrics().set_dimensions(memory_http_metric_dimensions());
}

pub fn memory_metric_environment_label() -> String {
    match metric_environment().as_str() {
        "production" | "prod" => "production".to_owned(),
        "test" | "staging" => metric_environment(),
        _ => "development".to_owned(),
    }
}
