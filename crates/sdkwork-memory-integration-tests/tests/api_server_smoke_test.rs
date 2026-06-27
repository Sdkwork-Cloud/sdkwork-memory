use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_memory_contract::runtime_env::env_test_lock;
use sdkwork_memory_test_support::web_auth::{
    memory_access_token, memory_auth_token_bearer, memory_dev_api_key,
};
use tower::util::ServiceExt;

const DEV_API_KEY: &str = "dev-key";

fn restore_optional_env(key: &str, value: Option<String>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

#[tokio::test]
async fn api_server_bootstrap_auth_and_healthz_contracts() {
    let _guard = env_test_lock();
    let previous_environment = std::env::var("SDKWORK_MEMORY_ENVIRONMENT").ok();
    let previous_profile = std::env::var("SDKWORK_MEMORY_CONFIG_PROFILE").ok();
    let previous_bypass = std::env::var("SDKWORK_MEMORY_DEV_AUTH_BYPASS").ok();
    let previous_database_url = std::env::var("SDKWORK_MEMORY_DATABASE_URL").ok();
    let previous_iam_database_url = std::env::var("SDKWORK_IAM_DATABASE_URL").ok();

    std::env::set_var("SDKWORK_MEMORY_ENVIRONMENT", "development");
    std::env::set_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS", "true");
    std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", "sqlite::memory:");
    let dev_app = sdkwork_memory_standalone_gateway::build_router()
        .await
        .expect("standalone-gateway bootstrap should succeed with in-memory sqlite");
    let dev_router = dev_app.router;

    let healthz = dev_router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(healthz.status(), StatusCode::OK);

    let readyz = dev_router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(readyz.status(), StatusCode::OK);

    let metrics = dev_router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8_lossy(&metrics_body);
    assert!(
        metrics_text.contains("http_requests_total") || metrics_text.contains("http_request"),
        "metrics endpoint must expose HTTP request counters"
    );

    std::env::set_var("SDKWORK_MEMORY_ENVIRONMENT", "production");
    std::env::set_var("SDKWORK_MEMORY_CONFIG_PROFILE", "production");
    std::env::remove_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS");
    std::env::remove_var("SDKWORK_IAM_DATABASE_URL");
    std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", "sqlite::memory:");

    let production_app = sdkwork_memory_standalone_gateway::build_router()
        .await
        .expect("standalone-gateway bootstrap should succeed with in-memory sqlite");
    let production_router = production_app.router;

    let protected = production_router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mem/v3/api/memory/capabilities")
                .header("x-api-key", memory_dev_api_key("2001", DEV_API_KEY))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(protected.status(), StatusCode::UNAUTHORIZED);

    let protected_app = production_router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/app/v3/api/memory/learning_settings")
                .header("Authorization", memory_auth_token_bearer("2001"))
                .header("Access-Token", memory_access_token("2001"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(protected_app.status(), StatusCode::UNAUTHORIZED);

    restore_optional_env("SDKWORK_MEMORY_ENVIRONMENT", previous_environment);
    restore_optional_env("SDKWORK_MEMORY_CONFIG_PROFILE", previous_profile);
    restore_optional_env("SDKWORK_MEMORY_DEV_AUTH_BYPASS", previous_bypass);
    restore_optional_env("SDKWORK_MEMORY_DATABASE_URL", previous_database_url);
    restore_optional_env("SDKWORK_IAM_DATABASE_URL", previous_iam_database_url);
}

#[tokio::test]
async fn database_migrate_only_succeeds_with_sqlite() {
    let _guard = env_test_lock();
    let previous_database_url = std::env::var("SDKWORK_MEMORY_DATABASE_URL").ok();
    std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", "sqlite::memory:");
    sdkwork_memory_standalone_gateway::run_database_migrate_only()
        .await
        .expect("db-migrate bootstrap must succeed with sqlite");
    restore_optional_env("SDKWORK_MEMORY_DATABASE_URL", previous_database_url);
}
