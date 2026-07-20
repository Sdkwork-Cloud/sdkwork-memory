//! Implementation profile migration — switches tenant primary profile and rebuilds indexes.

use sdkwork_memory_contract::{MemoryMigrationJobRequest, MemoryServiceError, MemoryServiceResult};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;

use crate::store_error::map_native_sql_store_error;

pub const ACTIVE_IMPLEMENTATION_PROFILE_KEY: &str = "implementation_profile.active";

pub fn is_production_qualified_implementation_kind(value: &str) -> bool {
    matches!(value, "native_sql" | "local_embedded")
}

/// Execute an implementation-profile migration for a tenant.
///
/// - `shadow`: validate profiles and return a comparison report without mutating state.
/// - `promote` / `switch`: demote source primary, promote target, persist preference, rebuild indexes.
pub async fn execute_implementation_profile_migration(
    store: &NativeSqlMemoryStore,
    tenant_id: i64,
    request: &MemoryMigrationJobRequest,
    active_runtime_profile_id: &str,
    active_runtime_implementation_kind: &str,
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
            "activeRuntimeImplementationKind": active_runtime_implementation_kind,
            "targetProductionQualified": is_production_qualified_implementation_kind(&target.implementation_kind),
        }));
    }

    if !is_production_qualified_implementation_kind(&target.implementation_kind) {
        return Err(MemoryServiceError::validation(format!(
            "target implementation kind {} is evaluation-only and cannot be promoted",
            target.implementation_kind
        )));
    }
    if target.implementation_kind != active_runtime_implementation_kind {
        return Err(MemoryServiceError::validation(format!(
            "target implementation kind {} is not loaded by the active runtime {}; select a qualified runtime profile and restart before promotion",
            target.implementation_kind, active_runtime_implementation_kind
        )));
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
        "activeRuntimeImplementationKind": active_runtime_implementation_kind,
        "requiresRuntimeCutover": false,
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
                "disabled",
                r#"{"keyword":true,"productionQualified":false}"#,
                None,
                None,
            )
            .await
            .unwrap();
        store
            .insert_mem_implementation_profile(
                1,
                "3",
                "local-embedded-qualified",
                "local_embedded",
                "secondary",
                "active",
                r#"{"keyword":true,"embedding":false}"#,
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
            reason: "migration contract test".to_string(),
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
            "local_embedded",
        )
        .await
        .unwrap();
        assert_eq!(result["migrationScope"], "control_plane_only");
        assert_eq!(result["liveRuntimeCutover"], false);
        assert_eq!(result["activeRuntimeProfileId"], "local-embedded-phase1");
    }

    #[tokio::test]
    async fn switch_rejects_evaluation_only_target() {
        let store = migration_store().await;
        let result = execute_implementation_profile_migration(
            &store,
            1,
            &request("switch"),
            "local-embedded-phase1",
            "local_embedded",
        )
        .await
        .expect_err("evaluation-only implementation must not be promoted");
        assert_eq!(
            result.kind,
            sdkwork_memory_contract::MemoryServiceErrorKind::Validation
        );
    }

    #[tokio::test]
    async fn switch_only_promotes_target_loaded_by_active_runtime() {
        let store = migration_store().await;
        let mut matching_request = request("switch");
        matching_request.target_implementation_profile_id = 3;
        let result = execute_implementation_profile_migration(
            &store,
            1,
            &matching_request,
            "local-embedded-phase1",
            "local_embedded",
        )
        .await
        .unwrap();
        assert_eq!(result["migrated"], true);
        assert_eq!(result["migrationScope"], "control_plane_only");
        assert_eq!(result["liveRuntimeCutover"], false);
        assert_eq!(result["requiresRuntimeCutover"], false);
    }
}
