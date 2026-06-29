use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sdkwork_memory_contract::{MemoryServiceError, MemoryServiceErrorKind};
use sdkwork_web_core::{
    problem_response, ProblemCorrelation, WebFrameworkError, WebFrameworkErrorKind,
};

use crate::correlation::MemoryProblemCorrelation;

pub type MemoryApiResult<T> = Result<T, MemoryApiError>;

#[derive(Debug, Clone)]
pub struct MemoryApiError {
    status: StatusCode,
    code: String,
    detail: String,
}

impl MemoryApiError {
    pub fn new(status: StatusCode, code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            detail: detail.into(),
        }
    }

    pub fn not_implemented(operation_id: &'static str) -> Self {
        Self::new(
            StatusCode::NOT_IMPLEMENTED,
            "operation_not_implemented",
            format!("operation is not implemented: {operation_id}"),
        )
    }

    fn framework_error(&self) -> WebFrameworkError {
        let kind = match self.status {
            StatusCode::BAD_REQUEST => WebFrameworkErrorKind::BadRequest,
            StatusCode::UNAUTHORIZED => WebFrameworkErrorKind::MissingCredentials,
            StatusCode::FORBIDDEN => WebFrameworkErrorKind::Forbidden,
            StatusCode::NOT_FOUND => WebFrameworkErrorKind::NotFound,
            StatusCode::CONFLICT => WebFrameworkErrorKind::Conflict,
            StatusCode::PAYLOAD_TOO_LARGE => WebFrameworkErrorKind::PayloadTooLarge,
            StatusCode::TOO_MANY_REQUESTS => WebFrameworkErrorKind::RateLimitExceeded,
            StatusCode::SERVICE_UNAVAILABLE => WebFrameworkErrorKind::DependencyUnavailable,
            StatusCode::REQUEST_TIMEOUT => WebFrameworkErrorKind::RequestTimeout,
            StatusCode::METHOD_NOT_ALLOWED => WebFrameworkErrorKind::MethodNotAllowed,
            StatusCode::NOT_IMPLEMENTED => WebFrameworkErrorKind::NotImplemented,
            _ if self.status.is_server_error() => WebFrameworkErrorKind::InternalServerError,
            _ => WebFrameworkErrorKind::BadRequest,
        };
        WebFrameworkError {
            kind,
            message: self.detail.clone(),
            retry_after_seconds: None,
        }
    }
}

impl From<MemoryServiceError> for MemoryApiError {
    fn from(error: MemoryServiceError) -> Self {
        let status = match error.kind {
            MemoryServiceErrorKind::NotFound => StatusCode::NOT_FOUND,
            MemoryServiceErrorKind::Conflict => StatusCode::CONFLICT,
            MemoryServiceErrorKind::Validation => StatusCode::BAD_REQUEST,
            MemoryServiceErrorKind::Forbidden => StatusCode::FORBIDDEN,
            MemoryServiceErrorKind::QuotaExceeded => StatusCode::TOO_MANY_REQUESTS,
            MemoryServiceErrorKind::Storage => StatusCode::INTERNAL_SERVER_ERROR,
            MemoryServiceErrorKind::NotImplemented => StatusCode::NOT_IMPLEMENTED,
        };
        Self::new(status, error.code, error.detail)
    }
}

#[derive(Debug, Clone)]
pub struct MemoryApiProblem {
    error: MemoryApiError,
}

impl MemoryApiProblem {
    pub fn new(status: StatusCode, code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            error: MemoryApiError::new(status, code, detail),
        }
    }
}

impl From<MemoryApiError> for MemoryApiProblem {
    fn from(error: MemoryApiError) -> Self {
        Self { error }
    }
}

impl From<MemoryServiceError> for MemoryApiProblem {
    fn from(error: MemoryServiceError) -> Self {
        MemoryApiError::from(error).into()
    }
}

impl IntoResponse for MemoryApiProblem {
    fn into_response(self) -> Response {
        let correlation = MemoryProblemCorrelation::current();
        let request_id = correlation.as_ref().map(|value| value.request_id.as_str());
        let trace_id = correlation
            .as_ref()
            .and_then(|value| value.trace_id.as_deref());
        problem_response(
            &self.error.framework_error(),
            ProblemCorrelation::new(request_id, trace_id),
        )
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
    use crate::problem::MemoryApiProblem;

    async fn failing_handler() -> Result<&'static str, MemoryApiProblem> {
        Err(MemoryApiProblem::new(
            StatusCode::BAD_REQUEST,
            "validation_error",
            "spaceId query parameter is required",
        ))
    }

    #[tokio::test]
    async fn problem_response_includes_trace_id_and_numeric_code() {
        let app = Router::new()
            .route("/test", get(failing_handler))
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

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(payload.get("requestId").is_none());
        assert_eq!(
            payload["traceId"],
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
        assert_eq!(40001, payload["code"].as_i64().unwrap());
    }
}
