use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_contract::MemoryAppRequestContext;
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_router_memory_app_api::{app_route_manifest, build_router_with_app_api};
use tower::util::ServiceExt;

fn app_context() -> MemoryAppRequestContext {
    MemoryAppRequestContext {
        tenant_id: 1001,
        actor_id: Some(2001),
        organization_id: None,
        session_id: Some("session-001".to_string()),
    }
}

#[tokio::test]
async fn app_learning_settings_route_returns_defaults() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = build_router_with_app_api(OpenMemoryService::new(store));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/app/v3/api/memory/learning_settings")
                .extension(app_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["autoPromoteCandidates"], false);
}

#[tokio::test]
async fn app_learning_settings_route_rejects_missing_context() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = build_router_with_app_api(OpenMemoryService::new(store));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/app/v3/api/memory/learning_settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn app_route_manifest_resolves_learning_settings_route() {
    let manifest = app_route_manifest();
    assert!(manifest
        .match_route("GET", "/app/v3/api/memory/learning_settings")
        .is_some());
}
