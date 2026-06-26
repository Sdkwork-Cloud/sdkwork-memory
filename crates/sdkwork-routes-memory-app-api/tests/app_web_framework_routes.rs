use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_memory_contract::{
    MemoryAppApi, MemoryAppRequestContext, MemoryLearningSettings, MemoryServiceResult,
};
use sdkwork_routes_memory_app_api::{
    build_router_with_shared_app_api, wrap_router_with_iam_database_web_framework,
};
use sdkwork_memory_test_support::web_auth::{
    lock_integration_test_env, memory_access_token, memory_auth_token_bearer,
};
use std::sync::{Arc, Mutex};
use tower::util::ServiceExt;

#[tokio::test]
async fn app_router_web_framework_rejects_unauthenticated_requests() {
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_shared_app_api(Arc::new(RecordingAppApi::default())),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/memory/learning_settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn app_router_web_framework_accepts_dev_jwt_dual_tokens_before_handler() {
    let _env = lock_integration_test_env();
    let service = RecordingAppApi::default();
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_shared_app_api(Arc::new(service.clone())),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/memory/learning_settings")
                .header("Authorization", memory_auth_token_bearer("9001"))
                .header("Access-Token", memory_access_token("9001"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(service.tenant_ids(), vec![1001]);
}

#[derive(Clone, Default)]
struct RecordingAppApi {
    tenant_ids: Arc<Mutex<Vec<u64>>>,
}

impl RecordingAppApi {
    fn tenant_ids(&self) -> Vec<u64> {
        self.tenant_ids.lock().unwrap().clone()
    }
}

#[async_trait]
impl MemoryAppApi for RecordingAppApi {
    async fn retrieve_learning_settings(
        &self,
        ctx: MemoryAppRequestContext,
    ) -> MemoryServiceResult<MemoryLearningSettings> {
        self.tenant_ids.lock().unwrap().push(ctx.tenant_id);
        Ok(MemoryLearningSettings {
            auto_promote_candidates: false,
            habit_learning_enabled: true,
            updated_at: "2026-06-10T00:00:00Z".to_string(),
        })
    }
}
