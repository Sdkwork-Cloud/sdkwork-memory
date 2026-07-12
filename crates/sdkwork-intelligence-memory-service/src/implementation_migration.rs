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
    active_runtime_profile_id: &str,
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
            .ok_or_else(|| {
                MemoryServiceError::not_found("target implementation profile not found")
            })?
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
            "migrationScope": "control_plane_only",
            "liveRuntimeCutover": false,
            "activeRuntimeProfileId": active_runtime_profile_id,
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
        "migrationScope": "control_plane_only",
        "liveRuntimeCutover": false,
        "activeRuntimeProfileId": active_runtime_profile_id,
        "requiresRuntimeCutover": true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn migration_store() -> NativeSqlMemoryStore {
        let store = NativeSqlMemoryStore::new_in_memory_sqlite()
            .await
            .expect("migration test store must open");
        store
            .ensure_default_implementation_profile_for_tenant(1)
            .await
            .unwrap();
        store
            .insert_mem_implementation_profile(
                1,
                "2",
                "search-first-eval",
                "search_first",
                "shadow",
                "active",
                r#"{"keyword":true}"#,
                None,
                None,
            )
            .await
            .unwrap();
        store
    }

    fn request(mode: &str) -> MemoryMigrationJobRequest {
        MemoryMigrationJobRequest {
            source_implementation_profile_id: 1,
            target_implementation_profile_id: 2,
            mode: mode.to_string(),
            space_ids: None,
            dry_run: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn shadow_result_never_claims_live_runtime_cutover() {
        let store = migration_store().await;
        let result = execute_implementation_profile_migration(
            &store,
            1,
            &request("shadow"),
            "local-embedded-phase1",
        )
        .await
        .unwrap();
        assert_eq!(result["migrationScope"], "control_plane_only");
        assert_eq!(result["liveRuntimeCutover"], false);
        assert_eq!(result["activeRuntimeProfileId"], "local-embedded-phase1");
    }

    #[tokio::test]
    async fn switch_result_distinguishes_metadata_promotion_from_runtime_cutover() {
        let store = migration_store().await;
        let result = execute_implementation_profile_migration(
            &store,
            1,
            &request("switch"),
            "local-embedded-phase1",
        )
        .await
        .unwrap();
        assert_eq!(result["migrated"], true);
        assert_eq!(result["migrationScope"], "control_plane_only");
        assert_eq!(result["liveRuntimeCutover"], false);
        assert_eq!(result["requiresRuntimeCutover"], true);
    }
}
