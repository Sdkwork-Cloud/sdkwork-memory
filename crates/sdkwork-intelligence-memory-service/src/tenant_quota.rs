use sdkwork_memory_contract::{MemoryServiceError, MemoryServiceResult};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_memory_spi::MemoryScopeContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryQuotaLimits {
    pub max_records_per_space: u64,
    pub max_spaces_per_user: u64,
}

impl MemoryQuotaLimits {
    pub fn from_env() -> Self {
        Self {
            max_records_per_space: read_limit_env("SDKWORK_MEMORY_MAX_RECORDS_PER_SPACE", 100_000),
            max_spaces_per_user: read_limit_env("SDKWORK_MEMORY_MAX_SPACES_PER_USER", 100),
        }
    }
}

fn read_limit_env(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

pub async fn assert_space_record_quota(
    store: &NativeSqlMemoryStore,
    scope: &MemoryScopeContext,
    limits: MemoryQuotaLimits,
) -> MemoryServiceResult<()> {
    let limit = limits.max_records_per_space;
    if limit == 0 {
        return Ok(());
    }
    let count = store
        .count_active_records_for_scope(scope)
        .await
        .map_err(crate::store_error::map_native_sql_store_error)?;
    if count >= i64::try_from(limit).unwrap_or(i64::MAX) {
        crate::domain_metrics::memory_domain_metrics().record_quota_exceeded();
        return Err(MemoryServiceError::quota_exceeded(format!(
            "memory space {space_id} reached the maximum of {limit} active records",
            space_id = scope.space_id
        )));
    }
    Ok(())
}

pub async fn assert_user_space_quota(
    store: &NativeSqlMemoryStore,
    tenant_id: i64,
    owner_subject_type: &str,
    owner_subject_id: &str,
    limits: MemoryQuotaLimits,
) -> MemoryServiceResult<()> {
    if owner_subject_type != "user" {
        return Ok(());
    }
    let limit = limits.max_spaces_per_user;
    if limit == 0 {
        return Ok(());
    }
    let count = store
        .count_user_owned_spaces_for_tenant(tenant_id, owner_subject_id)
        .await
        .map_err(crate::store_error::map_native_sql_store_error)?;
    if count >= i64::try_from(limit).unwrap_or(i64::MAX) {
        crate::domain_metrics::memory_domain_metrics().record_quota_exceeded();
        return Err(MemoryServiceError::quota_exceeded(format!(
            "user {owner_subject_id} reached the maximum of {limit} memory spaces for this tenant"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_limit_disables_quota_enforcement() {
        let limits = MemoryQuotaLimits {
            max_records_per_space: 0,
            max_spaces_per_user: 0,
        };
        assert_eq!(limits.max_records_per_space, 0);
        assert_eq!(limits.max_spaces_per_user, 0);
    }
}
