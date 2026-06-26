use axum::{http::StatusCode, Extension};

use sdkwork_memory_contract::MemoryBackendRequestContext;

use crate::BackendApiProblem;

pub fn require_backend_context(
    context: Option<Extension<MemoryBackendRequestContext>>,
) -> Result<MemoryBackendRequestContext, BackendApiProblem> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        BackendApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_backend_request_context",
            "authenticated backend request context is required",
        )
    })
}
