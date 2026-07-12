use sdkwork_memory_contract::{MemoryServiceError, MemoryServiceResult};
use sdkwork_memory_spi::{
    MemoryRecordQuotaAdmission, MemoryScopeContext, MemorySpaceQuotaAdmission,
};

use crate::platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryQuotaLimits {
    pub max_records_per_space: u64,
    pub max_spaces_per_user: u64,
}

impl MemoryQuotaLimits {
    pub fn from_env() -> Self {
        Self {
            max_records_per_space: platform::read_env_u64(
                "SDKWORK_MEMORY_MAX_RECORDS_PER_SPACE",
                100_000,
            ),
            max_spaces_per_user: platform::read_env_u64("SDKWORK_MEMORY_MAX_SPACES_PER_USER", 100),
        }
    }
}

pub fn resolve_space_record_quota_admission<T>(
    scope: &MemoryScopeContext,
    admission: MemoryRecordQuotaAdmission<T>,
) -> MemoryServiceResult<T> {
    match admission {
        MemoryRecordQuotaAdmission::Admitted(value) => Ok(value),
        MemoryRecordQuotaAdmission::QuotaExceeded {
            max_active_records, ..
        } => {
            crate::domain_metrics::memory_domain_metrics().record_quota_exceeded();
            Err(MemoryServiceError::quota_exceeded(format!(
                "memory space {space_id} reached the maximum of {max_active_records} active records",
                space_id = scope.space_id
            )))
        }
    }
}

pub fn resolve_user_space_quota_admission<T>(
    owner_subject_id: &str,
    admission: MemorySpaceQuotaAdmission<T>,
) -> MemoryServiceResult<T> {
    match admission {
        MemorySpaceQuotaAdmission::Admitted(value) => Ok(value),
        MemorySpaceQuotaAdmission::QuotaExceeded {
            max_active_spaces, ..
        } => {
            crate::domain_metrics::memory_domain_metrics().record_quota_exceeded();
            Err(MemoryServiceError::quota_exceeded(format!(
                "user {owner_subject_id} reached the maximum of {max_active_spaces} memory spaces for this tenant"
            )))
        }
    }
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
