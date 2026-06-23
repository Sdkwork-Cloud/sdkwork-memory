//! Memory plugin registry bootstrap and startup validation.

use sdkwork_memory_plugin_native_sql::{
    native_sql_manifest, validate_native_sql_port_builders, NATIVE_SQL_PLUGIN_ID,
};
use sdkwork_memory_runtime::{MemoryImplementationProfileDraft, MemoryRuntimeProfileResolver};
use sdkwork_memory_spi::MemoryPluginRegistry;

use crate::bootstrap::{bootstrap_memory_data_plane_from_env, MemoryDataPlane};

const NATIVE_SQL_REQUIRED_PORTS: &[&str] = &[
    "MemoryRecordStorePort",
    "MemoryEventStorePort",
    "MemoryAuditStorePort",
    "MemoryOutboxStorePort",
    "MemoryCandidateStorePort",
    "MemoryHabitStorePort",
    "MemoryRetrievalTraceStorePort",
];

/// Validated plugin registry plus materialized SQL data plane for phase 1.
pub struct MemoryRuntime {
    pub registry: MemoryPluginRegistry,
    pub data_plane: MemoryDataPlane,
    pub profile_id: String,
    pub primary_plugin_id: String,
}

pub fn bootstrap_memory_plugin_registry() -> MemoryPluginRegistry {
    let mut registry = MemoryPluginRegistry::default();
    registry
        .register(native_sql_manifest())
        .expect("native sql manifest must register");
    registry
}

pub fn validate_memory_plugin_registry(registry: &MemoryPluginRegistry) -> Result<(), String> {
    let manifest = registry
        .get(NATIVE_SQL_PLUGIN_ID)
        .ok_or_else(|| format!("plugin {NATIVE_SQL_PLUGIN_ID} is not registered"))?;
    validate_native_sql_port_builders(manifest)?;

    registry
        .validate_required_ports(NATIVE_SQL_PLUGIN_ID, NATIVE_SQL_REQUIRED_PORTS)
        .map_err(|error| error.to_string())?;

    let resolver = MemoryRuntimeProfileResolver::new(registry);
    let profile = resolver
        .resolve(MemoryImplementationProfileDraft::native_sql_phase1())
        .map_err(|error| error.to_string())?;
    if profile.primary_plugin_id != NATIVE_SQL_PLUGIN_ID {
        return Err(format!(
            "native sql profile must select plugin {NATIVE_SQL_PLUGIN_ID}, got {}",
            profile.primary_plugin_id
        ));
    }
    Ok(())
}

pub fn resolve_native_sql_phase1_profile(
    registry: &MemoryPluginRegistry,
) -> Result<(String, String), String> {
    let resolver = MemoryRuntimeProfileResolver::new(registry);
    let profile = resolver
        .resolve(MemoryImplementationProfileDraft::native_sql_phase1())
        .map_err(|error| error.to_string())?;
    Ok((profile.profile_id, profile.primary_plugin_id))
}

/// Single startup entry: validate SPI registry, resolve profile, bootstrap SQL store.
pub async fn bootstrap_memory_runtime_from_env() -> Result<MemoryRuntime, String> {
    let registry = bootstrap_memory_plugin_registry();
    validate_memory_plugin_registry(&registry)?;
    let (profile_id, primary_plugin_id) = resolve_native_sql_phase1_profile(&registry)?;
    let data_plane = bootstrap_memory_data_plane_from_env().await?;
    Ok(MemoryRuntime {
        registry,
        data_plane,
        profile_id,
        primary_plugin_id,
    })
}
