use sdkwork_memory_plugin_native_sql::{native_sql_manifest, NativeSqlMemoryStore};

#[tokio::test]
async fn native_sql_executable_runtime_materializes_every_phase1_port() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite()
        .await
        .expect("in-memory native SQL store must open");
    let runtime = sdkwork_memory_plugin_native_sql::NativeSqlPhase1Runtime::from_store(store)
        .executable_plugin_runtime();

    for export in native_sql_manifest().port_exports {
        assert!(
            runtime.has_port(&export.port),
            "missing executable native SQL port {}",
            export.port
        );
    }
}
