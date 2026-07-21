use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;

use crate::{MemoryApiError, MemoryApiProblem};

pub const INVALID_QUERY_DETAIL: &str =
    "query parameters must use the canonical names, types, and bounds";

/// Strict typed query extraction with the canonical SDKWork invalid-parameter response.
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for MemoryQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = MemoryApiProblem;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::try_from_uri(&parts.uri)
            .map(|Query(query)| Self(query))
            .map_err(|_| MemoryApiError::invalid_parameter(INVALID_QUERY_DETAIL).into())
    }
}
