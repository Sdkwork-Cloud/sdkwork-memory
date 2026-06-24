use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_contract::{MemoryOpenApiRequestContext, ProblemDetails};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_router_memory_open_api::{build_router_with_open_api, open_route_manifest};
use tower::util::ServiceExt;

fn open_context() -> MemoryOpenApiRequestContext {
    MemoryOpenApiRequestContext::for_open_surface("api-key-001", 1001, Some(2001))
}

#[tokio::test]
async fn open_capabilities_route_returns_no_embedding_profile() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = build_router_with_open_api(OpenMemoryService::new(store));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mem/v3/api/memory/capabilities")
                .extension(open_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["embeddingOptional"], true);
    assert!(json["retrievers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "keyword"));
}

#[tokio::test]
async fn open_capabilities_route_rejects_missing_context() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = build_router_with_open_api(OpenMemoryService::new(store));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mem/v3/api/memory/capabilities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let problem: ProblemDetails = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        problem.code.as_deref(),
        Some("missing_open_api_request_context")
    );
}

#[test]
fn open_route_manifest_has_seventeen_operations() {
    let manifest = open_route_manifest();
    assert!(manifest
        .match_route("GET", "/mem/v3/api/memory/capabilities")
        .is_some());
    assert!(manifest
        .match_route("POST", "/mem/v3/api/memory/retrievals")
        .is_some());
    assert!(manifest
        .match_route("GET", "/mem/v3/api/memory/provider_health")
        .is_some());
}
