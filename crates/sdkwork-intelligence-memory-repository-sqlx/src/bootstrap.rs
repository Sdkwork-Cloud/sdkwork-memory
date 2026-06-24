//! SDKWork Memory database pool bootstrap via `sdkwork-database`.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::create_pool_from_config;
use sdkwork_memory_plugin_native_sql::{
    normalize_memory_database_config, NativeSqlMemoryStore, NativeSqlPhase1Runtime,
};

use crate::db::{open_native_sql_store_from_pool, MemoryDatabasePool};

pub use sdkwork_memory_database_host::{
    bootstrap_memory_database, bootstrap_memory_database_from_env, MemoryDatabaseHost,
};

/// Materialized phase-1 SQL runtime plus optional postgres host pool for database-host migrations.
pub struct MemoryDataPlane {
    pub phase1: NativeSqlPhase1Runtime,
    /// Set when postgres host bootstrap runs via `sdkwork-memory-database-host`.
    pub host_pool: Option<MemoryDatabasePool>,
}

impl MemoryDataPlane {
    pub fn store(&self) -> &NativeSqlMemoryStore {
        self.phase1.store()
    }
}

pub async fn connect_and_bootstrap_memory_database_from_env() -> Result<MemoryDatabaseHost, String> {
    let config = DatabaseConfig::from_env("MEMORY").map_err(|error| error.to_string())?;
    let config = normalize_memory_database_config(config);
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| error.to_string())?;
    bootstrap_memory_database(pool).await
}

/// Single bootstrap entry for the API server and integration tests.
pub async fn bootstrap_memory_data_plane_from_env() -> Result<MemoryDataPlane, String> {
    let config = DatabaseConfig::from_env("MEMORY").map_err(|error| error.to_string())?;
    let config = normalize_memory_database_config(config);

    let auto_migrate = std::env::var("SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE")
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false);

    let host_pool = if config.engine == DatabaseEngine::Postgres && auto_migrate {
        let pool = create_pool_from_config(config.clone())
            .await
            .map_err(|error| error.to_string())?;
        bootstrap_memory_database(pool.clone()).await?;
        Some(pool)
    } else {
        None
    };

    let phase1 = if let Some(ref pool) = host_pool {
        let store = open_native_sql_store_from_pool(pool)
            .await
            .map_err(|error| error.to_string())?;
        NativeSqlPhase1Runtime::from_store(store)
    } else {
        NativeSqlPhase1Runtime::connect(&config)
            .await
            .map_err(|error| error.to_string())?
    };
    sdkwork_memory_plugin_native_sql::validate_native_sql_phase1_ports(phase1.store())
        .await
        .map_err(|error| error.to_string())?;

    Ok(MemoryDataPlane { phase1, host_pool })
}
