use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_router_memory_app_api::{
    build_router_with_app_api, wrap_router_with_iam_database_web_framework,
};
use sdkwork_router_memory_backend_api::{
    build_router_with_shared_backend_api,
    wrap_router_with_iam_database_web_framework as wrap_backend_router,
};
use sdkwork_router_memory_open_api::build_router_with_shared_open_api;
use serde_json::json;
use std::sync::Arc;
use tower::util::ServiceExt;

const DEV_AUTH_TOKEN: &str =
    "Bearer tenant_id=1001;user_id=2001;session_id=s-1;app_id=sdkwork-memory;auth_level=password";
const DEV_ACCESS_TOKEN: &str =
    "tenant_id=1001;user_id=2001;session_id=s-1;app_id=sdkwork-memory;environment=dev;deployment_mode=saas";

fn authed_json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", DEV_AUTH_TOKEN)
        .header("Access-Token", DEV_ACCESS_TOKEN)
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn authed_get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", DEV_AUTH_TOKEN)
        .header("Access-Token", DEV_ACCESS_TOKEN)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn app_api_forget_and_export_jobs_round_trip_via_dual_token() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let create_memory = app
        .clone()
        .oneshot(authed_json_request(
            "POST",
            "/app/v3/api/memory/memories",
            json!({
                "spaceId": "1",
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
            "POST",
            "/app/v3/api/memory/forget_requests",
            json!({
                "scope": "memory",
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
    let forget_request_id = forget_json["forgetRequestId"].as_str().unwrap();

    let retrieve_forget = app
        .clone()
        .oneshot(authed_get(&format!(
            "/app/v3/api/memory/forget_requests/{forget_request_id}"
        )))
        .await
        .unwrap();
    assert_eq!(retrieve_forget.status(), StatusCode::OK);

    let export = app
        .clone()
        .oneshot(authed_json_request(
            "POST",
            "/app/v3/api/memory/export_jobs",
            json!({
                "spaceIds": ["1"],
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
    let export_job_id = export_json["exportJobId"].as_str().unwrap();

    let retrieve_export = app
        .oneshot(authed_get(&format!(
            "/app/v3/api/memory/export_jobs/{export_job_id}"
        )))
        .await
        .unwrap();
    assert_eq!(retrieve_export.status(), StatusCode::OK);
}

#[tokio::test]
async fn backend_api_lists_audit_logs_after_open_api_feedback() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let service = Arc::new(OpenMemoryService::new(store));
    let open_app = build_router_with_shared_open_api(service.clone());
    let backend_app = wrap_backend_router(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_shared_backend_api(service),
    );

    let feedback = open_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/feedback")
                .header("content-type", "application/json")
                .extension(sdkwork_memory_contract::MemoryOpenApiRequestContext {
                    api_key_id: "key-1".to_string(),
                    tenant_id: 1001,
                    actor_id: Some(2001),
                })
                .body(Body::from(
                    json!({
                        "targetType": "memory",
                        "targetId": "1",
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
