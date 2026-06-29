use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_contract::MemoryBackendRequestContext;
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_memory_test_support::api_envelope;
use sdkwork_routes_memory_backend_api::{backend_route_manifest, build_router_with_backend_api};
use tower::util::ServiceExt;

fn backend_context() -> MemoryBackendRequestContext {
    MemoryBackendRequestContext {
        tenant_id: 100_001,
        operator_id: Some(9001),
    }
}

#[tokio::test]
async fn backend_provider_health_route_returns_healthy() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = build_router_with_backend_api(OpenMemoryService::new(store));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/backend/v3/api/memory/provider_health")
                .extension(backend_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(api_envelope::item(&json)["status"], "healthy");
}

#[test]
fn backend_route_manifest_resolves_provider_health_route() {
    let manifest = backend_route_manifest();
    assert!(manifest
        .match_route("GET", "/backend/v3/api/memory/provider_health")
        .is_some());
}
