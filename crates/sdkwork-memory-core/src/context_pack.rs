use sdkwork_memory_contract::MemoryRetrievalHit;
use serde_json::{json, Value};

pub fn estimate_tokens(text: &str) -> i32 {
    ((text.len() as f64) / 4.0).ceil() as i32
}

pub fn build_context_pack_from_hits(
    hits: &[MemoryRetrievalHit],
    budget_tokens: i32,
) -> (Value, i32, bool) {
    let mut fragments = Vec::new();
    let mut used_tokens = 0_i32;
    let mut truncated = false;

    for hit in hits {
        let Some(memory) = hit.memory.as_ref() else {
            continue;
        };
        let fragment_tokens = estimate_tokens(&memory.canonical_text);
        if used_tokens + fragment_tokens > budget_tokens && !fragments.is_empty() {
            truncated = true;
            break;
        }
        if fragment_tokens > budget_tokens && fragments.is_empty() {
            fragments.push(json!({
                "memoryId": memory.memory_id.to_string(),
                "canonicalText": memory.canonical_text,
                "retrieverName": hit.retriever_name,
                "rank": hit.result_rank,
            }));
            used_tokens = budget_tokens;
            truncated = true;
            break;
        }
        fragments.push(json!({
            "memoryId": memory.memory_id.to_string(),
            "canonicalText": memory.canonical_text,
            "retrieverName": hit.retriever_name,
            "rank": hit.result_rank,
        }));
        used_tokens += fragment_tokens;
    }

    let pack = json!({
        "fragments": fragments,
        "embeddingOptional": true,
    });
    (pack, used_tokens.max(1), truncated)
}
