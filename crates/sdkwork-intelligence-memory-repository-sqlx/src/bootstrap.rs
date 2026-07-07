//! SDKWork Memory database pool bootstrap via `sdkwork-database`.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_id::{NodeAllocatorConfig, SnowflakeNodeAllocator};
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
    let mut config = normalize_memory_database_config(config);

    // Apply pool tuning from environment (only for non-in-memory databases).
    if !config.url.contains("memory:") {
        if let Ok(value) = std::env::var("SDKWORK_MEMORY_DB_MAX_CONNECTIONS") {
            if let Ok(max_conn) = value.parse::<u32>() {
                if max_conn > 0 {
                    config.max_connections = max_conn;
                    tracing::info!(
                        max_connections = max_conn,
                        "memory database pool max_connections overridden from env"
                    );
                }
            }
        }
        if let Ok(value) = std::env::var("SDKWORK_MEMORY_DB_MIN_CONNECTIONS") {
            if let Ok(min_conn) = value.parse::<u32>() {
                config.min_connections = min_conn;
                tracing::info!(
                    min_connections = min_conn,
                    "memory database pool min_connections overridden from env"
                );
            }
        }
    }

    tracing::info!(
        engine = ?config.engine,
        max_connections = config.max_connections,
        min_connections = config.min_connections,
        url_safe = !config.url.contains("password"),
        "memory database pool configured"
    );

    let auto_migrate = std::env::var("SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE")
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false);

    // Guard: reject SQLite in production-like environments.
    if sdkwork_memory_contract::memory_is_production_like_environment() && config.engine == DatabaseEngine::Sqlite {
        return Err(
            "production-like environment detected with SQLite engine — "
                .to_string()
                + "PostgreSQL is required for production deployments. "
                + "Set SDKWORK_MEMORY_DATABASE_ENGINE=postgres and provide a valid PostgreSQL connection URL.",
        );
    }

    // Always create a pool up front so we can use it for both Snowflake
    // node_id allocation and store creation. This is especially important
    // for SQLite in-memory databases which must share the same connection.
    let pool = create_pool_from_config(config.clone())
        .await
        .map_err(|error| format!("create memory database pool failed: {error}"))?;

    // Allocate a Snowflake node_id from the database before creating the
    // store. This prevents ID collisions in multi-instance deployments.
    allocate_and_init_snowflake_node(&pool).await?;

    // Create the phase-1 runtime from the shared pool to avoid duplicate connections.
    let (phase1, host_pool) = if config.engine == DatabaseEngine::Postgres {
        if auto_migrate {
            bootstrap_memory_database(pool.clone())
                .await
                .map_err(|error| format!("memory database migrate failed: {error}"))?;
        }
        let store = open_native_sql_store_from_pool(&pool)
            .await
            .map_err(|error| error.to_string())?;
        let phase1 = NativeSqlPhase1Runtime::from_store(store);
        (phase1, Some(pool))
    } else if auto_migrate {
        let phase1 = NativeSqlPhase1Runtime::connect(&config)
            .await
            .map_err(|error| error.to_string())?;
        (phase1, None)
    } else {
        let store = open_native_sql_store_from_pool(&pool)
            .await
            .map_err(|error| error.to_string())?;
        let phase1 = NativeSqlPhase1Runtime::from_store(store);
        (phase1, None)
    };

    sdkwork_memory_plugin_native_sql::validate_native_sql_phase1_ports(phase1.store())
        .await
        .map_err(|error| error.to_string())?;

    Ok(MemoryDataPlane { phase1, host_pool })
}

/// Allocate a Snowflake node_id from the database and initialize the
/// global ID generator.
///
/// Falls back to env/hostname hash if database allocation fails (e.g.
/// in dev/test environments without a persistent database).
async fn allocate_and_init_snowflake_node(pool: &MemoryDatabasePool) -> Result<(), String> {
    let config = NodeAllocatorConfig::from_service_name("memory-service");
    match SnowflakeNodeAllocator::allocate_generator(pool, &config).await {
        Ok((generator, lease)) => {
            let node_id = generator.node_id();
            tracing::info!(
                node_id,
                "memory snowflake node_id allocated from database registry"
            );
            sdkwork_intelligence_memory_service::platform::init_id_generator(
                generator,
                Some(lease),
            );
            Ok(())
        }
        Err(error) => {
            if sdkwork_memory_contract::memory_is_production_like_environment() {
                Err(format!(
                    "memory snowflake database node_id allocation failed in production-like environment: {error}"
                ))
            } else {
                tracing::warn!(
                    %error,
                    "memory snowflake database node_id allocation failed; \
                     dev fallback will be used on first ID generation"
                );
                Ok(())
            }
        }
    }
}
