//! SDKWork Memory database pool bootstrap via `sdkwork-database`.

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::create_pool_from_config;
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;

pub use sdkwork_memory_database_host::{
    bootstrap_memory_database, bootstrap_memory_database_from_env, MemoryDatabaseHost,
};

use crate::db::{
    connect_memory_pool_from_env, install_sqlite_schema, open_native_sql_store_from_pool,
    MemoryDatabasePool,
};

pub struct MemoryDataPlane {
    pub pool: MemoryDatabasePool,
    pub store: NativeSqlMemoryStore,
}

pub async fn connect_and_bootstrap_memory_database_from_env() -> Result<MemoryDatabaseHost, String> {
    let config = DatabaseConfig::from_env("MEMORY")
        .map_err(|error| error.to_string())?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| error.to_string())?;
    bootstrap_memory_database(pool).await
}

/// Single bootstrap entry for the API server and integration tests.
pub async fn bootstrap_memory_data_plane_from_env() -> Result<MemoryDataPlane, String> {
    let pool = connect_memory_pool_from_env()
        .await
        .map_err(|error| error.to_string())?;

    if pool.as_postgres().is_some() {
        bootstrap_memory_database(pool.clone()).await?;
    } else {
        install_sqlite_schema(&pool).await?;
    }

    let store = open_native_sql_store_from_pool(&pool).await?;
    Ok(MemoryDataPlane { pool, store })
}
