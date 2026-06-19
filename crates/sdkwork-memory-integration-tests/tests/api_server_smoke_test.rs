use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

#[tokio::test]
async fn api_server_bootstrap_serves_healthz_with_in_memory_sqlite() {
    std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", "sqlite::memory:");
    let app = sdkwork_memory_api_server::build_router()
        .await
        .expect("api-server bootstrap should succeed with in-memory sqlite");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
