//! Verifies sensitivity tiers are enforced on memory list and retrieve paths.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_plugin_native_sql::NativeSqlCreateSpaceCommand;
use sdkwork_routes_memory_open_api::build_router_with_shared_open_api;
use sdkwork_memory_test_support::api_envelope;
use serde_json::json;
use std::sync::Arc;
use tower::util::ServiceExt;

async fn seed_tenant_space(store: &sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore, space_id: i64) {
    store
        .create_space_record(
            100_001,
            space_id,
            &NativeSqlCreateSpaceCommand {
                organization_id: None,
                owner_subject_type: "tenant".to_string(),
                owner_subject_id: "100001".to_string(),
                space_type: "shared".to_string(),
                display_name: "Tenant shared space".to_string(),
                default_scope: "tenant".to_string(),
            },
        )
        .await
        .expect("tenant space seed must succeed");
}

#[tokio::test]
async fn list_and_retrieve_hide_private_memories_from_non_owner_tenant_actors() {
    let store = sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore::new_in_memory_sqlite()
        .await
        .expect("sqlite store must initialize");
    seed_tenant_space(&store, 10).await;
    let service = Arc::new(OpenMemoryService::new(store));
    let app = build_router_with_shared_open_api(service);

    let owner_context =
        sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface("key-owner", 100_001, Some(2001));
    let peer_context =
        sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface("key-peer", 100_001, Some(9001));

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
                        "scope": "tenant",
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
    let memory_id = api_envelope::item(&create_json)["memoryId"].as_str().unwrap();

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mem/v3/api/memory/memories?spaceId=10")
                .extension(peer_context.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = to_bytes(list.into_body(), usize::MAX).await.unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&list_body).unwrap();
    let items = api_envelope::items(&list_json)
        .as_array()
        .expect("list response must contain data.items array");
    assert!(
        items
            .iter()
            .all(|item| item["memoryId"].as_str() != Some(memory_id)),
        "peer actor must not see private memories in list results"
    );

    let retrieve = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/mem/v3/api/memory/memories/{memory_id}?spaceId=10"))
                .extension(peer_context)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        retrieve.status(),
        StatusCode::NOT_FOUND,
        "peer actor retrieve must fail closed as not found"
    );
}
