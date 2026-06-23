use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_router_memory_app_api::{
    build_router_with_app_api, wrap_router_with_iam_database_web_framework,
};
use sdkwork_router_memory_open_api::build_router_with_shared_open_api;
use sdkwork_memory_test_support::web_auth::{
    lock_integration_test_env, memory_access_token, memory_auth_token_bearer,
    MEMORY_TEST_IDEMPOTENCY_KEY,
};
use serde_json::json;
use std::sync::Arc;
use tower::util::ServiceExt;

fn authed_json_request(user_id: &str, method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    let idempotency_key = format!("{MEMORY_TEST_IDEMPOTENCY_KEY}:{method}:{uri}");
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", memory_auth_token_bearer(user_id))
        .header("Access-Token", memory_access_token(user_id))
        .header("Idempotency-Key", idempotency_key)
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn authed_get(user_id: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", memory_auth_token_bearer(user_id))
        .header("Access-Token", memory_access_token(user_id))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn open_api_rejects_cross_space_memory_access() {
    let _env = lock_integration_test_env();
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let service = Arc::new(OpenMemoryService::new(store));
    let open_app = build_router_with_shared_open_api(service);

    let create_memory = open_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext {
                    api_key_id: "key-1".to_string(),
                    tenant_id: 1001,
                    actor_id: Some(2001),
                })
                .body(Body::from(
                    json!({
                        "spaceId": "2",
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "space two secret"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_memory.status(), StatusCode::CREATED);
    let memory_body = to_bytes(create_memory.into_body(), usize::MAX)
        .await
        .unwrap();
    let memory_json: serde_json::Value = serde_json::from_slice(&memory_body).unwrap();
    let memory_id = memory_json["memoryId"].as_str().unwrap();

    let wrong_space = open_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/mem/v3/api/memory/memories/{memory_id}?spaceId=1"))
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext {
                    api_key_id: "key-1".to_string(),
                    tenant_id: 1001,
                    actor_id: Some(2001),
                })
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_space.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn app_api_requires_space_id_for_memory_list() {
    let _env = lock_integration_test_env();
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let response = app
        .oneshot(authed_get("2001", "/app/v3/api/memory/memories"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["code"], "validation_error");
}

#[tokio::test]
async fn learning_settings_persist_across_retrieve() {
    let _env = lock_integration_test_env();
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let patch = app
        .clone()
        .oneshot(authed_json_request(
            "2001",
            "PATCH",
            "/app/v3/api/memory/learning_settings",
            json!({
                "autoPromoteCandidates": true,
                "habitLearningEnabled": false
            }),
        ))
        .await
        .unwrap();
    assert_eq!(patch.status(), StatusCode::OK);

    let retrieve = app
        .oneshot(authed_get("2001", "/app/v3/api/memory/learning_settings"))
        .await
        .unwrap();
    assert_eq!(retrieve.status(), StatusCode::OK);
    let body = to_bytes(retrieve.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["autoPromoteCandidates"], true);
    assert_eq!(payload["habitLearningEnabled"], false);
}
