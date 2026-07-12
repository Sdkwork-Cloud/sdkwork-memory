//! Optional Postgres contract tests — set `SDKWORK_MEMORY_POSTGRES_TEST_URL` to run.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_memory_spi::{
    AppendMemoryEventCommand, AppendMemoryRetrievalTraceCommand, CreateMemoryRecordCommand,
    MemoryContextPackSnapshot, MemoryEventStorePort, MemoryRecordStorePort,
    MemoryRetrievalHitDraft, MemoryRetrievalTraceStorePort, MemoryScopeContext,
    RetrieveMemoryRetrievalTraceQuery,
};

async fn postgres_store() -> Option<NativeSqlMemoryStore> {
    let url = match std::env::var("SDKWORK_MEMORY_POSTGRES_TEST_URL") {
        Ok(url) if !url.trim().is_empty() => url,
        _ => {
            eprintln!(
                "skip postgres contract test: set SDKWORK_MEMORY_POSTGRES_TEST_URL to a writable database"
            );
            return None;
        }
    };

    let config = DatabaseConfig {
        engine: DatabaseEngine::Postgres,
        url,
        max_connections: 2,
        ..DatabaseConfig::default()
    };
    Some(
        NativeSqlMemoryStore::connect(&config)
            .await
            .expect("postgres connect and migration must succeed"),
    )
}

#[tokio::test]
async fn postgres_store_applies_phase1_migration_when_url_configured() {
    let Some(store) = postgres_store().await else {
        return;
    };
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

#[tokio::test]
async fn postgres_store_rebuilds_search_index_for_space_scope() {
    let Some(store) = postgres_store().await else {
        return;
    };

    let scope = MemoryScopeContext::for_test(100_001, 77);
    store
        .create_record_open_api(
            &scope,
            "pg-pred-mem",
            "user",
            "semantic",
            Some("Berlin"),
            Some("capital_of"),
            "Germany",
            "Germany capital",
            "internal",
        )
        .await
        .expect("create open api record on postgres");

    let rebuilt = store
        .rebuild_record_search_indexes_for_space(100_001, 77)
        .await
        .expect("scoped rebuild on postgres");
    assert!(rebuilt >= 1, "scoped rebuild must touch at least one row");

    let hits = store
        .search_record_details_fulltext(&scope, "capital_of", 5)
        .await
        .expect("fulltext search on postgres");
    assert!(
        hits.iter().any(|row| row.memory_id == "pg-pred-mem"),
        "postgres tsvector must index predicate"
    );
}

#[tokio::test]
async fn postgres_store_claims_eval_runs_atomically() {
    let Some(store) = postgres_store().await else {
        return;
    };

    let tenant_id = 100_001_i64;
    let eval_uuid = "pg-eval-claim-1";
    store
        .insert_mem_eval_run(tenant_id, eval_uuid, "retrieval_quality", "queued", None)
        .await
        .expect("seed eval run");

    let first = store.claim_queued_eval_runs(4).await.expect("first claim");
    assert!(
        first.iter().any(|(_, uuid, _)| uuid == eval_uuid),
        "first claim must include seeded eval run"
    );
    let second = store.claim_queued_eval_runs(4).await.expect("second claim");
    assert!(
        !second.iter().any(|(_, uuid, _)| uuid == eval_uuid),
        "second claim must not re-select the same eval run"
    );
}

#[tokio::test]
async fn postgres_store_persists_retrieval_trace_boolean_fields() {
    let Some(store) = postgres_store().await else {
        return;
    };

    let scope = MemoryScopeContext::for_test(100_001, 88);
    store
        .create_record(&scope, "pg-trace-rec", "answer_style", "concise")
        .await
        .expect("create record for retrieval trace");

    let appended = MemoryRetrievalTraceStorePort::append(
        &store,
        AppendMemoryRetrievalTraceCommand {
            scope: scope.clone(),
            trace_id: "pg-trace-bool".to_string(),
            actor_id: Some("user-42".to_string()),
            query_text: Some("postgres bool roundtrip".to_string()),
            query_hash: "hash:pg-trace-bool".to_string(),
            retrievers_json: Some(r#"["native_sql"]"#.to_string()),
            latency_ms: Some(21),
            degraded: true,
            metadata_json: Some(r#"{"profile":"native_sql"}"#.to_string()),
            hits: vec![MemoryRetrievalHitDraft {
                hit_id: "pg-trace-bool-hit-1".to_string(),
                memory_id: Some("pg-trace-rec".to_string()),
                space_id: Some(scope.space_id),
                retriever_name: "native_sql".to_string(),
                result_rank: 1,
                raw_score: Some(0.75),
                fused_score: Some(0.9),
                explanation_json: None,
                status: "selected".to_string(),
            }],
            context_pack: Some(MemoryContextPackSnapshot {
                context_pack_id: "pg-trace-bool-pack".to_string(),
                pack_json: r#"{"memoryIds":["pg-trace-rec"]}"#.to_string(),
                estimated_tokens: 8,
                truncated: true,
            }),
        },
    )
    .await
    .expect("append retrieval trace on postgres");

    let retrieved = MemoryRetrievalTraceStorePort::retrieve(
        &store,
        RetrieveMemoryRetrievalTraceQuery {
            scope,
            trace_id: appended.trace_id,
        },
    )
    .await
    .expect("retrieve retrieval trace on postgres")
    .expect("retrieval trace must exist");

    assert!(
        retrieved.degraded,
        "postgres degraded boolean must roundtrip as true"
    );
    assert!(
        retrieved
            .context_pack
            .as_ref()
            .map(|pack| pack.truncated)
            .unwrap_or(false),
        "postgres truncated boolean must roundtrip as true"
    );
}
