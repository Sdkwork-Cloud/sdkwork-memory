use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_memory_contract::runtime_env::env_test_lock;
use tower::util::ServiceExt;

const DEV_API_KEY: &str = "api_key_id=dev-key;tenant_id=1001;user_id=2001;app_id=sdkwork-memory";

#[tokio::test]
async fn api_server_bootstrap_auth_and_healthz_contracts() {
    let _guard = env_test_lock();
    std::env::set_var("SDKWORK_MEMORY_ENVIRONMENT", "development");
    std::env::set_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS", "true");
    std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", "sqlite::memory:");
    let dev_app = sdkwork_memory_api_server::build_router()
        .await
        .expect("api-server bootstrap should succeed with in-memory sqlite");

    let healthz = dev_app
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

    std::env::set_var("SDKWORK_MEMORY_ENVIRONMENT", "production");
    std::env::set_var("SDKWORK_MEMORY_CONFIG_PROFILE", "production");
    std::env::remove_var("SDKWORK_MEMORY_DEV_AUTH_BYPASS");
    std::env::remove_var("SDKWORK_IAM_DATABASE_URL");
    std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", "sqlite::memory:");

    let production_app = sdkwork_memory_api_server::build_router()
        .await
        .expect("api-server bootstrap should succeed with in-memory sqlite");

    let protected = production_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mem/v3/api/memory/capabilities")
                .header("x-api-key", DEV_API_KEY)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(protected.status(), StatusCode::UNAUTHORIZED);
}
