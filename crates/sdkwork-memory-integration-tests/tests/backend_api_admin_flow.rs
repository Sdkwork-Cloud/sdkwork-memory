use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_routes_memory_backend_api::{
    build_router_with_backend_api, wrap_router_with_iam_database_web_framework,
};
use sdkwork_memory_test_support::web_auth::{
    lock_integration_test_env, memory_access_token, memory_auth_token_bearer,
    MEMORY_TEST_IDEMPOTENCY_KEY,
};
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
    let idempotency_key = format!("{MEMORY_TEST_IDEMPOTENCY_KEY}:{method}:{uri}");
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", memory_auth_token_bearer("9001"))
        .header("Access-Token", memory_access_token("9001"))
        .header("Idempotency-Key", idempotency_key)
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn backend_api_indexes_and_retrieval_profiles_return_phase1_defaults() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_backend_api(OpenMemoryService::new(store)),
    );

    let indexes = app
        .clone()
        .oneshot(authed_get("/backend/v3/api/memory/indexes"))
        .await
        .unwrap();
    assert_eq!(indexes.status(), StatusCode::OK);
    let indexes_body = to_bytes(indexes.into_body(), usize::MAX).await.unwrap();
    let indexes_json: serde_json::Value = serde_json::from_slice(&indexes_body).unwrap();
    assert_eq!(indexes_json["items"][0]["indexKind"], "keyword");

    let profiles = app
        .oneshot(authed_get("/backend/v3/api/memory/retrieval_profiles"))
        .await
        .unwrap();
    assert_eq!(profiles.status(), StatusCode::OK);
    let profiles_body = to_bytes(profiles.into_body(), usize::MAX).await.unwrap();
    let profiles_json: serde_json::Value = serde_json::from_slice(&profiles_body).unwrap();
    assert_eq!(profiles_json["items"][0]["name"], "keyword-default");
}

#[tokio::test]
async fn backend_api_migration_job_round_trip_via_dual_token() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_backend_api(OpenMemoryService::new(store)),
    );

    let create = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/migration_jobs",
            json!({
                "sourceImplementationProfileId": "1",
                "targetImplementationProfileId": "1",
                "mode": "shadow",
                "dryRun": true
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let create_body = to_bytes(create.into_body(), usize::MAX).await.unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let job_id = create_json["jobId"].as_str().unwrap();

    let retrieve = app
        .oneshot(authed_get(&format!(
            "/backend/v3/api/memory/migration_jobs/{job_id}"
        )))
        .await
        .unwrap();
    assert_eq!(retrieve.status(), StatusCode::OK);
    let retrieve_body = to_bytes(retrieve.into_body(), usize::MAX).await.unwrap();
    let retrieve_json: serde_json::Value = serde_json::from_slice(&retrieve_body).unwrap();
    assert_eq!(retrieve_json["jobType"], "migration");
}

#[tokio::test]
async fn backend_api_admin_config_persists_in_sql_tables() {
    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_backend_api(OpenMemoryService::new(store.clone())),
    );

    let indexes = app
        .clone()
        .oneshot(authed_get("/backend/v3/api/memory/indexes"))
        .await
        .unwrap();
    assert_eq!(indexes.status(), StatusCode::OK);

    let index_rows = store
        .list_mem_indexes_for_tenant(100_001, None, 20, None)
        .await
        .unwrap();
    assert!(!index_rows.is_empty());
    assert_eq!(index_rows[0].index_kind, "keyword");

    let create = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/eval_runs",
            json!({
                "evalType": "retrieval",
                "metrics": { "hitRate": 0.9 }
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let create_body = to_bytes(create.into_body(), usize::MAX).await.unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let eval_run_id = create_json["evalRunId"].as_str().unwrap();

    let eval_row = store
        .retrieve_mem_eval_run_for_tenant(100_001, eval_run_id)
        .await
        .unwrap()
        .expect("eval run should exist in ai_eval_run");
    assert_eq!(eval_row.eval_type, "retrieval");

    let binding = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/provider_bindings",
            json!({
                "providerKind": "memory",
                "providerCode": "native_sql",
                "displayName": "Native SQL Binding",
                "endpointRef": "providers/native-sql",
                "capabilities": { "keyword": true },
                "config": { "timeoutMs": 5000 }
            }),
        ))
        .await
        .unwrap();
    assert_eq!(binding.status(), StatusCode::CREATED);
    let binding_body = to_bytes(binding.into_body(), usize::MAX).await.unwrap();
    let binding_json: serde_json::Value = serde_json::from_slice(&binding_body).unwrap();
    assert_eq!(binding_json["capabilities"]["keyword"], true);
    assert_eq!(binding_json["endpointRef"], "providers/native-sql");
    assert_eq!(binding_json["config"]["timeoutMs"], 5000);

    let binding_id = binding_json["providerBindingId"].as_str().unwrap();
    let binding_row = store
        .retrieve_mem_provider_binding_for_tenant(100_001, binding_id)
        .await
        .unwrap()
        .expect("provider binding should exist in ai_provider_binding");
    assert_eq!(binding_row.endpoint_ref.as_deref(), Some("providers/native-sql"));
    assert!(binding_row.config_json.is_some());

    let profile = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/retrieval_profiles",
            json!({
                "name": "fusion-profile",
                "strategy": "hybrid",
                "retrievers": { "keyword": { "weight": 0.6 }, "vector": { "weight": 0.4 } },
                "fusionPolicy": { "mode": "rrf" },
                "topK": 8,
                "contextBudgetTokens": 4096
            }),
        ))
        .await
        .unwrap();
    assert_eq!(profile.status(), StatusCode::CREATED);
    let profile_body = to_bytes(profile.into_body(), usize::MAX).await.unwrap();
    let profile_json: serde_json::Value = serde_json::from_slice(&profile_body).unwrap();
    assert_eq!(profile_json["fusionPolicy"]["mode"], "rrf");

    let audit_config = store
        .retrieve_admin_config_entity(100_001, "eval_run", eval_run_id)
        .await
        .unwrap();
    assert!(
        audit_config.is_none(),
        "table-backed admin entities must not use ai_audit_log admin.config.save"
    );
}

#[tokio::test]
async fn backend_api_governance_jobs_consolidation_and_retention_succeed() {
    use sdkwork_memory_spi::MemoryScopeContext;

    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let scope = MemoryScopeContext {
        tenant_id: 100_001,
        space_id: 1,
        organization_id: None,
        user_id: Some(9001),
    };
    store
        .create_record(&scope, "rec-dup-1", "preference", "duplicate canonical text")
        .await
        .unwrap();
    store
        .create_record(&scope, "rec-dup-2", "preference", "duplicate canonical text")
        .await
        .unwrap();

    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_backend_api(OpenMemoryService::new(store)),
    );

    let consolidation = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/consolidation_jobs",
            json!({
                "spaceId": "1",
                "inputEvents": []
            }),
        ))
        .await
        .unwrap();
    assert_eq!(consolidation.status(), StatusCode::CREATED);
    let consolidation_body = to_bytes(consolidation.into_body(), usize::MAX)
        .await
        .unwrap();
    let consolidation_json: serde_json::Value =
        serde_json::from_slice(&consolidation_body).unwrap();
    assert_eq!(consolidation_json["state"], "succeeded");
    assert!(
        consolidation_json["result"]["mergedDuplicates"]
            .as_u64()
            .unwrap_or(0)
            >= 1
    );

    let retention = app
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/retention_jobs",
            json!({
                "scope": "space",
                "spaceId": "1",
                "dryRun": true
            }),
        ))
        .await
        .unwrap();
    assert_eq!(retention.status(), StatusCode::CREATED);
    let retention_body = to_bytes(retention.into_body(), usize::MAX).await.unwrap();
    let retention_json: serde_json::Value = serde_json::from_slice(&retention_body).unwrap();
    assert_eq!(retention_json["state"], "succeeded");
}

#[tokio::test]
async fn backend_api_supersede_memory_links_chain_and_marks_old_record() {
    use sdkwork_memory_spi::MemoryScopeContext;

    let _env = lock_integration_test_env();
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let scope = MemoryScopeContext {
        tenant_id: 100_001,
        space_id: 1,
        organization_id: None,
        user_id: Some(9001),
    };
    store
        .create_record_open_api(
            &scope,
            "100",
            "user",
            "semantic",
            None,
            None,
            "original preference text",
            "original preference text",
            "internal",
        )
        .await
        .unwrap();

    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_backend_api(OpenMemoryService::new(store)),
    );

    let supersede = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/memories/100/supersede",
            json!({
                "spaceId": "1",
                "scope": "user",
                "memoryType": "semantic",
                "canonicalText": "updated preference text",
                "objectText": "updated preference text"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(supersede.status(), StatusCode::OK);
    let supersede_body = to_bytes(supersede.into_body(), usize::MAX).await.unwrap();
    let supersede_json: serde_json::Value = serde_json::from_slice(&supersede_body).unwrap();
    let new_memory_id = supersede_json["memoryId"].as_str().unwrap();

    assert_eq!(supersede_json["status"], "active");
    assert_eq!(supersede_json["supersedesMemoryId"], "100");
    assert_ne!(new_memory_id, "100");

    let old_record = app
        .clone()
        .oneshot(authed_get("/backend/v3/api/memory/memories/100?spaceId=1"))
        .await
        .unwrap();
    assert_eq!(old_record.status(), StatusCode::OK);
    let old_body = to_bytes(old_record.into_body(), usize::MAX).await.unwrap();
    let old_json: serde_json::Value = serde_json::from_slice(&old_body).unwrap();
    assert_eq!(old_json["status"], "superseded");
    assert_eq!(old_json["supersededByMemoryId"], new_memory_id);
}
