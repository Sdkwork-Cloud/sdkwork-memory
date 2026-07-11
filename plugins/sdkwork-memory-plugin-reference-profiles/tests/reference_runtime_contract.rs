use sdkwork_memory_plugin_reference_profiles::ReferenceMemoryRuntime;
use sdkwork_memory_spi::{
    AppendMemoryAuditCommand, AppendMemoryEventCommand, AppendMemoryOutboxCommand,
    AppendMemoryRetrievalTraceCommand, ApproveMemoryCandidateCommand, AssembleMemoryContextCommand,
    CreateMemoryCandidateCommand, CreateMemoryRecordCommand, DecayMemoryHabitCommand,
    ExternalMemoryBridgePort, ExternalMemoryImportCommand, ListMemoryRetrievalTracesQuery,
    ListPendingMemoryOutboxQuery, MarkMemoryOutboxPublishedCommand, MemoryAuditStorePort,
    MemoryCandidateStorePort, MemoryContextAssemblerPort, MemoryContextPackSnapshot,
    MemoryEvaluationPort, MemoryEventStorePort, MemoryHabitStorePort, MemoryIndexPort,
    MemoryOutboxStorePort, MemoryRecordStorePort, MemoryRetrievalHitDraft,
    MemoryRetrievalTraceStorePort, MemoryRetrieverPort, MemoryScopeContext,
    PromoteMemoryHabitCommand, RejectMemoryCandidateCommand, RetrieveMemoryAuditQuery,
    RetrieveMemoryCandidateQuery, RetrieveMemoryCandidatesCommand, RetrieveMemoryEventQuery,
    RetrieveMemoryHabitQuery, RetrieveMemoryRecordQuery, RetrieveMemoryRetrievalTraceQuery,
    RunMemoryEvalCommand, UpsertMemoryHabitCommand,
};

#[tokio::test]
async fn reference_runtime_round_trips_core_ports_and_retrieves_by_keyword() {
    let runtime = ReferenceMemoryRuntime::new();
    let scope = MemoryScopeContext::for_test(1, 1);

    MemoryRecordStorePort::create(
        &runtime,
        CreateMemoryRecordCommand {
            scope: scope.clone(),
            memory_id: "rec-reference".to_string(),
            content: "reference memory supports keyword lookup".to_string(),
        },
    )
    .await
    .unwrap();
    MemoryEventStorePort::append(
        &runtime,
        AppendMemoryEventCommand {
            scope: scope.clone(),
            event_id: "evt-reference".to_string(),
            content: "event sourced baseline".to_string(),
        },
    )
    .await
    .unwrap();
    MemoryAuditStorePort::append(
        &runtime,
        AppendMemoryAuditCommand {
            scope: scope.clone(),
            audit_id: "aud-reference".to_string(),
            action: "memory.reference.checked".to_string(),
            resource_type: "ai_record".to_string(),
            resource_id: "rec-reference".to_string(),
            result: "success".to_string(),
        },
    )
    .await
    .unwrap();
    MemoryOutboxStorePort::append(
        &runtime,
        AppendMemoryOutboxCommand {
            scope: scope.clone(),
            outbox_id: "out-reference".to_string(),
            aggregate_type: "ai_record".to_string(),
            aggregate_id: "rec-reference".to_string(),
            event_type: "memory.record.created".to_string(),
            event_version: "1".to_string(),
            payload_json: r#"{"memoryId":"rec-reference"}"#.to_string(),
        },
    )
    .await
    .unwrap();

    let record = MemoryRecordStorePort::retrieve(
        &runtime,
        RetrieveMemoryRecordQuery {
            scope: scope.clone(),
            memory_id: "rec-reference".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let event = MemoryEventStorePort::retrieve(
        &runtime,
        RetrieveMemoryEventQuery {
            scope: scope.clone(),
            event_id: "evt-reference".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let audit = MemoryAuditStorePort::retrieve(
        &runtime,
        RetrieveMemoryAuditQuery {
            scope: scope.clone(),
            audit_id: "aud-reference".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let hits = MemoryRetrieverPort::retrieve_scoped(
        &runtime,
        scope.clone(),
        RetrieveMemoryCandidatesCommand {
            query: "keyword".to_string(),
        },
    )
    .await
    .unwrap();
    let receipt = MemoryIndexPort::index(&runtime, "rec-reference".to_string())
        .await
        .unwrap();

    assert_eq!(record.content, "reference memory supports keyword lookup");
    assert_eq!(event.content, "event sourced baseline");
    assert_eq!(audit.result, "success");
    assert_eq!(hits.memory_ids, vec!["rec-reference".to_string()]);
    assert_eq!(receipt.memory_id, "rec-reference");
}

#[tokio::test]
async fn reference_runtime_outbox_context_eval_and_bridge_fail_closed_are_deterministic() {
    let runtime = ReferenceMemoryRuntime::new();
    let scope = MemoryScopeContext::for_test(1, 1);

    MemoryRecordStorePort::create(
        &runtime,
        CreateMemoryRecordCommand {
            scope: scope.clone(),
            memory_id: "rec-context".to_string(),
            content: "context line".to_string(),
        },
    )
    .await
    .unwrap();
    MemoryOutboxStorePort::append(
        &runtime,
        AppendMemoryOutboxCommand {
            scope: scope.clone(),
            outbox_id: "out-context".to_string(),
            aggregate_type: "ai_record".to_string(),
            aggregate_id: "rec-context".to_string(),
            event_type: "memory.record.created".to_string(),
            event_version: "1".to_string(),
            payload_json: r#"{"memoryId":"rec-context"}"#.to_string(),
        },
    )
    .await
    .unwrap();

    let pending = MemoryOutboxStorePort::list_pending(
        &runtime,
        ListPendingMemoryOutboxQuery {
            scope: scope.clone(),
            limit: 10,
        },
    )
    .await
    .unwrap();
    let published = MemoryOutboxStorePort::mark_published(
        &runtime,
        MarkMemoryOutboxPublishedCommand {
            scope: scope.clone(),
            outbox_id: "out-context".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let context = MemoryContextAssemblerPort::assemble_scoped(
        &runtime,
        scope.clone(),
        AssembleMemoryContextCommand {
            memory_ids: vec!["rec-context".to_string()],
        },
    )
    .await
    .unwrap();
    let eval = MemoryEvaluationPort::run(
        &runtime,
        RunMemoryEvalCommand {
            eval_type: "baseline".to_string(),
        },
    )
    .await
    .unwrap();
    let bridge_error = ExternalMemoryBridgePort::import(&runtime, ExternalMemoryImportCommand)
        .await
        .unwrap_err();

    assert_eq!(pending.len(), 1);
    assert_eq!(published.publish_state, "published");
    assert!(published
        .published_at
        .as_deref()
        .is_some_and(|value| value.ends_with('Z')));
    assert_eq!(context.context_text, "context line");
    assert_eq!(eval.eval_type, "baseline");
    assert!(bridge_error
        .to_string()
        .contains("fail-closed until a reviewed provider adapter is configured"));
}

#[tokio::test]
async fn reference_runtime_round_trips_learning_and_trace_ports_by_scope() {
    let runtime = ReferenceMemoryRuntime::new();
    let tenant_one = MemoryScopeContext::for_test(1, 1);
    let tenant_two = MemoryScopeContext::for_test(2, 2);
    let wrong_space = MemoryScopeContext::for_test(1, 2);

    MemoryRecordStorePort::create(
        &runtime,
        CreateMemoryRecordCommand {
            scope: tenant_one.clone(),
            memory_id: "rec-trace".to_string(),
            content: "traceable reference memory".to_string(),
        },
    )
    .await
    .unwrap();

    let created_candidate = MemoryCandidateStorePort::create(
        &runtime,
        CreateMemoryCandidateCommand {
            scope: tenant_one.clone(),
            candidate_id: "cand-reference".to_string(),
            candidate_type: "observation".to_string(),
            memory_type: "semantic".to_string(),
            proposed_text: "User prefers concise answers".to_string(),
            proposed_payload_json: Some(r#"{"preference":"concise"}"#.to_string()),
            evidence_json: Some(r#"{"source":"event"}"#.to_string()),
            confidence: 0.91,
        },
    )
    .await
    .unwrap();
    MemoryCandidateStorePort::create(
        &runtime,
        CreateMemoryCandidateCommand {
            scope: tenant_two.clone(),
            candidate_id: "cand-reference".to_string(),
            candidate_type: "observation".to_string(),
            memory_type: "semantic".to_string(),
            proposed_text: "Tenant two candidate".to_string(),
            proposed_payload_json: None,
            evidence_json: None,
            confidence: 0.51,
        },
    )
    .await
    .unwrap();
    let approved = MemoryCandidateStorePort::approve(
        &runtime,
        ApproveMemoryCandidateCommand {
            scope: tenant_one.clone(),
            candidate_id: "cand-reference".to_string(),
            decision_reason: Some("confirmed".to_string()),
            decided_by: Some(7),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let rejected = MemoryCandidateStorePort::reject(
        &runtime,
        RejectMemoryCandidateCommand {
            scope: tenant_two.clone(),
            candidate_id: "cand-reference".to_string(),
            decision_reason: Some("stale".to_string()),
            decided_by: Some(8),
        },
    )
    .await
    .unwrap()
    .unwrap();

    let inserted_habit = MemoryHabitStorePort::upsert(
        &runtime,
        UpsertMemoryHabitCommand {
            scope: tenant_one.clone(),
            habit_id: "habit-reference".to_string(),
            user_id: 42,
            habit_key: "answer_style:concise".to_string(),
            habit_type: "preference".to_string(),
            description: "Prefers concise answers".to_string(),
            stage: "candidate".to_string(),
            strength: 0.4,
            confidence: 0.8,
            support_count: 2,
            metadata_json: Some(r#"{"source":"signals"}"#.to_string()),
        },
    )
    .await
    .unwrap();
    let promoted = MemoryHabitStorePort::promote(
        &runtime,
        PromoteMemoryHabitCommand {
            scope: tenant_one.clone(),
            user_id: 42,
            habit_key: "answer_style:concise".to_string(),
            promoted_memory_id: Some("rec-trace".to_string()),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let decayed = MemoryHabitStorePort::decay(
        &runtime,
        DecayMemoryHabitCommand {
            scope: tenant_one.clone(),
            user_id: 42,
            habit_key: "answer_style:concise".to_string(),
            strength_delta: 0.1,
        },
    )
    .await
    .unwrap()
    .unwrap();

    let trace = MemoryRetrievalTraceStorePort::append(
        &runtime,
        AppendMemoryRetrievalTraceCommand {
            scope: tenant_one.clone(),
            trace_id: "trace-reference".to_string(),
            actor_id: Some("user-42".to_string()),
            query_text: Some("traceable".to_string()),
            query_hash: "hash:trace-reference".to_string(),
            retrievers_json: Some(r#"["reference_keyword"]"#.to_string()),
            latency_ms: Some(3),
            degraded: false,
            metadata_json: Some(r#"{"profile":"reference"}"#.to_string()),
            hits: vec![MemoryRetrievalHitDraft {
                hit_id: "hit-reference".to_string(),
                memory_id: Some("rec-trace".to_string()),
                space_id: Some(tenant_one.space_id),
                retriever_name: "reference_keyword".to_string(),
                result_rank: 1,
                raw_score: Some(0.9),
                fused_score: Some(0.95),
                explanation_json: Some(r#"{"match":"keyword"}"#.to_string()),
                status: "selected".to_string(),
            }],
            context_pack: Some(MemoryContextPackSnapshot {
                context_pack_id: "pack-reference".to_string(),
                pack_json: r#"{"memoryIds":["rec-trace"]}"#.to_string(),
                estimated_tokens: 9,
                truncated: false,
            }),
        },
    )
    .await
    .unwrap();
    let retrieved_trace = MemoryRetrievalTraceStorePort::retrieve(
        &runtime,
        RetrieveMemoryRetrievalTraceQuery {
            scope: tenant_one.clone(),
            trace_id: "trace-reference".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let recent = MemoryRetrievalTraceStorePort::list_recent(
        &runtime,
        ListMemoryRetrievalTracesQuery {
            scope: tenant_one.clone(),
            limit: 1,
        },
    )
    .await
    .unwrap();

    assert_eq!(created_candidate.decision_state, "pending");
    assert_eq!(approved.decision_state, "approved");
    assert_eq!(approved.decided_by, Some(7));
    assert_eq!(rejected.decision_state, "rejected");
    assert_eq!(inserted_habit.strength, 0.4);
    assert_eq!(promoted.promoted_memory_id.as_deref(), Some("rec-trace"));
    assert_eq!(decayed.stage, "decayed");
    assert!((decayed.strength - 0.3).abs() < f64::EPSILON);
    assert_eq!(trace.result_count, 1);
    assert_eq!(
        retrieved_trace.hits[0].memory_id.as_deref(),
        Some("rec-trace")
    );
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].trace_id, "trace-reference");
    assert!(MemoryCandidateStorePort::retrieve(
        &runtime,
        RetrieveMemoryCandidateQuery {
            scope: wrong_space.clone(),
            candidate_id: "cand-reference".to_string(),
        },
    )
    .await
    .unwrap()
    .is_none());
    assert!(MemoryHabitStorePort::retrieve(
        &runtime,
        RetrieveMemoryHabitQuery {
            scope: wrong_space.clone(),
            user_id: 42,
            habit_key: "answer_style:concise".to_string(),
        },
    )
    .await
    .unwrap()
    .is_none());
    assert!(MemoryRetrievalTraceStorePort::retrieve(
        &runtime,
        RetrieveMemoryRetrievalTraceQuery {
            scope: wrong_space,
            trace_id: "trace-reference".to_string(),
        },
    )
    .await
    .unwrap()
    .is_none());
}

#[tokio::test]
async fn reference_retrieval_and_context_assembly_are_isolated_by_tenant_and_space() {
    let runtime = ReferenceMemoryRuntime::new();
    let tenant_one = MemoryScopeContext::for_test(1, 10);
    let tenant_two = MemoryScopeContext::for_test(2, 10);
    let tenant_one_other_space = MemoryScopeContext::for_test(1, 20);

    for (scope, memory_id, content) in [
        (
            tenant_one.clone(),
            "rec-tenant-one",
            "shared isolation keyword tenant one",
        ),
        (
            tenant_two.clone(),
            "rec-tenant-two",
            "shared isolation keyword tenant two",
        ),
        (
            tenant_one_other_space.clone(),
            "rec-other-space",
            "shared isolation keyword other space",
        ),
    ] {
        MemoryRecordStorePort::create(
            &runtime,
            CreateMemoryRecordCommand {
                scope,
                memory_id: memory_id.to_string(),
                content: content.to_string(),
            },
        )
        .await
        .unwrap();
    }

    let tenant_one_hits = MemoryRetrieverPort::retrieve_scoped(
        &runtime,
        tenant_one.clone(),
        RetrieveMemoryCandidatesCommand {
            query: "isolation keyword".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(tenant_one_hits.memory_ids, vec!["rec-tenant-one"]);

    let tenant_two_hits = MemoryRetrieverPort::retrieve_scoped(
        &runtime,
        tenant_two.clone(),
        RetrieveMemoryCandidatesCommand {
            query: "isolation keyword".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(tenant_two_hits.memory_ids, vec!["rec-tenant-two"]);

    let other_space_hits = MemoryRetrieverPort::retrieve_scoped(
        &runtime,
        tenant_one_other_space.clone(),
        RetrieveMemoryCandidatesCommand {
            query: "isolation keyword".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(other_space_hits.memory_ids, vec!["rec-other-space"]);

    let tenant_one_context = MemoryContextAssemblerPort::assemble_scoped(
        &runtime,
        tenant_one,
        AssembleMemoryContextCommand {
            memory_ids: vec![
                "rec-tenant-one".to_string(),
                "rec-tenant-two".to_string(),
                "rec-other-space".to_string(),
            ],
        },
    )
    .await
    .unwrap();
    assert_eq!(
        tenant_one_context.context_text,
        "shared isolation keyword tenant one"
    );
    assert_eq!(tenant_one_context.memory_ids, vec!["rec-tenant-one"]);

    let tenant_two_context = MemoryContextAssemblerPort::assemble_scoped(
        &runtime,
        tenant_two,
        AssembleMemoryContextCommand {
            memory_ids: vec![
                "rec-tenant-one".to_string(),
                "rec-tenant-two".to_string(),
                "rec-other-space".to_string(),
            ],
        },
    )
    .await
    .unwrap();
    assert_eq!(
        tenant_two_context.context_text,
        "shared isolation keyword tenant two"
    );
    assert_eq!(tenant_two_context.memory_ids, vec!["rec-tenant-two"]);

    let unscoped_retrieval_error = MemoryRetrieverPort::retrieve(
        &runtime,
        RetrieveMemoryCandidatesCommand {
            query: "isolation keyword".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(unscoped_retrieval_error
        .to_string()
        .contains("use retrieve_scoped"));

    let unscoped_context_error = MemoryContextAssemblerPort::assemble(
        &runtime,
        AssembleMemoryContextCommand {
            memory_ids: vec!["rec-tenant-one".to_string()],
        },
    )
    .await
    .unwrap_err();
    assert!(unscoped_context_error
        .to_string()
        .contains("use assemble_scoped"));
}
