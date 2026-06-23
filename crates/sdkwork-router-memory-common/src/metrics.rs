use std::sync::{Arc, OnceLock};

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

fn memory_http_metric_dimensions() -> HttpMetricsDimensions {
    HttpMetricsDimensions {
        service: "sdkwork-memory-api-server".to_owned(),
        environment: memory_metric_environment_label(),
        deployment_profile: metric_deployment_profile(),
        runtime_target: metric_runtime_target(),
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
