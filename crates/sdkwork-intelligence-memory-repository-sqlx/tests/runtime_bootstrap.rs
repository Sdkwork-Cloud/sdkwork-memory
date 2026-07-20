use sdkwork_intelligence_memory_repository_sqlx::{
    bootstrap_memory_plugin_registry, bootstrap_memory_runtime_from_env,
    resolve_memory_deployment_mode_from_env, resolve_memory_profile_for_runtime,
    resolve_memory_retrieval_strategy_from_env, resolve_native_sql_profile_for_dialect,
    resolve_native_sql_profile_for_runtime, MemoryImplementationProfileSelection,
};
use sdkwork_memory_contract::runtime_env::env_test_lock;
use sdkwork_memory_plugin_native_sql::MemorySqlDialect;
use sdkwork_memory_retrieval::MemoryRetrievalStrategy;
use sdkwork_memory_spi::{MemoryDeploymentMode, MemoryImplementationKind};

#[test]
fn native_sql_profile_selection_matches_database_dialect() {
    let registry = bootstrap_memory_plugin_registry();

    let postgres = resolve_native_sql_profile_for_dialect(&registry, MemorySqlDialect::Postgres)
        .expect("postgres profile must resolve");
    assert_eq!(postgres.profile_id, "native-sql-phase1");
    assert_eq!(
        postgres.implementation_kind,
        MemoryImplementationKind::NativeSql
    );
    assert_eq!(postgres.deployment_mode, MemoryDeploymentMode::Server);

    let sqlite = resolve_native_sql_profile_for_dialect(&registry, MemorySqlDialect::Sqlite)
        .expect("sqlite profile must resolve");
    assert_eq!(sqlite.profile_id, "local-embedded-phase1");
    assert_eq!(
        sqlite.implementation_kind,
        MemoryImplementationKind::LocalEmbedded
    );
    assert_eq!(sqlite.deployment_mode, MemoryDeploymentMode::Local);
}

#[test]
fn native_sql_profile_selection_separates_dialect_from_runtime_target() {
    let registry = bootstrap_memory_plugin_registry();

    let container = resolve_native_sql_profile_for_runtime(
        &registry,
        MemorySqlDialect::Postgres,
        MemoryDeploymentMode::Container,
    )
    .expect("postgres container profile must resolve");
    assert_eq!(container.profile_id, "native-sql-phase1");
    assert_eq!(container.deployment_mode, MemoryDeploymentMode::Container);

    let test_runner = resolve_native_sql_profile_for_runtime(
        &registry,
        MemorySqlDialect::Sqlite,
        MemoryDeploymentMode::Test,
    )
    .expect("sqlite test-runner profile must resolve");
    assert_eq!(test_runner.profile_id, "local-embedded-phase1");
    assert_eq!(test_runner.deployment_mode, MemoryDeploymentMode::Test);
}

#[test]
fn explicit_implementation_selection_is_typed_and_dialect_safe() {
    let registry = bootstrap_memory_plugin_registry();
    let native = resolve_memory_profile_for_runtime(
        &registry,
        MemorySqlDialect::Postgres,
        MemoryDeploymentMode::Container,
        MemoryImplementationProfileSelection::NativeSql,
    )
    .expect("explicit native SQL profile must resolve on PostgreSQL");
    assert_eq!(
        native.implementation_kind,
        MemoryImplementationKind::NativeSql
    );

    assert!(resolve_memory_profile_for_runtime(
        &registry,
        MemorySqlDialect::Sqlite,
        MemoryDeploymentMode::Local,
        MemoryImplementationProfileSelection::NativeSql,
    )
    .is_err());
    assert!(resolve_memory_profile_for_runtime(
        &registry,
        MemorySqlDialect::Postgres,
        MemoryDeploymentMode::Server,
        MemoryImplementationProfileSelection::LocalEmbedded,
    )
    .is_err());
}

#[test]
fn runtime_target_env_is_validated_separately_from_database_dialect() {
    let _guard = env_test_lock();
    let previous_target = std::env::var("SDKWORK_MEMORY_RUNTIME_TARGET").ok();

    std::env::set_var("SDKWORK_MEMORY_RUNTIME_TARGET", "container");
    assert_eq!(
        resolve_memory_deployment_mode_from_env(MemorySqlDialect::Postgres)
            .expect("container target must resolve"),
        MemoryDeploymentMode::Container
    );

    std::env::set_var("SDKWORK_MEMORY_RUNTIME_TARGET", "desktop");
    assert!(resolve_memory_deployment_mode_from_env(MemorySqlDialect::Sqlite).is_err());

    match previous_target {
        Some(value) => std::env::set_var("SDKWORK_MEMORY_RUNTIME_TARGET", value),
        None => std::env::remove_var("SDKWORK_MEMORY_RUNTIME_TARGET"),
    }
}

#[test]
fn retrieval_strategy_env_rejects_unimplemented_schemes() {
    let _guard = env_test_lock();
    let previous = std::env::var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY").ok();

    std::env::set_var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY", "event_aware");
    assert_eq!(
        resolve_memory_retrieval_strategy_from_env().unwrap(),
        MemoryRetrievalStrategy::EventAware
    );
    std::env::set_var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY", "graph_magic");
    assert!(resolve_memory_retrieval_strategy_from_env().is_err());

    match previous {
        Some(value) => std::env::set_var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY", value),
        None => std::env::remove_var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY"),
    }
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // Serializes process-wide environment mutation for the full bootstrap.
async fn bootstrap_memory_runtime_from_env_with_sqlite() {
    let _guard = env_test_lock();
    let previous_url = std::env::var("SDKWORK_MEMORY_DATABASE_URL").ok();
    let previous_target = std::env::var("SDKWORK_MEMORY_RUNTIME_TARGET").ok();
    let previous_implementation = std::env::var("SDKWORK_MEMORY_IMPLEMENTATION_PROFILE").ok();
    let previous_retrieval = std::env::var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY").ok();
    std::env::set_var("SDKWORK_MEMORY_DATABASE_URL", "sqlite::memory:");
    std::env::set_var("SDKWORK_MEMORY_RUNTIME_TARGET", "test-runner");
    std::env::set_var("SDKWORK_MEMORY_IMPLEMENTATION_PROFILE", "local_embedded");
    std::env::set_var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY", "search_first");

    let runtime = bootstrap_memory_runtime_from_env()
        .await
        .expect("runtime bootstrap must succeed with in-memory sqlite");

    assert_eq!(
        runtime.profile.primary_plugin_id,
        sdkwork_memory_plugin_native_sql::NATIVE_SQL_PLUGIN_ID
    );
    assert_eq!(runtime.profile.profile_id, "local-embedded-phase1");
    assert_eq!(runtime.profile.deployment_mode, MemoryDeploymentMode::Test);
    assert_eq!(
        runtime.retrieval_strategy,
        MemoryRetrievalStrategy::SearchFirst
    );
    assert_eq!(
        runtime.core_runtime.profile().profile_id,
        runtime.profile.profile_id
    );
    for port in [
        "MemoryRecordStorePort",
        "MemoryEventStorePort",
        "MemoryAuditStorePort",
        "MemoryOutboxStorePort",
        "MemoryCandidateStorePort",
        "MemoryHabitStorePort",
        "MemoryRetrievalTraceStorePort",
        "MemoryGovernanceAccessPort",
        "MemorySpaceStorePort",
        "MemoryRetrieverPort",
    ] {
        assert!(runtime.core_runtime.has_port(port));
        assert_eq!(
            runtime.core_runtime.port_owner(port),
            Some(sdkwork_memory_plugin_native_sql::NATIVE_SQL_PLUGIN_ID)
        );
    }
    assert!(runtime
        .registry
        .get(&runtime.profile.primary_plugin_id)
        .is_some());
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
    match previous_target {
        Some(value) => std::env::set_var("SDKWORK_MEMORY_RUNTIME_TARGET", value),
        None => std::env::remove_var("SDKWORK_MEMORY_RUNTIME_TARGET"),
    }
    match previous_implementation {
        Some(value) => std::env::set_var("SDKWORK_MEMORY_IMPLEMENTATION_PROFILE", value),
        None => std::env::remove_var("SDKWORK_MEMORY_IMPLEMENTATION_PROFILE"),
    }
    match previous_retrieval {
        Some(value) => std::env::set_var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY", value),
        None => std::env::remove_var("SDKWORK_MEMORY_RETRIEVAL_STRATEGY"),
    }
}
