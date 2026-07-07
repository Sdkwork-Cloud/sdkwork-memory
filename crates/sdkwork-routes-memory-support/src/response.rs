use axum::{
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};
use serde::Serialize;

use crate::correlation::MemoryProblemCorrelation;
use crate::problem::MemoryApiProblem;

pub fn resolved_trace_id() -> String {
    if let Some(correlation) = MemoryProblemCorrelation::current() {
        if let Some(trace_id) = correlation.trace_id.filter(|value| !value.is_empty()) {
            return trace_id;
        }
        if !correlation.request_id.is_empty() {
            return correlation.request_id;
        }
    }
    sdkwork_web_core::new_request_id()
}

fn attach_trace_header(response: &mut Response, trace_id: &str) {
    if let Ok(value) = HeaderValue::from_str(trace_id) {
        response.headers_mut().insert(
            HeaderName::from_static("x-sdkwork-trace-id"),
            value,
        );
    }
}

fn success_response<T: Serialize>(status: StatusCode, data: T) -> Response {
    let trace_id = resolved_trace_id();
    let envelope = SdkWorkApiResponse::success(data, trace_id.clone());
    let mut response = (status, Json(envelope)).into_response();
    attach_trace_header(&mut response, &trace_id);
    response
}

pub fn ok_resource_json<T, E>(result: Result<T, E>) -> Result<Response, MemoryApiProblem>
where
    T: Serialize,
    E: Into<MemoryApiProblem>,
{
    match result {
        Ok(value) => Ok(success_response(
            StatusCode::OK,
            SdkWorkResourceData { item: value },
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn ok_page_json<T, E>(result: Result<T, E>) -> Result<Response, MemoryApiProblem>
where
    T: Serialize,
    E: Into<MemoryApiProblem>,
{
    match result {
        Ok(value) => Ok(success_response(StatusCode::OK, value)),
        Err(error) => Err(error.into()),
    }
}

pub fn created_resource_json<T, E>(result: Result<T, E>) -> Result<Response, MemoryApiProblem>
where
    T: Serialize,
    E: Into<MemoryApiProblem>,
{
    match result {
        Ok(value) => Ok(success_response(
            StatusCode::CREATED,
            SdkWorkResourceData { item: value },
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn no_content_json<E>(result: Result<(), E>) -> Result<Response, MemoryApiProblem>
where
    E: Into<MemoryApiProblem>,
{
    match result {
        Ok(()) => {
            let trace_id = resolved_trace_id();
            let mut response = StatusCode::NO_CONTENT.into_response();
            attach_trace_header(&mut response, &trace_id);
            Ok(response)
        }
        Err(error) => Err(error.into()),
    }
}

pub fn success_resource_response<T: Serialize>(value: T) -> Response {
    success_response(
        StatusCode::OK,
        SdkWorkResourceData { item: value },
    )
}

pub fn success_created_resource_response<T: Serialize>(value: T) -> Response {
    success_response(
        StatusCode::CREATED,
        SdkWorkResourceData { item: value },
    )
}

pub fn success_page_response<T: Serialize>(value: T) -> Response {
    success_response(StatusCode::OK, value)
}

pub fn success_no_content_response() -> Response {
    let trace_id = resolved_trace_id();
    let mut response = StatusCode::NO_CONTENT.into_response();
    attach_trace_header(&mut response, &trace_id);
    response
}

pub fn finish_resource_response<T, E>(result: Result<T, E>) -> Response
where
    T: Serialize,
    E: Into<MemoryApiProblem>,
{
    match result {
        Ok(value) => success_resource_response(value),
        Err(error) => error.into().into_response(),
    }
}

pub fn finish_created_resource_response<T, E>(result: Result<T, E>) -> Response
where
    T: Serialize,
    E: Into<MemoryApiProblem>,
{
    match result {
        Ok(value) => success_created_resource_response(value),
        Err(error) => error.into().into_response(),
    }
}

pub fn finish_page_response<T, E>(result: Result<T, E>) -> Response
where
    T: Serialize,
    E: Into<MemoryApiProblem>,
{
    match result {
        Ok(value) => success_page_response(value),
        Err(error) => error.into().into_response(),
    }
}

pub fn success_created_page_response<T: Serialize>(value: T) -> Response {
    success_response(StatusCode::CREATED, value)
}

pub fn finish_no_content_response<E>(result: Result<(), E>) -> Response
where
    E: Into<MemoryApiProblem>,
{
    match result {
        Ok(()) => success_no_content_response(),
        Err(error) => error.into().into_response(),
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware::from_fn;
    use axum::routing::get;
    use axum::Router;
    use sdkwork_web_core::{REQUEST_ID_HEADER, TRACEPARENT_HEADER};
    use tower::util::ServiceExt;

    use crate::correlation::problem_correlation_middleware;
    use crate::response::success_resource_response;

    async fn success_handler() -> axum::response::Response {
        success_resource_response(serde_json::json!({ "spaceId": "1" }))
    }

    #[tokio::test]
    async fn success_response_uses_sdkwork_api_response_envelope() {
        let app = Router::new()
            .route("/test", get(success_handler))
            .layer(from_fn(problem_correlation_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(REQUEST_ID_HEADER, "req-memory-1")
                    .header(
                        TRACEPARENT_HEADER,
                        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(0, payload["code"].as_i64().unwrap());
        assert_eq!(
            "4bf92f3577b34da6a3ce929d0e0e4736",
            payload["traceId"].as_str().unwrap()
        );
        assert_eq!("1", payload["data"]["item"]["spaceId"].as_str().unwrap());
    }
}
