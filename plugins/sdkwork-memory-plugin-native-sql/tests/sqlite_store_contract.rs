use sdkwork_memory_plugin_native_sql::{NativeSqlMemoryStore, NativeSqlStoreError};
use sdkwork_memory_spi::{
    AppendMemoryAuditCommand, AppendMemoryEventCommand, CreateMemoryRecordCommand,
    MemoryAuditStorePort, MemoryEventStorePort, MemoryRecordStorePort, MemoryScopeContext,
    MemorySpiError, RetrieveMemoryAuditQuery, RetrieveMemoryEventQuery, RetrieveMemoryRecordQuery,
};

#[tokio::test]
async fn sqlite_store_applies_phase1_migration_and_round_trips_event_and_record() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .append_event(&scope, "evt-1", "User prefers concise answers")
        .await
        .unwrap();
    store
        .create_record(&scope, "rec-1", "answer_style", "concise")
        .await
        .unwrap();

    let event = store
        .retrieve_event(&scope, "evt-1")
        .await
        .unwrap()
        .unwrap();
    let record = store
        .retrieve_record(&scope, "rec-1")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(event.event_id, "evt-1");
    assert_eq!(event.content, "User prefers concise answers");
    assert_eq!(record.memory_id, "rec-1");
    assert_eq!(record.content, "concise");
}

#[tokio::test]
async fn sqlite_store_preserves_event_content_with_json_sensitive_characters() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    let content = r#"User said "use C:\sdkwork\memory" for local tests"#;
    store
        .append_event(&scope, "evt-json", content)
        .await
        .unwrap();

    let event = store
        .retrieve_event(&scope, "evt-json")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(event.content, content);
}

#[tokio::test]
async fn sqlite_store_reads_event_payload_as_structured_json() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    let content = "line one\nline two";
    store
        .append_event(&scope, "evt-payload", content)
        .await
        .unwrap();

    let payload = store
        .retrieve_event_payload(&scope, "evt-payload")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(payload["content"].as_str(), Some(content));
}

#[tokio::test]
async fn sqlite_store_implements_record_and_event_store_spi_ports() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    let event = MemoryEventStorePort::append(
        &store,
        AppendMemoryEventCommand {
            scope: scope.clone(),
            event_id: "evt-spi".to_string(),
            content: "SPI event payload".to_string(),
        },
    )
    .await
    .unwrap();
    let record = MemoryRecordStorePort::create(
        &store,
        CreateMemoryRecordCommand {
            scope: scope.clone(),
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

#[tokio::test]
async fn sqlite_store_keeps_records_and_events_isolated_by_tenant_and_space() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);
    let wrong_space = MemoryScopeContext::for_test(1, 2);

    store
        .append_event(&tenant_one, "evt-shared", "tenant one event")
        .await
        .unwrap();
    store
        .append_event(&tenant_two, "evt-shared", "tenant two event")
        .await
        .unwrap();
    store
        .create_record(&tenant_one, "rec-shared", "preference", "tenant one record")
        .await
        .unwrap();
    store
        .create_record(&tenant_two, "rec-shared", "preference", "tenant two record")
        .await
        .unwrap();

    let tenant_one_event = store
        .retrieve_event(&tenant_one, "evt-shared")
        .await
        .unwrap()
        .unwrap();
    let tenant_two_event = store
        .retrieve_event(&tenant_two, "evt-shared")
        .await
        .unwrap()
        .unwrap();
    let tenant_one_record = store
        .retrieve_record(&tenant_one, "rec-shared")
        .await
        .unwrap()
        .unwrap();
    let tenant_two_record = store
        .retrieve_record(&tenant_two, "rec-shared")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tenant_one_event.content, "tenant one event");
    assert_eq!(tenant_two_event.content, "tenant two event");
    assert_eq!(tenant_one_record.content, "tenant one record");
    assert_eq!(tenant_two_record.content, "tenant two record");
    assert!(store
        .retrieve_event(&wrong_space, "evt-shared")
        .await
        .unwrap()
        .is_none());
    assert!(store
        .retrieve_record(&wrong_space, "rec-shared")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn sqlite_store_spi_retrieve_methods_require_matching_scope() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);

    MemoryEventStorePort::append(
        &store,
        AppendMemoryEventCommand {
            scope: tenant_one.clone(),
            event_id: "evt-spi-scoped".to_string(),
            content: "tenant one event".to_string(),
        },
    )
    .await
    .unwrap();
    MemoryRecordStorePort::create(
        &store,
        CreateMemoryRecordCommand {
            scope: tenant_one.clone(),
            memory_id: "rec-spi-scoped".to_string(),
            content: "tenant one record".to_string(),
        },
    )
    .await
    .unwrap();

    assert!(MemoryEventStorePort::retrieve(
        &store,
        RetrieveMemoryEventQuery {
            scope: tenant_two.clone(),
            event_id: "evt-spi-scoped".to_string(),
        },
    )
    .await
    .unwrap()
    .is_none());
    assert!(MemoryRecordStorePort::retrieve(
        &store,
        RetrieveMemoryRecordQuery {
            scope: tenant_two,
            memory_id: "rec-spi-scoped".to_string(),
        },
    )
    .await
    .unwrap()
    .is_none());
}

#[tokio::test]
async fn sqlite_store_event_append_is_idempotent_for_same_scope_event_and_content() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .append_event(&scope, "evt-idempotent", "same content")
        .await
        .unwrap();
    store
        .append_event(&scope, "evt-idempotent", "same content")
        .await
        .unwrap();

    let event = store
        .retrieve_event(&scope, "evt-idempotent")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(event.content, "same content");
}

#[tokio::test]
async fn sqlite_store_event_append_rejects_same_scope_event_with_different_content() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .append_event(&scope, "evt-conflict", "alpha")
        .await
        .unwrap();
    let err = store
        .append_event(&scope, "evt-conflict", "omega")
        .await
        .unwrap_err();

    assert!(matches!(err, NativeSqlStoreError::EventConflict { .. }));
}

#[tokio::test]
async fn sqlite_store_event_append_rejects_same_tenant_event_reuse_in_different_space() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let first_space = MemoryScopeContext::for_test(1, 1);
    let second_space = MemoryScopeContext::for_test(1, 2);

    store
        .append_event(&first_space, "evt-space-conflict", "same content")
        .await
        .unwrap();
    let err = store
        .append_event(&second_space, "evt-space-conflict", "same content")
        .await
        .unwrap_err();

    assert!(matches!(err, NativeSqlStoreError::EventConflict { .. }));
    assert!(store
        .retrieve_event(&second_space, "evt-space-conflict")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn sqlite_store_spi_event_append_maps_idempotency_conflict_to_spi_conflict() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    MemoryEventStorePort::append(
        &store,
        AppendMemoryEventCommand {
            scope: scope.clone(),
            event_id: "evt-spi-conflict".to_string(),
            content: "alpha".to_string(),
        },
    )
    .await
    .unwrap();
    let err = MemoryEventStorePort::append(
        &store,
        AppendMemoryEventCommand {
            scope,
            event_id: "evt-spi-conflict".to_string(),
            content: "omega".to_string(),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(err, MemorySpiError::IdempotencyConflict { .. }));
}

#[tokio::test]
async fn sqlite_store_appends_and_retrieves_audit_records_by_scope() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);

    store
        .append_audit(
            &tenant_one,
            "aud-shared",
            "memory.record.created",
            "mem_record",
            "rec-1",
            "success",
        )
        .await
        .unwrap();
    store
        .append_audit(
            &tenant_two,
            "aud-shared",
            "memory.record.created",
            "mem_record",
            "rec-2",
            "success",
        )
        .await
        .unwrap();

    let tenant_one_audit = store
        .retrieve_audit(&tenant_one, "aud-shared")
        .await
        .unwrap()
        .unwrap();
    let tenant_two_audit = store
        .retrieve_audit(&tenant_two, "aud-shared")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tenant_one_audit.action, "memory.record.created");
    assert_eq!(tenant_one_audit.resource_id, "rec-1");
    assert_eq!(tenant_two_audit.resource_id, "rec-2");
    assert!(store
        .retrieve_audit(&MemoryScopeContext::for_test(3, 3), "aud-shared")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn sqlite_store_implements_audit_store_spi_port() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    let audit = MemoryAuditStorePort::append(
        &store,
        AppendMemoryAuditCommand {
            scope: scope.clone(),
            audit_id: "aud-spi".to_string(),
            action: "memory.event.appended".to_string(),
            resource_type: "mem_event".to_string(),
            resource_id: "evt-spi".to_string(),
            result: "success".to_string(),
        },
    )
    .await
    .unwrap();
    let retrieved = MemoryAuditStorePort::retrieve(
        &store,
        RetrieveMemoryAuditQuery {
            scope,
            audit_id: "aud-spi".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(audit.audit_id, "aud-spi");
    assert_eq!(retrieved.action, "memory.event.appended");
    assert_eq!(retrieved.resource_type, "mem_event");
    assert_eq!(retrieved.resource_id, "evt-spi");
    assert_eq!(retrieved.result, "success");
}
