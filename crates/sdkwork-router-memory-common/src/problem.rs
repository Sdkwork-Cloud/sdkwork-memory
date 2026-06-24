use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_memory_contract::{MemoryServiceError, MemoryServiceErrorKind, ProblemDetails};

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
    status: StatusCode,
    problem: Box<ProblemDetails>,
}

impl MemoryApiProblem {
    pub fn new(status: StatusCode, code: impl Into<String>, detail: impl Into<String>) -> Self {
        let title = status
            .canonical_reason()
            .unwrap_or("HTTP Error")
            .to_string();
        Self {
            status,
            problem: Box::new(ProblemDetails {
                r#type: "about:blank".to_string(),
                title,
                status: status.as_u16(),
                detail: Some(detail.into()),
                instance: None,
                code: Some(code.into()),
                request_id: None,
                trace_id: None,
            }),
        }
        .apply_current_correlation()
    }

    pub fn with_correlation(mut self, correlation: &MemoryProblemCorrelation) -> Self {
        self.problem.request_id = Some(correlation.request_id.clone());
        self.problem.trace_id = correlation.trace_id.clone();
        self
    }

    fn apply_current_correlation(self) -> Self {
        if let Some(correlation) = MemoryProblemCorrelation::current() {
            self.with_correlation(&correlation)
        } else {
            self
        }
    }
}

impl From<MemoryApiError> for MemoryApiProblem {
    fn from(error: MemoryApiError) -> Self {
        Self::new(error.status, error.code, error.detail).apply_current_correlation()
    }
}

impl From<MemoryServiceError> for MemoryApiProblem {
    fn from(error: MemoryServiceError) -> Self {
        MemoryApiError::from(error).into()
    }
}

impl IntoResponse for MemoryApiProblem {
    fn into_response(self) -> Response {
        let request_id = self.problem.request_id.clone();
        let mut response = (self.status, Json(*self.problem)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if let Some(request_id) = request_id.as_deref() {
            if let Ok(value) = HeaderValue::from_str(request_id) {
                response
                    .headers_mut()
                    .insert(sdkwork_web_core::REQUEST_ID_HEADER, value);
            }
        }
        response
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
    async fn problem_response_includes_request_and_trace_ids() {
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
        assert_eq!(payload["requestId"], "req-memory-1");
        assert_eq!(
            payload["traceId"],
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
    }
}
