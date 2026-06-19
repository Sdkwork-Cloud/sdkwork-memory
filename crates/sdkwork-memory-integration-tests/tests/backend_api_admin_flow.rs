use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_router_memory_backend_api::{
    build_router_with_backend_api, wrap_router_with_iam_database_web_framework,
};
use serde_json::json;
use tower::util::ServiceExt;

const DEV_AUTH_TOKEN: &str =
    "Bearer tenant_id=1001;user_id=9001;session_id=s-1;app_id=sdkwork-memory;auth_level=password";
const DEV_ACCESS_TOKEN: &str =
    "tenant_id=1001;user_id=9001;session_id=s-1;app_id=sdkwork-memory;environment=dev;deployment_mode=saas";

fn authed_get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", DEV_AUTH_TOKEN)
        .header("Access-Token", DEV_ACCESS_TOKEN)
        .body(Body::empty())
        .unwrap()
}

fn authed_json(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", DEV_AUTH_TOKEN)
        .header("Access-Token", DEV_ACCESS_TOKEN)
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn backend_api_indexes_and_retrieval_profiles_return_phase1_defaults() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
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
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_backend_api(OpenMemoryService::new(store)),
    );

    let create = app
        .clone()
        .oneshot(authed_json(
            "POST",
            "/backend/v3/api/memory/migration_jobs",
            json!({
                "targetImplementationKind": "native_sql",
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
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_backend_api(OpenMemoryService::new(store.clone())),
    );

    let indexes = app
        .clone()
        .oneshot(authed_get("/backend/v3/api/memory/indexes"))
        .await
        .unwrap();
    assert_eq!(indexes.status(), StatusCode::OK);

    let index_rows = store.list_mem_indexes_for_tenant(1001, 20).await.unwrap();
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
        .retrieve_mem_eval_run_for_tenant(1001, eval_run_id)
        .await
        .unwrap()
        .expect("eval run should exist in mem_eval_run");
    assert_eq!(eval_row.eval_type, "retrieval");

    let audit_config = store
        .retrieve_admin_config_entity(1001, "eval_run", eval_run_id)
        .await
        .unwrap();
    assert!(
        audit_config.is_none(),
        "table-backed admin entities must not use mem_audit_log admin.config.save"
    );
}
