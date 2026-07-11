use std::sync::Arc;

use sdkwork_intelligence_memory_service::{
    MemoryRuntimeDataPlane, MemoryRuntimeDataPlaneError, PHASE1_HTTP_DATA_PLANE_PORTS,
};
use sdkwork_memory_plugin_reference_profiles::{
    build_reference_executable_runtime, reference_profiles_manifest, ReferenceMemoryRuntime,
    REFERENCE_PROFILES_PLUGIN_ID,
};
use sdkwork_memory_spi::{
    AppendMemoryOutboxCommand, AssembleMemoryContextCommand, CreateCanonicalMemoryCommand,
    CreateMemoryCandidateCommand, DeleteCanonicalMemoryCommand, ExternalMemoryImportCommand,
    MemoryCoreRuntime, MemoryDeploymentMode, MemoryImplementationKind, MemoryMutationJournal,
    MemoryRuntimeProfileMetadata, MemoryScopeContext, PromoteMemoryHabitCommand,
    RejectMemoryCandidateCommand, RetrieveCanonicalMemoryQuery, RetrieveMemoryCandidateQuery,
    RetrieveMemoryCandidatesCommand, RetrieveMemoryOutboxQuery, UpsertMemoryHabitCommand,
};

fn mutation_journal(memory_id: &str, scope_tag: &str, action: &str) -> MemoryMutationJournal {
    MemoryMutationJournal {
        outbox_id: format!("outbox-{scope_tag}-{action}"),
        aggregate_type: "memory_record".to_string(),
        aggregate_id: memory_id.to_string(),
        event_type: format!("memory.record.{action}"),
        event_version: "1.0".to_string(),
        payload_json: "{}".to_string(),
        audit_id: format!("audit-{scope_tag}-{action}"),
        audit_action: format!("memory.record.{action}"),
        audit_resource_type: "memory_record".to_string(),
        audit_resource_id: memory_id.to_string(),
        audit_result: "accepted".to_string(),
    }
}

fn eval_runtime() -> MemoryRuntimeDataPlane {
    let reference = Arc::new(ReferenceMemoryRuntime::new());
    let executable = build_reference_executable_runtime(reference);
    let manifest = reference_profiles_manifest();
    let metadata = MemoryRuntimeProfileMetadata {
        profile_id: "reference-eval-contract".to_string(),
        implementation_kind: MemoryImplementationKind::HybridPlatform,
        primary_plugin_id: REFERENCE_PROFILES_PLUGIN_ID.to_string(),
        deployment_mode: MemoryDeploymentMode::EvalOnly,
    };
    let mut core = MemoryCoreRuntime::new(metadata);
    for port in PHASE1_HTTP_DATA_PLANE_PORTS.iter().copied().chain([
        "MemoryIndexPort",
        "ExternalMemoryBridgePort",
        "MemoryContextAssemblerPort",
        "MemoryEvaluationPort",
    ]) {
        assert!(manifest.port_exports.iter().any(|export| export.port == port));
        core.bind_port(REFERENCE_PROFILES_PLUGIN_ID, port, &executable)
            .expect("reference runtime port must bind");
    }
    MemoryRuntimeDataPlane::try_for_phase1_http(core)
        .expect("reference contract runtime exposes all phase-1 HTTP ports")
}

#[test]
fn phase1_data_plane_rejects_missing_required_port() {
    let runtime = MemoryCoreRuntime::new(MemoryRuntimeProfileMetadata {
        profile_id: "incomplete-eval".to_string(),
        implementation_kind: MemoryImplementationKind::HybridPlatform,
        primary_plugin_id: REFERENCE_PROFILES_PLUGIN_ID.to_string(),
        deployment_mode: MemoryDeploymentMode::EvalOnly,
    });

    match MemoryRuntimeDataPlane::try_for_phase1_http(runtime) {
        Err(MemoryRuntimeDataPlaneError::MissingRequiredPort { profile_id, port }) => {
            assert_eq!(profile_id, "incomplete-eval");
            assert_eq!(port, "MemoryRecordStorePort");
        }
        other => panic!("expected missing required port, got {other:?}"),
    }
}

#[tokio::test]
async fn canonical_record_retrieval_context_and_delete_are_scope_aware() {
    let plane = eval_runtime();
    let tenant_one = MemoryScopeContext::for_test(100, 10);
    let tenant_two = MemoryScopeContext::for_test(200, 10);

    plane
        .create_canonical_memory_atomic(CreateCanonicalMemoryCommand {
            scope: tenant_one.clone(),
            memory_id: "memory-1".to_string(),
            scope_label: "user".to_string(),
            memory_type: "semantic".to_string(),
            subject: Some("tenant-one".to_string()),
            predicate: Some("prefers".to_string()),
            object_text: "tenant one preference".to_string(),
            canonical_text: "tenant one preference".to_string(),
            sensitivity_level: "internal".to_string(),
            journal: mutation_journal("memory-1", "tenant-one", "created"),
        })
        .await
        .unwrap();
    plane
        .create_canonical_memory_atomic(CreateCanonicalMemoryCommand {
            scope: tenant_two.clone(),
            memory_id: "memory-1".to_string(),
            scope_label: "user".to_string(),
            memory_type: "semantic".to_string(),
            subject: Some("tenant-two".to_string()),
            predicate: Some("prefers".to_string()),
            object_text: "tenant two preference".to_string(),
            canonical_text: "tenant two preference".to_string(),
            sensitivity_level: "internal".to_string(),
            journal: mutation_journal("memory-1", "tenant-two", "created"),
        })
        .await
        .unwrap();

    assert_eq!(
        plane
            .retrieve_canonical_memory(RetrieveCanonicalMemoryQuery {
                scope: tenant_one.clone(),
                memory_id: "memory-1".to_string(),
            })
            .await
            .unwrap()
            .unwrap()
            .canonical_text,
        "tenant one preference"
    );
    assert_eq!(
        plane
            .retrieve_candidates_scoped(
                tenant_one.clone(),
                RetrieveMemoryCandidatesCommand {
                    query: "preference".to_string(),
                },
            )
            .await
            .unwrap()
            .memory_ids,
        vec!["memory-1"]
    );

    let context = plane
        .assemble_context_scoped(
            tenant_one.clone(),
            AssembleMemoryContextCommand {
                memory_ids: vec!["memory-1".to_string()],
            },
        )
        .await
        .unwrap();
    assert_eq!(context.memory_ids, vec!["memory-1"]);
    assert_eq!(context.context_text, "tenant one preference");

    plane
        .delete_canonical_memory_atomic(DeleteCanonicalMemoryCommand {
            scope: tenant_one.clone(),
            memory_id: "memory-1".to_string(),
            journal: mutation_journal("memory-1", "tenant-one", "deleted"),
        })
        .await
        .unwrap();
    assert!(plane
        .retrieve_canonical_memory(RetrieveCanonicalMemoryQuery {
            scope: tenant_one.clone(),
            memory_id: "memory-1".to_string(),
        })
        .await
        .unwrap()
        .is_none());
    assert!(plane
        .retrieve_candidates_scoped(
            tenant_one,
            RetrieveMemoryCandidatesCommand {
                query: "preference".to_string(),
            },
        )
        .await
        .unwrap()
        .memory_ids
        .is_empty());
    assert_eq!(
        plane
            .retrieve_canonical_memory(RetrieveCanonicalMemoryQuery {
                scope: tenant_two,
                memory_id: "memory-1".to_string(),
            })
            .await
            .unwrap()
            .unwrap()
            .canonical_text,
        "tenant two preference"
    );
}

#[tokio::test]
async fn outbox_candidates_and_habits_do_not_cross_scope_boundaries() {
    let plane = eval_runtime();
    let first = MemoryScopeContext::for_test(1, 11);
    let second = MemoryScopeContext::for_test(2, 22);

    for scope in [first.clone(), second.clone()] {
        plane
            .append_outbox(AppendMemoryOutboxCommand {
                scope,
                outbox_id: "outbox-1".to_string(),
                aggregate_type: "memory_record".to_string(),
                aggregate_id: "memory-1".to_string(),
                event_type: "memory.record.created".to_string(),
                event_version: "1.0".to_string(),
                payload_json: "{}".to_string(),
            })
            .await
            .unwrap();
    }
    assert!(plane
        .retrieve_outbox(RetrieveMemoryOutboxQuery {
            scope: first.clone(),
            outbox_id: "outbox-1".to_string(),
        })
        .await
        .unwrap()
        .is_some());
    assert!(plane
        .retrieve_outbox(RetrieveMemoryOutboxQuery {
            scope: second.clone(),
            outbox_id: "outbox-1".to_string(),
        })
        .await
        .unwrap()
        .is_some());

    plane
        .create_candidate(CreateMemoryCandidateCommand {
            scope: first.clone(),
            candidate_id: "candidate-1".to_string(),
            candidate_type: "extraction".to_string(),
            memory_type: "semantic".to_string(),
            proposed_text: "first candidate".to_string(),
            proposed_payload_json: None,
            evidence_json: None,
            confidence: 0.9,
        })
        .await
        .unwrap();
    plane
        .create_candidate(CreateMemoryCandidateCommand {
            scope: second.clone(),
            candidate_id: "candidate-1".to_string(),
            candidate_type: "extraction".to_string(),
            memory_type: "semantic".to_string(),
            proposed_text: "second candidate".to_string(),
            proposed_payload_json: None,
            evidence_json: None,
            confidence: 0.8,
        })
        .await
        .unwrap();
    plane
        .reject_candidate(RejectMemoryCandidateCommand {
            scope: first.clone(),
            candidate_id: "candidate-1".to_string(),
            decision_reason: Some("not stable".to_string()),
            decided_by: Some(1),
        })
        .await
        .unwrap();
    assert_eq!(
        plane
            .retrieve_candidate(RetrieveMemoryCandidateQuery {
                scope: first.clone(),
                candidate_id: "candidate-1".to_string(),
            })
            .await
            .unwrap()
            .unwrap()
            .decision_state,
        "rejected"
    );
    assert_eq!(
        plane
            .retrieve_candidate(RetrieveMemoryCandidateQuery {
                scope: second.clone(),
                candidate_id: "candidate-1".to_string(),
            })
            .await
            .unwrap()
            .unwrap()
            .decision_state,
        "pending"
    );

    for (scope, description) in [
        (first.clone(), "light"),
        (second.clone(), "dark"),
    ] {
        plane
            .upsert_habit(UpsertMemoryHabitCommand {
                scope,
                habit_id: "habit-1".to_string(),
                user_id: 9,
                habit_key: "editor.theme".to_string(),
                habit_type: "preference".to_string(),
                description: description.to_string(),
                stage: "candidate".to_string(),
                strength: 0.4,
                confidence: 0.8,
                support_count: 1,
                metadata_json: None,
            })
            .await
            .unwrap();
    }
    plane
        .promote_habit(PromoteMemoryHabitCommand {
            scope: first.clone(),
            user_id: 9,
            habit_key: "editor.theme".to_string(),
            promoted_memory_id: Some("memory-1".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(
        plane
            .retrieve_habit(sdkwork_memory_spi::RetrieveMemoryHabitQuery {
                scope: first,
                user_id: 9,
                habit_key: "editor.theme".to_string(),
            })
            .await
            .unwrap()
            .unwrap()
            .stage,
        "promoted"
    );
    assert_eq!(
        plane
            .retrieve_habit(sdkwork_memory_spi::RetrieveMemoryHabitQuery {
                scope: second,
                user_id: 9,
                habit_key: "editor.theme".to_string(),
            })
            .await
            .unwrap()
            .unwrap()
            .stage,
        "candidate"
    );
}

#[tokio::test]
async fn external_bridge_is_present_but_fail_closed_until_configured() {
    let plane = eval_runtime();
    let bridge = plane.external_memory_bridge().unwrap();
    let error = bridge
        .import(ExternalMemoryImportCommand)
        .await
        .expect_err("reference bridge must fail closed");
    assert!(error.to_string().contains("fail-closed"));
}
