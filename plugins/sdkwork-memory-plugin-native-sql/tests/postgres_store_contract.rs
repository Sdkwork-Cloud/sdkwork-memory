//! Optional Postgres contract test — set `SDKWORK_MEMORY_POSTGRES_TEST_URL` to run.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_memory_spi::{
    AppendMemoryEventCommand, CreateMemoryRecordCommand, MemoryEventStorePort,
    MemoryRecordStorePort, MemoryScopeContext,
};

#[tokio::test]
async fn postgres_store_applies_phase1_migration_when_url_configured() {
    let url = match std::env::var("SDKWORK_MEMORY_POSTGRES_TEST_URL") {
        Ok(url) if !url.trim().is_empty() => url,
        _ => {
            eprintln!(
                "skip postgres_store_applies_phase1_migration_when_url_configured: \
                 set SDKWORK_MEMORY_POSTGRES_TEST_URL to a writable database"
            );
            return;
        }
    };

    let config = DatabaseConfig {
        engine: DatabaseEngine::Postgres,
        url,
        max_connections: 2,
        ..DatabaseConfig::default()
    };
    let store = NativeSqlMemoryStore::connect(&config)
        .await
        .expect("postgres connect and migration must succeed");
    store.ping().await.expect("postgres ping must succeed");

    let scope = MemoryScopeContext::for_test(100_001, 42);
    MemoryEventStorePort::append(
        &store,
        AppendMemoryEventCommand {
            scope: scope.clone(),
            event_id: "pg-event-1".to_string(),
            content: "postgres contract probe".to_string(),
        },
    )
    .await
    .expect("append event on postgres");

    MemoryRecordStorePort::create(
        &store,
        CreateMemoryRecordCommand {
            scope,
            memory_id: "pg-rec-1".to_string(),
            content: "postgres contract probe".to_string(),
        },
    )
    .await
    .expect("create record on postgres");
}
