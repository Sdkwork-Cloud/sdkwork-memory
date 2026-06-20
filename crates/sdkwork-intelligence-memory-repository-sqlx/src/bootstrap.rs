//! SDKWork Memory database pool bootstrap via `sdkwork-database`.

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolError};

pub use sdkwork_memory_database_host::{
    bootstrap_memory_database, bootstrap_memory_database_from_env, MemoryDatabaseHost,
};

pub async fn connect_and_bootstrap_memory_database_from_env() -> Result<MemoryDatabaseHost, String> {
    let config = DatabaseConfig::from_env("MEMORY")
        .map_err(|error| error.to_string())?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| error.to_string())?;
    bootstrap_memory_database(pool).await
}

pub async fn connect_memory_database_pool_from_env() -> Result<DatabasePool, PoolError> {
    let config = DatabaseConfig::from_env("MEMORY")?;
    create_pool_from_config(config).await
}
