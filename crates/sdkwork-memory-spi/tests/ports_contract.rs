use sdkwork_memory_spi::{
    AppendMemoryRetrievalTraceCommand, ApproveMemoryCandidateCommand, CreateMemoryCandidateCommand,
    DecayMemoryHabitCommand, MemoryCandidate, MemoryCandidateStorePort, MemoryContextPackSnapshot,
    MemoryHabit, MemoryHabitStorePort, MemoryRetrievalHitDraft, MemoryRetrievalTrace,
    MemoryRetrievalTraceStorePort, MemoryScopeContext, PromoteMemoryHabitCommand,
    RejectMemoryCandidateCommand, RetrieveMemoryCandidateQuery, RetrieveMemoryHabitQuery,
    RetrieveMemoryRetrievalTraceQuery, UpsertMemoryHabitCommand,
};

#[test]
fn candidate_lifecycle_port_contract_types_are_public_and_scoped() {
    accept_candidate_port_object(None);

    let scope = MemoryScopeContext::for_test(1, 10);
    let create = CreateMemoryCandidateCommand {
        scope: scope.clone(),
        candidate_id: "cand-1".to_string(),
        candidate_type: "observation".to_string(),
        memory_type: "semantic".to_string(),
        proposed_text: "User prefers concise answers".to_string(),
        proposed_payload_json: Some(r#"{"preference":"concise"}"#.to_string()),
        evidence_json: Some(r#"{"source":"event"}"#.to_string()),
        confidence: 0.91,
    };
    let retrieve = RetrieveMemoryCandidateQuery {
        scope: scope.clone(),
        candidate_id: create.candidate_id.clone(),
    };
    let approve = ApproveMemoryCandidateCommand {
        scope: scope.clone(),
        candidate_id: create.candidate_id.clone(),
        decision_reason: Some("confirmed by user".to_string()),
        decided_by: Some(7),
    };
    let reject = RejectMemoryCandidateCommand {
        scope,
        candidate_id: create.candidate_id.clone(),
        decision_reason: Some("stale signal".to_string()),
        decided_by: Some(8),
    };
    let candidate = MemoryCandidate {
        candidate_id: create.candidate_id,
        candidate_type: create.candidate_type,
        memory_type: create.memory_type,
        proposed_text: create.proposed_text,
        proposed_payload_json: create.proposed_payload_json,
        evidence_json: create.evidence_json,
        confidence: create.confidence,
        decision_state: "pending".to_string(),
        decision_reason: None,
        decided_by: None,
        decided_at: None,
    };

    assert_eq!(retrieve.candidate_id, "cand-1");
    assert_eq!(approve.decided_by, Some(7));
    assert_eq!(reject.decided_by, Some(8));
    assert_eq!(candidate.decision_state, "pending");
}

#[test]
fn habit_learning_port_contract_types_are_public_and_user_scoped() {
    accept_habit_port_object(None);

    let scope = MemoryScopeContext::for_test(1, 10);
    let upsert = UpsertMemoryHabitCommand {
        scope: scope.clone(),
        habit_id: "habit-1".to_string(),
        user_id: 42,
        habit_key: "answer_style:concise".to_string(),
        habit_type: "preference".to_string(),
        description: "Prefers concise answers".to_string(),
        stage: "candidate".to_string(),
        strength: 0.4,
        confidence: 0.8,
        support_count: 2,
        metadata_json: Some(r#"{"source":"signals"}"#.to_string()),
    };
    let retrieve = RetrieveMemoryHabitQuery {
        scope: scope.clone(),
        user_id: upsert.user_id,
        habit_key: upsert.habit_key.clone(),
    };
    let promote = PromoteMemoryHabitCommand {
        scope: scope.clone(),
        user_id: upsert.user_id,
        habit_key: upsert.habit_key.clone(),
        promoted_memory_id: Some("rec-1".to_string()),
    };
    let decay = DecayMemoryHabitCommand {
        scope,
        user_id: upsert.user_id,
        habit_key: upsert.habit_key.clone(),
        strength_delta: 0.1,
    };
    let habit = MemoryHabit {
        habit_id: upsert.habit_id,
        user_id: upsert.user_id,
        habit_key: upsert.habit_key,
        habit_type: upsert.habit_type,
        description: upsert.description,
        stage: upsert.stage,
        strength: upsert.strength,
        confidence: upsert.confidence,
        support_count: upsert.support_count,
        last_signal_at: Some("2026-06-10T00:00:00Z".to_string()),
        promoted_memory_id: None,
        decay_after: None,
        metadata_json: upsert.metadata_json,
    };

    assert_eq!(retrieve.user_id, 42);
    assert_eq!(promote.promoted_memory_id.as_deref(), Some("rec-1"));
    assert_eq!(decay.strength_delta, 0.1);
    assert_eq!(habit.habit_key, "answer_style:concise");
}

#[test]
fn retrieval_trace_port_contract_types_are_public_and_bounded() {
    accept_retrieval_trace_port_object(None);

    let scope = MemoryScopeContext::for_test(1, 10);
    let hit = MemoryRetrievalHitDraft {
        hit_id: "hit-1".to_string(),
        memory_id: Some("rec-1".to_string()),
        space_id: Some(scope.space_id),
        retriever_name: "native_sql".to_string(),
        result_rank: 1,
        raw_score: Some(0.75),
        fused_score: Some(0.9),
        explanation_json: Some(r#"{"match":"keyword"}"#.to_string()),
        status: "selected".to_string(),
    };
    let context_pack = MemoryContextPackSnapshot {
        context_pack_id: "pack-1".to_string(),
        pack_json: r#"{"memoryIds":["rec-1"]}"#.to_string(),
        estimated_tokens: 12,
        truncated: false,
    };
    let append = AppendMemoryRetrievalTraceCommand {
        scope: scope.clone(),
        trace_id: "trace-1".to_string(),
        actor_id: Some("user-42".to_string()),
        query_text: Some("concise answer preference".to_string()),
        query_hash: "hash:trace-1".to_string(),
        retrievers_json: Some(r#"["native_sql"]"#.to_string()),
        latency_ms: Some(17),
        degraded: false,
        metadata_json: Some(r#"{"profile":"native"}"#.to_string()),
        hits: vec![hit],
        context_pack: Some(context_pack),
    };
    let retrieve = RetrieveMemoryRetrievalTraceQuery {
        scope,
        trace_id: append.trace_id.clone(),
    };
    let trace = MemoryRetrievalTrace {
        trace_id: append.trace_id,
        actor_id: append.actor_id,
        query_text: append.query_text,
        query_hash: append.query_hash,
        retrievers_json: append.retrievers_json,
        latency_ms: append.latency_ms,
        result_count: 1,
        degraded: append.degraded,
        metadata_json: append.metadata_json,
        hits: append.hits,
        context_pack: append.context_pack,
    };

    assert_eq!(retrieve.trace_id, "trace-1");
    assert_eq!(trace.hits.len(), 1);
    assert_eq!(trace.result_count, 1);
    assert!(!trace.degraded);
}

fn accept_candidate_port_object(_port: Option<&dyn MemoryCandidateStorePort>) {}

fn accept_habit_port_object(_port: Option<&dyn MemoryHabitStorePort>) {}

fn accept_retrieval_trace_port_object(_port: Option<&dyn MemoryRetrievalTraceStorePort>) {}
