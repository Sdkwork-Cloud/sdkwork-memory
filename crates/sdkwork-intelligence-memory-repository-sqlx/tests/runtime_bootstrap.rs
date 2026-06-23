use sdkwork_intelligence_memory_repository_sqlx::bootstrap_memory_runtime_from_env;
use sdkwork_memory_contract::runtime_env::env_test_lock;

#[tokio::test]
async fn bootstrap_memory_runtime_from_env_with_sqlite() {
    let _guard = env_test_lock();
    let previous_url = std::env::var("SDKWORK_MEMORY_DATABASE_URL").ok();
    std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", "sqlite::memory:");

    let runtime = bootstrap_memory_runtime_from_env()
        .await
        .expect("runtime bootstrap must succeed with in-memory sqlite");

    assert_eq!(
        runtime.primary_plugin_id,
        sdkwork_memory_plugin_native_sql::NATIVE_SQL_PLUGIN_ID
    );
    assert_eq!(runtime.profile_id, "native-sql-phase1");
    assert!(runtime.registry.get(&runtime.primary_plugin_id).is_some());
    assert!(
        runtime.data_plane.host_pool.is_none(),
        "sqlite bootstrap must not allocate an unused database-host pool"
    );
    runtime
        .data_plane
        .store()
        .ping()
        .await
        .expect("store ping must succeed");

    match previous_url {
        Some(value) => std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", value),
        None => std::env::remove_var("SDKWORK_MEMORY_DATABASE_URL"),
    }
}
