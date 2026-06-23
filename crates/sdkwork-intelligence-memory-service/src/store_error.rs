use sdkwork_memory_contract::{MemoryServiceError, MemoryServiceErrorKind};
use sdkwork_memory_plugin_native_sql::NativeSqlStoreError;

pub fn map_native_sql_store_error(error: NativeSqlStoreError) -> MemoryServiceError {
    if let NativeSqlStoreError::EventConflict { .. } = error {
        return MemoryServiceError::conflict(
            "event already exists with different payload for the same idempotency key",
        );
    }
    tracing::error!(error = %error, "memory store operation failed");
    MemoryServiceError {
        kind: MemoryServiceErrorKind::Storage,
        code: "storage_error".to_string(),
        detail: "internal storage error".to_string(),
    }
}
