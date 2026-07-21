//! Verifies sensitivity tiers are enforced on memory list and retrieve paths.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_test_support::api_envelope;
use sdkwork_memory_test_support::space_fixtures::seed_user_space;
use sdkwork_routes_memory_open_api::build_router_with_shared_open_api;
use serde_json::json;
use std::sync::Arc;
use tower::util::ServiceExt;

#[tokio::test]
async fn list_and_retrieve_hide_private_memories_from_non_owner_tenant_actors() {
    let store = sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore::new_in_memory_sqlite()
        .await
        .expect("sqlite store must initialize");
    seed_user_space(&store, 100_001, 10, "2001").await;
    let service = Arc::new(OpenMemoryService::new(store));
    let app = build_router_with_shared_open_api(service);

    let owner_context = sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
        "key-owner",
        100_001,
        Some(2001),
    );
    let peer_context = sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
        "key-peer",
        100_001,
        Some(9001),
    );

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(owner_context.clone())
                .body(Body::from(
                    json!({
                        "spaceId": "10",
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "owner private note",
                        "sensitivityLevel": "private"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let create_body = to_bytes(create.into_body(), usize::MAX).await.unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let memory_id = api_envelope::item(&create_json)["memoryId"]
        .as_str()
        .unwrap();

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mem/v3/api/memory/memories?space_id=10")
                .extension(peer_context.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        list.status(),
        StatusCode::FORBIDDEN,
        "peer actor must not access a foreign user-owned space"
    );

    let retrieve = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/mem/v3/api/memory/memories/{memory_id}?space_id=10"
                ))
                .extension(peer_context)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        retrieve.status(),
        StatusCode::FORBIDDEN,
        "peer actor retrieve must fail closed on foreign user-owned space"
    );
}
