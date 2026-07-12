//! Memory plugin registry bootstrap and startup validation.

use sdkwork_memory_plugin_native_sql::{
    native_sql_manifest, validate_native_sql_port_builders, MemorySqlDialect, NATIVE_SQL_PLUGIN_ID,
};
use sdkwork_memory_profile_resolver::{
    MemoryImplementationProfileDraft, MemoryRuntimeProfileResolver,
    ResolvedMemoryImplementationProfile,
};
use sdkwork_memory_spi::{MemoryCoreRuntime, MemoryDeploymentMode, MemoryPluginRegistry};

use crate::bootstrap::{bootstrap_memory_data_plane_from_env, MemoryDataPlane};

const NATIVE_SQL_REQUIRED_PORTS: &[&str] = &[
    "MemoryRecordStorePort",
    "MemoryEventStorePort",
    "MemoryAuditStorePort",
    "MemoryOutboxStorePort",
    "MemoryCandidateStorePort",
    "MemoryHabitStorePort",
    "MemoryRetrievalTraceStorePort",
    "MemoryGovernanceAccessPort",
    "MemorySpaceStorePort",
    "MemoryRetrieverPort",
];

/// Validated plugin registry plus materialized SQL data plane for phase 1.
pub struct MemoryRuntime {
    pub registry: MemoryPluginRegistry,
    pub data_plane: MemoryDataPlane,
    pub profile: ResolvedMemoryImplementationProfile,
    pub core_runtime: MemoryCoreRuntime,
    /// Compatibility projections; `profile` is the authoritative typed value.
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

    for profile in [
        MemoryImplementationProfileDraft::native_sql_phase1(),
        MemoryImplementationProfileDraft::local_embedded_phase1(),
    ] {
        let resolved = MemoryRuntimeProfileResolver::new(registry)
            .resolve(profile)
            .map_err(|error| error.to_string())?;
        if resolved.primary_plugin_id != NATIVE_SQL_PLUGIN_ID {
            return Err(format!(
                "native SQL profile must select plugin {NATIVE_SQL_PLUGIN_ID}, got {}",
                resolved.primary_plugin_id
            ));
        }
    }
    Ok(())
}

pub fn resolve_native_sql_phase1_profile(
    registry: &MemoryPluginRegistry,
) -> Result<(String, String), String> {
    let profile = resolve_native_sql_profile_for_dialect(registry, MemorySqlDialect::Postgres)?;
    Ok((profile.profile_id, profile.primary_plugin_id))
}

/// Resolves the executable native SQL profile that matches the materialized database dialect.
/// PostgreSQL is the server profile; SQLite is the local embedded profile.
pub fn resolve_native_sql_profile_for_dialect(
    registry: &MemoryPluginRegistry,
    dialect: MemorySqlDialect,
) -> Result<ResolvedMemoryImplementationProfile, String> {
    let deployment_mode = match dialect {
        MemorySqlDialect::Postgres => MemoryDeploymentMode::Server,
        MemorySqlDialect::Sqlite => MemoryDeploymentMode::Local,
    };
    resolve_native_sql_profile_for_runtime(registry, dialect, deployment_mode)
}

/// Resolves the native SQL implementation family independently from the process target.
pub fn resolve_native_sql_profile_for_runtime(
    registry: &MemoryPluginRegistry,
    dialect: MemorySqlDialect,
    deployment_mode: MemoryDeploymentMode,
) -> Result<ResolvedMemoryImplementationProfile, String> {
    let mut profile = match dialect {
        MemorySqlDialect::Postgres => MemoryImplementationProfileDraft::native_sql_phase1(),
        MemorySqlDialect::Sqlite => MemoryImplementationProfileDraft::local_embedded_phase1(),
    };
    profile.deployment_mode = deployment_mode;

    MemoryRuntimeProfileResolver::new(registry)
        .resolve(profile)
        .map_err(|error| error.to_string())
}

pub fn resolve_memory_deployment_mode_from_env(
    dialect: MemorySqlDialect,
) -> Result<MemoryDeploymentMode, String> {
    match std::env::var("SDKWORK_MEMORY_RUNTIME_TARGET") {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "server" => Ok(MemoryDeploymentMode::Server),
            "container" => Ok(MemoryDeploymentMode::Container),
            "test-runner" => Ok(MemoryDeploymentMode::Test),
            other => Err(format!(
                "SDKWORK_MEMORY_RUNTIME_TARGET must be server, container, or test-runner for the Memory API runtime; got {other}"
            )),
        },
        Err(std::env::VarError::NotPresent) => Ok(match dialect {
            MemorySqlDialect::Postgres => MemoryDeploymentMode::Server,
            MemorySqlDialect::Sqlite => MemoryDeploymentMode::Local,
        }),
        Err(error) => Err(format!(
            "SDKWORK_MEMORY_RUNTIME_TARGET could not be read: {error}"
        )),
    }
}

/// Single startup entry: validate SPI registry, resolve profile, bootstrap SQL store.
pub async fn bootstrap_memory_runtime_from_env() -> Result<MemoryRuntime, String> {
    let mut registry = bootstrap_memory_plugin_registry();
    validate_memory_plugin_registry(&registry)?;
    let data_plane = bootstrap_memory_data_plane_from_env().await?;
    registry
        .register_executable_runtime(
            NATIVE_SQL_PLUGIN_ID,
            data_plane.phase1.executable_plugin_runtime(),
        )
        .map_err(|error| error.to_string())?;
    let dialect = data_plane.store().dialect();
    let deployment_mode = resolve_memory_deployment_mode_from_env(dialect)?;
    let profile = resolve_native_sql_profile_for_runtime(&registry, dialect, deployment_mode)?;
    let core_runtime = MemoryRuntimeProfileResolver::new(&registry)
        .assemble(&profile)
        .map_err(|error| error.to_string())?;
    let profile_id = profile.profile_id.clone();
    let primary_plugin_id = profile.primary_plugin_id.clone();
    Ok(MemoryRuntime {
        registry,
        data_plane,
        profile,
        core_runtime,
        profile_id,
        primary_plugin_id,
    })
}
