use sdkwork_memory_plugin_native_sql::{NativeSqlCreateSpaceCommand, NativeSqlMemoryStore};

pub async fn new_seeded_in_memory_store() -> NativeSqlMemoryStore {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite()
        .await
        .expect("in-memory sqlite store must initialize");
    seed_standard_integration_spaces(&store).await;
    store
}

pub async fn seed_user_space(
    store: &NativeSqlMemoryStore,
    tenant_id: i64,
    space_id: i64,
    owner_id: &str,
) {
    store
        .create_space_record(
            tenant_id,
            space_id,
            &NativeSqlCreateSpaceCommand {
                organization_id: None,
                owner_subject_type: "user".to_string(),
                owner_subject_id: owner_id.to_string(),
                space_type: "personal".to_string(),
                display_name: format!("Test Space {space_id}"),
                default_scope: "user".to_string(),
            },
        )
        .await
        .expect("seed_user_space must succeed in tests");
}

pub async fn seed_standard_integration_spaces(store: &NativeSqlMemoryStore) {
    seed_user_space(store, 100_001, 1, "9001").await;
    seed_user_space(store, 100_001, 2, "2001").await;
}
