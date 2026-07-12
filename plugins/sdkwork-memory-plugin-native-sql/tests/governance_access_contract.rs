use sdkwork_memory_plugin_native_sql::{
    InsertSubjectCommand, NativeSqlCreateSpaceCommand, NativeSqlMemoryStore,
};
use sdkwork_memory_spi::{
    CountActiveMemoryRecordsQuery, CountUserOwnedMemorySpacesQuery, MemoryGovernanceAccessPort,
    MemoryGovernanceActor, MemoryScopeContext, ResolveMemorySpaceGovernanceQuery,
    MAX_MEMORY_GOVERNANCE_FACTS,
};

async fn seed_space(store: &NativeSqlMemoryStore, tenant_id: i64, space_id: i64, owner_id: &str) {
    store
        .create_space_record(
            tenant_id,
            space_id,
            &NativeSqlCreateSpaceCommand {
                organization_id: None,
                owner_subject_type: "user".to_string(),
                owner_subject_id: owner_id.to_string(),
                space_type: format!("personal-{space_id}"),
                display_name: format!("space-{tenant_id}-{space_id}"),
                default_scope: "user".to_string(),
            },
        )
        .await
        .expect("space fixture must be created");
}

fn governance_query(
    tenant_id: i64,
    space_id: i64,
    actor: Option<MemoryGovernanceActor>,
    capability_code: Option<&str>,
) -> ResolveMemorySpaceGovernanceQuery {
    ResolveMemorySpaceGovernanceQuery {
        scope: MemoryScopeContext::for_test(tenant_id, space_id),
        actor,
        capability_code: capability_code.map(str::to_string),
        fact_limit: MAX_MEMORY_GOVERNANCE_FACTS,
    }
}

#[tokio::test]
async fn sqlite_governance_facts_are_tenant_scoped_and_target_exactly() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    seed_space(&store, 1, 10, "owner-one").await;
    seed_space(&store, 2, 20, "owner-two").await;
    store
        .insert_subject(InsertSubjectCommand {
            id: 101,
            uuid: "subject-user-101",
            tenant_id: 1,
            organization_id: None,
            subject_type: "user",
            subject_ref: "actor-1",
            display_name: "actor",
            default_space_id: None,
            metadata_json: None,
        })
        .await
        .unwrap();
    store
        .insert_binding(
            201,
            "binding-201",
            1,
            Some(10),
            "access",
            "viewer",
            Some(101),
            None,
            Some(10),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    let actor = Some(MemoryGovernanceActor {
        subject_type: Some("user".to_string()),
        subject_id: "actor-1".to_string(),
    });
    let tenant_one = MemoryGovernanceAccessPort::resolve_space_governance(
        &store,
        governance_query(1, 10, actor.clone(), None),
    )
    .await
    .unwrap();
    assert_eq!(
        tenant_one.space.as_ref().unwrap().owner_subject_id,
        "owner-one"
    );
    assert_eq!(tenant_one.actor_bindings.len(), 1);

    let tenant_two = MemoryGovernanceAccessPort::resolve_space_governance(
        &store,
        governance_query(2, 20, actor, None),
    )
    .await
    .unwrap();
    assert_eq!(
        tenant_two.space.as_ref().unwrap().owner_subject_id,
        "owner-two"
    );
    assert!(tenant_two.actor_bindings.is_empty());

    // A binding recorded in a contextual `space_id` without an exact target must not grant
    // whole-space access through a legacy fallback.
    store
        .insert_binding(
            202,
            "binding-202",
            1,
            Some(10),
            "access",
            "viewer",
            Some(101),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    let exact_only = MemoryGovernanceAccessPort::resolve_space_governance(
        &store,
        governance_query(
            1,
            10,
            Some(MemoryGovernanceActor {
                subject_type: Some("user".to_string()),
                subject_id: "actor-1".to_string(),
            }),
            None,
        ),
    )
    .await
    .unwrap();
    assert_eq!(exact_only.actor_bindings.len(), 1);
    assert_eq!(exact_only.actor_bindings[0].binding_id, "binding-201");
}

#[tokio::test]
async fn sqlite_governance_rejects_ambiguous_actor_namespace_and_bounds_facts() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    seed_space(&store, 1, 10, "owner").await;
    for (id, subject_type) in [(101, "user"), (102, "application")] {
        store
            .insert_subject(InsertSubjectCommand {
                id,
                uuid: &format!("subject-{id}"),
                tenant_id: 1,
                organization_id: None,
                subject_type,
                subject_ref: "same-ref",
                display_name: subject_type,
                default_space_id: None,
                metadata_json: None,
            })
            .await
            .unwrap();
    }
    let ambiguous = MemoryGovernanceAccessPort::resolve_space_governance(
        &store,
        governance_query(
            1,
            10,
            Some(MemoryGovernanceActor {
                subject_type: None,
                subject_id: "same-ref".to_string(),
            }),
            None,
        ),
    )
    .await
    .unwrap();
    assert!(!ambiguous.complete);

    for index in 0..=MAX_MEMORY_GOVERNANCE_FACTS {
        store
            .insert_capability_binding(
                1000 + i64::from(index),
                &format!("cap-{index}"),
                1,
                "memory.retrieve",
                "space",
                10,
                "deny",
                100 - i32::try_from(index).unwrap(),
                None,
                None,
                None,
            )
            .await
            .unwrap();
    }
    let bounded = MemoryGovernanceAccessPort::resolve_space_governance(
        &store,
        governance_query(1, 10, None, Some("memory.retrieve")),
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
async fn sqlite_governance_capability_facts_preserve_priority_and_validity_values() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    seed_space(&store, 1, 10, "owner").await;
    store
        .insert_capability_binding(
            1,
            "cap-future",
            1,
            "memory.retrieve",
            "space",
            10,
            "deny",
            20,
            Some("2999-01-01T00:00:00.000Z"),
            None,
            None,
        )
        .await
        .unwrap();
    store
        .insert_capability_binding(
            2,
            "cap-active",
            1,
            "memory.retrieve",
            "space",
            10,
            "deny",
            10,
            Some("2000-01-01T00:00:00.000Z"),
            Some("2999-01-01T00:00:00.000Z"),
            None,
        )
        .await
        .unwrap();
    let facts = MemoryGovernanceAccessPort::resolve_space_governance(
        &store,
        governance_query(1, 10, None, Some("memory.retrieve")),
    )
    .await
    .unwrap();
    assert_eq!(facts.capability_bindings[0].binding_id, "cap-future");
    assert_eq!(facts.capability_bindings[1].binding_id, "cap-active");
    assert_eq!(
        facts.capability_bindings[0].valid_from.as_deref(),
        Some("2999-01-01T00:00:00.000Z")
    );
}

#[tokio::test]
async fn sqlite_governance_quota_counts_are_scoped_and_exclude_deleted_records() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
    seed_space(&store, 1, 10, "owner").await;
    seed_space(&store, 1, 11, "owner").await;
    seed_space(&store, 1, 12, "other").await;
    seed_space(&store, 2, 20, "owner").await;
    let scope = MemoryScopeContext::for_test(1, 10);
    store
        .create_record(&scope, "record-1", "preference", "one")
        .await
        .unwrap();
    store
        .create_record(&scope, "record-2", "preference", "two")
        .await
        .unwrap();
    store.mark_record_deleted(&scope, "record-2").await.unwrap();
    let count = MemoryGovernanceAccessPort::count_active_records(
        &store,
        CountActiveMemoryRecordsQuery { scope },
    )
    .await
    .unwrap();
    assert_eq!(count, 1);

    let owned = MemoryGovernanceAccessPort::count_user_owned_spaces(
        &store,
        CountUserOwnedMemorySpacesQuery {
            tenant_id: 1,
            owner_subject_id: "owner".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(owned, 2);
}
