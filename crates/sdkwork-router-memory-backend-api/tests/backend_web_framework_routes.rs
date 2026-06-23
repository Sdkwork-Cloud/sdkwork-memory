use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_memory_contract::{
    MemoryBackendApi, MemoryBackendRequestContext, MemoryProviderHealth,
    MemoryProviderHealthStatus, MemoryServiceResult,
};
use sdkwork_router_memory_backend_api::{
    build_router_with_shared_backend_api, wrap_router_with_iam_database_web_framework,
};
use sdkwork_memory_test_support::web_auth::{
    lock_integration_test_env, memory_access_token, memory_auth_token_bearer,
};
use std::sync::{Arc, Mutex};
use tower::util::ServiceExt;

#[tokio::test]
async fn backend_router_web_framework_rejects_unauthenticated_requests() {
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_shared_backend_api(Arc::new(RecordingBackendApi::default())),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/backend/v3/api/memory/provider_health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn backend_router_web_framework_accepts_dev_jwt_dual_tokens_before_handler() {
    let _env = lock_integration_test_env();
    let service = RecordingBackendApi::default();
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_shared_backend_api(Arc::new(service.clone())),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/backend/v3/api/memory/provider_health")
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
struct RecordingBackendApi {
    tenant_ids: Arc<Mutex<Vec<u64>>>,
}

impl RecordingBackendApi {
    fn tenant_ids(&self) -> Vec<u64> {
        self.tenant_ids.lock().unwrap().clone()
    }
}

#[async_trait]
impl MemoryBackendApi for RecordingBackendApi {
    async fn retrieve_provider_health(
        &self,
        ctx: MemoryBackendRequestContext,
    ) -> MemoryServiceResult<MemoryProviderHealth> {
        self.tenant_ids.lock().unwrap().push(ctx.tenant_id);
        Ok(MemoryProviderHealth {
            status: MemoryProviderHealthStatus::Healthy,
            checked_at: "2026-06-10T00:00:00Z".to_string(),
            providers: vec![],
        })
    }
}
