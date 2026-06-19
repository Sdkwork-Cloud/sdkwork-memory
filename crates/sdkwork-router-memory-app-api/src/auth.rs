use axum::{http::StatusCode, Extension};

use sdkwork_memory_contract::MemoryAppRequestContext;

use crate::ApiProblem;

pub fn require_app_context(
    context: Option<Extension<MemoryAppRequestContext>>,
) -> Result<MemoryAppRequestContext, ApiProblem> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        ApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_app_request_context",
            "authenticated app request context is required",
        )
    })
}
