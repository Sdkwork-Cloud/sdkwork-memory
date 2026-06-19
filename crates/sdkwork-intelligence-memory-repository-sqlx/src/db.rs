use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolError};

pub type MemoryDatabasePool = DatabasePool;

pub async fn connect_memory_pool_from_env() -> Result<MemoryDatabasePool, PoolError> {
    let config = DatabaseConfig::from_env("memory")?;
    create_pool_from_config(config).await
}

pub async fn install_sqlite_schema(pool: &MemoryDatabasePool) -> Result<(), sqlx::Error> {
    if let Some(sqlite) = pool.as_sqlite() {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS mem_space (
                id TEXT PRIMARY KEY NOT NULL,
                tenant_id TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .execute(sqlite)
        .await?;
    }
    Ok(())
}
