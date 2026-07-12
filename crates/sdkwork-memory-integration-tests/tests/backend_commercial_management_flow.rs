use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_routes_memory_backend_api::{
    build_router_with_backend_api, wrap_router_with_iam_database_web_framework,
};
use sdkwork_memory_test_support::api_envelope;
use sdkwork_memory_test_support::web_auth::{
    lock_integration_test_env, memory_access_token, memory_auth_token_bearer,
    memory_content_sha256, memory_idempotency_key,
};
use sdkwork_web_core::CONTENT_SHA256_HEADER;
use serde_json::json;
use tower::util::ServiceExt;

fn authed_get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", memory_auth_token_bearer("9001"))
        .header("Access-Token", memory_access_token("9001"))
        .body(Body::empty())
        .unwrap()
}

fn authed_json(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    let body_text = body.to_string();
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", memory_auth_token_bearer("9001"))
        .header("Access-Token", memory_access_token("9001"))
        .header("Idempotency-Key", memory_idempotency_key(method, uri, &body_text))
        .header(CONTENT_SHA256_HEADER, memory_content_sha256(&body_text))
        .body(Body::from(body_text))
        .unwrap()
}

#[tokio::test]
async fn backend_commercial_entity_edge_policy_and_readiness_flow() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_backend_api(OpenMemoryService::new(store)),
    );

    let entity_a = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/entities",
            json!({
                "tenantId": "100001",
                "spaceId": "1",
                "entityType": "person",
                "canonicalName": "Alice Example"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(entity_a.status(), StatusCode::CREATED);
    let entity_a_body = to_bytes(entity_a.into_body(), usize::MAX).await.unwrap();
    let entity_a_json: serde_json::Value = serde_json::from_slice(&entity_a_body).unwrap();
    let entity_a_id = api_envelope::item(&entity_a_json)["entityId"]
        .as_str()
        .expect("entityId");

    let entity_b = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/entities",
            json!({
                "tenantId": "100001",
                "spaceId": "1",
                "entityType": "person",
                "canonicalName": "Bob Example"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(entity_b.status(), StatusCode::CREATED);
    let entity_b_body = to_bytes(entity_b.into_body(), usize::MAX).await.unwrap();
    let entity_b_json: serde_json::Value = serde_json::from_slice(&entity_b_body).unwrap();
    let entity_b_id = api_envelope::item(&entity_b_json)["entityId"]
        .as_str()
        .expect("entityId");
    assert_ne!(entity_a_id, entity_b_id);

    let edge = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/edges",
            json!({
                "tenantId": "100001",
                "spaceId": "1",
                "sourceEntityId": entity_a_id,
                "targetEntityId": entity_b_id,
                "relationType": "knows"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(edge.status(), StatusCode::CREATED);

    let policy = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/policies",
            json!({
                "tenantId": "100001",
                "policyType": "retrieval",
                "scope": "tenant",
                "policy": { "allowLearning": true }
            }),
        ))
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);
    let policy_body = to_bytes(policy.into_body(), usize::MAX).await.unwrap();
    let policy_json: serde_json::Value = serde_json::from_slice(&policy_body).unwrap();
    let policy_id = api_envelope::item(&policy_json)["policyId"]
        .as_str()
        .expect("policyId");

    let assignment = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/policy_assignments",
            json!({
                "tenantId": "100001",
                "policyId": policy_id,
                "targetType": "space",
                "targetId": "1",
                "inheritanceMode": "inherit"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(assignment.status(), StatusCode::CREATED);

    let resolved = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/capabilities/resolve",
            json!({
                "tenantId": "100001",
                "targetType": "space",
                "targetId": "1"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resolved.status(), StatusCode::OK);
    let resolved_body = to_bytes(resolved.into_body(), usize::MAX).await.unwrap();
    let resolved_json: serde_json::Value = serde_json::from_slice(&resolved_body).unwrap();
    api_envelope::assert_cursor_page_info(&resolved_json);

    let rebuild = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/commercial_readiness/rebuild",
            json!({ "tenantId": "100001" }),
        ))
        .await
        .unwrap();
    assert_eq!(rebuild.status(), StatusCode::OK);
    let rebuild_body = to_bytes(rebuild.into_body(), usize::MAX).await.unwrap();
    let rebuild_json: serde_json::Value = serde_json::from_slice(&rebuild_body).unwrap();
    let readiness_item = api_envelope::item(&rebuild_json);
    assert!(readiness_item["score"].as_f64().unwrap_or(0.0) > 0.0);
    assert_eq!(
        readiness_item["managementCoverage"]["entities"].as_i64(),
        Some(2)
    );

    let list = app
        .clone()
        .oneshot(authed_get(
            "/backend/v3/api/memory/entities?tenantId=100001&spaceId=1",
        ))
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = to_bytes(list.into_body(), usize::MAX).await.unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&list_body).unwrap();
    api_envelope::assert_cursor_page_info(&list_json);
    assert_eq!(api_envelope::items(&list_json).as_array().unwrap().len(), 2);
}
