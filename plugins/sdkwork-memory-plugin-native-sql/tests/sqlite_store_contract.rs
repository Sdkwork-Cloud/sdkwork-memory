use sdkwork_memory_plugin_native_sql::{
    build_native_sql_candidate_store, build_native_sql_habit_store,
    build_native_sql_retrieval_trace_store, NativeSqlAppendOutboxEventCommand,
    NativeSqlCreateSpaceCommand, NativeSqlMemoryStore, NativeSqlStoreError,
};
use sdkwork_memory_spi::{
    AppendMemoryAuditCommand, AppendMemoryEventCommand, AppendMemoryOutboxCommand,
    AppendMemoryRetrievalTraceCommand, ApproveMemoryCandidateCommand, CreateMemoryCandidateCommand,
    CreateMemoryRecordCommand, DecayMemoryHabitCommand, DeleteMemoryRecordCommand,
    ListMemoryRetrievalTracesQuery, ListPendingMemoryOutboxQuery, MarkMemoryOutboxFailedCommand,
    MarkMemoryOutboxPublishedCommand, MemoryAuditStorePort, MemoryCandidateStorePort,
    MemoryContextPackSnapshot, MemoryEventStorePort, MemoryHabitStorePort, MemoryOutboxStorePort,
    MemoryRecordStorePort, MemoryRetrievalHitDraft, MemoryRetrievalTraceStorePort,
    MemoryScopeContext, MemorySpiError, PromoteMemoryHabitCommand, RejectMemoryCandidateCommand,
    RetrieveMemoryAuditQuery, RetrieveMemoryCandidateQuery, RetrieveMemoryEventQuery,
    RetrieveMemoryHabitQuery, RetrieveMemoryOutboxQuery, RetrieveMemoryRecordQuery,
    RetrieveMemoryRetrievalTraceQuery, UpsertMemoryHabitCommand,
};

fn assert_utc_timestamp(value: Option<&str>) {
    let Some(text) = value else {
        panic!("expected UTC timestamp");
    };
    assert!(text.ends_with('Z'), "timestamp must be UTC RFC3339: {text}");
}

fn outbox_command<'a>(
    scope: &'a MemoryScopeContext,
    outbox_id: &'a str,
    aggregate_id: &'a str,
    payload_json: &'a str,
) -> NativeSqlAppendOutboxEventCommand<'a> {
    NativeSqlAppendOutboxEventCommand {
        scope,
        outbox_id,
        aggregate_type: "mem_record",
        aggregate_id,
        event_type: "memory.record.created",
        event_version: "1",
        payload_json,
    }
}

fn candidate_command(
    scope: MemoryScopeContext,
    candidate_id: &str,
) -> CreateMemoryCandidateCommand {
    CreateMemoryCandidateCommand {
        scope,
        candidate_id: candidate_id.to_string(),
        candidate_type: "observation".to_string(),
        memory_type: "semantic".to_string(),
        proposed_text: "User prefers concise answers".to_string(),
        proposed_payload_json: Some(r#"{"preference":"concise"}"#.to_string()),
        evidence_json: Some(r#"{"eventId":"evt-1"}"#.to_string()),
        confidence: 0.91,
    }
}

fn habit_command(
    scope: MemoryScopeContext,
    habit_id: &str,
    user_id: i64,
) -> UpsertMemoryHabitCommand {
    UpsertMemoryHabitCommand {
        scope,
        habit_id: habit_id.to_string(),
        user_id,
        habit_key: "answer_style:concise".to_string(),
        habit_type: "preference".to_string(),
        description: "Prefers concise answers".to_string(),
        stage: "candidate".to_string(),
        strength: 0.4,
        confidence: 0.8,
        support_count: 2,
        metadata_json: Some(r#"{"source":"signals"}"#.to_string()),
    }
}

fn retrieval_trace_command(
    scope: MemoryScopeContext,
    trace_id: &str,
) -> AppendMemoryRetrievalTraceCommand {
    AppendMemoryRetrievalTraceCommand {
        scope,
        trace_id: trace_id.to_string(),
        actor_id: Some("user-42".to_string()),
        query_text: Some("concise answer preference".to_string()),
        query_hash: format!("hash:{trace_id}"),
        retrievers_json: Some(r#"["native_sql"]"#.to_string()),
        latency_ms: Some(17),
        degraded: false,
        metadata_json: Some(r#"{"profile":"native_sql"}"#.to_string()),
        hits: vec![
            MemoryRetrievalHitDraft {
                hit_id: format!("{trace_id}-hit-1"),
                memory_id: Some("rec-trace-1".to_string()),
                retriever_name: "native_sql".to_string(),
                result_rank: 1,
                raw_score: Some(0.75),
                fused_score: Some(0.9),
                explanation_json: Some(r#"{"match":"keyword"}"#.to_string()),
                status: "selected".to_string(),
            },
            MemoryRetrievalHitDraft {
                hit_id: format!("{trace_id}-hit-2"),
                memory_id: None,
                retriever_name: "native_sql".to_string(),
                result_rank: 2,
                raw_score: Some(0.5),
                fused_score: Some(0.6),
                explanation_json: None,
                status: "candidate".to_string(),
            },
        ],
        context_pack: Some(MemoryContextPackSnapshot {
            context_pack_id: format!("{trace_id}-pack"),
            pack_json: r#"{"memoryIds":["rec-trace-1"]}"#.to_string(),
            estimated_tokens: 12,
            truncated: false,
        }),
    }
}

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
async fn sqlite_store_soft_deletes_records_and_suppresses_retrieve() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .create_record(&scope, "rec-delete", "preference", "delete me")
        .await
        .unwrap();

    let receipt = store
        .mark_record_deleted(&scope, "rec-delete")
        .await
        .unwrap();
    let retrieved = store.retrieve_record(&scope, "rec-delete").await.unwrap();
    let lifecycle = store
        .retrieve_record_lifecycle(&scope, "rec-delete")
        .await
        .unwrap()
        .unwrap();

    assert!(receipt.deleted);
    assert!(!receipt.already_deleted);
    assert!(retrieved.is_none());
    assert_eq!(lifecycle.memory_id, "rec-delete");
    assert_eq!(lifecycle.status, "deleted");
    assert_utc_timestamp(lifecycle.deleted_at.as_deref());
}

#[tokio::test]
async fn sqlite_store_record_delete_is_idempotent_for_already_deleted_records() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .create_record(&scope, "rec-delete-repeat", "preference", "delete me")
        .await
        .unwrap();

    let first = store
        .mark_record_deleted(&scope, "rec-delete-repeat")
        .await
        .unwrap();
    let second = store
        .mark_record_deleted(&scope, "rec-delete-repeat")
        .await
        .unwrap();

    assert!(first.deleted);
    assert!(!first.already_deleted);
    assert!(second.deleted);
    assert!(second.already_deleted);
}

#[tokio::test]
async fn sqlite_store_record_delete_does_not_cross_tenant_or_space_scope() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);
    let wrong_space = MemoryScopeContext::for_test(1, 2);

    store
        .create_record(&tenant_one, "rec-delete-scoped", "preference", "tenant one")
        .await
        .unwrap();
    store
        .create_record(&tenant_two, "rec-delete-scoped", "preference", "tenant two")
        .await
        .unwrap();

    let missing = store
        .mark_record_deleted(&wrong_space, "rec-delete-scoped")
        .await
        .unwrap();
    let deleted = store
        .mark_record_deleted(&tenant_one, "rec-delete-scoped")
        .await
        .unwrap();
    let tenant_two_record = store
        .retrieve_record(&tenant_two, "rec-delete-scoped")
        .await
        .unwrap()
        .unwrap();

    assert!(!missing.deleted);
    assert!(deleted.deleted);
    assert!(store
        .retrieve_record(&tenant_one, "rec-delete-scoped")
        .await
        .unwrap()
        .is_none());
    assert_eq!(tenant_two_record.content, "tenant two");
}

#[tokio::test]
async fn sqlite_store_implements_record_delete_spi_port() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    MemoryRecordStorePort::create(
        &store,
        CreateMemoryRecordCommand {
            scope: scope.clone(),
            memory_id: "rec-spi-delete".to_string(),
            content: "SPI delete payload".to_string(),
        },
    )
    .await
    .unwrap();

    let receipt = MemoryRecordStorePort::mark_deleted(
        &store,
        DeleteMemoryRecordCommand {
            scope: scope.clone(),
            memory_id: "rec-spi-delete".to_string(),
        },
    )
    .await
    .unwrap();
    let retrieved = MemoryRecordStorePort::retrieve(
        &store,
        RetrieveMemoryRecordQuery {
            scope,
            memory_id: "rec-spi-delete".to_string(),
        },
    )
    .await
    .unwrap();

    assert_eq!(receipt.memory_id, "rec-spi-delete");
    assert!(receipt.deleted);
    assert!(retrieved.is_none());
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

#[tokio::test]
async fn sqlite_store_appends_and_retrieves_outbox_events_by_tenant_scope() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);

    store
        .append_outbox_event(outbox_command(
            &tenant_one,
            "out-shared",
            "rec-1",
            r#"{"memoryId":"rec-1"}"#,
        ))
        .await
        .unwrap();
    store
        .append_outbox_event(outbox_command(
            &tenant_two,
            "out-shared",
            "rec-2",
            r#"{"memoryId":"rec-2"}"#,
        ))
        .await
        .unwrap();

    let tenant_one_outbox = store
        .retrieve_outbox_event(&tenant_one, "out-shared")
        .await
        .unwrap()
        .unwrap();
    let tenant_two_outbox = store
        .retrieve_outbox_event(&tenant_two, "out-shared")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tenant_one_outbox.aggregate_id, "rec-1");
    assert_eq!(tenant_one_outbox.publish_state, "pending");
    assert_eq!(tenant_one_outbox.retry_count, 0);
    assert_eq!(tenant_two_outbox.aggregate_id, "rec-2");
    assert!(store
        .retrieve_outbox_event(&MemoryScopeContext::for_test(3, 3), "out-shared")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn sqlite_store_outbox_append_is_idempotent_for_same_tenant_event_and_payload() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .append_outbox_event(outbox_command(
            &scope,
            "out-idempotent",
            "rec-1",
            r#"{"memoryId":"rec-1"}"#,
        ))
        .await
        .unwrap();
    store
        .append_outbox_event(outbox_command(
            &scope,
            "out-idempotent",
            "rec-1",
            r#"{"memoryId":"rec-1"}"#,
        ))
        .await
        .unwrap();

    let outbox = store
        .retrieve_outbox_event(&scope, "out-idempotent")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(outbox.aggregate_id, "rec-1");
}

#[tokio::test]
async fn sqlite_store_outbox_append_rejects_same_tenant_event_with_different_payload() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .append_outbox_event(outbox_command(
            &scope,
            "out-conflict",
            "rec-1",
            r#"{"memoryId":"rec-1"}"#,
        ))
        .await
        .unwrap();
    let err = store
        .append_outbox_event(outbox_command(
            &scope,
            "out-conflict",
            "rec-1",
            r#"{"memoryId":"rec-other"}"#,
        ))
        .await
        .unwrap_err();

    assert!(matches!(err, NativeSqlStoreError::OutboxConflict { .. }));
}

#[tokio::test]
async fn sqlite_store_implements_outbox_store_spi_port() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    let outbox = MemoryOutboxStorePort::append(
        &store,
        AppendMemoryOutboxCommand {
            scope: scope.clone(),
            outbox_id: "out-spi".to_string(),
            aggregate_type: "mem_event".to_string(),
            aggregate_id: "evt-spi".to_string(),
            event_type: "memory.event.appended".to_string(),
            event_version: "1".to_string(),
            payload_json: r#"{"eventId":"evt-spi"}"#.to_string(),
        },
    )
    .await
    .unwrap();
    let retrieved = MemoryOutboxStorePort::retrieve(
        &store,
        RetrieveMemoryOutboxQuery {
            scope,
            outbox_id: "out-spi".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(outbox.outbox_id, "out-spi");
    assert_eq!(retrieved.aggregate_type, "mem_event");
    assert_eq!(retrieved.aggregate_id, "evt-spi");
    assert_eq!(retrieved.event_type, "memory.event.appended");
    assert_eq!(retrieved.event_version, "1");
    assert_eq!(retrieved.payload_json, r#"{"eventId":"evt-spi"}"#);
    assert_eq!(retrieved.publish_state, "pending");
}

#[tokio::test]
async fn sqlite_store_spi_outbox_append_maps_idempotency_conflict_to_spi_conflict() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    MemoryOutboxStorePort::append(
        &store,
        AppendMemoryOutboxCommand {
            scope: scope.clone(),
            outbox_id: "out-spi-conflict".to_string(),
            aggregate_type: "mem_record".to_string(),
            aggregate_id: "rec-1".to_string(),
            event_type: "memory.record.created".to_string(),
            event_version: "1".to_string(),
            payload_json: r#"{"memoryId":"rec-1"}"#.to_string(),
        },
    )
    .await
    .unwrap();
    let err = MemoryOutboxStorePort::append(
        &store,
        AppendMemoryOutboxCommand {
            scope,
            outbox_id: "out-spi-conflict".to_string(),
            aggregate_type: "mem_record".to_string(),
            aggregate_id: "rec-1".to_string(),
            event_type: "memory.record.created".to_string(),
            event_version: "1".to_string(),
            payload_json: r#"{"memoryId":"rec-other"}"#.to_string(),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(err, MemorySpiError::IdempotencyConflict { .. }));
}

#[tokio::test]
async fn sqlite_store_lists_pending_outbox_events_by_tenant_scope_and_limit() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);

    store
        .append_outbox_event(outbox_command(
            &tenant_one,
            "out-pending-1",
            "rec-1",
            r#"{"memoryId":"rec-1"}"#,
        ))
        .await
        .unwrap();
    store
        .append_outbox_event(outbox_command(
            &tenant_one,
            "out-pending-2",
            "rec-2",
            r#"{"memoryId":"rec-2"}"#,
        ))
        .await
        .unwrap();
    store
        .append_outbox_event(outbox_command(
            &tenant_two,
            "out-pending-tenant-two",
            "rec-3",
            r#"{"memoryId":"rec-3"}"#,
        ))
        .await
        .unwrap();

    let pending = store
        .list_pending_outbox_events(&tenant_one, 1)
        .await
        .unwrap();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].outbox_id, "out-pending-1");
    assert_eq!(pending[0].publish_state, "pending");
}

#[tokio::test]
async fn sqlite_store_marks_outbox_published_and_excludes_it_from_pending() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .append_outbox_event(outbox_command(
            &scope,
            "out-publish",
            "rec-1",
            r#"{"memoryId":"rec-1"}"#,
        ))
        .await
        .unwrap();

    let published = store
        .mark_outbox_published(&scope, "out-publish")
        .await
        .unwrap()
        .unwrap();
    let retrieved = store
        .retrieve_outbox_event(&scope, "out-publish")
        .await
        .unwrap()
        .unwrap();
    let pending = store.list_pending_outbox_events(&scope, 10).await.unwrap();

    assert_eq!(published.publish_state, "published");
    assert_utc_timestamp(published.published_at.as_deref());
    assert_eq!(retrieved.publish_state, "published");
    assert!(pending.is_empty());
}

#[tokio::test]
async fn sqlite_store_marks_outbox_failed_increments_retry_and_excludes_it_from_pending() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    store
        .append_outbox_event(outbox_command(
            &scope,
            "out-fail",
            "rec-1",
            r#"{"memoryId":"rec-1"}"#,
        ))
        .await
        .unwrap();

    let failed = store
        .mark_outbox_failed(&scope, "out-fail")
        .await
        .unwrap()
        .unwrap();
    let pending = store.list_pending_outbox_events(&scope, 10).await.unwrap();

    assert_eq!(failed.publish_state, "failed");
    assert_eq!(failed.retry_count, 1);
    assert!(failed.published_at.is_none());
    assert!(pending.is_empty());
}

#[tokio::test]
async fn sqlite_store_outbox_delivery_lifecycle_does_not_cross_tenant_scope() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);
    let missing_tenant = MemoryScopeContext::for_test(3, 3);

    store
        .append_outbox_event(outbox_command(
            &tenant_one,
            "out-scoped",
            "rec-1",
            r#"{"memoryId":"rec-1"}"#,
        ))
        .await
        .unwrap();
    store
        .append_outbox_event(outbox_command(
            &tenant_two,
            "out-scoped",
            "rec-2",
            r#"{"memoryId":"rec-2"}"#,
        ))
        .await
        .unwrap();

    let missing = store
        .mark_outbox_published(&missing_tenant, "out-scoped")
        .await
        .unwrap();
    let tenant_one_published = store
        .mark_outbox_published(&tenant_one, "out-scoped")
        .await
        .unwrap()
        .unwrap();
    let tenant_two_pending = store
        .list_pending_outbox_events(&tenant_two, 10)
        .await
        .unwrap();

    assert!(missing.is_none());
    assert_eq!(tenant_one_published.publish_state, "published");
    assert_eq!(tenant_two_pending.len(), 1);
    assert_eq!(tenant_two_pending[0].aggregate_id, "rec-2");
}

#[tokio::test]
async fn sqlite_store_implements_outbox_delivery_lifecycle_spi_port() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);

    MemoryOutboxStorePort::append(
        &store,
        AppendMemoryOutboxCommand {
            scope: scope.clone(),
            outbox_id: "out-spi-lifecycle".to_string(),
            aggregate_type: "mem_record".to_string(),
            aggregate_id: "rec-1".to_string(),
            event_type: "memory.record.created".to_string(),
            event_version: "1".to_string(),
            payload_json: r#"{"memoryId":"rec-1"}"#.to_string(),
        },
    )
    .await
    .unwrap();

    let pending = MemoryOutboxStorePort::list_pending(
        &store,
        ListPendingMemoryOutboxQuery {
            scope: scope.clone(),
            limit: 10,
        },
    )
    .await
    .unwrap();
    let published = MemoryOutboxStorePort::mark_published(
        &store,
        MarkMemoryOutboxPublishedCommand {
            scope: scope.clone(),
            outbox_id: "out-spi-lifecycle".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let pending_after_publish = MemoryOutboxStorePort::list_pending(
        &store,
        ListPendingMemoryOutboxQuery {
            scope: scope.clone(),
            limit: 10,
        },
    )
    .await
    .unwrap();

    MemoryOutboxStorePort::append(
        &store,
        AppendMemoryOutboxCommand {
            scope: scope.clone(),
            outbox_id: "out-spi-failed".to_string(),
            aggregate_type: "mem_event".to_string(),
            aggregate_id: "evt-1".to_string(),
            event_type: "memory.event.appended".to_string(),
            event_version: "1".to_string(),
            payload_json: r#"{"eventId":"evt-1"}"#.to_string(),
        },
    )
    .await
    .unwrap();
    let failed = MemoryOutboxStorePort::mark_failed(
        &store,
        MarkMemoryOutboxFailedCommand {
            scope,
            outbox_id: "out-spi-failed".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].outbox_id, "out-spi-lifecycle");
    assert_eq!(published.publish_state, "published");
    assert_utc_timestamp(published.published_at.as_deref());
    assert!(pending_after_publish.is_empty());
    assert_eq!(failed.publish_state, "failed");
    assert_eq!(failed.retry_count, 1);
}

#[tokio::test]
async fn sqlite_store_creates_and_decides_candidates_by_tenant_and_space_scope() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);
    let wrong_space = MemoryScopeContext::for_test(1, 2);

    let tenant_one_candidate = MemoryCandidateStorePort::create(
        &store,
        candidate_command(tenant_one.clone(), "cand-shared"),
    )
    .await
    .unwrap();
    let tenant_two_candidate = MemoryCandidateStorePort::create(
        &store,
        candidate_command(tenant_two.clone(), "cand-shared"),
    )
    .await
    .unwrap();

    let approved = MemoryCandidateStorePort::approve(
        &store,
        ApproveMemoryCandidateCommand {
            scope: tenant_one.clone(),
            candidate_id: "cand-shared".to_string(),
            decision_reason: Some("confirmed by user".to_string()),
            decided_by: Some(7),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let rejected = MemoryCandidateStorePort::reject(
        &store,
        RejectMemoryCandidateCommand {
            scope: tenant_two.clone(),
            candidate_id: "cand-shared".to_string(),
            decision_reason: Some("stale signal".to_string()),
            decided_by: Some(8),
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(tenant_one_candidate.decision_state, "pending");
    assert_eq!(tenant_two_candidate.decision_state, "pending");
    assert_eq!(approved.decision_state, "approved");
    assert_eq!(
        approved.decision_reason.as_deref(),
        Some("confirmed by user")
    );
    assert_eq!(approved.decided_by, Some(7));
    assert_utc_timestamp(approved.decided_at.as_deref());
    assert_eq!(rejected.decision_state, "rejected");
    assert_eq!(rejected.decision_reason.as_deref(), Some("stale signal"));
    assert!(MemoryCandidateStorePort::retrieve(
        &store,
        RetrieveMemoryCandidateQuery {
            scope: wrong_space,
            candidate_id: "cand-shared".to_string(),
        },
    )
    .await
    .unwrap()
    .is_none());
}

#[tokio::test]
async fn sqlite_store_upserts_promotes_and_decays_habits_by_tenant_space_and_user_scope() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);
    let wrong_user = 43;

    store
        .create_record(&tenant_one, "rec-promoted", "answer_style", "concise")
        .await
        .unwrap();
    let inserted =
        MemoryHabitStorePort::upsert(&store, habit_command(tenant_one.clone(), "habit-1", 42))
            .await
            .unwrap();
    let updated = MemoryHabitStorePort::upsert(
        &store,
        UpsertMemoryHabitCommand {
            strength: 0.7,
            support_count: 4,
            ..habit_command(tenant_one.clone(), "habit-1", 42)
        },
    )
    .await
    .unwrap();
    let tenant_two_habit =
        MemoryHabitStorePort::upsert(&store, habit_command(tenant_two.clone(), "habit-2", 42))
            .await
            .unwrap();
    let promoted = MemoryHabitStorePort::promote(
        &store,
        PromoteMemoryHabitCommand {
            scope: tenant_one.clone(),
            user_id: 42,
            habit_key: "answer_style:concise".to_string(),
            promoted_memory_id: Some("rec-promoted".to_string()),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let decayed = MemoryHabitStorePort::decay(
        &store,
        DecayMemoryHabitCommand {
            scope: tenant_one.clone(),
            user_id: 42,
            habit_key: "answer_style:concise".to_string(),
            strength_delta: 0.2,
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(inserted.strength, 0.4);
    assert_eq!(updated.strength, 0.7);
    assert_eq!(updated.support_count, 4);
    assert_eq!(tenant_two_habit.habit_id, "habit-2");
    assert_eq!(promoted.stage, "promoted");
    assert_eq!(promoted.promoted_memory_id.as_deref(), Some("rec-promoted"));
    assert_eq!(decayed.stage, "decayed");
    assert!((decayed.strength - 0.5).abs() < f64::EPSILON);
    assert!(MemoryHabitStorePort::retrieve(
        &store,
        RetrieveMemoryHabitQuery {
            scope: tenant_one,
            user_id: wrong_user,
            habit_key: "answer_style:concise".to_string(),
        },
    )
    .await
    .unwrap()
    .is_none());
}

#[tokio::test]
async fn sqlite_store_appends_retrieval_trace_with_hits_and_context_pack_by_scope() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);
    let wrong_space = MemoryScopeContext::for_test(1, 2);

    store
        .create_record(&tenant_one, "rec-trace-1", "answer_style", "concise")
        .await
        .unwrap();
    let appended = MemoryRetrievalTraceStorePort::append(
        &store,
        retrieval_trace_command(tenant_one.clone(), "trace-shared"),
    )
    .await
    .unwrap();
    MemoryRetrievalTraceStorePort::append(
        &store,
        AppendMemoryRetrievalTraceCommand {
            query_text: Some("tenant two query".to_string()),
            ..retrieval_trace_command(tenant_two.clone(), "trace-shared")
        },
    )
    .await
    .unwrap();

    let retrieved = MemoryRetrievalTraceStorePort::retrieve(
        &store,
        RetrieveMemoryRetrievalTraceQuery {
            scope: tenant_one.clone(),
            trace_id: "trace-shared".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let tenant_two_trace = MemoryRetrievalTraceStorePort::retrieve(
        &store,
        RetrieveMemoryRetrievalTraceQuery {
            scope: tenant_two,
            trace_id: "trace-shared".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let recent = MemoryRetrievalTraceStorePort::list_recent(
        &store,
        ListMemoryRetrievalTracesQuery {
            scope: tenant_one.clone(),
            limit: 1,
        },
    )
    .await
    .unwrap();

    assert_eq!(appended.trace_id, "trace-shared");
    assert_eq!(retrieved.query_hash, "hash:trace-shared");
    assert_eq!(retrieved.result_count, 2);
    assert_eq!(retrieved.hits.len(), 2);
    assert_eq!(retrieved.hits[0].hit_id, "trace-shared-hit-1");
    assert_eq!(retrieved.hits[0].memory_id.as_deref(), Some("rec-trace-1"));
    assert_eq!(retrieved.hits[1].memory_id, None);
    assert_eq!(
        retrieved
            .context_pack
            .as_ref()
            .map(|pack| pack.context_pack_id.as_str()),
        Some("trace-shared-pack")
    );
    assert_eq!(
        tenant_two_trace.query_text.as_deref(),
        Some("tenant two query")
    );
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].trace_id, "trace-shared");
    assert!(MemoryRetrievalTraceStorePort::retrieve(
        &store,
        RetrieveMemoryRetrievalTraceQuery {
            scope: wrong_space,
            trace_id: "trace-shared".to_string(),
        },
    )
    .await
    .unwrap()
    .is_none());
}

#[test]
fn native_sql_manifest_exports_candidate_habit_and_retrieval_trace_builders() {
    let candidate = build_native_sql_candidate_store();
    let habit = build_native_sql_habit_store();
    let retrieval_trace = build_native_sql_retrieval_trace_store();

    assert_eq!(candidate.port_name, "MemoryCandidateStorePort");
    assert_eq!(candidate.builder_name, "build_native_sql_candidate_store");
    assert!(candidate.ready);
    assert_eq!(habit.port_name, "MemoryHabitStorePort");
    assert_eq!(habit.builder_name, "build_native_sql_habit_store");
    assert!(habit.ready);
    assert_eq!(retrieval_trace.port_name, "MemoryRetrievalTraceStorePort");
    assert_eq!(
        retrieval_trace.builder_name,
        "build_native_sql_retrieval_trace_store"
    );
    assert!(retrieval_trace.ready);
}

#[tokio::test]
async fn sqlite_store_lists_candidates_with_cursor_pagination() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);
    for candidate_id in ["cand-a", "cand-b", "cand-c"] {
        MemoryCandidateStorePort::create(
            &store,
            candidate_command(scope.clone(), candidate_id),
        )
        .await
        .unwrap();
    }

    let first_page = store
        .list_candidates_for_tenant(1, Some(1), 2, None)
        .await
        .unwrap();
    assert_eq!(first_page.len(), 3);
    let next_cursor = first_page[1].candidate_id.clone();

    let second_page = store
        .list_candidates_for_tenant(1, Some(1), 2, Some(next_cursor.as_str()))
        .await
        .unwrap();
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].candidate_id, "cand-c");
}

#[tokio::test]
async fn sqlite_store_lists_spaces_with_cursor_pagination() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    for space_id in [1_i64, 2, 3] {
        store
            .create_space_record(
                1,
                space_id,
                &NativeSqlCreateSpaceCommand {
                    organization_id: None,
                    owner_subject_type: "user".to_string(),
                    owner_subject_id: format!("user-{space_id}"),
                    space_type: "personal".to_string(),
                    display_name: format!("Space {space_id}"),
                    default_scope: "user".to_string(),
                },
            )
            .await
            .unwrap();
    }

    let first_page = store.list_spaces_for_tenant(1, 2, 0).await.unwrap();
    assert_eq!(first_page.len(), 3);
    let next_cursor = first_page[1].space_id;

    let second_page = store
        .list_spaces_for_tenant(1, 2, next_cursor)
        .await
        .unwrap();
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].space_id, 3);
}

#[tokio::test]
async fn sqlite_store_lists_record_sources_with_cursor_pagination() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    let scope = MemoryScopeContext::for_test(1, 1);
    store
        .create_space_record(
            1,
            1,
            &NativeSqlCreateSpaceCommand {
                organization_id: None,
                owner_subject_type: "user".to_string(),
                owner_subject_id: "user-1".to_string(),
                space_type: "personal".to_string(),
                display_name: "Source pagination space".to_string(),
                default_scope: "user".to_string(),
            },
        )
        .await
        .unwrap();
    store
        .create_record(&scope, "100", "user", "concise answers")
        .await
        .unwrap();
    for (source_id, event_id) in [("8101", "8001"), ("8102", "8002"), ("8103", "8003")] {
        store
            .append_open_api_event(
                &scope,
                event_id,
                "message.user",
                "chat",
                "2026-06-10T00:00:00Z",
                &serde_json::json!({ "text": "seed" }),
            )
            .await
            .unwrap();
        store
            .append_record_source_for_tenant(1, source_id, "100", event_id, "evidence", Some(0.1))
            .await
            .unwrap();
    }

    let first_page = store
        .list_record_sources_for_memory(1, "100", 2, None, None)
        .await
        .unwrap();
    assert_eq!(first_page.len(), 3);
    let next_cursor = first_page[1].source_uuid.clone();

    let second_page = store
        .list_record_sources_for_memory(1, "100", 2, Some(next_cursor.as_str()), None)
        .await
        .unwrap();
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].source_uuid, "8101");
}
