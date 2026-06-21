use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolError};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;

pub type MemoryDatabasePool = DatabasePool;

pub async fn connect_memory_pool_from_env() -> Result<MemoryDatabasePool, PoolError> {
    // DATABASE_SPEC serviceCode `MEMORY` → env prefix `SDKWORK_MEMORY_*`
    let config = DatabaseConfig::from_env("MEMORY")?;
    create_pool_from_config(config).await
}

pub async fn install_sqlite_schema(pool: &MemoryDatabasePool) -> Result<(), String> {
    if let Some(sqlite) = pool.as_sqlite() {
        NativeSqlMemoryStore::install_sqlite_phase1_schema(sqlite)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub async fn open_native_sql_store_from_pool(
    pool: &MemoryDatabasePool,
) -> Result<sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore, String> {
    let sqlite = pool
        .as_sqlite()
        .ok_or_else(|| "memory database pool is not sqlite".to_string())?
        .clone();
    NativeSqlMemoryStore::from_sqlite_pool(sqlite)
        .await
        .map_err(|error| error.to_string())
}
