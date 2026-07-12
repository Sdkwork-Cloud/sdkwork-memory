use std::sync::Arc;

use sdkwork_memory_plugin_reference_profiles::ReferenceMemoryRuntime;
use sdkwork_memory_spi::{
    CountActiveMemoryRecordsQuery, CountUserOwnedMemorySpacesQuery, CreateMemoryRecordCommand,
    CreateMemorySpaceCommand, MemoryActorSpaceBindingFact, MemoryCapabilityBindingFact,
    MemoryGovernanceAccessPort, MemoryGovernanceActor, MemoryRecordStorePort, MemoryScopeContext,
    MemorySpaceGovernanceFact, MemorySpaceQuotaAdmission, MemorySpaceStorePort,
    ResolveMemorySpaceGovernanceQuery, MAX_MEMORY_GOVERNANCE_FACTS,
};
use tokio::sync::Barrier;

fn space(space_id: i64, owner_id: &str) -> MemorySpaceGovernanceFact {
    MemorySpaceGovernanceFact {
        space_id,
        organization_id: None,
        owner_subject_type: "user".to_string(),
        owner_subject_id: owner_id.to_string(),
        lifecycle_status: "active".to_string(),
    }
}

fn query(
    scope: MemoryScopeContext,
    actor: Option<MemoryGovernanceActor>,
    capability_code: Option<&str>,
) -> ResolveMemorySpaceGovernanceQuery {
    ResolveMemorySpaceGovernanceQuery {
        scope,
        actor,
        capability_code: capability_code.map(str::to_string),
        fact_limit: MAX_MEMORY_GOVERNANCE_FACTS,
    }
}

fn create_space_command(
    space_id: i64,
    owner_id: &str,
    space_type: &str,
) -> CreateMemorySpaceCommand {
    CreateMemorySpaceCommand {
        tenant_id: 1,
        space_id,
        organization_id: None,
        owner_subject_type: "user".to_string(),
        owner_subject_id: owner_id.to_string(),
        space_type: space_type.to_string(),
        display_name: format!("Space {space_id}"),
        default_scope: "user".to_string(),
    }
}

#[tokio::test]
async fn reference_governance_facts_are_scoped_and_bounded() {
    let runtime = ReferenceMemoryRuntime::new();
    let scope = MemoryScopeContext::for_test(1, 10);
    let actor = MemoryGovernanceActor {
        subject_type: Some("user".to_string()),
        subject_id: "actor-1".to_string(),
    };
    runtime
        .seed_governance_space(1, space(10, "owner-1"))
        .unwrap();
    runtime
        .seed_governance_space(2, space(10, "owner-2"))
        .unwrap();
    runtime
        .seed_actor_space_binding(
            &scope,
            &actor,
            MemoryActorSpaceBindingFact {
                binding_id: "binding-1".to_string(),
                binding_kind: "access".to_string(),
                binding_role: "viewer".to_string(),
                status: "active".to_string(),
                valid_from: None,
                valid_to: None,
            },
        )
        .unwrap();
    let facts = MemoryGovernanceAccessPort::resolve_space_governance(
        &runtime,
        query(scope.clone(), Some(actor), None),
    )
    .await
    .unwrap();
    assert!(facts.complete);
    assert_eq!(facts.space.unwrap().owner_subject_id, "owner-1");
    assert_eq!(facts.actor_bindings.len(), 1);

    let other_tenant = MemoryGovernanceAccessPort::resolve_space_governance(
        &runtime,
        query(MemoryScopeContext::for_test(2, 10), None, None),
    )
    .await
    .unwrap();
    assert_eq!(other_tenant.space.unwrap().owner_subject_id, "owner-2");
    assert!(other_tenant.actor_bindings.is_empty());

    for index in 0..=MAX_MEMORY_GOVERNANCE_FACTS {
        runtime
            .seed_capability_binding(
                &scope,
                MemoryCapabilityBindingFact {
                    binding_id: format!("cap-{index}"),
                    capability_code: "memory.retrieve".to_string(),
                    mode: "deny".to_string(),
                    priority: i32::try_from(index).unwrap(),
                    status: "active".to_string(),
                    valid_from: None,
                    valid_to: None,
                },
            )
            .unwrap();
    }
    let bounded = MemoryGovernanceAccessPort::resolve_space_governance(
        &runtime,
        query(scope, None, Some("memory.retrieve")),
    )
    .await
    .unwrap();
    assert!(!bounded.complete);
    assert_eq!(
        bounded.capability_bindings.len(),
        MAX_MEMORY_GOVERNANCE_FACTS as usize
    );
}

#[tokio::test]
async fn reference_governance_quota_counts_match_scoped_runtime_state() {
    let runtime = ReferenceMemoryRuntime::new();
    let scope = MemoryScopeContext::for_test(1, 10);
    runtime
        .seed_governance_space(1, space(10, "owner"))
        .unwrap();
    runtime
        .seed_governance_space(1, space(11, "owner"))
        .unwrap();
    runtime
        .seed_governance_space(2, space(10, "owner"))
        .unwrap();
    MemoryRecordStorePort::create(
        &runtime,
        CreateMemoryRecordCommand {
            scope: scope.clone(),
            memory_id: "record-1".to_string(),
            content: "one".to_string(),
        },
    )
    .await
    .unwrap();

    assert_eq!(
        MemoryGovernanceAccessPort::count_active_records(
            &runtime,
            CountActiveMemoryRecordsQuery { scope },
        )
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        MemoryGovernanceAccessPort::count_user_owned_spaces(
            &runtime,
            CountUserOwnedMemorySpacesQuery {
                tenant_id: 1,
                owner_subject_id: "owner".to_string(),
            },
        )
        .await
        .unwrap(),
        2
    );
}

#[tokio::test]
async fn reference_space_quota_admission_is_atomic_and_reuses_deleted_slots() {
    let runtime = Arc::new(ReferenceMemoryRuntime::new());
    assert!(runtime.supports_atomic_user_space_quota_admission());

    let barrier = Arc::new(Barrier::new(2));
    let first = {
        let runtime = runtime.clone();
        let barrier = barrier.clone();
        tokio::spawn(async move {
            barrier.wait().await;
            MemorySpaceStorePort::create_space_atomic_with_quota(
                runtime.as_ref(),
                create_space_command(20, "owner-race", "personal-a"),
                1,
            )
            .await
        })
    };
    let second = {
        let runtime = runtime.clone();
        let barrier = barrier.clone();
        tokio::spawn(async move {
            barrier.wait().await;
            MemorySpaceStorePort::create_space_atomic_with_quota(
                runtime.as_ref(),
                create_space_command(21, "owner-race", "personal-b"),
                1,
            )
            .await
        })
    };
    let (first, second) = tokio::join!(first, second);
    let outcomes = [first.unwrap().unwrap(), second.unwrap().unwrap()];
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, MemorySpaceQuotaAdmission::Admitted(_)))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, MemorySpaceQuotaAdmission::QuotaExceeded { .. }))
            .count(),
        1
    );
    assert_eq!(
        MemoryGovernanceAccessPort::count_user_owned_spaces(
            runtime.as_ref(),
            CountUserOwnedMemorySpacesQuery {
                tenant_id: 1,
                owner_subject_id: "owner-race".to_string(),
            },
        )
        .await
        .unwrap(),
        1
    );

    let mut deleted = space(30, "owner-deleted");
    deleted.lifecycle_status = "deleted".to_string();
    runtime.seed_governance_space(1, deleted).unwrap();
    let reused = MemorySpaceStorePort::create_space_atomic_with_quota(
        runtime.as_ref(),
        create_space_command(31, "owner-deleted", "replacement"),
        1,
    )
    .await
    .unwrap();
    assert!(matches!(reused, MemorySpaceQuotaAdmission::Admitted(_)));
}
