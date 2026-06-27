use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_routes_memory_app_api::{
    build_router_with_app_api, wrap_router_with_iam_database_web_framework,
};
use sdkwork_routes_memory_backend_api::{
    build_router_with_shared_backend_api,
    wrap_router_with_iam_database_web_framework as wrap_backend_router,
};
use sdkwork_routes_memory_open_api::build_router_with_shared_open_api;
use sdkwork_memory_test_support::web_auth::{
    lock_integration_test_env, memory_access_token, memory_auth_token_bearer,
    MEMORY_TEST_IDEMPOTENCY_KEY,
};
use serde_json::json;
use std::sync::Arc;
use tower::util::ServiceExt;

fn authed_get(user_id: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", memory_auth_token_bearer(user_id))
        .header("Access-Token", memory_access_token(user_id))
        .body(Body::empty())
        .unwrap()
}

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

#[tokio::test]
async fn app_api_forget_and_export_jobs_round_trip_via_dual_token() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let create_memory = app
        .clone()
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/memories",
            json!({
                "spaceId": "2",
                "scope": "user",
                "memoryType": "semantic",
                "canonicalText": "temporary preference"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create_memory.status(), StatusCode::CREATED);
    let memory_body = to_bytes(create_memory.into_body(), usize::MAX)
        .await
        .unwrap();
    let memory_json: serde_json::Value = serde_json::from_slice(&memory_body).unwrap();
    let memory_id = memory_json["memoryId"].as_str().unwrap();

    let forget = app
        .clone()
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/forget_requests",
            json!({
                "scope": "memory",
                "spaceId": "2",
                "memoryIds": [memory_id],
                "reason": "user requested deletion"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(forget.status(), StatusCode::CREATED);
    let forget_body = to_bytes(forget.into_body(), usize::MAX).await.unwrap();
    let forget_json: serde_json::Value = serde_json::from_slice(&forget_body).unwrap();
    assert_eq!(forget_json["state"], "succeeded");
    assert_eq!(forget_json["result"]["deletedCount"], 1);
    let forget_request_id = forget_json["forgetRequestId"].as_str().unwrap();

    let deleted_memory = app
        .clone()
        .oneshot(authed_get(
            "2001",
            &format!("/app/v3/api/memory/memories/{memory_id}?spaceId=2"),
        ))
        .await
        .unwrap();
    assert_eq!(deleted_memory.status(), StatusCode::NOT_FOUND);

    let retrieve_forget = app
        .clone()
        .oneshot(authed_get(
            "2001",
            &format!("/app/v3/api/memory/forget_requests/{forget_request_id}"),
        ))
        .await
        .unwrap();
    assert_eq!(retrieve_forget.status(), StatusCode::OK);

    let export = app
        .clone()
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/export_jobs",
            json!({
                "spaceIds": ["2"],
                "format": "json",
                "includeEvents": true
            }),
        ))
        .await
        .unwrap();
    assert_eq!(export.status(), StatusCode::CREATED);
    let export_body = to_bytes(export.into_body(), usize::MAX).await.unwrap();
    let export_json: serde_json::Value = serde_json::from_slice(&export_body).unwrap();
    assert_eq!(export_json["state"], "succeeded");
    assert!(export_json["result"]["exportPayload"].is_object());
    let export_job_id = export_json["exportJobId"].as_str().unwrap();

    let retrieve_export = app
        .oneshot(authed_get(
            "2001",
            &format!("/app/v3/api/memory/export_jobs/{export_job_id}"),
        ))
        .await
        .unwrap();
    assert_eq!(retrieve_export.status(), StatusCode::OK);
    let stored_export_body = to_bytes(retrieve_export.into_body(), usize::MAX)
        .await
        .unwrap();
    let stored_export_json: serde_json::Value = serde_json::from_slice(&stored_export_body).unwrap();
    assert!(stored_export_json["result"]["exportRef"].is_string());
    assert!(stored_export_json["result"].get("exportPayload").is_none());
}

#[tokio::test]
async fn app_api_drive_export_job_stages_artifact_and_emits_outbox_event() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store.clone())),
    );

    let create_memory = app
        .clone()
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/memories",
            json!({
                "spaceId": "2",
                "scope": "user",
                "memoryType": "semantic",
                "canonicalText": "drive export preference"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create_memory.status(), StatusCode::CREATED);

    let export = app
        .clone()
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/export_jobs",
            json!({
                "spaceIds": ["2"],
                "format": "json",
                "driveTargetRef": "drive://app-upload/sdkwork-memory/exports"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(export.status(), StatusCode::CREATED);
    let export_body = to_bytes(export.into_body(), usize::MAX).await.unwrap();
    let export_json: serde_json::Value = serde_json::from_slice(&export_body).unwrap();
    assert_eq!(export_json["state"], "accepted");
    assert!(export_json["driveObjectRef"].as_str().unwrap().starts_with("mem-export/"));
    assert!(export_json["result"].get("exportPayload").is_none());
    let export_job_id = export_json["exportJobId"].as_str().unwrap();

    let artifact = store
        .retrieve_admin_config_entity(100_001, "export_artifact", export_job_id)
        .await
        .unwrap();
    assert!(artifact.is_some(), "drive export must stage artifact for pickup");

    let pending = store
        .list_pending_outbox_events(&sdkwork_memory_spi::MemoryScopeContext::for_test(100_001, 1), 10)
        .await
        .unwrap();
    assert!(
        pending
            .iter()
            .any(|event| event.event_type == "memory.export.drive_upload_requested"),
        "drive export must emit domain outbox event"
    );
}

#[tokio::test]
async fn backend_api_lists_audit_logs_after_open_api_feedback() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let service = Arc::new(OpenMemoryService::new(store));
    let open_app = build_router_with_shared_open_api(service.clone());
    let backend_app = wrap_backend_router(
        IamWebRequestContextResolver::new(None),
        build_router_with_shared_backend_api(service),
    );

    let create_memory = open_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    100_001,
                    Some(2001),
                ))
                .body(Body::from(
                    json!({
                        "spaceId": "2",
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "feedback target memory"
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

    let feedback = open_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/feedback")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext::for_open_surface(
                    "key-1",
                    100_001,
                    Some(2001),
                ))
                .body(Body::from(
                    json!({
                        "targetType": "memory",
                        "targetId": memory_id,
                        "feedbackType": "helpful"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(feedback.status(), StatusCode::CREATED);

    let audits = backend_app
        .oneshot(authed_get(
            "9001",
            "/backend/v3/api/memory/audit_logs?action=feedback.create",
        ))
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_body = to_bytes(audits.into_body(), usize::MAX).await.unwrap();
    let audits_json: serde_json::Value = serde_json::from_slice(&audits_body).unwrap();
    assert!(audits_json["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["action"] == "feedback.create"));
}

#[tokio::test]
async fn app_api_rejects_foreign_actor_retrieving_forget_job() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let create_memory = app
        .clone()
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/memories",
            json!({
                "spaceId": "2",
                "scope": "user",
                "memoryType": "semantic",
                "canonicalText": "forget idor target"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create_memory.status(), StatusCode::CREATED);
    let memory_body = to_bytes(create_memory.into_body(), usize::MAX)
        .await
        .unwrap();
    let memory_json: serde_json::Value = serde_json::from_slice(&memory_body).unwrap();
    let memory_id = memory_json["memoryId"].as_str().unwrap();

    let forget = app
        .clone()
        .oneshot(authed_json_request(
            "2001",
            "POST",
            "/app/v3/api/memory/forget_requests",
            json!({
                "scope": "memory",
                "spaceId": "2",
                "memoryIds": [memory_id],
                "reason": "user requested deletion"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(forget.status(), StatusCode::CREATED);
    let forget_body = to_bytes(forget.into_body(), usize::MAX).await.unwrap();
    let forget_json: serde_json::Value = serde_json::from_slice(&forget_body).unwrap();
    let forget_request_id = forget_json["forgetRequestId"].as_str().unwrap();

    let foreign_retrieve = app
        .oneshot(authed_get(
            "3002",
            &format!("/app/v3/api/memory/forget_requests/{forget_request_id}"),
        ))
        .await
        .unwrap();
    assert_eq!(foreign_retrieve.status(), StatusCode::FORBIDDEN);
}
