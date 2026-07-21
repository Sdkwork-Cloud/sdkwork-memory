use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::create_pool_from_config;
use sdkwork_memory_database_host::bootstrap_memory_database;

#[tokio::test]
#[ignore = "run through pnpm test:postgres:contract against ephemeral PostgreSQL"]
async fn canonical_postgres_migrations_are_complete_and_idempotent() {
    let database_url = std::env::var("SDKWORK_MEMORY_POSTGRES_LIFECYCLE_TEST_URL")
        .expect("SDKWORK_MEMORY_POSTGRES_LIFECYCLE_TEST_URL must be configured");
    let config = DatabaseConfig {
        engine: DatabaseEngine::Postgres,
        url: database_url,
        max_connections: 2,
        ..DatabaseConfig::default()
    };
    let pool = create_pool_from_config(config)
        .await
        .expect("create PostgreSQL lifecycle pool");

    std::env::set_var("SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE", "true");
    let first = bootstrap_memory_database(pool.clone())
        .await
        .expect("apply canonical PostgreSQL migrations");
    bootstrap_memory_database(pool.clone())
        .await
        .expect("repeat canonical PostgreSQL migrations");
    std::env::remove_var("SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE");

    let postgres = first.pool().as_postgres().expect("PostgreSQL pool");
    let applied: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ops_schema_migration_history WHERE module_id = 'memory'",
    )
    .fetch_one(postgres)
    .await
    .expect("read PostgreSQL migration history");
    assert_eq!(applied, 9);

    for table in [
        "ai_space",
        "ai_event",
        "ai_record",
        "ai_learning_job",
        "ai_subject",
        "ai_commercial_readiness_snapshot",
    ] {
        let exists: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(table)
            .fetch_one(postgres)
            .await
            .expect("inspect migrated PostgreSQL schema");
        assert!(exists, "missing migrated table {table}");
    }

    let search_document_type: Option<String> = sqlx::query_scalar(
        "SELECT data_type FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = 'ai_record' AND column_name = 'search_document'",
    )
    .fetch_optional(postgres)
    .await
    .expect("inspect PostgreSQL search_document");
    assert_eq!(search_document_type.as_deref(), Some("tsvector"));

    let outbox_lease_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = 'ai_outbox_event' AND column_name IN ('lease_owner', 'lease_token', 'lease_expires_at', 'next_attempt_at')",
    )
    .fetch_one(postgres)
    .await
    .expect("inspect PostgreSQL outbox lease columns");
    assert_eq!(outbox_lease_columns, 4);

    let job_lease_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_schema = current_schema() AND table_name IN ('ai_learning_job', 'ai_eval_run') AND column_name IN ('lease_owner', 'lease_token', 'lease_expires_at')",
    )
    .fetch_one(postgres)
    .await
    .expect("inspect PostgreSQL job lease columns");
    assert_eq!(job_lease_columns, 6);
}
