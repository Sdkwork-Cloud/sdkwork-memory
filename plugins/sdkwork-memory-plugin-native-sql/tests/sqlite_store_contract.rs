use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_memory_spi::{
    AppendMemoryEventCommand, CreateMemoryRecordCommand, MemoryEventStorePort,
    MemoryRecordStorePort,
};

#[tokio::test]
async fn sqlite_store_applies_phase1_migration_and_round_trips_event_and_record() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();

    store
        .append_event("evt-1", "User prefers concise answers")
        .await
        .unwrap();
    store
        .create_record("rec-1", "answer_style", "concise")
        .await
        .unwrap();

    let event = store.retrieve_event("evt-1").await.unwrap().unwrap();
    let record = store.retrieve_record("rec-1").await.unwrap().unwrap();

    assert_eq!(event.event_id, "evt-1");
    assert_eq!(event.content, "User prefers concise answers");
    assert_eq!(record.memory_id, "rec-1");
    assert_eq!(record.content, "concise");
}

#[tokio::test]
async fn sqlite_store_preserves_event_content_with_json_sensitive_characters() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();

    let content = r#"User said "use C:\sdkwork\memory" for local tests"#;
    store.append_event("evt-json", content).await.unwrap();

    let event = store.retrieve_event("evt-json").await.unwrap().unwrap();

    assert_eq!(event.content, content);
}

#[tokio::test]
async fn sqlite_store_reads_event_payload_as_structured_json() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();

    let content = "line one\nline two";
    store.append_event("evt-payload", content).await.unwrap();

    let payload = store
        .retrieve_event_payload("evt-payload")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(payload["content"].as_str(), Some(content));
}

#[tokio::test]
async fn sqlite_store_implements_record_and_event_store_spi_ports() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();

    let event = MemoryEventStorePort::append(
        &store,
        AppendMemoryEventCommand {
            event_id: "evt-spi".to_string(),
            content: "SPI event payload".to_string(),
        },
    )
    .await
    .unwrap();
    let record = MemoryRecordStorePort::create(
        &store,
        CreateMemoryRecordCommand {
            memory_id: "rec-spi".to_string(),
            content: "SPI record payload".to_string(),
        },
    )
    .await
    .unwrap();

    assert_eq!(event.event_id, "evt-spi");
    assert_eq!(event.content, "SPI event payload");
    assert_eq!(record.memory_id, "rec-spi");
    assert_eq!(record.content, "SPI record payload");
}
