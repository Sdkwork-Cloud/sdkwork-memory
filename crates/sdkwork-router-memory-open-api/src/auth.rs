use axum::{http::StatusCode, Extension};

use sdkwork_memory_contract::MemoryOpenApiRequestContext;

use crate::ApiProblem;

pub fn require_context(
    context: Option<Extension<MemoryOpenApiRequestContext>>,
) -> Result<MemoryOpenApiRequestContext, ApiProblem> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        ApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_open_api_request_context",
            "authenticated open API credential context is required",
        )
    })
}
