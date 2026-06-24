use sdkwork_memory_contract::{MemoryServiceError, MemoryServiceErrorKind};
use sdkwork_memory_plugin_native_sql::NativeSqlStoreError;

pub fn map_native_sql_store_error(error: NativeSqlStoreError) -> MemoryServiceError {
    if let NativeSqlStoreError::EventConflict { .. } = error {
        return MemoryServiceError::conflict(
            "event already exists with different payload for the same idempotency key",
        );
    }
    if let NativeSqlStoreError::OutboxConflict { .. } = error {
        return MemoryServiceError::conflict(
            "outbox event already exists with different payload for the same idempotency key",
        );
    }
    if let NativeSqlStoreError::Database(ref db_err) = error {
        if db_err
            .as_database_error()
            .is_some_and(|db| db.is_unique_violation())
        {
            return MemoryServiceError::conflict(
                "resource already exists for the requested unique key",
            );
        }
    }
    tracing::error!(error = %error, "memory store operation failed");
    MemoryServiceError {
        kind: MemoryServiceErrorKind::Storage,
        code: "storage_error".to_string(),
        detail: "internal storage error".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_memory_contract::MemoryServiceErrorKind;

    #[test]
    fn maps_event_conflict_to_http_conflict() {
        let mapped = map_native_sql_store_error(NativeSqlStoreError::EventConflict {
            tenant_id: 1,
            event_id: "evt-1".to_string(),
        });
        assert_eq!(mapped.kind, MemoryServiceErrorKind::Conflict);
    }

    #[test]
    fn maps_outbox_conflict_to_http_conflict() {
        let mapped = map_native_sql_store_error(NativeSqlStoreError::OutboxConflict {
            tenant_id: 1,
            outbox_id: "ob-1".to_string(),
        });
        assert_eq!(mapped.kind, MemoryServiceErrorKind::Conflict);
    }
}
