use sdkwork_memory_contract::MemoryRecord;

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalCandidate {
    pub memory: MemoryRecord,
    pub retriever_name: String,
    pub raw_score: f64,
    pub rank: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FusedRetrievalHit {
    pub memory: MemoryRecord,
    pub retriever_name: String,
    pub raw_score: f64,
    pub fused_score: f64,
    pub rank: i32,
}

pub fn fuse_retrieval_candidates(
    candidates: Vec<RetrievalCandidate>,
    top_k: usize,
) -> Vec<FusedRetrievalHit> {
    let mut sorted = candidates;
    sorted.sort_by(|left, right| {
        right
            .raw_score
            .partial_cmp(&left.raw_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.memory.memory_id.cmp(&right.memory.memory_id))
    });

    sorted
        .into_iter()
        .take(top_k)
        .enumerate()
        .map(|(index, candidate)| FusedRetrievalHit {
            memory: candidate.memory,
            retriever_name: candidate.retriever_name,
            raw_score: candidate.raw_score,
            fused_score: candidate.raw_score,
            rank: (index as i32) + 1,
        })
        .collect()
}

pub fn keyword_match_score(query: &str, canonical_text: &str) -> f64 {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return 0.0;
    }

    let haystack = canonical_text.to_lowercase();
    if haystack == query {
        return 1.0;
    }
    if haystack.contains(&query) {
        return 0.85;
    }

    let tokens: Vec<&str> = query.split_whitespace().collect();
    if tokens.is_empty() {
        return 0.0;
    }

    let matched = tokens
        .iter()
        .filter(|token| haystack.contains(**token))
        .count();
    matched as f64 / tokens.len() as f64
}
