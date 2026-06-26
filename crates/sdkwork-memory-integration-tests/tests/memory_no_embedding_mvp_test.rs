use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_contract::{
    MemoryContextPackRequest, MemoryOpenApi, MemoryOpenApiRequestContext, MemoryRecordRequest,
    MemoryRetrievalRequest, MemoryType,
};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;

fn open_context() -> MemoryOpenApiRequestContext {
    MemoryOpenApiRequestContext::for_open_surface("api-key-001", 100_001, Some(2001))
}

#[tokio::test]
async fn remembers_retrieves_and_builds_context_without_embeddings() {
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let service = OpenMemoryService::new(store);
    let context = open_context();

    service
        .create_memory(
            context.clone(),
            MemoryRecordRequest {
                space_id: 2,
                scope: "user".to_string(),
                memory_type: MemoryType::Semantic,
                subject: None,
                predicate: None,
                object_text: Some("concise".to_string()),
                canonical_text: "User prefers concise answers".to_string(),
                summary_text: None,
                user_id: None,
                language: None,
                sensitivity_level: None,
                metadata: None,
                tags: None,
            },
        )
        .await
        .expect("create memory");

    let retrieval = service
        .create_retrieval(
            context.clone(),
            MemoryRetrievalRequest {
                query: "concise answers".to_string(),
                space_ids: vec![2],
                actor_id: None,
                retrieval_profile_id: None,
                memory_types: None,
                filters: None,
                top_k: 5,
                context_budget_tokens: 512,
                include_trace: None,
            },
        )
        .await
        .expect("retrieve");

    assert!(retrieval
        .hits
        .iter()
        .any(|hit| hit.retriever_name == "keyword"));
    assert!(!retrieval
        .hits
        .iter()
        .any(|hit| hit.retriever_name == "vector"));

    let pack = service
        .create_context_pack(
            context,
            MemoryContextPackRequest {
                query: "concise answers".to_string(),
                space_ids: vec![2],
                actor_id: None,
                retrieval_profile_id: None,
                context_budget_tokens: 512,
                include_citations: None,
                filters: None,
            },
        )
        .await
        .expect("context pack");

    let fragments = pack.pack["fragments"].as_array().expect("fragments");
    assert!(fragments.iter().any(|fragment| {
        fragment["canonicalText"]
            .as_str()
            .unwrap_or("")
            .contains("concise")
    }));
    assert_eq!(pack.pack["embeddingOptional"], true);
}
