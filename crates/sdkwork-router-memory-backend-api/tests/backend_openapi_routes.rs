use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_memory_contract::MemoryBackendApi;
use sdkwork_router_memory_backend_api::build_router_with_shared_backend_api;
use serde_json::Value;
use std::sync::Arc;
use tower::util::ServiceExt;

#[tokio::test]
async fn backend_router_mounts_every_backend_openapi_operation_path() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-memory-backend-sdk/openapi/memory-backend-api.openapi.json"
    ))
    .unwrap();
    let app = build_router_with_shared_backend_api(Arc::new(StubBackendApi));

    let paths = spec["paths"].as_object().unwrap();
    for (template_path, methods) in paths {
        for (method_name, operation) in methods.as_object().unwrap() {
            if !["get", "post", "put", "patch", "delete"].contains(&method_name.as_str()) {
                continue;
            }
            let operation_id = operation["operationId"].as_str().unwrap();
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method_from_openapi(method_name))
                        .uri(concrete_uri(template_path))
                        .header("content-type", "application/json")
                        .body(Body::from(request_body(operation_id)))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_ne!(
                response.status(),
                StatusCode::NOT_FOUND,
                "{operation_id} route from OpenAPI is not mounted: {method_name} {template_path}",
            );
        }
    }
}

struct StubBackendApi;

impl MemoryBackendApi for StubBackendApi {}

fn method_from_openapi(method_name: &str) -> Method {
    match method_name {
        "delete" => Method::DELETE,
        "get" => Method::GET,
        "patch" => Method::PATCH,
        "post" => Method::POST,
        "put" => Method::PUT,
        value => panic!("unsupported OpenAPI method: {value}"),
    }
}

fn concrete_uri(template_path: &str) -> String {
    template_path
        .replace("{spaceId}", "1")
        .replace("{memoryId}", "1")
        .replace("{eventId}", "1")
        .replace("{candidateId}", "1")
        .replace("{extractionJobId}", "1")
        .replace("{consolidationJobId}", "1")
        .replace("{indexId}", "1")
        .replace("{retrievalProfileId}", "1")
        .replace("{implementationProfileId}", "1")
        .replace("{providerBindingId}", "1")
        .replace("{evalRunId}", "1")
        .replace("{retrievalTraceId}", "1")
        .replace("{migrationJobId}", "1")
        .replace("{retentionJobId}", "1")
}

fn request_body(_operation_id: &str) -> &'static str {
    "{}"
}
