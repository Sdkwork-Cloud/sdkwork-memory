use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_memory_service::{spawn_background_workers, OpenMemoryService};
use sdkwork_memory_plugin_native_sql::{MemorySqlDialect, NativeSqlMemoryStore};
use sdkwork_memory_spi::{MemoryHabitStorePort, MemoryScopeContext, UpsertMemoryHabitCommand};
use sdkwork_routes_memory_app_api::{
    build_router_with_app_api, build_router_with_open_memory_service,
    wrap_router_with_iam_database_web_framework,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tower::util::ServiceExt;

use sdkwork_memory_test_support::api_envelope;
use sdkwork_memory_test_support::web_auth::{
    lock_integration_test_env, memory_access_token, memory_auth_token_bearer,
    MEMORY_TEST_IDEMPOTENCY_KEY,
};

fn authed_json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    let idempotency_key = format!("{MEMORY_TEST_IDEMPOTENCY_KEY}:{method}:{uri}");
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", memory_auth_token_bearer("2001"))
        .header("Access-Token", memory_access_token("2001"))
        .header("Idempotency-Key", idempotency_key)
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn authed_get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", memory_auth_token_bearer("2001"))
        .header("Access-Token", memory_access_token("2001"))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn app_api_mvp_flow_space_memory_and_retrieval_via_dual_token() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let space_id = "2";

    let create_memory = app
        .clone()
        .oneshot(authed_json_request(
            "POST",
            "/app/v3/api/memory/memories",
            json!({
                "spaceId": space_id,
                "scope": "user",
                "memoryType": "semantic",
                "canonicalText": "User prefers concise answers"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create_memory.status(), StatusCode::CREATED);

    let retrieval = app
        .oneshot(authed_json_request(
            "POST",
            "/app/v3/api/memory/retrievals",
            json!({
                "query": "concise answers",
                "spaceIds": [space_id],
                "topK": 5,
                "contextBudgetTokens": 512
            }),
        ))
        .await
        .unwrap();
    assert_eq!(retrieval.status(), StatusCode::CREATED);
    let retrieval_body = to_bytes(retrieval.into_body(), usize::MAX).await.unwrap();
    let retrieval_json: serde_json::Value = serde_json::from_slice(&retrieval_body).unwrap();
    let hits = api_envelope::item(&retrieval_json)["hits"]
        .as_array()
        .expect("retrieval response must contain a hit array");
    assert!(!hits.is_empty(), "exact lexical query must retrieve memory");
    assert!(hits.iter().any(|hit| {
        hit["explanation"]["contributingRetrievers"]
            .as_array()
            .is_some_and(|retrievers| retrievers.iter().any(|name| name == "keyword"))
    }));
}

#[tokio::test]
async fn app_api_habit_confirm_flow_via_dual_token() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let scope = MemoryScopeContext {
        tenant_id: 100_001,
        space_id: 2,
        organization_id: None,
        user_id: Some(2001),
    };
    MemoryHabitStorePort::upsert(
        &store,
        UpsertMemoryHabitCommand {
            scope: scope.clone(),
            habit_id: "9001".to_string(),
            user_id: 2001,
            habit_key: "answer_style:concise".to_string(),
            habit_type: "preference".to_string(),
            description: "Prefers concise answers".to_string(),
            stage: "candidate".to_string(),
            strength: 0.4,
            confidence: 0.8,
            support_count: 2,
            metadata_json: None,
        },
    )
    .await
    .expect("seed habit");

    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let confirm = app
        .oneshot(authed_json_request(
            "POST",
            "/app/v3/api/memory/habits/9001/confirm",
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(confirm.status(), StatusCode::OK);
    let confirm_body = to_bytes(confirm.into_body(), usize::MAX).await.unwrap();
    let confirm_json: serde_json::Value = serde_json::from_slice(&confirm_body).unwrap();
    assert_eq!(api_envelope::item(&confirm_json)["stage"], "confirmed");
}

#[tokio::test]
async fn app_api_memory_sources_list_returns_linked_event_sources() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let pool = store.pool().clone();
    let space_id = "2";
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_app_api(OpenMemoryService::new(store)),
    );

    let create_memory = app
        .clone()
        .oneshot(authed_json_request(
            "POST",
            "/app/v3/api/memory/memories",
            json!({
                "spaceId": space_id,
                "scope": "user",
                "memoryType": "semantic",
                "canonicalText": "User prefers concise answers"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create_memory.status(), StatusCode::CREATED);
    let memory_body = to_bytes(create_memory.into_body(), usize::MAX)
        .await
        .unwrap();
    let memory_json: serde_json::Value = serde_json::from_slice(&memory_body).unwrap();
    let memory_id = api_envelope::item(&memory_json)["memoryId"]
        .as_str()
        .unwrap();

    let seed_store =
        NativeSqlMemoryStore::from_any_pool(pool.clone(), MemorySqlDialect::Sqlite).await;
    let scope = MemoryScopeContext::for_test(100_001, space_id.parse().unwrap());
    seed_store
        .append_open_api_event(
            &scope,
            "8001",
            "message.user",
            "chat",
            "2026-06-10T00:00:00Z",
            &json!({ "text": "keep answers concise" }),
            "internal",
        )
        .await
        .expect("seed event");
    seed_store
        .append_record_source_for_tenant(100_001, "8101", memory_id, "8001", "evidence", Some(0.2))
        .await
        .expect("seed record source");

    let sources = app
        .oneshot(authed_get_request(&format!(
            "/app/v3/api/memory/memories/{memory_id}/sources"
        )))
        .await
        .unwrap();
    assert_eq!(sources.status(), StatusCode::OK);
    let sources_body = to_bytes(sources.into_body(), usize::MAX).await.unwrap();
    let sources_json: serde_json::Value = serde_json::from_slice(&sources_body).unwrap();
    let items = api_envelope::items(&sources_json).as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["sourceId"], "8101");
    assert_eq!(items[0]["eventId"], "8001");
    assert_eq!(items[0]["sourceRole"], "evidence");
}

#[tokio::test]
async fn app_api_candidate_approve_promotes_memory_and_links_event_sources() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let pool = store.pool().clone();
    let space_id = "2";
    let service = Arc::new(OpenMemoryService::new(store));
    let _shutdown = spawn_background_workers(service.clone());
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_open_memory_service(service),
    );

    let seed_store =
        NativeSqlMemoryStore::from_any_pool(pool.clone(), MemorySqlDialect::Sqlite).await;
    let scope = MemoryScopeContext::for_test(100_001, space_id.parse().unwrap());
    seed_store
        .append_open_api_event(
            &scope,
            "7001",
            "message.user",
            "chat",
            "2026-06-10T00:00:00Z",
            &json!({ "content": "User prefers concise answers" }),
            "internal",
        )
        .await
        .expect("seed event");

    let extraction = app
        .clone()
        .oneshot(authed_json_request(
            "POST",
            "/app/v3/api/memory/extractions",
            json!({
                "spaceId": space_id,
                "inputEvents": ["7001"],
                "extractionMode": "deterministic"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(extraction.status(), StatusCode::CREATED);

    let mut candidate_id = String::new();
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let candidates = app
            .clone()
            .oneshot(authed_get_request(&format!(
                "/app/v3/api/memory/candidates?spaceId={space_id}"
            )))
            .await
            .unwrap();
        assert_eq!(candidates.status(), StatusCode::OK);
        let candidates_body = to_bytes(candidates.into_body(), usize::MAX).await.unwrap();
        let candidates_json: serde_json::Value = serde_json::from_slice(&candidates_body).unwrap();
        let items = api_envelope::items(&candidates_json)
            .as_array()
            .expect("candidates list must return items array");
        if !items.is_empty() {
            candidate_id = items[0]["candidateId"]
                .as_str()
                .expect("candidate id")
                .to_string();
            break;
        }
    }
    assert!(
        !candidate_id.is_empty(),
        "extraction worker must produce at least one candidate"
    );

    let approve = app
        .clone()
        .oneshot(authed_json_request(
            "POST",
            &format!("/app/v3/api/memory/candidates/{candidate_id}/approve"),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(approve.status(), StatusCode::OK);
    let approve_body = to_bytes(approve.into_body(), usize::MAX).await.unwrap();
    let approve_json: serde_json::Value = serde_json::from_slice(&approve_body).unwrap();
    assert_eq!(
        api_envelope::item(&approve_json)["decisionState"],
        "approved"
    );

    let memories = app
        .clone()
        .oneshot(authed_get_request(&format!(
            "/app/v3/api/memory/memories?spaceId={space_id}"
        )))
        .await
        .unwrap();
    assert_eq!(memories.status(), StatusCode::OK);
    let memories_body = to_bytes(memories.into_body(), usize::MAX).await.unwrap();
    let memories_json: serde_json::Value = serde_json::from_slice(&memories_body).unwrap();
    let memory_id = api_envelope::items(&memories_json)[0]["memoryId"]
        .as_str()
        .unwrap();
    let promotion_outbox_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ai_outbox_event WHERE aggregate_id = ? AND event_type = 'memory.candidate.promoted'",
    )
    .bind(memory_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let promotion_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ai_audit_log WHERE resource_id = ? AND action = 'memory.candidate.promoted'",
    )
    .bind(memory_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(promotion_outbox_count, 1);
    assert_eq!(promotion_audit_count, 1);

    let sources = app
        .oneshot(authed_get_request(&format!(
            "/app/v3/api/memory/memories/{memory_id}/sources"
        )))
        .await
        .unwrap();
    assert_eq!(sources.status(), StatusCode::OK);
    let sources_body = to_bytes(sources.into_body(), usize::MAX).await.unwrap();
    let sources_json: serde_json::Value = serde_json::from_slice(&sources_body).unwrap();
    assert_eq!(api_envelope::items(&sources_json)[0]["eventId"], "7001");
    assert_eq!(
        api_envelope::items(&sources_json)[0]["sourceRole"],
        "evidence"
    );
}
