use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_contract::{
    MemoryFeedbackRequest, MemoryOpenApi, MemoryOpenApiRequestContext, MemoryRecordRequest,
    MemoryRetrievalRequest, MemoryServiceErrorKind, MemoryType,
};
use sdkwork_memory_plugin_native_sql::{
    InsertSubjectCommand, NativeSqlCreateSpaceCommand, NativeSqlMemoryStore,
};

const TENANT_ID: u64 = 91_001;
const ACTOR_ID: u64 = 42;

fn context() -> MemoryOpenApiRequestContext {
    MemoryOpenApiRequestContext::for_open_surface(
        "retrieval-contract-key",
        TENANT_ID,
        Some(ACTOR_ID),
    )
}

async fn service_with_spaces() -> OpenMemoryService {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite()
        .await
        .expect("retrieval contract sqlite store must open");
    for (space_id, space_type) in [(1_i64, "workspace"), (2_i64, "shared")] {
        store
            .create_space_record(
                TENANT_ID as i64,
                space_id,
                &NativeSqlCreateSpaceCommand {
                    organization_id: None,
                    owner_subject_type: "user".to_string(),
                    owner_subject_id: ACTOR_ID.to_string(),
                    space_type: space_type.to_string(),
                    display_name: format!("Retrieval Contract Space {space_id}"),
                    default_scope: "user".to_string(),
                },
            )
            .await
            .expect("retrieval contract space must be created");
    }
    OpenMemoryService::new(store)
}

fn memory_request(space_id: u64, memory_type: MemoryType, text: &str) -> MemoryRecordRequest {
    MemoryRecordRequest {
        space_id,
        scope: "user".to_string(),
        memory_type,
        subject: Some("retrieval".to_string()),
        predicate: Some("matches".to_string()),
        object_text: Some(text.to_string()),
        canonical_text: text.to_string(),
        summary_text: None,
        user_id: Some(ACTOR_ID),
        language: Some("en".to_string()),
        sensitivity_level: Some("internal".to_string()),
        metadata: None,
        tags: None,
    }
}

fn retrieval_request(
    query: &str,
    space_ids: Vec<u64>,
    top_k: i32,
    memory_types: Option<Vec<MemoryType>>,
) -> MemoryRetrievalRequest {
    MemoryRetrievalRequest {
        query: query.to_string(),
        space_ids,
        actor_id: Some(ACTOR_ID.to_string()),
        retrieval_profile_id: None,
        memory_types,
        filters: None,
        top_k,
        context_budget_tokens: 512,
        include_trace: Some(true),
    }
}

#[tokio::test]
async fn multi_space_retrieval_trace_round_trips_each_hit_scope() {
    let service = service_with_spaces().await;
    let context = context();
    for (space_id, text) in [
        (1, "shared multispace needle from space one"),
        (2, "shared multispace needle from space two"),
    ] {
        service
            .create_memory(
                context.clone(),
                memory_request(space_id, MemoryType::Semantic, text),
            )
            .await
            .unwrap();
    }

    let created = service
        .create_retrieval(
            context.clone(),
            retrieval_request("multispace needle", vec![1, 2], 5, None),
        )
        .await
        .unwrap();
    let mut created_spaces = created
        .hits
        .iter()
        .filter_map(|hit| hit.memory.as_ref().map(|memory| memory.space_id))
        .collect::<Vec<_>>();
    created_spaces.sort_unstable();
    assert_eq!(created_spaces, vec![1, 2]);

    let retrieved = service
        .retrieve_retrieval(context, created.retrieval_id)
        .await
        .unwrap();
    let mut retrieved_spaces = retrieved
        .hits
        .iter()
        .filter_map(|hit| hit.memory.as_ref().map(|memory| memory.space_id))
        .collect::<Vec<_>>();
    retrieved_spaces.sort_unstable();
    assert_eq!(retrieved_spaces, vec![1, 2]);
    assert!(retrieved.hits.iter().all(|hit| hit.memory.is_some()));
}

#[tokio::test]
async fn retrieval_filters_before_top_k_and_rejects_invalid_query_bounds() {
    let service = service_with_spaces().await;
    let context = context();
    service
        .create_memory(
            context.clone(),
            memory_request(
                1,
                MemoryType::Episodic,
                "typefilterneedle episodic should be excluded",
            ),
        )
        .await
        .unwrap();
    service
        .create_memory(
            context.clone(),
            memory_request(
                1,
                MemoryType::Semantic,
                "typefilterneedle semantic should be returned",
            ),
        )
        .await
        .unwrap();

    let result = service
        .create_retrieval(
            context.clone(),
            retrieval_request(
                "typefilterneedle",
                vec![1],
                1,
                Some(vec![MemoryType::Semantic]),
            ),
        )
        .await
        .unwrap();
    assert_eq!(result.hits.len(), 1);
    assert_eq!(
        result.hits[0].memory.as_ref().unwrap().memory_type,
        MemoryType::Semantic
    );

    for request in [
        retrieval_request("   ", vec![1], 1, None),
        retrieval_request("typefilterneedle", vec![1], 0, None),
        retrieval_request("typefilterneedle", vec![1], 101, None),
    ] {
        let error = service
            .create_retrieval(context.clone(), request)
            .await
            .expect_err("invalid retrieval input must fail before adapter search");
        assert_eq!(error.kind, MemoryServiceErrorKind::Validation);
    }
}

#[tokio::test]
async fn retrieval_trace_drops_hits_after_cross_space_access_is_revoked() {
    let store = NativeSqlMemoryStore::new_in_memory_sqlite()
        .await
        .expect("retrieval contract sqlite store must open");
    store
        .create_space_record(
            TENANT_ID as i64,
            1,
            &NativeSqlCreateSpaceCommand {
                organization_id: None,
                owner_subject_type: "user".to_string(),
                owner_subject_id: ACTOR_ID.to_string(),
                space_type: "workspace".to_string(),
                display_name: "Owned Space".to_string(),
                default_scope: "user".to_string(),
            },
        )
        .await
        .unwrap();
    store
        .create_space_record(
            TENANT_ID as i64,
            2,
            &NativeSqlCreateSpaceCommand {
                organization_id: None,
                owner_subject_type: "user".to_string(),
                owner_subject_id: "other-user".to_string(),
                space_type: "shared".to_string(),
                display_name: "Shared Space".to_string(),
                default_scope: "user".to_string(),
            },
        )
        .await
        .unwrap();
    let actor_ref = ACTOR_ID.to_string();
    store
        .insert_subject(InsertSubjectCommand {
            id: 701,
            uuid: "retrieval-contract-actor",
            tenant_id: TENANT_ID as i64,
            organization_id: None,
            subject_type: "user",
            subject_ref: &actor_ref,
            display_name: "Retrieval Contract Actor",
            default_space_id: Some(1),
            metadata_json: None,
        })
        .await
        .unwrap();
    store
        .insert_binding(
            801,
            "retrieval-contract-space-binding",
            TENANT_ID as i64,
            None,
            "access",
            "learner",
            Some(701),
            None,
            Some(2),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    let service = OpenMemoryService::new(store.clone());
    let context = context();
    for (space_id, text) in [
        (1, "revocation needle from owned space"),
        (2, "revocation needle from shared space"),
    ] {
        service
            .create_memory(
                context.clone(),
                memory_request(space_id, MemoryType::Semantic, text),
            )
            .await
            .unwrap();
    }

    let created = service
        .create_retrieval(
            context.clone(),
            retrieval_request("revocation needle", vec![1, 2], 5, None),
        )
        .await
        .unwrap();
    assert_eq!(created.hits.len(), 2);
    service
        .create_feedback(
            context.clone(),
            MemoryFeedbackRequest {
                target_type: "retrieval".to_string(),
                target_id: created.retrieval_id,
                feedback_type: "useful".to_string(),
                rating: Some(1),
                comment: None,
                metadata: None,
            },
        )
        .await
        .expect("retrieval feedback must resolve the trace through the typed trace port");

    assert!(store
        .delete_binding(TENANT_ID as i64, "retrieval-contract-space-binding")
        .await
        .unwrap());
    let retrieved = service
        .retrieve_retrieval(context, created.retrieval_id)
        .await
        .expect("revoked secondary-space hits must be filtered without failing the trace");
    assert_eq!(retrieved.hits.len(), 1);
    assert_eq!(retrieved.hits[0].memory.as_ref().unwrap().space_id, 1);
    assert_eq!(retrieved.hits[0].result_rank, 1);
    assert_eq!(retrieved.trace.as_ref().unwrap().result_count, 1);
}
