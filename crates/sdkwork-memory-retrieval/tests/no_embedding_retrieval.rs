use sdkwork_memory_contract::{MemoryRecord, MemoryType};
use sdkwork_memory_retrieval::{fuse_retrieval_candidates, keyword_match_score, RetrievalCandidate};

fn sample_record(id: u64, text: &str) -> MemoryRecord {
    MemoryRecord {
        memory_id: id,
        uuid: Some(id.to_string()),
        space_id: 1,
        user_id: None,
        scope: "user".to_string(),
        memory_type: MemoryType::Semantic,
        subject: None,
        predicate: None,
        object_text: Some(text.to_string()),
        canonical_text: text.to_string(),
        summary_text: None,
        confidence: 1.0,
        evidence_count: Some(1),
        contradiction_count: Some(0),
        status: "active".to_string(),
        sensitivity_level: "internal".to_string(),
        supersedes_memory_id: None,
        superseded_by_memory_id: None,
        created_at: "2026-06-10T00:00:00Z".to_string(),
        updated_at: "2026-06-10T00:00:00Z".to_string(),
        version: 1,
    }
}

#[test]
fn keyword_match_scores_exact_and_partial_queries() {
    assert_eq!(keyword_match_score("renewal", "renewal"), 1.0);
    assert!(keyword_match_score("renewal support", "customer renewal support plan") > 0.8);
    assert_eq!(keyword_match_score("missing", "unrelated text"), 0.0);
}

#[test]
fn fuse_retrieval_candidates_ranks_by_score_without_embeddings() {
    let candidates = vec![
        RetrievalCandidate {
            memory: sample_record(1, "low relevance"),
            retriever_name: "keyword".to_string(),
            raw_score: 0.2,
            rank: 0,
        },
        RetrievalCandidate {
            memory: sample_record(2, "enterprise renewal support"),
            retriever_name: "keyword".to_string(),
            raw_score: 0.95,
            rank: 0,
        },
    ];

    let fused = fuse_retrieval_candidates(candidates, 2);
    assert_eq!(fused.len(), 2);
    assert_eq!(fused[0].memory.memory_id, 2);
    assert_eq!(fused[0].retriever_name, "keyword");
}
