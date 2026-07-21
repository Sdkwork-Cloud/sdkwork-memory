use std::path::PathBuf;

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::create_pool_from_config;
use sdkwork_memory_database_host::bootstrap_memory_database;

#[tokio::test]
async fn canonical_sqlite_migrations_are_complete_and_idempotent() {
    let database_path = temporary_database_path();
    let database_url = format!("sqlite://{}?mode=rwc", database_path.display());
    let config = DatabaseConfig {
        engine: DatabaseEngine::Sqlite,
        url: database_url,
        max_connections: 1,
        ..DatabaseConfig::default()
    };
    let pool = create_pool_from_config(config)
        .await
        .expect("create SQLite lifecycle pool");

    std::env::set_var("SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE", "true");
    let first = bootstrap_memory_database(pool.clone())
        .await
        .expect("apply canonical SQLite migrations");
    bootstrap_memory_database(pool.clone())
        .await
        .expect("repeat canonical SQLite migrations");
    std::env::remove_var("SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE");

    let sqlite = first.pool().as_sqlite().expect("SQLite pool");
    let applied: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ops_schema_migration_history WHERE module_id = 'memory'",
    )
    .fetch_one(sqlite)
    .await
    .expect("read migration history");
    assert_eq!(applied, 10);

    for table in [
        "ai_space",
        "ai_event",
        "ai_record",
        "ai_learning_job",
        "ai_subject",
        "ai_commercial_readiness_snapshot",
        "ai_record_fts",
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE name = ? AND type IN ('table', 'view')",
        )
        .bind(table)
        .fetch_one(sqlite)
        .await
        .expect("inspect migrated SQLite schema");
        assert_eq!(exists, 1, "missing migrated table {table}");
    }

    let outbox_lease_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('ai_outbox_event') WHERE name IN ('lease_owner', 'lease_token', 'lease_expires_at', 'next_attempt_at')",
    )
    .fetch_one(sqlite)
    .await
    .expect("inspect SQLite outbox lease columns");
    assert_eq!(outbox_lease_columns, 4);

    let learning_lease_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('ai_learning_job') WHERE name IN ('lease_owner', 'lease_token', 'lease_expires_at')",
    )
    .fetch_one(sqlite)
    .await
    .expect("inspect SQLite learning job lease columns");
    let eval_lease_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('ai_eval_run') WHERE name IN ('lease_owner', 'lease_token', 'lease_expires_at')",
    )
    .fetch_one(sqlite)
    .await
    .expect("inspect SQLite eval run lease columns");
    assert_eq!(learning_lease_columns, 3);
    assert_eq!(eval_lease_columns, 3);

    drop(first);
    drop(pool);
    let _ = std::fs::remove_file(database_path);
}

fn temporary_database_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "sdkwork-memory-lifecycle-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos()
    ))
}
