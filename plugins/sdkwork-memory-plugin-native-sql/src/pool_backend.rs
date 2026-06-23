use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::any::create_any_pool;
use sqlx::AnyPool;

use crate::store::NativeSqlStoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySqlDialect {
    Sqlite,
    Postgres,
}

impl MemorySqlDialect {
    pub fn from_config(config: &DatabaseConfig) -> Self {
        match config.engine {
            DatabaseEngine::Postgres => Self::Postgres,
            DatabaseEngine::Sqlite => Self::Sqlite,
        }
    }
}

pub fn normalize_memory_database_url(url: &str) -> String {
    match url {
        "sqlite::memory:" | "sqlite:memory:" => "sqlite::memory:?cache=shared".to_string(),
        other => other.to_string(),
    }
}

pub fn normalize_memory_database_config(mut config: DatabaseConfig) -> DatabaseConfig {
    config.url = normalize_memory_database_url(&config.url);
    if config.url.contains("memory:") {
        config.max_connections = 1;
    }
    config
}

pub async fn connect_any_pool(config: &DatabaseConfig) -> Result<(AnyPool, MemorySqlDialect), NativeSqlStoreError> {
    sqlx::any::install_default_drivers();
    let config = normalize_memory_database_config(config.clone());
    let dialect = MemorySqlDialect::from_config(&config);
    let pool = create_any_pool(&config).await?;
    Ok((pool, dialect))
}
