//! Implementation profile migration — switches tenant primary profile and rebuilds indexes.

use sdkwork_memory_contract::{MemoryMigrationJobRequest, MemoryServiceError, MemoryServiceResult};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;

use crate::store_error::map_native_sql_store_error;

pub const ACTIVE_IMPLEMENTATION_PROFILE_KEY: &str = "implementation_profile.active";

/// Execute an implementation-profile migration for a tenant.
///
/// - `shadow`: validate profiles and return a comparison report without mutating state.
/// - `promote` / `switch`: demote source primary, promote target, persist preference, rebuild indexes.
pub async fn execute_implementation_profile_migration(
    store: &NativeSqlMemoryStore,
    tenant_id: i64,
    request: &MemoryMigrationJobRequest,
) -> MemoryServiceResult<serde_json::Value> {
    store
        .ensure_default_implementation_profile_for_tenant(tenant_id)
        .await
        .map_err(map_native_sql_store_error)?;

    let source_id = request.source_implementation_profile_id.to_string();
    let target_id = request.target_implementation_profile_id.to_string();

    let dry_run = request.dry_run.unwrap_or(false);
    if source_id == target_id && !dry_run {
        return Err(MemoryServiceError::validation(
            "sourceImplementationProfileId and targetImplementationProfileId must differ",
        ));
    }

    let mode = request.mode.trim();
    if mode != "switch" && mode != "promote" && mode != "shadow" {
        return Err(MemoryServiceError::validation(
            "mode must be 'switch', 'promote', or 'shadow'",
        ));
    }

    let source = store
        .retrieve_mem_implementation_profile_for_tenant(tenant_id, &source_id)
        .await
        .map_err(map_native_sql_store_error)?
        .ok_or_else(|| MemoryServiceError::not_found("source implementation profile not found"))?;
    let target = if source_id == target_id {
        source.clone()
    } else {
        store
            .retrieve_mem_implementation_profile_for_tenant(tenant_id, &target_id)
            .await
            .map_err(map_native_sql_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("target implementation profile not found"))?
    };

    if target.status == "deleted" {
        return Err(MemoryServiceError::validation(
            "target implementation profile is not active",
        ));
    }

    if dry_run || mode == "shadow" {
        return Ok(serde_json::json!({
            "dryRun": dry_run,
            "mode": mode,
            "sourceImplementationProfileId": request.source_implementation_profile_id,
            "targetImplementationProfileId": request.target_implementation_profile_id,
            "sourceImplementationKind": source.implementation_kind,
            "targetImplementationKind": target.implementation_kind,
            "sourceRole": source.role,
            "targetRole": target.role,
            "shadow": mode == "shadow",
        }));
    }

    store
        .apply_implementation_profile_switch(
            tenant_id,
            &source_id,
            &target_id,
            source.role == "primary",
            &target.implementation_kind,
            request.target_implementation_profile_id,
        )
        .await
        .map_err(map_native_sql_store_error)?;

    let rebuilt = store
        .rebuild_all_record_search_indexes(tenant_id)
        .await
        .map_err(map_native_sql_store_error)?;

    Ok(serde_json::json!({
        "mode": mode,
        "sourceImplementationProfileId": request.source_implementation_profile_id,
        "targetImplementationProfileId": request.target_implementation_profile_id,
        "implementationKind": target.implementation_kind,
        "rebuiltRecords": rebuilt,
        "migrated": true,
    }))
}
