use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_plugin_native_sql::{NativeSqlCreateSpaceCommand, NativeSqlMemoryStore};
use sdkwork_memory_spi::{CreateMemoryCandidateCommand, MemoryCandidateStorePort, MemoryScopeContext};
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
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let service = Arc::new(OpenMemoryService::new(store));
    let open_app = build_router_with_shared_open_api(service);

    let create_memory = open_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    1001,
                    Some(2001),
                ))
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
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    1001,
                    Some(2001),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_space.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_api_requires_space_id_for_memory_list() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
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
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
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

#[tokio::test]
async fn app_api_requires_space_id_for_candidate_list() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let response = app
        .oneshot(authed_get("2001", "/app/v3/api/memory/candidates"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn open_api_rejects_cross_space_candidate_retrieve() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    store
        .create_space_record(
            1001,
            3,
            &NativeSqlCreateSpaceCommand {
                organization_id: None,
                owner_subject_type: "user".to_string(),
                owner_subject_id: "3002".to_string(),
                space_type: "personal".to_string(),
                display_name: "Other user space".to_string(),
                default_scope: "user".to_string(),
            },
        )
        .await
        .unwrap();
    MemoryCandidateStorePort::create(
        &store,
        CreateMemoryCandidateCommand {
            scope: MemoryScopeContext::for_test(1001, 3),
            candidate_id: "7001".to_string(),
            candidate_type: "observation".to_string(),
            memory_type: "semantic".to_string(),
            proposed_text: "foreign candidate".to_string(),
            proposed_payload_json: None,
            evidence_json: None,
            confidence: 0.8,
        },
    )
    .await
    .unwrap();

    let open_app = build_router_with_shared_open_api(Arc::new(OpenMemoryService::new(store)));
    let response = open_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mem/v3/api/memory/candidates/7001")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    1001,
                    Some(2001),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_api_lists_only_actor_owned_spaces() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    for (owner, name) in [("3002", "Foreign space")] {
        let create = app
            .clone()
            .oneshot(authed_json_request(
                owner,
                "POST",
                "/app/v3/api/memory/spaces",
                json!({
                    "ownerSubjectType": "user",
                    "ownerSubjectId": owner,
                    "spaceType": "personal",
                    "displayName": name
                }),
            ))
            .await
            .unwrap();
        assert_eq!(create.status(), StatusCode::CREATED);
    }

    let list = app
        .oneshot(authed_get("2001", "/app/v3/api/memory/spaces"))
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let body = to_bytes(list.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let items = payload["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["displayName"], "Test Space 2");
}

#[tokio::test]
async fn open_api_rejects_cross_space_memory_write() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    store
        .create_space_record(
            1001,
            3,
            &NativeSqlCreateSpaceCommand {
                organization_id: None,
                owner_subject_type: "user".to_string(),
                owner_subject_id: "3002".to_string(),
                space_type: "personal".to_string(),
                display_name: "Foreign space".to_string(),
                default_scope: "user".to_string(),
            },
        )
        .await
        .unwrap();

    let open_app = build_router_with_shared_open_api(Arc::new(OpenMemoryService::new(store)));
    let response = open_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    1001,
                    Some(2001),
                ))
                .body(Body::from(
                    json!({
                        "spaceId": "3",
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "must not be written"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_api_rejects_space_owner_impersonation() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let response = app
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/spaces",
            json!({
                "ownerSubjectType": "user",
                "ownerSubjectId": "3002",
                "spaceType": "personal",
                "displayName": "Impersonated space"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn open_api_rejects_write_to_missing_space() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let open_app = build_router_with_shared_open_api(Arc::new(OpenMemoryService::new(store)));

    let response = open_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    1001,
                    Some(2001),
                ))
                .body(Body::from(
                    json!({
                        "spaceId": "99",
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "must not auto-provision"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn open_api_rejects_memory_create_when_space_record_quota_exceeded() {
    let _env = lock_integration_test_env();
    let previous_limit = std::env::var("SDKWORK_MEMORY_MAX_RECORDS_PER_SPACE").ok();
    std::env::set_var("SDKWORK_MEMORY_MAX_RECORDS_PER_SPACE", "1");

    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let open_app = build_router_with_shared_open_api(Arc::new(OpenMemoryService::new(store)));

    let first = open_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    1001,
                    Some(2001),
                ))
                .body(Body::from(
                    json!({
                        "spaceId": "2",
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "first memory"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = open_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    1001,
                    Some(2001),
                ))
                .body(Body::from(
                    json!({
                        "spaceId": "2",
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "second memory"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);

    match previous_limit {
        Some(value) => std::env::set_var("SDKWORK_MEMORY_MAX_RECORDS_PER_SPACE", value),
        None => std::env::remove_var("SDKWORK_MEMORY_MAX_RECORDS_PER_SPACE"),
    }
}

#[tokio::test]
async fn app_api_rejects_space_create_when_user_space_quota_exceeded() {
    let _env = lock_integration_test_env();
    let previous_limit = std::env::var("SDKWORK_MEMORY_MAX_SPACES_PER_USER").ok();
    std::env::set_var("SDKWORK_MEMORY_MAX_SPACES_PER_USER", "1");

    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let response = app
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/spaces",
            json!({
                "ownerSubjectType": "user",
                "ownerSubjectId": "2001",
                "spaceType": "personal",
                "displayName": "quota blocked space"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    match previous_limit {
        Some(value) => std::env::set_var("SDKWORK_MEMORY_MAX_SPACES_PER_USER", value),
        None => std::env::remove_var("SDKWORK_MEMORY_MAX_SPACES_PER_USER"),
    }
}
